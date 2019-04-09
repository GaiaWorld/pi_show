// use wasm_bindgen::prelude::*;

// use bindgen::data::*;

use wcs::component::{Builder};
use wcs::world::{World};

use world_doc::WorldDocMgr;
use world_doc::component::node::{ NodeWriteRef};
use world_doc::component::style::transform::{ TransformWriteRef, TransformBuilder};

// #[no_mangle]
// pub fn transform_matrix(_world: u32, _node_id: u32, _n1: f32, _n2: f32, _n3: f32, _n4: f32, _n5: f32, _n6: f32) {

// }

// #[no_mangle]
// pub fn transform_matrix3d(_world: u32, _node_id: u32, _n1: f32, _n2: f32, _n3: f32, _n4: f32, _n5: f32, _n6: f32, _n7: f32, _n8: f32, _n9: f32, _n10: f32, _n11: f32, _n12: f32, _n13: f32, _n14: f32, _n15: f32, _n16: f32) {

// }

// #[no_mangle]
// pub fn transform_translate(_world: u32, _node_id: u32, _x: u32, _y: u32) {

// }

// #[no_mangle]
// pub fn transform_translate3d(_world: u32, _node_id: u32, _x: u32, _y: u32, _z: u32) {
    
// }

// #[no_mangle]
// pub fn transform_translate_x(_world: u32, _node_id: u32, _value: u32) {
    
// }

// #[no_mangle]
// pub fn transform_translate_y(_world: u32, _node_id: u32, _value: u32) {
    
// }

// #[no_mangle]
// pub fn transform_translate_z(_world: u32, _node_id: u32, _value: u32) {
    
// }

// #[no_mangle]
// pub fn transform_scale(_world: u32, _node_id: u32, _x: f32, _y: f32) {
    
// }

// #[no_mangle]
// pub fn transform_scale3d(_world: u32, _node_id: u32, _x: f32, _y: f32, _z: f32) {
    
// }

// #[no_mangle]
// pub fn transform_scale_x(_world: u32, _node_id: u32, _value: f32) {

// }

// #[no_mangle]
// pub fn transform_scale_y(_world: u32, _node_id: u32, _value: f32) {

// }

// #[no_mangle]
// pub fn transform_scale_z(_world: u32, _node_id: u32, _value: f32) {

// }

#[no_mangle]
pub fn transform_rotate(world: u32, node_id: u32, value: f32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let transform_id = world.component_mgr.node._group.get(node_id).transform;
    if transform_id == 0 {
        let transform = TransformBuilder::new()
        .rotation(value)
        .build(&mut world.component_mgr.node.transform);
        let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_transform(transform);
    }else {
        let mut transform_ref = TransformWriteRef::new(transform_id, world.component_mgr.node.transform.to_usize(), &mut world.component_mgr);
        transform_ref.set_rotation(value);
        return;
    }
}

// #[no_mangle]
// pub fn transform_rotate3d(_world: u32, _node_id: u32, _x: f32, _y: f32, _z: f32) {
    
// }

// #[no_mangle]
// pub fn transform_rotate_x(_world: u32, _node_id: u32, _value: f32) {
    
// }

// #[no_mangle]
// pub fn transform_rotate_y(_world: u32, _node_id: u32, _value: f32) {
    
// }

// #[no_mangle]
// pub fn transform_rotate_z(_world: u32, _node_id: u32, _value: f32) {
    
// }