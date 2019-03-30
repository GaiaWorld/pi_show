#![feature(nll)] 
#![feature(rustc_const_unstable)] 
#![feature(core_intrinsics)]
#![feature(custom_attribute)] 
#![feature(type_ascription)]
#![feature(link_args)]
#[allow(unused_attributes)]

extern crate deque;
extern crate cg;
extern crate wcs;
extern crate slab;
extern crate vecmap;
#[macro_use]
extern crate wcs_macro;
#[macro_use]
extern crate enum_default_macro;
#[macro_use]
extern crate pointer;
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "web")]
#[macro_use]
extern crate stdweb;
#[cfg(feature = "web")]
extern crate webgl_rendering_context;

extern crate num_traits;
extern crate heap;
extern crate fnv;
extern crate ucd;
extern crate atom;

pub mod component;
pub mod world;
pub mod system;
pub mod layout;
pub mod text_layout;

#[cfg(feature = "web")]
pub mod render;

#[cfg(feature = "web")]
pub mod shaders;
// pub mod bind;


//测试
// mod test;
// fn main(){
//     test::yoga::test();
//     test::yoga::test_layout_system();
//     // test::render::test();
// }