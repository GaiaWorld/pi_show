/// 用户操作的组件


use std::{
  sync::Arc,
  default::Default,
  mem::transmute,
};

use map::{vecmap::VecMap};
use hal_core::Context;

use render::res::TextureRes;
use ecs::component::Component;
use atom::Atom;
use component::{LengthUnit, Display, Color, CgColor};
use font::font_sheet::{FontSize, LineHeight};

#[derive(Deref, DerefMut, Component, Default)]
pub struct ZIndex(pub isize);

#[derive(Deref, DerefMut, Component, Default)]
pub struct Overflow(pub bool);

//不透明度
#[derive(Deref, DerefMut, Component, Debug)]
pub struct Opacity(pub f32);

#[derive(Deref, DerefMut, Component, Clone, Debug, PartialEq)]
pub struct Show(pub usize);

#[derive(Debug, Clone, Component, Default)]
pub struct Transform {
    pub funcs: Vec<TransformFunc>,
    pub origin: TransformOrigin,
}

#[derive(Debug, Clone, Component, Default)]
pub struct BoxColor{
	pub background: Color,
	pub border: CgColor,
}

#[derive(Clone, Component)]
pub struct BackgroundImage<C: Context + 'static + Send + Sync>(pub Arc<TextureRes<C>>);

#[derive(Clone, Component)]
pub struct Image<C: Context + 'static + Send + Sync>{
  pub src: Arc<TextureRes<C>>,
}

#[derive(Clone, Component)]
pub struct BorderImage<C: Context + 'static + Send + Sync>(pub Arc<TextureRes<C>>);

#[derive(Debug, Clone, Component)]
pub struct BorderRadius(pub LengthUnit);

#[derive(Debug, Clone, Default, Component)]
pub struct BoxShadow{
    pub h: f32,
    pub v: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: CgColor,
}

#[derive(Debug, Clone, Component, Default)]
pub struct TextStyle{
    pub letter_spacing: f32, //字符间距， 单位：像素
    pub word_spacing: f32, //字符间距， 单位：像素
    pub line_height: LineHeight, //设置行高
    pub indent: f32, // 缩进， 单位： 像素
    pub white_space: WhiteSpace, //空白处理
    pub color: Color, //颜色
    pub stroke: Stroke,
    pub vertical_align: VerticalAlign,
}

#[derive(Debug, Clone, Component, Default)]
pub struct Text(pub Arc<String>);

#[derive(Debug, Clone, Component, Default)]
pub struct TextShadow{
    pub h: f32, //	必需。水平阴影的位置。允许负值。	测试
    pub v: f32, //	必需。垂直阴影的位置。允许负值。	测试
    pub blur: f32, //	可选。模糊的距离。	测试
    pub color: CgColor, //	可选。阴影的颜色。参阅 CSS 颜色值。
}

#[derive(Component, Debug, Clone, Default)]
pub struct Font{
    pub style: FontStyle, //	规定字体样式。参阅：font-style 中可能的值。
    pub weight: f32, //	规定字体粗细。参阅：font-weight 中可能的值。
    pub size: FontSize, //
    pub family: Atom, //	规定字体系列。参阅：font-family 中可能的值。
}

impl Default for Opacity {
  fn default() -> Opacity{
    Opacity(1.0)
  }
}

impl Show {
  #[inline]
  pub fn get_display(&self) -> Display {
    unsafe { transmute((self.0 & (ShowType::Display as usize)) as u8) }
  }

  #[inline]
  pub fn set_display(&mut self, display: Display){
    match display {
      Display::Flex => self.0 &= !(ShowType::Display as usize),
      Display::None => self.0 |= ShowType::Display as usize,
    }
  }

  #[inline]
  pub fn get_visibility(&self) -> bool{
    (self.0 & (ShowType::Visibility as usize)) != 0
  }

  #[inline]
  pub fn set_visibility(&mut self, visibility: bool){
    if visibility {
      self.0 |= ShowType::Visibility as usize;
    }else{
      self.0 &= !(ShowType::Visibility as usize);
    }
  }

  #[inline]
  pub fn get_enable(&self) -> bool{
    (self.0 & (ShowType::Enable as usize)) != 0
  }

  #[inline]
  pub fn set_enable(&mut self, enable: bool){
    if enable {
      self.0 |= ShowType::Enable as usize;
    }else{
      self.0 &= !(ShowType::Enable as usize);
    }
  }
}

impl Default for Show {
  fn default() -> Show {
    Show((ShowType::Enable as usize) | (ShowType::Visibility as usize))
  }
}

#[derive(Debug, Clone)]
pub enum TransformFunc {
    TranslateX(f32),
    TranslateY(f32),
    Translate(f32, f32),

    //平移， 单位： %
    TranslateXPercent(f32),
    TranslateYPercent(f32),
    TranslatePercent(f32, f32),

    ScaleX(f32),
    ScaleY(f32),
    Scale(f32, f32),

    RotateZ(f32),
}

#[derive(Debug, Clone, EnumDefault)]
pub enum TransformOrigin{
    Center,
    XY(LengthUnit, LengthUnit),
}

