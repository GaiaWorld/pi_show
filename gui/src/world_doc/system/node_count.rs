use std::rc::Rc;

use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent};
use wcs::world::System;
use world_doc::WorldDocMgr;

use world_doc::component::node::{Node};


pub struct NodeCountSys();

impl NodeCountSys {
  pub fn init(mgr: &mut WorldDocMgr) -> Rc<NodeCountSys> {
    let rc = Rc::new(NodeCountSys());
    mgr.node._group.register_create_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<Node, CreateEvent, WorldDocMgr>>),
    ));
    mgr.node._group.register_delete_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<Node, DeleteEvent, WorldDocMgr>>),
    ));
    rc
  }
}

impl System<(), WorldDocMgr> for NodeCountSys{
    fn run(&self, _e: &(), _component_mgr: &mut WorldDocMgr){
    }
}

impl ComponentHandler<Node, CreateEvent, WorldDocMgr> for NodeCountSys {
  fn handle(&self, event: &CreateEvent, component_mgr: &mut WorldDocMgr) {
    let CreateEvent{id: _, parent} = event;
    let mut p = *parent;
    while p > 0 {
      let n = component_mgr.node._group.get_mut(p);
      n.count += 1;
      p = n.parent;
    }
  }
}
impl ComponentHandler<Node, DeleteEvent, WorldDocMgr> for NodeCountSys {
  fn handle(&self, event: &DeleteEvent, component_mgr: &mut WorldDocMgr) {
    let DeleteEvent{id: _, parent} = event;
    let mut p = *parent;
    while p > 0 {
      let n = component_mgr.node._group.get_mut(p);
      n.count -= 1;
      p = n.parent;
    }
  }
}
