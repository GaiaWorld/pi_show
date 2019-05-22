
/** 
 * 抽象硬件层HAL的WebGL实现
 */

#[macro_use]
extern crate stdweb;

extern crate webgl_rendering_context;

extern crate atom;
extern crate hal_core;

mod context;
mod geometry;
mod render_target;
mod sampler;
mod texture;

mod convert;
mod extension;
// mod shader;

use self::context::{WebGLContextImpl};

use std::sync::{Arc};
use webgl_rendering_context::{WebGLRenderingContext};

pub fn create_hal_webgl(context: Arc<WebGLRenderingContext>) -> WebGLContextImpl {
    WebGLContextImpl::new(context)
}