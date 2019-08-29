/// 中间计算的组件

use std::ops::{Deref, DerefMut, Mul};

use map::{vecmap::VecMap};
use ecs::component::Component;

use share::{ Share };
use hal_core::*;

use super::user::*;
use layout::FlexNode;
use render::res_mgr::*;

#[derive(Component, Default, Deref, DerefMut)]
pub struct ZDepth(pub f32);

#[derive(Component, Default, Deref, DerefMut)]
pub struct ByOverflow(pub usize);

#[derive(Debug, Clone, Component, Default)]
pub struct WorldMatrix(pub Matrix4, pub bool);

//是否可见， 不可见时， 也会占据布局位置
#[derive(Deref, DerefMut, Component, Debug, Default)]
pub struct Visibility(pub bool);

// 是否在裁剪平面的可视范围内
#[derive(Component, Debug)]
pub struct Culling(pub bool);

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

#[derive(Debug)]
// 样式类型
pub enum StyleType{
    Text = 1,
    FontStyle = 2, 
    FontWeight = 4, 
    FontSize = 0x8, 
    FontFamily = 0x10, 
    LetterSpacing = 0x20, 
    WordSpacing = 0x40, 
    LineHeight = 0x80, 
    Indent = 0x100, 
    WhiteSpace = 0x200, 
    TextAlign = 0x400, 
    VerticalAlign = 0x800, 
    Color = 0x1000, 
    Stroke = 0x2000, 
    TextShadow = 0x4000, 

    Image = 0x8000,
    ImageClip = 0x10000,
    ObjectFit = 0x20000,

    BorderImage = 0x40000,
    BorderImageClip = 0x80000,
    BorderImageSlice = 0x100000,
    BorderImageRepeat = 0x200000,

    BorderColor = 0x400000,

    BackgroundColor = 0x800000,

    BoxShadow = 0x1000000,

    Matrix = 0x2000000,
    Opacity = 0x4000000,
    Layout = 0x8000000,
    BorderRadius = 0x10000000,
    ByOverflow = 0x20000000,
    Filter = 0x40000000,
}

#[derive(Debug)]
// 样式类型
pub enum StyleType1{
    Width = 1,
    Height = 2, 
    Margin = 4,  
    Padding = 8, 
    Border = 0x10, 
    Position = 0x20,
    MinWidth = 0x40, 
    MinHeight = 0x80,
    MaxHeight = 0x100,
    MaxWidth = 0x200,
    FlexBasis = 0x400,
    FlexShrink = 0x800,
    FlexGrow = 0x1000,
    PositionType = 0x2000,
    FlexWrap = 0x4000,
    FlexDirection = 0x8000,
    AlignContent = 0x10000,
    AlignItems = 0x20000,
    AlignSelf = 0x40000,
    JustifyContent = 0x80000,

    Display = 0x100000,
    Visibility = 0x200000,
    Enable = 0x400000,
    ZIndex = 0x800000,
    Transform = 0x1000000,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct StyleMark{
    pub dirty: usize,
    pub dirty1: usize,
    pub local_style: usize,
    pub class_style: usize,
    pub local_style1: usize,
}

#[derive(Component, Debug)]
pub struct CharBlock<L: FlexNode + 'static> {
  pub font_size: f32, // 字体高度
  pub font_height: f32, // 字体高度
  pub stroke_width: f32, //描边宽度
  pub line_height: f32,
  pub chars: Vec<CharNode<L>>, // 字符集合
  pub lines: Vec<(usize, usize, f32)>, // 不折行下的每行的起始字符位置、单词数量和总宽度。 自动折行不会影响该值
  pub last_line: (usize, usize, f32), // 最后一行的起始字符位置、单词数量和总宽度
  pub size: Vector2,
  pub wrap_size: Vector2,
  pub pos: Point2,
  pub line_count: usize, // 行数，
  pub fix_width: bool, // 如果有字宽不等于font_size
  pub style_class: usize, // 使用的那个样式类
  pub is_pixel: bool,
}

#[derive(Debug)]
pub struct CharNode<L: FlexNode + 'static> {
  pub ch: char, // 字符
  pub width: f32, // 字符宽度
  pub pos: Point2, // 位置
  pub ch_id_or_count: usize, // 字符id或单词的字符数量
  pub base_width: f32, // font_size 为32 的字符宽度
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
                    self.w.x + (other.w.x * self.x.x), self.w.y + (other.w.y * self.y.y), 0.0, 1.0,
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
            self.w.x = self.w.x + (other.w.x * self.x.x);
            self.w.y = self.w.y + (other.w.y * self.y.y);
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
            other.w.x = self.w.x + (other.w.x * self.x.x);
            other.w.y = self.w.y + (other.w.y * self.y.y);
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
            other.w.x = self.w.x + (other.w.x * self.x.x);
            other.w.y = self.w.y + (other.w.y * self.y.y);
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
            self.0 * other
        }
    }
}

impl<'a> Mul<Vector4> for &'a WorldMatrix{
    type Output = Vector4;
    fn mul(self, mut other: Vector4) -> Vector4 {
        if self.1 == false {
            other.x = other.x * self.x.x + self.w.x;
            other.y = other.y * self.y.y + self.w.y;
            other.z = other.z * self.z.z + self.w.z;
            other
        } else {
            self.0 * other
        }
    }
}

