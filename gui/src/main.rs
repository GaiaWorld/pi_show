#![feature(nll)] 
#![feature(rustc_const_unstable)] 
#![feature(core_intrinsics)]
#![feature(custom_attribute)] 
#![feature(type_ascription)]
#![feature(link_args)]
#[allow(unused_attributes)]

#[macro_use]
extern crate ecs;
#[macro_use]
extern crate ecs_derive;
#[macro_use]
extern crate pointer;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate derive_deref;
#[macro_use]
extern crate enum_default_macro;

extern crate deque;
extern crate cg;
extern crate slab;
extern crate map;
extern crate num_traits;
extern crate heap;
extern crate fnv;
extern crate atom;
extern crate ucd;
extern crate data_view;
extern crate dirty;

pub mod system;
pub mod component;
pub mod single;
pub mod layout;

pub mod entity{
    pub struct Node;
    pub struct Char;
}
pub type IdBind = usize;
pub const Z_MAX: f32 = 4194304.0;
pub const Root: usize = 1;

fn main(){
    // test::yoga::test();
    // test::yoga::test_layout_system();
    // test::render::test();
    // test::bind::test();
    // test::bind::test_query();
    // test::bind::test11()
}
