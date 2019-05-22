use std::{
  sync::Arc,
  usize::MAX as UMAX,
  f32::INFINITY as FMAX,
};


use ecs::{World, Lend, LendMut};
use ecs::idtree::{IdTree, InsertType};

use component::user::*;
use entity::Node;

fn create(world: &World) -> usize {
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.lend_mut();
    let node = world.create_entity::<Node>();
    idtree.create(node);
    node
}

fn insert_child(world: u32, child: u32, parent: u32, index: usize){
    let world = unsafe {&mut *(world as usize as *mut World)};
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.lend_mut();
    let notify = idtree.get_notify();
    idtree.insert_child(child as usize, parent as usize, index, Some(&notify));
}

//创建容器节点， 容器节点可设置背景颜色
#[no_mangle]
pub fn create_node(world: u32) -> u32{
    let world = unsafe {&mut *(world as usize as *mut World)};
    let node = create(world);
    debug_println!("create_node, node:{}", node);
    node as u32
}

#[no_mangle]
pub fn create_text_node(world: u32) -> u32 {
    let world = unsafe {&mut *(world as usize as *mut World)};
    let node = create(world);
    let text = world.fetch_multi::<Node, Text>().unwrap();
    let text = text.lend_mut();
    text.insert(node, Text(Arc::new("".to_string())));
    debug_println!("create_text_node, node:{}", node);
    node as u32
}

//创建图片节点
#[no_mangle]
pub fn create_image_node(world: u32) -> u32{
    let world = unsafe {&mut *(world as usize as *mut World)};
    let node = create(world);
    debug_println!("create_image_node, node:{}", node);
    node as u32
}

// 在尾部插入子节点
#[no_mangle]
pub fn append_child(world: u32, child: u32, parent: u32){
    let world = unsafe {&mut *(world as usize as *mut World)};
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.lend_mut();
    let notify = idtree.get_notify();
    idtree.insert_child(child as usize, parent as usize, UMAX, Some(&notify));
    debug_println!("append_child"); 
}

#[no_mangle]
pub fn insert_before(world: u32, child: u32, brother: u32){
    let world = unsafe {&mut *(world as usize as *mut World)};
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.lend_mut();
    let notify = idtree.get_notify();
    idtree.insert_brother(child as usize, brother as usize, InsertType::Front, Some(&notify));
    debug_println!("insert_before"); 
}

#[no_mangle]
pub fn remove_child(world: u32, node: u32){
    let world = unsafe {&mut *(world as usize as *mut World)};
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.lend_mut();
    let notify = idtree.get_notify();
    idtree.remove(node as usize, Some(&notify));
    debug_println!("remove_child");  
}

// #[no_mangle]
// pub fn set_text_content(world: u32, node: u32){
//     let value: String = js!(return __jsObj;).try_into().unwrap();
//     let node = node as usize;
//     let world = unsafe {&mut *(world as usize as *mut World)};
//     let element_id = world.component_mgr.node._group.get(node).element.clone();
//     match element_id {
//         ElementId::Text(text_id) => {
//             if text_id == 0 {
//                 let mut node_ref = NodeWriteRef::new(node, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr);
//                 let mut text = Text::default();
//                 text.value = value;
//                 node_ref.set_element(Element::Text(text));
//             } else {
//                 let mut text_ref = TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr);
//                 text_ref.set_value(value);
//             }
//             debug_println!("set_text_content");
//         },
//         _ => (),
//     }
// }

// // __jsObj: image, __jsObj1: image_name(String)
// // 设置图片的src
// #[no_mangle]
// pub fn set_src(world: u32, node: u32, opacity: u8, compress: u8){
//     let name: String = js!{return __jsObj1}.try_into().unwrap();
//     let name = Atom::from(name);
//     let node = node as usize;
//     let world = unsafe {&mut *(world as usize as *mut World)};
//     let (width, height, texture) = match world.component_mgr.world_2d.component_mgr.engine.res_mgr.textures.get(&name) {
//         Some(res) => {
//           (res.width as u32, res.height as u32, Box::into_raw(Box::new(res)) as u32)
//         },
//         None => {
//           let gl = world.component_mgr.world_2d.component_mgr.engine.gl.clone();
//           let texture = match gl.create_texture() {
//               Some(v) => v,
//               None => panic!("create_texture is None"),
//           };
//           gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&texture));
//           js!{
//             @{&gl}.texImage2D(@{&gl}.TEXTURE_2D, 0, @{&gl}.RGBA, @{&gl}.RGBA, @{&gl}.UNSIGNED_BYTE, __jsObj);
//           };
//           let width: u32 = js!{return __jsObj.width}.try_into().unwrap();
//           let height: u32 = js!{return __jsObj.height}.try_into().unwrap();
//           gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MAG_FILTER, WebGLRenderingContext::NEAREST as i32);
//           gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MIN_FILTER, WebGLRenderingContext::NEAREST as i32);
//           let res = world.component_mgr.world_2d.component_mgr.engine.res_mgr.textures.create(TextureRes::new(name, width as usize, height as usize, unsafe{transmute(opacity)}, unsafe{transmute(compress)}, texture, gl.clone()) );
//           (width, height, Box::into_raw(Box::new( res)) as u32)
//         },
//     };

//     let (yoga, element_id) = {
//       let node = world.component_mgr.node._group.get(node);
//       (node.yoga, node.element.clone())
//     };
//     match element_id {
//         ElementId::Image(image_id) => {
//             if image_id == 0 {
//               let mut node_ref = NodeWriteRef::new(node, world.component_mgr.node.to_usize(), &mut world.component_mgr);
//               let mut image = Image::default();
//               image.src = texture as usize;
//               node_ref.set_element(Element::Image(image));
//             } else {
//               let mut image_ref = ImageWriteRef::new(image_id, world.component_mgr.node.element.image.to_usize(), &mut world.component_mgr);
//               image_ref.set_src(texture as usize);
//             }
//         },
//         _ => println!("it's not image, node: {}", node),
//     }

//     match yoga.get_width().unit {
//         YGUnit::YGUnitUndefined | YGUnit::YGUnitAuto => yoga.set_width(width as f32),
//         _ => (),
//     };
//     match yoga.get_height().unit {
//         YGUnit::YGUnitUndefined | YGUnit::YGUnitAuto => yoga.set_height(height as f32),
//         _ => (),
//     };
//     debug_println!("set_src"); 
// }

#[no_mangle]
pub fn offset_top(world: u32, node: u32) -> f32 {
    let world = unsafe {&mut *(world as usize as *mut World)};
    let layout = world.fetch_multi::<Node, Layout>().unwrap();
    unsafe {layout.lend().get_unchecked(node as usize)}.top
}

#[no_mangle]
pub fn offset_left(world: u32, node: u32) -> f32 {
    let world = unsafe {&mut *(world as usize as *mut World)};
    let layout = world.fetch_multi::<Node, Layout>().unwrap();
    unsafe {layout.lend().get_unchecked(node as usize)}.left
}

#[no_mangle]
pub fn offset_width(world: u32, node: u32) -> f32 {
    let world = unsafe {&mut *(world as usize as *mut World)};
    let layout = world.fetch_multi::<Node, Layout>().unwrap();
    unsafe {layout.lend().get_unchecked(node as usize)}.width
}

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
