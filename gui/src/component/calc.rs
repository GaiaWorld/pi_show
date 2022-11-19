/// 中间计算的组件
use std::ops::{Deref, DerefMut, Mul};

use densevec::DenseVecMap;
use ecs::component::Component;
use map::vecmap::VecMap;
use nalgebra::Matrix4;
use share::Share;

use hal_core::*;

use super::user::*;
// use layout::FlexNode;
use flex_layout::*;
use res::Res;

use crate::render::res::TextureRes;
use crate::{
    render::res::{BufferRes, TexturePartRes},
    single::{DirtyViewRect, PostProcessContext, PostProcessObj, ProjectionMatrix, RenderObj},
    util::vecmap_default::VecMapWithDefault,
};

// // 布局计算结果
// #[derive(Clone, Debug, Default, Component, PartialEq)]
// pub struct Layout {
//     pub left: f32,
//     pub top: f32,
//     pub width: f32,
//     pub height: f32,
//     pub border_left: f32,
//     pub border_top: f32,
//     pub border_right: f32,
//     pub border_bottom: f32,
//     pub padding_left: f32,
//     pub padding_top: f32,
//     pub padding_right: f32,
//     pub padding_bottom: f32,
// }

/// 渲染上下文，一些具有特殊属性的节点，可以是一个新的渲染上下文，另外根节点也是一个渲染上下文，
#[derive(Component)]
pub struct RenderContext {
    pub index_count: usize, // 索引数量，表示由几种属性创建的context， 如节点上值存在MaskImage，则count=1， 如果同时存在MaskImage和opcaity， 则count = 2， 当indexCount数量为0时，应该删除RenderContext

    pub view_matrix: Option<WorldMatrix>,
    pub projection_matrix: Option<ProjectionMatrix>,
    pub view_matrix_ubo: Option<Share<dyn UniformBuffer>>,
    pub projection_matrix_ubo: Option<Share<dyn UniformBuffer>>,
    pub dirty_view_rect: DirtyViewRect,
    pub render_rect: Aabb2,           // 大小、尺寸
    pub content_box: Aabb2,           // 内容的最大包围盒
    pub render_target: Option<usize>, // 一个在dyn_texture中的索引，如果为None，将渲染在gui的默认渲染目标上
    pub geo_change: bool,             // 与上一帧相比，在dyn_texture中分配的位置是否改变
    // pub post_process: Vec<usize>, // 后处理渲染
    pub opacity_list: Vec<usize>,
    pub transparent_list: Vec<usize>,
    pub render_count: usize,
    pub render_obj_index: usize,
    pub post_process: Option<Box<PostProcessContext>>,
}

/// 内容最大包围盒范围(不包含自身)
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
#[storage(VecMapWithDefault)]
pub struct ContentBox(pub Aabb2);

impl Default for ContentBox {
    fn default() -> Self { Self(Aabb2::new(Point2::new(f32::MAX, f32::MAX), Point2::new(f32::MIN, f32::MIN))) }
}

/// 所属上下文索引
#[derive(Component, Clone, Debug)]
pub struct ContextIndex(pub usize);

impl RenderContext {
    pub fn new(
        index_count: usize,
        render_rect: Aabb2,
        content_box: Aabb2,
        view_matrix: Option<WorldMatrix>,
        projection_matrix: Option<ProjectionMatrix>,
        view_matrix_ubo: Option<Share<dyn UniformBuffer>>,
        projection_matrix_ubo: Option<Share<dyn UniformBuffer>>,
        dirty_view_rect: DirtyViewRect,
        render_target: Option<usize>,
        render_obj_index: usize,
    ) -> Self {
        Self {
            index_count,
            render_rect,
            content_box,
            view_matrix,
            projection_matrix,
            view_matrix_ubo,
            projection_matrix_ubo,
            dirty_view_rect,
            render_target,
            render_obj_index,
            geo_change: false,
            opacity_list: Vec::new(),
            transparent_list: Vec::new(),
            render_count: 0,
            post_process: None,
        }
    }
}
impl PostProcessObj for RenderContext {
    fn get_post(&self) -> &Option<Box<PostProcessContext>> { &self.post_process }
    fn get_post_mut(&mut self) -> &mut Option<Box<PostProcessContext>> { &mut self.post_process }
    fn set_post(&mut self, v: Option<Box<PostProcessContext>>) { self.post_process = v; }
}

