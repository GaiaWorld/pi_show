#![feature(link_args)]

pub mod yoga;
extern crate stdweb;

use yoga::*;

fn main() {
    let node = yg_node_new();
    yg_node_style_set_flex_direction(node, YGFlexDirection::YGFlexDirectionRow);
    yg_node_style_set_width(node, 200.0);
    yg_node_style_set_height(node, 200.0);

	let node2 = yg_node_new();
    yg_node_style_set_width(node2, 50.0);
    yg_node_style_set_height(node2, 50.0);
    yg_node_insert_child(node, node2, 0);
	yg_node_style_set_position(node2, YGEdge::YGEdgeRight, 20.0);
    yg_node_style_set_align_items(node, YGAlign::YGAlignFlexStart);

	yg_node_calculate_layout(node,200.0,200.0,YGDirection::YGDirectionLTR);

	println!("node,layout: {:?}",  get_layout(node));
    println!("node2,layout: {:?}",  get_layout(node2));
}

// fn main() {
//     let node = yg_node_new();
//     yg_node_style_set_flex_direction(node, YGFlexDirection::YGFlexDirectionRow);
//     yg_node_style_set_width(node, 200.0);
//     yg_node_style_set_height(node, 200.0);
//     yg_node_style_set_flex_wrap(node, YGWrap::YGWrapNoWrap);


//     let node2 = yg_node_new();
//     yg_node_style_set_width(node2, 50.0);
//     yg_node_style_set_height(node2, 50.0);
//     yg_node_insert_child(node, node2, 0);
//     yg_node_style_set_align_items(node, YGAlign::YGAlignFlexStart);

//     // let text_node1 = yg_node_new();
//     // println!("text_node1: {:?}", text_node1);
//     // yg_node_set_measure_func(text_node1, Some(callback));
//     // yg_node_insert_child(node, text_node1, 0);

//     let node1 = yg_node_new();
//     yg_node_style_set_position_type(node1, YGPositionType::YGPositionTypeRelative);
//     yg_node_insert_child(node, node1, 1);
//     yg_node_style_set_width(node1, 50.0);
//     yg_node_style_set_height(node1, 50.0);
//     yg_node_style_set_align_items(node, YGAlign::YGAlignFlexStart);

//     let text_node2 = yg_node_new();
//     yg_node_set_measure_func(text_node2, Some(callback));
//     yg_node_insert_child(node1, text_node2, 0);
//     println!("text_node2: {:?}, ty: {:?}", text_node2, yg_node_get_node_type(text_node2));

//     yg_node_calculate_layout(node,200.0,200.0,YGDirection::YGDirectionLTR);

//     println!("node,layout: {:?}",  get_layout(node));
//     println!("node2,layout: {:?}",  get_layout(node2));
//     // println!("text_node1,layout: {:?}", yg_node_layout_get_width(text_node1), yg_node_layout_get_height(text_node1));
//     println!("node1,layout: {:?}",  get_layout(node1));
//     println!("text_node2,layout: {:?}",  get_layout(text_node2));

// }

extern "C" fn callback(node: YGNodeRef, width: f32, width_mode: YGMeasureMode, height: f32, height_mode: YGMeasureMode) -> YGSize {
    println!("node: {:?}, width: {}, widthMode: {:?}, height: {}, heightMode: {:?}", node, width, width_mode, height, height_mode);
    println!("parent: {:?}", get_layout(yg_node_get_parent(node)));
    println!("layout: {:?}", get_layout(node));
    yg_node_style_set_max_width(node, width);
    yg_node_style_set_max_height(node, height);
    YGSize { 
        width: 200.0, 
        height: 200.0, 
    }
}

fn get_layout(node: YGNodeRef) -> Layout {
    Layout{
        left: yg_node_layout_get_left(node),
        top: yg_node_layout_get_top(node),
        width: yg_node_layout_get_width(node),
        height: yg_node_layout_get_height(node),
        border_left: yg_node_layout_get_border(node, YGEdge::YGEdgeLeft),
        border_top: yg_node_layout_get_border(node, YGEdge::YGEdgeTop),
        border_right: yg_node_layout_get_border(node, YGEdge::YGEdgeRight),
        border_bottom: yg_node_layout_get_border(node, YGEdge::YGEdgeBottom),
        padding_left: yg_node_layout_get_padding(node, YGEdge::YGEdgeLeft),
        padding_top: yg_node_layout_get_padding(node, YGEdge::YGEdgeTop),
        padding_right: yg_node_layout_get_padding(node, YGEdge::YGEdgeRight),
        padding_bottom: yg_node_layout_get_padding(node, YGEdge::YGEdgeBottom),
    }
}


#[derive(Debug)]
pub struct Layout{
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub border_left: f32,
    pub border_top: f32,
    pub border_right: f32,
    pub border_bottom: f32,
    pub padding_left: f32,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
}