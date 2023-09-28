

#![feature(nll)]
#![feature(proc_macro_hygiene)]
#![feature(rustc_const_unstable)]
#![feature(core_intrinsics)]
#![feature(custom_attribute)]
#![feature(type_ascription)]
#![feature(link_args)]
#![feature(vec_remove_item)]
#![allow(unused_attributes)]
#![allow(dead_code)]
#![feature(rustc_private)]
#![recursion_limit = "512"]

#[macro_use]
extern crate serde;
extern crate stdweb_derive;
extern crate webgl_rendering_context;
#[macro_use]
extern crate stdweb;
extern crate ecs;
extern crate gui;
extern crate paste;
#[macro_use]
extern crate ecs_derive;
extern crate map;
#[macro_use]
extern crate debug_info;
extern crate atom;
extern crate bincode;
extern crate pi_cg2d;
extern crate data_view;
extern crate gui_tool;
extern crate hal_core;
extern crate hal_webgl;
extern crate hash;
extern crate octree;
extern crate ordered_float;
extern crate res;
extern crate share;
extern crate idtree;
extern crate flex_layout;

// use std::cell::RefCell;
use std::mem::transmute;

use ordered_float::OrderedFloat;
// use res::ResMgr;
use stdweb::unstable::TryInto;
use stdweb::Object;
use webgl_rendering_context::WebGLRenderingContext;
use flex_layout::{Dimension};


#[cfg(feature = "debug")]
#[derive(Serialize, Debug)]
pub struct RunTime {
    pub draw_text_sys_time: f64,
    pub load_image_time: f64,
    pub run_all_time: f64,
    pub run_sum_time: f64,
    pub sys_time: Vec<(String, f64)>,
}

#[cfg(feature = "debug")]
js_serializable!(RunTime);

fn main(){}