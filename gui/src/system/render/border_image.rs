/**
 * 边框图片渲染对象的构建及其属性设置
 */
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use hash::DefaultHasher;

use pi_assets::asset::Handle;
use pi_atom::Atom;
use ecs::monitor::{Event, NotifyImpl};
use ecs::{DeleteEvent, EntityListener, MultiCaseImpl, Runner, SingleCaseImpl};

use hal_core::*;
use map::vecmap::VecMap;
use pi_style::style::ImageRepeatOption;
use pi_share::Share;

use crate::component::calc::LayoutR;
use crate::component::calc::*;
use crate::component::user::*;
use crate::entity::Node;
use crate::render::engine::{AttributeDecs, Engine, ResWrapper, ShareEngine};
use crate::render::res::{GeometryRes, OpacityType as ROpacity, SamplerRes};
use crate::single::*;
use crate::system::render::shaders::image::{IMAGE_FS_SHADER_NAME, IMAGE_VS_SHADER_NAME};
use crate::system::util::*;



lazy_static! {
    static ref BORDER_IMAGE: Atom = Atom::from("border_image");

	// 本系统关心的脏
	static ref DIRTY_TY: StyleBit = style_bit().set_bit(StyleType::Opacity as usize)
	.set_bit(StyleType::BorderImageClip as usize)
	.set_bit(StyleType::BorderImageSlice as usize)
	.set_bit(StyleType::BorderImageRepeat as usize);

	// 一些与BorderImage渲染对象的几何体相关的属性脏
	static ref GEO_DIRTY: StyleBit = style_bit().set_bit(StyleType::BorderImageClip as usize) .set_bit(StyleType::BorderImageSlice as usize) .set_bit(StyleType::BorderImageRepeat as usize);
}

const DIRTY_TY1: usize = CalcType::BorderImageTexture as usize | GEO_DIRTY_TYPE;

