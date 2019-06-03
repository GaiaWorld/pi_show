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

#[macro_use]
extern crate ecs;
#[macro_use]
extern crate ecs_derive;
extern crate pointer;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate derive_deref;
#[macro_use]
extern crate enum_default_macro;
#[macro_use]
extern crate debug_info;
pub extern crate paste;
#[cfg(feature = "web")]
extern crate stdweb_derive;

#[cfg(feature = "web")]
extern crate webgl_rendering_context;
#[cfg(feature = "web")]
#[macro_use]
extern crate stdweb;
 

extern crate deque;
extern crate cg2d;
extern crate cgmath;
extern crate octree;
extern crate collision;
extern crate slab;
extern crate map;
extern crate num_traits;
extern crate heap;
extern crate fnv;
extern crate atom;
extern crate ucd;
extern crate data_view;
extern crate dirty;
extern crate color;
extern crate util as lib_util;
extern crate hal_core;
extern crate polygon;
#[cfg(feature = "web")]
extern crate hal_webgl;

pub mod system;
pub mod component;
pub mod single;
pub mod layout;
pub mod font;
pub mod render;
#[cfg(feature = "web")]
pub mod bind;
pub mod util;
pub mod world;

pub mod entity{
    pub struct Node;
}

pub type IdBind = usize;
pub const Z_MAX: f32 = 419430.0;
pub const ROOT: usize = 1;

fn main(){
    // a
    // test::yoga::test();
    // test::yoga::test_layout_system();
    // test::render::test();
    // test::bind::test();
    // test::bind::test_query();
    // test::bind::test11()
}
