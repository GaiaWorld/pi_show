use std::mem::transmute;
use std::f32::MAX as FMAX;

use cg::query::{include_quad2, InnOuter};
use cg::octree::intersects;
use cg::{Aabb3, Point3, Point2, Vector4};
use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::unstable::TryInto;

use wcs::world::World;
use atom::Atom;

use world_doc::{Z_MAX, WorldDocMgr};
use world_doc::component::style::element::{ElementId, Text, Element, TextWriteRef, Image, ImageWriteRef};
use world_doc::component::node::{InsertType, NodeWriteRef};
use render::res::TextureRes;
use layout::YGUnit;

#[no_mangle]
pub fn append_child(world: u32, node_id: u32, child_id: u32){
    let node_id = node_id as usize;
    let child_id = child_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    node_ref.insert_child_with_id(child_id, InsertType::Back);
    debug_println!("append_child"); 
}

#[no_mangle]
pub fn insert_before(world: u32, node_id: u32, child_id: u32, brother_id: u32, brother_index: u32){
    let node_id = node_id as usize;
    let child_id = child_id as usize;
    let brother_id = brother_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let brother_qid = world.component_mgr.node._group.get(brother_id).qid;
    let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    node_ref.insert_child_with_id(child_id, InsertType::ToFront(brother_index as usize, brother_qid));
    debug_println!("insert_before"); 
}

#[no_mangle]
pub fn remove_child(world: u32, node_id: u32, child_id: u32){
    let node_id = node_id as usize;
    let child_id = child_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    node_ref.remove_child(child_id);
    debug_println!("remove_child");  
}

#[no_mangle]
pub fn set_text_content(world: u32, node_id: u32){
    let value: String = js!(return __jsObj;).try_into().unwrap();
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let element_id = world.component_mgr.node._group.get(node_id).element.clone();
    match element_id {
        ElementId::Text(text_id) => {
            if text_id == 0 {
                let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr);
                let mut text = Text::default();
                text.value = value;
                node_ref.set_element(Element::Text(text));
            } else {
                let mut text_ref = TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr);
                text_ref.set_value(value);
            }
            debug_println!("set_text_content");
        },
        _ => (),
    }
}

// __jsObj: image, __jsObj1: image_name(String)
// 设置图片的src
#[no_mangle]
pub fn set_src(world: u32, node_id: u32, opacity: u8, compress: u8){
    let name: String = js!{return __jsObj1}.try_into().unwrap();
    let name = Atom::from(name);
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let (width, height, texture) = match world.component_mgr.world_2d.component_mgr.engine.res_mgr.textures.get(&name) {
        Some(res) => {
          (res.width as u32, res.height as u32, Box::into_raw(Box::new(res)) as u32)
        },
        None => {
          let gl = world.component_mgr.world_2d.component_mgr.engine.gl.clone();
          let texture = match gl.create_texture() {
              Some(v) => v,
              None => panic!("create_texture is None"),
          };
          gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&texture));
          js!{
            @{&gl}.texImage2D(@{&gl}.TEXTURE_2D, 0, @{&gl}.RGBA, @{&gl}.RGBA, @{&gl}.UNSIGNED_BYTE, __jsObj);
          };
          let width: u32 = js!{return __jsObj.width}.try_into().unwrap();
          let height: u32 = js!{return __jsObj.height}.try_into().unwrap();
          gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MAG_FILTER, WebGLRenderingContext::NEAREST as i32);
          gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MIN_FILTER, WebGLRenderingContext::NEAREST as i32);
          let res = world.component_mgr.world_2d.component_mgr.engine.res_mgr.textures.create(TextureRes::new(name, width as usize, height as usize, unsafe{transmute(opacity)}, unsafe{transmute(compress)}, texture, gl.clone()) );
          (width, height, Box::into_raw(Box::new( res)) as u32)
        },
    };

    let (yoga, element_id) = {
      let node = world.component_mgr.node._group.get(node_id);
      (node.yoga, node.element.clone())
    };
    match element_id {
        ElementId::Image(image_id) => {
            if image_id == 0 {
              let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
              let mut image = Image::default();
              image.src = texture as usize;
              node_ref.set_element(Element::Image(image));
            } else {
              let mut image_ref = ImageWriteRef::new(image_id, world.component_mgr.node.element.image.to_usize(), &mut world.component_mgr);
              image_ref.set_src(texture as usize);
            }
        },
        _ => println!("it's not image, node_id: {}", node_id),
    }

    match yoga.get_width().unit {
        YGUnit::YGUnitUndefined | YGUnit::YGUnitAuto => yoga.set_width(width as f32),
        _ => (),
    };
    match yoga.get_height().unit {
        YGUnit::YGUnitUndefined | YGUnit::YGUnitAuto => yoga.set_height(height as f32),
        _ => (),
    };
    debug_println!("set_src"); 
}

#[no_mangle]
pub fn offset_top(world: u32, node_id: u32) -> f32 {
  let node_id = node_id as usize;
  let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
  world.component_mgr.node._group.get(node_id).yoga.get_layout().top
}