pub struct BorderImageSys<C: HalContext + 'static> {
    render_map: VecMap<usize>,
    default_sampler: Handle<SamplerRes>,
    default_paramter: ImageParamter,
    marker: PhantomData<C>,
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for BorderImageSys<C> {
    type ReadData = (
        &'a MultiCaseImpl<Node, BorderImage>,
        &'a MultiCaseImpl<Node, BorderImageTexture>,
        &'a MultiCaseImpl<Node, BorderImageClip>,
        &'a MultiCaseImpl<Node, BorderImageSlice>,
        &'a MultiCaseImpl<Node, BorderImageRepeat>,
        &'a MultiCaseImpl<Node, LayoutR>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a SingleCaseImpl<DirtyList>,
        &'a SingleCaseImpl<DefaultState>,
		&'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, IsLeaf>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<ShareEngine<C>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        if (read.10).0.len() == 0 {
            return;
        }

        let (
            border_images,
            border_image_textures,
            border_image_clips,
            border_image_slices,
            border_image_repeats,
            layouts,
            world_matrixs,
            transforms,
            opacitys,
            style_marks,
            dirty_list,
            default_state,
			border_radiuses,
            is_leaf,
        ) = read;
        let (render_objs, mut engine) = write;

        let default_transform = Transform::default();
        let notify = unsafe { &*(render_objs.get_notify_ref() as *const NotifyImpl) };
        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                }
            };

            // 不存在Image关心的脏, 跳过
            if !(style_mark.dirty & &*DIRTY_TY).any() && style_mark.dirty1 & DIRTY_TY1 == 0 {
                continue;
            }

            let mut dirty = style_mark.dirty;
            let mut dirty1 = style_mark.dirty1;

            let texture = match border_image_textures.get(*id) {
                Some(r) => r,
                None => {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                }
            };

            if texture.1.as_str() != "" {// 
                let image = match border_images.get(*id) {
                    Some(r) => r,
                    None => continue,
                };
                // 如果纹理和image的url不相等， 什么也不做，保持现在的渲染状态
                // 新的图片可能处于未加载完成状态，首先，不应该删除原有的渲染对象， 如果是帧动画，会造成闪烁
                // 另外，也不应更新当前设置的最新数据， 可能造成image_clip和image本身不匹配，计算出错误的uv，也会造成闪烁
                if image.0 != texture.1 {
                    // if *url == 1196902338 || *url == 1483981615 {
                    // 	log::info!("!!!!!image==============={:?}, image_clip: {:?}, url: {:?}", images.get(*id), image_clips.get(*id), url);
                    // }
                    continue;
                }
            }
             let alpha = if is_leaf[*id].0 {
                opacitys[*id].0
            } else {
                1.0
            };
			
            // BorderImage脏， 如果不存在BorderImage的本地样式和class样式， 删除渲染对象
            let render_index = if dirty1 & DIRTY_TY1 != 0 {
                dirty |= DIRTY_TY.clone();
				dirty1 |= DIRTY_TY1;
                match self.render_map.get_mut(*id) {
                    Some(r) => *r,
                    None => self.create_render_obj(*id, render_objs, default_state, alpha),
                }
            } else {
                match self.render_map.get_mut(*id) {
                    Some(r) => *r,
                    None => continue,
                }
            };

            let render_obj = &mut render_objs[render_index];
            let layout = &layouts[*id];

            // 世界矩阵脏， 设置世界矩阵ubo
            if dirty1 & CalcType::Matrix as usize != 0 {
                let world_matrix = &world_matrixs[*id];
                let transform = match transforms.get(*id) {
                    Some(r) => r,
                    None => &default_transform,
                };
                modify_matrix(
                    render_index,
                    create_let_top_offset_matrix(
                        layout,
                        world_matrix,
                        transform,
                        0.0,
                        0.0,
                        // render_obj.depth,
                    ),
                    render_obj,
                    &notify,
                );
            }

            let image = &border_images[*id];
            if (dirty & &*GEO_DIRTY).any() || dirty1 & CalcType::Layout as usize != 0 {
                let image_clip = border_image_clips.get(*id);
                let image_slice = border_image_slices.get(*id);
                let image_repeat = border_image_repeats.get(*id);
                render_obj.geometry = create_geo(image, texture, image_clip, image_slice, image_repeat, layout, &mut engine);

                // BorderImage修改， 修改texture
                if dirty1 & CalcType::BorderImageTexture as usize != 0 {
                    // 如果四边形与图片宽高一样， 使用点采样， TODO
                    render_obj.paramter.set_texture("texture", (&texture.0.bind, &self.default_sampler));
                    notify.modify_event(render_index, "ubo", 0);
                }
                notify.modify_event(render_index, "geometry", 0);
            }
			let border_radius = border_radiuses.get(*id);

            if dirty[StyleType::Opacity as usize] {
                if is_leaf[*id].0 {
                    render_obj.paramter.set_single_uniform("alpha", UniformValue::Float1(alpha));
                    notify.modify_event(render_index, "ubo", 0);
                }
            }
            // 不透明度脏或图片脏， 设置is_opacity
            if dirty[StyleType::Opacity as usize] || dirty1 & CalcType::BorderImageTexture as usize != 0 {
                // let opacity = opacitys[*id].0;
                let is_opacity_old = render_obj.is_opacity;
                let is_opacity = if let ROpacity::Opaque = texture.0.opacity { true } else { false };
                render_obj.is_opacity = is_opacity && border_radius.is_none();
                if render_obj.is_opacity != is_opacity_old {
                    notify.modify_event(render_index, "is_opacity", 0);
                }
                modify_opacity(engine, render_obj, default_state);
            }
            notify.modify_event(render_index, "", 0);
        }
    }
}

// 监听实体销毁，删除索引
impl<'a, C: HalContext + 'static> EntityListener<'a, Node, DeleteEvent> for BorderImageSys<C> {
    type ReadData = ();
    type WriteData = ();

    fn listen(&mut self, event: &Event, _read: Self::ReadData, _: Self::WriteData) {
        self.render_map.remove(event.id); // 移除索引
    }
}

