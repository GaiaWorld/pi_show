use std::mem::transmute;
use stdweb::web::html_element::CanvasElement;
use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::web::{
    IParentNode,
    document,
};
use stdweb::unstable::TryInto;

use layout::yoga::YGEdge;

use bind::*;
use bind::node::*;
use bind::layout::*;
use bind::style::*;
use bind::transform::*;

pub fn test(){
    let canvas: CanvasElement = document().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap();
    let gl: WebGLRenderingContext = canvas.get_context().unwrap();

    js!{
        window.__gl = @{&gl};
    }
    let engine = create_engine();

    let world = create_gui(engine, 1000.0,  700.0);
    // set_gui_size(world: u32, 1000.0, 700.0);

    let node2 = create_node(world);
    append_child(world, 1, node2);
    set_width_percent(world, node2, 100.0);
    set_height_percent(world, node2, 100.0);
    set_backgroud_rgba_color(world, node2, 1.0, 0.0, 0.0, 0.5);

    let node3 = create_node(world);
    append_child(world, node2, node3);
    set_width_percent(world, node3, 50.0);
    set_height_percent(world, node3, 70.0);
    set_position(world, node3, unsafe{transmute(YGEdge::YGEdgeLeft)}, 50.0);
    set_backgroud_rgba_color(world, node3, 0.0, 0.0, 1.0, 1.0);
    set_border_color(world, node3, 0.0, 1.0, 0.0, 1.0);
    set_border(world, node3, unsafe{transmute(YGEdge::YGEdgeLeft)}, 10.0);
    set_border(world, node3, unsafe{transmute(YGEdge::YGEdgeRight)}, 10.0);
    set_border(world, node3, unsafe{transmute(YGEdge::YGEdgeTop)}, 10.0);
    set_border(world, node3, unsafe{transmute(YGEdge::YGEdgeBottom)}, 10.0);
    // set_overflow(world, node3, true);
    set_box_shadow_color(world, node3, 0.0, 0.0, 0.0, 0.8);
    set_box_shadow_h(world, node3, 50.0);
    set_box_shadow_v(world, node3, 50.0);

    let node4 = create_node(world);
    append_child(world, node3, node4);
    set_width(world, node4, 480.0);
    set_height(world, node4, 200.0);
    set_margin(world, node4, unsafe{transmute(YGEdge::YGEdgeTop)}, 10.0);
    set_backgroud_rgba_color(world, node4, 0.5, 0.0, 0.5, 1.0);

    let node5 = create_node(world);
    append_child(world, node3, node5);
    set_width(world, node5, 480.0);
    set_height(world, node5, 200.0);
    set_margin(world, node5, unsafe{transmute(YGEdge::YGEdgeTop)}, 10.0);
    set_backgroud_rgba_color(world, node5, 0.5, 0.0, 0.5, 1.0);

    let node6 = create_node(world);
    append_child(world, node3, node6);
    set_width(world, node6, 480.0);
    set_height(world, node6, 200.0);
    set_margin(world, node6, unsafe{transmute(YGEdge::YGEdgeTop)}, 10.0);
    set_backgroud_rgba_color(world, node6, 0.5, 0.0, 0.5, 1.0);

    // let node4 = create_node(world);
    // append_child(world, node2, node4);
    // set_width(world, node4, 400.0);
    // set_height(world, node4, 400.0);
    // set_backgroud_rgba_color(world, node4, 0.0, 1.0, 0.0, 1.0);

    run(world);

    remove_child(world, 3,  6);

    run(world);

    let node6 = create_node(world);
    append_child(world, node3, node6);
    set_width(world, node6, 480.0);
    set_height(world, node6, 200.0);
    set_margin(world, node6, unsafe{transmute(YGEdge::YGEdgeTop)}, 10.0);
    set_backgroud_rgba_color(world, node6, 0.5, 0.0, 0.5, 1.0);

    run(world);
}

