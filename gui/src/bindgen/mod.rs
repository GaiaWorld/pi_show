use std::rc::Rc;
use std::cell::RefCell;
use std::mem::transmute;

use wcs::world::{World};

use world::GuiComponentMgr;
// use component::style::*;
use component::style::generic::{StyleUnit};
use component::style::style;

#[wasm]
pub struct Gui{
    world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

impl Gui{
    pub fn run(&self){
        self.world.borrow().run(());
    }
}

pub struct Node{
    id: usize
    world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

impl Node {
    pub fn style() -> {

    }
}

pub struct Style{
    id: usize
    world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

impl Style {
    pub fn set_display(&self, value: Display){
        let mut world = self.world.borrow_mut();
        let mut style_ref = Style::get_ref(&mut world.component_mgr);
        style_ref.set_display(Some(transmute(value)));
    }

    pub fn del_display(&self){
        let mut world = self.world.borrow_mut();
        let mut style_ref = Style::get_ref(&mut world.component_mgr);
        style_ref.set_display(None);
    }

    pub fn layout(&self) ->  {
        let mut world = self.world.borrow_mut();
        let mut style = Style::get_ref(&mut world.component_mgr);
        style.set
    }

    fn get_ref(mgr: &mut GuiComponentMgr) -> StyleWriteRef<GuiComponentMgr>{
        StyleWriteRef::new(id, mgr.node.style.to_usize(), mgr)
    }
}

pub enum StyleUnitType{
    Auto,
    UndefinedValue,
    Length,
    Percent
}

pub struct StyleUnit{
    pub ty: StyleUnitType,
    pub value : f32,
}

#[derive(Clone, Copy, Default)]
pub enum Display {
    Flex,
    Inline,
    Display,
}