use std::ops::{Deref};

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Handlers};
use wcs::world::{ComponentMgr};
use component::math::{Vector2};
use generic_component::color::{Color};

#[allow(unused_attributes)]
#[derive(Debug, Component, Default)]
pub struct Sdf{

    //alpha
    #[listen]
    pub alpha: f32,

    // z深度
    #[listen]
    pub z_depth: f32,

    // 被裁剪
    #[listen]
    pub by_overflow: usize,

    //圓角
    pub radius: usize,

    // blur
    pub blur: f32,

    // 中心點
    pub center: Vector2,

    // extend
    pub extend: Vector2,

    // 旋轉角度
    pub rotate: f32,

    //顏色
    pub color: Color,

    // 邊框寬度
    pub border_size: usize,

    // 邊框顏色
    pub border_color: Color,
}
