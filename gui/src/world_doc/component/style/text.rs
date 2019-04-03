use std::ops::{Deref};
use std::default::{Default};

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent};
use wcs::world::{ComponentMgr};

use text_layout::{TextAlign, LineHeight, WhiteSpace};
use component::color::Color;

#[derive(Component, Default, Debug, Clone)]
pub struct TextStyle{
    pub text_align: Option<TextAlign>, //对齐方式
    pub letter_spacing: Option<f32>, //字符间距， 单位：像素
    pub line_height: Option<LineHeight>, //设置行高
    pub text_indent: Option<f32>, // 缩进， 单位： 像素
    pub white_space: Option<WhiteSpace>, //空白处理
    pub color: Option<Color>, //颜色
    pub shadow: Option<Shadow>,
    pub out_line: Option<OutLine>,
}

rc!(#[derive(Debug)]RcText, TextStyle, TEXT_STYLE_SLAB);
impl Default for RcText {
    fn default() -> RcText {
        RcText::new(TextStyle::default())
    }
}

// impl TextLayout for TextStyle {
//     fn text_align(&self) -> TextAlign {
//         self.text_align
//     } 
//     fn letter_spacing(&self) -> f32 {
//         self.letter_spacing
//     }
//     fn line_height(&self) -> LineHeight {
//         self.line_height
//     }
//     fn text_indent(&self) -> f32 {
//         self.text_indent
//     }
//     fn white_space(&self) -> WhiteSpace {
//         self.white_space
//     }
// }

#[derive(Component, Default, Debug, Clone)]
pub struct OutLine{
    thickness: f32, //	必需。轮廓的粗细。
    blur: f32, //	可选。轮廓的模糊半径。
    color: cg::color::Color<f32>, //	必需。轮廓的颜色。参阅 CSS 颜色值。
}

#[derive(Component, Default, Debug, Clone)]
pub struct Shadow{
    h: f32, //	必需。水平阴影的位置。允许负值。	测试
    v: f32, //	必需。垂直阴影的位置。允许负值。	测试
    blur: f32, //	可选。模糊的距离。	测试
    color: cg::color::Color<f32>, //	可选。阴影的颜色。参阅 CSS 颜色值。
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum VerticalAlign{
    Center,
    Top,
    Bootom
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum TextDirection{
    Left,
    Right,
    Top,
    Bootom
}