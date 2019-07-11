/// 中间计算的组件
use std::ops::{Deref, DerefMut, Mul};

use map::{vecmap::VecMap};
use ecs::component::Component;
use ecs::Share as ShareTrait;

use super::user::*;
use layout::FlexNode;

#[derive(Component, Default, Deref, DerefMut)]
pub struct ZDepth(pub f32);

#[derive(Component, Default, Deref, DerefMut)]
pub struct ByOverflow(pub usize);

#[derive(Debug, Clone, Component, Default)]
pub struct WorldMatrix(pub Matrix4, pub bool);

#[derive(Debug, Clone, Component, Default)]
pub struct WorldMatrixRender(pub Matrix4);

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
#[derive(Clone, Debug, Component, Default)]
pub struct HSV {
  pub h: f32, // 0-360 
  pub s: f32, // 0 ~ 正无穷  0表示变灰， 1表示不变， 2表示更饱和
  pub v: f32, // 0 ~ 正无穷 0表示黑色， 1表示不变， 2表示更亮
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
pub enum DirtyType{
  Text = 1, // 1表示文字脏
  LocalStyle = 2, // 2表示局部样式脏
  StyleClass = 4, // 4表示样式类脏
}
#[derive(Component, Debug)]
pub struct CharBlock<L: FlexNode + ShareTrait> {
  pub clazz: TextStyleClazz,
  pub font_size: f32, // 字体高度
  pub line_height: f32,
  pub chars: Vec<CharNode<L>>, // 字符集合
  pub lines: Vec<(usize, f32)>, // 不折行下的每行的单词数量和总宽度。 自动折行不会影响该值
  pub last_line: (usize, f32), // 最后一行的单词数量和总宽度
  pub size: Vector2,
  pub wrap_size: Vector2,
  pub pos: Point2,
  pub line_count: usize, // 行数，
  pub fix_width: bool, // 如果有字宽不等于font_size
  pub dirty: usize, // 1表示文字脏， 2表示局部样式脏， 4表示样式类脏
  pub local_style: bool, // 是否有局部样式修改，// 以后可以改为usize， 记录间距、字体、字号是否修改
  pub style_class: usize, // 使用的那个样式类
}
#[derive(Debug)]
pub struct CharNode<L: FlexNode + ShareTrait> {
  pub ch: char, // 字符
  pub width: f32, // 字符宽度
  pub pos: Point2, // 位置
  pub node: L, // 对应的yoga节点
}

impl Default for Enable {
  fn default() -> Self{
    Self(true)
  }
}

impl Deref for WorldMatrix {
    type Target = Matrix4;
    fn deref(&self) -> &Self::Target{
        &self.0
    }
}

impl DerefMut for WorldMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target{
        &mut self.0
    }
}

impl<'a, 'b> Mul<&'a WorldMatrix> for &'b WorldMatrix{
    type Output = WorldMatrix;
    fn mul(self, other: &'a WorldMatrix) -> Self::Output {
        if self.1 == false && other.1 == false {
            WorldMatrix(
                Matrix4::new(
                    self.x.x * other.x.x,             0.0,                              0.0, 0.0,
                    0.0,                              self.y.y * other.y.y,             0.0, 0.0,
                    0.0,                              0.0,                              1.0, 0.0,
                    self.w.x + other.w.x, self.w.y + other.w.y, 0.0, 1.0,
                ),
                false
            )
        } else {
            WorldMatrix(self.0 * other.0, true)
        }
    }
}

impl<'a> Mul<&'a WorldMatrix> for WorldMatrix{
    type Output = WorldMatrix;
    #[inline]
    fn mul(mut self, other: &'a WorldMatrix) -> Self::Output {
        if self.1 == false && other.1 == false {
            self.x.x = self.x.x * other.x.x;
            self.y.y = self.y.y * other.y.y;
            self.w.x = self.w.x + other.w.x;
            self.w.y = self.w.y + other.w.y;
            self
            // WorldMatrix(
            //     Matrix4::new(
            //         self.x.x * other.x.x,             0.0,                              0.0, 0.0,
            //         0.0,                              self.y.y * other.y.y,             0.0, 0.0,
            //         0.0,                              0.0,                              1.0, 0.0,
            //         self.w.x + other.w.x, self.w.y + other.w.y, 0.0, 1.0,
            //     ),
            //     false
            // )
        } else {
            WorldMatrix(self.0 * other.0, true)
        }
    }
}

impl<'a> Mul<WorldMatrix> for &'a WorldMatrix{
    type Output = WorldMatrix;
    #[inline]
    fn mul(self, mut other: WorldMatrix) -> Self::Output {
        if self.1 == false && other.1 == false {
            other.x.x = self.x.x * other.x.x;
            other.y.y = self.y.y * other.y.y;
            other.w.x = self.w.x + other.w.x;
            other.w.y = self.w.y + other.w.y;
            other
            // WorldMatrix(
            //     Matrix4::new(
            //         self.x.x * other.x.x,             0.0,                              0.0, 0.0,
            //         0.0,                              self.y.y * other.y.y,             0.0, 0.0,
            //         0.0,                              0.0,                              1.0, 0.0,
            //         self.w.x + other.w.x, self.w.y + other.w.y, 0.0, 1.0,
            //     ),
            //     false
            // )
        } else {
            WorldMatrix(self.0 * other.0, true)
        }
    }
}

impl Mul<WorldMatrix> for WorldMatrix{
    type Output = WorldMatrix;
    #[inline]
    fn mul(self, mut other: WorldMatrix) -> Self::Output {
        if self.1 == false && other.1 == false {
            other.x.x = self.x.x * other.x.x;
            other.y.y = self.y.y * other.y.y;
            other.w.x = self.w.x + other.w.x;
            other.w.y = self.w.y + other.w.y;
            other
            // WorldMatrix(
            //     Matrix4::new(
            //         self.x.x * other.x.x,             0.0,                              0.0, 0.0,
            //         0.0,                              self.y.y * other.y.y,             0.0, 0.0,
            //         0.0,                              0.0,                              1.0, 0.0,
            //         self.w.x + other.w.x, self.w.y + other.w.y, 0.0, 1.0,
            //     ),
            //     false
            // )
        } else {
            WorldMatrix(self.0 * other.0, true)
        }
    }
}

impl<'a> Mul<&'a Vector4> for WorldMatrix{
    type Output = Vector4;
    fn mul(self, other: &'a Vector4) -> Vector4 {
        if self.1 == false {
            Vector4::new(other.x * self.x.x + self.w.x, other.y * self.y.y + self.w.y, other.z * self.z.z + self.w.z, other.w)
        } else {
            self * other
        }
    }
}

impl Mul<Vector4> for WorldMatrix{
    type Output = Vector4;
    fn mul(self, other: Vector4) -> Vector4 {
        if self.1 == false {
            Vector4::new(other.x * self.x.x + self.w.x, other.y * self.y.y + self.w.y, other.z * self.z.z + self.w.z, other.w)
        } else {
            self * other
        }
    }
}
