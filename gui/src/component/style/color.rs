use std::ops::{Deref};

use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify};
use wcs::world::{ComponentMgr};

use component::math::{Color as CgColor, ColorReadRef as CgColorReadRef, ColorGroup as CgColorGroup, ColorWriteRef as CgColorWriteRef};

// 颜色， 支持rgb，rgba， 线性渐变， 劲向渐变
#[derive(Debug, Clone, EnumDefault, EnumComponent)]
pub enum Color{
    RGB(CgColor),
    RGBA(CgColor),
    LinearGradient(LinearGradientColor),
    RadialGradient(RadialGradientColor),
}


//颜色，线性渐变
#[derive(Debug, Clone, Component)]
pub struct LinearGradientColor{
    pub direction: f32,
    pub list: Vec<ColorAndPosition>,
}

//颜色， 径向渐变
#[derive(Debug, Clone, Component)]
pub struct RadialGradientColor{
    pub center: (f32, f32),
    pub shape: RadialGradientShape,
    pub size: RadialGradientSize,
    pub list: Vec<ColorAndPosition>,
}

//定义一个颜色和颜色所在的位置， position取值为0 ~ 1
#[derive(Debug, Clone)]
pub struct ColorAndPosition{
    pub rgba: CgColor,
    pub position: f32,
}

//
#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum RadialGradientSize{
    Farthescorner,
    ClosestCorner,
    ClosestSide,
    FarthesSide,
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum RadialGradientShape{
    Ellipse,
    Circle,
}