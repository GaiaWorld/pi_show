/** 
 * 抽象硬件层HAL 的 核心Trait
 */

extern crate atom;

mod traits;
mod common;

use std::sync::Arc;

pub use traits::*;
pub use common::*;

pub type ShareRef<T> = Arc<AsRef<T> + 'static>;