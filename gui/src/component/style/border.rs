use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify, Builder};
use std::ops::{Deref};
use wcs::world::{ComponentMgr};

#[derive(Debug, Component, Default, Builder)]
pub struct Border{
    pub value: f32, //暂时只支持统一的border， 可能会分解为left， top， right， bootom
}
