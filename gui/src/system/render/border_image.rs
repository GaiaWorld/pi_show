/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
// use std::mem::transmute;
use std::sync::Arc;

use fnv::FnvHashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use map::{ vecmap::VecMap };
use hal_core::*;
use atom::Atom;
// use std::collections::hash_map::DefaultHasher;
// use std::hash::{ Hasher, Hash };

// use ordered_float::NotNan;

use component::user::*;
use component::calc::{Opacity, ZDepth, WorldMatrixRender};
use entity::{Node};
use single::*;
use render::engine::{Engine};
use render::res::{Opacity as ROpacity, SamplerRes, GeometryRes};
use system::util::*;
use system::util::constant::*;
use system::render::shaders::image::{IMAGE_FS_SHADER_NAME, IMAGE_VS_SHADER_NAME};
use util::res_mgr::Res;


lazy_static! {
    static ref BORDER_IMAGE: Atom = Atom::from("border_image");
    static ref CLIP: Atom = Atom::from("clip");
    static ref SLICE: Atom = Atom::from("slice");
    static ref REPEAT: Atom = Atom::from("repeat");
}

pub struct BorderImageSys<C: Context + Share>{
    render_map: VecMap<Item>,
    geometry_dirtys: Vec<usize>,
    mark: PhantomData<C>,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
    default_sampler: Option<Res<SamplerRes<C>>>,
}

impl<C: Context + Share> BorderImageSys<C> {
    pub fn new() -> Self{
        BorderImageSys {
            render_map: VecMap::default(),
            geometry_dirtys: Vec::new(),
            mark: PhantomData,
            rs: Arc::new(RasterState::new()),
            bs: Arc::new(BlendState::new()),
            ss: Arc::new(StencilState::new()),
            ds: Arc::new(DepthState::new()),
            default_sampler: None,
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: Context + Share> Runner<'a> for BorderImageSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, BorderImageClip>,
        &'a MultiCaseImpl<Node, BorderImageSlice>,
        &'a MultiCaseImpl<Node, BorderImageRepeat>,
        &'a MultiCaseImpl<Node, BorderImage<C>>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let map = &mut self.render_map;
        let (layouts, z_depths, clips, slices, repeats, images) = read;
        let (render_objs, engine) = write;
        for id in  self.geometry_dirtys.iter() {
            let item = unsafe { map.get_unchecked_mut(*id) };
            item.position_change = false;
            let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;
            let layout = unsafe { layouts.get_unchecked(*id) };
            let image = unsafe { images.get_unchecked(*id) };
            let slice = slices.get(*id);
            let repeat = repeats.get(*id);
            let clip = clips.get(*id);

            let (positions, uvs, indices) = get_border_image_stream(image, clip, slice, repeat, layout, z_depth - 0.1, Vec::new(), Vec::new(), Vec::new());

            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
            if positions.len() == 0 {
                render_obj.geometry = None;
            } else {
                let mut geometry = create_geometry(&mut engine.gl);
                geometry.set_vertex_count((positions.len()/3) as u32);
                geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false).unwrap();
                geometry.set_attribute(&AttributeName::UV0, 2, Some(uvs.as_slice()), false).unwrap();
                geometry.set_indices_short(indices.as_slice(), false).unwrap();
                render_obj.geometry = Some(Res::new(500, Arc::new(GeometryRes{name: 0, bind: geometry})));
            };
            render_objs.get_notify().modify_event(item.index, "geometry", 0);
        }
        self.geometry_dirtys.clear();
    }

