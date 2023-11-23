/**
 * 图片渲染对象的构建及其属性设置
	*/
use std::marker::PhantomData;

use pi_style::style::ImageRepeatOption;
use share::Share;
use std::hash::{Hash, Hasher};

// use ordered_float::NotNan;
use hash::DefaultHasher;

use pi_atom::Atom;
use ecs::monitor::{Event, NotifyImpl};
use ecs::{DeleteEvent, EntityListener, MultiCaseImpl, Runner, SingleCaseImpl};
use hal_core::*;
use map::vecmap::VecMap;
use pi_polygon::*;

use crate::component::calc::LayoutR;
use crate::component::calc::*;
use crate::component::user::*;
use crate::entity::Node;
use crate::render::engine::{AttributeDecs, Engine, ShareEngine};
use crate::render::res::Opacity as ROpacity;
use crate::render::res::*;
use crate::single::*;
use crate::system::render::shaders::image::{CANVAS_FS_SHADER_NAME, CANVAS_VS_SHADER_NAME, IMAGE_FS_SHADER_NAME, IMAGE_VS_SHADER_NAME};
use crate::system::util::constant::*;
use crate::system::util::*;

use super::push_quad;

lazy_static! {
    static ref UV: Atom = Atom::from("UV");
    static ref POSITION: Atom = Atom::from("Position");
    static ref INDEX: Atom = Atom::from("Index");

	static ref DIRTY_TY: StyleBit = style_bit().set_bit(StyleType::BorderRadius as usize)
		.set_bit(StyleType::Opacity as usize)
		.set_bit(StyleType::BackgroundImageClip as usize)
		.set_bit(StyleType::ObjectFit as usize);

	static ref GEO_DIRTY: StyleBit =
		style_bit().set_bit(StyleType::BorderRadius as usize) .set_bit(StyleType::BackgroundImageClip as usize) .set_bit(StyleType::ObjectFit as usize);
}

const DIRTY_TY1: usize = CalcType::BackgroundImageTexture as usize;



pub struct ImageSys<C> {
    render_map: VecMap<usize>,
    default_sampler: Share<SamplerRes>,
    unit_geo: Share<GeometryRes>, // 含uv， index， pos
    default_paramter: ImageParamter,
    marker: PhantomData<C>,
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for ImageSys<C> {
    type ReadData = (
        &'a MultiCaseImpl<Node, LayoutR>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZRange>,
        &'a MultiCaseImpl<Node, ImageTexture>,
		&'a MultiCaseImpl<Node, BackgroundImage>,
        &'a MultiCaseImpl<Node, BackgroundImageClip>,
        &'a MultiCaseImpl<Node, BackgroundImageMod>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a SingleCaseImpl<DirtyList>,
        &'a SingleCaseImpl<DefaultState>,
        &'a SingleCaseImpl<PremultiState>,
		&'a SingleCaseImpl<VertType>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<ShareEngine<C>>);

    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        let (
            layouts,
            border_radiuss,
            z_depths,
            image_textures,
			images,
            image_clips,
            object_fits,
            world_matrixs,
            transforms,
            style_marks,
            dirty_list,
            default_state,
            premulti_state,
			single_vert_type,
        ) = read;
        if dirty_list.0.len() == 0 {
            return;
        }

        let (render_objs, engine) = write;
        let notify = unsafe { &*(render_objs.get_notify_ref() as *const NotifyImpl) };

        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                }
            };

            let mut dirty = style_mark.dirty;
            let mut dirty1 = style_mark.dirty1;

            // 不存在Image关心的脏, 跳过
            if (!(dirty & &*DIRTY_TY).any()) && (dirty1 & DIRTY_TY1 == 0) && (dirty1 & GEO_DIRTY_TYPE == 0) {
                continue;
            }

			// 如果不存在Texture， 删除渲染对象
            let texture = match image_textures.get(*id) {
                Some(r) => r,
                None => {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                }
            };
			
