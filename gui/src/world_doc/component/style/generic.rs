use std::ops::{Deref};
use std::default::Default;

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Handlers, Builder};
use wcs::world::{ComponentMgr};

use component::color::*;
use component::math::{Color as MathColor, ColorReadRef as MathColorReadRef, ColorWriteRef as MathColorWriteRef, ColorGroup as MathColorGroup};

#[derive(Debug, Copy, Clone)]
pub enum StyleUnit{
    Auto,
    UndefinedValue,
    Percentage(f32),
    Length(f32),
}

#[derive(Debug, Copy, Clone)]
pub enum LengthUnit{
    Percentage(f32),
    Length(f32),
}

#[derive(Debug, Clone)]
pub enum Clip{
    MarginBox,
    BorderBox,
    PaddingBox,
    ContentBox,
    Polygon(Polygon),
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pub value: Vec<LengthUnit>,
}

#[derive(Debug, Clone, Component)]
pub struct ClipPath{
    pub value: Clip,
}

impl Deref for ClipPath {
    type Target = Clip;
    fn deref(&self) -> &Clip{
        &self.value
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct Overflow {
    pub x: ShowType,
    pub y: ShowType
}

#[derive(Debug, Copy, Clone)]
pub enum ShowType {
    Visible,
    Hidden,
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum Display {
    Flex,
    Inline,
    None,
}

#[allow(unused_attributes)]
#[derive(Component, Debug, Clone)]
pub struct Opacity {
    pub value: f32
}

impl Default for Opacity {
    fn default() -> Self {
        Opacity{
            value: 1.0,
        }
    }
}

#[allow(unused_attributes)]
#[derive(Component, Debug, Clone, Builder, Default)]
pub struct Decorate {
    #[component(Color)]
    #[builder(export)]
    pub background_color: usize,

    #[listen]
    #[builder(export)]
    pub backgroud_image: usize,

    #[component(BoxShadow)]
    #[builder(export)]
    pub box_shadow: usize,

    #[component(MathColor)]
    #[builder(export)]
    pub border_color: usize,

    #[listen]
    #[builder(export)]
    pub border_radius: f32,
}

#[allow(unused_attributes)]
#[derive(Component, Debug, Clone, Default, Builder)]
pub struct BoxShadow{
    #[builder(export)]
    pub h: f32,
    #[builder(export)]
    pub v: f32,
    #[builder(export)]
    pub blur: f32,
    #[builder(export)]
    pub spread: f32,
    #[builder(export)]
    pub color: MathColor,
}