extern crate atom;
extern crate hal_null;
extern crate fnv;
extern crate gui;
#[macro_use]
extern crate lazy_static;
extern crate hal_core;
extern crate ecs;
#[macro_use]
extern crate ecs_derive;
extern crate cgmath;
extern crate map;

pub mod yoga;

// pub mod fetch;
// use fetch::test_time;

pub mod trucell;
use trucell::test_time;

fn main() {
    test_time();
}