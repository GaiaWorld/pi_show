//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
extern crate gui;
use wasm_bindgen_test::*;
use gui::yoga::*;
use gui::{alert};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_yoga() {
    let mut root = YgNode::create();
    root.set_width(200.0);
    root.set_height(200.0);

    let mut node0 = YgNode::create();
    node0.set_width(100.0);
    node0.set_height(100.0);

    root.insert_child(node0.clone_node(), 0);

    root.calculate_layout(200.0, 200.0, Direction::RTL);
    let size = root.get_child(0).get_computed_size();
    alert(("Hello, wasmtest! width:".to_string() + size.x.to_string().as_str() + "height:" + size.y.to_string().as_str()).as_str());
}
