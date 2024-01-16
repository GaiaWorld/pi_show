#![feature(nll)]
#![feature(proc_macro_hygiene)]
#![feature(core_intrinsics)]
#![feature(type_ascription)]
#![feature(link_args)]
#![allow(unused_attributes)]
#![allow(dead_code)]
#![feature(rustc_private)]
#![recursion_limit = "512"]
#![feature(unboxed_closures)]
#![feature(maybe_uninit_extra)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn offset_height(a: u32, b: u32) -> f32 {
	return (a + b) as f32;
}

// #[macro_use]
// extern crate serde;

// extern crate js_sys;
// extern crate web_sys;
// extern crate wasm_bindgen;
// extern crate ecs;
// extern crate gui;
// extern crate lazy_static;
// pub extern crate paste;
// extern crate map;
// #[macro_use]
// extern crate debug_info;
// extern crate atom;
// extern crate bincode;
// extern crate cg2d;
// extern crate data_view;
// extern crate gui_tool;
// extern crate hal_core;
// extern crate hal_webgl;
// extern crate hash;
// extern crate octree;
// extern crate ordered_float;
// extern crate res;
// extern crate share;
// extern crate idtree;
// extern crate flex_layout;
// // extern crate cross_performance;

// mod class;
// mod layout;
// mod node;
// mod style;
// mod debug;
// mod text;
// mod transform;
// mod world;
// mod index;
// // mod res;

// pub use class::*;
// pub use layout::*;
// pub use node::*;
// pub use style::*;
// pub use debug::*;
// pub use text::*;
// pub use transform::*;
// pub use world::*;
// pub use index::*;
// pub use res::*;
// pub use res::*;