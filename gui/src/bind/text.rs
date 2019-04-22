// use wasm_bindgen::prelude::*;

// use bindgen::data::*;

pub struct TextStyle {
    // _id: usize,
    // world: Rc<RefCell<World<WorldDocMgr, ()>>>,
}

#[no_mangle]
pub fn font(_world: u32, _node_id: u32) -> u32 {
    js!{console.log("font")} unimplemented!()
}

#[no_mangle]
pub fn text(_world: u32, _node_id: u32) -> u32 {
    js!{console.log("text")} unimplemented!()
}

#[no_mangle]
pub fn text_align(_world: u32, _node_id: u32) -> u32{
    js!{console.log("text_align")} 
    1
}

#[no_mangle]
pub fn letter_spacing(_world: u32, _node_id: u32) -> f32{
    js!{console.log("letter_spacing")}
    1.0 
}

#[no_mangle]
pub fn line_height(_world: u32, _node_id: u32) -> u32{
    js!{console.log("line_height")} 
    1
}

#[no_mangle]
pub fn text_indent(_world: u32, _node_id: u32) -> u32{
    js!{console.log("text_indent")} 
    1
}

#[no_mangle]
pub fn white_space(_world: u32, _node_id: u32) -> u32{
    js!{console.log("white_space")} 
    1
}

#[no_mangle]
pub fn text_color(_world: u32, _node_id: u32) -> u32{
    js!{console.log("text_color")} 
    1
}

#[no_mangle]
pub fn text_shadow(_world: u32, _node_id: u32) -> u32{
    js!{console.log("text_shadow")} 
    1
}

#[no_mangle]
pub fn h_text_shadow(_world: u32, _node_id: u32) -> f32{
    js!{console.log("h_text_shadow")} 
    1.0
}

#[no_mangle]
pub fn v_text_shadow(_world: u32, _node_id: u32) -> f32{
    js!{console.log("v_text_shadow")} 
    1.0
}

#[no_mangle]
pub fn blur_text_shadow(_world: u32, _node_id: u32) -> f32{
    js!{console.log("blur_text_shadow")} 
    1.0
}

#[no_mangle]
pub fn color_text_shadow(_world: u32, _node_id: u32) -> u32{
    js!{console.log("color_text_shadow")} 
    1
}

#[no_mangle]
pub fn font_style(_world: u32, _node_id: u32) -> u32{
    js!{console.log("font_style")} unimplemented!()
}

#[no_mangle]
pub fn font_weight(_world: u32, _node_id: u32) -> u32{
    js!{console.log("font_weight")} unimplemented!()
}

#[no_mangle]
pub fn font_size(_world: u32, _node_id: u32) -> f32{
    js!{console.log("font_size")} unimplemented!()
}

#[no_mangle]
pub fn font_family(_world: u32, _node_id: u32) -> String{
    js!{console.log("font_family")} unimplemented!()
}

#[no_mangle]
pub fn set_text_align(_world: u32, _node_id: u32, _value: u32){
    js!{console.log("set_text_align")}
}

#[no_mangle]
pub fn set_letter_spacing(_world: u32, _node_id: u32, _value: u32){
    js!{console.log("set_letter_spacing")} 
}

#[no_mangle]
pub fn set_line_height(_world: u32, _node_id: u32, _value: u32){
    js!{console.log("set_line_height")} 
}

#[no_mangle]
pub fn set_text_indent(_world: u32, _node_id: u32, _value: u32){
    js!{console.log("set_text_indent")} 
}

#[no_mangle]
pub fn set_white_space(_world: u32, _node_id: u32, _value: u32){
    js!{console.log("set_white_space")} 
}

pub struct TextShadow {
    _id: usize,
    // world: Rc<RefCell<World<WorldDocMgr, ()>>>,
}

#[no_mangle]
pub fn set_h_text_shadow(_world: u32, _node_id: u32, _value: f32){
    js!{console.log("set_h_text_shadow")} 
}

#[no_mangle]
pub fn set_v_text_shadow(_world: u32, _node_id: u32, _value: f32){
    js!{console.log("set_v_text_shadow")} 
}

#[no_mangle]
pub fn set_blur_text_shadow(_world: u32, _node_id: u32, _value: f32){
    js!{console.log("set_blur_text_shadow")} 
}

#[no_mangle]
pub fn set_color_text_shadow(_world: u32, _node_id: u32, _value: u32){
    js!{console.log("set_color_text_shadow")} 
}

pub struct Font {
    _id: usize,
    // world: Rc<RefCell<World<WorldDocMgr, ()>>>,
}


#[no_mangle]
pub fn set_font_style(_world: u32, _node_id: u32, _value: u32){
    js!{console.log("set_font_style")} unimplemented!()
}

#[no_mangle]
pub fn set_font_weight(_world: u32, _node_id: u32, _value: u32){
    js!{console.log("set_font_weight")} unimplemented!()
}

#[no_mangle]
pub fn set_font_size(_world: u32, _node_id: u32, _value: f32){
    js!{console.log("set_font_size")} unimplemented!()
}

#[no_mangle]
pub fn set_font_family(_world: u32, _node_id: u32, _value: String){
    js!{console.log("set_font_family")} unimplemented!()
}