			if let ImageTexture::All(_, url ) = texture {
				if url.as_str() != "" {// 
					let image = match images.get(*id) {
						Some(r) => r,
						None => continue,
					};
					// 如果纹理和image的url不相等， 什么也不做，保持现在的渲染状态
					// 新的图片可能处于未加载完成状态，首先，不应该删除原有的渲染对象， 如果是帧动画，会造成闪烁
					// 另外，也不应更新当前设置的最新数据， 可能造成image_clip和image本身不匹配，计算出错误的uv，也会造成闪烁
					if image.0 != *url {
						// if *url == 1196902338 || *url == 1483981615 {
						// 	log::info!("!!!!!image==============={:?}, image_clip: {:?}, url: {:?}", images.get(*id), image_clips.get(*id), url);
						// }
						continue;
					}
				}
				// if *url == 1196902338 || *url == 1483981615 {
				// 	log::info!("image==============={:?}, image_clip: {:?}, url: {:?}", images.get(*id), image_clips.get(*id), url);
				// }
			}
			
			let border_radius = border_radiuss.get(*id);
            let render_index = if dirty1 & DIRTY_TY1 != 0 {
                dirty |= DIRTY_TY.clone(); // Image脏， 所有属性重新设置
				dirty1 |= GEO_DIRTY_TYPE;
                match self.render_map.get_mut(*id) {
                    Some(r) => *r,
                    None => {
                        let (state, vs, fs) = {
                            match texture {
                                ImageTexture::Part(_r) => (&***premulti_state, CANVAS_VS_SHADER_NAME.clone(), CANVAS_FS_SHADER_NAME.clone()),
                                ImageTexture::All(_r, _) => (&***default_state, IMAGE_VS_SHADER_NAME.clone(), IMAGE_FS_SHADER_NAME.clone()),
                            }
                        };
                        self.create_render_obj(*id, render_objs, state, vs, fs)
                    }
                }
            } else {
                match self.render_map.get_mut(*id) {
                    Some(r) => *r,
                    None => continue,
                }
            };

            let render_obj = &mut render_objs[render_index];
			let vert_type = render_obj.vert_type;

            let z_depth = z_depths[*id].start as f32;
            let layout = &layouts[*id];

            let image_clip = image_clips.get(*id);
            let object_fit = object_fits.get(*id);
            let transform = &transforms[*id];
            let world_matrix = &world_matrixs[*id];
            let object_fit = match object_fit {
                Some(r) => r.clone(),
                None => BackgroundImageMod::default(),
            };

            if (dirty & &*GEO_DIRTY).any() || dirty1 & CalcType::Layout as usize != 0 {
                // log::info!("uv1============{:?}", id);
                let vert_type = update_geo(
                    render_obj,
                    layout,
                    texture,
                    image_clip,
                    &object_fit,
                    engine,
                    &self.unit_geo,
                );

				if render_obj.vert_type != vert_type {
					render_obj.vert_type = vert_type;
					single_vert_type.get_notify_ref().modify_event(render_index, "", 0);
				}
                modify_matrix(render_obj, layout, z_depth, world_matrix, transform, vert_type);

                notify.modify_event(render_index, "geometry", 0);
                notify.modify_event(render_index, "ubo", 0);
                dirty1 &= !(CalcType::Matrix as usize); // 已经计算了世界矩阵， 设置世界矩阵不脏
            }

            // 世界矩阵脏， 设置世界矩阵ubo
            if dirty1 & CalcType::Matrix as usize != 0 {
                modify_matrix(render_obj, layout, z_depth, world_matrix, transform, vert_type);
                notify.modify_event(render_index, "ubo", 0);
            }

