use std::ops::{Deref};

use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify};
use wcs::world::{ComponentMgr};

#[derive(Debug, Copy, Clone)]
pub enum StyleUnit{
    Auto,
    UndefinedValue,
    Percentage(f32),
    Length(f32),
}

#[derive(Debug, Copy, Clone)]
pub enum Clip{
    MarginBox,
    BorderBox,
    PaddingBox,
    ContentBox,
}

#[derive(Debug, Copy, Clone, Component)]
pub struct ClipPath{
    value: Clip,
}

impl Deref for ClipPath {
    type Target = Clip;
    fn deref(&self) -> &Clip{
        &self.value
    }
}