/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::hash::{ Hasher, Hash };
use std::marker::PhantomData;

use fxhash::FxHasher32;

use share::Share;
use ecs::{SingleCaseImpl, MultiCaseImpl, MultiCaseListener, DeleteEvent, Runner};
use map::{ vecmap::VecMap };
use hal_core::*;
use atom::Atom;
// use std::hash::{ Hasher, Hash };

// use ordered_float::NotNan;

use component::user::*;
use component::calc::*;
use component::calc::{Opacity};
use entity::{Node};
use single::*;
use render::engine::{Engine};
use render::res::{Opacity as ROpacity, GeometryRes};
use system::util::*;
use system::render::shaders::image::{IMAGE_FS_SHADER_NAME, IMAGE_VS_SHADER_NAME};

const DIRTY_TY: usize = StyleType::Matrix as usize |
                        StyleType::Opacity as usize |
                        StyleType::Layout as usize |
                        StyleType::BorderImage as usize |
                        StyleType::BorderImageClip as usize |
                        StyleType::BorderImageSlice as usize |
                        StyleType::BorderImageRepeat as usize;

const GEO_DIRTY: usize = StyleType::Layout as usize |
                        StyleType::BorderImage as usize |
                        StyleType::BorderImageClip as usize |
                        StyleType::BorderImageSlice as usize |
                        StyleType::BorderImageRepeat as usize;

lazy_static! {
    static ref BORDER_IMAGE: Atom = Atom::from("border_image");
}

pub struct BorderImageSys<C: HalContext + 'static>{
    render_map: VecMap<usize>,
    default_sampler: Share<HalSampler>,
    marker: PhantomData<C>,
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for BorderImageSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, BorderImage>,
        &'a MultiCaseImpl<Node, BorderImageClip>,
        &'a MultiCaseImpl<Node, BorderImageSlice>,
        &'a MultiCaseImpl<Node, BorderImageRepeat>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a SingleCaseImpl<DefaultTable>,
        &'a SingleCaseImpl<DirtyList>,
        &'a SingleCaseImpl<DefaultState>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<Engine<C>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
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
            default_table,
            dirty_list,
            default_state,
        ) = read;
        let (render_objs, mut engine) = write;
        let default_transform = default_table.get::<Transform>().unwrap();
        let notify = render_objs.get_notify();

        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => continue,
            };
            let mut dirty = style_mark.dirty;

            // 不存在Image关心的脏, 跳过
            if dirty & DIRTY_TY == 0 {
                continue;
            }

            // BorderImage脏， 如果不存在BorderImage的本地样式和class样式， 删除渲染对象
            let render_index = if dirty & StyleType::BorderImage as usize != 0 {
                if style_mark.local_style & StyleType::BorderImage as usize == 0 && style_mark.class_style & StyleType::BorderImage as usize == 0  {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                } else {
                    match self.render_map.get_mut(*id) {
                        Some(r) => *r,
                        None => {
                            dirty |= DIRTY_TY;
                            self.create_render_obj(*id, render_objs, default_state)
                        },
                    }
                }  
            } else {
                match self.render_map.get_mut(*id) {
                    Some(r) => *r,
                    None => continue,
                }
            };

            let render_obj = unsafe {render_objs.get_unchecked_mut(render_index)};

            let layout = unsafe { layouts.get_unchecked(*id) };
            let image = unsafe { border_images.get_unchecked(*id) };
            let image_clip = border_image_clips.get(*id);
            let image_slice = border_image_slices.get(*id);
            let image_repeat = border_image_repeats.get(*id);
            let transform =  match transforms.get(*id) {
                Some(r) => r,
                None => default_transform,
            };
            let world_matrix = unsafe { world_matrixs.get_unchecked(*id) };
            
            if dirty & GEO_DIRTY != 0 {
                render_obj.geometry = create_geo(image, image_clip, image_slice, image_repeat, layout, &mut engine);
                
                // BorderImage修改， 修改texture
                if dirty & StyleType::BorderImage as usize != 0 {
                    // 如果四边形与图片宽高一样， 使用点采样， TODO
                    render_obj.paramter.set_texture("texture", (&image.src.bind, &self.default_sampler));
                    notify.modify_event(render_index, "ubo", 0);
                }
                notify.modify_event(render_index, "geometry", 0);
            }
            // 世界矩阵脏， 设置世界矩阵ubo
            if dirty & StyleType::Matrix as usize != 0 {
                modify_matrix(
                    render_index,
                    create_let_top_offset_matrix(layout, world_matrix, transform, 0.0, 0.0, render_obj.depth),
                    render_obj,
                    &notify,
                );
            }

            // 不透明度脏或图片脏， 设置is_opacity
            if dirty & StyleType::Opacity as usize != 0 || dirty & StyleType::BorderImage as usize != 0 {
                let opacity = unsafe {opacitys.get_unchecked(*id)}.0;
                let is_opacity = if opacity < 1.0 {
                    false
                }else if let ROpacity::Opaque = image.src.opacity{
                    true
                }else {
                    false
                };
                render_obj.is_opacity = is_opacity;
                notify.modify_event(render_index, "is_opacity", 0);
                modify_opacity(engine, render_obj);
            }
            notify.modify_event(render_index, "", 0);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImage, DeleteEvent> for BorderImageSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, render_objs: Self::WriteData){
        self.remove_render_obj(event.id, render_objs)
    }
}

