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
#[macro_use]
extern crate debug_info;
// #[macro_use]
// extern crate stdweb_derive;

// extern crate webgl_rendering_context;
// #[macro_use]
// extern crate stdweb;

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
extern crate hal_core;

pub mod system;
pub mod component;
pub mod single;
pub mod layout;
pub mod font;
pub mod render;
// pub mod bind;
pub mod util;

pub mod entity{
    pub struct Node;
    pub struct Char;
}

use ecs::{World};
use ecs::idtree::IdTree;
use component::user::*;
use component::calc::*;
use layout::Layout;
use entity::Node;

pub type IdBind = usize;
pub const Z_MAX: f32 = 4194304.0;
pub const Root: usize = 1;

pub fn create_world() -> World{
    let mut world = World::default();

    //user
    world.register_entity::<Node>();
    world.register_multi::<Node, Transform>();
    world.register_multi::<Node, WorldMatrix>();
    world.register_multi::<Node, ZIndex>();
    world.register_multi::<Node, BoxColor>();
    world.register_multi::<Node, BoxShadow>();
    world.register_multi::<Node, BorderImage>();
    world.register_multi::<Node, BorderRadius>();
    world.register_multi::<Node, Overflow>();
    world.register_multi::<Node, Show>();

    //calc
    world.register_multi::<Node, ZDepth>();
    world.register_multi::<Node, Enable>();
    world.register_multi::<Node, Visibility>();
    world.register_multi::<Node, WorldMatrix>();
    world.register_multi::<Node, Layout>();

    //single
    world.register_single::<IdTree>(IdTree::default());
    
    world
}

fn main(){
    // a
    // test::yoga::test();
    // test::yoga::test_layout_system();
    // test::render::test();
    // test::bind::test();
    // test::bind::test_query();
    // test::bind::test11()
}
