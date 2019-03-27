use std::rc::Rc;
use std::cell::RefCell;

use wasm_bindgen::prelude::*;

use wcs::world::{World};

use world::GuiComponentMgr;

use bindgen::data::*;
use bindgen::layout::*;
use bindgen::text::*;
use bindgen::transform::*;
use component::style::style::{StyleBuilder};


#[wasm_bindgen]
#[derive(Clone)]
pub struct Style{
    id: usize,
    world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl Style {
    pub fn display(&self) -> Option<Display>{
        unimplemented!()
    }

    pub fn layout(&self) -> Layout{
        unimplemented!()
    }

    pub fn clip_path(&self) -> ClipPath{
        unimplemented!()
    }

    pub fn text(&self) -> Text{
        unimplemented!()
    }

    pub fn transform(&self) -> Transform{
        unimplemented!()
    }

    pub fn background_color(&self) -> Color{
        unimplemented!()
    }

    pub fn set_background_color(&self, _value: Color){
        unimplemented!()
    }

    pub fn set_clip_path(&self, _value: ClipPath){
        unimplemented!()
    }
}

pub struct StyleDate {
    builder: StyleBuilder,
    world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

impl StyleDate {
    pub fn display(&mut self, value: Display){
        // self.builder.display()
    }
}