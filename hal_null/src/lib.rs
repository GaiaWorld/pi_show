/** 
 * 抽象硬件层（HAL）的空实现
 * 
 */

extern crate atom;
extern crate hal_core;

mod context;
mod geometry;
mod render_target;
mod sampler;
mod texture;

pub use self::context::{NullContextImpl};
pub use self::geometry::{NullGeometryImpl};
pub use self::render_target::{NullRenderTargetImpl, NullRenderBufferImpl};
pub use self::sampler::{NullSamplerImpl};
pub use self::texture::{NullTextureImpl};