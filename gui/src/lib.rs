#![feature(rustc_const_unstable)] 
#![feature(core_intrinsics)]
#![feature(custom_attribute)] 
#[allow(unused_attributes)]

extern crate cfg_if;
extern crate wasm_bindgen;
extern crate fast_deque;

extern crate cg;
extern crate wcs;
#[macro_use]
extern crate wcs_macro;

// #[macro_use]
// extern crate lazy_static;

pub mod component;
pub mod system;
pub mod layout;

mod utils;
pub mod test;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;


// #[link(name = "Project1")]
// extern {
//     fn get_int() -> i32;
// }

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub extern {
    pub fn alert(s: &str);
}

