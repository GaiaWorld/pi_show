#![feature(nll)]
#![feature(proc_macro_hygiene)]
#![feature(core_intrinsics)]
#![feature(type_ascription)]
#![allow(unused_attributes)]
#![allow(dead_code)]
#![feature(rustc_private)]
#![recursion_limit = "512"]
#![feature(unboxed_closures)]
#![feature(maybe_uninit_extra)]

#[macro_use]
extern crate serde;

extern crate js_sys;
extern crate web_sys;
extern crate wasm_bindgen;
extern crate ecs;
extern crate gui;
extern crate lazy_static;
pub extern crate paste;
extern crate map;
#[macro_use]
extern crate debug_info;
extern crate atom;
extern crate bincode;
extern crate cg2d;
extern crate data_view;
extern crate gui_tool;
extern crate hal_core;
extern crate hal_webgl;
extern crate hash;
extern crate ordered_float;
extern crate res;
extern crate share;
extern crate idtree;
extern crate flex_layout;
extern crate cross_performance;

mod class;
mod layout;
mod node;
mod style;
mod debug;
mod text;
mod transform;
mod world;
mod index;
// mod res;
mod reset_style;

pub use class::*;
pub use layout::*;
pub use node::*;
pub use style::*;
pub use debug::*;
pub use text::*;
pub use transform::*;
pub use world::*;
pub use index::*;
// pub use res::*;