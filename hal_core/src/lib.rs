/** 
 * 抽象硬件层HAL 的 核心Trait
 */

extern crate atom;
extern crate share;
extern crate fnv;

mod traits;
mod common;

use share::Share;

pub use traits::*;
pub use common::*;

pub type ShareRef<T> = Share<dyn AsRef<T> + 'static>;