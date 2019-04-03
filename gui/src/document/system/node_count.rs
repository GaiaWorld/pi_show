use std::rc::Rc;

use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent};
use document::DocumentMgr;

use document::component::node::{Node};


pub struct NodeCountSys();

impl NodeCountSys {
  pub fn init(mgr: &mut DocumentMgr) -> Rc<NodeCountSys> {
    let rc = Rc::new(NodeCountSys());
    mgr.node._group.register_create_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<Node, CreateEvent, DocumentMgr>>),
    ));
    mgr.node._group.register_delete_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<Node, DeleteEvent, DocumentMgr>>),
    ));
    rc
  }
}

impl ComponentHandler<Node, CreateEvent, DocumentMgr> for NodeCountSys {
  fn handle(&self, event: &CreateEvent, component_mgr: &mut DocumentMgr) {
    let CreateEvent{id: _, parent} = event;
    let mut p = *parent;
    while p > 0 {
      let n = component_mgr.node._group.get_mut(p);
      n.count += 1;
      p = n.parent;
    }
  }
}
impl ComponentHandler<Node, DeleteEvent, DocumentMgr> for NodeCountSys {
  fn handle(&self, event: &DeleteEvent, component_mgr: &mut DocumentMgr) {
    let DeleteEvent{id: _, parent} = event;
    let mut p = *parent;
    while p > 0 {
      let n = component_mgr.node._group.get_mut(p);
      n.count -= 1;
      p = n.parent;
    }
  }
}
