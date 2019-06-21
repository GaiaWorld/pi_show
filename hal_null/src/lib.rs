/** 
 * 抽象硬件层（HAL）的空实现
 * 
 */

extern crate atom;
extern crate hal_core;
extern crate fnv;

mod context;
mod geometry;
mod render_target;
mod sampler;
mod texture;

pub use context::*;
pub use geometry::*;
pub use render_target::*;
pub use sampler::*;
pub use texture::*;

pub fn create_hal_null() -> NullContextImpl {
    NullContextImpl::new()
}