            // 不透明度脏或图片脏， 设置is_opacity
            if dirty[StyleType::Opacity as usize] || dirty1 & DIRTY_TY1 != 0 {
                // let opacity = opacitys[*id].0;
                let dyn_texture_set;
                let tex = match texture {
                    ImageTexture::All(r, _) => &r,
                    ImageTexture::Part(r) => {
                        let index = r.index();
                        dyn_texture_set = r.get_dyn_texture_set().borrow();
                        dyn_texture_set.get_texture(index).unwrap()
                    }
                };
                // 如果四边形与图片宽高一样， 使用点采样?， TODO
                if dirty1 & DIRTY_TY1 != 0 {
                    render_obj.paramter.set_texture("texture", (&tex.bind, &self.default_sampler));
                    notify.modify_event(render_index, "ubo", 0);
                }

                let is_opacity = if let ROpacity::Opaque = tex.opacity { true && border_radius.is_none() } else { false };

                if render_obj.is_opacity != is_opacity {
                    render_obj.is_opacity = is_opacity;
                    notify.modify_event(render_index, "is_opacity", 0);
                    modify_opacity(
                        engine,
                        render_obj,
                        match texture {
                            ImageTexture::All(_r, _) => premulti_state,
                            ImageTexture::Part(_r) => default_state,
                        },
                    );
                }
            }
            notify.modify_event(render_index, "", 0);
        }
    }
}

// 监听实体销毁，删除索引
impl<'a, C: HalContext + 'static> EntityListener<'a, Node, DeleteEvent> for ImageSys<C> {
    type ReadData = ();
    type WriteData = ();

    fn listen(&mut self, event: &Event, _read: Self::ReadData, _: Self::WriteData) {
        self.render_map.remove(event.id); // 移除索引
    }
}

impl<C: HalContext + 'static> ImageSys<C> {
    pub fn with_capacity(engine: &mut Engine<C>, capacity: usize) -> Self {
        let mut sm = SamplerDesc::default();
        sm.u_wrap = TextureWrapMode::ClampToEdge;
        sm.v_wrap = TextureWrapMode::ClampToEdge;
        // sm.min_filter = TextureFilterMode::Nearest;
        // sm.mag_filter = TextureFilterMode::Nearest;

        let default_sampler = engine.create_sampler_res(sm);

        let positions = engine.buffer_res_map.get(&(POSITIONUNIT.get_hash() as u64)).unwrap();
        let indices = engine.buffer_res_map.get(&(INDEXUNIT.get_hash() as u64)).unwrap();

        let geo = engine.create_geometry();
        engine.gl.geometry_set_attribute(&geo, &AttributeName::Position, &positions, 2).unwrap();
        engine.gl.geometry_set_attribute(&geo, &AttributeName::UV0, &positions, 2).unwrap();
        engine.gl.geometry_set_indices_short(&geo, &indices).unwrap();

        ImageSys {
            render_map: VecMap::with_capacity(capacity),
            default_sampler: default_sampler,
            unit_geo: Share::new(GeometryRes {
                geo: geo,
                buffers: vec![indices, positions.clone(), positions],
            }),
            default_paramter: ImageParamter::default(),
            marker: PhantomData,
        }
    }

    #[inline]
    fn remove_render_obj(&mut self, id: usize, render_objs: &mut SingleCaseImpl<RenderObjs>) {
        match self.render_map.remove(id) {
            Some(index) => {
                let notify = unsafe { &*(render_objs.get_notify_ref() as *const NotifyImpl) };
                render_objs.remove(index, Some(notify));
            }
            None => (),
        };
    }

    #[inline]
    fn create_render_obj(
        &mut self,
        id: usize,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &CommonState,
        vs: Atom,
        fs: Atom,
    ) -> usize {
        create_render_obj(
            id,
            -0.1,
            true,
            vs,
            fs,
            Share::new(self.default_paramter.clone()),
            default_state,
            render_objs,
            &mut self.render_map,
        )
    }
}

