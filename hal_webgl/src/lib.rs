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

mod wrap;
mod implement;

use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::{Object};

pub use self::wrap::{
    WebGLBufferWrap, 
    WebGLContextWrap, 
    WebGLGeometryWrap, 
    WebGLProgramWrap, 
    WebGLRenderBufferWrap, WebGLRenderTargetWrap, 
    WebGLSamplerWrap, 
    WebGLBlendStateWrap, WebGLDepthStateWrap, WebGLRasterStateWrap, WebGLStencilStateWrap, 
    WebGLTextureWrap
};

/** 
 * fbo用js创建的WebGLFramebuffer，如果为None，说明要渲染到屏幕上；否则用fbo当渲染目标
 * 注：WebGLFramebuffer在小游戏真机上不是真正的Object对象，所以要封装成：{wrap: WebGLFramebuffer}
 */
pub fn create_hal_webgl(context: WebGLRenderingContext, fbo: Option<Object>) -> WebGLContextWrap {
    WebGLContextWrap::new(context, fbo)
}