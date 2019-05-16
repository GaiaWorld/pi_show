pub mod calc;
pub mod user;

use cg::color::Color as CgColor;

#[derive(Clone, Copy, Debug)]
pub enum LengthUnit {
	Pixel(f32),
	Percent(f32),
}

#[derive(Clone, Copy, Debug)]
pub enum Display{
  Flex,
  None,
}

// 颜色， 支持rgb，rgba， 线性渐变， 劲向渐变
#[derive(Debug, Clone)]
pub enum Color{
    RGB(CgColor<f32>),
    RGBA(CgColor<f32>),
    LinearGradient(LinearGradientColor),
    RadialGradient(RadialGradientColor),
}

impl Color {
    //是否不透明
    pub fn is_opaque(&self) -> bool {
        match self {
            Color::RGB(c) | Color::RGBA(c) => {
                if c.a < 1.0 {
                    return false;
                }
                return true;
            },
            Color::LinearGradient(l) => {
                for c in l.list.iter() {
                    if c.rgba.a < 1.0 {
                    return false;
                    }
                }
                return true;
            },
            Color::RadialGradient(g) => {
                for c in g.list.iter() {
                    if c.rgba.a < 1.0 {
                        return false
                    }
                }
                return true;
            }
        }
    }
}

//颜色，线性渐变
#[derive(Debug, Clone)]
pub struct LinearGradientColor{
    pub direction: f32,
    pub list: Vec<ColorAndPosition>,
}

//颜色， 径向渐变
#[derive(Debug, Clone)]
pub struct RadialGradientColor{
    pub center: (f32, f32),
    pub shape: RadialGradientShape,
    pub size: RadialGradientSize,
    pub list: Vec<ColorAndPosition>,
}

//定义一个颜色和颜色所在的位置， position取值为0 ~ 1
#[derive(Debug, Clone)]
pub struct ColorAndPosition{
    pub rgba: CgColor<f32>,
    pub position: f32,
}

//
#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum RadialGradientSize{
    ClosestSide,
    FarthesSide,
    ClosestCorner,
    Farthescorner,
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum RadialGradientShape{
    Ellipse,
    Circle,
}