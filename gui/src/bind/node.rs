use std::{
  sync::Arc,
  usize::MAX as UMAX,
  f32::INFINITY as FMAX,
  mem::transmute,
};

use stdweb::unstable::TryInto;
use stdweb::web::html_element::{ImageElement, CanvasElement};

use ecs::{World, Lend, LendMut, MultiCaseImpl, SingleCaseImpl};
use ecs::idtree::{IdTree, InsertType};
use hal_core::*;
use hal_webgl::*;
use atom::Atom;
use octree::intersects;
use cg2d::{include_quad2, InnOuter};

use component::user::*;
use component::calc::{ Enable, ByOverflow, ZDepth};
use single::oct::Oct;
use single::{ OverflowClip};
use entity::Node;
use render::engine::Engine;
use render::res::{TextureRes};
use layout::*;
use Z_MAX;


fn create(world: &World) -> usize {
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.lend_mut();
    let node = world.create_entity::<Node>();
    let border_radius = world.fetch_multi::<Node, BorderRadius>().unwrap();
    let border_radius = border_radius.lend_mut();
    border_radius.insert(node, BorderRadius{x: LengthUnit::Pixel(0.0), y: LengthUnit::Pixel(0.0)});
    idtree.create(node);
    node
}

#[allow(unused_attributes)]
#[no_mangle]
fn insert_child(world: u32, child: u32, parent: u32, index: usize){
    let world = unsafe {&mut *(world as usize as *mut World)};
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.lend_mut();
    let notify = idtree.get_notify();
    idtree.insert_child(child as usize, parent as usize, index, Some(&notify));
}