impl<C: HalContext + 'static> BorderImageSys<C> {
    pub fn new(engine: &mut Engine<C>) -> Self{
        BorderImageSys {
            render_map: VecMap::default(),
            default_sampler: create_default_sampler(engine),
            marker: PhantomData,
        }
    }

    #[inline]
    fn remove_render_obj(&mut self, id: usize, render_objs: &mut SingleCaseImpl<RenderObjs>) {
        match self.render_map.remove(id) {
            Some(index) => {
                let notify = render_objs.get_notify();
                render_objs.remove(index, Some(notify));
            },
            None => ()
        };
    }

    #[inline]
    fn create_render_obj(
        &mut self,
        id: usize,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &DefaultState,
    ) -> usize{
        create_render_obj(
            id,
            -0.1,
            true,
            IMAGE_VS_SHADER_NAME.clone(),
            IMAGE_FS_SHADER_NAME.clone(),
            Share::new(ImageParamter::default()),
            default_state, render_objs,
            &mut self.render_map
        )
    }
}

#[inline]
fn create_geo<C: HalContext + 'static>(
    img: &BorderImage,
    clip: Option<&BorderImageClip>,
    slice: Option<&BorderImageSlice>,
    repeat: Option<&BorderImageRepeat>,
    layout: &Layout,
    engine: &mut Engine<C>,
) -> Option<Share<GeometryRes>>{
    let h = geo_hash(img, clip, slice, repeat, layout);
    match engine.res_mgr.get::<GeometryRes>(&h) {
        Some(r) => Some(r.clone()),
        None => {
            let (positions, uvs, indices) = get_border_image_stream(img, clip, slice, repeat, layout, Vec::new(), Vec::new(), Vec::new());
            let p_buffer = Share::new(create_buffer(&engine.gl, BufferType::Attribute, positions.len(), Some(BufferData::Float(&positions[..])), false));
            let u_buffer = Share::new(create_buffer(&engine.gl, BufferType::Attribute, uvs.len(), Some(BufferData::Float(&uvs[..])), false));
            let i_buffer = Share::new(create_buffer(&engine.gl, BufferType::Indices, indices.len(), Some(BufferData::Short(&indices[..])), false));
            let geo = create_geometry(&engine.gl);
            engine.gl.geometry_set_attribute(&geo, &AttributeName::Position, &p_buffer, 2).unwrap();
            engine.gl.geometry_set_attribute(&geo, &AttributeName::UV0, &u_buffer, 2).unwrap();
            engine.gl.geometry_set_indices_short(&geo, &i_buffer).unwrap();
            let geo_res = GeometryRes{geo: geo, buffers: vec![p_buffer, u_buffer, i_buffer]};
            Some(engine.res_mgr.create(h, geo_res))
        }
    }
}

