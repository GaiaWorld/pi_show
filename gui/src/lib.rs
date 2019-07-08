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
pub extern crate paste;
extern crate serde;
 
extern crate share;
extern crate deque;
extern crate cg2d;
extern crate cgmath;
extern crate octree;
extern crate collision;
extern crate slab;
extern crate map;
extern crate num_traits;
extern crate heap;
extern crate atom;
extern crate ucd;
extern crate data_view;
extern crate dirty;
extern crate color;
extern crate util as lib_util;
extern crate hal_core;
extern crate polygon;
extern crate hashmap;
extern crate ordered_float;
#[cfg(feature = "web")]
#[macro_use]
extern crate stdweb;
extern crate fx_hashmap;
extern crate fxhash;

pub mod system;
pub mod component;
pub mod single;
pub mod layout;
pub mod font;
pub mod render;
pub mod util;
pub mod world;

#[cfg(feature = "web")]
use std::mem::transmute;
#[cfg(feature = "web")]
use stdweb::unstable::TryInto;

pub mod entity{
    pub struct Node;
}

pub use fx_hashmap::FxHashMap32;

pub type IdBind = usize;
pub const Z_MAX: f32 = 419430.0;
// pub const Z_MAX: f32 = 50.0;
pub const ROOT: usize = 1;
pub type HashMap<T> = hashmap::HashMap<usize, T>;

#[cfg(feature = "web")]
pub fn cancel_timeout(id: usize){
    js!{
        clearTimeout(@{id as u32});
    }
}

#[cfg(feature = "web")]
pub fn set_timeout(ms: usize, f: Box<dyn FnOnce()>) -> usize{
    let (x, y): (usize, usize) = unsafe { transmute(f) };
    js!{
        return setTimeout(function(){
            Module._notify_timeout(@{x as u32}, @{y as u32});
        }, @{ms as u32});
    }
    0
}

#[cfg(feature = "web")]
pub fn now_time() -> u64{
    TryInto::<u64>::try_into(js!{
        return Date.now();
    }).unwrap()
}

#[cfg(not(feature = "web"))]
pub fn cancel_timeout(_id: usize){
}

#[cfg(not(feature = "web"))]
pub fn set_timeout(_ms: usize, _f: Box<dyn FnOnce()>) -> usize{
    0
}

#[cfg(not(feature = "web"))]
pub fn now_time() -> u64{
    0
}