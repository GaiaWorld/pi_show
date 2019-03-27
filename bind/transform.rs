use wasm_bindgen::prelude::*;

use bindgen::data::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Transform{
    _id: usize,
    // world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl Transform {
    pub fn matrix(&self, _n1: f32, _n2: f32, _n3: f32, _n4: f32, _n5: f32, _n6: f32) {

    }
    
    pub fn matrix3d(&self, _n1: f32, _n2: f32, _n3: f32, _n4: f32, _n5: f32, _n6: f32, _n7: f32, _n8: f32, _n9: f32, _n10: f32, _n11: f32, _n12: f32, _n13: f32, _n14: f32, _n15: f32, _n16: f32) {

    }

    pub fn translate(&self, _x: LengthPercent, _y: LengthPercent) {

    }

    pub fn translate3d(&self, _x: LengthPercent, _y: LengthPercent, _z: LengthPercent) {
        
    }

    #[wasm_bindgen(js_name = translateX)]
    pub fn translate_x(&self, _value: LengthPercent) {
        
    }

    #[wasm_bindgen(js_name = translateY)]
    pub fn translate_y(&self, _value: LengthPercent) {
        
    }

    #[wasm_bindgen(js_name = translateZ)]
    pub fn translate_z(&self, _value: LengthPercent) {
        
    }

    pub fn scale(&self, _x: f32, _y: f32) {
        
    }

    pub fn scale3d(&self, _x: f32, _y: f32, _z: f32) {
        
    }

    #[wasm_bindgen(js_name = scaleX)]
    pub fn scale_x(&self, _value: f32) {

    }

    #[wasm_bindgen(js_name = scaleY)]
    pub fn scale_y(&self, _value: f32) {

    }

    #[wasm_bindgen(js_name = scaleZ)]
    pub fn scale_z(&self, _value: f32) {

    }

    pub fn rotate(&self, _value: f32) {

    }

    pub fn rotate3d(&self, _x: f32, _y: f32, _z: f32) {
        
    }

    #[wasm_bindgen(js_name = rotateX)]
    pub fn rotate_x(&self, _value: f32) {
        
    }

    #[wasm_bindgen(js_name = rotateY)]
    pub fn rotate_y(&self, _value: f32) {
        
    }

    #[wasm_bindgen(js_name = rotateZ)]
    pub fn rotate_z(&self, _value: f32) {
        
    }
}