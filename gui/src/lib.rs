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
#[macro_use]
extern crate wcs_macro;
#[macro_use]
extern crate enum_default_macro;
#[macro_use]
extern crate pointer;
#[macro_use]
extern crate lazy_static;
// #[macro_use]
// extern crate stdweb;
// // #[macro_use]
// extern crate stdweb_derive;
// extern crate webgl_rendering_context;

extern crate num_traits;
extern crate heap;
extern crate fnv;
extern crate ucd;
extern crate atom;

pub mod component;
pub mod world;
pub mod system;
// pub mod render;
pub mod layout;
pub mod text_layout;
// pub mod bind;


//测试
// mod test;
// fn main(){
//     test::yoga::test();
//     test::yoga::test_layout_system();
//     // test::render::test();
// }