//创建容器节点， 容器节点可设置背景颜色
#[allow(unused_attributes)]
#[no_mangle]
pub fn create_node(world: u32) -> u32{
    let world = unsafe {&mut *(world as usize as *mut World)};
    let node = create(world);
    debug_println!("create_node, node:{}", node);
    node as u32
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn create_text_node(world: u32) -> u32 {
    let world = unsafe {&mut *(world as usize as *mut World)};
    let node = create(world);
    debug_println!("create_text_node, node:{}", node);
    node as u32
}

//创建图片节点
#[allow(unused_attributes)]
#[no_mangle]
pub fn create_image_node(world: u32) -> u32{
    let world = unsafe {&mut *(world as usize as *mut World)};
    let node = create(world);
    debug_println!("create_image_node, node:{}", node);
    node as u32
}

// 在尾部插入子节点
#[allow(unused_attributes)]
#[no_mangle]
pub fn append_child(world: u32, child: u32, parent: u32){
    let world = unsafe {&mut *(world as usize as *mut World)};
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.lend_mut();
    let notify = idtree.get_notify();
    idtree.insert_child(child as usize, parent as usize, UMAX, Some(&notify));
    debug_println!("append_child"); 
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn insert_before(world: u32, child: u32, brother: u32){
    let world = unsafe {&mut *(world as usize as *mut World)};
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.lend_mut();
    let notify = idtree.get_notify();
    idtree.insert_brother(child as usize, brother as usize, InsertType::Front, Some(&notify));
    debug_println!("insert_before"); 
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn remove_child(world: u32, node: u32){
    let world = unsafe {&mut *(world as usize as *mut World)};
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.lend_mut();
    let notify = idtree.get_notify();
    idtree.remove(node as usize, Some(&notify));
    debug_println!("remove_child");  
}

// __jsObj 文字字符串
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_content(world: u32, node: u32){
    let value: String = js!(return __jsObj;).try_into().unwrap();
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut World)};
    let text = world.fetch_multi::<Node, Text>().unwrap();
    text.lend_mut().insert(node as usize, Text(Arc::new(value)));
}

// __jsObj: image, __jsObj1: image_name(String)
// 设置图片的src
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_src(world: u32, node: u32, opacity: u8, compress: u8){
  let name: String = js!{return __jsObj1}.try_into().unwrap();
  let name = Atom::from(name);
  let node = node as usize;
  let world = unsafe {&mut *(world as usize as *mut World)};
  let engine = world.fetch_single::<Engine<WebGLContextImpl>>().unwrap();
  let engine = engine.lend_mut();
  let (width, height, texture) = match engine.res_mgr.textures.get(&name) {
      Some(res) => {
        (res.width as u32, res.height as u32, res.clone())
      },
      None => {
        let width: u32 = js!{return __jsObj.width}.try_into().unwrap();
        let height: u32 = js!{return __jsObj.height}.try_into().unwrap();

        let texture = match TryInto::<ImageElement>::try_into(js!{return __jsObj}) {
          Ok(r) => engine.gl.create_texture_2d_webgl(0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &WebGLTextureData::Image(r)),
          Err(_s) => match TryInto::<CanvasElement>::try_into(js!{return __jsObj}){
            Ok(r) => engine.gl.create_texture_2d_webgl(0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &WebGLTextureData::Canvas(r)),
            Err(_s) => panic!("set_src error"),
          },
        };

        // gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MAG_FILTER, WebGLRenderingContext::NEAREST as i32);
        // gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MIN_FILTER, WebGLRenderingContext::NEAREST as i32);
        let res = engine.res_mgr.textures.create(TextureRes::new(name, width as usize, height as usize, unsafe{transmute(opacity)}, unsafe{transmute(compress)}, texture.unwrap()) );
        (width, height, res)
      },
  };

  
  let yg_nodes = world.fetch_multi::<Node, YgNode>().unwrap();
  let yg_nodes = yg_nodes.lend_mut();
  let yoga = unsafe {yg_nodes.get_unchecked(node)};
  match yoga.get_width().unit {
    YGUnit::YGUnitUndefined | YGUnit::YGUnitAuto => yoga.set_width(width as f32),
    _ => (),
  };
  match yoga.get_height().unit {
      YGUnit::YGUnitUndefined | YGUnit::YGUnitAuto => yoga.set_height(height as f32),
      _ => (),
  };
  
  let images = world.fetch_multi::<Node, Image<WebGLContextImpl>>().unwrap();
  let image = images.lend_mut();
  image.insert(node, Image{src: texture});

  debug_println!("set_src"); 
}

// __jsObj: image, __jsObj1: image_name(String)
// 设置图片的src

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_border_src(world: u32, node: u32, opacity: u8, compress: u8){
  let name: String = js!{return __jsObj1}.try_into().unwrap();
  let name = Atom::from(name);
  let node = node as usize;
  let world = unsafe {&mut *(world as usize as *mut World)};
  let engine = world.fetch_single::<Engine<WebGLContextImpl>>().unwrap();
  let engine = engine.lend_mut();
  let texture = match engine.res_mgr.textures.get(&name) {
      Some(res) => {
        res.clone()
      },
      None => {
        let width: u32 = js!{return __jsObj.width}.try_into().unwrap();
        let height: u32 = js!{return __jsObj.height}.try_into().unwrap();

        let texture = match TryInto::<ImageElement>::try_into(js!{return __jsObj}) {
          Ok(r) => engine.gl.create_texture_2d_webgl(0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &WebGLTextureData::Image(r)),
          Err(_s) => match TryInto::<CanvasElement>::try_into(js!{return __jsObj}){
            Ok(r) => engine.gl.create_texture_2d_webgl(0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &WebGLTextureData::Canvas(r)),
            Err(_s) => panic!("set_src error"),
          },
        };

        // gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MAG_FILTER, WebGLRenderingContext::NEAREST as i32);
        // gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MIN_FILTER, WebGLRenderingContext::NEAREST as i32);
        let res = engine.res_mgr.textures.create(TextureRes::new(name, width as usize, height as usize, unsafe{transmute(opacity)}, unsafe{transmute(compress)}, texture.unwrap()) );
        res
      },
  };
  
  let images = world.fetch_multi::<Node, BorderImage<WebGLContextImpl>>().unwrap();
  let image = images.lend_mut();
  image.insert(node, BorderImage{src: texture});

  debug_println!("set_border_src"); 
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn offset_top(world: u32, node: u32) -> f32 {
    let world = unsafe {&mut *(world as usize as *mut World)};
    let layout = world.fetch_multi::<Node, Layout>().unwrap();
    unsafe {layout.lend().get_unchecked(node as usize)}.top
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn offset_left(world: u32, node: u32) -> f32 {
    let world = unsafe {&mut *(world as usize as *mut World)};
    let layout = world.fetch_multi::<Node, Layout>().unwrap();
    unsafe {layout.lend().get_unchecked(node as usize)}.left
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn offset_width(world: u32, node: u32) -> f32 {
    let world = unsafe {&mut *(world as usize as *mut World)};
    let layout = world.fetch_multi::<Node, Layout>().unwrap();
    unsafe {layout.lend().get_unchecked(node as usize)}.width
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn offset_height(world: u32, node: u32) -> f32 {
    let world = unsafe {&mut *(world as usize as *mut World)};
    let layout = world.fetch_multi::<Node, Layout>().unwrap();
    unsafe {layout.lend().get_unchecked(node as usize)}.height
}

// #[no_mangle]
// pub fn offset_document(world: u32, node: u32) {
//   let node = node as usize;
//   let world = unsafe {&mut *(world as usize as *mut World)};
//   let node = world.component_mgr.node._group.get(node);
//   let layout = node.get_layout();
//   let world_matrix = &world.component_mgr.node.world_matrix._group.get(node.world_matrix).owner;

//   let left_top = Vector4::new(-(layout.width - layout.border -layout.padding_left)/2.0, -(layout.height - layout.border -layout.padding_top)/2.0, 1.0, 1.0);
//   let left_top = (world_matrix.0)*left_top;

//   js!{
//     __jsObj.left = @{left_top.x};
//     __jsObj.top = @{left_top.y}
//   }
// }

// #[no_mangle]
// pub fn set_event_type(world: u32, node: u32, ty: u8) {
//   let node = node as usize;
//   let world = unsafe {&mut *(world as usize as *mut World)};
//   let node_ref = world.component_mgr.get_node_mut(node);
//   node_ref.set_event_type(ty);
// }

//content宽高的累加值
#[allow(unused_attributes)]
#[no_mangle]
pub fn content_box(world: u32, node: u32) {
    let world = unsafe {&mut *(world as usize as *mut World)};
    let layout = world.fetch_multi::<Node, Layout>().unwrap();
    let layout = layout.lend();
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.borrow();
    let (mut left, mut right, mut top, mut bottom) = (FMAX, 0.0, FMAX, 0.0);
    for (id, _) in idtree.iter(unsafe {idtree.get_unchecked(node as usize)}.children.head) {
      let l = unsafe {layout.get_unchecked(id)};
      let r = l.left + l.width;
      let b = l.top + l.height;
      if l.left < left {
        left = l.left;
      }
      if r > right {
        right = r;
      }
      if b > bottom {
        bottom = b;
      }
      if l.top < top {
        top = l.top;
      }
    }
    // js!{
    //   __jsObj.width = @{right - left};
    //   __jsObj.height = @{bottom - top}
    // }
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn query(world: u32, x: f32, y: f32)-> u32{
    let world = unsafe {&mut *(world as usize as *mut World)};    
    let octree = world.fetch_single::<Oct>().unwrap();
    let enables = world.fetch_multi::<Node, Enable>().unwrap();
    let overflow_clip = world.fetch_single::<OverflowClip>().unwrap();
    let by_overflows = world.fetch_multi::<Node, ByOverflow>().unwrap();
    let z_depths = world.fetch_multi::<Node, ZDepth>().unwrap();

    let octree = octree.lend();
    let enables = enables.lend();
    let overflow_clip = overflow_clip.lend();
    let by_overflows = by_overflows.lend();
    let z_depths = z_depths.lend();

    let aabb = Aabb3::new(Point3::new(x,y,-Z_MAX), Point3::new(x,y,Z_MAX));
    let mut args = AbQueryArgs::new(enables, by_overflows, z_depths, overflow_clip, aabb.clone(), 0);
    octree.query(&aabb, intersects, &mut args, ab_query_func);
    args.result as u32
}

/// aabb的查询函数的参数
struct AbQueryArgs<'a> {
  enables: &'a MultiCaseImpl<Node, Enable>,
  by_overflows: &'a MultiCaseImpl<Node, ByOverflow>,
  z_depths: &'a MultiCaseImpl<Node, ZDepth>,
  overflow_clip: &'a SingleCaseImpl<OverflowClip>,
  aabb: Aabb3,
  ev_type: u32,
  max_z: f32,
  result: usize,
}
impl<'a> AbQueryArgs<'a> {
  pub fn new(
    enables: &'a MultiCaseImpl<Node, Enable>,
    by_overflows: &'a MultiCaseImpl<Node, ByOverflow>,
    z_depths: &'a MultiCaseImpl<Node, ZDepth>,
    overflow_clip: &'a SingleCaseImpl<OverflowClip>,
    aabb: Aabb3,
    ev_type: u32,
  ) -> AbQueryArgs<'a> {
    AbQueryArgs{
      enables,
      by_overflows,
      z_depths,
      overflow_clip,
      aabb: aabb,
      ev_type: ev_type,
      max_z: -Z_MAX,
      result: 0,
    }
  }
}
/// aabb的ab查询函数, aabb的oct查询函数应该使用intersects
fn ab_query_func(arg: &mut AbQueryArgs, _id: usize, aabb: &Aabb3, bind: &usize) {
  // debug_println!("ab_query_func----------------------------{}, {:?}, {:?}", *bind, aabb, arg.aabb);
  if intersects(&arg.aabb, aabb) {
    // debug_println!("bind----------------------------{}", *bind);
    let enable = unsafe { arg.enables.get_unchecked(*bind) }.0;
    // debug_println!("enable----------------------------{}", enable);
    //如果enable true 表示不接收事件
    match enable {
      true => (),
      false => return,
    };

    let z_depth = unsafe { arg.z_depths.get_unchecked(*bind) }.0;
    // debug_println!("----------------------------z_depth: {}, arg.max_z: {}", z_depth, arg.max_z);
    // 取最大z的node
    if z_depth > arg.max_z {
      let by_overflow = unsafe { arg.by_overflows.get_unchecked(*bind) }.0;
      // debug_println!("by_overflow---------------------------{}",by_overflow);
      // 检查是否有裁剪，及是否在裁剪范围内
      if by_overflow == 0 || in_overflow(&arg.overflow_clip, by_overflow, aabb.min.x, aabb.min.y) {
        arg.result = *bind;
        arg.max_z = z_depth;
      }
    }
    // // 判断类型是否有相交
    // if (node.event_type as u32) & arg.ev_type != 0 {
    //     // 取最大z的node
    //     if node.z_depth > arg.max_z {
    //       // 检查是否有裁剪，及是否在裁剪范围内
    //       if node.by_overflow == 0 || in_overflow(&arg.mgr, node.by_overflow, aabb.min.x, aabb.min.y) {
    //         arg.result = *bind;
    //         arg.max_z = node.z_depth;
    //        }
    //     }
    // }
  }
}

/// 检查坐标是否在裁剪范围内， 直接在裁剪面上检查
fn in_overflow(overflow_clip: &SingleCaseImpl<OverflowClip>, by_overflow: usize, x: f32, y: f32) -> bool{
  let xy = Point2::new(x, y);
  // debug_println!("overflow_clip len---------------------------{}",overflow_clip.id_vec.len());
  for i in 0..overflow_clip.id_vec.len() {
    // debug_println!("i + 1---------------------------{}",i + 1);
    // debug_println!("overflow_clip.id_vec[i]---------------------------{}",overflow_clip.id_vec[i]);
    if by_overflow & (i + 1) != 0 && overflow_clip.id_vec[i] > 0 {
      let p = &overflow_clip.clip[i];
      // debug_println!("p---------------------------{:?}",p);
      match include_quad2(&xy, &p[0], &p[1], &p[2], &p[3]) {
        InnOuter::Inner => (),
        _ => return false
      }
    }
  }
  return true
}
