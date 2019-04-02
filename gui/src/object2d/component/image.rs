use std::ops::{Deref};

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Handlers};
use wcs::world::{ComponentMgr};
use atom::Atom;

use component::math::{Vector2};

#[allow(unused_attributes)]
#[derive(Debug, Component, Default)]
pub struct Image{

    //alpha
    #[listen]
    pub alpha: f32,

    // z深度
    #[listen]
    pub z_depth: f32,

    // 被裁剪
    #[listen]
    pub by_overflow: usize,

    // 中心點
    pub center: Vector2,

    // extend
    pub extend: Vector2,

    // 旋轉角度
    pub rotate: f32,

    //url
    pub url: Atom,
}
