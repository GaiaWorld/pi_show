use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify, Builder};
use std::ops::{Deref};
use wcs::world::{ComponentMgr};

#[derive(Debug, Component, Default, Builder)]
pub struct ViewPort{
    pub width: f32,
    pub height: f32,
}
