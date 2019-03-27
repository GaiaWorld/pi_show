use wasm_bindgen::prelude::*;

use bindgen::data::*;
// use layout;

#[wasm_bindgen]
pub struct Layout{
    _id: usize,
    // world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl Layout {
    pub fn container(&self) -> FlexContainer{
        unimplemented!()
    }

    pub fn item(&self) -> FlexItem{
        unimplemented!()
    }

    pub fn margin(&self) -> Boundary {
        unimplemented!()
    }

    pub fn border(&self) -> Boundary {
        unimplemented!()
    }

    pub fn padding(&self) -> Boundary {
        unimplemented!()
    }

    pub fn wh(&self) -> Rect {
        unimplemented!()
    }

    pub fn position(&self) -> Position {
        unimplemented!()
    }
}

#[wasm_bindgen]
pub struct FlexContainer {
    _id: usize,
    // world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl FlexContainer {
    pub fn align_content(&self) -> Option<AlignContent>{
        unimplemented!()
    }

    pub fn align_items(&self) -> Option<AlignItems>{
        unimplemented!()
    }

    pub fn justify_ontent(&self) -> Option<JustifyContent>{
        unimplemented!()
    }

    pub fn flex_direction(&self) -> Option<FlexDirection>{
        unimplemented!()
    }

    pub fn flex_wrap(&self) -> Option<FlexWrap>{
        unimplemented!()
    }

    pub fn set_align_content(&self, _value: Option<AlignContent>){
        unimplemented!()
    }

    pub fn set_align_items(&self, _value: Option<AlignItems>){
        unimplemented!()
    }

    pub fn set_justify_ontent(&self, _value: Option<JustifyContent>){
        unimplemented!()
    }

    pub fn set_flex_direction(&self, _value: Option<FlexDirection>){
        unimplemented!()
    }

    pub fn set_flex_wrap(&self, _value: Option<FlexWrap>){
        unimplemented!()
    }
}

#[wasm_bindgen]
pub struct FlexItem {
    _id: usize,
    // world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl FlexItem {
    pub fn flex_grow(&self) -> Option<f32> {
        unimplemented!()
    }

    pub fn flex_shrink(&self) -> Option<f32> {
        unimplemented!()
    }

    pub fn flex_basis(&self) -> StyleUnit {
        unimplemented!()
    }

    pub fn align_self(&self) -> Option<AlignSelf>{
        unimplemented!()
    }

    pub fn set_flex_grow(&self, _value: Option<f32>){
        unimplemented!()
    }

    pub fn set_flex_shrink(&self, _value: Option<f32>){
        unimplemented!()
    }

    pub fn set_flex_basis(&self, _value: StyleUnit){
        unimplemented!()
    }

    pub fn set_align_self(&self, _value: Option<AlignSelf>){
        unimplemented!()
    }
}

#[wasm_bindgen]
pub struct Boundary {
    _id: usize,
    // world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl Boundary {
    pub fn top(&self) -> StyleUnit {
        unimplemented!()
    }

    pub fn right(&self) -> StyleUnit {
        unimplemented!()
    }

    pub fn bottom(&self) -> StyleUnit {
        unimplemented!()
    }

    pub fn left(&self) -> StyleUnit{
        unimplemented!()
    }

    pub fn set_top(&self, _value: StyleUnit){
        unimplemented!()
    }

    pub fn set_right(&self, _value: StyleUnit){
        unimplemented!()
    }

    pub fn set_bottom(&self, _value: StyleUnit){
        unimplemented!()
    }

    pub fn set_left(&self, _value: StyleUnit){
        unimplemented!()
    }
}

#[wasm_bindgen]
pub struct Rect {
    _id: usize,
    // world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}


#[wasm_bindgen]
impl Rect {
    pub fn width(&self) -> StyleUnit {
        unimplemented!()
    }

    pub fn height(&self) -> StyleUnit {
        unimplemented!()
    }

    pub fn set_width(&self, _value: StyleUnit){
        unimplemented!()
    }

    pub fn set_height(&self, _value: StyleUnit){
        unimplemented!()
    }

}

#[wasm_bindgen]
pub struct MinMax {
    _id: usize,
    // world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl MinMax {
    pub fn max_width(&self) -> StyleUnit {
        unimplemented!()
    }

    pub fn max_height(&self) -> StyleUnit {
        unimplemented!()
    }

    pub fn min_width(&self) -> StyleUnit {
        unimplemented!()
    }

    pub fn min_height(&self) -> StyleUnit {
        unimplemented!()
    }

    pub fn set_max_width(&self, _value: StyleUnit){
        unimplemented!()
    }

    pub fn set_max_height(&self, _value: StyleUnit){
        unimplemented!()
    }

    pub fn set_min_width(&self, _value: StyleUnit){
        unimplemented!()
    }

    pub fn set_min_height(&self, _value: StyleUnit){
        unimplemented!()
    }
}

#[wasm_bindgen]
pub struct Position {
    _id: usize,
    // world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl Position {
    pub fn ty(&self) -> Option<PositionType> {
        unimplemented!()
    }

    pub fn top(&self) -> StyleUnit {
        unimplemented!()
    }

    pub fn right(&self) -> StyleUnit {
        unimplemented!()
    }

    pub fn bottom(&self) -> StyleUnit {
        unimplemented!()
    }

    pub fn left(&self) -> StyleUnit{
        unimplemented!()
    }

    pub fn set_top(&self, _value: StyleUnit){
        unimplemented!()
    }

    pub fn set_right(&self, _value: StyleUnit){
        unimplemented!()
    }

    pub fn set_bottom(&self, _value: StyleUnit){
        unimplemented!()
    }

    pub fn set_left(&self, _value: StyleUnit){
        unimplemented!()
    }

    pub fn set_ty(&self, _value: StyleUnit){
        unimplemented!()
    }
}