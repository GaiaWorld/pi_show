#![feature(weak_ptr_eq)]

/** 
 * 抽象硬件层HAL的WebGL实现
 */

#[macro_use]
extern crate stdweb;

extern crate webgl_rendering_context;

extern crate atom;
#[macro_use]
extern crate debug_info;
extern crate hal_core;

mod context;
mod geometry;
mod render_target;
mod sampler;
mod texture;

mod convert;
mod extension;
mod state;
mod shader;

use std::sync::{Arc};
use webgl_rendering_context::{WebGLRenderingContext};

pub use self::sampler::{WebGLSamplerImpl};
pub use self::render_target::{WebGLRenderTargetImpl};
pub use self::geometry::{WebGLGeometryImpl};
pub use self::context::{WebGLContextImpl};
pub use self::texture::{WebGLTextureImpl, WebGLTextureData};

pub fn create_hal_webgl(context: Arc<WebGLRenderingContext>) -> WebGLContextImpl {
    WebGLContextImpl::new(context)
}