impl<C: HalContext + 'static> BorderImageSys<C> {
    pub fn new(engine: &mut Engine<C>) -> Self {
        let mut default_sampler = SamplerDesc::default();
        default_sampler.u_wrap = TextureWrapMode::ClampToEdge;
        default_sampler.v_wrap = TextureWrapMode::ClampToEdge;
        BorderImageSys {
            render_map: VecMap::default(),
            default_sampler: engine.create_sampler_res(default_sampler),
            default_paramter: ImageParamter::default(),
            marker: PhantomData,
        }
    }

    pub fn with_capacity(engine: &mut Engine<C>, capacity: usize) -> Self {
        let mut default_sampler = SamplerDesc::default();
        default_sampler.u_wrap = TextureWrapMode::ClampToEdge;
        default_sampler.v_wrap = TextureWrapMode::ClampToEdge;
        BorderImageSys {
            render_map: VecMap::with_capacity(capacity),
            default_sampler: engine.create_sampler_res(default_sampler),
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
    fn create_render_obj(&mut self, id: usize, render_objs: &mut SingleCaseImpl<RenderObjs>, default_state: &DefaultState, alpha: f32) -> usize {
        let index = create_render_obj(
            id,
            -0.1,
            true,
            IMAGE_VS_SHADER_NAME.clone(),
            IMAGE_FS_SHADER_NAME.clone(),
            Share::new(self.default_paramter.clone()),
            default_state,
            render_objs,
            &mut self.render_map,
        );
        render_objs[index].paramter.set_single_uniform("alpha", UniformValue::Float1(alpha));
        index
    }
}

fn create_geo<C: HalContext + 'static>(
    img: &BorderImage,
    texture: &BorderImageTexture,
    clip: Option<&BorderImageClip>,
    slice: Option<&BorderImageSlice>,
    repeat: Option<&BorderImageRepeat>,
    layout: &LayoutR,
    engine: &mut Engine<C>,
) -> ResWrapper<GeometryRes> {
    let h = geo_hash(img, texture, clip, slice, repeat, layout);
    match engine.geometry_res_map.get(&h) {
        Some(r) => ResWrapper::Handle(r),
        None => {
            let (positions, uvs, indices) = get_border_image_stream(texture, clip, slice, repeat, layout, Vec::new(), Vec::new(), Vec::new());
            engine.create_geo_res(
                h,
                indices.as_slice(),
                &[
                    AttributeDecs::new(AttributeName::Position, positions.as_slice(), 2),
                    AttributeDecs::new(AttributeName::UV0, uvs.as_slice(), 2),
                ],
            )
        }
    }
}


#[inline]
fn geo_hash(
    img: &BorderImage,
    texture: &BorderImageTexture,
    clip: Option<&BorderImageClip>,
    slice: Option<&BorderImageSlice>,
    repeat: Option<&BorderImageRepeat>,
    layout: &LayoutR,
) -> u64 {
    let mut hasher = DefaultHasher::default();
    BORDER_IMAGE.hash(&mut hasher);
    img.0.hash(&mut hasher);
    clip.hash(&mut hasher);
    slice.hash(&mut hasher);
    repeat.hash(&mut hasher);
    let width = layout.rect.right - layout.rect.left;
    let height = layout.rect.bottom - layout.rect.top;
    f32_3_hash_(width, height, layout.border.left, &mut hasher);
    // 四条边框都参与几何计算，必须全部纳入 hash，避免缓存复用错误几何
    f32_3_hash_(layout.border.right, layout.border.top, layout.border.bottom, &mut hasher);
    f32_3_hash_(texture.0.width as f32, texture.0.height as f32, 0.0, &mut hasher);
    hasher.finish()
}

