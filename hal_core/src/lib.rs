/** 
 * 抽象硬件层HAL 的 核心Trait
 */
extern crate atom;
extern crate share;
extern crate ordered_float;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

mod traits;
mod common;

pub use traits::*;
pub use common::*;