// 获得图片的4个点(逆时针)的坐标和uv的Aabb
fn get_pos_uv(texture: &ImageTexture, clip: Option<&BackgroundImageClip>, fit: &BackgroundImageMod, layout: &LayoutR) -> (Aabb2, Aabb2, Vector2, bool) {
    let mut p1 = Point2::new(layout.border.left + layout.padding.left, layout.border.top + layout.padding.top);
    let width = layout.rect.right - layout.rect.left - layout.border.right - layout.border.left;
    let height = layout.rect.bottom - layout.rect.top - layout.border.bottom - layout.border.top;
    let mut p2 = Point2::new(width, height);

    match texture {
        ImageTexture::All(src, _) => {
            let (size, mut uv1, mut uv2) = match clip {
                Some(c) => {
                    let size = Vector2::new(
                        src.width as f32 * (c.right - c.left).abs(),
                        src.height as f32 * (c.bottom - c.top).abs(),
                    );
                    // log::info!("size================={:?}");
                    (size, Point2::new(*c.left, *c.top), Point2::new(*c.right, *c.bottom))
                }
                _ => (
                    Vector2::new(src.width as f32, src.height as f32),
                    Point2::new(0.0, 0.0),
                    Point2::new(1.0, 1.0),
                ),
            };

            let w = p2.x - p1.x;
            let h = p2.y - p1.y;
            // 如果不是填充，总是居中显示。 如果在范围内，则修改点坐标。如果超出的部分，会进行剪切，剪切会修改uv坐标。
            match fit.object_fit {
                FitType::None => {
                    // 保持原有尺寸比例。同时保持内容原始尺寸大小。 超出部分会被剪切
                    if size.x <= w {
                        let x = (w - size.x) / 2.0;
                        p1.x += x;
                        p2.x -= x;
                    } else {
                        let x = (size.x - w) * (uv2.x - uv1.x) * 0.5 / size.x;
                        uv1.x += x;
                        uv2.x -= x;
                    }
                    if size.y <= h {
                        let y = (h - size.y) / 2.0;
                        p1.y += y;
                        p2.y -= y;
                    } else {
                        let y = (size.y - h) * (uv2.y - uv1.y) * 0.5 / size.y;
                        uv1.y += y;
                        uv2.y -= y;
                    }
                }
                FitType::Contain => {
                    // 保持原有尺寸比例。保证内容尺寸一定可以在容器里面放得下。因此，此参数可能会在容器内留下空白。
                    fill(&size, &mut p1, &mut p2, w, h);
                }
                FitType::Cover => {
                    // 保持原有尺寸比例。保证内容尺寸一定大于容器尺寸，宽度和高度至少有一个和容器一致。超出部分会被剪切
                    if w != 0.0 && h != 0.0 {
                        let rw = size.x / w;
                        let rh = size.y / h;

                        if rw > rh {
                            let x = (size.x - w * rh) * (uv2.x - uv1.x) * 0.5 / size.x;
                            uv1.x += x;
                            uv2.x -= x;
                        } else {
                            let y = (size.y - h * rw) * (uv2.y - uv1.y) * 0.5 / size.y;
                            uv1.y += y;
                            uv2.y -= y;
                        }
                    }
                }
                FitType::ScaleDown => {
                    // 如果内容尺寸小于容器尺寸，则直接显示None。否则就是Contain
                    if size.x <= w && size.y <= h {
                        let x = (w - size.x) / 2.0;
                        let y = (h - size.y) / 2.0;
                        p1.x += x;
                        p1.y += y;
                        p2.x -= x;
                        p2.y -= y;
                    } else {
                        fill(&size, &mut p1, &mut p2, w, h);
                    }
                }
                // FitType::Repeat => panic!("TODO"),  // TODO
                // FitType::RepeatX => panic!("TODO"), // TODO
                // FitType::RepeatY => panic!("TODO"), // TODO
                FitType::Fill => (), // 填充。 内容拉伸填满整个容器，不保证保持原有的比例
            };
            (Aabb2::new(p1, p2), Aabb2::new(uv1, uv2), size, false)
        }
        ImageTexture::Part(r) => {
			let uv = r.get_uv();
			let size = r.size();
			let texture_size = Vector2::new((size.0 as f32) * (uv.maxs.x - uv.mins.x), (size.1 as f32) * (uv.maxs.y - uv.mins.y));
			(Aabb2::new(p1, p2), r.get_uv(), texture_size, true)
		},
    }
}

