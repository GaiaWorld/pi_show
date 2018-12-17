extern crate gui;
extern crate wcs;

use std::rc::Rc;

use gui::components::{GuiComponentMgr, NodePoint, LayoutPoint, Node, Layout};
use gui::system::world_matrix::{WorldMatrix};
use wcs::world::{World, System};
use wcs::component::{ComponentHandler};

#[test]
fn test(){
    let mut world: World<GuiComponentMgr, ()> = World::new();
    let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![WorldMatrix::init(&mut world.component_mgr.borrow_mut())];
    world.set_systems(systems);
    
    {
        let mut component_mgr = world.component_mgr.borrow_mut();
        let mut layout = {
            let mut node = component_mgr.add_node(Node::default());
            node.set_layout(Layout::default());
            node.get_layout()
        };
        layout.set_left(5);
    }

    world.run(());
}