#[inline]
fn get_border_image_stream(
    texture: &BorderImageTexture,
    clip: Option<&BorderImageClip>,
    slice: Option<&BorderImageSlice>,
    repeat: Option<&BorderImageRepeat>,
    layout: &LayoutR,
    mut point_arr: Polygon,
    mut uv_arr: Polygon,
    mut index_arr: Vec<u16>,
) -> (Polygon, Polygon, Vec<u16>) {
    let (uv1, uv2) = match clip {
        Some(c) => (Point2::new(*c.left, *c.top), Point2::new(*c.right, *c.bottom)),
        _ => (Point2::new(0.0, 0.0), Point2::new(1.0, 1.0)),
    };
    let width = layout.rect.right - layout.rect.left;
    let height = layout.rect.bottom - layout.rect.top;
    let p1 = Point2::new(0.0, 0.0);
    let p2 = Point2::new(width, height);
    // let w = p2.x - p1.x;
    // let h = p2.y - p1.y;
    let left = layout.border.left;
    let right = width - layout.border.right;
    let top = layout.border.top;
    let bottom = height - layout.border.bottom;
    let uvw = uv2.x - uv1.x;
    let uvh = uv2.y - uv1.y;
    let (uv_left, uv_right, uv_top, uv_bottom) = match slice {
        Some(slice) => (
            uv1.x + *slice.left * uvw,
            uv2.x - *slice.right * uvw,
            uv1.y + *slice.top * uvh,
            uv2.y - *slice.bottom * uvh,
        ),
        None => (uv1.x + 0.25 * uvw, uv2.x - 0.25 * uvw, uv1.y + 0.25 * uvh, uv2.y - 0.25 * uvh),
    };

    //  p1, p2, w, h, left, right, top, bottom, "UV::", uv1, uv2, uvw, uvh, uv_left, uv_right, uv_top, uv_bottom);
    // TODO 在仅使用左或上的边框时， 应该优化成8个顶点
    // 先将16个顶点和uv放入数组，记录偏移量
    let mut pi = (point_arr.len() / 3) as u16;
    // 左上的4个点
    let p_x1_y1 = push_vertex(&mut point_arr, &mut uv_arr, p1.x, p1.y, uv1.x, uv1.y, &mut pi);
    let p_x1_top = push_vertex(&mut point_arr, &mut uv_arr, p1.x, top, uv1.x, uv_top, &mut pi);
    let p_left_top = push_vertex(&mut point_arr, &mut uv_arr, left, top, uv_left, uv_top, &mut pi);
    let p_left_y1 = push_vertex(&mut point_arr, &mut uv_arr, left, p1.y, uv_left, uv1.y, &mut pi);
    push_quad(&mut index_arr, p_x1_y1, p_x1_top, p_left_top, p_left_y1);

    // 左下的4个点
    let p_x1_bottom = push_vertex(&mut point_arr, &mut uv_arr, p1.x, bottom, uv1.x, uv_bottom, &mut pi);
    let p_x1_y2 = push_vertex(&mut point_arr, &mut uv_arr, p1.x, p2.y, uv1.x, uv2.y, &mut pi);
    let p_left_y2 = push_vertex(&mut point_arr, &mut uv_arr, left, p2.y, uv_left, uv2.y, &mut pi);
    let p_left_bottom = push_vertex(&mut point_arr, &mut uv_arr, left, bottom, uv_left, uv_bottom, &mut pi);
    push_quad(&mut index_arr, p_x1_bottom, p_x1_y2, p_left_y2, p_left_bottom);

    // 右下的4个点
    let p_right_bottom = push_vertex(&mut point_arr, &mut uv_arr, right, bottom, uv_right, uv_bottom, &mut pi);
    let p_right_y2 = push_vertex(&mut point_arr, &mut uv_arr, right, p2.y, uv_right, uv2.y, &mut pi);
    let p_x2_y2 = push_vertex(&mut point_arr, &mut uv_arr, p2.x, p2.y, uv2.x, uv2.y, &mut pi);
    let p_x2_bottom = push_vertex(&mut point_arr, &mut uv_arr, p2.x, bottom, uv2.x, uv_bottom, &mut pi);
    push_quad(&mut index_arr, p_right_bottom, p_right_y2, p_x2_y2, p_x2_bottom);

    // 右上的4个点
    let p_right_y1 = push_vertex(&mut point_arr, &mut uv_arr, right, p1.y, uv_right, uv1.y, &mut pi);
    let p_right_top = push_vertex(&mut point_arr, &mut uv_arr, right, top, uv_right, uv_top, &mut pi);
    let p_x2_top = push_vertex(&mut point_arr, &mut uv_arr, p2.x, top, uv2.x, uv_top, &mut pi);
    let p_x2_y1 = push_vertex(&mut point_arr, &mut uv_arr, p2.x, p1.y, uv2.x, uv1.y, &mut pi);
    push_quad(&mut index_arr, p_right_y1, p_right_top, p_x2_top, p_x2_y1);


    let (texture_left_width, texture_right_width, texture_top_height, texture_bottom_height, texture_center_width, texture_center_height) = (
        texture.0.width as f32 * (uv_left - uv1.x),
        texture.0.width as f32 * (uv2.x - uv_right),
        texture.0.height as f32 * (uv_top - uv1.y),
        texture.0.height as f32 * (uv2.y - uv_bottom),
        texture.0.width as f32 * (uv_right - uv_left),
        texture.0.height as f32 * (uv_bottom - uv_top),
    );

    let repeat = match repeat {
        Some(r) => r.clone(),
        None => BorderImageRepeat::default(),
    };
    let (
        (offset_top, space_top, step_top),
        (offset_bottom, space_bottom, step_bottom),
        (offset_left, space_left, step_left),
        (offset_right, space_right, step_right),
    ) = (
        calc_step(right - left, scaled_center_size(layout.border.top, texture_top_height, texture_center_width), repeat.x),
        calc_step(
            right - left,
            scaled_center_size(layout.border.bottom, texture_bottom_height, texture_center_width),
            repeat.x,
        ),
        calc_step(bottom - top, scaled_center_size(layout.border.left, texture_left_width, texture_center_height), repeat.y),
        calc_step(bottom - top, scaled_center_size(layout.border.right, texture_right_width, texture_center_height), repeat.y),
    );

    if step_top > 0.0 {
        push_u_arr(
            &mut point_arr,
            &mut uv_arr,
            &mut index_arr,
            p_left_y1,
            p_left_top,
            p_right_top,
            p_right_y1,
            uv_left,
            uv1.y,
            uv_right,
            uv_top,
            step_top,
            offset_top,
            space_top,
            &mut pi,
        ); // 上边
    }
    if step_bottom > 0.0 {
        push_u_arr(
            &mut point_arr,
            &mut uv_arr,
            &mut index_arr,
            p_left_bottom,
            p_left_y2,
            p_right_y2,
            p_right_bottom,
            uv_left,
            uv_bottom,
            uv_right,
            uv2.y,
            step_bottom,
            offset_bottom,
            space_bottom,
            &mut pi,
        ); // 下边
    }

    if step_left > 0.0 {
        push_v_arr(
            &mut point_arr,
            &mut uv_arr,
            &mut index_arr,
            p_x1_top,
            p_x1_bottom,
            p_left_bottom,
            p_left_top,
            uv1.x,
            uv_top,
            uv_left,
            uv_bottom,
            step_left,
            offset_left,
            space_left,
            &mut pi,
        ); // 左边
    }
    if step_right > 0.0 {
        push_v_arr(
            &mut point_arr,
            &mut uv_arr,
            &mut index_arr,
            p_right_top,
            p_right_bottom,
            p_x2_bottom,
            p_x2_top,
            uv_right,
            uv_top,
            uv2.x,
            uv_bottom,
            step_right,
            offset_right,
            space_right,
            &mut pi,
        ); // 右边
    }

    // 处理中间：中间区域独立于四条边，按 repeat 真正平铺；不再复用边的 step，避免 slice 为 0 时 0/0 产生 NaN
    if let Some(slice) = slice {
        if slice.fill {
            let center_w = center_tile_size(
                layout.border.top, texture_top_height,
                layout.border.bottom, texture_bottom_height,
                texture_center_width,
            );
            let center_h = center_tile_size(
                layout.border.left, texture_left_width,
                layout.border.right, texture_right_width,
                texture_center_height,
            );
            let xs = center_axis(left, right, center_w, repeat.x, uv_left, uv_right);
            let ys = center_axis(top, bottom, center_h, repeat.y, uv_top, uv_bottom);
            for &(y0, y1, v0, v1) in ys.iter() {
                for &(x0, x1, u0, u1) in xs.iter() {
                    let a = push_vertex(&mut point_arr, &mut uv_arr, x0, y0, u0, v0, &mut pi);
                    let b = push_vertex(&mut point_arr, &mut uv_arr, x0, y1, u0, v1, &mut pi);
                    let c = push_vertex(&mut point_arr, &mut uv_arr, x1, y1, u1, v1, &mut pi);
                    let d = push_vertex(&mut point_arr, &mut uv_arr, x1, y0, u1, v0, &mut pi);
                    push_quad(&mut index_arr, a, b, c, d);
                }
            }
        }
    }

    (point_arr, uv_arr, index_arr)
}
// 将四边形放进数组中
fn push_vertex(point_arr: &mut Polygon, uv_arr: &mut Polygon, x: f32, y: f32, u: f32, v: f32, i: &mut u16) -> u16 {
    point_arr.extend_from_slice(&[x, y]);
    uv_arr.extend_from_slice(&[u, v]);
    let r = *i;
    *i += 1;
    r
}
// 将四边形放进数组中
pub fn push_quad(index_arr: &mut Vec<u16>, p1: u16, p2: u16, p3: u16, p4: u16) { index_arr.extend_from_slice(&[p1, p2, p3, p1, p3, p4]); }