// #[derive(Debug, Default, Deserialize, Serialize,Clone)]
// pub struct DirtyViewRect(pub f32, pub f32, pub f32, pub f32, pub bool/*是否与最大视口相等（RenderBeginDesc中的视口）*/);

#[derive(Clone, Debug, Component, PartialEq, Deserialize, Serialize)]
pub struct LayoutR {
    pub rect: Rect<f32>,
    pub border: Rect<f32>,
    pub padding: Rect<f32>,
}

impl Default for LayoutR {
    fn default() -> LayoutR {
        LayoutR {
            rect: Rect {
                start: 0.0,
                end: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
            border: Rect {
                start: 0.0,
                end: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
            padding: Rect {
                start: 0.0,
                end: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
        }
    }
}

// ZIndex计算结果， 按照节点的ZIndex分配的一个全局唯一的深度表示
#[derive(Component, Default, Deref, DerefMut, Clone, Debug)]
#[storage(VecMapWithDefault)]
pub struct ZDepth(pub f32);

// gui支持最多32个裁剪面， 该值按位表示节点被哪些裁剪面裁剪， 等于0时， 表示不被任何裁剪面裁剪， 等于1时， 被第一个裁剪面裁剪， 等于2时，表示被第二个裁剪面裁剪， 等于3表示被第一个和第二个裁剪面共同裁剪。。。。。
#[derive(Component, Clone, Default, Deref, DerefMut, Debug)]
#[storage(VecMapWithDefault)]
pub struct ByOverflow(pub usize);

// 世界矩阵，  WorldMatrix(矩阵, 矩阵描述的变换是存在旋转变换)， 如果不存在旋转变换， 可以简化矩阵的乘法
#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
#[storage(VecMapWithDefault)]
pub struct WorldMatrix(pub Matrix4<f32>, pub bool);

impl WorldMatrix {
	#[inline]
	pub fn is_rotate(&self) -> bool {
		self.1
	}
}

//是否可见,
#[derive(Deref, DerefMut, Component, Clone, Debug, Default)]
#[storage(VecMapWithDefault)]
pub struct Visibility(pub bool);

// 是否被裁剪
#[derive(Component, Clone, Debug, Default)]
#[storage(VecMapWithDefault)]
pub struct Culling(pub bool);

//不透明度
#[derive(Deref, DerefMut, Component, Clone, Debug)]
#[storage(VecMapWithDefault)]
pub struct Opacity(pub f32);

//是否响应事件
#[derive(Deref, DerefMut, Component, Clone, Debug)]
#[storage(VecMapWithDefault)]
pub struct Enable(pub bool);

// HSV
#[derive(Clone, Debug, Component, Default)]
#[storage(VecMapWithDefault)]
pub struct HSV {
    pub h: f32, // 0-360
    pub s: f32, // 0 ~ 正无穷  0表示变灰， 1表示不变， 2表示更饱和
    pub v: f32, // 0 ~ 正无穷 0表示黑色， 1表示不变， 2表示更亮
}

// 枚举样式的类型
#[derive(Debug)]
pub enum StyleType {
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
    Oct = std::isize::MIN,
}

// 枚举样式的类型
#[derive(Debug)]
pub enum StyleType1 {
    // Width = 1,
    // Height = 2,
    // Margin = 4,
    // Padding = 8,
    // Border = 0x10,
    // Position = 0x20,
    // MinWidth = 0x40,
    // MinHeight = 0x80,
    // MaxHeight = 0x100,
    // MaxWidth = 0x200,
    // FlexBasis = 0x400,
    // FlexShrink = 0x800,
    // FlexGrow = 0x1000,
    // PositionType = 0x2000,
    // FlexWrap = 0x4000,
    // FlexDirection = 0x8000,
    // AlignContent = 0x10000,
    // AlignItems = 0x20000,
    BackgroundRepeat = 0x400,
    Blur = 0x800,
    BorderImageTexture = 0x1000,
    ImageTexture = 0x2000,
    TransformOrigin = 0x4000,
    ContentBox = 0x8000,
    Direction = 0x10000,
    AspectRatio = 0x20000,
    Order = 0x40000,
    FlexBasis = 0x80000,

    Display = 0x100000,
    Visibility = 0x200000,
    Enable = 0x400000,
    ZIndex = 0x800000,
    Transform = 0x1000000,
    TransformWillChange = 0x2000000,
    Overflow = 0x4000000,

    Create = 0x8000000,
    Delete = 0x10000000,

    MaskImage = 0x20000000,
    MaskImageClip = 0x40000000,
    MaskTexture = std::isize::MIN,
}

// 布局属性标记
pub enum StyleType2 {
    Width = 1,
    Height = 2,

    MarginTop = 4,
    MarginRight = 8,
    MarginBottom = 0x10,
    MarginLeft = 0x20,

    PaddingTop = 0x40,
    PaddingRight = 0x80,
    PaddingBottom = 0x100,
    PaddingLeft = 0x200,

    BorderTop = 0x400,
    BorderRight = 0x800,
    BorderBottom = 0x1000,
    BorderLeft = 0x2000,

    PositionTop = 0x4000,
    PositionRight = 0x8000,
    PositionBottom = 0x10000,
    PositionLeft = 0x20000,

    MinWidth = 0x40000,
    MinHeight = 0x80000,
    MaxHeight = 0x100000,
    MaxWidth = 0x200000,
    JustifyContent = 0x400000,
    FlexShrink = 0x800000,
    FlexGrow = 0x1000000,
    PositionType = 0x2000000,
    FlexWrap = 0x4000000,
    FlexDirection = 0x8000000,
    AlignContent = 0x10000000,
    AlignItems = 0x20000000,
    AlignSelf = 0x40000000,
    BlendMode = std::isize::MIN,
}

// margin标记
pub const LAYOUT_MARGIN_MARK: usize =
    StyleType2::MarginTop as usize | StyleType2::MarginRight as usize | StyleType2::MarginBottom as usize | StyleType2::MarginLeft as usize;
// pading标记
pub const LAYOUT_PADDING_MARK: usize =
    StyleType2::PaddingTop as usize | StyleType2::PaddingRight as usize | StyleType2::PaddingBottom as usize | StyleType2::PaddingLeft as usize;
// border标记
pub const LAYOUT_BORDER_MARK: usize =
    StyleType2::BorderTop as usize | StyleType2::BorderRight as usize | StyleType2::BorderBottom as usize | StyleType2::BorderLeft as usize;
// border标记
pub const LAYOUT_POSITION_MARK: usize =
    StyleType2::PositionTop as usize | StyleType2::PositionRight as usize | StyleType2::PositionBottom as usize | StyleType2::PositionLeft as usize;
// 矩形属性标记
pub const LAYOUT_RECT_MARK: usize = StyleType2::Width as usize | StyleType2::Height as usize | LAYOUT_MARGIN_MARK;


#[derive(Component, Clone)]
pub enum MaskTexture {
    All(Share<TextureRes>),
    Part(Share<TexturePartRes>),
}

// 图片使用纹理（可以是canvas，如果是canvas，不要再同时渲染一张图片，否则会冲突，他们的纹理是共用该组件）
#[derive(Component, Clone)]
pub enum ImageTexture {
    All(Share<TextureRes>),
    Part(Share<TexturePartRes>),
}

// 边框图片使用纹理
#[derive(Component, Clone)]
pub struct BorderImageTexture(pub Share<TextureRes>);

// impl Deref for MaskTexture {
// 	type Target = Share<TextureRes>;
// 	fn deref(&self) -> &Self::Target {
// 		match self {
// 			MaskTexture::All(r) => &r.bind,
// 			MaskTexture::Part(r) => *&r,
// 		}
// 	}
// }

// 样式标记
#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct StyleMark {
    pub dirty: usize,  // 脏， StyleType值的组合， 如：StyleType::TextShadow as usize | StyleType::Image as usize 表示TextShadow和Image脏了
    pub dirty1: usize, // 脏， StyleType1值的组合， 如：StyleType1::Width as usize | StyleType2::Height as usize 表示Width和Height脏了
    pub dirty2: usize, // 脏， StyleType1值的组合， 如：StyleType1::Width as usize | StyleType2::Height as usize 表示Width和Height脏了
    pub dirty_other: usize, // 其它脏， 仅标记，不会记入脏列表
    pub local_style: usize, // 本地样式， 表示节点样式中，哪些样式是由style设置的（而非class设置）
    pub local_style1: usize, // 本地样式， 表示节点样式中，哪些样式是由style设置的（而非class设置）
    pub local_style2: usize, // 本地样式， 表示节点样式中，哪些样式是由style设置的（而非class设置）
    pub class_style: usize, // class样式， 表示节点样式中，哪些样式是由class设置的
    pub class_style1: usize, // class样式， 表示节点样式中，哪些样式是由class设置的
    pub class_style2: usize, // class样式， 表示节点样式中，哪些样式是由class设置的
}
pub enum LayoutDirtyType {
    Rect = 1,        // 矩形区间发生改变时，设置脏
    SelfStyle = 2,   // 矩形区间发生改变时，设置脏
    NormalStyle = 4, // 矩形区间发生改变时，设置脏
}

#[derive(Component, Clone, Default, Deref, DerefMut, Debug, Serialize, Deserialize)]
pub struct NodeState(pub INode);


// // 字符块， 一个文字节点的计算结果， 包含节点中每字符的位置信息等
// #[derive(Component, Debug)]
// pub struct CharBlock<L: FlexNode + 'static> {
//     pub old_position: (f32, f32),
//     pub font_size: f32,    // 字体高度
//     pub font_height: f32,  // 字体高度
//     pub stroke_width: f32, //描边宽度
//     pub line_height: f32,
//     pub chars: Vec<CharNode<L>>,         // 字符集合
//     pub lines: Vec<(usize, usize, f32)>, // 不折行下的每行的起始字符位置、单词数量和总宽度。 自动折行不会影响该值
//     pub last_line: (usize, usize, f32),  // 最后一行的起始字符位置、单词数量和总宽度
//     pub size: Vector2,
//     pub wrap_size: Vector2,
//     pub pos: Point2,
//     pub line_count: usize,  // 行数，
//     pub fix_width: bool,    // 如果有字宽不等于font_size
//     pub style_class: usize, // 使用的那个样式类
//     pub is_pixel: bool,
// }

// 字符节点， 对应一个字符的
#[derive(Component, Debug, Clone, Default)]
pub struct CharNode {
    pub ch: char, // 字符
    // pub pos: Point2,           // 位置
    pub ch_id_or_count: usize, // 字符id或单词的字符数量
    pub base_width: f32,       // font_size 为32 的字符宽度
    pub width: f32,
    // pub node: L,               // 对应的yoga节点
}

// // span节点对应的字符布局
// #[derive(Component, Debug, Clone, Default)]
// pub struct CharBlock{
// 	pub chars: Vec<CharNode>,
// }

// TransformWillChange的矩阵计算结果， 用于优化Transform的频繁改变
#[derive(Component, Debug, Clone, Default)]
#[storage(DenseVecMap)]
pub struct TransformWillChangeMatrix(pub WorldMatrix);

impl Default for Enable {
    fn default() -> Self { Self(true) }
}

impl Deref for WorldMatrix {
    type Target = Matrix4<f32>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for WorldMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<'a, 'b> Mul<&'a WorldMatrix> for &'b WorldMatrix {
    type Output = WorldMatrix;
    fn mul(self, other: &'a WorldMatrix) -> Self::Output {
        if self.1 == false && other.1 == false {
            WorldMatrix(self.0 * other.0, false)
        //     WorldMatrix(
        //         Matrix4::new(
        //             self.x.x * other.x.x,
        //             0.0,
        //             0.0,
        //             0.0,
        //             0.0,
        //             self.y.y * other.y.y,
        //             0.0,
        //             0.0,
        //             0.0,
        //             0.0,
        //             1.0,
        //             0.0,
        //             self.w.x + (other.w.x * self.x.x),
        //             self.w.y + (other.w.y * self.y.y),
        //             0.0,
        //             1.0,
        //         ),
        //         false,
        //     )
        } else {
            WorldMatrix(self.0 * other.0, true)
        }
    }
}

impl<'a> Mul<&'a WorldMatrix> for WorldMatrix {
    type Output = WorldMatrix;
    #[inline]
    fn mul(mut self, other: &'a WorldMatrix) -> Self::Output {
        if self.1 == false && other.1 == false {
            WorldMatrix(self.0 * other.0, false)
        //     self.x.x = self.x.x * other.x.x;
        //     self.y.y = self.y.y * other.y.y;
        //     self.w.x = self.w.x + (other.w.x * self.x.x);
        //     self.w.y = self.w.y + (other.w.y * self.y.y);
        //     self
        } else {
            WorldMatrix(self.0 * other.0, true)
        }
    }
}

impl<'a> Mul<WorldMatrix> for &'a WorldMatrix {
    type Output = WorldMatrix;
    #[inline]
    fn mul(self, mut other: WorldMatrix) -> Self::Output {
        if self.1 == false && other.1 == false {
            WorldMatrix(self.0 * other.0, false)
        //     other.x.x = self.x.x * other.x.x;
        //     other.y.y = self.y.y * other.y.y;
        //     other.w.x = self.w.x + (other.w.x * self.x.x);
        //     other.w.y = self.w.y + (other.w.y * self.y.y);
        //     other
        } else {
            WorldMatrix(self.0 * other.0, true)
        }
    }
}

impl Mul<WorldMatrix> for WorldMatrix {
    type Output = WorldMatrix;
    #[inline]
    fn mul(self, mut other: WorldMatrix) -> Self::Output {
        if self.1 == false && other.1 == false {
            WorldMatrix(self.0 * other.0, false)
        //     other.x.x = self.x.x * other.x.x;
        //     other.y.y = self.y.y * other.y.y;
        //     other.w.x = self.w.x + (other.w.x * self.x.x);
        //     other.w.y = self.w.y + (other.w.y * self.y.y);
        //     other
        } else {
            WorldMatrix(self.0 * other.0, true)
        }
    }
}

impl<'a> Mul<&'a Vector4> for WorldMatrix {
    type Output = Vector4;
    fn mul(self, other: &'a Vector4) -> Vector4 {
        // if self.1 == false {
        //     Vector4::new(
        //         other.x * self.x.x + self.w.x,
        //         other.y * self.y.y + self.w.y,
        //         other.z * self.z.z + self.w.z,
        //         other.w,
        //     )
        // } else {
        self.0 * other
        // }
    }
}

impl<'a> Mul<Vector4> for &'a WorldMatrix {
    type Output = Vector4;
    fn mul(self, mut other: Vector4) -> Vector4 {
        // if self.1 == false {
        //     other.x = other.x * self.x.x + self.w.x;
        //     other.y = other.y * self.y.y + self.w.y;
        //     other.z = other.z * self.z.z + self.w.z;
        //     other
        // } else {
        self.0 * other
        // }
    }
}

impl WorldMatrix {
    pub fn invert(&self) -> Option<Self> {
        // None   // TODO!!!!!!!!!!!!
        // if !self.1 {
        //     Some(Self(
        //         Matrix4::new(
        //             1.0 / self.x.x,
        //             0.0,
        //             0.0,
        //             0.0,
        //             0.0,
        //             1.0 / self.y.y,
        //             0.0,
        //             0.0,
        //             0.0,
        //             0.0,
        //             1.0 / self.z.z,
        //             0.0,
        //             -self.w.x / self.x.x,
        //             -self.w.y / self.y.y,
        //             -self.w.z / self.z.z,
        //             1.0,
        //         ),
        //         false,
        //     ))
        // } else {
        // Matrix4::new(
        // 	1.0, 0.0, 0.0,0.0,
        // 	0.0, 1.0, 0.0,0.0,
        // 	0.0, 0.0, 1.0,0.0,
        // 	0.0, 0.0, 0.0,1.0,
        // )/self.0
        // match self.0.invert() {
        //     Some(r) => Some(Self(r, true)),
        //     None => None,
        // }
        // }
        match self.0.try_inverse() {
            Some(r) => Some(Self(r, self.1)),
            None => None,
        }
    }
}

impl Default for Opacity {
    fn default() -> Opacity { Opacity(1.0) }
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
        MASK_IMAGE: String,
        HORIZONTAL: String,
        VERTICAL: String,
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
        STROKE: String,
        MASK_IMAGE: String,
        OPACITY: String,
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
        // viewMatrix: ViewMatrixUbo,
        // projectMatrix: ProjectMatrixUbo,
        hsvValue: HsvUbo,
        clipIndices1: UniformValue,
        clipIndices2: UniformValue,
        clipTexture: (HalTexture, HalSampler),
        clipTextureSize: ClipTextureSize,
        clipBox: ClipBox,
        texture: (HalTexture, HalSampler),
        alpha: UniformValue,
        pixelRatio: UniformValue,
        weight: UniformValue,
        fontSize: UniformValue,
        depth: UniformValue,
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
        clipIndices1: UniformValue,
        clipIndices2: UniformValue,
        clipTexture: (HalTexture, HalSampler),
        clipTextureSize: ClipTextureSize,
        clipBox: ClipBox,
        texture: (HalTexture, HalSampler),
        alpha: UniformValue,
        depth: UniformValue,
    }
}

program_paramter! {
    #[derive(Clone)]
    struct GaussBlurParamter {
        // worldMatrix: WorldMatrixUbo,
        // viewMatrix: ViewMatrixUbo,
        // projectMatrix: ProjectMatrixUbo,
        texture: (HalTexture, HalSampler),
        blurRadius: UniformValue, // 模糊半径
        textureSize: UniformValue, // 纹理尺寸
        uvRegion: UniformValue, // 模糊区域（左下、右上坐标，以像素为单位）
    }
}

program_paramter! {
    #[derive(Clone)]
    struct BlurParamter {
        // worldMatrix: WorldMatrixUbo,
        // viewMatrix: ViewMatrixUbo,
        // projectMatrix: ProjectMatrixUbo,
        texture: (HalTexture, HalSampler),
        offset: UniformValue,
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
        clipIndices1: UniformValue,
        clipIndices2: UniformValue,
        clipTexture: (HalTexture, HalSampler),
        clipTextureSize: ClipTextureSize,
        clipBox: ClipBox,
        alpha: UniformValue,
        uRect: UniformValue,
        blur: UniformValue,
        depth: UniformValue,
        texture: (HalTexture, HalSampler),
    }
}

program_paramter! {
    #[derive(Clone)]
    struct ImageParamter {
        worldMatrix: WorldMatrixUbo,
        viewMatrix: ViewMatrixUbo,
        projectMatrix: ProjectMatrixUbo,
        hsvValue: HsvUbo,
        clipIndices1: UniformValue,
        clipIndices2: UniformValue,
        clipTexture: (HalTexture, HalSampler),
        clipTextureSize: ClipTextureSize,
        clipBox: ClipBox,
        texture: (HalTexture, HalSampler),
        alpha: UniformValue,
        depth: UniformValue,
    }
}

program_paramter! {
    #[derive(Clone)]
    struct FboParamter {
        worldMatrix: WorldMatrixUbo,
        viewMatrix: ViewMatrixUbo,
        projectMatrix: ProjectMatrixUbo,
        clipIndices1: UniformValue,
        clipIndices2: UniformValue,
        clipTexture: (HalTexture, HalSampler),
        clipTextureSize: ClipTextureSize,
        clipBox: ClipBox,
        texture: (HalTexture, HalSampler),
        alpha: UniformValue,
        maskTexture: (HalTexture, HalSampler),
        maskRect: UniformValue,
        maskUv: UniformValue,
        depth: UniformValue,
        context_id: UniformValue,
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
