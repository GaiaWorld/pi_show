// use std::mem::{uninitialized, forget};
use std::rc::Rc;
use std::os::raw::{c_void};

use stdweb::web::html_element::CanvasElement;
use stdweb::*;
use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::web::{
    IParentNode,
};
use stdweb::unstable::TryInto;

use wcs::world::{World, System};
use wcs::component::Builder;

use world_doc::WorldDocMgr;
use world_doc::system::layout::Layout;
use world_doc::component::node::{NodeBuilder, InsertType};
use layout::{YgNode, YGDirection, YGFlexDirection, YGEdge};

// pub fn test_layout_system(){

//     let mut world = new_world();
//     let node1 = NodeBuilder::new().build(&mut world.component_mgr.node);
//     let node2 = NodeBuilder::new().build(&mut world.component_mgr.node);
//     let node3 = NodeBuilder::new().build(&mut world.component_mgr.node);
//     let node4 = NodeBuilder::new().build(&mut world.component_mgr.node);

//     node1.yoga.set_width(100.0);
//     node1.yoga.set_height(100.0);
//     node2.yoga.set_width(200.0);
//     node2.yoga.set_height(200.0); 
//     node3.yoga.set_width(300.0);
//     node3.yoga.set_height(300.0); 
//     node4.yoga.set_width(400.0);
//     node4.yoga.set_height(500.0);

//     js!{
//         console.log("11111111111111111111111111111111111111");
//     }

//     world.component_mgr.set_size(500.0, 500.0);
//     let (root, root_yoga, node_ids) = {
//         let root = NodeBuilder::new().build(&mut world.component_mgr.node);
//         let root_yoga = root.yoga;
//         let mut root_ref = world.component_mgr.add_node(root);
//         (   
//             root_ref.id,
//             root_yoga,
//             [
//                 root_ref.insert_child(node1, InsertType::Back).id,
//                 root_ref.insert_child(node2, InsertType::Back).id,
//                 root_ref.insert_child(node3, InsertType::Back).id,
//                 root_ref.insert_child(node4, InsertType::Back).id,
//             ]
//         )
//     };
//     let yoga_context = Box::into_raw(Box::new(YogaContex {
//         node_id: root,
//         mgr: &world.component_mgr as *const WorldDocMgr as usize,
//     })) as usize;
//     root_yoga.set_context(yoga_context as *mut c_void);
//     root_yoga.set_flex_direction(YGFlexDirection::YGFlexDirectionRow);
//     world.component_mgr.root_id = root;
    
//     js!{
//         console.log("22222222222222222222222222222");
//     }

//     // root_yoga.calculate_layout(500.0, 500.0, YGDirection::YGDirectionLTR);
//     world.run(());
    
//     js!{
//         console.log("333333333333333333333333333");
//     }
//     for i in node_ids.iter(){
//         {
//             let node_ref = world.component_mgr.get_node_mut(*i);
//             let width = node_ref.get_extend().get_width().clone();
//             let height = node_ref.get_extend().get_height().clone();
//             let x = node_ref.get_position().get_x().clone();
//             let y = node_ref.get_position().get_y().clone();

//             let node_s = format!("test_layout_system, node{} position_x:{:?}, position_y:{:?}, width:{:?}, heigth: {:?}", i, x, y, width, height);
//             js!{
//                 console.log(@{node_s} );
//             }
//         }
//     }

//     // forget(world);
// }

// fn new_world() -> World<WorldDocMgr, ()>{
//     let canvas: CanvasElement = world_doc().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap();
//     let gl: WebGLRenderingContext = canvas.get_context().unwrap();

//     let mut world: World<WorldDocMgr, ()> = World::new(WorldDocMgr::new(gl));
//     let systems: Vec<Rc<System<(), WorldDocMgr>>> = vec![Layout::init(&mut world.component_mgr)];
//     world.set_systems(systems);
//     world
// }



// pub fn test(){
//     let gui = YgNode::new();
//     gui.set_width(1000.0);
//     gui.set_height(800.0);

//     let body = YgNode::new();
//     body.set_width_percent(100.0);
//     body.set_height_percent(100.0);
//     gui.insert_child(body.clone(), 0);

//     let root = YgNode::new();
//     root.set_width(500.0);
//     root.set_height_percent(10.0);
//     root.set_flex_direction(YGFlexDirection::YGFlexDirectionRow);
//     body.insert_child(root.clone(), 0);
//     // node1.set_border(YGEdge::YGEdgeLeft, 10.0);
//     // node1.set_border(YGEdge::YGEdgeTop, 10.0);
//     // node1.set_border(YGEdge::YGEdgeRight, 10.0);
//     // node1.set_border(YGEdge::YGEdgeBottom, 10.0);
//     // node1.set_padding(YGEdge::YGEdgeLeft, 10.0);
//     // node1.set_padding(YGEdge::YGEdgeTop, 10.0);
//     // node1.set_padding(YGEdge::YGEdgeRight, 10.0);
//     // node1.set_padding(YGEdge::YGEdgeBottom, 10.0);

//     let node1 = YgNode::new();
//     node1.set_width(700.0);
//     node1.set_height(30.0);
//     node1.set_flex_shrink(1.0);
//     // node2.set_width_auto();
//     // node2.set_border(YGEdge::YGEdgeLeft, 10.0);
//     // node2.set_border(YGEdge::YGEdgeTop, 10.0);
//     // node2.set_border(YGEdge::YGEdgeRight, 10.0);
//     // node2.set_border(YGEdge::YGEdgeBottom, 10.0);
//     // node2.set_padding(YGEdge::YGEdgeLeft, 10.0);
//     // node2.set_padding(YGEdge::YGEdgeTop, 10.0);
//     // node2.set_padding(YGEdge::YGEdgeRight, 10.0);
//     // node2.set_padding(YGEdge::YGEdgeBottom, 10.0);
//     root.insert_child(node1.clone(), 0);

