use std::ops::{Deref};
use std::default::Default;

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent};
use wcs::world::{ComponentMgr};
use atom::Atom;

use component::color::*;

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
    Display,
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
#[derive(Component, Debug, Clone)]
pub struct BackGround {
    #[component(Color)]
    color: usize,
    image: Option<Atom>,
}