#[no_mangle]
pub fn offset_left(world: u32, node_id: u32) -> f32 {
  let node_id = node_id as usize;
  let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
  world.component_mgr.node._group.get(node_id).yoga.get_layout().left
}

#[no_mangle]
pub fn offset_width(world: u32, node_id: u32) -> f32 {
  let node_id = node_id as usize;
  let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
  world.component_mgr.node._group.get(node_id).yoga.get_layout().width
}

#[no_mangle]
pub fn offset_height(world: u32, node_id: u32) -> f32 {
  let node_id = node_id as usize;
  let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
  world.component_mgr.node._group.get(node_id).yoga.get_layout().height
}

#[no_mangle]
pub fn offset_document(world: u32, node_id: u32) {
  let node_id = node_id as usize;
  let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
  let node = world.component_mgr.node._group.get(node_id);
  let layout = node.get_layout();
  let world_matrix = &world.component_mgr.node.world_matrix._group.get(node.world_matrix).owner;

  let left_top = Vector4::new(-(layout.width - layout.border -layout.padding_left)/2.0, -(layout.height - layout.border -layout.padding_top)/2.0, 1.0, 1.0);
  let left_top = (world_matrix.0)*left_top;

  js!{
    __jsObj.left = @{left_top.x};
    __jsObj.top = @{left_top.y}
  }
}

//content宽高的累加值
#[no_mangle]
pub fn content_box(world: u32, node_id: u32) {
  let node_id = node_id as usize;
  let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
  let node = world.component_mgr.node._group.get(node_id);
  let mut child = node.childs.get_first();
  let (mut left, mut right, mut top, mut bottom) = (FMAX, 0.0, FMAX, 0.0);

  loop {
    if child == 0 {
        break;
    }
    let node_id = {
        let v = unsafe{ world.component_mgr.node_container.get_unchecked(child) };
        child = v.next;
        v.elem.clone()
    };
    let node = world.component_mgr.node._group.get(node_id);
    let layout = &node.layout;
    let right_ = layout.left + layout.width;
    let bottom_ = layout.top + layout.height;
    if layout.left < left {
      left = layout.left;
    }
    if right_ > right {
      right = right_;
    }
    if bottom_ > bottom {
      bottom = bottom_;
    }
    if layout.top < top {
      top = layout.top;
    }
  }

  js!{
    __jsObj.width = @{right - left};
    __jsObj.height = @{bottom - top}
  }
}


#[no_mangle]
pub fn query(world: u32, x: f32, y: f32, ty: u32)-> u32{
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let aabb = Aabb3::new(Point3::new(x,y,-Z_MAX), Point3::new(x,y,Z_MAX));
    let mut args = AbQueryArgs::new(&world.component_mgr, aabb.clone(), ty);
    world.component_mgr.octree.query(&aabb, intersects, &mut args, ab_query_func);
    args.result as u32
}
/// aabb的查询函数的参数
struct AbQueryArgs<'a> {
  mgr: &'a WorldDocMgr,
  aabb: Aabb3<f32>,
  ev_type: u32,
  max_z: f32,
  result: usize,
}
impl<'a> AbQueryArgs<'a> {
  pub fn new(mgr: &WorldDocMgr, aabb: Aabb3<f32>, ev_type: u32) -> AbQueryArgs {
    AbQueryArgs{
      mgr: mgr,
      aabb: aabb,
      ev_type: ev_type,
      max_z: -Z_MAX,
      result: 0,
    }
  }
}
/// aabb的ab查询函数, aabb的oct查询函数应该使用intersects
fn ab_query_func(arg: &mut AbQueryArgs, _id: usize, aabb: &Aabb3<f32>, bind: &usize) {
  if intersects(&arg.aabb, aabb) {
    let node = arg.mgr.node._group.get(*bind);
    // 判断类型是否有相交
    if (node.event_type as u32) & arg.ev_type != 0 {
        // 取最大z的node
        if node.z_depth > arg.max_z {
          // 检查是否有裁剪，及是否在裁剪范围内
          if node.by_overflow == 0 || in_overflow(&arg.mgr, node.by_overflow, aabb.min.x, aabb.min.y) {
            arg.result = *bind;
            arg.max_z = node.z_depth;
           }
        }
    }
  }
}

/// 检查坐标是否在裁剪范围内， 直接在裁剪面上检查
fn in_overflow(mgr: &WorldDocMgr, by_overflow: usize, x: f32, y: f32) -> bool{
  let xy = Point2::new(x, y);
  for i in 0..mgr.world_2d.component_mgr.overflow.0.len() {
    if by_overflow & (i + 1) != 0 && mgr.world_2d.component_mgr.overflow.0[i] > 0 {
      let p = &mgr.world_2d.component_mgr.overflow.1[i];
      match include_quad2(&xy, &p[0], &p[1], &p[2], &p[3]) {
        InnOuter::Inner => (),
        _ => return false
      }
    }
  }
  return true
}
