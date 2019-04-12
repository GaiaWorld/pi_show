use atom::Atom;
use cg::query::{include_quad2, InnOuter};
use cg::octree::intersects;

use wcs::world::World;

use world_doc::{Z_MAX, WorldDocMgr};
// use world_doc::{ WorldDocMgr};
use world_doc::component::style::element::{ElementId, Text, Element, TextWriteRef, Image, ImageWriteRef};
use world_doc::component::node::{InsertType, NodeWriteRef};
use cg::{Aabb3, Point3, Point2};
use render::res::TextureRes;

// use bind::{Pointer};

// // #[no_mangle] pub fn get_style(world: u32, node_id: u32) -> u32 {
// //     
// //     let world = node.world.borrow_mut();
// //     to_raw(Pointer{
// //         id: world.component_mgr.node._group.get(node_id).style,
// //         world: node.world.clone(),
// //     })
// // }

// // #[no_mangle] pub fn attributes(_world: u32, node_id: u32) -> u32{
// //     1
// // }

// // #[no_mangle] pub fn class_name(_world: u32, node_id: u32) -> String{
// //     "".to_string()
// // }

// // #[no_mangle] pub fn text_content(_world: u32, node_id: u32) -> String{
// //     "".to_string()
// // }

// // #[no_mangle] pub fn src(_world: u32, node_id: u32) -> String{
// //     "".to_string()
// // }

#[no_mangle]
pub fn append_child(world: u32, node_id: u32, child_id: u32){
    let node_id = node_id as usize;
    let child_id = child_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    node_ref.insert_child_with_id(child_id, InsertType::Back);
    js!{console.log("append_child");} 
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
    js!{console.log("insert_before");} 
}

#[no_mangle]
pub fn remove_child(world: u32, node_id: u32, child_id: u32){
    let node_id = node_id as usize;
    let child_id = child_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    node_ref.remove_child(child_id);
    js!{console.log("remove_child");}  
}

#[no_mangle]
pub fn set_class_name(_world: u32, _node_id: u32, _value: &str){
    
    // let class = value.split(" ");
    // let mut arr = Vec::new();
    // for c in class {
    //     arr.push(Atom::from(c.trim()));
    // }

    // let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    // let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    // node_ref.set_class_name(arr);
    js!{console.log("set_class_name");} 
}

#[no_mangle]
pub fn set_text_content(world: u32, node_id: u32, value: &str){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let element_id = world.component_mgr.node._group.get(node_id).element.clone();
    match element_id {
        ElementId::Text(text_id) => {
            if text_id == 0 {
                let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
                let mut text = Text::default();
                text.value = value.to_string();
                node_ref.set_element(Element::Text(text));
            } else {
                let mut text_ref = TextWriteRef::new(text_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
                text_ref.modify(|text: &mut Text| {
                    text.value = value.to_string();
                    true
                });
            }
        },
        _ => (),
    }
}

enum ImageFormat {
    RGB,
    RGBA,
}

// 设置图片的src， texture为TextureRes
#[no_mangle]
pub fn set_image(world: u32, node_id: u32, texture: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let element_id = world.component_mgr.node._group.get(node_id).element.clone();
    match element_id {
        ElementId::Image(image_id) => {
            if image_id == 0 {
                let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
                let mut image = Image::default();
                image.src = texture as usize;
                node_ref.set_element(Element::Image(image));
            } else {
                let mut image_ref = ImageWriteRef::new(image_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
                image_ref.modify(|image: &mut Image| {
                    image.src = texture as usize;
                    true
                });
            }
        },
        _ => (),
    }
    js!{console.log("set_src");} 
}

// #[no_mangle]
// pub fn set_src(world: u32, node_id: u32, value: &str){
//     let node_id = node_id as usize;
//     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
//     let element_id = world.component_mgr.node._group.get(node_id).element.clone();
//     match element_id {
//         ElementId::Image(image_id) => {
//             if image_id == 0 {
//                 let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
//                 let mut image = Image::default();
//                 image.url = Atom::from(value);
//                 node_ref.set_element(Element::Image(image));
//             } else {
//                 let mut image_ref = ImageWriteRef::new(image_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
//                 image_ref.modify(|image: &mut Image| {
//                     image.url = Atom::from(value);
//                     true
//                 });
//             }
//         },
//         _ => (),
//     }
//     js!{console.log("set_src");} 
// }

#[no_mangle]
pub fn query(world: u32, x: f32, y: f32, ty: u32)-> u32{
    js!{console.log("query-------------------1");} 
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    js!{console.log("query-------------------2");} 
    let aabb = Aabb3::new(Point3::new(x,y,-Z_MAX), Point3::new(x,y,Z_MAX));
    js!{console.log("query-------------------3");} 
    let mut args = AbQueryArgs::new(&world.component_mgr, aabb.clone(), ty);
    js!{console.log("query-------------------4");} 
    world.component_mgr.octree.query(&aabb, intersects, &mut args, ab_query_func);
    js!{console.log("result", @{args.result as u32});} 
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
          if node.by_overflow > 0 && in_overflow(&arg.mgr, node.by_overflow, aabb.min.x, aabb.min.y) {
            arg.result = *bind;
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