#[inline]
fn update_geo<C: HalContext + 'static>(
    render_obj: &mut RenderObj,
    layout: &LayoutR,
    texture: &ImageTexture,
    image_clip: Option<&BackgroundImageClip>,
    background_image_mod: &BackgroundImageMod,
    engine: &mut Engine<C>,
    unit_geo: &Share<GeometryRes>,
) -> VertType {
    let (pos, uv, texture_size, is_part) = get_pos_uv(texture, image_clip, background_image_mod, layout);
	if (background_image_mod.object_fit == FitType::Fill || background_image_mod.object_fit == FitType::Cover) 
		&& background_image_mod.repeat.x == ImageRepeatOption::Stretch
		&& layout.border.left == 0.0 
		&& layout.border.right == 0.0
		&& layout.border.top == 0.0 
		&& layout.border.bottom == 0.0
		&& background_image_mod.repeat.y == ImageRepeatOption::Stretch {
		update_geo_quad(render_obj, &uv, image_clip, engine, unit_geo, is_part); // 单位四边形
		VertType::ContentRect
	} else {
		let hash = calc_hash(&("image geo", background_image_mod), calc_float_hash(&[layout.rect.top, layout.rect.right, layout.rect.bottom, layout.rect.left], 0));
		match engine.geometry_res_map.get(&hash) {
			Some(r) => render_obj.geometry = Some(r.clone()),
			None => {
				let (positions, uvs, indices) = get_pos_uv_buffer(&pos, &uv, texture_size, background_image_mod);
				render_obj.geometry = Some(engine.create_geo_res(
					0,
					indices.as_slice(),
					&[
						AttributeDecs::new(AttributeName::Position, positions.as_slice(), 2),
						AttributeDecs::new(AttributeName::UV0, uvs.as_slice(), 2),
					],
				));
			}
		}
		VertType::ContentNone
	}
}

fn get_pos_uv_buffer(pos: &Aabb2, clip: &Aabb2, texture_size: Vector2, image_mod: &BackgroundImageMod) -> (Vec<f32>, Vec<f32>, Vec<u16>) {
    let (p1, p2) = (&pos.mins, &pos.maxs);
	let (uv1, uv2) = (&clip.mins, &clip.maxs);
	let w = p2.x - p1.x;
	let h = p2.y - p1.y;

	let (uoffset, uspace, ustep) = calc_step(w, texture_size.x, image_mod.repeat.x);
	let (voffset, vspace, vstep) = calc_step(h, texture_size.y, image_mod.repeat.y);

	let mut vert_arr = Vec::default();
	let mut uv_arr = Vec::default();
	let mut index_arr = Vec::default();
    let mut index = index_arr.len() as u16;

    let (mut cur_y, mut next_y) = (p1.y, p1.y + vstep);
    let mut v2 = uv2.y;

	// 第一个四边形的u2
    let mut u2 = uv2.x;
    if uoffset > 0.0 {
        u2 = uv1.x + uoffset / ustep * (uv2.x - uv1.x);
    }

    let mut u_end = pos.maxs.x;
	let mut v_end = pos.maxs.y;
    if uspace > 0.0 && w < ustep * 2.0 {
        u_end = (pos.maxs.x - uspace).min(u_end);
    }
    if vspace > 0.0 && h < vstep * 2.0 {
        v_end = (pos.maxs.y - vspace).min(v_end);
    }

    loop {
        if next_y > v_end {
            next_y = v_end;
            v2 = uv1.y + voffset / vstep * (uv2.y - uv1.y);
        }

        let p_left_top = push_vertex(&mut vert_arr, p1.x, cur_y, &mut index);
        let p_right_top = push_vertex(&mut vert_arr, u_end, cur_y, &mut index);
        uv_arr.extend_from_slice(&[uv1.x, uv1.y]);
        uv_arr.extend_from_slice(&[u2, uv1.y]);

        let p_left_bootom = push_vertex(&mut vert_arr, p1.x, next_y, &mut index);
        let p_right_bottom = push_vertex(&mut vert_arr, u_end, next_y, &mut index);
        uv_arr.extend_from_slice(&[uv1.x, v2]);
        uv_arr.extend_from_slice(&[u2, v2]);

        push_u_arr(
            &mut vert_arr,
            &mut uv_arr,
            &mut index_arr,
            p_left_top,
            p_left_bootom,
            p_right_bottom,
            p_right_top,
            uv1.x,
            uv1.y,
            uv2.x,
            v2,
            ustep,
            uspace,
            &mut index,
        ); // 上边
        if next_y > v_end || eq_f32(next_y, v_end) {
            break;
        }

        cur_y = next_y + vspace;
        next_y = cur_y + vstep;
    }

    return (vert_arr, uv_arr, index_arr);
}

