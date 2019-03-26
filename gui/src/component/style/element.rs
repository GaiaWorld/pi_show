use std::ops::{Deref};

use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify, Builder};
use wcs::world::{ComponentMgr};
use atom::Atom;

use component::style::color::{ColorReadRef, ColorWriteRef, Color, ColorId, ColorGroup};
// use component::style::sdf::*;
// use component::style::shape::*;
use component::style::text::*;
use component::style::font::*;
// use component::style::generic::*;

#[allow(unused_attributes)]
#[derive(Debug, EnumComponent, Builder)]
pub enum Element {
    Rect(#[builder(build(Builder))]Rect),
    // Circle(CircleElem),
    Text(Text),
    Image(Image),
}

// #[allow(unused_attributes)]
// #[derive(Debug, Component, Builder)]
// pub struct RectElem {
//     #[component(Rect)]
//     #[builder(build(Default), export)]
//     pub shape: usize,
//     #[component(SdfStyle)]
//     #[builder(build(Default), export)]
//     pub style: usize
// }

// #[allow(unused_attributes)]
// #[derive(Debug, Component, Builder)]
// pub struct CircleElem {
//     #[component(Circle)]
//     #[builder(build(Default), export)]
//     pub shape: usize,
//     #[component(SdfStyle)]
//     #[builder(build(Default), export)]
//     pub style: usize
// }

// #[allow(unused_attributes)]
// #[derive(Component, Default, Debug, Clone, Builder)]
// pub struct Text{
//     // #[builder(export)]
//     // pub font: RcFont,
//     #[builder(export)]
//     pub text_class: Vec<RcText>,

//     #[builder(export)]
//     pub font_class: Vec<RcFont>,

//     #[builder(export)]
//     pub text: RcText,

//     #[builder(export)]
//     pub font: RcFont,

//     #[builder(export)]
//     pub value: String,
// }

#[derive(Component, Default, Debug, Clone)]
pub struct Image{
    pub url: Atom,
}

#[allow(unused_attributes)]
#[derive(Component, Default, Debug, Clone, Builder)]
pub struct Text{
    pub value: Atom,

    #[builder(export)]
    #[component(TextStyle)]
    pub text_style: usize,

    #[builder(export)]
    #[component(Font)]
    pub font: usize,
}

#[allow(unused_attributes)]
#[derive(Component, Debug, Clone, Builder, Default)]
pub struct Rect{
    #[builder(export)]
    pub radius: f32,

    #[builder(export)]
    #[enum_component(Color)]
    pub color: ColorId,

    #[builder(export)]
    #[enum_component(Color)]
    pub border_color: ColorId,

    #[builder(export)]
    #[component(BoxShadow)]
    pub shadow: usize,
}

#[allow(unused_attributes)]
#[derive(Component, Debug, Clone)]
pub struct BoxShadow{
    h: f32,
    v: f32,
    blur: f32,
    spread: f32,
    #[enum_component(Color)]
    color: ColorId
}