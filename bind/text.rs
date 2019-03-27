use wasm_bindgen::prelude::*;

use bindgen::data::*;

#[wasm_bindgen]
pub struct TextStyle {
    _id: usize,
    // world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl TextStyle {
    pub fn text_align(&self) -> Option<TextAlign>{
        unimplemented!()
    }

    pub fn letter_spacing(&self) -> Option<f32>{
        unimplemented!()
    }

    pub fn line_height(&self) -> LineHeight{
        unimplemented!()
    }

    pub fn text_indent(&self) -> StyleUnit{
        unimplemented!()
    }

    pub fn white_space(&self) -> Option<WhiteSpace>{
        unimplemented!()
    }

    pub fn color(&self) -> Color{
        unimplemented!()
    }

    pub fn text_shadow(&self) -> TextShadow{
        unimplemented!()
    }

    pub fn set_text_align(&self, _value: Option<TextAlign>){
        unimplemented!()
    }

    pub fn set_letter_spacing(&self, _value: Option<f32>){
        unimplemented!()
    }

    pub fn set_line_height(&self, _value: LineHeight){
        unimplemented!()
    }

    pub fn set_text_indent(&self, _value: StyleUnit){
        unimplemented!()
    }

    pub fn set_white_space(&self, _value: Option<WhiteSpace>){
        unimplemented!()
    }
}

#[wasm_bindgen]
pub struct TextShadow {
    _id: usize,
    // world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl TextShadow {
    pub fn h(&self) -> Option<f32>{
        unimplemented!()
    }

    pub fn v(&self) -> Option<f32>{
        unimplemented!()
    }

    pub fn blur(&self) -> Option<f32>{
        unimplemented!()
    }

    pub fn color(&self) -> Color{
        unimplemented!()
    }

    pub fn set_h(&self, _value: Option<f32>){
        unimplemented!()
    }

    pub fn set_v(&self, _value: Option<f32>){
        unimplemented!()
    }

    pub fn set_blur(&self, _value: Option<f32>){
        unimplemented!()
    }
}

#[wasm_bindgen]
pub struct Font {
    _id: usize,
    // world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[allow(unused_attributes)]
impl Font{
    pub fn font_style(&self) -> Option<FontStyle>{
        unimplemented!()
    }

    pub fn font_weight(&self) -> Option<FontWeight>{
        unimplemented!()
    }

    pub fn font_size(&self) -> FontSize{
        unimplemented!()
    }

    pub fn font_family(&self) -> String{
        unimplemented!()
    }

    pub fn set_font_style(&self, _value: Option<FontStyle>){
        unimplemented!()
    }

    pub fn set_font_weight(&self, _value: Option<FontWeight>){
        unimplemented!()
    }

    pub fn set_font_size(&self, _value: FontSize){
        unimplemented!()
    }

    pub fn set_font_family(&self, _value: String){
        unimplemented!()
    }
}

#[wasm_bindgen]
pub struct Text {
    _id: usize,
    // world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl Text {
    pub fn font(&self) -> Font {
        unimplemented!()
    }

    pub fn text(&self) -> TextStyle {
        unimplemented!()
    }
}