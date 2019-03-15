// use std::default::Default;
// use std::ops::{Deref};

// use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify, Builder};
// use wcs::world::{ComponentMgr};

// use component::math::*;

// #[allow(unused_attributes)]
// #[derive(Component, Debug, Clone, Builder)]
// pub struct Rect{
//     #[builder(export)]
//     pub radius: f32,
//     #[builder(export)]
//     pub color: f32,
// }

// // impl Default for Rect {
// //     fn default() -> Rect{
// //         Rect{
// //             left_top: Point2::default(),
// //             width: 1.0,
// //             height: 1.0,
// //             radius: 0.0
// //         }
// //     }
// // }

// #[allow(unused_attributes)]
// #[derive(Component, Debug, Clone, Builder, Default)]
// pub struct Circle{
//     #[builder(export)]
//     pub center: Point2,
//     #[builder(export)]
//     pub radius: f32,
// }