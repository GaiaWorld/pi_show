use std::ops::{Deref};
use std::default::{Default};

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Builder};
use wcs::world::{ComponentMgr};

use text_layout::layout::{LineHeight, WhiteSpace};
use component::color::Color;
use component::math::Color as MathColor;

#[allow(unused_attributes)]
#[derive(Component, Default, Debug, Clone, Builder)]
pub struct TextStyle{
    #[builder(export)]
    pub letter_spacing: f32, //字符间距， 单位：像素
    #[builder(export)]
    pub word_spacing: f32, //字符间距， 单位：像素
    #[builder(export)]
    pub line_height: LineHeight, //设置行高
    #[builder(export)]
    pub text_indent: f32, // 缩进， 单位： 像素
    #[builder(export)]
    pub white_space: WhiteSpace, //空白处理
    #[builder(export)]
    pub color: Color, //颜色
    #[builder(export)]
    #[component(Shadow)]
    pub shadow: usize,
    #[builder(export)]
    pub stroke: Stroke,
    #[builder(export)]
    pub vertical_align: VerticalAlign,
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

// #[derive(Component, Default, Debug, Clone)]
// pub struct OutLine{
//     pub thickness: f32, //	必需。轮廓的粗细。
//     pub blur: f32, //	可选。轮廓的模糊半径。
//     pub color: MathColor, //	必需。轮廓的颜色。参阅 CSS 颜色值。
// }

#[derive(Component, Default, Debug, Clone)]
pub struct Stroke{
    pub width: f32, //	描边宽度
    pub color: MathColor, //	描边颜色
}

#[allow(unused_attributes)]
#[derive(Component, Default, Debug, Clone, Builder)]
pub struct Shadow{
    #[builder(export)]
    pub h: f32, //	必需。水平阴影的位置。允许负值。	测试
    #[builder(export)]
    pub v: f32, //	必需。垂直阴影的位置。允许负值。	测试
    #[builder(export)]
    pub blur: f32, //	可选。模糊的距离。	测试
    #[builder(export)]
    pub color: MathColor, //	可选。阴影的颜色。参阅 CSS 颜色值。
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum VerticalAlign{
    Top,
    Middle,
    Bottom
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum TextDirection{
    Left,
    Right,
    Top,
    Bootom
}