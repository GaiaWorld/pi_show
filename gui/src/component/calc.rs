/// 中间计算的组件


use std::{
  f32,
  default::Default,
};

use map::{vecmap::VecMap};
use cg::Matrix4;
use ecs::component::Component;

#[derive(Component, Default)]
pub struct ZDepth(pub f32);

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

#[derive(Debug, Clone, Component, Default, Deref, DerefMut)]
pub struct WorldMatrix(pub Matrix4<f32>);