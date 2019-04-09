// use wasm_bindgen::prelude::*;

// use bindgen::data::*;


#[derive(Debug, Clone)]
pub struct Transform{
    _id: usize,
    // world: Rc<RefCell<World<WorldDocMgr, ()>>>,
}

#[no_mangle]
pub fn transform_matrix(_world: u32, _node_id: u32, _n1: f32, _n2: f32, _n3: f32, _n4: f32, _n5: f32, _n6: f32) {

}

#[no_mangle]
pub fn transform_matrix3d(_world: u32, _node_id: u32, _n1: f32, _n2: f32, _n3: f32, _n4: f32, _n5: f32, _n6: f32, _n7: f32, _n8: f32, _n9: f32, _n10: f32, _n11: f32, _n12: f32, _n13: f32, _n14: f32, _n15: f32, _n16: f32) {

}

#[no_mangle]
pub fn transform_translate(_world: u32, _node_id: u32, _x: u32, _y: u32) {

}

#[no_mangle]
pub fn transform_translate3d(_world: u32, _node_id: u32, _x: u32, _y: u32, _z: u32) {
    
}

#[no_mangle]
pub fn transform_translate_x(_world: u32, _node_id: u32, _value: u32) {
    
}

#[no_mangle]
pub fn transform_translate_y(_world: u32, _node_id: u32, _value: u32) {
    
}

#[no_mangle]
pub fn transform_translate_z(_world: u32, _node_id: u32, _value: u32) {
    
}

#[no_mangle]
pub fn transform_scale(_world: u32, _node_id: u32, _x: f32, _y: f32) {
    
}

#[no_mangle]
pub fn transform_scale3d(_world: u32, _node_id: u32, _x: f32, _y: f32, _z: f32) {
    
}

#[no_mangle]
pub fn transform_scale_x(_world: u32, _node_id: u32, _value: f32) {

}

#[no_mangle]
pub fn transform_scale_y(_world: u32, _node_id: u32, _value: f32) {

}

#[no_mangle]
pub fn transform_scale_z(_world: u32, _node_id: u32, _value: f32) {

}

#[no_mangle]
pub fn transform_rotate(_world: u32, _node_id: u32, _value: f32) {

}

#[no_mangle]
pub fn transform_rotate3d(_world: u32, _node_id: u32, _x: f32, _y: f32, _z: f32) {
    
}

#[no_mangle]
pub fn transform_rotate_x(_world: u32, _node_id: u32, _value: f32) {
    
}

#[no_mangle]
pub fn transform_rotate_y(_world: u32, _node_id: u32, _value: f32) {
    
}

#[no_mangle]
pub fn transform_rotate_z(_world: u32, _node_id: u32, _value: f32) {
    
}