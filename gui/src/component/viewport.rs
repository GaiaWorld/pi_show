use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify, Builder};
use std::ops::{Deref};
use wcs::world::{ComponentMgr};

#[derive(Debug, Component, Default, Builder)]
pub struct ViewPort{
    width: f32,
    height: f32,
}
