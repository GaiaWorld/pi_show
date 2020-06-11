#![feature(weak_ptr_eq)]

/**
 * 抽象硬件层HAL的WebGL实现
 */

#[macro_use]
extern crate stdweb;

extern crate atom;
extern crate deque;
extern crate hash;
extern crate ordered_float;
extern crate share;
extern crate slab;
extern crate webgl_rendering_context;
#[macro_use]
extern crate debug_info;
extern crate hal_core;

mod context;

mod buffer;
mod geometry;
mod program;
mod render_target;
mod texture;

mod convert;
mod extension;
mod shader_cache;
mod state_machine;
mod util;

use webgl_rendering_context::WebGLRenderingContext;

pub use context::WebglHalContext;

/**
 * 注: 苹果最好不要用VAO版本
 */
pub fn create_hal_webgl(gl: WebGLRenderingContext, use_vao: bool) -> WebglHalContext {
    WebglHalContext::new(gl, use_vao)
}
