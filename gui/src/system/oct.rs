// //包围盒系统

// use std::cell::RefCell;
// use std::rc::{Rc, Weak};

// use wcs::world::{System, ID, ComponentMgr};
// use wcs::component::{EventType, ComponentHandler};
// use cg::{Vector2};

// use components::{NodePoint, GuiComponentMgr, NodeGroup, Vector2Point, RectPoint};

// pub struct Oct(RefCell<OctImpl>);

// impl ComponentHandler<RectPoint, GuiComponentMgr> for Oct{
//     fn handle(&self, event: EventType<RectPoint>, component_mgr: &mut GuiComponentMgr){
//         match event {
//             EventType::ModifyField(point, _) => {
//                 //设置脏列表
//             },
//             EventType::Create(point) => (),
//             EventType::Delete(point) => (),
//             _ => unreachable!(),
//         }
//     }
// }

// impl System<(), GuiComponentMgr> for Oct{
//     fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
//         self.0.borrow_mut().update(&mut component_mgr.node.borrow_mut());
//     }

//     fn init(component_mgr: &mut GuiComponentMgr) -> Rc<Oct>{
//         let oct = Rc::new(Oct(RefCell::new(OctImpl::new())));
//         let node = component_mgr.node.borrow_mut();
//         node.bound_box.borrow_mut()._group.register_handlers(Rc::downgrade(&oct) as Weak<ComponentHandler<RectPoint, GuiComponentMgr>>);
//         oct
//     }
// }

// pub struct OctImpl {
//     dirty_list: Vec<NodePoint>,
// }

// impl OctImpl {
//     pub fn new() -> OctImpl{
//         OctImpl{
//             dirty_list: Vec::new(),
//         }
//     }

//     //更新八叉树
//     pub fn update<M: ComponentMgr>(&mut self, node_group: &mut NodeGroup<M>){
        
//     }
// }

