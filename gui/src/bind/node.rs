use atom::Atom;
use cg::octree::intersects;

use world::{Z_MAX, World, DocumentMgr};
use document::component::style::element::{ElementId, Text, Element, TextWriteRef, Image, ImageWriteRef};
use document::component::node::{InsertType, NodeWriteRef};

use bind::{Pointer};

// #[no_mangle] pub fn get_style(own: u32) -> u32 {
//     let node = unsafe {&*(own as *const Pointer)};
//     let world = node.world.borrow_mut();
//     to_raw(Pointer{
//         id: world.component_mgr.node._group.get(node.id).style,
//         world: node.world.clone(),
//     })
// }

// #[no_mangle] pub fn attributes(_own: u32) -> u32{
//     1
// }

// #[no_mangle] pub fn class_name(_own: u32) -> String{
//     "".to_string()
// }

// #[no_mangle] pub fn text_content(_own: u32) -> String{
//     "".to_string()
// }

// #[no_mangle] pub fn src(_own: u32) -> String{
//     "".to_string()
// }

#[no_mangle]
pub fn append_child(own: u32, child: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let child = unsafe {&*(child as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    node_ref.insert_child_with_id(child.id, InsertType::Back);
    js!{console.log("append_child");} 
}

#[no_mangle]
pub fn insert_before(own: u32, child: u32, brother: u32, brother_index: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let child = unsafe {&*(child as *const Pointer)};
    let brother = unsafe {&*(brother as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let brother_qid = world.component_mgr.node._group.get(brother.id).qid;
    let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    node_ref.insert_child_with_id(child.id, InsertType::ToFront(brother_index as usize, brother_qid));
    js!{console.log("insert_before");} 
}

#[no_mangle] pub fn remove_child(own: u32, child: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let child = unsafe {&*(child as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    node_ref.remove_child(child.id);
    js!{console.log("remove_child");}  
}

#[no_mangle] pub fn set_class_name(own: u32, value: &str){
    let node = unsafe {&*(own as *const Pointer)};
    let class = value.split(" ");
    let mut arr = Vec::new();
    for c in class {
        arr.push(Atom::from(c.trim()));
    }

    let mut world = node.world.borrow_mut();
    let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    node_ref.set_class_name(arr);
    js!{console.log("set_class_name");} 
}

#[no_mangle] pub fn set_text_content(own: u32, value: &str){
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let element_id = world.component_mgr.node._group.get(node.id).element.clone();
    match element_id {
        ElementId::Text(text_id) => {
            if text_id == 0 {
                let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
                let mut text = Text::default();
                text.value = Atom::from(value);
                node_ref.set_element(Element::Text(text));
            } else {
                let mut text_ref = TextWriteRef::new(text_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
                text_ref.modify(|text: &mut Text| {
                    text.value = Atom::from(value);
                    true
                });
            }
        },
        _ => (),
    }
}

#[no_mangle] pub fn set_src(own: u32, value: &str){
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let element_id = world.component_mgr.node._group.get(node.id).element.clone();
    match element_id {
        ElementId::Image(image_id) => {
            if image_id == 0 {
                let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
                let mut image = Image::default();
                image.url = Atom::from(value);
                node_ref.set_element(Element::Image(image));
            } else {
                let mut image_ref = ImageWriteRef::new(image_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
                image_ref.modify(|image: &mut Image| {
                    image.url = Atom::from(value);
                    true
                });
            }
        },
        _ => (),
    }
    js!{console.log("set_src");} 
}

#[no_mangle] pub fn query(world_p: u32, x: u32, y: u32, type: u32)-> u32{
    let world = unsafe {&*(world_p as *const World)};
    let aabb = Aabb3::new(Point3::new(x,y,-Z_MAX), Point3::new(x,y,Z_MAX));
    let mut args = AbQueryArgs::new(&world.component_mgr, aabb.clone(), type);
    world.component_mgr.tree.query(&aabb, intersects, &mut args, ab_query_func);
    js!{console.log("result");} 
    args.result
}
/// aabb的查询函数的参数
struct AbQueryArgs<'a> {
  mgr: &'a DocumentMgr,
  aabb: Aabb3<f32>,
  type: u32,
  max_z: f32,
  result: u32,
}
impl<'a> AbQueryArgs<'a> {
  pub fn new(mgr: &DocumentMgr, aabb: Aabb3<f32>, type: u32) -> AbQueryArgs {
    AbQueryArgs{
      mgr: mgr,
      aabb: aabb,
      type: type,
      max_z: -Z_MAX,
      result: 0,
    }
  }
}
/// aabb的ab查询函数, aabb的oct查询函数应该使用intersects
fn ab_query_func(arg: &mut AbQueryArgs, _id: usize, aabb: &Aabb3<f32>, bind: &u32) {
  if intersects(&arg.aabb, aabb) {
    let node = mgr.node._group.get(*bind);
    // 判断类型是否有相交
    if node.event_type & arg.type != 0 {
        // 取最大z的node
        if node.z_depth > arg.max_z {
           arg.result = bind.clone();
        }
    }
  }
}