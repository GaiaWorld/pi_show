use std::ops::{Deref};

use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify, Builder};
use wcs::world::{ComponentMgr};

use component::style::flex::{Layout, LayoutReadRef, LayoutWriteRef, LayoutGroup};
use component::style::generic::*;
use component::style::text::*;
use component::style::font::*;
use component::style::color::*;
use component::style::transform::*;

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum Display {
    Flex,
    Inline,
    Display,
}

#[allow(unused_attributes)]
#[derive(Debug, Component, Default)]
pub struct Style{
    pub display: Option<Display>,

    #[component(Layout)]
    pub layout: usize,

    #[component(ClipPath)]
    pub clip: usize,

    #[component(Text)]
    pub text: usize,

    #[enum_component(Color)]
    pub rect: ColorId,

    #[component(Transform)]
    pub transform: usize,

    #[component(Opacity)]
    pub opacity: usize,
}

#[allow(unused_attributes)]
#[derive(Component, Default, Debug, Clone)]
pub struct Opacity {
    value: f32
}

#[allow(unused_attributes)]
#[derive(Component, Default, Debug, Clone, Builder)]
pub struct Text{
    #[component(TextStyle)]
    #[builder(export)]
    pub text: usize,

    #[component(Font)]
    #[builder(export)]
    pub font: usize,
}

#[allow(unused_attributes)]
#[derive(Component, Default, Debug, Clone)]
pub struct Image{
    // #[builder(export)]
    // pub font: RcFont,
    pub url: String,
}