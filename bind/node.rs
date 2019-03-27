use std::rc::Rc;
use std::cell::RefCell;

use wasm_bindgen::prelude::*;

use wcs::world::{World};

use world::GuiComponentMgr;
use bindgen::style::*;
use bindgen::attribute::*;
use component::node::{Node as CNode, NodeBuilder};

pub enum NodeType {
    Text,
    Img,
    Div
}


#[wasm_bindgen]
pub struct Node{
    id: usize,
    world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

impl Node {
    pub fn new (id: usize, world: Rc<RefCell<World<GuiComponentMgr, ()>>>) -> Node {
        Node{
            id,
            world
        }
    }
}

#[wasm_bindgen]
impl Node {

    #[wasm_bindgen(js_name = appendChild)]
    pub fn append_child(&self, node: Node) {
        let mut world = self.world.borrow_mut();
        
    }

    #[wasm_bindgen(js_name = removeChild)]
    pub fn remove_child(&self) {

    }	

    // #[wasm_bindgen(js_name = replaceChild)]
    // pub fn replaceChild(&self) {

    // }

    #[wasm_bindgen(js_name = insertBefore)]
    pub fn insert_before(&self) {

    }

    #[wasm_bindgen(js_name = createElement)]
    pub fn create_node(&self) -> Node {
        let world_copy = self.world.clone();
        let mut world = self.world.borrow_mut();
        let node = CNode::default();
        let index = world.component_mgr.node._group.insert(node, 0);
        Node::new(index, world_copy)
    }

    pub fn style(&self) -> Style{
        unimplemented!()
    }

    pub fn attributes(&self) -> Attributes{
        unimplemented!()
    }

    #[wasm_bindgen(js_name = className)]
    pub fn class_name(&self) -> String{
        unimplemented!()
    }

    #[wasm_bindgen(js_name = tagName)]
    pub fn tag_name(&self) -> String{
        unimplemented!()
    }

    pub fn src(&self) -> String{
        unimplemented!()
    }

    #[wasm_bindgen(js_name = textContent)]
    pub fn text_content(&self) -> String{
        unimplemented!()
    }

    #[wasm_bindgen(js_name = setClassName)]
    pub fn set_class_name(&self, _value: &str){

    }

    #[wasm_bindgen(js_name = setSrc)]
    pub fn set_src(&self, _value: &str){
        unimplemented!()
    }

    #[wasm_bindgen(js_name = setTextContent)]
    pub fn set_text_content(&self, _value: &str){
        unimplemented!()
    }
}

#[wasm_bindgen]
pub struct NodeDate{
    builder: NodeBuilder,
    world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

impl NodeDate {
    // pub style() {

    // }

    // pub target() {

    // }
}