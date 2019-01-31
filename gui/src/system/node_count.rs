use std::rc::Rc;

use wcs::component::{ComponentHandler, Event};

use component::component_def::{GuiComponentMgr, NodePoint};

pub struct NodeCount();

impl NodeCount {
  pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<NodeCount> {
    let rc = Rc::new(NodeCount());
    component_mgr.node._group.register_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<NodePoint, GuiComponentMgr>>),
    ));
    rc
  }
}

impl ComponentHandler<NodePoint, GuiComponentMgr> for NodeCount {
  fn handle(&self, event: &Event<NodePoint>, component_mgr: &mut GuiComponentMgr) {
    match event {
      Event::Create { point: _, parent} => {
        let mut p = *parent;
        while p > 0 {
          let n = component_mgr.node._group.get_mut(&p);
          n.count += 1;
          p = n.parent;
        }
      },
      Event::Delete { point:_, parent} => {
        let mut p = *parent;
        while p > 0 {
          let n = component_mgr.node._group.get_mut(&p);
          n.count -= 1;
          p = n.parent;
        }
      },
      _ => {
        unreachable!();
      }
    }
  }
}