fn update_geo_quad<C: HalContext + 'static>(
    render_obj: &mut RenderObj,
    uv: &Aabb2,
    image_clip: Option<&BackgroundImageClip>,
    engine: &mut Engine<C>,
    unit_geo: &Share<GeometryRes>,
    is_part: bool,
) {
    match (image_clip, is_part) {
        (None, false) => render_obj.geometry = Some(unit_geo.clone()),
        _ => {
            let (uv1, uv2) = (uv.mins, uv.maxs);
            // log::info!("clip===={:?}, {:?}, {:?}, {:?}", _clip, &uv1, &uv2, flip_y);
            let uv_hash = cal_uv_hash(&uv1, &uv2);
            let geo_hash = unit_geo_hash(&uv_hash);
            match engine.geometry_res_map.get(&geo_hash) {
                Some(r) => render_obj.geometry = Some(r),
                None => {
                    let uv_buffer = create_uv_buffer(uv_hash, &uv1, &uv2, engine);
                    let geo = engine.create_geometry();
                    engine
                        .gl
                        .geometry_set_attribute(&geo, &AttributeName::Position, &unit_geo.buffers[1], 2)
                        .unwrap();
                    engine.gl.geometry_set_attribute(&geo, &AttributeName::UV0, &uv_buffer, 2).unwrap();
                    engine.gl.geometry_set_indices_short(&geo, &unit_geo.buffers[0]).unwrap();
                    let geo_res = GeometryRes {
                        geo: geo,
                        buffers: vec![unit_geo.buffers[0].clone(), unit_geo.buffers[1].clone(), uv_buffer],
                    };
                    render_obj.geometry = Some(engine.geometry_res_map.create(geo_hash, geo_res, 0, 0));
                }
            };
        }
    }

    // 修改世界矩阵 TODO
}

#[inline]
fn cal_uv_hash(uv1: &Point2, uv2: &Point2) -> u64 {
    let mut hasher = DefaultHasher::default();
    UV.hash(&mut hasher);
    f32_4_hash_(uv1.x, uv1.y, uv2.x, uv2.y, &mut hasher);
    hasher.finish()
}

fn create_uv_buffer<C: HalContext + 'static>(uv_hash: u64, uv1: &Point2, uv2: &Point2, engine: &mut Engine<C>) -> Share<BufferRes> {
    match engine.buffer_res_map.get(&uv_hash) {
        Some(r) => r,
        None => {
            let uvs = [uv1.x, uv1.y, uv1.x, uv2.y, uv2.x, uv2.y, uv2.x, uv1.y];
            engine.create_buffer_res(uv_hash, BufferType::Attribute, 8, Some(BufferData::Float(&uvs[..])), false)
        }
    }
}

#[inline]
fn unit_geo_hash(uv_hash: &u64) -> u64 {
    let mut hasher = DefaultHasher::default();
    uv_hash.hash(&mut hasher);
    POSITIONUNIT.hash(&mut hasher);
    INDEXUNIT.hash(&mut hasher);
    hasher.finish()
}

