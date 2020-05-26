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
#![feature(fnbox)]

extern crate atom;
extern crate hal_null;
extern crate fnv;
extern crate gui;
#[macro_use]
extern crate lazy_static;
extern crate hal_core;
extern crate ecs;
#[macro_use]
extern crate ecs_derive;
extern crate cgmath;
extern crate map;

#[cfg(feature = "web")]
pub mod bc;
#[cfg(feature = "web")]
pub mod yoga;

#[cfg(not(feature = "web"))]
pub mod yoga_unimpl;

#[cfg(feature = "web")]
pub use bc as layout;
#[cfg(not(feature = "web"))]
pub use yoga_unimpl as layout;
// pub mod fetch;
// use fetch::test_time;

pub mod fetch;
use fetch::test_time;

fn main() {
    test_time();
}