    fn setup(&mut self, _: Self::ReadData, write: Self::WriteData){
        let (_, engine) = write;
        let s = SamplerDesc::default();
        let hash = sampler_desc_hash(&s);
        match engine.res_mgr.get::<SamplerRes<C>>(&hash) {
            Some(r) => self.default_sampler = Some(r.clone()),
            None => {
                let res = SamplerRes::new(hash, engine.gl.create_sampler(Arc::new(s)).unwrap());
                self.default_sampler = Some(engine.res_mgr.create::<SamplerRes<C>>(res));
            }
        };
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BorderImage<C>, CreateEvent> for BorderImageSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, BorderImage<C>>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, Opacity>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs<C>>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (images, z_depths, layouts, opacitys) = read;
        let (render_objs, engine) = write;
        let image = unsafe { images.get_unchecked(event.id) };
        let z_depth = unsafe { z_depths.get_unchecked(event.id) }.0;
        let _layout = unsafe { layouts.get_unchecked(event.id) };
        let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;

        let mut ubos: FnvHashMap<Atom, Arc<Uniforms<C>>> = FnvHashMap::default();
        let defines = Vec::new();

        let mut common_ubo = engine.gl.create_uniforms();
        common_ubo.set_sampler(
            &TEXTURE,
            &(self.default_sampler.as_ref().unwrap().value.clone() as Arc<dyn AsRef<<C as Context>::ContextSampler>>),
            &(image.src.value.clone() as Arc<dyn AsRef<<C as Context>::ContextTexture>>)
        );
        ubos.insert(COMMON.clone(), Arc::new(common_ubo)); // COMMON

        let pipeline = engine.create_pipeline(0, &IMAGE_VS_SHADER_NAME.clone(), &IMAGE_FS_SHADER_NAME.clone(), defines.as_slice(), self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone());
        
        let is_opacity = if opacity < 1.0 {
            false
        }else if let ROpacity::Opaque = image.src.opacity{
            true
        }else {
            false
        };
        let render_obj: RenderObj<C> = RenderObj {
            depth: z_depth - 0.1,
            depth_diff: -0.1,
            visibility: false,
            is_opacity: is_opacity,
            ubos: ubos,
            geometry: None,
            pipeline: pipeline.clone(),
            context: event.id,
            defines: defines,
        };

        let notify = render_objs.get_notify();
        let index = render_objs.insert(render_obj, Some(notify));
        self.render_map.insert(event.id, Item{index: index, position_change: true});
        self.geometry_dirtys.push(event.id);
    }
}

// 修改渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BorderImage<C>, ModifyEvent> for BorderImageSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, BorderImage<C>>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (opacitys, images) = read;
        if let Some(item) = self.render_map.get_mut(event.id) {
            let opacity = unsafe { opacitys.get_unchecked(event.id).0 };
            let image = unsafe { images.get_unchecked(event.id) };

            // 图片改变， 更新common_ubo中的纹理
            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
            let common_ubo = render_obj.ubos.get_mut(&COMMON).unwrap();
            let common_ubo = Arc::make_mut(common_ubo);
            common_ubo.set_sampler(
                &TEXTURE,
                &(self.default_sampler.as_ref().unwrap().value.clone() as Arc<dyn AsRef<<C as Context>::ContextSampler>>),
                &(image.src.value.clone() as Arc<dyn  AsRef<<C as Context>::ContextTexture>>)
            );

            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(event.id);
            }

            // 图片改变， 修改渲染对象的不透明属性
            let index = item.index;
            self.change_is_opacity(event.id, opacity, image, index, render_objs);
 
        }
    }
}

// 删除渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BorderImage<C>, DeleteEvent> for BorderImageSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, write: Self::WriteData){
        let item = self.render_map.remove(event.id).unwrap();
        let notify = write.get_notify();
        write.remove(item.index, Some(notify));
        if item.position_change == true {
            self.geometry_dirtys.remove_item(&event.id);
        }
    }
}

//布局修改， 需要重新计算顶点
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Layout, ModifyEvent> for BorderImageSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        if let Some(item) = self.render_map.get_mut(event.id) {
            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(event.id);
            }
        };
    }
}

//不透明度变化， 修改渲染对象的is_opacity属性
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for BorderImageSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, BorderImage<C>>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (opacitys, images) = read;
        if let Some(item) = self.render_map.get(event.id) {
            let opacity = unsafe { opacitys.get_unchecked(event.id).0 };
            let image = unsafe { images.get_unchecked(event.id) };
            let index = item.index;
            self.change_is_opacity(event.id, opacity, image, index, write);
        }
    }
}

type MatrixRead<'a> = &'a MultiCaseImpl<Node, WorldMatrixRender>;

impl<'a, C: Context + Share> MultiCaseListener<'a, Node, WorldMatrixRender, ModifyEvent> for BorderImageSys<C>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read, render_objs);
    }
}

impl<'a, C: Context + Share> MultiCaseListener<'a, Node, WorldMatrixRender, CreateEvent> for BorderImageSys<C>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read, render_objs);
    }
}

