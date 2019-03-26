use std::ops::{Deref};

use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify, Builder};
use wcs::world::{ComponentMgr};

use component::style::flex::{Layout, LayoutReadRef, LayoutWriteRef, LayoutGroup};
// use component::style::generic::*;
// use component::style::text::*;
// use component::style::font::*;
// use component::style::color::*;
use component::style::transform::*;

// #[derive(Debug, Clone, Copy, EnumDefault)]
// pub enum Display {
//     Flex,
//     Inline,
//     Display,
//     None,
// }

// #[allow(unused_attributes)]
// #[derive(Component, Default, Debug, Clone)]
// pub struct Opacity {
//     value: f32
// }

// #[allow(unused_attributes)]
// #[derive(Debug, Component, Default, Builder)]
// pub struct Style{
//     #[builder(export)]
//     pub display: Option<Display>,

//     #[builder(export)]
//     #[component(Layout)]
//     pub layout: usize,

//     // #[builder(export)]
//     // #[component(ClipPath)]
//     // pub clip: usize,

//     // #[builder(export)]
//     // #[component(Overflow)]
//     // pub overflow: usize,

//     // #[builder(export)]
//     // #[component(Text)]
//     // pub text: usize,

//     // #[builder(export)]
//     // #[enum_component(Color)]
//     // pub rect_color: ColorId,

//     #[builder(export)]
//     #[component(Transform)]
//     pub transform: usize,

//     #[builder(export)]
//     #[component(Opacity)]
//     pub opacity: usize,
// }


// #[allow(unused_attributes)]
// #[derive(Component, Default, Debug, Clone)]
// pub struct Image{
//     // #[builder(export)]
//     // pub font: RcFont,
//     pub url: String,
// }