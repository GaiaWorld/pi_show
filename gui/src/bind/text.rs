// // use std::rc::Rc;
// // use std::cell::RefCell;
// use std::mem::transmute;

// use stdweb::web::TypedArray;
// use stdweb::unstable::TryInto;

// use wcs::component::{Builder};
// use wcs::world::{World};

// use world_doc::WorldDocMgr;
// use world_doc::component::node::{ NodeWriteRef};
// use world_doc::component::style::generic::{DecorateBuilder, DecorateWriteRef, BoxShadowBuilder, BoxShadowWriteRef};
// use world_doc::component::style::element::{ElementId, Text, Element, TextWriteRef, Image, ImageWriteRef};
// use world_doc::component::style::text::{TextStyleWriteRef, TextStyleGroup};
// use component::color::{Color};
// // use world_doc::component::style::generic::{ClipPath, Clip, Opacity, OpacityWriteRef};
// use component::math::{Color as CgColor};
// use cg::color::{Color as CgColor1};

// pub use layout::yoga::{YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType};
// use bind::data::*;

// pub struct TextStyle {
//     // _id: usize,
//     // world: Rc<RefCell<World<WorldDocMgr, ()>>>,
// }

// #[no_mangle]
// pub fn set_text_align(world: u32, node_id: u32, value: u32){
//     let node_id = node_id as usize;
//     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
//     get_text_style_ref(node_id, world).set_text_align(unsafe {transmute(value)});
//     debug_println!("set_text_align");
// }

// #[no_mangle]
// pub fn set_letter_spacing(world: u32, node_id: u32, value: u32){
//     let node_id = node_id as usize;
//     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
//     get_text_style_ref(node_id, world).set_letter_spacing(unsafe {transmute(value)});
//     debug_println!("set_letter_spacing"); 
// }

// #[no_mangle]
// pub fn set_line_height(world: u32, node_id: u32, value: u32){
//     let node_id = node_id as usize;
//     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
//     get_text_style_ref(node_id, world).set_line_height(unsafe {transmute(value)});
//     debug_println!("set_line_height"); 
// }

// #[no_mangle]
// pub fn set_text_indent(world: u32, node_id: u32, value: u32){
//     let node_id = node_id as usize;
//     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
//     get_text_style_ref(node_id, world).set_text_indent(unsafe {transmute(value)});
//     debug_println!("set_text_indent"); 
// }

// #[no_mangle]
// pub fn set_white_space(world: u32, node_id: u32, value: u32){
//     let node_id = node_id as usize;
//     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
//     get_text_style_ref(node_id, world).set_white_space(unsafe {transmute(value)});
//     debug_println!("set_white_space"); 
// }

// pub struct TextShadow {
//     _id: usize,
//     // world: Rc<RefCell<World<WorldDocMgr, ()>>>,
// }

// // #[no_mangle]
// // pub fn set_h_text_shadow(world: u32, node_id: u32, value: f32){
// //     let node_id = node_id as usize;
// //     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
// //     get_text_style_ref(node_id, world).set_h_text_shadow(unsafe {transmute(value)});
// //     debug_println!("set_h_text_shadow"); 
// // }

// // #[no_mangle]
// // pub fn set_v_text_shadow(world: u32, node_id: u32, value: f32){
// //     let node_id = node_id as usize;
// //     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
// //     get_text_style_ref(node_id, world).set_v_text_shadow(unsafe {transmute(value)});
// //     debug_println!("set_v_text_shadow"); 
// // }

// // #[no_mangle]
// // pub fn set_blur_text_shadow(world: u32, node_id: u32, value: f32){
// //     let node_id = node_id as usize;
// //     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
// //     get_text_style_ref(node_id, world).set_blur_text_shadow(unsafe {transmute(value)});
// //     debug_println!("set_blur_text_shadow"); 
// // }

// // #[no_mangle]
// // pub fn set_color_text_shadow(world: u32, node_id: u32, value: u32){
// //     let node_id = node_id as usize;
// //     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
// //     get_text_style_ref(node_id, world).set_color_text_shadow(unsafe {transmute(value)});
// //     debug_println!("set_color_text_shadow"); 
// // }

// // pub struct Font {
// //     _id: usize,
// //     // world: Rc<RefCell<World<WorldDocMgr, ()>>>,
// // }


// // #[no_mangle]
// // pub fn set_font_style(world: u32, node_id: u32, value: u32){
// //     let node_id = node_id as usize;
// //     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
// //     get_text_style_ref(node_id, world).set_font_style(unsafe {transmute(value)});
// //     debug_println!("set_font_style"); unimplemented!()
// // }

// // #[no_mangle]
// // pub fn set_font_weight(world: u32, node_id: u32, value: u32){
// //     let node_id = node_id as usize;
// //     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
// //     get_text_style_ref(node_id, world).set_font_weight(unsafe {transmute(value)});
// //     debug_println!("set_font_weight"); unimplemented!()
// // }

// // #[no_mangle]
// // pub fn set_font_size(world: u32, node_id: u32, value: f32){
// //     let node_id = node_id as usize;
// //     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
// //     get_text_style_ref(node_id, world).set_font_size(unsafe {transmute(value)});
// //     debug_println!("set_font_size"); unimplemented!()
// // }

// // #[no_mangle]
// // pub fn set_font_family(world: u32, node_id: u32, value: String){
// //     let node_id = node_id as usize;
// //     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
// //     get_text_style_ref(node_id, world).set_font_family(unsafe {transmute(value)});
// //     debug_println!("set_font_family"); unimplemented!()
// // }

// fn get_text_id(node_id: usize, world: &mut World<WorldDocMgr, ()>) -> usize {
//     match world.component_mgr.node._group.get(node_id).element {
//         ElementId::Text(id) => id,
//         _ => panic!("it's not a text"),
//     }
// }

// fn get_text_style_ref(node_id: usize, world: &mut World<WorldDocMgr, ()>) -> TextStyleWriteRef<WorldDocMgr>{
//     let text_id = get_text_id(node_id, world);
//     TextStyleWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr)
// }