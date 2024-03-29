/**
 * 边框图片渲染对象的构建及其属性设置
 */
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use hash::DefaultHasher;

use atom::Atom;
use ecs::{DeleteEvent, MultiCaseImpl, MultiCaseListener, Runner, SingleCaseImpl};
use ecs::monitor::NotifyImpl;
use hal_core::*;
use map::vecmap::VecMap;
use map::Map;
use share::Share;

use component::calc::{Opacity, LayoutR};
use component::calc::*;
use component::user::*;
use entity::Node;
use render::engine::{AttributeDecs, Engine, ShareEngine};
use render::res::{GeometryRes, Opacity as ROpacity, SamplerRes};
use single::*;
use system::render::shaders::image::{IMAGE_FS_SHADER_NAME, IMAGE_VS_SHADER_NAME};
use system::util::*;

// 本系统关心的脏
const DIRTY_TY: usize = StyleType::Matrix as usize
    | StyleType::Opacity as usize
    | StyleType::Layout as usize
    | StyleType::BorderImage as usize
    | StyleType::BorderImageClip as usize
    | StyleType::BorderImageSlice as usize
    | StyleType::BorderImageRepeat as usize;

// 一些与BorderImage渲染对象的几何体相关的属性脏
const GEO_DIRTY: usize = StyleType::Layout as usize
    | StyleType::BorderImage as usize
    | StyleType::BorderImageClip as usize
    | StyleType::BorderImageSlice as usize
    | StyleType::BorderImageRepeat as usize;

lazy_static! {
    static ref BORDER_IMAGE: Atom = Atom::from("border_image");
}

pub struct BorderImageSys<C: HalContext + 'static> {
    render_map: VecMap<usize>,
    default_sampler: Share<SamplerRes>,
    default_paramter: ImageParamter,
    marker: PhantomData<C>,
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for BorderImageSys<C> {
    type ReadData = (
        &'a MultiCaseImpl<Node, BorderImage>,
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
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<ShareEngine<C>>,
    );
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        if (read.9).0.len() == 0 {
            return;
        }

        let (
            border_images,
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
        ) = read;
        let (render_objs, mut engine) = write;

		let default_transform = Transform::default();
		let notify = unsafe { &* (render_objs.get_notify_ref() as *const NotifyImpl)} ;
        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                }
            };

            // 不存在Image关心的脏, 跳过
            if style_mark.dirty & DIRTY_TY == 0 {
                continue;
            }

            let mut dirty = style_mark.dirty;

            // BorderImage脏， 如果不存在BorderImage的本地样式和class样式， 删除渲染对象
            let render_index = if dirty & StyleType::BorderImage as usize != 0 {
                if style_mark.local_style & StyleType::BorderImage as usize == 0
                    && style_mark.class_style & StyleType::BorderImage as usize == 0
                {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                } else {
                    dirty |= DIRTY_TY;
                    match self.render_map.get_mut(*id) {
                        Some(r) => *r,
                        None => self.create_render_obj(*id, render_objs, default_state)
                    }
                }
            } else {
                match self.render_map.get_mut(*id) {
                    Some(r) => *r,
                    None => continue,
                }
            };

            let image = &border_images[*id];
            let render_obj = &mut render_objs[render_index];
            // 图片不存在， 跳过
            if image.0.src.is_none() {
                render_obj.geometry = None;
                continue;
            }

            let layout = &layouts[*id];
			
            // 世界矩阵脏， 设置世界矩阵ubo
            if dirty & StyleType::Matrix as usize != 0 {
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
                        render_obj.depth,
                    ),
                    render_obj,
                    &notify,
                );
            }

            let image = &border_images[*id];
            if dirty & GEO_DIRTY != 0 {
                let image_clip = border_image_clips.get(*id);
                let image_slice = border_image_slices.get(*id);
                let image_repeat = border_image_repeats.get(*id);
                render_obj.geometry = create_geo(
                    image,
                    image_clip,
                    image_slice,
                    image_repeat,
                    layout,
                    &mut engine,
                );

                // BorderImage修改， 修改texture
                if dirty & StyleType::BorderImage as usize != 0 {
                    // 如果四边形与图片宽高一样， 使用点采样， TODO
                    render_obj.paramter.set_texture(
                        "texture",
                        (&image.0.src.as_ref().unwrap().bind, &self.default_sampler),
                    );
                    notify.modify_event(render_index, "ubo", 0);
                }
                notify.modify_event(render_index, "geometry", 0);
            }

            // 不透明度脏或图片脏， 设置is_opacity
            if dirty & StyleType::Opacity as usize != 0
                || dirty & StyleType::BorderImage as usize != 0
            {
				let opacity = opacitys[*id].0;
				let is_opacity_old = render_obj.is_opacity;
                let is_opacity = if opacity < 1.0 {
                    false
                } else if let ROpacity::Opaque = image.0.src.as_ref().unwrap().opacity {
                    true
                } else {
                    false
                };
				render_obj.is_opacity = is_opacity;
				if render_obj.is_opacity != is_opacity_old {
					notify.modify_event(render_index, "is_opacity", 0);
				}
                modify_opacity(engine, render_obj, default_state);
            }
            notify.modify_event(render_index, "", 0);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImage, DeleteEvent>
    for BorderImageSys<C>
{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, render_objs: Self::WriteData) {
        self.remove_render_obj(event.id, render_objs)
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
				let notify = unsafe { &* (render_objs.get_notify_ref() as *const NotifyImpl)} ;
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
        default_state: &DefaultState,
    ) -> usize {
        create_render_obj(
            id,
            -0.1,
            true,
            IMAGE_VS_SHADER_NAME.clone(),
            IMAGE_FS_SHADER_NAME.clone(),
            Share::new(self.default_paramter.clone()),
            default_state,
            render_objs,
            &mut self.render_map,
        )
    }
}

