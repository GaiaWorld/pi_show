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

pub mod component;
pub mod system;

