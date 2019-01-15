// use std::cell::RefCell;
// use std::rc::{Rc};

// use wcs::world::{System, ID, ComponentMgr};
// use wcs::component::{ComponentHandler, EventType};

// use components::{NodePoint, GuiComponentMgr, NodeGroup, RectPoint, Object};
// use alert;

// pub struct ShapeRender(RefCell<ShapeRenderImpl>);

// impl System<(), GuiComponentMgr> for ShapeRender{
//     fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
//         self.0.borrow_mut().render(&mut component_mgr.node.borrow_mut());
//     }

//     fn init(_component_mgr: &mut GuiComponentMgr) -> Rc<ShapeRender>{
//         Rc::new(ShapeRender(RefCell::new(ShapeRenderImpl::new())))
//     }
// }

// pub struct ShapeRenderImpl;

// impl ShapeRenderImpl {
//     pub fn new() -> ShapeRenderImpl{
//         ShapeRenderImpl{
// }
//     }

//     //计算世界矩阵
//     pub fn render<M: ComponentMgr>(&mut self, node_group: &mut NodeGroup<M>){
//         for (index, node) in node_group._group.iter() {
//             match &node.owner.object {
//                 Object::Rect(ref r) => {

//                 },
//                 _ => panic!("cccc"),
//             }
//         }
//     }
// }