// // 根据参数计算uv的step
// fn calc_step(csize: f32, img_size: f32, rtype: ImageRepeatType) -> f32 {
//     let c = csize / img_size;
//     if c <= 1.0 {
//         return std::f32::INFINITY;
//     }
//     match rtype {
//         ImageRepeatType::Repeat => csize / c.round(),
//         ImageRepeatType::Round => csize / c.ceil(),
//         ImageRepeatType::Space => csize / c.floor(),
//         _ => std::f32::INFINITY,
//     }
// }


// 计算边框切片缩放后的中间图尺寸；分母为 0 时返回 0，避免 0/0 产生 NaN
#[inline]
fn scaled_center_size(border: f32, texture_border: f32, texture_center: f32) -> f32 {
    if texture_border > 0.0 {
        border / texture_border * texture_center
    } else {
        0.0
    }
}

// 计算中心图在某一方向上的平铺单元尺寸：优先用边框缩放，缩放无效时退回图片中心区域原始尺寸
#[inline]
fn center_tile_size(
    border1: f32, texture_border1: f32,
    border2: f32, texture_border2: f32,
    texture_center: f32,
) -> f32 {
    let scaled = scaled_center_size(border1, texture_border1, texture_center);
    if scaled > 0.0 {
        return scaled;
    }
    let scaled = scaled_center_size(border2, texture_border2, texture_center);
    if scaled > 0.0 {
        return scaled;
    }
    texture_center
}