#[inline]
fn modify_matrix(
    render_obj: &mut RenderObj,
    layout: &LayoutR,
    depth: f32,
    world_matrix: &WorldMatrix,
    transform: &Transform,
    vert_type: VertType,
) {
    if let VertType::ContentNone = vert_type {
        let arr = create_let_top_offset_matrix(layout, world_matrix, transform, layout.border.left, layout.border.top);
        render_obj
            .paramter
            .set_value("worldMatrix", Share::new(WorldMatrixUbo::new(UniformValue::MatrixV4(arr))));
    } else {
        let arr = create_unit_offset_matrix(
            layout.rect.right - layout.rect.left - layout.border.left -  layout.border.right,
            layout.rect.bottom - layout.rect.top - layout.border.top -  layout.border.bottom,
            layout.border.left,
            layout.border.top,
            layout,
            world_matrix,
            transform,
            depth,
        );
        render_obj
            .paramter
            .set_value("worldMatrix", Share::new(WorldMatrixUbo::new(UniformValue::MatrixV4(arr))));
    }
}

fn use_layout_pos<C: HalContext + 'static>(render_obj: &mut RenderObj, uv: Aabb2, layout: &LayoutR, radius: &Point2, engine: &mut Engine<C>) {
    let width = layout.rect.right - layout.rect.left;
    let height = layout.rect.bottom - layout.rect.top;
    let start_x = layout.border.left;
    let start_y = layout.border.top;
    let end_x = width - layout.border.right;
    let end_y = height - layout.border.bottom;
    let (positions, indices) = if radius.x == 0.0 || width == 0.0 || height == 0.0 {
        (vec![start_x, start_y, start_x, end_y, end_x, end_y, end_x, start_y], vec![0, 1, 2, 3])
    } else {
        split_by_radius(start_x, start_y, end_x - start_x, end_y - start_y, radius.x - start_x, None)
    };
    // debug_println!("indices: {:?}", indices);
    // debug_println!("split_by_lg,  positions:{:?}, indices:{:?}, top_percent: {}, bottom_percent: {}, start: ({}, {}) , end: ({}, {})", positions, indices, 0.0, 1.0, 0.0, 0.0, 0.0, layout.height);
    let (positions, indices_arr) = split_by_lg(positions, indices, &[0.0, 1.0], (0.0, 0.0), (0.0, height));
    // debug_println!("split_mult_by_lg, positions: {:?}, indices_arr: {:?}, cfg: {:?}, percent: [{}, {}], start: [{}, {}], end: [{}, {}]",  &positions, indices_arr, vec![LgCfg{unit: 1, data: vec![uv.min.x, uv.max.x]}], 0.0, 1.0, 0.0, 0.0, layout.width, 0.0);
    let (positions, indices_arr) = split_mult_by_lg(positions, indices_arr, &[0.0, 1.0], (0.0, 0.0), (width, 0.0));
    let indices = mult_to_triangle(&indices_arr, Vec::new());
    // debug_println!("u positions: {:?}, indices_arr: {:?}, cfg: {:?}, percent: [{}, {}], start: [{}, {}], end: [{}, {}]",  &positions, indices_arr, vec![LgCfg{unit: 1, data: vec![uv.min.x, uv.max.x]}], 0.0, 1.0, 0.0, 0.0, layout.width, 0.0);
    let u = interp_mult_by_lg(
        &positions,
        &indices_arr,
        vec![Vec::new()],
        vec![LgCfg {
            unit: 1,
            data: vec![uv.mins.x, uv.maxs.x],
        }],
        &[0.0, 1.0],
        (0.0, 0.0),
        (width, 0.0),
    );
    let v = interp_mult_by_lg(
        &positions,
        &indices_arr,
        vec![Vec::new()],
        vec![LgCfg {
            unit: 1,
            data: vec![uv.mins.y, uv.maxs.y],
        }],
        &[0.0, 1.0],
        (0.0, 0.0),
        (0.0, height),
    );
    // debug_println!("v positions: {:?}, indices_arr: {:?}, cfg: {:?}, percent: [}, {}], start: [{}, {}], end: [{}, {}]",  &positions, indices_arr, vec![LgCfg{unit: 1, data: vec![uv.min.y, uv.max.y]}], 0.0, 1.0, 0.0, 0.0, 0.0, layout.height);
    let mut uvs = Vec::with_capacity(u[0].len());
    for i in 0..u[0].len() {
        uvs.push(u[0][i]);
        uvs.push(v[0][i]);
    }

    render_obj.geometry = Some(engine.create_geo_res(
        0,
        indices.as_slice(),
        &[
            AttributeDecs::new(AttributeName::Position, positions.as_slice(), 2),
            AttributeDecs::new(AttributeName::UV0, uvs.as_slice(), 2),
        ],
    ));
}


