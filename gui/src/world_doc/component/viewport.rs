use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Builder};
use std::ops::{Deref};
use wcs::world::{ComponentMgr};

#[derive(Debug, Component, Default, Builder)]
pub struct ViewPort{
    pub width: f32,
    pub height: f32,
}