impl TransformOrigin {
    pub fn to_value(&self, width: f32, height: f32) -> super::Point2 {
        match self {
            TransformOrigin::Center => super::Point2::new(0.5 * width, 0.5 * height),
            TransformOrigin::XY(x, y) => {
                super::Point2::new(
                    match x {
                        LengthUnit::Pixel(v) => v.clone(),
                        LengthUnit::Percent(v) => v * width,
                    },
                    match y {
                        LengthUnit::Pixel(v) => v.clone(),
                        LengthUnit::Percent(v) => v * height,
                    }
                )
            },
        }
    }
}

enum ShowType{
  Display = 1, // 0表示 Flex
  Visibility = 2, // 0表示no Visible
  Enable = 4, // 0表示no Enable
}
impl Transform {
    pub fn matrix(&self, width: f32, height: f32, origin: &super::Point2) -> super::Matrix4 {
        // M = T * R * S
        // let mut m = cg::Matrix4::new(
        //     1.0, 0.0, 0.0, 0.0,
        //     0.0, 1.0, 0.0, 0.0,
        //     0.0, 0.0, 1.0, 0.0,
        //     0.0, 0.0, 0.0, 1.0,
        // );

        let value = self.origin.to_value(width, height);
        let mut m = super::Matrix4::from_translation(super::Vector3::new(origin.x + value.x, origin.y + value.y, 0.0));

        for func in self.funcs.iter() {
            match func {
                TransformFunc::TranslateX(x) => {
                    m = m * super::Matrix4::from_translation(super::Vector3::new(*x, 0.0, 0.0))
                },
                TransformFunc::TranslateY(y) => m = m * super::Matrix4::from_translation(super::Vector3::new(0.0, *y, 0.0)),
                TransformFunc::Translate(x, y) => m = m * super::Matrix4::from_translation(super::Vector3::new(*x, *y, 0.0)),

                TransformFunc::TranslateXPercent(x) => m = m * super::Matrix4::from_translation(super::Vector3::new(*x * width, 0.0, 0.0)),
                TransformFunc::TranslateYPercent(y) => m = m * super::Matrix4::from_translation(super::Vector3::new(0.0, *y * height, 0.0)),
                TransformFunc::TranslatePercent(x, y) => m = m * super::Matrix4::from_translation(super::Vector3::new(*x * width, *y * height, 0.0)),

                TransformFunc::ScaleX(x) => m = m * super::Matrix4::from_nonuniform_scale(*x, 1.0, 1.0),
                TransformFunc::ScaleY(y) => m = m * super::Matrix4::from_nonuniform_scale(1.0, *y, 1.0),
                TransformFunc::Scale(x, y) => m = m * super::Matrix4::from_nonuniform_scale(*x, *y, 1.0),
                
                TransformFunc::RotateZ(z) => m = m * super::Matrix4::from_angle_z(cgmath::Deg(*z)),
            }
        }
        m
    }
}

//对齐元素中的文本
#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum TextAlign{
    Left,	//把文本排列到左边。默认值：由浏览器决定。
    Right,	//把文本排列到右边。
    Center,	//把文本排列到中间。
    Justify,	//实现两端对齐文本效果。
}

//设置元素中空白的处理方式
#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum WhiteSpace{
    Normal, //	默认。空白会被浏览器忽略(其实是所有的空白被合并成一个空格), 超出范围会换行。
    Nowrap, //	空白会被浏览器忽略(其实是所有的空白被合并成一个空格), 超出范围文本也不会换行，文本会在在同一行上继续，直到遇到 <br> 标签为止。
    PreWrap, //	保留所有空白符序列，超出范围会换行。
    Pre, //	保留空白符，超出范围不会换行(利用yoga无法支持， 暂不支持)
    PreLine, //	合并空白符序列，如果存在换行符，优先保留换行符， 超出范围会换行。
}
impl WhiteSpace {
    pub fn allow_wrap(&self) -> bool {
        match *self {
            WhiteSpace::Nowrap |
            WhiteSpace::Pre => false,
            WhiteSpace::Normal |
            WhiteSpace::PreWrap |
            WhiteSpace::PreLine => true,
        }
    }

    pub fn preserve_newlines(&self) -> bool {
        match *self {
            WhiteSpace::Normal |
            WhiteSpace::Nowrap => false,
            WhiteSpace::Pre |
            WhiteSpace::PreWrap |
            WhiteSpace::PreLine => true,
        }
    }

    pub fn preserve_spaces(&self) -> bool {
        match *self {
            WhiteSpace::Normal |
            WhiteSpace::Nowrap |
            WhiteSpace::PreLine => false,
            WhiteSpace::Pre |
            WhiteSpace::PreWrap => true,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Stroke{
    pub width: f32, //	描边宽度
    pub color: CgColor, //	描边颜色
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum FontStyle{
    Normal, //	默认值。标准的字体样式。
    Ttalic, //	斜体的字体样式。
    Oblique, //	倾斜的字体样式。
}
#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum VerticalAlign{
    Top,
    Middle,
    Bottom
}