// 按 repeat 规则把一个方向切成若干段，返回 (布局起点, 布局终点, uv 起点, uv 终点)
fn center_axis(
    lo: f32, hi: f32, tile: f32, rtype: ImageRepeatOption, uv_lo: f32, uv_hi: f32,
) -> Vec<(f32, f32, f32, f32)> {
    let mut out = Vec::new();
    let region = hi - lo;
    if !(region > 0.0) || !(tile > 0.0) {
        return out;
    }
    match rtype {
        ImageRepeatOption::Stretch => out.push((lo, hi, uv_lo, uv_hi)),
        ImageRepeatOption::Repeat => {
            let count = (region / tile).ceil();
            if count <= 0.0 {
                return out;
            }
            let total = count * tile;
            let start = lo - (total - region) / 2.0;
            let n = count as i32;
            for i in 0..n {
                let p0 = start + i as f32 * tile;
                let p1 = p0 + tile;
                let cs = p0.max(lo);
                let ce = p1.min(hi);
                if ce <= cs {
                    continue;
                }
                let u0 = uv_lo + (cs - p0) / tile * (uv_hi - uv_lo);
                let u1 = uv_lo + (ce - p0) / tile * (uv_hi - uv_lo);
                out.push((cs, ce, u0, u1));
            }
        }
        ImageRepeatOption::Round => {
            let count = (region / tile).round().max(1.0);
            let step = region / count;
            let n = count as i32;
            for i in 0..n {
                let p0 = lo + i as f32 * step;
                out.push((p0, p0 + step, uv_lo, uv_hi));
            }
        }
        ImageRepeatOption::Space => {
            let count = (region / tile).floor();
            if count < 1.0 {
                return out;
            }
            let space = (region - count * tile) / (count + 1.0);
            let n = count as i32;
            for i in 0..n {
                let p0 = lo + space + i as f32 * (tile + space);
                out.push((p0, p0 + tile, uv_lo, uv_hi));
            }
        }
    }
    out
}

