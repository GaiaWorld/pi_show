 
#![feature(const_trait_impl)]
#![feature(proc_macro_hygiene)]
#![feature(core_intrinsics)]
#![feature(type_ascription)]
#![feature(if_let_guard)]
#![allow(unused_attributes)]
#![allow(dead_code)]
#![feature(rustc_private)]
#![allow(non_snake_case)]

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
#[macro_use]
extern crate hal_derive;
pub extern crate paste;
#[macro_use]
extern crate serde;

extern crate share;
extern crate res;
extern crate deque;
extern crate pi_cg2d;
extern crate nalgebra;
extern crate pi_spatial ;
extern crate slab;
extern crate map;
extern crate num_traits;
extern crate heap;
extern crate fnv;
extern crate ucd;
extern crate data_view;
extern crate dirty;
extern crate color;
extern crate util as lib_util;
extern crate hal_core;
extern crate pi_polygon;
extern crate ordered_float;
extern crate hash;
extern crate densevec;
extern crate idtree;
extern crate flex_layout;
extern crate rectangle_pack;
extern crate guillotiere;
extern crate smallvec;
extern crate getset;

#[cfg(feature = "native")]
extern crate cross_performance;
#[cfg(feature = "wasm-bindgen")]
extern crate cross_performance;

pub mod system;
pub mod component;
pub mod single;
pub mod layout;
pub mod font;
pub mod render;
pub mod world;
pub mod util;
// pub mod exports;

pub mod entity{
    pub struct Node;
    // pub struct RenderObj;
    // pub struct Class;
}

pub type IdBind = usize;
pub const Z_MAX: f32 = 65535.0;
// pub const Z_MAX: f32 = 419429.0; // IEEE 754 单精度浮点数，尾数23，所以能表达范围是800万，现在GUI需要精确到0.1，所以这个地方是 正负40万；
// pub const Z_MAX: f32 = 10000.0;
// pub const Z_MAX: f32 = 6000.0;
// pub const Z_MAX: f32 = 1048575; / 2^20 1-1
pub const ROOT: usize = 1;
pub static mut DIRTY: bool = false; // 全局脏， 临时使用
