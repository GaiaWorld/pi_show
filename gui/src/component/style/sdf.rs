use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent};
use std::ops::{Deref};
use wcs::world::{ComponentMgr};

use component::style::color::*;

// 矢量图形的style
// #[allow(unused_attributes)]
// #[derive(Debug, Component, Default)]
// pub struct SdfStyle{

//     #[enum_component(Color)]
//     pub color: ColorId,

//     // TODO 阴影， 颜色渐变
// }

#[allow(unused_attributes)]
#[derive(Component, Debug, Clone)]
pub struct Rect{
    #[builder(export)]
    pub radius: f32,

    #[enum_component(Color)]
    pub color: ColorId,

    #[enum_component(Color)]
    pub border_color: ColorId,

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