#[inline]
fn geo_hash(
    img: &BorderImage,
    clip: Option<&BorderImageClip>,
    slice: Option<&BorderImageSlice>,
    repeat: Option<&BorderImageRepeat>,
    layout: &Layout,
) -> u64 {
    let mut hasher = FxHasher32::default();
    BORDER_IMAGE.hash(&mut hasher);
    img.url.hash(&mut hasher);
    match clip {
        Some(r) => f32_4_hash_(r.min.x, r.min.y, r.max.x, r.max.y, &mut hasher),
        None => 0.hash(&mut hasher),
    };
    match slice {
        Some(r) => {
            f32_4_hash_(r.left, r.top, r.bottom, r.right, &mut hasher);
            r.fill.hash(&mut hasher);
        },
        None => 0.hash(&mut hasher),
    };
    match repeat {
        Some(r) => {
            (r.0 as u8).hash(&mut hasher);
            (r.1 as u8).hash(&mut hasher);
        },
        None => 0.hash(&mut hasher),
    };
    f32_3_hash_(layout.width, layout.height, layout.border_left, &mut hasher);
    hasher.finish()
}

fn get_border_image_stream (
  img: &BorderImage,
  clip: Option<&BorderImageClip>,
  slice: Option<&BorderImageSlice>,
  repeat: Option<&BorderImageRepeat>,
  layout: &Layout,
  mut point_arr: Polygon,
  mut uv_arr: Polygon,
  mut index_arr: Vec<u16>
) -> (Polygon, Polygon, Vec<u16>){
    let (uv1, uv2) = match clip {
        Some(c) => (c.min, c.max),
        _ => (Point2::new(0.0,0.0), Point2::new(1.0,1.0))
    };
    let p1 = Point2::new(0.0, 0.0);
    let p2 = Point2::new(layout.width, layout.height);
    let w = p2.x - p1.x;
    let h = p2.y - p1.y;
    let left = layout.border_left;
    let right = layout.width - layout.border_right;
    let top = layout.border_top;
    let bottom = layout.height - layout.border_bottom;
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
    let mut pi = (point_arr.len() / 3)  as u16;
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

    let (ustep, vstep) = match repeat {
      Some(&BorderImageRepeat(utype, vtype)) => {
        // 根据图像大小和uv计算
        let ustep = calc_step(right - left, img.src.width as f32 * (uv_right - uv_left), utype);
        let vstep = calc_step(bottom - top, img.src.height as f32 * (uv_bottom - uv_top), vtype);
        (ustep, vstep)
      },
      _ => (w, h)
    };
    push_u_arr(&mut point_arr, &mut uv_arr, &mut index_arr,
    p_left_y1, p_left_top, p_right_top, p_right_y1,
    uv_left, uv1.y, uv_right, uv_top, ustep, &mut pi); // 上边
    push_u_arr(&mut point_arr, &mut uv_arr, &mut index_arr,
    p_left_bottom, p_left_y2, p_right_y2, p_right_bottom,
    uv_left, uv_bottom, uv_right, uv2.y, ustep, &mut pi); // 下边
    push_v_arr(&mut point_arr, &mut uv_arr, &mut index_arr,
    p_x1_top, p_x1_bottom, p_left_bottom, p_left_top,
    uv1.x, uv_top, uv_left, uv_bottom, vstep, &mut pi); // 左边
    push_v_arr(&mut point_arr, &mut uv_arr, &mut index_arr,
    p_right_top, p_right_bottom, p_x2_bottom, p_x2_top,
    uv_right, uv_top, uv2.x, uv_bottom, vstep, &mut pi); // 右边
    // 处理中间
    if let Some(slice) = slice{
        if slice.fill {
            push_quad(&mut index_arr, p_left_top, p_left_bottom, p_right_bottom, p_right_top);
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
fn push_quad(index_arr: &mut Vec<u16>, p1: u16, p2: u16, p3: u16, p4: u16){
    index_arr.extend_from_slice(&[p1, p2, p3, p1, p3, p4]);
}
 
// fn border_image_geo_hash(
//     img: &BorderImage,
//     clip: Option<&BorderImageClip>,
//     slice: Option<&BorderImageSlice>,
//     repeat: Option<&BorderImageRepeat>,
//     layout: &Layout
// ) -> u64 {
//     let mut hasher = DefaultHasher::new();
//     BORDER_IMAGE.hash(&mut hasher);
//     img.src.name.hash(&mut hasher);
//     match clip {
//         Some(r) => {
//             CLIP.hash(&mut hasher);
//             NotNan::unchecked_new(r.0.min.x).hash(&mut hasher);
//             NotNan::unchecked_new(r.0.min.y).hash(&mut hasher);
//             NotNan::unchecked_new(r.0.max.x).hash(&mut hasher);
//             NotNan::unchecked_new(r.0.max.y).hash(&mut hasher);
//         },
//         None => (),
//     };
//     match slice {
//         Some(r) => {
//             SLICE.hash(&mut hasher);
//             NotNan::unchecked_new(r.left).hash(&mut hasher);
//             NotNan::unchecked_new(r.top).hash(&mut hasher);
//             NotNan::unchecked_new(r.right).hash(&mut hasher);
//             NotNan::unchecked_new(r.bottom).hash(&mut hasher);
//             r.fill.hash(&mut hasher);
//         },
//         None => (),
//     };
//     match repeat {
//         Some(r) => {
//             REPEAT.hash(&mut hasher);
//             unsafe { transmute::<_, u8>(r.0) }.hash(&mut hasher) ;
//             unsafe { transmute::<_, u8>(r.1) }.hash(&mut hasher) ;
//         },
//         None => (),
//     };
//     NotNan::unchecked_new(layout.width).hash(&mut hasher);
//     NotNan::unchecked_new(layout.height).hash(&mut hasher);
//     NotNan::unchecked_new(layout.border_left).hash(&mut hasher);
//     NotNan::unchecked_new(layout.border_right).hash(&mut hasher);
//     NotNan::unchecked_new(layout.border_top).hash(&mut hasher);
//     NotNan::unchecked_new(layout.border_bottom).hash(&mut hasher);
//     hasher.finish()
// }

// 根据参数计算uv的step
fn calc_step(csize: f32, img_size: f32, rtype: BorderImageRepeatType) -> f32 {
  let c = csize/img_size;
  if c <= 1.0 {
    return std::f32::INFINITY
  } 
  match rtype {
    BorderImageRepeatType::Repeat => csize / c.round(),
    BorderImageRepeatType::Round => csize / c.ceil(),
    BorderImageRepeatType::Space => csize / c.floor(),
    _ => std::f32::INFINITY
  }
}

// 将指定区域按u切开
fn push_u_arr(point_arr: &mut Polygon, uv_arr: &mut Polygon, index_arr: &mut Vec<u16>,
  p1: u16, p2: u16, p3: u16, p4: u16, u1: f32, v1: f32, u2: f32, v2: f32, step: f32, i: &mut u16){
  let y1 = point_arr[p1 as usize *2 + 1];
  let y2 = point_arr[p2 as usize *2 + 1];
  let mut cur = point_arr[p1 as usize *2] + step;
  let max = point_arr[p3 as usize *2];
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
fn push_v_arr(point_arr: &mut Polygon, uv_arr: &mut Polygon, index_arr: &mut Vec<u16>,
  p1: u16, p2: u16, p3: u16, p4: u16, u1: f32, v1: f32, u2: f32, v2: f32, step: f32, i: &mut u16){
  let x1 = point_arr[p1 as usize *2];
  let x2 = point_arr[p4 as usize *2];
  let mut cur = point_arr[p1 as usize *2 + 1] + step;
  let max = point_arr[p3 as usize *2 + 1];
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


impl_system!{
    BorderImageSys<C> where [C: HalContext + 'static],
    true,
    {
        // MultiCaseListener<Node, BorderImage, CreateEvent>
        // MultiCaseListener<Node, BorderImage, ModifyEvent>
        // MultiCaseListener<Node, BorderImage, DeleteEvent>
        // MultiCaseListener<Node, Layout, ModifyEvent>
        // MultiCaseListener<Node, Opacity, ModifyEvent>
        // MultiCaseListener<Node, WorldMatrixRender, CreateEvent>
        // MultiCaseListener<Node, WorldMatrixRender, ModifyEvent>
        // MultiCaseListener<Node, BorderImageClip, CreateEvent>
        // MultiCaseListener<Node, BorderImageClip, ModifyEvent>
        // MultiCaseListener<Node, BorderImageSlice, CreateEvent>
        // MultiCaseListener<Node, BorderImageSlice, ModifyEvent>
        // MultiCaseListener<Node, BorderImageRepeat, CreateEvent>
        // MultiCaseListener<Node, BorderImageRepeat, ModifyEvent>
        MultiCaseListener<Node, BorderImage, DeleteEvent>
    }
}