#[inline]
pub fn push_vertex(point_arr: &mut Vec<f32>, x: f32, y: f32, i: &mut u16) -> u16 {
    point_arr.extend_from_slice(&[x, y]);
    let r = *i;
    *i += 1;
    r
}

pub fn calc_step(csize: f32, img_size: f32, rtype: ImageRepeatOption) -> (f32, f32, f32) {
    if let ImageRepeatOption::Stretch = rtype {
        return (0.0, 0.0, csize);
    }
    if img_size == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let c = csize / img_size;
    let f = c.round();
    if eq_f32(f, c) {
        // 整数倍的情况（这里消除了浮点误差，大致为整数倍，都认为是整数倍）
        return (0.0, 0.0, img_size);
    }

    match rtype {
        ImageRepeatOption::Repeat => (csize % img_size, 0.0, img_size),
        ImageRepeatOption::Round => (0.0, 0.0, if c > 1.0 { csize / c.round() } else { csize }),
        ImageRepeatOption::Space => {
            let space = csize % img_size; // 空白尺寸
            let pre_space = if c > 2.0 { space / (c.floor() - 1.0) } else { space };
            (0.0, pre_space, img_size)
        }
        _ => (0.0, 0.0, csize),
    }
}

pub fn push_u_arr(
    point_arr: &mut Vec<f32>,
    uv_arr: &mut Vec<f32>,
    index_arr: &mut Vec<u16>,
    p1: u16,
    p2: u16,
    p3: u16,
    p4: u16,
    u1: f32,
    v1: f32,
    u2: f32,
    v2: f32,
    step: f32,
    space: f32,
    i: &mut u16,
) {
    let y1 = point_arr[p1 as usize * 2 + 1];
    let y2 = point_arr[p2 as usize * 2 + 1];
    let mut cur = point_arr[p1 as usize * 2] + step;
    let max = point_arr[p3 as usize * 2];

    let mut pt1 = p1;
    let mut pt2 = p2;
    while !(cur > max || eq_f32(max, cur)) {
        let i3 = push_vertex(point_arr, cur, y2, i);
        let i4 = push_vertex(point_arr, cur, y1, i);
        uv_arr.extend_from_slice(&[u2, v2]);
        uv_arr.extend_from_slice(&[u2, v1]);
        push_quad(index_arr, pt1, pt2, i3, i4);
        // 因为uv不同，新插入2个顶点
        cur += space;
        // if cur
        pt1 = push_vertex(point_arr, cur, y1, i);
        pt2 = push_vertex(point_arr, cur, y2, i);
        uv_arr.extend_from_slice(&[u1, v1]);
        uv_arr.extend_from_slice(&[u1, v2]);
        cur += step;
    }
    push_quad(index_arr, pt1, pt2, p3, p4);
}
// 按比例缩放到容器大小，居中显示
fn fill(size: &Vector2, p1: &mut Point2, p2: &mut Point2, w: f32, h: f32) {
    if w != 0.0 && h != 0.0 {
        let rw = size.x / w;
        let rh = size.y / h;
        if rw > rh {
            let y = (h - size.y / rw) / 2.0;
            p1.y += y;
            p2.y -= y;
        } else {
            let x = (w - size.x / rh) / 2.0;
            p1.x += x;
            p2.x -= x;
        }
    }
}

impl_system! {
    ImageSys<C> where [C: HalContext + 'static],
    true,
    {
        EntityListener<Node, DeleteEvent>
    }
}