fn create_geo<C: HalContext + 'static>(
    img: &BorderImage,
    clip: Option<&BorderImageClip>,
    slice: Option<&BorderImageSlice>,
    repeat: Option<&BorderImageRepeat>,
    layout: &LayoutR,
    engine: &mut Engine<C>,
) -> Option<Share<GeometryRes>> {
    let h = geo_hash(img, clip, slice, repeat, layout);
    match engine.geometry_res_map.get(&h) {
        Some(r) => Some(r.clone()),
        None => {
            let (positions, uvs, indices) = get_border_image_stream(
                img,
                clip,
                slice,
                repeat,
                layout,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
            Some(engine.create_geo_res(
                h,
                indices.as_slice(),
                &[
                    AttributeDecs::new(AttributeName::Position, positions.as_slice(), 2),
                    AttributeDecs::new(AttributeName::UV0, uvs.as_slice(), 2),
                ],
            ))
        }
    }
}

#[inline]
fn geo_hash(
    img: &BorderImage,
    clip: Option<&BorderImageClip>,
    slice: Option<&BorderImageSlice>,
    repeat: Option<&BorderImageRepeat>,
    layout: &LayoutR,
) -> u64 {
    let mut hasher = DefaultHasher::default();
    BORDER_IMAGE.hash(&mut hasher);
    img.0.url.hash(&mut hasher);
    match clip {
        Some(r) => f32_4_hash_(r.min.x, r.min.y, r.max.x, r.max.y, &mut hasher),
        None => 0.hash(&mut hasher),
    };
    match slice {
        Some(r) => {
            f32_4_hash_(r.left, r.top, r.bottom, r.right, &mut hasher);
            r.fill.hash(&mut hasher);
        }
        None => 0.hash(&mut hasher),
    };
    match repeat {
        Some(r) => {
            (r.0 as u8).hash(&mut hasher);
            (r.1 as u8).hash(&mut hasher);
        }
        None => 0.hash(&mut hasher),
	};
	let width = layout.rect.end - layout.rect.start;
	let height = layout.rect.bottom - layout.rect.top;
    f32_3_hash_(width, height, layout.border.start, &mut hasher);
    hasher.finish()
}

#[inline]
fn get_border_image_stream(
    img: &BorderImage,
    clip: Option<&BorderImageClip>,
    slice: Option<&BorderImageSlice>,
    repeat: Option<&BorderImageRepeat>,
    layout: &LayoutR,
    mut point_arr: Polygon,
    mut uv_arr: Polygon,
    mut index_arr: Vec<u16>,
) -> (Polygon, Polygon, Vec<u16>) {
    let src = &img.0.src.as_ref().unwrap();
    let (uv1, uv2) = match clip {
        Some(c) => (c.min, c.max),
        _ => (Point2::new(0.0, 0.0), Point2::new(1.0, 1.0)),
	};
	let width = layout.rect.end - layout.rect.start;
	let height = layout.rect.bottom - layout.rect.top;
    let p1 = Point2::new(0.0, 0.0);
    let p2 = Point2::new(width, height);
    let w = p2.x - p1.x;
    let h = p2.y - p1.y;
    let left = layout.border.start;
    let right = width - layout.border.end;
    let top = layout.border.top;
    let bottom = height - layout.border.bottom;
    let uvw = uv2.x - uv1.x;
    let uvh = uv2.y - uv1.y;
    let (uv_left, uv_right, uv_top, uv_bottom) = match slice {
        Some(slice) => (
            uv1.x + slice.left * uvw,
            uv2.x - slice.right * uvw,
            uv1.y + slice.top * uvh,
            uv2.y - slice.bottom * uvh,
        ),
        None => (
            uv1.x + 0.25 * uvw,
            uv2.x - 0.25 * uvw,
            uv1.y + 0.25 * uvh,
            uv2.y - 0.25 * uvh,
        ),
	};

    //  p1, p2, w, h, left, right, top, bottom, "UV::", uv1, uv2, uvw, uvh, uv_left, uv_right, uv_top, uv_bottom);
    // TODO 在仅使用左或上的边框时， 应该优化成8个顶点
    // 先将16个顶点和uv放入数组，记录偏移量
    let mut pi = (point_arr.len() / 3) as u16;
    // 左上的4个点
    let p_x1_y1 = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        p1.x,
        p1.y,
        uv1.x,
        uv1.y,
        &mut pi,
    );
    let p_x1_top = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        p1.x,
        top,
        uv1.x,
        uv_top,
        &mut pi,
    );
    let p_left_top = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        left,
        top,
        uv_left,
        uv_top,
        &mut pi,
    );
    let p_left_y1 = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        left,
        p1.y,
        uv_left,
        uv1.y,
        &mut pi,
    );
    push_quad(&mut index_arr, p_x1_y1, p_x1_top, p_left_top, p_left_y1);

    // 左下的4个点
    let p_x1_bottom = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        p1.x,
        bottom,
        uv1.x,
        uv_bottom,
        &mut pi,
    );
    let p_x1_y2 = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        p1.x,
        p2.y,
        uv1.x,
        uv2.y,
        &mut pi,
    );
    let p_left_y2 = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        left,
        p2.y,
        uv_left,
        uv2.y,
        &mut pi,
    );
    let p_left_bottom = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        left,
        bottom,
        uv_left,
        uv_bottom,
        &mut pi,
    );
    push_quad(
        &mut index_arr,
        p_x1_bottom,
        p_x1_y2,
        p_left_y2,
        p_left_bottom,
    );

    // 右下的4个点
    let p_right_bottom = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        right,
        bottom,
        uv_right,
        uv_bottom,
        &mut pi,
    );
    let p_right_y2 = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        right,
        p2.y,
        uv_right,
        uv2.y,
        &mut pi,
    );
    let p_x2_y2 = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        p2.x,
        p2.y,
        uv2.x,
        uv2.y,
        &mut pi,
    );
    let p_x2_bottom = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        p2.x,
        bottom,
        uv2.x,
        uv_bottom,
        &mut pi,
    );
    push_quad(
        &mut index_arr,
        p_right_bottom,
        p_right_y2,
        p_x2_y2,
        p_x2_bottom,
    );

    // 右上的4个点
    let p_right_y1 = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        right,
        p1.y,
        uv_right,
        uv1.y,
        &mut pi,
    );
    let p_right_top = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        right,
        top,
        uv_right,
        uv_top,
        &mut pi,
    );
    let p_x2_top = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        p2.x,
        top,
        uv2.x,
        uv_top,
        &mut pi,
    );
    let p_x2_y1 = push_vertex(
        &mut point_arr,
        &mut uv_arr,
        p2.x,
        p1.y,
        uv2.x,
        uv1.y,
        &mut pi,
    );
    push_quad(&mut index_arr, p_right_y1, p_right_top, p_x2_top, p_x2_y1);

    let (ustep, vstep) = match repeat {
        Some(&BorderImageRepeat(utype, vtype)) => {
            // 根据图像大小和uv计算
            let ustep = calc_step(right - left, src.width as f32 * (uv_right - uv_left), utype);
            let vstep = calc_step(
                bottom - top,
                src.height as f32 * (uv_bottom - uv_top),
                vtype,
            );
            (ustep, vstep)
        }
        _ => (w, h),
	};
	if ustep > 0.0 {
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
			ustep,
			&mut pi,
		); // 上边
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
			ustep,
			&mut pi,
		); // 下边
	}
	
	if vstep > 0.0 {
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
			vstep,
			&mut pi,
		); // 左边
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
			vstep,
			&mut pi,
		); // 右边
	}
	
       // 处理中间
    if let Some(slice) = slice {
        if slice.fill {
            push_quad(
                &mut index_arr,
                p_left_top,
                p_left_bottom,
                p_right_bottom,
                p_right_top,
            );
        }
	}

    (point_arr, uv_arr, index_arr)
}
// 将四边形放进数组中
fn push_vertex(
    point_arr: &mut Polygon,
    uv_arr: &mut Polygon,
    x: f32,
    y: f32,
    u: f32,
    v: f32,
    i: &mut u16,
) -> u16 {
    point_arr.extend_from_slice(&[x, y]);
    uv_arr.extend_from_slice(&[u, v]);
    let r = *i;
    *i += 1;
    r
}
// 将四边形放进数组中
fn push_quad(index_arr: &mut Vec<u16>, p1: u16, p2: u16, p3: u16, p4: u16) {
    index_arr.extend_from_slice(&[p1, p2, p3, p1, p3, p4]);
}

