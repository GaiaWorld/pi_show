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
extern crate map;
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
extern crate atom;
extern crate ucd;
extern crate data_view;
#[macro_use]
extern crate debug_info;

pub mod world_doc;
pub mod layout;
pub mod text_layout;
#[cfg(feature = "web")]
mod util;
#[cfg(feature = "web")]
mod font;

#[cfg(feature = "web")]
pub mod render;
#[cfg(feature = "web")]
pub mod bind;
#[cfg(feature = "web")]
pub mod world_2d;

pub mod component;


//测试
#[cfg(feature = "web")]
mod test;
fn main(){
    // test::yoga::test();
    // test::yoga::test_layout_system();
    // test::render::test();
    // test::bind::test();
    // test::bind::test_query();
    // test::bind::test11()
}
