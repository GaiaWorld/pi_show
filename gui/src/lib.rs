#![feature(nll)] 
#![feature(rustc_const_unstable)] 
#![feature(core_intrinsics)]
#![feature(custom_attribute)] 
#[allow(unused_attributes)]

extern crate deque;
//extern crate cg;
extern crate wcs;
extern crate slab;
#[macro_use]
extern crate wcs_macro;
#[macro_use]
extern crate enum_default_macro;
#[macro_use]
extern crate pointer;
#[macro_use]
extern crate lazy_static;
extern crate num_traits;
extern crate heap;
extern crate fnv;
extern crate web_sys;
extern crate wasm_bindgen;
extern crate ucd;

pub mod component;
pub mod world;
pub mod system;
pub mod render;
pub mod layout;
pub mod text_layout;
// pub mod bindgen;
// pub mod render_wcs;
// pub mod render_new;
// pub mod render_wcs;

