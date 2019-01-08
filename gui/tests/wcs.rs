//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
extern crate gui;
extern crate wcs;

use std::rc::Rc;

use wasm_bindgen_test::*;
use wcs::world::{World, System};

use gui::components::{GuiComponentMgr, Node};
use gui::system::world_matrix::WorldMatrix;
use gui::yoga::{Direction};
use gui::alert;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
pub fn test_wcs() {
    let mut world: World<GuiComponentMgr, ()> = World::new();
    let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![WorldMatrix::init(&mut world.component_mgr.borrow_mut())];
    world.set_systems(systems);

    {
        let mut component_mgr = world.component_mgr.borrow_mut();
        {
            let mut root = component_mgr.add_node(Node::default());
            {
                let yoga = root.get_yoga_node_mut();
                yoga.set_width(200.0);
                yoga.set_height(200.0);
            }

            let mut node0 = root.add_child(Node::default(), 0);
            {
                let yoga = node0.get_yoga_node_mut();
                yoga.set_width(100.0);
                yoga.set_height(100.0);
            }

            let mut node1 = root.add_child(Node::default(), 1);
            {
                let yoga = node1.get_yoga_node_mut();
                yoga.set_width(100.0);
                yoga.set_height(100.0);
            }

            root.get_yoga_node_mut().calculate_layout(200.0, 200.0, Direction::LTR);
        };
    }

    world.run(());

    alert("ccccccccccccccccc");
}