pub fn test11(){
    let canvas: CanvasElement = document().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap();
    let gl: WebGLRenderingContext = canvas.get_context().unwrap();

    js!{
        window.__gl = @{&gl};
    }
    let engine = create_engine();

    let world = create_gui(engine, 1000.0,  700.0);
    // set_gui_size(world: u32, 1000.0, 700.0);

    let node2 = create_node(world);
    append_child(world, 1, node2);
    set_width_percent(world, node2, 100.0);
    set_height_percent(world, node2, 100.0);
    set_backgroud_rgba_color(world, node2, 1.0, 0.0, 0.0, 1.0);

    run(world);


    let node3 = create_node(world);
    let node4 = create_node(world);
    append_child(world, node3, node4);
    let node5 = create_node(world);
    append_child(world, node3, node5);
    let node6 = create_node(world);
    append_child(world, node3, node6);
    let node7 = create_node(world);
    append_child(world, node3, node7);
    append_child(world, node2, node3);

    set_width_percent(world, node3, 100.0);
    set_height_percent(world, node3, 100.0);
    set_backgroud_rgba_color(world, node3, 0.0, 0.0, 1.0, 1.0);

    set_width_percent(world, node4, 100.0);
    set_height_percent(world, node4, 100.0);
    set_backgroud_rgba_color(world, node4, 0.0, 0.0, 1.0, 1.0);

    set_width_percent(world, node5, 100.0);
    set_height_percent(world, node5, 100.0);
    set_backgroud_rgba_color(world, node5, 0.0, 1.0, 1.0, 1.0);

    set_width_percent(world, node6, 10.0);
    set_height_percent(world, node6, 10.0);
    set_backgroud_rgba_color(world, node6, 0.0, 0.0, 1.0, 1.0);

    set_width_percent(world, node7, 10.0);
    set_height_percent(world, node7, 10.0);
    set_backgroud_rgba_color(world, node7, 0.0, 0.0, 1.0, 1.0);

    run(world);

    let node8 = create_node(world);
    let node9 = create_node(world);
    append_child(world, node8, node9);
    let node10 = create_node(world);
    append_child(world, node8, node10);
    let node11 = create_node(world);
    append_child(world, node8, node11);
    let node12 = create_node(world);
    append_child(world, node8, node12);
    let node13 = create_node(world);
    append_child(world, node12, node13);
    append_child(world, node7, node8);

    set_width_percent(world, node8, 100.0);
    set_height_percent(world, node8, 100.0);
    set_backgroud_rgba_color(world, node8, 0.0, 0.0, 1.0, 1.0);

    set_width_percent(world, node9, 100.0);
    set_height_percent(world, node9, 100.0);
    set_backgroud_rgba_color(world, node9, 0.0, 0.0, 1.0, 1.0);

    set_width_percent(world, node10, 100.0);
    set_height_percent(world, node10, 100.0);
    set_backgroud_rgba_color(world, node10, 0.0, 1.0, 1.0, 1.0);

    set_width_percent(world, node11, 10.0);
    set_height_percent(world, node11, 10.0);
    set_backgroud_rgba_color(world, node11, 0.0, 0.0, 1.0, 1.0);

    set_width_percent(world, node12, 10.0);
    set_height_percent(world, node12, 10.0);
    set_backgroud_rgba_color(world, node12, 0.0, 0.0, 1.0, 1.0);

    set_width_percent(world, node13, 10.0);
    set_height_percent(world, node13, 10.0);
    set_backgroud_rgba_color(world, node13, 0.0, 1.0, 1.0, 1.0);

    run(world);

    remove_child(world,  12,  13);
    // query(world, 500.0, 500.0, 1);
}

pub fn test_query(){
    let canvas: CanvasElement = document().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap();
    let gl: WebGLRenderingContext = canvas.get_context().unwrap();

    js!{
        window.__gl = @{&gl};
    }
    let mut engine = create_engine();

    let mut world = create_gui(engine, 1000.0, 700.0);
    // set_gui_size(world: u32, 1000.0, 700.0);

    let node2 = create_node(world);
    append_child(world, 1, node2);
    set_width(world, node2, 1000.0);
    set_height(world, node2, 700.0);
    set_backgroud_rgba_color(world, node2, 1.0, 0.0, 0.0, 1.0);

    let node3 = create_node(world);
    append_child(world, node2, node3);
    set_width(world, node3, 500.0);
    set_height(world, node3, 500.0);
    set_position(world, node3, unsafe{transmute(YGEdge::YGEdgeLeft)}, 50.0);
    set_backgroud_rgba_color(world, node3, 0.0, 0.0, 1.0, 1.0);
    set_border_color(world, node3, 0.0, 1.0, 0.0, 1.0);
    set_border(world, node3, unsafe{transmute(YGEdge::YGEdgeLeft)}, 10.0);
    set_border(world, node3, unsafe{transmute(YGEdge::YGEdgeRight)}, 10.0);
    set_border(world, node3, unsafe{transmute(YGEdge::YGEdgeTop)}, 10.0);
    set_border(world, node3, unsafe{transmute(YGEdge::YGEdgeBottom)}, 10.0);
    // set_overflow(world, node3, true);
    // set_box_shadow_color(world, node3, 0.0, 0.0, 0.0, 0.5);
    // set_box_shadow_h(world, node3, 50.0);
    // set_box_shadow_v(world, node3, 50.0);

    let node4 = create_node(world);
    append_child(world, node3, node4);
    set_width(world, node4, 480.0);
    set_height(world, node4, 200.0);
    set_margin(world, node4, unsafe{transmute(YGEdge::YGEdgeTop)}, 10.0);
    set_backgroud_rgba_color(world, node4, 0.5, 0.0, 0.5, 1.0);

    let node5 = create_node(world);
    append_child(world, node3, node5);
    set_width(world, node5, 480.0);
    set_height(world, node5, 200.0);
    set_margin(world, node5, unsafe{transmute(YGEdge::YGEdgeTop)}, 10.0);
    set_backgroud_rgba_color(world, node5, 0.5, 0.0, 0.5, 1.0);

    let node6 = create_node(world);
    append_child(world, node3, node6);
    set_width(world, node6, 480.0);
    set_height(world, node6, 200.0);
    set_margin(world, node6, unsafe{transmute(YGEdge::YGEdgeTop)}, 10.0);
    set_backgroud_rgba_color(world, node6, 0.5, 0.0, 0.5, 1.0);

    // let node4 = create_node(world);
    // append_child(world, node2, node4);
    // set_width(world, node4, 400.0);
    // set_height(world, node4, 400.0);
    // set_backgroud_rgba_color(world, node4, 0.0, 1.0, 0.0, 1.0);

    run(world);

    query(world, 70.0, 80.0, 1);
}