//     let node2 = YgNode::new();
//     node2.set_width(700.0);
//     node2.set_height(30.0);
//     node2.set_flex_shrink(1.0);
//     // node2.set_width_auto();
//     // node2.set_border(YGEdge::YGEdgeLeft, 10.0);
//     // node2.set_border(YGEdge::YGEdgeTop, 10.0);
//     // node2.set_border(YGEdge::YGEdgeRight, 10.0);
//     // node2.set_border(YGEdge::YGEdgeBottom, 10.0);
//     // node2.set_padding(YGEdge::YGEdgeLeft, 10.0);
//     // node2.set_padding(YGEdge::YGEdgeTop, 10.0);
//     // node2.set_padding(YGEdge::YGEdgeRight, 10.0);
//     // node2.set_padding(YGEdge::YGEdgeBottom, 10.0);
//     root.insert_child(node2.clone(), 1);
    

//     // let node3 = YgNode::new();
//     // node3.set_width(100.0);
//     // node3.set_height(100.0);
//     // node3.set_flex_shrink(1.0);
//     // node1.insert_child(node3.clone(), 1);

//     // root.calculate_layout_by_callback(500.0, 500.0, YGDirection::YGDirectionLTR, callback1, 0 as *const c_void);
//     gui.calculate_layout(1000.0, 800.0, YGDirection::YGDirectionLTR);

//     let layout = node1.get_layout();
//     println!("node1_layout: {:?}", layout);

//     let layout = node2.get_layout();
//     println!("node2_layout: {:?}", layout);

//     let layout = root.get_layout();
//     println!("root_layout: {:?}", layout);
// }

pub fn test(){
    let root = YgNode::new();
    root.set_width(500.0);
    root.set_height(40.0);
    root.set_flex_shrink(1.0);
    root.set_flex_direction(YGFlexDirection::YGFlexDirectionRow);

    // let root = YgNode::new();
    // root.set_width(500.0);
    // root.set_height(40.0);
    // gui.insert_child(root.clone(), 0);
    // node1.set_border(YGEdge::YGEdgeLeft, 10.0);
    // node1.set_border(YGEdge::YGEdgeTop, 10.0);
    // node1.set_border(YGEdge::YGEdgeRight, 10.0);
    // node1.set_border(YGEdge::YGEdgeBottom, 10.0);
    // node1.set_padding(YGEdge::YGEdgeLeft, 10.0);
    // node1.set_padding(YGEdge::YGEdgeTop, 10.0);
    // node1.set_padding(YGEdge::YGEdgeRight, 10.0);
    // node1.set_padding(YGEdge::YGEdgeBottom, 10.0);

    let node1 = YgNode::new();
    node1.set_width(700.0);
    node1.set_height(30.0);
    node1.set_flex_shrink(1.0);
    // node2.set_width_auto();
    // node2.set_border(YGEdge::YGEdgeLeft, 10.0);
    // node2.set_border(YGEdge::YGEdgeTop, 10.0);
    // node2.set_border(YGEdge::YGEdgeRight, 10.0);
    // node2.set_border(YGEdge::YGEdgeBottom, 10.0);
    // node2.set_padding(YGEdge::YGEdgeLeft, 10.0);
    // node2.set_padding(YGEdge::YGEdgeTop, 10.0);
    // node2.set_padding(YGEdge::YGEdgeRight, 10.0);
    // node2.set_padding(YGEdge::YGEdgeBottom, 10.0);
    root.insert_child(node1.clone(), 0);

    let node2 = YgNode::new();
    node2.set_width(700.0);
    node2.set_height(30.0);
    node2.set_flex_shrink(1.0);
    // node2.set_width_auto();
    // node2.set_border(YGEdge::YGEdgeLeft, 10.0);
    // node2.set_border(YGEdge::YGEdgeTop, 10.0);
    // node2.set_border(YGEdge::YGEdgeRight, 10.0);
    // node2.set_border(YGEdge::YGEdgeBottom, 10.0);
    // node2.set_padding(YGEdge::YGEdgeLeft, 10.0);
    // node2.set_padding(YGEdge::YGEdgeTop, 10.0);
    // node2.set_padding(YGEdge::YGEdgeRight, 10.0);
    // node2.set_padding(YGEdge::YGEdgeBottom, 10.0);
    root.insert_child(node2.clone(), 1);
    

    // let node3 = YgNode::new();
    // node3.set_width(100.0);
    // node3.set_height(100.0);
    // node3.set_flex_shrink(1.0);
    // node1.insert_child(node3.clone(), 1);

    // root.calculate_layout_by_callback(500.0, 500.0, YGDirection::YGDirectionLTR, callback1, 0 as *const c_void);
    root.calculate_layout(500.0, 40.0, YGDirection::YGDirectionLTR);

    let layout = node1.get_layout();
    println!("node1_layout: {:?}", layout);

    let layout = node2.get_layout();
    println!("node2_layout: {:?}", layout);

    let layout = root.get_layout();
    println!("root_layout: {:?}", layout);
}

extern "C" fn callback1(callback_context: *const c_void, context: *const c_void) {
}
