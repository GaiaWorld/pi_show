extern crate gui;
extern crate hal_null;
extern crate ecs;
#[macro_use]
extern crate ecs_derive;
extern crate map;

pub mod yoga_unimpl;

use yoga_unimpl::YgNode;
use hal_null::NullContextImpl;

fn main() { 
    let engine = gui::render::engine::Engine::new(NullContextImpl::new(), 1);
    gui::world::create_world::<NullContextImpl, YgNode>(engine, 1000.0, 1000.0);
}
