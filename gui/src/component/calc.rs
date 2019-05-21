/// 中间计算的组件

use std::{
  sync::Arc,
	default::Default,
};

use atom::Atom;
use map::{vecmap::VecMap};
use ecs::component::Component;

use layout::YgNode;

#[derive(Component, Default, Deref, DerefMut)]
pub struct ZDepth(pub f32);

#[derive(Component, Default, Deref, DerefMut)]
pub struct ByOverflow(pub usize);

#[derive(Debug, Clone, Component, Default, Deref, DerefMut)]
pub struct WorldMatrix(pub super::Matrix4);

//是否可见， 不可见时， 也会占据布局位置
#[derive(Deref, DerefMut, Component, Debug, Default)]
pub struct Visibility(pub bool);

//不透明度
#[derive(Deref, DerefMut, Component, Debug)]
pub struct Opacity(pub f32);

impl Default for Opacity {
	fn default() -> Opacity{
	Opacity(1.0)
	}
}

//是否响应事件
#[derive(Deref, DerefMut, Component, Debug, Default)]
pub struct Enable(pub bool);

#[derive(Component, Debug, Default)]
pub struct RenderObj{
    pub pipeline: usize, //Rc<Pipeline>
    pub depth: f32,
    pub visibility: bool,
    pub is_opacity: bool,
    pub geometry: usize,
    pub ubo: usize, //geometry 对象
}

#[derive(Component, Debug)]
pub struct CharBlock {
  pub family: Atom,
  pub font_size: f32, // 字体高度
  pub line_height: f32,
  pub letter_spacing: f32,
  pub vertical_align: super::user::VerticalAlign,
  pub indent: f32,
  pub chars: Vec<CharNode>, // 字符集合
  pub dirty: bool,
  pub layout_dirty: bool,
}
#[derive(Debug)]
pub struct CharNode {
  pub ch: char, // 字符
  pub width: f32, // 字符宽度
  pub pos: super::Point2, // 位置
  pub node: YgNode, // 对应的yoga节点
}