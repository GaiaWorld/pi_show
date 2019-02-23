#![feature(nll)] 
#![feature(rustc_const_unstable)] 
#![feature(core_intrinsics)]
#![feature(custom_attribute)] 
#[allow(unused_attributes)]

extern crate deque;
extern crate cg;
extern crate wcs;
extern crate slab;
#[macro_use]
extern crate wcs_macro;
#[macro_use]
extern crate enum_default_macro;
extern crate num_traits;
extern crate heap;
extern crate fnv;
extern crate web_sys;
extern crate wasm_bindgen;

pub mod component;
pub mod system;
pub mod render;
pub mod layout;
// pub mod render_new;
// pub mod render_wcs;