// 渲染--------------------------------------------------------------------------------------------------------------------------
uniform_buffer! {
    #[derive(Hash)]
    struct ClipTextureSize {
        clipTextureSize: UniformValue,
    }
}

uniform_buffer! {
    #[derive(Hash)]
    struct ClipBox {
        clipBox: UniformValue,
    }
}

uniform_buffer! {
    #[derive(Hash)]
    struct TextTextureSize {
        textureSize: UniformValue,
    }
}

uniform_buffer! {
    #[derive(Hash)]
    struct WorldMatrixUbo {
        worldMatrix: UniformValue,
    }
}

uniform_buffer! {
    #[derive(Hash)]
    struct ViewMatrixUbo {
        viewMatrix: UniformValue,
    }
}

uniform_buffer! {
    #[derive(Hash)]
    struct ProjectMatrixUbo {
        projectMatrix: UniformValue,
    }
}

uniform_buffer! {
    #[derive(Hash)]
    struct UColorUbo {
        uColor: UniformValue,
    }
}
impl Res for UColorUbo {
    type Key = u64;
}

uniform_buffer! {
    #[derive(Hash)]
    struct HsvUbo {
        hsvValue: UniformValue,
    }
}
impl Res for HsvUbo {
    type Key = u64;
}

defines! {
    #[derive(Clone)]
    struct VsDefines {
        VERTEX_COLOR: String,
        CLIP_BOX: String,
        BOX_SHADOW_BLUR: String,
    }
}

defines! {
    #[derive(Clone)]
    struct FsDefines {
        UCOLOR: String,
        VERTEX_COLOR: String,
        BOX_SHADOW_BLUR: String,
        CLIP: String,
        CLIP_BOX: String,
        HSV: String,
        GRAY: String,
    }
}

defines! {
    #[derive(Clone)]
    struct FsBaseDefines {
        CLIP: String,
        CLIP_BOX: String,
        HSV: String,
        GRAY: String,
    }
}

uniform_buffer! {
    #[derive(Hash)]
    struct MsdfStrokeUbo {
        strokeSize: UniformValue,
        strokeColor: UniformValue,
    }
}
impl Res for MsdfStrokeUbo {
    type Key = u64;
}

program_paramter! {
    #[derive(Clone)]
    struct MsdfParamter {
        uColor: UColorUbo,
        stroke: MsdfStrokeUbo,
        textureSize: TextTextureSize,
        worldMatrix: WorldMatrixUbo,
        viewMatrix: ViewMatrixUbo,
        projectMatrix: ProjectMatrixUbo,
        hsvValue: HsvUbo,
        clipIndices: UniformValue,
        clipTexture: (HalTexture, HalSampler),
        clipTextureSize: ClipTextureSize,
        clipBox: ClipBox,
        texture: (HalTexture, HalSampler),
        alpha: UniformValue,
    }
}

defines! {
    #[derive(Clone)]
    struct MsdfFsDefines {
        STROKE: String,
        UCOLOR: String,
        VERTEX_COLOR: String,
        CLIP: String,
        CLIP_BOX: String,
        HSV: String,
        GRAY: String,
    }
}

uniform_buffer! {
    #[derive(Hash)]
    struct CanvasTextStrokeColorUbo {
        strokeColor: UniformValue,
    }
}
impl Res for CanvasTextStrokeColorUbo {
    type Key = u64;
}

program_paramter! {
    #[derive(Clone)]
    struct CanvasTextParamter {
        uColor: UColorUbo,
        strokeColor: CanvasTextStrokeColorUbo,
        textureSize: TextTextureSize,
        worldMatrix: WorldMatrixUbo,
        viewMatrix: ViewMatrixUbo,
        hsvValue: HsvUbo,
        projectMatrix: ProjectMatrixUbo,
        clipIndices: UniformValue,
        clipTexture: (HalTexture, HalSampler),
        clipTextureSize: ClipTextureSize,
        clipBox: ClipBox,
        texture: (HalTexture, HalSampler),
        alpha: UniformValue,
    }
}


program_paramter! {
    #[derive(Clone)]
    struct ColorParamter {
        uColor: UColorUbo,
        worldMatrix: WorldMatrixUbo,
        viewMatrix: ViewMatrixUbo,
        projectMatrix: ProjectMatrixUbo,
        hsvValue: HsvUbo,
        clipIndices: UniformValue,
        clipTexture: (HalTexture, HalSampler),
        clipTextureSize: ClipTextureSize,
        clipBox: ClipBox,
        alpha: UniformValue,
        uRect: UniformValue,
        blur: UniformValue,
    }
}

program_paramter! {
    #[derive(Clone)]
    struct ImageParamter {
        worldMatrix: WorldMatrixUbo,
        viewMatrix: ViewMatrixUbo,
        projectMatrix: ProjectMatrixUbo,
        hsvValue: HsvUbo,
        clipIndices: UniformValue,
        clipTexture: (HalTexture, HalSampler),
        clipTextureSize: ClipTextureSize,
        clipBox: ClipBox,
        texture: (HalTexture, HalSampler),
        alpha: UniformValue,
    }
}

program_paramter! {
    #[derive(Clone)]
    struct ClipParamter {
        meshNum: UniformValue,
        viewMatrix: ViewMatrixUbo,
        projectMatrix: ProjectMatrixUbo,
    }
}

defines! {
    #[derive(Clone)]
    struct ImageFsDefines {
        CLIP: String,
        CLIP_BOX: String,
        HSV: String,
        GRAY: String,
    }
}