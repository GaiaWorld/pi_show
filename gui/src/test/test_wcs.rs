// //! Test suite for the Web and headless browsers.

// use std::rc::Rc;

// use wcs::world::{World, System};

// use component::component_def::{GuiComponentMgr};
// use system::{layout::Layout, world_matrix::WorldMatrix, oct::Oct};
// use layout::{Direction};
// use component::math::{Aabb3, Point3};

// pub fn create_world() -> World<GuiComponentMgr, ()> {
//     let mut world: World<GuiComponentMgr, ()> = World::new();
//     let max = Point3::new(2000.0, 2000.0, 2000.0);
//     let min = Point3::new(0.0, 0.0, 0.0);

//     let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![Layout::init(&mut world.component_mgr), WorldMatrix::init(&mut world.component_mgr), Oct::init(&mut world.component_mgr, Aabb3::new(min, max))];
//     world.set_systems(systems);

//     {
//         let component_mgr = &mut world.component_mgr;
//         {
//             let mut root = component_mgr.create_node(&0);
//             {
//                 let yoga = root.get_yoga_node_mut();
//                 yoga.set_width(200.0);
//                 yoga.set_height(200.0);
//             }

//             {
//                 let node0 = root.create_child(0);
//                 let yoga = node0.get_yoga_node_mut();
//                 yoga.set_width(50.0);
//                 yoga.set_height(50.0);
//             }

//             {
//                 let node1 = root.create_child(1);
//                 let yoga = node1.get_yoga_node_mut();
//                 yoga.set_width(50.0);
//                 yoga.set_height(50.0);
//             }

//             {
//                 let node2 = root.create_child(2);
//                 let yoga = node2.get_yoga_node_mut();
//                 yoga.set_width(50.0);
//                 yoga.set_height(50.0);
//             }

//             {
//                 let node3 = root.create_child(2);
//                 let yoga = node3.get_yoga_node_mut();
//                 yoga.set_width(50.0);
//                 yoga.set_height(50.0);
//             }

//             {
//                 let node4 = root.create_child(3);
//                 let yoga = node4.get_yoga_node_mut();
//                 yoga.set_width(50.0);
//                 yoga.set_height(50.0);
//             }

//             root.get_yoga_node_mut().calculate_layout(200.0, 200.0, Direction::LTR);
//         };
//     }

//     world.run(());
//     let mut s = "{".to_string();
//     s = s + "node_group: " + format!("{:?}", world.component_mgr.node._group).as_str();
//     s = s + "transform_group:" + format!("{:?}", world.component_mgr.node.transform._group).as_str();
//     s = s + "size_group:" + format!("{:?}", world.component_mgr.node.size._group).as_str();
//     s = s + "world_matrix_group:" + format!("{:?}", world.component_mgr.node.world_matrix._group).as_str();
//     s = s + "}";

//     //let arr = Array::new();
//     //let js_value = format_args!("{}", s).to_string().into();
//     //arr.push(&js_value);
//     // log(&format_args!("{}", s).to_string());
//     //console::log(&arr);
//     world
// }

