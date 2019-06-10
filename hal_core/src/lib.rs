/** 
 * 抽象硬件层HAL 的 核心Trait
 */

extern crate atom;
extern crate fnv;

mod traits;
mod common;

pub use traits::*;
pub use common::*;