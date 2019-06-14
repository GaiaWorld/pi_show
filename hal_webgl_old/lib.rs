#![feature(weak_ptr_eq)]

/** 
 * 抽象硬件层HAL的WebGL实现
 */

#[macro_use]
extern crate stdweb;

extern crate webgl_rendering_context;
extern crate ordered_float;

extern crate atom;
#[macro_use]
extern crate debug_info;
extern crate hal_core;
extern crate fnv;

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
pub use self::texture::{WebGLTextureImpl};
pub use self::convert::*;

use stdweb::{Object};

/** 
 * fbo用js创建的WebGLFramebuffer，如果为None，说明要渲染到屏幕上；否则用fbo当渲染目标
 * 注：WebGLFramebuffer在小游戏真机上不是真正的Object对象，所以要封装成：{wrap: WebGLFramebuffer}
 */
pub fn create_hal_webgl(context: Arc<WebGLRenderingContext>, fbo: Option<Object>) -> WebGLContextImpl {
    WebGLContextImpl::new(context, fbo)
}