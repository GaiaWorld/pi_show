// use std::rc::Rc;

// use wcs::component::{ComponentHandler, Event};
// use world::GuiComponentMgr;
// use wcs::world::System;

// use component::node::{Node};


// pub struct NodeCount();

// impl NodeCount {
//   pub fn init(mgr: &mut GuiComponentMgr) -> Rc<NodeCount> {
//     let rc = Rc::new(NodeCount());
//     mgr.node._group.register_handler(Rc::downgrade(
//       &(rc.clone() as Rc<ComponentHandler<Node, GuiComponentMgr>>),
//     ));
//     rc
//   }
// }

// impl ComponentHandler<Node, GuiComponentMgr> for NodeCount {
//   fn handle(&self, event: &Event, component_mgr: &mut GuiComponentMgr) {
//     match event {
//       Event::Create {id:_, parent} => {
//         let mut p = *parent;
//         while p > 0 {
//           let n = component_mgr.node._group.get_mut(p);
//           n.count += 1;
//           p = n.parent;
//         }
//       },
//       Event::Delete {id:_, parent} => {
//         let mut p = *parent;
//         while p > 0 {
//           let n = component_mgr.node._group.get_mut(p);
//           n.count -= 1;
//           p = n.parent;
//         }
//       },
//       _ => {
//         unreachable!();
//       }
//     }
//   }
// }
// // TODO 应该可以不要
// impl System<(), GuiComponentMgr> for NodeCount {
//   fn run(&self, _e: &(), mgr: &mut GuiComponentMgr) {

//   }
// }