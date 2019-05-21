
/** 
 * 抽象硬件层HAL的WebGL实现
 */

#[macro_use]
extern crate stdweb;

extern crate webgl_rendering_context;

extern crate atom;
extern crate hal_core;

mod extension;
mod context;
mod geometry;
mod render_target;
mod sampler;
mod texture;
// mod shader;

pub use self::context::{WebGLContextImpl};
pub use self::geometry::{WebGLGeometryImpl};
pub use self::render_target::{WebGLRenderTargetImpl, WebGLRenderBufferImpl};
pub use self::sampler::{WebGLSamplerImpl};
pub use self::texture::{WebGLTextureImpl};