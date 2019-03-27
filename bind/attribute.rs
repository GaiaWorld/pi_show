use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Attributes;

#[wasm_bindgen]
impl Attributes {
    pub fn get_named_item(&self) -> String{
        unimplemented!()
    }

    pub fn length(&self) -> u32{
        unimplemented!()
    }

    pub fn remove_named_item(&self, _name: &str){
        unimplemented!()
    }

    pub fn set_named_item(&self, _name: &str, _value: &str){
        unimplemented!()
    }
}