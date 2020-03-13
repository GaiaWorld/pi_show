#![feature(duration_float)]

extern crate gui;
#[macro_use]
extern crate ecs;
extern crate map;
extern crate share;
extern crate atom;
extern crate hal_core;
#[macro_use]
extern crate lazy_static;

mod performance;

use hal_core::HalContext;
use gui::world::GuiWorld;
use gui::layout::FlexNode;
pub use performance::PerformanceStatisticians;

pub fn open_performance_inspection<L: FlexNode, C: HalContext>(world: &mut GuiWorld<L, C>, performance_sys: PerformanceStatisticians) {
	PerformanceStatisticians::register_to_world(world, performance_sys);
}

pub fn close_performance_inspection<L: FlexNode, C: HalContext>(world: &mut GuiWorld<L, C>) {
	PerformanceStatisticians::unregister_to_world(&mut world.world);
}