fn calc_step(csize: f32, img_size: f32, rtype: ImageRepeatOption) -> (f32, f32, f32) {
    if let ImageRepeatOption::Stretch = rtype {
        return (0.0, 0.0, csize);
    }
    // img_size 或可绘制区域为 0（含 NaN）时直接返回 0，避免 0/0 产生 NaN 污染后续边/中心判断
    if !(img_size > 0.0) || !(csize > 0.0) {
        return (0.0, 0.0, 0.0);
    }
    let c = csize / img_size;
    let f = c.round();
    if eq_f32(f, c) {
        // 整数倍的情况（这里消除了浮点误差，大致为整数倍，都认为是整数倍）
        return (0.0, 0.0, img_size);
    }

    match rtype {
        ImageRepeatOption::Repeat => (-(csize % img_size) / 2.0, 0.0, img_size),
        ImageRepeatOption::Round => (0.0, 0.0, if f > 0.0 { csize / f } else { csize }),
        ImageRepeatOption::Space => {
            let space = csize % img_size; // 空白尺寸
            let pre_space = space / (c.floor() + 1.0);
            (0.0, pre_space, if c >= 1.0 { img_size } else { 0.0 })
        }
        _ => (0.0, 0.0, csize),
    }
}

// 将指定区域按u切开
fn push_u_arr(
    point_arr: &mut Polygon,
    uv_arr: &mut Polygon,
    index_arr: &mut Vec<u16>,
    mut p1: u16,
    mut p2: u16,
    mut p3: u16,
    mut p4: u16,
    u1: f32,
    v1: f32,
    u2: f32,
    v2: f32,
    step: f32,
    offset: f32,
    space: f32,
    i: &mut u16,
) {
    let y1 = point_arr[p1 as usize * 2 + 1];
    let y2 = point_arr[p2 as usize * 2 + 1];
    let mut cur = point_arr[p1 as usize * 2];
    let mut max = point_arr[p3 as usize * 2];

    if offset != 0.0 {
        // repeat
        let u_diff = offset / step * (u2 - u1);
        let (u_start, u_end) = (u1 - u_diff, u2 + u_diff);
        p1 = push_vertex(point_arr, uv_arr, cur, y1, u_start, v1, i);
        p2 = push_vertex(point_arr, uv_arr, cur, y2, u_start, v2, i);
        p3 = push_vertex(point_arr, uv_arr, max, y2, u_end, v2, i);
        p4 = push_vertex(point_arr, uv_arr, max, y1, u_end, v1, i);
        cur += offset;
    }

    if space != 0.0 {
        max = max - space;
        cur = cur + space;
        p1 = push_vertex(point_arr, uv_arr, cur, y1, u1, v1, i);
        p2 = push_vertex(point_arr, uv_arr, cur, y2, u1, v2, i);
        p3 = push_vertex(point_arr, uv_arr, max, y2, u2, v2, i);
        p4 = push_vertex(point_arr, uv_arr, max, y1, u2, v1, i);
    }
    cur += step;

    let mut pt1 = p1;
    let mut pt2 = p2;
    while cur < max {
        let i3 = push_vertex(point_arr, uv_arr, cur, y2, u2, v2, i);
        let i4 = push_vertex(point_arr, uv_arr, cur, y1, u2, v1, i);
        push_quad(index_arr, pt1, pt2, i3, i4);
        cur += space;
        // 因为uv不同，新插入2个顶点
        pt1 = push_vertex(point_arr, uv_arr, cur, y1, u1, v1, i);
        pt2 = push_vertex(point_arr, uv_arr, cur, y2, u1, v2, i);
        cur += step;
    }
    push_quad(index_arr, pt1, pt2, p3, p4);
}
// 将指定区域按v切开
fn push_v_arr(
    point_arr: &mut Polygon,
    uv_arr: &mut Polygon,
    index_arr: &mut Vec<u16>,
    mut p1: u16,
    mut p2: u16,
    mut p3: u16,
    mut p4: u16,
    u1: f32,
    v1: f32,
    u2: f32,
    v2: f32,
    step: f32,
    offset: f32,
    space: f32,
    i: &mut u16,
) {
    let x1 = point_arr[p1 as usize * 2];
    let x2 = point_arr[p4 as usize * 2];
    let (mut cur, mut max) = (0.0, 0.0);

    push_v_box(
        point_arr, uv_arr, &mut p1, &mut p2, &mut p3, &mut p4, &mut cur, &mut max, u1, v1, u2, v2, step, offset, space, i,
    );

    cur += step;

    let mut pt1 = p1;
    let mut pt4 = p4;
    while cur < max {
        let i2 = push_vertex(point_arr, uv_arr, x1, cur, u1, v2, i);
        let i3 = push_vertex(point_arr, uv_arr, x2, cur, u2, v2, i);
        push_quad(index_arr, pt1, i2, i3, pt4);
        // 因为uv不同，新插入2个顶点
        cur += space;
        pt1 = push_vertex(point_arr, uv_arr, x1, cur, u1, v1, i);
        pt4 = push_vertex(point_arr, uv_arr, x2, cur, u2, v1, i);
        cur += step;
    }
    push_quad(index_arr, pt1, p2, p3, pt4);
}


