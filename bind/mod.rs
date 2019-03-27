use std::rc::Rc;
use std::cell::RefCell;

use wcs::world::{World};
use wcs::component::Builder;

use world::GuiComponentMgr;
use wasm_bindgen::prelude::*;
use component::node::{NodeBuilder};
use component::style::style::{StyleBuilder};
use component::style::flex::{Rect, LayoutBuilder};
use component::style::generic::{StyleUnit};

pub mod data;
pub mod layout;
pub mod text;
pub mod style;
pub mod transform;
pub mod node;
pub mod attribute;

#[wasm_bindgen]
pub struct Gui{
    root: usize,
    world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl Gui{
    pub fn new(&self, width: f32, height: f32) -> Gui{
        let mut world: World<GuiComponentMgr, ()> = World::new();
        let root = {
            let mgr = &mut world.component_mgr;
            let node = NodeBuilder::new().style(StyleBuilder::new().layout(LayoutBuilder::new().wh(Rect::new(Some(StyleUnit::Length(width)), Some(StyleUnit::Length(height)))).build(&mut mgr.node.style.layout)).build(&mut mgr.node.style)).build(&mut mgr.node);
            let r = mgr.add_node(node);
            r.id
        };
        Gui{
            root: root,
            world: Rc::new(RefCell::new(world))
        }
    }

    pub fn run(&self){
        self.world.borrow_mut().run(());
    }

    pub fn root(&self) -> node::Node{
        node::Node::new(self.root, self.world.clone())
    }
}