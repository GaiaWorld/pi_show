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

use self::context::{NullContextImpl};

pub fn create_hal_null() -> NullContextImpl {
    NullContextImpl::new()
}