#[inline]
pub fn push_v_box(
    point_arr: &mut Polygon,
    uv_arr: &mut Polygon,
    p1: &mut u16,
    p2: &mut u16,
    p3: &mut u16,
    p4: &mut u16,
    cur: &mut f32,
    max: &mut f32,
    u1: f32,
    v1: f32,
    u2: f32,
    v2: f32,
    step: f32,
    offset: f32,
    space: f32,
    i: &mut u16,
) {
    let x1 = point_arr[*p1 as usize * 2];
    let x2 = point_arr[*p4 as usize * 2];

    *max = point_arr[*p3 as usize * 2 + 1];
    *cur = point_arr[*p1 as usize * 2 + 1];
    if offset != 0.0 {
        // repeat
        let v_diff = offset / step * (v2 - v1);
        let (v_start, v_end) = (v1 - v_diff, v2 + v_diff);
        *p1 = push_vertex(point_arr, uv_arr, x1, *cur, u1, v_start, i);
        *p2 = push_vertex(point_arr, uv_arr, x1, *max, u1, v_end, i);
        *p3 = push_vertex(point_arr, uv_arr, x2, *max, u2, v_end, i);
        *p4 = push_vertex(point_arr, uv_arr, x2, *cur, u2, v_start, i);
        *cur += offset;
    }

    if space != 0.0 {
        *cur = *cur + space;
        *max = *max - space;
        *p1 = push_vertex(point_arr, uv_arr, x1, *cur, u1, v1, i);
        *p2 = push_vertex(point_arr, uv_arr, x1, *max, u1, v2, i);
        *p3 = push_vertex(point_arr, uv_arr, x2, *max, u2, v2, i);
        *p4 = push_vertex(point_arr, uv_arr, x2, *cur, u2, v1, i);
    }
}


impl_system! {
    BorderImageSys<C> where [C: HalContext + 'static],
    true,
    {
        EntityListener<Node, DeleteEvent>
    }
}
