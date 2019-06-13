/// 中间计算的组件

use atom::Atom;
use map::{vecmap::VecMap};
use ecs::component::Component;

use super::user::*;
use layout::YgNode;

#[derive(Component, Default, Deref, DerefMut)]
pub struct ZDepth(pub f32);

#[derive(Component, Default, Deref, DerefMut)]
pub struct ByOverflow(pub usize);

#[derive(Debug, Clone, Component, Default, Deref, DerefMut)]
pub struct WorldMatrix(pub Matrix4);

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
#[derive(Deref, DerefMut, Component, Debug)]
pub struct Enable(pub bool);

// HSV
#[derive(Clone, Debug, Component)]
pub struct HSV {
  pub h: f32, // 0-360
  pub s: f32, // 0-1
  pub v: f32, // 0-Infinity
}

impl Default for HSV {
    fn default() -> Self {
        Self {
            h: 0.0,
            s: 0.0,
            v: 1.0,
        }
    }
}

// #[derive(Component, Debug, Default)]
// pub struct RenderObj{
//     pub pipeline: usize, //Rc<Pipeline>
//     pub depth: f32,
//     pub visibility: bool,
//     pub is_opacity: bool,
//     pub geometry: usize,
//     pub ubo: usize, //geometry 对象
// }

#[derive(Component, Debug)]
pub struct CharBlock {
  pub family: Atom,
  pub font_size: f32, // 字体高度
  pub line_height: f32,
  pub letter_spacing: f32,
  pub vertical_align: VerticalAlign,
  pub indent: f32,
  pub chars: Vec<CharNode>, // 字符集合
  pub dirty: bool,
}
#[derive(Debug)]
pub struct CharNode {
  pub ch: char, // 字符
  pub width: f32, // 字符宽度
  pub pos: Point2, // 位置
  pub node: YgNode, // 对应的yoga节点
}

impl Default for Enable {
  fn default() -> Self{
    Self(true)
  }
}