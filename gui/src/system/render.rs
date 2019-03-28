// use std::cell::RefCell;
// use std::rc::{Rc};

// use wcs::world::{System};
// use wcs::component::{ComponentHandler, Event};

// use component::style::transform::{Transform};
// use world::GuiComponentMgr;
// use component::math::{Matrix4, Vector3};

// pub struct Render(RefCell<WorldMatrixImpl>);

// impl WorldMatrix {
//     pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<WorldMatrix>{
//         let system = Rc::new(WorldMatrix(RefCell::new(WorldMatrixImpl::new())));
//         component_mgr.node.transform._group.register_modify_field_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Transform, ModifyFieldEvent, GuiComponentMgr>>)));
//         component_mgr.node.transform._group.register_delete_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Transform, DeleteEvent, GuiComponentMgr>>)));
//         component_mgr.node.transform._group.register_create_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Transform, CreateEvent, GuiComponentMgr>>)));
//         component_mgr.node.position._group.register_modify_field_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Vector3, ModifyFieldEvent, GuiComponentMgr>>)));
//         component_mgr.node._group.register_create_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, CreateEvent, GuiComponentMgr>>)));
//         system
//     }
// }