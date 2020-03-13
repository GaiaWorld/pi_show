/**
 * 抽象硬件层HAL 的 核心Trait
 */
extern crate atom;
extern crate ordered_float;
extern crate share;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

mod common;
mod traits;

pub use common::*;
pub use traits::*;
