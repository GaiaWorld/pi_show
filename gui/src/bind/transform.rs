// use wasm_bindgen::prelude::*;

// use bindgen::data::*;


#[derive(Debug, Clone)]
pub struct Transform{
    _id: usize,
    // world: Rc<RefCell<World<DocumentMgr, ()>>>,
}

#[no_mangle]
pub fn transform_matrix(_own: u32, _n1: f32, _n2: f32, _n3: f32, _n4: f32, _n5: f32, _n6: f32) {

}

#[no_mangle]
pub fn transform_matrix3d(_own: u32, _n1: f32, _n2: f32, _n3: f32, _n4: f32, _n5: f32, _n6: f32, _n7: f32, _n8: f32, _n9: f32, _n10: f32, _n11: f32, _n12: f32, _n13: f32, _n14: f32, _n15: f32, _n16: f32) {

}

#[no_mangle]
pub fn transform_translate(_own: u32, _x: u32, _y: u32) {

}

#[no_mangle]
pub fn transform_translate3d(_own: u32, _x: u32, _y: u32, _z: u32) {
    
}

#[no_mangle]
pub fn transform_translate_x(_own: u32, _value: u32) {
    
}

#[no_mangle]
pub fn transform_translate_y(_own: u32, _value: u32) {
    
}

#[no_mangle]
pub fn transform_translate_z(_own: u32, _value: u32) {
    
}

#[no_mangle]
pub fn transform_scale(_own: u32, _x: f32, _y: f32) {
    
}

#[no_mangle]
pub fn transform_scale3d(_own: u32, _x: f32, _y: f32, _z: f32) {
    
}

#[no_mangle]
pub fn transform_scale_x(_own: u32, _value: f32) {

}

#[no_mangle]
pub fn transform_scale_y(_own: u32, _value: f32) {

}

#[no_mangle]
pub fn transform_scale_z(_own: u32, _value: f32) {

}

#[no_mangle]
pub fn transform_rotate(_own: u32, _value: f32) {

}

#[no_mangle]
pub fn transform_rotate3d(_own: u32, _x: f32, _y: f32, _z: f32) {
    
}

#[no_mangle]
pub fn transform_rotate_x(_own: u32, _value: f32) {
    
}

#[no_mangle]
pub fn transform_rotate_y(_own: u32, _value: f32) {
    
}

#[no_mangle]
pub fn transform_rotate_z(_own: u32, _value: f32) {
    
}