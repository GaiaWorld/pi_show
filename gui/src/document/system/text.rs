use std::cell::RefCell;
use std::rc::{Rc};

use web_sys::*;
use wcs::world::{System};
use wcs::component::{ComponentHandler, Event};
use slab::Slab;
use fnv::FnvHashMap;

use world::{DocumentMgr};
use generic_component::math::{Matrix4, Vector3};
use document::component::style::element::{Text};
use document::component::node::{Node};
use layout::{YgNode};

pub struct TextSys(RefCell<TextSysImpl>);

impl System<(), DocumentMgr> for TextSys{
    fn run(&self, _e: &(), component_mgr: &mut DocumentMgr){
        self.0.borrow_mut().layout(component_mgr);
    }
}

impl ComponentHandler<Text, DocumentMgr> for TextSys{
    fn handle(&self, event: &Event, _component_mgr: &mut DocumentMgr){
        match event {
            Event::Create{id: _, parent} => {
                self.0.borrow_mut().dirtys.insert(*parent, true);
            },
            Event::Delete{id:_, parent} => {
                self.0.borrow_mut().dirtys.insert(*parent, true);
            },
            Event::ModifyField{id:_, parent, field: _} => {
                self.0.borrow_mut().dirtys.insert(*parent, true);
            },
            _ => ()
        }
    }
}

pub struct TextSysImpl {
    yogas: Vec<YgNode>,
    dirtys: FnvHashMap<usize, bool>,
    // text_type: FnvHashMap<usize, TextType>,
}

//Text类型
enum TextType {
    Text, //直接子节点中只存在文字
    TextAndOther, //直接子节点中存在文字，同时存在其他类型的节点
}

impl TextSysImpl {
    pub fn new() -> TextSysImpl{
        TextSysImpl{
            yogas: Vec::new(),
            dirtys: FnvHashMap::default(),
            // text_type: FnvHashMap::default(),
        }
    }

    pub fn layout(&self, component_mgr: &mut DocumentMgr){
        if self.dirtys.len() == 0 {
            return;
        }

        for (k, v) in self.dirtys.iter(){
            
        }
    }
}