impl<'a, C: Context + Share> BorderImageSys<C> {
    fn change_is_opacity(&mut self, _id: usize, opacity: f32, image: &BorderImage<C>, index: usize, render_objs: &mut SingleCaseImpl<RenderObjs<C>>){
        let is_opacity = if opacity < 1.0 {
            false
        }else if let ROpacity::Opaque = image.src.opacity{
            true
        }else {
            false
        };

        let notify = render_objs.get_notify();
        unsafe { render_objs.get_unchecked_write(index, &notify).set_is_opacity(is_opacity)};
    }

    fn modify_matrix(
        &self,
        id: usize,
        world_matrixs: &MultiCaseImpl<Node, WorldMatrixRender>,
        render_objs: &mut SingleCaseImpl<RenderObjs<C>>
    ){
        if let Some(item) = self.render_map.get(id) {
            let world_matrix = unsafe { world_matrixs.get_unchecked(id) };
            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };

            // 渲染物件的顶点不是一个四边形， 保持其原有的矩阵
            let ubos = &mut render_obj.ubos;
            let slice: &[f32; 16] = world_matrix.0.as_ref();
            Arc::make_mut(ubos.get_mut(&WORLD).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
            debug_println!("border_image, id: {}, world_matrix: {:?}", render_obj.context, &slice[0..16]);
            render_objs.get_notify().modify_event(item.index, "ubos", 0);
        }
    }
}

struct Item {
    index: usize,
    position_change: bool,
}

unsafe impl<C: Context + Share> Sync for BorderImageSys<C>{}
unsafe impl<C: Context + Share> Send for BorderImageSys<C>{}