// 根据参数计算uv的step
fn calc_step(csize: f32, img_size: f32, rtype: BorderImageRepeatType) -> f32 {
    let c = csize / img_size;
    if c <= 1.0 {
        return std::f32::INFINITY;
    }
    match rtype {
        BorderImageRepeatType::Repeat => csize / c.round(),
        BorderImageRepeatType::Round => csize / c.ceil(),
        BorderImageRepeatType::Space => csize / c.floor(),
        _ => std::f32::INFINITY,
    }
}

// 将指定区域按u切开
fn push_u_arr(
    point_arr: &mut Polygon,
    uv_arr: &mut Polygon,
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
    i: &mut u16,
) {
    let y1 = point_arr[p1 as usize * 2 + 1];
    let y2 = point_arr[p2 as usize * 2 + 1];
    let mut cur = point_arr[p1 as usize * 2] + step;
    let max = point_arr[p3 as usize * 2];
    let mut pt1 = p1;
    let mut pt2 = p2;
    while cur < max {
        let i3 = push_vertex(point_arr, uv_arr, cur, y2, u2, v2, i);
        let i4 = push_vertex(point_arr, uv_arr, cur, y1, u2, v1, i);
        push_quad(index_arr, pt1, pt2, i3, i4);
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
    p1: u16,
    p2: u16,
    p3: u16,
    p4: u16,
    u1: f32,
    v1: f32,
    u2: f32,
    v2: f32,
    step: f32,
    i: &mut u16,
) {
    let x1 = point_arr[p1 as usize * 2];
    let x2 = point_arr[p4 as usize * 2];
    let mut cur = point_arr[p1 as usize * 2 + 1] + step;
    let max = point_arr[p3 as usize * 2 + 1];
    let mut pt1 = p1;
    let mut pt4 = p4;
    while cur < max {
        let i2 = push_vertex(point_arr, uv_arr, x1, cur, u1, v2, i);
        let i3 = push_vertex(point_arr, uv_arr, x2, cur, u2, v2, i);
        push_quad(index_arr, pt1, i2, i3, pt4);
        // 因为uv不同，新插入2个顶点
        pt1 = push_vertex(point_arr, uv_arr, x1, cur, u1, v1, i);
        pt4 = push_vertex(point_arr, uv_arr, x2, cur, u2, v1, i);
        cur += step;
    }
    push_quad(index_arr, pt1, p2, p3, pt4);
}

impl_system! {
    BorderImageSys<C> where [C: HalContext + 'static],
    true,
    {
        MultiCaseListener<Node, BorderImage, DeleteEvent>
    }
}
