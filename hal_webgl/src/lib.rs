#![feature(weak_ptr_eq)]

/** 
 * 抽象硬件层HAL的WebGL实现
 */

#[macro_use]
extern crate stdweb;

extern crate webgl_rendering_context;
extern crate ordered_float;
extern crate slab;
extern crate atom;
extern crate share;
#[macro_use]
extern crate debug_info;
extern crate hal_core;
extern crate fnv;


mod context;

mod buffer;
mod geometry;
mod program;
mod render_target;
mod sampler;
mod state;
mod texture;

mod util;
mod convert;

use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::{Object};

use context::{WebglHalContext};

/** 
 * fbo用js创建的WebGLFramebuffer，如果为None，说明要渲染到屏幕上；否则用fbo当渲染目标
 * 注：WebGLFramebuffer在小游戏真机上不是真正的Object对象，所以要封装成：{wrap: WebGLFramebuffer}
 */
pub fn create_hal_webgl(gl: WebGLRenderingContext, fbo: Option<Object>) -> WebglHalContext {
    WebglHalContext::new(gl, fbo)
}