pub fn get_border_image_stream<'a, C: Context + 'static + Send + Sync> (
  img: &BorderImage<C>,
  clip: Option<&BorderImageClip>,
  slice: Option<&BorderImageSlice>,
  repeat: Option<&BorderImageRepeat>,
  layout: &Layout, z: f32, mut point_arr: Polygon, mut uv_arr: Polygon, mut index_arr: Vec<u16>) -> (Polygon, Polygon, Vec<u16>){
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

    // debug_println!("start 111111: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
    //  p1, p2, w, h, left, right, top, bottom, "UV::", uv1, uv2, uvw, uvh, uv_left, uv_right, uv_top, uv_bottom);
    // TODO 在仅使用左或上的边框时， 应该优化成8个顶点
    // 先将16个顶点和uv放入数组，记录偏移量
    let mut pi = (point_arr.len() / 3)  as u16;
    // 左上的4个点
    let p_x1_y1 = push_vertex(&mut point_arr, &mut uv_arr, p1.x, p1.y, z, uv1.x, uv1.y, &mut pi);
    let p_x1_top = push_vertex(&mut point_arr, &mut uv_arr, p1.x, top, z, uv1.x, uv_top, &mut pi);
    let p_left_top = push_vertex(&mut point_arr, &mut uv_arr, left, top, z, uv_left, uv_top, &mut pi);
    let p_left_y1 = push_vertex(&mut point_arr, &mut uv_arr, left, p1.y, z, uv_left, uv1.y, &mut pi);
    push_quad(&mut index_arr, p_x1_y1, p_x1_top, p_left_top, p_left_y1);

    // 左下的4个点
    let p_x1_bottom = push_vertex(&mut point_arr, &mut uv_arr, p1.x, bottom, z, uv1.x, uv_bottom, &mut pi);
    let p_x1_y2 = push_vertex(&mut point_arr, &mut uv_arr, p1.x, p2.y, z, uv1.x, uv2.y, &mut pi);
    let p_left_y2 = push_vertex(&mut point_arr, &mut uv_arr, left, p2.y, z, uv_left, uv2.y, &mut pi);
    let p_left_bottom = push_vertex(&mut point_arr, &mut uv_arr, left, bottom, z, uv_left, uv_bottom, &mut pi);
    push_quad(&mut index_arr, p_x1_bottom, p_x1_y2, p_left_y2, p_left_bottom);

    // 右下的4个点
    let p_right_bottom = push_vertex(&mut point_arr, &mut uv_arr, right, bottom, z, uv_right, uv_bottom, &mut pi);
    let p_right_y2 = push_vertex(&mut point_arr, &mut uv_arr, right, p2.y, z, uv_right, uv2.y, &mut pi);
    let p_x2_y2 = push_vertex(&mut point_arr, &mut uv_arr, p2.x, p2.y, z, uv2.x, uv2.y, &mut pi);
    let p_x2_bottom = push_vertex(&mut point_arr, &mut uv_arr, p2.x, bottom, z, uv2.x, uv_bottom, &mut pi);
    push_quad(&mut index_arr, p_right_bottom, p_right_y2, p_x2_y2, p_x2_bottom);

    // 右上的4个点
    let p_right_y1 = push_vertex(&mut point_arr, &mut uv_arr, right, p1.y, z, uv_right, uv1.y, &mut pi);
    let p_right_top = push_vertex(&mut point_arr, &mut uv_arr, right, top, z, uv_right, uv_top, &mut pi);
    let p_x2_top = push_vertex(&mut point_arr, &mut uv_arr, p2.x, top, z, uv2.x, uv_top, &mut pi);
    let p_x2_y1 = push_vertex(&mut point_arr, &mut uv_arr, p2.x, p1.y, z, uv2.x, uv1.y, &mut pi);
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
    p_left_y1, p_left_top, p_right_top, p_right_y1, z,
    uv_left, uv1.y, uv_right, uv_top, ustep, &mut pi); // 上边
    push_u_arr(&mut point_arr, &mut uv_arr, &mut index_arr,
    p_left_bottom, p_left_y2, p_right_y2, p_right_bottom, z,
    uv_left, uv_bottom, uv_right, uv2.y, ustep, &mut pi); // 下边
    push_v_arr(&mut point_arr, &mut uv_arr, &mut index_arr,
    p_x1_top, p_x1_bottom, p_left_bottom, p_left_top, z,
    uv1.x, uv_top, uv_left, uv_bottom, vstep, &mut pi); // 左边
    push_v_arr(&mut point_arr, &mut uv_arr, &mut index_arr,
    p_right_top, p_right_bottom, p_x2_bottom, p_x2_top, z,
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
fn push_vertex(point_arr: &mut Polygon, uv_arr: &mut Polygon, x: f32, y: f32, z: f32, u: f32, v: f32, i: &mut u16) -> u16 {
    point_arr.extend_from_slice(&[x, y, z]);
    uv_arr.extend_from_slice(&[u, v]);
    let r = *i;
    *i += 1;
    r
}
// 将四边形放进数组中
fn push_quad(index_arr: &mut Vec<u16>, p1: u16, p2: u16, p3: u16, p4: u16){
    index_arr.extend_from_slice(&[p1, p2, p3, p1, p3, p4]);
}
 
// fn border_image_geo_hash<C: Context + Share>(
//     img: &BorderImage<C>,
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
  p1: u16, p2: u16, p3: u16, p4: u16, z: f32, u1: f32, v1: f32, u2: f32, v2: f32, step: f32, i: &mut u16){
  let y1 = point_arr[p1 as usize *3 + 1];
  let y2 = point_arr[p2 as usize *3 + 1];
  let mut cur = point_arr[p1 as usize *3] + step;
  let max = point_arr[p3 as usize *3];
  let mut pt1 = p1;
  let mut pt2 = p2;
  while cur < max {
    let i3 = push_vertex(point_arr, uv_arr, cur, y2, z, u2, v2, i);
    let i4 = push_vertex(point_arr, uv_arr, cur, y1, z, u2, v1, i);
    push_quad(index_arr, pt1, pt2, i3, i4);
    // 因为uv不同，新插入2个顶点
    pt1 = push_vertex(point_arr, uv_arr, cur, y1, z, u1, v1, i);
    pt2 = push_vertex(point_arr, uv_arr, cur, y2, z, u1, v2, i);
    cur += step;
  }
  push_quad(index_arr, pt1, pt2, p3, p4);
}
// 将指定区域按v切开
fn push_v_arr(point_arr: &mut Polygon, uv_arr: &mut Polygon, index_arr: &mut Vec<u16>,
  p1: u16, p2: u16, p3: u16, p4: u16, z: f32, u1: f32, v1: f32, u2: f32, v2: f32, step: f32, i: &mut u16){
  let x1 = point_arr[p1 as usize *3];
  let x2 = point_arr[p4 as usize *3];
  let mut cur = point_arr[p1 as usize *3 + 1] + step;
  let max = point_arr[p3 as usize *3 + 1];
  let mut pt1 = p1;
  let mut pt4 = p4;
  while cur < max {
    let i2 = push_vertex(point_arr, uv_arr, x1, cur, z, u1, v2, i);
    let i3 = push_vertex(point_arr, uv_arr, x2, cur, z, u2, v2, i);
    push_quad(index_arr, pt1, i2, i3, pt4);
    // 因为uv不同，新插入2个顶点
    pt1 = push_vertex(point_arr, uv_arr, x1, cur, z, u1, v1, i);
    pt4 = push_vertex(point_arr, uv_arr, x2, cur, z, u2, v1, i);
    cur += step;
  }
  push_quad(index_arr, pt1, p2, p3, pt4);
}


impl_system!{
    BorderImageSys<C> where [C: Context + Share],
    true,
    {
        MultiCaseListener<Node, BorderImage<C>, CreateEvent>
        MultiCaseListener<Node, BorderImage<C>, ModifyEvent>
        MultiCaseListener<Node, BorderImage<C>, DeleteEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
        MultiCaseListener<Node, WorldMatrixRender, CreateEvent>
        MultiCaseListener<Node, WorldMatrixRender, ModifyEvent>
    }
}