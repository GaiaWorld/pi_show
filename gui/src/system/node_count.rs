use std::rc::Rc;

use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent};
use world::GuiComponentMgr;

use component::node::{Node};


pub struct NodeCountSys();

impl NodeCountSys {
  pub fn init(mgr: &mut GuiComponentMgr) -> Rc<NodeCountSys> {
    let rc = Rc::new(NodeCountSys());
    mgr.node._group.register_create_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<Node, CreateEvent, GuiComponentMgr>>),
    ));
    mgr.node._group.register_delete_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<Node, DeleteEvent, GuiComponentMgr>>),
    ));
    rc
  }
}

impl ComponentHandler<Node, CreateEvent, GuiComponentMgr> for NodeCountSys {
  fn handle(&self, event: &CreateEvent, component_mgr: &mut GuiComponentMgr) {
    let CreateEvent{id: _, parent} = event;
    let mut p = *parent;
    while p > 0 {
      let n = component_mgr.node._group.get_mut(p);
      n.count += 1;
      p = n.parent;
    }
  }
}
impl ComponentHandler<Node, DeleteEvent, GuiComponentMgr> for NodeCountSys {
  fn handle(&self, event: &DeleteEvent, component_mgr: &mut GuiComponentMgr) {
    let DeleteEvent{id: _, parent} = event;
    let mut p = *parent;
    while p > 0 {
      let n = component_mgr.node._group.get_mut(p);
      n.count -= 1;
      p = n.parent;
    }
  }
}
