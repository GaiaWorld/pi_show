/// 中间计算的组件

use std::{
  f32,
  default::Default,
};

use map::{vecmap::VecMap};
use ecs::component::Component;

#[derive(Component, Default, Deref, DerefMut)]
pub struct ZDepth(f32);

#[derive(Component, Default, Deref, DerefMut)]
pub struct ByOverflow(usize);

#[derive(Debug, Clone, Component, Default, Deref, DerefMut)]
pub struct WorldMatrix(pub super::Matrix4);

//是否可见， 不可见时， 也会占据布局位置
#[derive(Deref, DerefMut, Component, Debug)]
pub struct Visibility(bool);

impl Default for Visibility {
  fn default() -> Visibility{
    Visibility(true)
  }
}

//不透明度
#[derive(Deref, DerefMut, Component, Debug)]
pub struct Opacity(f32);

impl Default for Opacity {
  fn default() -> Opacity{
    Opacity(1.0)
  }
}

//是否响应事件
#[derive(Deref, DerefMut, Component, Debug)]
pub struct Enable(bool);
impl Default for Enable {
  fn default() -> Enable{
    Enable(true)
  }
}

#[derive(Component, Debug)]
pub struct CharBlock{
    pub stroke_size: f32,
    pub stroke_color: super::CgColor,
    pub font_size: f32,
    pub line_height: f32, //设置行高
    pub font_weight: f32, // 粗细
    pub color: super::CgColor,
    pub chars: Vec<CharPos>,
}

#[derive(Debug)]
pub struct CharPos {
    pub value: char,
    pub pos: super::Point2,
}
