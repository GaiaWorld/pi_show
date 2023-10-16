//！ 定义用户设置的组件

use std::mem::{transmute, forget};
use std::ptr::read_unaligned;
use std::{collections::VecDeque, fmt::Debug};


use bitvec::prelude::BitArray;
use flex_layout::Size;
use ordered_float::NotNan;
use pi_atom::Atom;
use ecs::component::Component;
// use bevy_ecs::prelude::{Changed, Component, DetectChangesMut, Entity};
// use pi_bevy_ecs_extend::system_param::layer_dirty::ComponentEvent;
pub use pi_flex_layout::prelude::{Dimension, Number, Rect, Size as FlexSize};
use pi_flex_layout::style::{AlignContent, AlignItems, AlignSelf, Direction, Display, FlexDirection, FlexWrap, JustifyContent, PositionType};
// use pi_slotmap::Key;
pub use pi_style::style::{
    Aabb2, AnimationDirection, AnimationFillMode, AnimationName, AnimationPlayState, AnimationTimingFunction, CgColor, Color, ColorAndPosition,
    Enable, FitType, FontSize, FontStyle, ImageRepeat, IterationCount, LengthUnit, LineHeight, LinearGradientColor, NotNanRect, ShowType, Stroke,
    StyleType, TextAlign, TextShadow as TextShadow1, Time, TransformFunc, TransformFuncs, TransformOrigin, VerticalAlign, WhiteSpace,
};
// pub use pi_style::style::{ImageRepeat as BorderImageRepeatType, ImageRepeat as BackgroundRepeatType};
use pi_style::style_parse::style_to_buffer;
use pi_style::{
    style::{
        AllTransform, BaseShape, BlendMode as BlendMode1, BorderImageSlice as BorderImageSlice1, BorderRadius as BorderRadius1,
        BoxShadow as BoxShadow1, Hsi as Hsi1, MaskImage as MaskImage1, TextContent as TextContent1,
    },
    style_parse::Attribute,
    style_type::ClassMeta,
};

// use super::calc::{EntityKey};
// pub use super::root::{ClearColor, RenderDirty, RenderTargetType, Viewport};
use smallvec::SmallVec;
use crate::util::vecmap_default::VecMapWithDefault;
use crate::world::GuiWorldExt;
use map::vecmap::VecMap;

pub type Matrix4 = nalgebra::Matrix4<f32>;
pub type Point2 = nalgebra::Point2<f32>;
pub type Point3 = nalgebra::Point3<f32>;
pub type Vector2 = nalgebra::Vector2<f32>;
pub type Vector3 = nalgebra::Vector3<f32>;
pub type Vector4 = nalgebra::Vector4<f32>;
pub type Entity = usize;
pub type ComponentId = usize;
pub type Setting = GuiWorldExt;
pub type Polygon = Vec<f32>;

#[derive(Clone, Debug, Component, Default, Deref, DerefMut, Serialize, Deserialize)]
#[storage(VecMapWithDefault)]
pub struct RenderContextMark(bitvec::prelude::BitArray);

// type Rectf32 = NotNanRect;

// pub struct RadialWave {
//     /// 是否应用纵横比 - 应用则为 圆形， 否则随纵横比形变
//     pub aspect_ratio: bool,
//     /// 扭曲半径起点 - 渲染范围 [-1, 1]
//     pub start: f32,
//     /// 扭曲半径终点 - 渲染范围 [-1, 1]
//     pub end: f32,
//     /// 扭曲中心点坐标 x - 渲染范围 [-1, 1]
//     pub center_x: f32,
//     /// 扭曲中心点坐标 y - 渲染范围 [-1, 1]
//     pub center_y: f32,
//     /// 波纹周期数
//     pub cycle: u8,
//     /// 扭曲强度
//     pub weight: f32,
// }
pub trait NeedMark {
    fn need_mark(&self) -> bool;
}

// #[derive(Deref, Clone, Debug, Component)]
// pub struct RadialWave(pub pi_postprocess::prelude::RadialWave);

// impl NeedMark for RadialWave {
//     #[inline]
//     fn need_mark(&self) -> bool {
// 		// 不在扭曲范围内， 则不需要扭曲
//         if (self.start >= 1.0 || self.start <= -1.0) && (self.end >= 1.0 || self.end <= -1.0) {
// 			return false;
// 		}
// 		true
//     }
// }

#[derive(Clone, Component, Serialize, Deserialize, Debug)]
#[storage(VecMapWithDefault)]
pub struct RectLayoutStyle {
    pub margin: Rect<Dimension>,
    pub size: Size<Dimension>,
}

impl Default for RectLayoutStyle {
    fn default() -> RectLayoutStyle {
        RectLayoutStyle {
            margin: Default::default(),
            size: Default::default(),
        }
    }
}

#[derive(Clone, Component, Serialize, Deserialize, Debug)]
#[storage(VecMapWithDefault)]
pub struct OtherLayoutStyle {
    pub display: Display,
    pub position_type: PositionType,
    pub direction: Direction,

    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,

    pub order: isize,
    pub flex_basis: Dimension,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub align_self: AlignSelf,

    // pub overflow: Overflow,
    pub position: Rect<Dimension>,
    pub padding: Rect<Dimension>,
    pub border: Rect<Dimension>,
    pub min_size: Size<Dimension>,
    pub max_size: Size<Dimension>,
    pub aspect_ratio: Number,
}

impl Default for OtherLayoutStyle {
    fn default() -> OtherLayoutStyle {
        OtherLayoutStyle {
            display: Default::default(),
            position_type: Default::default(),
            direction: Default::default(),
            flex_direction: Default::default(),
            flex_wrap: Default::default(),
            // overflow: Default::default(),
            align_items: Default::default(),
            align_self: Default::default(),
            // align_content: Default::default(),
            align_content: AlignContent::FlexStart,
            justify_content: Default::default(),
            position: Rect {
                left: Dimension::Undefined,
                right: Dimension::Undefined,
                top: Dimension::Undefined,
                bottom: Dimension::Undefined,
            },
            // position:Default::default(),
            padding: Default::default(),
            border: Default::default(),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            order: 0,
            flex_basis: Dimension::Auto,
            min_size: Default::default(),
            max_size: Default::default(),
            aspect_ratio: Default::default(),
        }
    }
}

#[derive(Deref, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Debug, Component)]
#[storage(VecMapWithDefault)]
pub struct ZIndex(pub isize);

#[derive(Clone, Default, Deref, Debug, Serialize, Deserialize, Component)]
pub struct BoxShadow(pub BoxShadow1);

#[derive(Clone, Default, Deref, Debug, Serialize, Deserialize, Component)]
pub struct Hsi(pub Hsi1);

impl NeedMark for Hsi {
    #[inline]
    fn need_mark(&self) -> bool {
        if self.saturate != 0.0 || self.hue_rotate != 0.0 || self.bright_ness != 0.0 {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Default, Deref, Debug, Serialize, Deserialize, Component)]
pub struct MaskImage(pub MaskImage1);

impl NeedMark for MaskImage {
    fn need_mark(&self) -> bool { true }
}

impl From<Atom> for MaskImage {
    fn from(value: Atom) -> Self { Self(MaskImage1::Path(value)) }
}

// 仅支持Atom的比较， 如果是渐变颜色，一律不相等
impl PartialEq for MaskImage {
    fn eq(&self, other: &Self) -> bool {
        if let MaskImage1::Path(r1) = &self.0 {
            if let MaskImage1::Path(r2) = &other.0 {
                if r1 == r2 {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Clone, Default, Deref, Debug, Serialize, Deserialize, Component)]
pub struct ClipPath(pub BaseShape);
impl NeedMark for ClipPath {
    fn need_mark(&self) -> bool { true }
}

#[derive(Clone, Default, Deref, Debug, Serialize, Deserialize, Component)]
#[storage(VecMapWithDefault)]
pub struct BlendMode(pub BlendMode1);



#[derive(Default, Debug, Clone, Serialize, Deserialize, Component)]
pub struct Animation {
    pub name: AnimationName,                                     // 指定要绑定到选择器的关键帧的名称
    pub duration: SmallVec<[Time; 1]>,                           // 动画指定需要多少毫秒完成
    pub timing_function: SmallVec<[AnimationTimingFunction; 1]>, // 设置动画将如何完成一个周期(插值函数)
    pub iteration_count: SmallVec<[IterationCount; 1]>,
    pub delay: SmallVec<[Time; 1]>,                    // 设置动画在启动前的延迟间隔。
    pub direction: SmallVec<[AnimationDirection; 1]>,  // 指定是否应该轮流反向播放动画。
    pub fill_mode: SmallVec<[AnimationFillMode; 1]>,   // 规定当动画不播放时（当动画完成时，或当动画有一个延迟未开始播放时），要应用到元素的样式。
    pub play_state: SmallVec<[AnimationPlayState; 1]>, // 指定动画是否正在运行或已暂停
}

impl Animation {
    pub fn get_attr<T: Default + Clone>(i: usize, vec: &SmallVec<[T; 1]>) -> T {
        if vec.len() > 0 {
            let i = i % vec.len();
            vec[i].clone()
        } else {
            T::default()
        }
    }
}

//ObjectFit
#[derive(Debug, Clone, Default, Serialize, Deserialize, Hash, Component)]
pub struct BackgroundImageMod {
    pub object_fit: FitType,
    pub repeat: ImageRepeat,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Deref, Component)]
pub struct Blur(pub f32);

impl NeedMark for Blur {
    #[inline]
    fn need_mark(&self) -> bool {
        if self.0 > 0.0 {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Deref, Component)]
#[storage(VecMapWithDefault)]
pub struct BorderRadius(pub BorderRadius1);

#[derive(Clone, Debug, Default, Serialize, Deserialize, Deref, Component, Hash)]
pub struct BorderImageSlice(pi_style::style::BorderImageSlice);

//超出部分的裁剪方式
#[derive(Deref, Clone, Default, Serialize, Deserialize, Debug, Component)]
#[storage(VecMapWithDefault)]
pub struct Overflow(pub bool);

impl NeedMark for Overflow {
    #[inline]
    fn need_mark(&self) -> bool {
        if self.0 == true {
            true
        } else {
            false
        }
    }
}

//不透明度
#[derive(Deref, Clone, Debug, Serialize, Deserialize, Component)]
#[storage(VecMapWithDefault)]
pub struct Opacity(pub f32);

impl NeedMark for Opacity {
    #[inline]
    fn need_mark(&self) -> bool {
        if self.0 < 1.0 {
            true
        } else {
            false
        }
    }
}

#[derive(Deref, Clone, Debug, Serialize, Deserialize, Component, Default)]
pub struct TextContent(pub TextContent1);


// 将display、visibility、enable合并为show组件
#[derive(Deref, Clone, Debug, PartialEq, Serialize, Deserialize, Component)]
#[storage(VecMapWithDefault)]
pub struct Show(pub usize);

// 变换
#[derive(Debug, Clone, Default, Serialize, Deserialize, Component)]
#[storage(VecMapWithDefault)]
pub struct Transform {
    pub all_transform: AllTransform,
    pub origin: TransformOrigin,
}

impl Transform {
    pub fn add_func(&mut self, f: TransformFunc) { self.all_transform.transform.push(f); }
    pub fn set_origin(&mut self, o: TransformOrigin) { self.origin = o; }
}
// 背景色和class
#[derive(Debug, Clone, Default, Serialize, Deserialize, Deref, Component)]
pub struct BackgroundColor(pub Color);

// class名称， 支持多个class， 当只有一个或两个class时， 有优化
#[derive(Debug, Clone, Default, Serialize, Deserialize, Deref, Component)]
pub struct ClassName(pub SmallVec<[usize; 1]>);

// 边框颜色
#[derive(Debug, Clone, Default, Serialize, Deserialize, Deref, Component)]
pub struct BorderColor(pub CgColor);

// 图片路劲及纹理
#[derive(Debug, Deref, Clone, Serialize, Deserialize, Default, Hash, Component, PartialEq, Eq)]
pub struct BackgroundImage(pub Atom);

impl From<Atom> for BackgroundImage {
    fn from(value: Atom) -> Self { BackgroundImage(value) }
}

impl BackgroundImage {
    pub fn set_url() {}
}

#[derive(Debug, Deref, Clone, Serialize, Deserialize, Component)]
pub struct MaskImageClip(pub NotNanRect);

impl Default for MaskImageClip {
    fn default() -> Self {
		MaskImageClip(NotNanRect(Rect {
			left: unsafe{NotNan::new_unchecked(0.0)},
			right: unsafe{NotNan::new_unchecked(1.0)},
			top: unsafe{NotNan::new_unchecked(0.0)},
			bottom: unsafe{NotNan::new_unchecked(1.0)},
		}))
    }
}

// image图像的uv（仅支持百分比， 不支持像素值）
#[derive(Debug, Deref, Clone, Serialize, Deserialize, Component)]
pub struct BackgroundImageClip(pub NotNanRect);

impl Default for BackgroundImageClip {
    fn default() -> Self {
		BackgroundImageClip(NotNanRect(Rect {
			left: unsafe{NotNan::new_unchecked(0.0)},
			right: unsafe{NotNan::new_unchecked(1.0)},
			top: unsafe{NotNan::new_unchecked(0.0)},
			bottom: unsafe{NotNan::new_unchecked(1.0)},
		}))
    }
}

// 边框图片
#[derive(Debug, Deref, Clone, Serialize, Deserialize, Default, Hash, Component, PartialEq, Eq)]
pub struct BorderImage(pub Atom);

impl From<Atom> for BorderImage {
    fn from(value: Atom) -> Self { Self(value) }
}

// borderImage图像的uv（仅支持百分比， 不支持像素值）
#[derive(Debug, Deref, Clone, Serialize, Deserialize, Component, Hash)]
pub struct BorderImageClip(pub NotNanRect);

impl Default for BorderImageClip {
    fn default() -> Self {
		BorderImageClip(NotNanRect(Rect {
			left: unsafe{NotNan::new_unchecked(0.0)},
			right: unsafe{NotNan::new_unchecked(1.0)},
			top: unsafe{NotNan::new_unchecked(0.0)},
			bottom: unsafe{NotNan::new_unchecked(1.0)},
		}))
	}
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Hash, Deref, Component)]
pub struct BorderImageRepeat(pub ImageRepeat);

// 文字样式
#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
pub struct Text {
    pub letter_spacing: f32,     //字符间距， 单位：像素
    pub word_spacing: f32,       //字符间距， 单位：像素
    pub line_height: LineHeight, //设置行高
    pub indent: f32,             // 缩进， 单位： 像素
    pub white_space: WhiteSpace, //空白处理
    pub color: Color,            //颜色
    pub stroke: Stroke,
    pub text_align: TextAlign,
    pub vertical_align: VerticalAlign,
}

#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
#[storage(VecMapWithDefault)]
pub struct TextStyle {
    pub text: Text,
    pub font: Font,
    pub shadow: SmallVec<[TextShadow1; 1]>,
}


#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
pub struct Font {
    pub style: FontStyle, //	规定字体样式。参阅：font-style 中可能的值。
    pub weight: usize,    //	规定字体粗细。参阅：font-weight 中可能的值。
    pub size: FontSize,   //
    pub family: Atom,    //	规定字体系列。参阅：font-family 中可能的值。
}



// impl Default for TextStyle {
//     fn default() -> Self {
//         Self {
//             color: Default::default(),
//             text_indent: Default::default(),
//             text_stroke: Default::default(),
//             text_align: Default::default(),
//             letter_spacing: Default::default(),
//             word_spacing: Default::default(),
//             white_space: Default::default(),
//             line_height: Default::default(),
//             vertical_align: Default::default(),
//             font_style: Default::default(),
//             font_weight: 500,
//             font_size: Default::default(),
//             font_family: Default::default(),
//         }
//     }
// }


pub type TextShadowList = SmallVec<[TextShadow1; 1]>;

// TransformWillChange， 用于优化频繁变化的Transform
#[derive(Default, Debug, Clone, Serialize, Deserialize, Component)]
pub struct TransformWillChange(pub AllTransform);

impl NeedMark for TransformWillChange {
    #[inline]
    fn need_mark(&self) -> bool { true }
}

impl Default for Opacity {
    fn default() -> Opacity { Opacity(1.0) }
}

impl Show {
    #[inline]
    pub fn get_display(&self) -> Display { unsafe { transmute((self.0 & (ShowType::Display as usize)) as u8) } }

    #[inline]
    pub fn set_display(&mut self, display: Display) {
        match display {
            Display::Flex => self.0 &= !(ShowType::Display as usize),
            Display::None => self.0 |= ShowType::Display as usize,
        }
    }

    #[inline]
    pub fn get_visibility(&self) -> bool { (self.0 & (ShowType::Visibility as usize)) != 0 }

    #[inline]
    pub fn set_visibility(&mut self, visibility: bool) {
        if visibility {
            self.0 |= ShowType::Visibility as usize;
        } else {
            self.0 &= !(ShowType::Visibility as usize);
        }
    }

    #[inline]
    pub fn get_enable(&self) -> Enable {
        let r = unsafe { transmute(((self.0 & (ShowType::Enable as usize)) >> 2) as u8) };
        r
    }

    #[inline]
    pub fn set_enable(&mut self, enable: Enable) { self.0 = self.0 & !(ShowType::Enable as usize) | ((enable as usize) << 2); }
}

impl Default for Show {
    fn default() -> Show { Show(ShowType::Visibility as usize) }
}

/// 布局外边距
#[derive(Default, Deref, Clone, Serialize, Deserialize, Debug, Component)]
pub struct Margin(pub Rect<Dimension>);

/// 布局内边距
#[derive(Default, Deref, Clone, Serialize, Deserialize, Debug, Component)]
pub struct Padding(pub Rect<Dimension>);

/// 布局边框尺寸
#[derive(Default, Deref, Clone, Serialize, Deserialize, Debug, Component)]
pub struct Border(pub Rect<Dimension>);

#[derive(Deref, Clone, Serialize, Deserialize, Debug, Component)]
pub struct Position(pub Rect<Dimension>);

#[derive(Default, Clone, Serialize, Deserialize, Debug, Component)]
pub struct MinMax {
    pub min: FlexSize<Dimension>,
    pub max: FlexSize<Dimension>,
}

// 描述子节点行为的flex布局属性
#[derive(Clone, Serialize, Deserialize, Debug, Component)]
pub struct FlexContainer {
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub direction: Direction,
}

// 描述节点自身行为的flex布局属性
#[derive(Clone, Serialize, Deserialize, Debug, Component)]
pub struct FlexNormal {
    pub order: isize,
    pub flex_basis: Dimension,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub align_self: AlignSelf,
    pub position_type: PositionType,
    pub aspect_ratio: Number,
}

impl Default for Position {
    fn default() -> Self {
        Position(Rect {
            left: Dimension::Undefined,
            right: Dimension::Undefined,
            top: Dimension::Undefined,
            bottom: Dimension::Undefined,
        })
    }
}

impl Default for FlexContainer {
    fn default() -> Self {
        FlexContainer {
            flex_direction: Default::default(),
            flex_wrap: Default::default(),
            justify_content: Default::default(),
            align_items: Default::default(),
            align_content: AlignContent::FlexStart,
            direction: Default::default(),
        }
    }
}

impl Default for FlexNormal {
    fn default() -> Self {
        Self {
            order: 0,
            flex_basis: Dimension::Auto,
            flex_grow: Default::default(),
            flex_shrink: Default::default(),
            align_self: Default::default(),
            position_type: Default::default(),
            aspect_ratio: Default::default(),
        }
    }
}

/// 绘制canvas的图节点
#[derive(Debug, Clone, Serialize, Deserialize, Component, Deref)]
pub struct Canvas(pub Entity);

/// 显示改变（一般是指canvas，gui不能感知除了style属性以外的属性改变，如果canvas内容发生改变，应该通过style设置，以便gui能感知，从而设置脏区域）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Component)]
pub struct ShowChange;

// pub fn get_size(s: &FontSize) -> usize {
//     match s {
//         &FontSize::None => {
//             // size
//             32 // 默认32px
//         }
//         &FontSize::Length(r) => r,
//         &FontSize::Percent(_r) => {
//             // (r * size as f32).round() as usize;
//             panic!()
//         }
//     }
// }

pub mod serialize {
    use std::mem::forget;
	use crate::ecs::LendMut;

    use crate::component::user::*;
    use pi_atom::Atom;
    // use pi_ecs::{
    //     prelude::{Query, ResMut},
    //     query::{DefaultComponent, Write},
    // };
    // use bevy_ecs::{
    //     component::ComponentId,
    //     prelude::{Entity, FromWorld, World, Events},
    // };
    // use pi_bevy_ecs_extend::prelude::DefaultComponent;
    use pi_flex_layout::{
        prelude::Number,
        style::{
            AlignContent, AlignItems, AlignSelf, Dimension, Direction, Display, FlexDirection, FlexWrap, JustifyContent,
            PositionType as PositionType1,
        },
    };
    use pi_style::{
        style::{NotNanRect, StyleType},
        style_parse::Attribute,
        style_type::*,
    };


    /// 定义trait ConvertToComponent， 可将buffer转化到ecs组件上
    pub trait ConvertToComponent: Attr {
        /// 将样式属性设置到组件上
        /// ptr为样式属性的指针
        /// 安全： entity必须存在
        fn set<'w, 's>(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity, is_clone: bool)
        where
            Self: Sized;

        /// 为样式设置默认值
        fn set_default(buffer: &Vec<u8>, offset: usize, world: &Setting)
        where
            Self: Sized;

        fn to_attr(ptr: *const u8) -> Attribute
        where
            Self: Sized;
    }

    /// 从Buffer中读取StyleType
    pub struct StyleTypeReader<'a> {
        buffer: &'a Vec<u8>,
        cursor: usize,
        end: usize,
    }

    pub enum StyleSet {
        Set,
        Cancel,
    }

    impl<'a> StyleTypeReader<'a> {
        pub fn default(buffer: &Vec<u8>) -> StyleTypeReader {
            StyleTypeReader {
                buffer,
                cursor: 0,
                end: buffer.len(),
            }
        }

        pub fn new(buffer: &Vec<u8>, start: usize, end: usize) -> StyleTypeReader { StyleTypeReader { buffer, cursor: start, end } }

        // 将当前style写入组件
        // 小心使用该方法， 保证self.buffer中的内存只被使用一次
        pub fn write_to_component(&mut self, cur_style_mark: &mut BitArray<[u32; 3]>, entity: Entity, query: &Setting, is_clone: bool) -> bool {
            let next_type = self.next_type();
            // log::warn!("write_to_component ty: {:?}, cursor:{}, buffer_len:{}", next_type, self.cursor, self.buffer.len());
            if let Some(style_type) = next_type {
                StyleAttr::set(cur_style_mark, style_type, &self.buffer, self.cursor, query, entity, is_clone);
                let size = StyleAttr::size(style_type);
                self.cursor += size;
                return true;
                // return Some(StyleAttr::get_type(style_type));
            }
            false
        }

        // 将当前style写入默认组件
        pub fn write_to_default(&mut self, world: &Setting) -> Option<StyleType> {
            let next_type = self.next_type();
            // log::info!("write_to_component ty: {:?}, cursor:{}, buffer_len:{}", next_type, self.cursor, self.buffer.len());
            if let Some(style_type) = next_type {
                StyleAttr::set_default(style_type, &self.buffer, self.cursor, world);
                let size = StyleAttr::size(style_type);
                self.cursor += size;
                return Some(StyleAttr::get_type(style_type));
            }

            None
        }

        // 将当前style写入组件
        pub fn to_attr(&mut self) -> Option<StyleAttribute> {
            let next_type = self.next_type();
            // log::info!("write_to_component ty: {:?}, cursor:{}, buffer_len:{}", next_type, self.cursor, self.buffer.len());
            if let Some(style_type) = next_type {
                let r = if style_type <= 91 {
                    let r = StyleAttr::to_attr(style_type, &self.buffer, self.cursor);
                    StyleAttribute::Set(r)
                } else {
                    // reset
                    StyleAttribute::Reset(style_type)
                };

                let size = StyleAttr::size(style_type);
                self.cursor += size;
                return Some(r);
                // return Some(StyleAttr::get_type(style_type));
            }
            None
        }

        // f函数返回true，则写入到组件，否则不写入,跳过该属性
        // 同时，使用该函数， 属性将被clone后，放入world中 （设置class时使用， 因为class的buffer会被共享， 如果属性中存在堆属性， 堆被共享为多个所有权， 将会出现未知错误）
        pub fn or_write_to_component<F: Fn(StyleType) -> bool>(
            &mut self,
            cur_style_mark: &mut BitArray<[u32; 3]>,
            entity: Entity,
            query: &Setting,
            f: F,
        ) -> Option<StyleType> {
            let next_type = self.next_type();
            if let Some(style_type) = next_type {
                let ty = StyleAttr::get_type(style_type);
                if f(ty) {
                    StyleAttr::set(cur_style_mark, style_type, &self.buffer, self.cursor, query, entity, true);
                }
                let size = StyleAttr::size(style_type);
                self.cursor += size;
                return Some(ty);
            }
            None
        }

        // 读下一个样式类型
        fn next_type(&mut self) -> Option<u8> {
            if self.cursor >= self.end {
                return None;
            }

            // let ty_size = std::mem::size_of::<u8>();
            let ty = unsafe { Some(self.buffer.as_ptr().add(self.cursor).cast::<u8>().read_unaligned()) };

            // log::info!("next_type ty: {:?}, type_size:{:?}", ty, ty_size);
            // self.cursor += ty_size;
            self.cursor += 1;
            ty
        }
    }


	// macro_rules! get_value {
	// 	($value_ty: ty, $ptr: ident, $is_clone: expr) => {
	// 		{
	// 			// 
	// 			let v = $ptr.cast::<$value_ty>();
	// 			if $is_clone {
	// 				clone_unaligned(v)
	// 			} else {
	// 				unsafe { v.read_unaligned() }
	// 			}
	// 		}
	// 	}
	// }

	macro_rules! set_fun {
		($value_ty: ty, $query: ident, $entity: ident, $v: ident, $set_expr: expr) => {
            fn set(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, $query: &Setting, $entity: Entity, is_clone: bool) {
				let v = ptr.cast::<$value_ty>();
				let $v = if is_clone {
					clone_unaligned(v)
				} else {
					unsafe { v.read_unaligned() }
				};

				log::debug!("set_style, id: {:?}, type={:?}, value={:?}", $entity, std::any::type_name::<Self>(), $v);
				set_fun!(@inner cur_style_mark, $set_expr);
            }
        };

		(@inner $cur_style_mark: ident, $set_expr: expr) => {
			$cur_style_mark.set(Self::get_type() as usize, true);
			$set_expr;
        };
	}
	

    macro_rules! set {
        // 整体插入
        ($name: ident, $value_ty: ty) => {
			set_fun!($value_ty, query, entity, v, query.$name.lend_mut().insert(entity, v));
        };
        // // 表达式
        // (@fun $name: ident, $value_ty: ty, $f: expr) => {
		// 	set_fun!($value_ty, query, entity, v, $crate::paste::item! {
		// 		let attr = query.$name.lend_mut();
		// 		let r = &mut attr[entity];
		// 		r.$feild = v;
		// 	});

        //     fn set(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity, is_clone: bool) {
        //         let v = get_value!($value_ty, ptr, is_clone);
        //         set_style_attr(&mut query.world, entity, query.style.$name, query.style.default.$name, v, $f);
        //     }
        // };

        // 包装
        (@pack $name: ident, $pack_ty: ident, $value_ty: ty) => {
            set_fun!($value_ty, query, entity, v, query.$name.lend_mut().insert(entity, $pack_ty(v)));
        };

        // 一层字段修改
        ($name: ident, $c_ty: ty, $feild: ident, $value_ty: ty) => {
			set_fun!($value_ty, query, entity, v, $crate::paste::item! {
				let attr = query.$name.lend_mut();
				let r = &mut attr[entity];
				r.$feild = v;
				attr.get_notify_ref().modify_event(entity, "", 0);
			});
        };

		// 两层属性修改属性修改
        ($name: ident, $c_ty: ty, $feild1: ident, $feild2: ident, $value_ty: ty) => {
			set_fun!($value_ty, query, entity, v, $crate::paste::item! {
				let attr = query.$name.lend_mut();
				let r = &mut attr[entity];
				r.$feild1.$feild2 = v;
				attr.get_notify_ref().modify_event(entity, "", 0);
			});
        };

        // 用方法修改属性
        (@func $name: ident, $c_ty: ty, $set_func: ident, $value_ty: ty) => {
			set_fun!($value_ty, query, entity, v, $crate::paste::item! {
				let attr = query.$name.lend_mut();
				let r = &mut attr[entity];
				r.$set_func(v);
				attr.get_notify_ref().modify_event(entity, "", 0);
			});
        };

        // // 盒模属性（上右下左）
        // (@box_model $name: ident, $value_ty: ty) => {
		// 	set_fun!($value_ty, query, entity, v, $crate::paste::item! {
		// 		let attr = query.$name.lend_mut();
		// 		attr.set_func(v);
		// 	});

        //     fn set(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity, is_clone: bool) {
        //         let v = ptr.cast::<$value_ty>();
        //         let v = if is_clone {
        //             clone_unaligned(v)
        //         } else {
        //             unsafe { v.read_unaligned() }
        //         };

        //         set_style_attr(
        //             &mut query.world,
        //             entity,
        //             query.style.$name,
        //             query.style.default.$name,
        //             v,
        //             |item: &mut $value_ty, v: $value_ty| {
        //                 *item = v;
        //             },
        //         );

        //         cur_style_mark.set(Self::get_type() as usize, true);
        //     }
        // };
    }

	macro_rules! set_default_fun {
		($value_ty: ty, $query: ident, $v: ident, $set_expr: expr) => {
            fn set_default<'a>(buffer: &Vec<u8>, offset: usize, $query: &Setting) {
				let $v = unsafe { buffer.as_ptr().add(offset).cast::<$value_ty>().read_unaligned() };
				log::debug!("set_default_style, type={:?}, value={:?}", std::any::type_name::<Self>(), $v);
				$set_expr;
            }
        };
	}


    // 设置默认值
    macro_rules! set_default {
        (@empty) => {
            fn set_default<'a>(_buffer: &Vec<u8>, _offset: usize, _query: &Setting) {}
        };
        // 整体插入
        ($name: ident, $value_ty: ty) => {
			set_default_fun!($value_ty, query, v, {
				let attr = query.default_components.$name.lend_mut();
				**attr = v.clone();
				if let Some(attr) = query.$name.lend_mut().get_default_mut() {
					*attr = v;
				}
			});
        };
        // 属性修改
        ($name: ident, $c_ty: ty, $feild: ident, $value_ty: ty) => {
			set_default_fun!($value_ty, query, v, {
				let attr = query.default_components.$name.lend_mut();
				attr.$feild = v.clone();
				if let Some(attr) = query.$name.lend_mut().get_default_mut() {
					attr.$feild = v;
				}
			});
        };
        // 属性修改
        (@func $name: ident, $c_ty: ty, $set_func: ident, $value_ty: ty) => {
			set_default_fun!($value_ty, query, v, {
				let attr = query.default_components.$name.lend_mut();
				attr.$set_func(v.clone());
				if let Some(attr) = query.$name.lend_mut().get_default_mut() {
					attr.$set_func(v);
				}
			});
        };

        // 属性修改
        ($name: ident, $c_ty: ty, $feild1: ident, $feild2: ident, $value_ty: ty) => {
			set_default_fun!($value_ty, query, v, {
				let attr = query.default_components.$name.lend_mut();
				attr.$feild1.$feild2 = v.clone();
				if let Some(attr) = query.$name.lend_mut().get_default_mut() {
					attr.$feild1.$feild2 = v;
				}
			});
        };

        // // 盒模属性（上右下左）
        // (@box_model $name: ident, $c_ty: ty, $value_ty: ty) => {
        //     fn set_default<'a>(buffer: &Vec<u8>, offset: usize, query: &DefaultStyle, world: &Setting) {
        //         set_default_style_attr(
        //             world,
        //             query.$name,
        //             unsafe { buffer.as_ptr().add(offset).cast::<$value_ty>().read_unaligned() },
        //             |item: &mut $c_ty, v: $value_ty| {
        //                 c.top = v.top;
        //                 c.right = v.right;
        //                 c.bottom = v.bottom;
        //                 c.left = v.left;
        //             },
        //         );
        //     }
        // };
    }

	macro_rules! reset_fun {
		($query: ident, $entity: ident, $v: ident, $set_expr: expr) => {
            fn set(cur_style_mark: &mut BitArray<[u32; 3]>, _ptr: *const u8, $query: &Setting, $entity: Entity, _is_clone: bool) {
				cur_style_mark.set(Self::get_type() as usize, false);
				log::debug!("reset_style, id: {:?}, type={:?}", $entity, std::any::type_name::<Self>());
				$set_expr;
            }
        };
	}

    macro_rules! reset {
        // 空实现
        (@empty) => {
            fn set(_cur_style_mark: &mut BitArray<[u32; 3]>, _ptr: *const u8, _query: &Setting, _entity: Entity, _is_clone: bool) {}
        };
        ($name: ident) => {
			reset_fun!(query, entity, v, {
				let component = query.$name.lend_mut();
				if let Some(component) = component.get_mut(entity) {
					let attr = query.default_components.$name.lend_mut();
					*component = attr.clone();
					attr.get_notify_ref().modify_event(entity, "", 0);
				}
			});
        };
        // 属性修改
        ($name: ident, $c_ty: ty, $feild: ident) => {
			reset_fun!(query, entity, v, {
				let component = query.$name.lend_mut();
				if let Some(component) = component.get_mut(entity) {
					let attr = query.default_components.$name.lend_mut();
					component.$feild = attr.$feild.clone();
					attr.get_notify_ref().modify_event(entity, "", 0);
				}
			});
        };

        // 属性修改
        (@func $name: ident, $c_ty: ty, $set_func: ident, $get_func: ident) => {
			reset_fun!(query, entity, v, {
				let component = query.$name.lend_mut();
				if let Some(component) = component.get_mut(entity) {
					let attr = query.default_components.$name.lend_mut();
					component.$set_func(attr.$get_func());
					attr.get_notify_ref().modify_event(entity, "", 0);
				}
			});
        };
        // 属性修改
        ($name: ident, $c_ty: ty, $feild1: ident, $feild2: ident) => {
			reset_fun!(query, entity, v, {
				let component = query.$name.lend_mut();
				if let Some(component) = component.get_mut(entity) {
					let attr = query.default_components.$name.lend_mut();
					component.$feild1.$feild2 = attr.$feild1.$feild2.clone();
				}
			});
        };

        (@box_model $name: ident, $ty: ident) => {
            fn set(cur_style_mark: &mut BitArray<[u32; 3]>, _ptr: *const u8, query: &Setting, entity: Entity, _is_clone: bool) {
				reset_fun!(query, entity, v, {
					let component = query.$name.lend_mut();
					if let Some(component) = component.get_mut(entity) {
						let attr = query.default_components.$name.lend_mut();
						$crate::paste::item! {
							if !cur_style_mark[StyleType::[<$ty Top>] as usize] {
								is_changed = true;
								item.top = v.top;
							}
							if !cur_style_mark[StyleType::[<$ty Right>] as usize] {
								is_changed = true;
								item.right = v.right;
							}
							if !cur_style_mark[StyleType::[<$ty Bottom>] as usize] {
								is_changed = true;
								item.bottom = v.bottom;
							}
							if !cur_style_mark[StyleType::[<$ty Left>] as usize] {
								is_changed = true;
								item.left = v.left;
							}
						}
						attr.get_notify_ref().modify_event(entity, "", 0);
					}
				});
            }
        };
    }

    macro_rules! impl_style {
	($struct_name: ident) => {
		impl ConvertToComponent for $struct_name {
			reset!(@empty);
			// reset!($name, $ty);
			#[allow(unused_variables)]
			fn set_default<'a>(_buffer: &Vec<u8>, _offset: usize, _query: &Setting) {

			}
			fn to_attr(_ptr: *const u8) -> Attribute
			{
				todo!()
			}
		}
	};
	($struct_name: ident, $name: ident, $ty: ident) => {

		impl ConvertToComponent for $struct_name {
			set!($name, $ty);
			// reset!($name, $ty);
			set_default!($name, $ty);
			fn to_attr(ptr: *const u8) -> Attribute
			{
				Attribute::$ty($struct_name(clone_unaligned(ptr.cast::<$ty>())))
			}
		}

		$crate::paste::item! {

			impl ConvertToComponent for [<Reset $struct_name>] {
				reset!($name, $ty);
				set_default!($name, $ty);
				fn to_attr(_ptr: *const u8) -> Attribute
				{
					Attribute::$ty($struct_name(Default::default()))
				}
			}

		}
	};

	(@pack $struct_name: ident, $name: ident, $pack_ty: ident, $value_ty: ident) => {

		impl ConvertToComponent for $struct_name {
			set!(@pack $name, $pack_ty, $value_ty);
			// set!(@pack $name, $pack_ty, $value_ty);
			// reset!($name, $ty);
			set_default!($name, $pack_ty);
			fn to_attr(ptr: *const u8) -> Attribute
			{
				Attribute::$pack_ty($struct_name(clone_unaligned(ptr.cast::<$value_ty>())))
			}
		}

		$crate::paste::item! {

			impl ConvertToComponent for [<Reset $struct_name>] {
				reset!($name);
				set_default!($name, $pack_ty);
				fn to_attr(_ptr: *const u8) -> Attribute
				{
					Attribute::$pack_ty( $struct_name(Default::default()) )
				}
			}
		}
	};
	(@pack_send $struct_name: ident, $name: ident, $pack_ty: ident, $value_ty: ident) => {

		impl ConvertToComponent for $struct_name {
			// set!(@fun $name, $value_ty, $pack_ty, |item: &mut $pack_ty, v: $value_ty| *item = $pack_ty(v));
			set!(@pack $name, $pack_ty, $value_ty);
			// reset!($name, $ty);
			set_default!($name, $pack_ty);
			fn to_attr(ptr: *const u8) -> Attribute
			{
				Attribute::$pack_ty($struct_name(clone_unaligned(ptr.cast::<$value_ty>())))
			}
		}

		$crate::paste::item! {

			impl ConvertToComponent for [<Reset $struct_name>] {
				reset!($name);
				set_default!($name, $pack_ty);
				fn to_attr(_ptr: *const u8) -> Attribute
				{
					Attribute::$pack_ty( $struct_name(Default::default()) )
				}
			}
		}
	};
	($struct_name: ident, $name: ident, $c_ty: ident, $ty: ident, $value_ty: ident) => {
		impl ConvertToComponent for $struct_name {
			set!($name, $c_ty, $value_ty);
			// reset!($name);
			set_default!($name, $c_ty, $value_ty);
			fn to_attr(ptr: *const u8) -> Attribute
			{
				Attribute::$ty($struct_name(clone_unaligned(ptr.cast::<$value_ty>())))
			}
		}

		$crate::paste::item! {

			impl ConvertToComponent for [<Reset $struct_name>] {
				reset!($name, $c_ty);
				set_default!($name, $c_ty, $value_ty);
				fn to_attr(_ptr: *const u8) -> Attribute
				{
					Attribute::$ty( $struct_name(Default::default()) )
				}
			}
		}
	};
	($struct_name: ident, $name: ident, $c_ty: ident, $feild: ident, $ty: ident, $value_ty: ty) => {
		impl ConvertToComponent for $struct_name {
			set!($name, $c_ty, $feild, $value_ty);
			// reset!($name, $feild);
			set_default!($name, $c_ty, $feild, $value_ty);
			fn to_attr(ptr: *const u8) -> Attribute
        where
            Self: Sized
			{
				Attribute::$ty($struct_name(clone_unaligned(ptr.cast::<$value_ty>())))
			}
		}

		$crate::paste::item! {

			impl ConvertToComponent for [<Reset $struct_name>] {
				reset!($name, $c_ty, $feild);
				set_default!($name, $c_ty, $feild, $value_ty);
				fn to_attr(_ptr: *const u8) -> Attribute
				{
					Attribute::$ty($struct_name(Default::default()) )
				}
			}
		}
	};
	($struct_name: ident, $name: ident, $c_ty: ident, $feild1: ident, $feild2: ident, $ty: ident, $value_ty: ident) => {
		impl ConvertToComponent for $struct_name {
			set!($name, $c_ty, $feild1, $feild2, $value_ty);
			// reset!($name, $feild1, $feild2);
			set_default!($name, $c_ty, $feild1, $feild2, $value_ty);
			fn to_attr(ptr: *const u8) -> Attribute
        where
            Self: Sized
			{
				Attribute::$ty($struct_name(clone_unaligned(ptr.cast::<$value_ty>())))
			}
		}

		$crate::paste::item! {

			impl ConvertToComponent for [<Reset $struct_name>] {
				reset!($name, $c_ty, $feild1, $feild2);
				set_default!($name, $c_ty, $feild1, $feild2, $value_ty);
				fn to_attr(_ptr: *const u8) -> Attribute
				{
					Attribute::$ty( $struct_name(Default::default()) )
				}
			}
		}
	};
	(@func $struct_name: ident, $name: ident, $c_ty: ty, $set_func: ident, $get_func: ident, $ty: ident, $value_ty: ident) => {
		impl ConvertToComponent for $struct_name {
			set!(@func $name, $c_ty, $set_func, $value_ty);
			// reset!(@func $name, $set_func, $get_func);
			set_default!(@func $name, $c_ty, $set_func, $value_ty);
			fn to_attr(ptr: *const u8) -> Attribute
			{
				Attribute::$ty($struct_name(clone_unaligned(ptr.cast::<$value_ty>())))
			}
		}

		$crate::paste::item! {

			impl ConvertToComponent for [<Reset $struct_name>] {
				reset!(@func $name, $c_ty, $set_func, $get_func);
				set_default!(@func $name, $c_ty, $set_func, $value_ty);
				fn to_attr(_ptr: *const u8) -> Attribute
				{
					Attribute::$ty( $struct_name(Default::default()) )
				}
			}
		}
	};
	// 方法设置，并且不实现set_default和reset
	(@func $struct_name: ident,  $name: ident, $c_ty: ty, $set_func: ident, $ty: ident, $value_ty: ident) => {
		impl ConvertToComponent for $struct_name {
			set!(@func $name, $c_ty, $set_func, $value_ty);
			// reset!(@empty);
			set_default!(@empty);
			fn to_attr(ptr: *const u8) -> Attribute
			{
				Attribute::$ty($struct_name(clone_unaligned(ptr.cast::<$value_ty>())))
			}
		}

		$crate::paste::item! {

			impl ConvertToComponent for [<Reset $struct_name>] {
				reset!(@empty);
				set_default!(@empty);
				fn to_attr(_ptr: *const u8) -> Attribute
				{
					Attribute::$ty( $struct_name(Default::default()) )
				}
			}
		}
	};

	(@func1 $struct_name: ident,  $name: ident, $c_ty: ty, $set_func: ident, $ty: ident, $attr_ty: ident,  $value_ty: ident) => {
		impl ConvertToComponent for $struct_name {
			set!(@func $name, $c_ty, $set_func, $value_ty);
			// reset!(@empty);
			set_default!(@empty);
			fn to_attr(ptr: *const u8) -> Attribute
			{
				Attribute::$attr_ty($struct_name(clone_unaligned(ptr.cast::<$value_ty>())))
			}
		}

		$crate::paste::item! {

			impl ConvertToComponent for [<Reset $struct_name>] {
				reset!(@empty);
				set_default!(@empty);
				fn to_attr(_ptr: *const u8) -> Attribute
				{
					Attribute::$attr_ty( $struct_name(Default::default()) )
				}
			}
		}
	};

	(@box_model_single $struct_name: ident, $name: ident, $c_ty: ty, $feild: ident, $ty: ident, $value_ty: ident) => {
		impl ConvertToComponent for $struct_name {
			set!($name, $c_ty, $feild, $value_ty);
			// reset!(@box_model_single $name, $feild, $ty_all);
			set_default!($name, $c_ty, $feild, $value_ty);
			fn to_attr(ptr: *const u8) -> Attribute
			{
				Attribute::$ty($struct_name(clone_unaligned(ptr.cast::<$value_ty>())))
			}
		}

		$crate::paste::item! {

			impl ConvertToComponent for [<Reset $struct_name>] {
				reset!($name, $c_ty, $feild);
				set_default!($name, $c_ty, $feild, $value_ty);
				fn to_attr(_ptr: *const u8) -> Attribute
				{
					Attribute::$ty($struct_name(Default::default()) )
				}
			}
		}
	};
	(@box_model $struct_name: ident, $name: ident, $ty: ident) => {
		impl ConvertToComponent for $struct_name {
			set!(@box_model $name, $ty);
			// reset!(@box_model $name, $ty);
			set_default!(@box_model $name, $ty);
			fn to_attr(ptr: *const u8) -> Attribute
			{
				Attribute::$ty($struct_name(clone_unaligned(ptr.cast::<$ty>())))
			}
		}

		$crate::paste::item! {
			impl ConvertToComponent for [<Reset $struct_name>] {
				reset!(@box_model $name, $ty);
				set_default!(@box_model $name, $ty);
				fn to_attr(_ptr: *const u8) -> Attribute
				{
					Attribute::$ty( $struct_name(Default::default()) )
				}
			}
		}
	};
}

    impl_style!(EmptyType);


    impl_style!(FontStyleType, text_style, TextStyle, font, style, FontStyle, FontStyle);

    impl_style!(FontWeightType, text_style, TextStyle, font, weight, FontWeight, usize);
    impl_style!(FontSizeType, text_style, TextStyle, font, size, FontSize, FontSize);
    impl_style!(FontFamilyType, text_style, TextStyle, font, family, FontFamily, Atom);
    impl_style!(LetterSpacingType, text_style, TextStyle, text, letter_spacing, LetterSpacing, f32);
    impl_style!(WordSpacingType, text_style, TextStyle, text, word_spacing, WordSpacing, f32);
    impl_style!(LineHeightType, text_style, TextStyle, text, line_height, LineHeight, LineHeight);
    impl_style!(TextIndentType, text_style, TextStyle, text, indent, TextIndent, f32);
    impl_style!(WhiteSpaceType, text_style, TextStyle, text, white_space, WhiteSpace, WhiteSpace);
    // impl ConvertToComponent for WhiteSpaceType {
    // 	// 设置white_space,需要同时设置flex_wrap
    // 	fn set(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity)
    // 		where
    // 			Self: Sized {
    // 		// 取不到说明实体已经销毁
    // 		let white_space = query.style.default.text_style.white_space.clone();
    // 		// let flex_wrap = query.style.default.text_style.flex_container.flex_wrap.clone();

    // 		if let (Ok(mut text_style_item), Ok(mut flex_container_item)) = (query.style.text_style.get_mut(entity), query.style.flex_container.get_mut(entity)) {
    // 			let v = unsafe { ptr.cast::<WhiteSpace>().read_unaligned() };
    // 			log::info!("set_style, type: {:?}, value: {:?}, entity: {:?}", std::any::type_name::<Self>(), v, entity);

    // 			cur_style_mark.set(Self::get_type() as usize, true);
    // 			cur_style_mark.set(JustifyContentType::get_type() as usize, true);

    // 			text_style_item.white_space = white_space;
    // 			// text_style_item.notify_modify();

    // 			flex_container_item.flex_wrap = if v.allow_wrap() {
    // 				FlexWrap::Wrap
    // 			} else {
    // 				FlexWrap::NoWrap
    // 			};
    // 			// flex_container_item.notify_modify();
    // 		}


    // 	}

    // 	set_default!(text_style, white_space, WhiteSpace);
    // 	fn to_attr(ptr: *const u8) -> Attribute{
    // 		Attribute::WhiteSpace(unsafe { WhiteSpaceType(ptr.cast::<WhiteSpace>().read_unaligned()) })
    // 	}
    // }

    // impl ConvertToComponent for ResetWhiteSpaceType {
    // 	fn set(_cur_style_mark: &mut BitArray<[u32; 3]>, _ptr: *const u8, query: &Setting, entity: Entity)
    // 		where
    // 			Self: Sized {

    // 		if let (Ok(mut text_style_item), Ok(mut flex_container_item)) = (query.style.text_style.get_mut(entity), query.style.flex_container.get_mut(entity)) {
    // 			let white_space = query.style.default.text_style.white_space.clone();
    // 			text_style_item.white_space = white_space;
    // 			// text_style_item.notify_modify();

    // 			flex_container_item.flex_wrap = if white_space.allow_wrap() {
    // 				FlexWrap::Wrap
    // 			} else {
    // 				FlexWrap::NoWrap
    // 			};
    // 			// flex_container_item.notify_modify();
    // 		}
    // 	}

    // 	set_default!(text_style, white_space, WhiteSpace);
    // 	fn to_attr(_ptr: *const u8) -> Attribute{
    // 		todo!()
    // 		// Attribute::WhiteSpace(unsafe { WhiteSpaceType(ptr.cast::<WhiteSpace>().read_unaligned()) })
    // 	}
    // }
    impl_style!(TextAlignType, text_style, TextStyle, text, text_align, TextAlign, TextAlign);

    impl_style!(@pack TextContentType, text_content, TextContent, TextContent1);
    // impl ConvertToComponent for TextContentType {
    //     // 设置text_align,需要同时设置justify_content
    //     fn set(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity, is_clone: bool)
    //     where
    //         Self: Sized,
    //     {
    //         let v = ptr.cast::<TextContent1>();
    //         let v = if is_clone {
    //             clone_unaligned(v)
    //         } else {
    //             unsafe { v.read_unaligned() }
    //         };
    //         cur_style_mark.set(Self::get_type() as usize, true);
    //         set_style_attr(
    //             &mut query.world,
    //             entity,
    //             query.style.text_content,
    //             query.style.default.text_content,
    //             v,
    //             |item: &mut TextContent, v| {
    //                 item.0 = v;
    //             },
    //         );
    //         // 发送事件
    //         if let Some(component) = query.world.get_resource_mut_by_id(query.style.event.text_content) {
    //             unsafe { component.into_inner().deref_mut::<Events<ComponentEvent<Changed<TextContent>>>>() }
    //                 .send(ComponentEvent::<Changed<TextContent>>::new(entity));
    //         };


    //         // 插入默认的FlexContainer组件
    //         if let None = query.world.get_mut_by_id(entity, query.style.flex_container) {
    //             let default_value = query.world.get_resource_by_id(query.style.default.flex_container).unwrap();
    //             let r = unsafe { default_value.deref::<DefaultComponent<FlexContainer>>() }.0.clone();
    //             query.world.entity_mut(entity).insert(r);
    //         };
    //     }

    //     set_default!(text_content, TextContent);
    //     fn to_attr(ptr: *const u8) -> Attribute { 
	// 		let r = Attribute::TextContent(TextContentType(clone_unaligned(ptr.cast::<TextContent1>())));
	// 		r
	// 	}
    // }

    // impl ConvertToComponent for ResetTextContentType {
    //     reset!(text_content);
    //     set_default!(text_content, TextContent);
    //     fn to_attr(_ptr: *const u8) -> Attribute { Attribute::TextContent(TextContentType(Default::default())) }
    // }

    // impl ConvertToComponent for TextAlignType {
    // 	// 设置text_align,需要同时设置justify_content
    // 	fn set(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity)
    // 		where
    // 			Self: Sized {
    // 		// 取不到说明实体已经销毁
    // 		if let (Ok(mut text_style_item), Ok(mut flex_container_item)) = (query.style.text_style.get_mut(entity), query.style.flex_container.get_mut(entity)) {
    // 			let v = unsafe { ptr.cast::<TextAlign>().read_unaligned() };
    // 			log::info!("set_style, type: {:?}, value: {:?}, entity: {:?}", std::any::type_name::<Self>(), v, entity);

    // 			cur_style_mark.set(Self::get_type() as usize, true);
    // 			cur_style_mark.set(JustifyContentType::get_type() as usize, true);

    // 			text_style_item.text_align = v;
    // 			// text_style_item.notify_modify();

    // 			flex_container_item.justify_content = match v {
    // 				TextAlign::Center => JustifyContent::Center,
    // 				TextAlign::Right => JustifyContent::FlexEnd,
    // 				TextAlign::Left => JustifyContent::FlexStart,
    // 				TextAlign::Justify => JustifyContent::SpaceBetween,
    // 			};
    // 			// flex_container_item.notify_modify();
    // 		}


    // 	}

    // 	set_default!(text_style, text_align, TextAlign);
    // 	fn to_attr(ptr: *const u8) -> Attribute{
    // 		Attribute::TextAlign(unsafe { TextAlignType(ptr.cast::<TextAlign>().read_unaligned()) })
    // 	}
    // }

    // impl ConvertToComponent for ResetTextAlignType {
    // 	fn set(_cur_style_mark: &mut BitArray<[u32; 3]>, _ptr: *const u8, query: &Setting, entity: Entity)
    // 		where
    // 			Self: Sized {
    // 		if let (Ok(mut text_style_item), Ok(mut flex_container_item)) = (query.style.text_style.get_mut(entity), query.style.flex_container.get_mut(entity)) {
    // 			let v = query.style.default.text_style.text_align.clone();
    // 			text_style_item.text_align = v;
    // 			// text_style_item.notify_modify();

    // 			flex_container_item.justify_content = match v {
    // 				TextAlign::Center => JustifyContent::Center,
    // 				TextAlign::Right => JustifyContent::FlexEnd,
    // 				TextAlign::Left => JustifyContent::FlexStart,
    // 				TextAlign::Justify => JustifyContent::SpaceBetween,
    // 			};
    // 			// flex_container_item.notify_modify();
    // 		}

    // 	}

    // 	set_default!(text_style, text_align, TextAlign);
    // 	fn to_attr(_ptr: *const u8) -> Attribute{
    // 		todo!()
    // 	}
    // }

    impl_style!(VerticalAlignType, text_style, TextStyle, text, vertical_align, VerticalAlign, VerticalAlign);
    // impl ConvertToComponent for VerticalAlignType {
    // 	// 设置vertical_align,需要同时设置jalign_items, align_content
    // 	fn set(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity)
    // 		where
    // 			Self: Sized {
    // 		if let (Ok(mut text_style_item), Ok(mut flex_container_item)) = (query.style.text_style.get_mut(entity), query.style.flex_container.get_mut(entity)) {
    // 			let v = unsafe { ptr.cast::<VerticalAlign>().read_unaligned() };
    // 			log::info!("set_style, type: {:?}, value: {:?}, entity: {:?}", std::any::type_name::<Self>(), v, entity);

    // 			cur_style_mark.set(Self::get_type() as usize, true);
    // 			cur_style_mark.set(JustifyContentType::get_type() as usize, true);

    // 			text_style_item.vertical_align = v;
    // 			// text_style_item.notify_modify();

    // 			flex_container_item.align_content = match v {
    // 				VerticalAlign::Middle => AlignContent::Center,
    // 				VerticalAlign::Bottom => AlignContent::FlexEnd,
    // 				VerticalAlign::Top => AlignContent::FlexStart,
    // 			};
    // 			flex_container_item.align_items = match v {
    // 				VerticalAlign::Middle => AlignItems::Center,
    // 				VerticalAlign::Bottom => AlignItems::FlexEnd,
    // 				VerticalAlign::Top => AlignItems::FlexStart,
    // 			};
    // 			// flex_container_item.notify_modify();
    // 		}
    // 	}

    // 	set_default!(text_style, vertical_align, VerticalAlign);
    // 	fn to_attr(ptr: *const u8) -> Attribute{
    // 		Attribute::VerticalAlign(unsafe { VerticalAlignType(ptr.cast::<VerticalAlign>().read_unaligned()) })
    // 	}
    // }

    // impl ConvertToComponent for ResetVerticalAlignType {
    // 	fn set(_cur_style_mark: &mut BitArray<[u32; 3]>, _ptr: *const u8, query: &Setting, entity: Entity)
    // 		where
    // 			Self: Sized {
    // 		if let (Ok(mut text_style_item), Ok(mut flex_container_item)) = (query.style.text_style.get_mut(entity), query.style.flex_container.get_mut(entity)) {
    // 			let v = query.style.default.text_style.vertical_align.clone();
    // 			text_style_item.vertical_align = v;
    // 			// text_style_item.notify_modify();

    // 			flex_container_item.align_content = match v {
    // 				VerticalAlign::Middle => AlignContent::Center,
    // 				VerticalAlign::Bottom => AlignContent::FlexEnd,
    // 				VerticalAlign::Top => AlignContent::FlexStart,
    // 			};
    // 			flex_container_item.align_items = match v {
    // 				VerticalAlign::Middle => AlignItems::Center,
    // 				VerticalAlign::Bottom => AlignItems::FlexEnd,
    // 				VerticalAlign::Top => AlignItems::FlexStart,
    // 			};
    // 			// flex_container_item.notify_modify();
    // 		}
    // 	}

    // 	set_default!(text_style, vertical_align, VerticalAlign);
    // 	fn to_attr(_ptr: *const u8) -> Attribute{
    // 		todo!()
    // 	}
    // }

    impl_style!(ColorType, text_style, TextStyle, text, color, Color, Color);
    impl_style!(TextStrokeType, text_style, TextStyle, text, stroke, TextStroke, Stroke);
    impl_style!(TextShadowType, text_style, TextStyle, shadow, TextShadow, TextShadowList);

	impl_style!(@pack_send BackgroundImageType, background_image, BackgroundImage, Atom);
    
    impl_style!(@pack BackgroundImageClipType, background_image_clip, BackgroundImageClip, NotNanRect);
    impl_style!(ObjectFitType, background_image_mod, BackgroundImageMod, object_fit, ObjectFit, FitType);
    impl_style!(
        BackgroundRepeatType,
        background_image_mod,
        BackgroundImageMod,
        repeat,
        BackgroundRepeat,
        ImageRepeat
    );

	impl_style!(@pack_send BorderImageType, border_image, BorderImage, Atom);

    impl ConvertToComponent for TransformFuncType {
        /// 将样式属性设置到组件上
        /// ptr为样式属性的指针
        /// 安全： entity必须存在
        fn set<'w, 's>(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity, is_clone: bool)
        where
            Self: Sized,
        {
            let v = ptr.cast::<TransformFunc>();
            let v = if is_clone {
                clone_unaligned(v)
            } else {
                unsafe { v.read_unaligned() }
            };
            cur_style_mark.set(Self::get_type() as usize, true);

            log::debug!(
                "set_style_attr, type: {:?}, value: {:?}, id: {:?}",
                std::any::type_name::<TransformFunc>(),
                v,
                entity
            );
            match query.transform_will_change.lend_mut().get_mut(entity) {
                Some(component) => {
                    component.0.transform.push(v);
					query.transform_will_change.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                }
                None => {
                    // 不存在transform_willChange， 则设置在Transfrom上
                    match query.transform.lend_mut().get_mut(entity) {
                        Some(component) => {
                            // 如果存在transform_willChange,则将Transform设置在TransformWillChange上
                            component.all_transform.transform.push(v);
							query.transform.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                        }
                        None => {
							query.transform.lend_mut().insert(entity, Transform {
                                all_transform: AllTransform {
                                    transform: vec![v],
                                    ..Default::default()
                                },
                                ..Default::default()
                            });
                        }
                    }
                }
            };
        }

        /// 为样式设置默认值
        fn set_default(_buffer: &Vec<u8>, _offset: usize, _world: &Setting)
        where
            Self: Sized,
        {
        }

        fn to_attr(_ptr: *const u8) -> Attribute
        where
            Self: Sized,
        {
            todo!();
            // Attribute::Transform(unsafe { TransformType(ptr.cast::<TransformFuncs>().read_unaligned()) })
        }
    }
    impl ConvertToComponent for ResetTransformFuncType {
        /// 将样式属性设置到组件上
        /// ptr为样式属性的指针
        /// 安全： entity必须存在
        fn set<'w, 's>(_cur_style_mark: &mut BitArray<[u32; 3]>, _ptr: *const u8, _query: &Setting, _entity: Entity, _is_clone: bool)
        where
            Self: Sized,
        {
            todo!()
        }

        /// 为样式设置默认值
        fn set_default(_buffer: &Vec<u8>, _offset: usize, _world: &Setting)
        where
            Self: Sized,
        {
            todo!()
        }

        fn to_attr(_ptr: *const u8) -> Attribute
        where
            Self: Sized,
        {
            todo!()
        }
    }

    impl ConvertToComponent for TransformType {
        /// 将样式属性设置到组件上
        /// ptr为样式属性的指针
        /// 安全： entity必须存在
        fn set<'w, 's>(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity, is_clone: bool)
        where
            Self: Sized,
        {
            let v = ptr.cast::<TransformFuncs>();
            let v = if is_clone {
                clone_unaligned(v)
            } else {
                unsafe { v.read_unaligned() }
            };
            cur_style_mark.set(Self::get_type() as usize, true);

            log::debug!(
                "set_style_attr, type: {:?}, value: {:?}, id: {:?}",
                std::any::type_name::<Transform>(),
                v,
                entity
            );
            match query.transform_will_change.lend_mut().get_mut(entity) {
                Some(component) => {
                    // 如果存在transform_willChange,则将Transform设置在TransformWillChange上
                    // component.set_changed();
                    component.0.transform = v;
					query.transform_will_change.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                }
                None => {
                    // 不存在transform_willChange， 则设置在Transfrom上
                    match query.transform.lend_mut().get_mut(entity) {
                        Some(component) => {
                            // 如果存在transform_willChange,则将Transform设置在TransformWillChange上
                            component.all_transform.transform = v;
							query.transform.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                        }
                        None => {
                            query.transform.lend_mut().insert(entity, Transform {
                                all_transform: AllTransform {
                                    transform: v,
                                    ..Default::default()
                                },
                                ..Default::default()
                            });
                        }
                    }
                }
            };
        }

        /// 为样式设置默认值
        fn set_default(_buffer: &Vec<u8>, _offset: usize, _world: &Setting)
        where
            Self: Sized,
        {
        }

        fn to_attr(ptr: *const u8) -> Attribute
        where
            Self: Sized,
        {
            Attribute::Transform(TransformType(clone_unaligned(ptr.cast::<TransformFuncs>())))
        }
    }
    impl ConvertToComponent for ResetTransformType {
        /// 将样式属性设置到组件上
        /// ptr为样式属性的指针
        /// 安全： entity必须存在
        fn set<'w, 's>(_cur_style_mark: &mut BitArray<[u32; 3]>, _ptr: *const u8, query: &Setting, entity: Entity, _is_clone: bool)
        where
            Self: Sized,
        {
            match query.transform_will_change.lend_mut().get_mut(entity) {
                Some(component) => {
                    component.0.transform = Default::default();
					query.transform_will_change.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                }
                None => match query.transform.lend_mut().get_mut(entity) {
                    Some(component) => {
                        component.all_transform.transform = Default::default();
						query.transform.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                    }
                    None => (),
                },
            };
        }

        /// 为样式设置默认值
        fn set_default(_buffer: &Vec<u8>, _offset: usize, _world: &Setting)
        where
            Self: Sized,
        {
        }

        fn to_attr(ptr: *const u8) -> Attribute
        where
            Self: Sized,
        {
            Attribute::Transform(TransformType(clone_unaligned(ptr.cast::<TransformFuncs>())))
        }
    }

    impl ConvertToComponent for TranslateType {
        /// 将样式属性设置到组件上
        /// ptr为样式属性的指针
        /// 安全： entity必须存在
        fn set<'w, 's>(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity, is_clone: bool)
        where
            Self: Sized,
        {
            let v = ptr.cast::<[LengthUnit; 2]>();
            let v = if is_clone {
                clone_unaligned(v)
            } else {
                unsafe { v.read_unaligned() }
            };
            cur_style_mark.set(Self::get_type() as usize, true);

            log::debug!(
                "set_style_attr, type: {:?}, value: {:?}, id: {:?}",
                std::any::type_name::<Transform>(),
                v,
                entity
            );
            match query.transform_will_change.lend_mut().get_mut(entity) {
                Some(component) => {
                    // 如果存在transform_willChange,则将Transform设置在TransformWillChange上
                    component.0.translate = Some(v);
					query.transform_will_change.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                }
                None => {
                    // 不存在transform_willChange， 则设置在Transfrom上
                    match query.transform.lend_mut().get_mut(entity) {
                        Some(component) => {
                            // 如果存在transform_willChange,则将Transform设置在TransformWillChange上
                            component.all_transform.translate = Some(v);
							query.transform.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                        }
                        None => {
                            query.transform.lend_mut().insert(entity, Transform {
                                all_transform: AllTransform {
                                    translate: Some(v),
                                    ..Default::default()
                                },
                                ..Default::default()
                            });
                        }
                    }
                }
            };
        }

        /// 为样式设置默认值
        fn set_default(_buffer: &Vec<u8>, _offset: usize, _world: &Setting)
        where
            Self: Sized,
        {
        }

        fn to_attr(ptr: *const u8) -> Attribute
        where
            Self: Sized,
        {
            Attribute::Translate(TranslateType(clone_unaligned(ptr.cast::<[LengthUnit; 2]>())))
        }
    }
    impl ConvertToComponent for ResetTranslateType {
        /// 将样式属性设置到组件上
        /// ptr为样式属性的指针
        /// 安全： entity必须存在
        fn set<'w, 's>(_cur_style_mark: &mut BitArray<[u32; 3]>, _ptr: *const u8, query: &Setting, entity: Entity, _is_clone: bool)
        where
            Self: Sized,
        {
            match query.transform_will_change.lend_mut().get_mut(entity) {
                Some(component) => {
                    component.0.translate = None;
					query.transform_will_change.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                }
                None => match query.transform.lend_mut().get_mut(entity) {
                    Some(component) => {
                        component.all_transform.translate = None;
						query.transform.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                    }
                    None => (),
                },
            };
        }

        /// 为样式设置默认值
        fn set_default(_buffer: &Vec<u8>, _offset: usize, _world: &Setting)
        where
            Self: Sized,
        {
        }

        fn to_attr(ptr: *const u8) -> Attribute
        where
            Self: Sized,
        {
            Attribute::Translate(TranslateType(clone_unaligned(ptr.cast::<[LengthUnit; 2]>())))
        }
    }

    impl ConvertToComponent for ScaleType {
        /// 将样式属性设置到组件上
        /// ptr为样式属性的指针
        /// 安全： entity必须存在
        fn set<'w, 's>(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity, is_clone: bool)
        where
            Self: Sized,
        {
            let v = ptr.cast::<[f32; 2]>();
            let v = if is_clone {
                clone_unaligned(v)
            } else {
                unsafe { v.read_unaligned() }
            };
            cur_style_mark.set(Self::get_type() as usize, true);

            log::debug!(
                "set_style_attr, type: {:?}, value: {:?}, id: {:?}",
                std::any::type_name::<Transform>(),
                v,
                entity
            );
            match query.transform_will_change.lend_mut().get_mut(entity) {
                Some(component) => {
                    // 如果存在transform_willChange,则将Transform设置在TransformWillChange上
                    component.0.scale = Some(v);
					query.transform_will_change.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                }
                None => {
                    // 不存在transform_willChange， 则设置在Transfrom上
                    match query.transform.lend_mut().get_mut(entity) {
                        Some(component) => {
                            // 如果存在transform_willChange,则将Transform设置在TransformWillChange上
                            component.all_transform.scale = Some(v);
							query.transform.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                        }
                        None => {
                            query.transform.lend_mut().insert(entity, Transform {
                                all_transform: AllTransform {
                                    scale: Some(v),
                                    ..Default::default()
                                },
                                ..Default::default()
                            });
                        }
                    }
                }
            };
        }

        /// 为样式设置默认值
        fn set_default(_buffer: &Vec<u8>, _offset: usize, _world: &Setting)
        where
            Self: Sized,
        {
        }

        fn to_attr(ptr: *const u8) -> Attribute
        where
            Self: Sized,
        {
            Attribute::Scale(ScaleType(clone_unaligned(ptr.cast::<[f32; 2]>())))
        }
    }
    impl ConvertToComponent for ResetScaleType {
        /// 将样式属性设置到组件上
        /// ptr为样式属性的指针
        /// 安全： entity必须存在
        fn set<'w, 's>(_cur_style_mark: &mut BitArray<[u32; 3]>, _ptr: *const u8, query: &Setting, entity: Entity, _is_clone: bool)
        where
            Self: Sized,
        {
            match query.transform_will_change.lend_mut().get_mut(entity) {
                Some(component) => {
                    component.0.scale = None;
					query.transform_will_change.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                }
                None => match query.transform.lend_mut().get_mut(entity) {
                    Some( component) => {
                        component.all_transform.scale = None;
						query.transform.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                    }
                    None => (),
                },
            };
        }

        /// 为样式设置默认值
        fn set_default(_buffer: &Vec<u8>, _offset: usize, _world: &Setting)
        where
            Self: Sized,
        {
        }

        fn to_attr(ptr: *const u8) -> Attribute
        where
            Self: Sized,
        {
            Attribute::Scale(ScaleType(clone_unaligned(ptr.cast::<[f32; 2]>())))
        }
    }

    impl ConvertToComponent for RotateType {
        /// 将样式属性设置到组件上
        /// ptr为样式属性的指针
        /// 安全： entity必须存在
        fn set<'w, 's>(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity, is_clone: bool)
        where
            Self: Sized,
        {
            let v = ptr.cast::<f32>();
            let v = if is_clone {
                clone_unaligned(v)
            } else {
                unsafe { v.read_unaligned() }
            };
            cur_style_mark.set(Self::get_type() as usize, true);

            log::debug!(
                "set_style_attr, type: {:?}, value: {:?}, id: {:?}",
                std::any::type_name::<Transform>(),
                v,
                entity
            );
            match query.transform_will_change.lend_mut().get_mut(entity) {
                Some(component) => {
                    // 如果存在transform_willChange,则将Transform设置在TransformWillChange上
                    component.0.rotate = Some(v);
					query.transform_will_change.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                }
                None => {
                    // 不存在transform_willChange， 则设置在Transfrom上
                    match query.transform.lend_mut().get_mut(entity) {
                        Some(component) => {
                            // 如果存在transform_willChange,则将Transform设置在TransformWillChange上
							component.all_transform.rotate = Some(v);
							query.transform.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                        }
                        None => {
                            query.transform.lend_mut().insert(entity, Transform {
                                all_transform: AllTransform {
                                    rotate: Some(v),
                                    ..Default::default()
                                },
                                ..Default::default()
                            });
                        }
                    }
                }
            };
        }

        /// 为样式设置默认值
        fn set_default(_buffer: &Vec<u8>, _offset: usize, _world: &Setting)
        where
            Self: Sized,
        {
        }

        fn to_attr(ptr: *const u8) -> Attribute
        where
            Self: Sized,
        {
            Attribute::Rotate(unsafe { RotateType(ptr.cast::<f32>().read_unaligned()) })
        }
    }
    impl ConvertToComponent for ResetRotateType {
        /// 将样式属性设置到组件上
        /// ptr为样式属性的指针
        /// 安全： entity必须存在
        fn set<'w, 's>(_cur_style_mark: &mut BitArray<[u32; 3]>, _ptr: *const u8, query: &Setting, entity: Entity, _is_clone: bool)
        where
            Self: Sized,
        {
            match query.transform_will_change.lend_mut().get_mut(entity) {
                Some(component) => {
                    component.0.rotate = None;
					query.transform_will_change.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                }
                None => match query.transform.lend_mut().get_mut(entity) {
                    Some(component) => {
                        component.all_transform.rotate = None;
						query.transform.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                    }
                    None => (),
                },
            };
        }

        /// 为样式设置默认值
        fn set_default(_buffer: &Vec<u8>, _offset: usize, _world: &Setting)
        where
            Self: Sized,
        {
        }

        fn to_attr(ptr: *const u8) -> Attribute
        where
            Self: Sized,
        {
            Attribute::Rotate(unsafe { RotateType(ptr.cast::<f32>().read_unaligned()) })
        }
    }

    impl ConvertToComponent for TransformWillChangeType {
        /// 将样式属性设置到组件上
        /// ptr为样式属性的指针
        /// 安全： entity必须存在
        fn set<'w, 's>(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity, is_clone: bool)
        where
            Self: Sized,
        {
            let v = ptr.cast::<bool>();
            let v = if is_clone {
                clone_unaligned(v)
            } else {
                unsafe { v.read_unaligned() }
            };
            cur_style_mark.set(Self::get_type() as usize, true);

            log::debug!(
                "set_style_attr, type: {:?}, value: {:?}, id: {:?}",
                std::any::type_name::<TransformWillChange>(),
                v,
                entity
            );
            match query.transform_will_change.lend_mut().get_mut(entity) {
                Some(component) => {
                    // 删除TransformWillChange, 设置Transform
                    if !v {
                        let c = component.clone();
                        // 设置transform
                        match query.transform.lend_mut().get_mut(entity) {
                            Some(component) => {
                                // 如果存在transform_willChange,则将Transform设置在TransformWillChange上
                                component.all_transform = c.0;
								query.transform.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                            }
                            None => {
                                query.transform.lend_mut().insert(entity, Transform {
                                    all_transform: c.0,
                                    ..Default::default()
                                });
                            }
                        }
                        query.transform_will_change.lend_mut().delete(entity);
                    }
                }
                None => {
                    if v {
                        // 不存在transform_willChange， 则设置在Transfrom上
                        match query.transform.lend_mut().get_mut(entity) {
                            Some(component) => {
                                let c = component.clone();
                                query.transform_will_change.lend_mut().insert(entity, TransformWillChange(c.all_transform));
                            }
                            None => {
                                query.transform_will_change.lend_mut().insert(entity, TransformWillChange::default());
                            }
                        }
                    }
                }
            };
        }

        /// 为样式设置默认值
        fn set_default(_buffer: &Vec<u8>, _offset: usize, _world: &Setting)
        where
            Self: Sized,
        {
        }

        fn to_attr(ptr: *const u8) -> Attribute
        where
            Self: Sized,
        {
            Attribute::TransformWillChange(unsafe { TransformWillChangeType(ptr.cast::<bool>().read_unaligned()) })
        }
    }
    impl ConvertToComponent for ResetTransformWillChangeType {
        fn set<'w, 's>(_cur_style_mark: &mut BitArray<[u32; 3]>, _ptr: *const u8, query: &Setting, entity: Entity, _is_clone: bool)
        where
            Self: Sized,
        {
            log::debug!("reset_style_attr, type: TransformWillChange, id: {:?}", entity);
            if let Some(component) = query.transform_will_change.lend_mut().get_mut(entity) {
                // 删除TransformWillChange, 设置Transform
                let c = component.clone();
                // 设置transform
                match query.transform.lend_mut().get_mut(entity) {
                    Some(component) => {
                        // 如果存在transform_willChange,则将Transform设置在TransformWillChange上
                        component.all_transform = c.0;
						query.transform.lend_mut().get_notify_ref().modify_event(entity, "", 0);
                    }
                    None => {
                        query.transform.lend_mut().insert(entity, Transform {
                            all_transform: c.0,
                            ..Default::default()
                        });
                    }
                }
                query.transform_will_change.lend_mut().delete(entity);
            }
        }

        /// 为样式设置默认值
        fn set_default(_buffer: &Vec<u8>, _offset: usize, _world: &Setting)
        where
            Self: Sized,
        {
        }

        fn to_attr(ptr: *const u8) -> Attribute
        where
            Self: Sized,
        {
            Attribute::TransformWillChange(unsafe { TransformWillChangeType(ptr.cast::<bool>().read_unaligned()) })
        }
    }
    // impl_style!(@pack TransformWillChangeType, transform_will_change, TransformWillChange, TransformFuncs);

    // impl_style!(TransformType, transform, Transform, funcs, Transform, TransformFuncs);
    impl_style!(@pack BorderImageClipType, border_image_clip, BorderImageClip, NotNanRect);
    impl_style!(@pack BorderImageSliceType, border_image_slice, BorderImageSlice, BorderImageSlice1);
    impl_style!(@pack BorderImageRepeatType, border_image_repeat, BorderImageRepeat, ImageRepeat);

    impl_style!(@pack_send BorderColorType, border_color, BorderColor, CgColor);

    impl_style!(@pack_send BackgroundColorType, background_color, BackgroundColor, Color);

    impl_style!(@pack_send BoxShadowType, box_shadow, BoxShadow, BoxShadow1);

    impl_style!(@pack OpacityType, opacity, Opacity, f32);
    impl_style!(@pack BorderRadiusType, border_radius, BorderRadius, BorderRadius1);
    impl_style!(@pack HsiType, filter, Hsi, Hsi1);
    impl_style!(@pack BlurType, blur, Blur, f32);
    impl_style!(TransformOriginType, transform, Transform, origin, TransformOrigin, TransformOrigin);
    impl_style!(DirectionType, other_layout_style, OtherLayoutStyle, direction, Direction, Direction);
    impl_style!(AspectRatioType, other_layout_style, OtherLayoutStyle, aspect_ratio, AspectRatio, Number);
    impl_style!(OrderType, other_layout_style, OtherLayoutStyle, order, Order, isize);
    impl_style!(FlexBasisType, other_layout_style, OtherLayoutStyle, flex_basis, FlexBasis, Dimension);


    impl_style!(@func DisplayType, show, Show, set_display, get_display, Display, Display);
    impl_style!(@func VisibilityType, show, Show, set_visibility, get_visibility, Visibility, bool);
    impl_style!(@func EnableType, show, Show, set_enable, get_enable, Enable, Enable);

    impl_style!(@func1 VNodeType, node_state, NodeState, set_vnode, NodeState, VNode, bool);
    // impl_style!(@func VNodeType, node_state, set_vnode, NodeState, bool);

    impl_style!(@pack ZIndexType, z_index, ZIndex, isize);
    impl_style!(@pack OverflowType, overflow, Overflow, bool);

    impl_style!(@pack MaskImageType, mask_image, MaskImage, MaskImage1);
    impl_style!(@pack MaskImageClipType, mask_image_clip, MaskImageClip, NotNanRect);
    impl_style!(@pack ClipPathType, clip_path, ClipPath, BaseShape);

	// impl_style!(AsImageType, as_image, AsImage, level, AsImage, AsImage1);
	// impl_style!(AsImageType, as_image, AsImage, level, AsImage, AsImage1);

    impl_style!(WidthType, rect_layout_style, RectLayoutStyle, size, width, Width, Dimension);
    impl_style!(HeightType, rect_layout_style, RectLayoutStyle, size, height, Height, Dimension);

    impl_style!(MarginTopType, rect_layout_style, RectLayoutStyle, margin, top, MarginTop, Dimension);
    impl_style!(MarginRightType, rect_layout_style, RectLayoutStyle, margin, right, MarginRight, Dimension);
    impl_style!(MarginBottomType, rect_layout_style, RectLayoutStyle, margin, bottom, MarginBottom, Dimension);
    impl_style!(MarginLeftType, rect_layout_style, RectLayoutStyle, margin, left, MarginLeft, Dimension);

    impl_style!(PaddingTopType, other_layout_style, OtherLayoutStyle, padding, top, PaddingTop, Dimension);
    impl_style!(PaddingRightType, other_layout_style, OtherLayoutStyle, padding, right, PaddingRight, Dimension);
    impl_style!(PaddingBottomType, other_layout_style, OtherLayoutStyle, padding, bottom, PaddingBottom, Dimension);
    impl_style!(PaddingLeftType, other_layout_style, OtherLayoutStyle, padding, left, PaddingLeft, Dimension);

    impl_style!(BorderTopType, other_layout_style, OtherLayoutStyle, border, top, BorderTop, Dimension);
    impl_style!(BorderRightType, other_layout_style, OtherLayoutStyle, border, right, BorderRight, Dimension);
    impl_style!(BorderBottomType, other_layout_style, OtherLayoutStyle, border, bottom, BorderBottom, Dimension);
    impl_style!(BorderLeftType, other_layout_style, OtherLayoutStyle, border, left, BorderLeft, Dimension);

    impl_style!(PositionTopType, other_layout_style, OtherLayoutStyle, position, top, PositionTop, Dimension);
    impl_style!(PositionRightType, other_layout_style, OtherLayoutStyle, position, right, PositionRight, Dimension);
    impl_style!(PositionBottomType, other_layout_style, OtherLayoutStyle, position, bottom, PositionBottom, Dimension);
    impl_style!(PositionLeftType, other_layout_style, OtherLayoutStyle, position, left, PositionLeft, Dimension);

    impl_style!(MinWidthType, other_layout_style, OtherLayoutStyle, min_size, width, MinWidth, Dimension);
    impl_style!(MinHeightType, other_layout_style, OtherLayoutStyle, min_size, height, MinHeight, Dimension);
    impl_style!(MaxHeightType, other_layout_style, OtherLayoutStyle, max_size, height, MaxHeight, Dimension);
    impl_style!(MaxWidthType, other_layout_style, OtherLayoutStyle, max_size, width, MaxWidth, Dimension);
    impl_style!(
        JustifyContentType,
        other_layout_style,
        OtherLayoutStyle,
        justify_content,
        JustifyContent,
        JustifyContent
    );
    impl_style!(
        FlexDirectionType,
        other_layout_style,
        OtherLayoutStyle,
        flex_direction,
        FlexDirection,
        FlexDirection
    );
    impl_style!(AlignContentType, other_layout_style, OtherLayoutStyle, align_content, AlignContent, AlignContent);
    impl_style!(AlignItemsType, other_layout_style, OtherLayoutStyle, align_items, AlignItems, AlignItems);
    impl_style!(FlexWrapType, other_layout_style, OtherLayoutStyle, flex_wrap, FlexWrap, FlexWrap);

    impl_style!(FlexShrinkType, other_layout_style, OtherLayoutStyle, flex_shrink, FlexShrink, f32);
    impl_style!(FlexGrowType, other_layout_style, OtherLayoutStyle, flex_grow, FlexGrow, f32);
    impl_style!(PositionTypeType, other_layout_style, OtherLayoutStyle, position_type, PositionType, PositionType1);
    impl_style!(AlignSelfType, other_layout_style, OtherLayoutStyle, align_self, AlignSelf, AlignSelf);

    impl_style!(@pack BlendModeType, blend_mode, BlendMode, BlendMode1);
    // impl_style!(AnimationNameType, animation, Animation, name, AnimationName, AnimationName);
    // impl_style!(
    //     AnimationDurationType,
    //     animation,
    //     Animation,
    //     duration,
    //     AnimationDuration,
    //     SmallVec<[Time; 1]>
    // );
    // impl_style!(
    //     AnimationTimingFunctionType,
    //     animation,
    //     Animation,
    //     timing_function,
    //     AnimationTimingFunction,
    //     SmallVec<[AnimationTimingFunction; 1]>
    // );
    // impl_style!(AnimationDelayType, animation, Animation, delay, AnimationDelay, SmallVec<[Time; 1]>);
    // impl_style!(
    //     AnimationIterationCountType,
    //     animation,
    //     Animation,
    //     iteration_count,
    //     AnimationIterationCount,
    //     SmallVec<[IterationCount; 1]>
    // );
    // impl_style!(
    //     AnimationDirectionType,
    //     animation,
    //     Animation,
    //     direction,
    //     AnimationDirection,
    //     SmallVec<[AnimationDirection; 1]>
    // );
    // impl_style!(
    //     AnimationFillModeType,
    //     animation,
    //     Animation,
    //     fill_mode,
    //     AnimationFillMode,
    //     SmallVec<[AnimationFillMode; 1]>
    // );
    // impl_style!(
    //     AnimationPlayStateType,
    //     animation,
    //     Animation,
    //     play_state,
    //     AnimationPlayState,
    //     SmallVec<[AnimationPlayState; 1]>
    // );


    pub struct StyleFunc {
        get_type: fn() -> StyleType,
        // get_style_index: fn() -> u8,
        size: fn() -> usize,
        // /// 安全： entity必须存在
        // fn set(&self, cur_style_mark: &mut BitArray<[u32;3]>, buffer: &Vec<u8>, offset: usize, query: &Setting, entity: Entity);
        /// 安全： entity必须存在
        set: fn(cur_style_mark: &mut BitArray<[u32; 3]>, ptr: *const u8, query: &Setting, entity: Entity, is_clone: bool),

        /// 设置默认值
        set_default: fn(buffer: &Vec<u8>, offset: usize, world: &Setting),
        to_attr: fn(ptr: *const u8) -> Attribute,
    }

    impl StyleFunc {
        fn new<T: ConvertToComponent>() -> StyleFunc {
            StyleFunc {
                get_type: T::get_type,
                // get_style_index: T::get_style_index,
                size: T::size,
                set: T::set,
                set_default: T::set_default,
                to_attr: T::to_attr,
                // add: T::add,
                // scale: T::scale,
            }
        }
    }

    lazy_static::lazy_static! {

        static ref STYLE_ATTR: [StyleFunc; 183] = [
            StyleFunc::new::<EmptyType>(), // 0 empty 占位， 无实际作用
            StyleFunc::new::<BackgroundRepeatType>(), // 1
            StyleFunc::new::<FontStyleType>(), // 2
            StyleFunc::new::<FontWeightType>(), // 3
            StyleFunc::new::<FontSizeType>(), // 4
            StyleFunc::new::<FontFamilyType>(), // 5
            StyleFunc::new::<LetterSpacingType>(), // 6
            StyleFunc::new::<WordSpacingType>(), // 7
            StyleFunc::new::<LineHeightType>(), // 8
            StyleFunc::new::<TextIndentType>(), // 9
            StyleFunc::new::<WhiteSpaceType>(), // 10

            StyleFunc::new::<TextAlignType>(), // 11
            StyleFunc::new::<VerticalAlignType>(), // 12
            StyleFunc::new::<ColorType>(), // 13
            StyleFunc::new::<TextStrokeType>(), // 14
            StyleFunc::new::<TextShadowType>(), // 15

            StyleFunc::new::<BackgroundImageType>(), // 16
            StyleFunc::new::<BackgroundImageClipType>(), // 17
            StyleFunc::new::<ObjectFitType>(), // 18
            StyleFunc::new::<BackgroundColorType>(), // 19
            StyleFunc::new::<BoxShadowType>(), // 20
            StyleFunc::new::<BorderImageType>(), // 21
            StyleFunc::new::<BorderImageClipType>(), // 22
            StyleFunc::new::<BorderImageSliceType>(), // 23
            StyleFunc::new::<BorderImageRepeatType>(), // 24

            StyleFunc::new::<BorderColorType>(), // 25


            StyleFunc::new::<HsiType>(), // 26
            StyleFunc::new::<BlurType>(), // 27
            StyleFunc::new::<MaskImageType>(), // 28
            StyleFunc::new::<MaskImageClipType>(), // 29
            StyleFunc::new::<TransformType>(), // 30
            StyleFunc::new::<TransformOriginType>(), // 31
            StyleFunc::new::<TransformWillChangeType>(), // 32
            StyleFunc::new::<BorderRadiusType>(), // 33
            StyleFunc::new::<ZIndexType>(), // 34
            StyleFunc::new::<OverflowType>(), // 35


            StyleFunc::new::<BlendModeType>(), // 36
            StyleFunc::new::<DisplayType>(), // 37
            StyleFunc::new::<VisibilityType>(), // 38
            StyleFunc::new::<EnableType>(), // 30


            StyleFunc::new::<WidthType>(), // 40
            StyleFunc::new::<HeightType>(), // 41

            StyleFunc::new::<MarginTopType>(), // 42
            StyleFunc::new::<MarginRightType>(), // 43
            StyleFunc::new::<MarginBottomType>(), // 44
            StyleFunc::new::<MarginLeftType>(), // 45

            StyleFunc::new::<PaddingTopType>(), // 46
            StyleFunc::new::<PaddingRightType>(), // 47
            StyleFunc::new::<PaddingBottomType>(), // 48
            StyleFunc::new::<PaddingLeftType>(), // 49

            StyleFunc::new::<BorderTopType>(), // 50
            StyleFunc::new::<BorderRightType>(), // 51
            StyleFunc::new::<BorderBottomType>(), // 52
            StyleFunc::new::<BorderLeftType>(), // 53

            StyleFunc::new::<PositionTopType>(), // 54
            StyleFunc::new::<PositionRightType>(), // 55
            StyleFunc::new::<PositionBottomType>(), // 56
            StyleFunc::new::<PositionLeftType>(), // 57

            StyleFunc::new::<MinWidthType>(), // 58
            StyleFunc::new::<MinHeightType>(), // 59
            StyleFunc::new::<MaxHeightType>(), // 60
            StyleFunc::new::<MaxWidthType>(), // 61
            StyleFunc::new::<DirectionType>(), // 62
            StyleFunc::new::<FlexDirectionType>(), // 63
            StyleFunc::new::<FlexWrapType>(), // 64
            StyleFunc::new::<JustifyContentType>(), // 65
            StyleFunc::new::<AlignContentType>(), // 66
            StyleFunc::new::<AlignItemsType>(), // 67


            StyleFunc::new::<PositionTypeType>(), // 68
            StyleFunc::new::<AlignSelfType>(), // 69
            StyleFunc::new::<FlexShrinkType>(), // 70
            StyleFunc::new::<FlexGrowType>(), // 71
            StyleFunc::new::<AspectRatioType>(), // 72
            StyleFunc::new::<OrderType>(), // 73
            StyleFunc::new::<FlexBasisType>(), // 74
            StyleFunc::new::<OpacityType>(), // 75

            StyleFunc::new::<TextContentType>(), // 76

            StyleFunc::new::<VNodeType>(), // 77

            StyleFunc::new::<TransformFuncType>(), // 78

			StyleFunc::new::<EmptyType>(), // 79
            StyleFunc::new::<EmptyType>(), // 80
            StyleFunc::new::<EmptyType>(), // 81
            StyleFunc::new::<EmptyType>(), // 82
            StyleFunc::new::<EmptyType>(), // 83
            StyleFunc::new::<EmptyType>(), // 84
            StyleFunc::new::<EmptyType>(), // 85
            StyleFunc::new::<EmptyType>(), // 86

            // StyleFunc::new::<AnimationNameType>(), // 79
            // StyleFunc::new::<AnimationDurationType>(), // 80
            // StyleFunc::new::<AnimationTimingFunctionType>(), // 81
            // StyleFunc::new::<AnimationDelayType>(), // 82
            // StyleFunc::new::<AnimationIterationCountType>(), // 83
            // StyleFunc::new::<AnimationDirectionType>(), // 84
            // StyleFunc::new::<AnimationFillModeType>(), // 85
            // StyleFunc::new::<AnimationPlayStateType>(), // 86
            StyleFunc::new::<ClipPathType>(), // 87
            StyleFunc::new::<TranslateType>(), // 88
            StyleFunc::new::<ScaleType>(), // 89
            StyleFunc::new::<RotateType>(), // 90
			StyleFunc::new::<EmptyType>(), // 91
            // StyleFunc::new::<AsImageType>(), // 91

        /******************************* reset ******************************************************/
            StyleFunc::new::<ResetBackgroundRepeatType>(), // 1 text
            StyleFunc::new::<ResetFontStyleType>(), // 2
            StyleFunc::new::<ResetFontWeightType>(), // 3
            StyleFunc::new::<ResetFontSizeType>(), // 4
            StyleFunc::new::<FontFamilyType>(), // 5
            StyleFunc::new::<LetterSpacingType>(), // 6
            StyleFunc::new::<WordSpacingType>(), // 7
            StyleFunc::new::<ResetLineHeightType>(), // 8
            StyleFunc::new::<TextIndentType>(), // 9
            StyleFunc::new::<ResetWhiteSpaceType>(), // 10

            StyleFunc::new::<ResetTextAlignType>(), // 11
            StyleFunc::new::<ResetVerticalAlignType>(), // 12
            StyleFunc::new::<ResetColorType>(), // 13
            StyleFunc::new::<ResetTextStrokeType>(), // 14
            StyleFunc::new::<ResetTextShadowType>(), // 15

            StyleFunc::new::<ResetBackgroundImageType>(), // 16
            StyleFunc::new::<ResetBackgroundImageClipType>(), // 17
            StyleFunc::new::<ResetObjectFitType>(), // 18
            StyleFunc::new::<ResetBackgroundColorType>(), // 19
            StyleFunc::new::<ResetBoxShadowType>(), // 20
            StyleFunc::new::<ResetBorderImageType>(), // 21
            StyleFunc::new::<ResetBorderImageClipType>(), // 22
            StyleFunc::new::<ResetBorderImageSliceType>(), // 23
            StyleFunc::new::<ResetBorderImageRepeatType>(), // 24

            StyleFunc::new::<ResetBorderColorType>(), // 25


            StyleFunc::new::<ResetHsiType>(), // 26
            StyleFunc::new::<ResetBlurType>(), // 27
            StyleFunc::new::<ResetMaskImageType>(), // 28
            StyleFunc::new::<ResetMaskImageClipType>(), // 29
            StyleFunc::new::<ResetTransformType>(), // 30
            StyleFunc::new::<ResetTransformOriginType>(), // 31
            StyleFunc::new::<ResetTransformWillChangeType>(), // 32
            StyleFunc::new::<ResetBorderRadiusType>(), // 33
            StyleFunc::new::<ResetZIndexType>(), // 34
            StyleFunc::new::<ResetOverflowType>(), // 35


            StyleFunc::new::<ResetBlendModeType>(), // 36
            StyleFunc::new::<ResetDisplayType>(), // 37
            StyleFunc::new::<ResetVisibilityType>(), // 38
            StyleFunc::new::<ResetEnableType>(), // 39


            StyleFunc::new::<ResetWidthType>(), // 40
            StyleFunc::new::<ResetHeightType>(), // 41

            StyleFunc::new::<ResetMarginTopType>(), // 42
            StyleFunc::new::<ResetMarginRightType>(), // 43
            StyleFunc::new::<ResetMarginBottomType>(), // 44
            StyleFunc::new::<ResetMarginLeftType>(), // 45

            StyleFunc::new::<ResetPaddingTopType>(), // 46
            StyleFunc::new::<ResetPaddingRightType>(), // 47
            StyleFunc::new::<ResetPaddingBottomType>(), // 48
            StyleFunc::new::<ResetPaddingLeftType>(), // 49

            StyleFunc::new::<ResetBorderTopType>(), // 50
            StyleFunc::new::<ResetBorderRightType>(), // 51
            StyleFunc::new::<ResetBorderBottomType>(), // 52
            StyleFunc::new::<ResetBorderLeftType>(), // 53

            StyleFunc::new::<ResetPositionTopType>(), // 54
            StyleFunc::new::<ResetPositionRightType>(), // 55
            StyleFunc::new::<ResetPositionBottomType>(), // 56
            StyleFunc::new::<ResetPositionLeftType>(), // 57

            StyleFunc::new::<ResetMinWidthType>(), // 58
            StyleFunc::new::<ResetMinHeightType>(), // 59
            StyleFunc::new::<ResetMaxHeightType>(), // 60
            StyleFunc::new::<ResetMaxWidthType>(), // 61
            StyleFunc::new::<ResetDirectionType>(), // 62
            StyleFunc::new::<ResetFlexDirectionType>(), // 63
            StyleFunc::new::<ResetFlexWrapType>(), // 64
            StyleFunc::new::<ResetJustifyContentType>(), // 65
            StyleFunc::new::<ResetAlignContentType>(), // 66
            StyleFunc::new::<ResetAlignItemsType>(), // 67


            StyleFunc::new::<ResetPositionTypeType>(), // 68
            StyleFunc::new::<ResetAlignSelfType>(), // 69
            StyleFunc::new::<FlexShrinkType>(), // 70
            StyleFunc::new::<FlexGrowType>(), // 71
            StyleFunc::new::<ResetAspectRatioType>(), // 72
            StyleFunc::new::<ResetOrderType>(), // 73
            StyleFunc::new::<ResetFlexBasisType>(), // 74
            StyleFunc::new::<ResetOpacityType>(), // 75

            StyleFunc::new::<ResetTextContentType>(), // 76

            StyleFunc::new::<ResetVNodeType>(), // 77

            StyleFunc::new::<ResetTransformFuncType>(), // 78

			StyleFunc::new::<EmptyType>(), // 79
            StyleFunc::new::<EmptyType>(), // 80
            StyleFunc::new::<EmptyType>(), // 81
            StyleFunc::new::<EmptyType>(), // 82
            StyleFunc::new::<EmptyType>(), // 83
            StyleFunc::new::<EmptyType>(), // 84
            StyleFunc::new::<EmptyType>(), // 85
            StyleFunc::new::<EmptyType>(), // 86
            // StyleFunc::new::<ResetAnimationNameType>(), // 79
            // StyleFunc::new::<ResetAnimationDurationType>(), // 80
            // StyleFunc::new::<ResetAnimationTimingFunctionType>(), // 81
            // StyleFunc::new::<ResetAnimationDelayType>(), // 82
            // StyleFunc::new::<ResetAnimationIterationCountType>(), // 83
            // StyleFunc::new::<ResetAnimationDirectionType>(), // 84
            // StyleFunc::new::<ResetAnimationFillModeType>(), // 85
            // StyleFunc::new::<ResetAnimationPlayStateType>(), // 86

            StyleFunc::new::<ResetClipPathType>(), // 87
            StyleFunc::new::<ResetTranslateType>(), // 88
            StyleFunc::new::<ResetScaleType>(), // 89
            StyleFunc::new::<ResetRotateType>(), // 90
			StyleFunc::new::<EmptyType>(), // 91
            // StyleFunc::new::<ResetAsImageType>(), // 91

        ];
    }

    // pub struct Setting<'w> {
    //     pub style: &'w StyleQuery,
    //     pub world: &'w mut World,
    // }

    // impl<'w> Setting<'w> {
    //     // #[inline]
    //     // pub fn style_mut(&mut self) -> &mut StyleQuery<'w, 's> {
    //     // 	&mut self.style
    //     // }

    //     // #[inline]
    //     // pub fn world_mut(&mut self) -> &mut World {
    //     // 	&mut self.world
    //     // }

    //     pub fn new(style: &'w StyleQuery, world: &'w mut World) -> Self { Self { style, world } }
    // }

    pub struct StyleAttr;

    impl StyleAttr {
        #[inline]
        pub fn get_type(style_type: u8) -> StyleType { (STYLE_ATTR[style_type as usize].get_type)() }

        #[inline]
        pub unsafe fn write<T: Attr>(value: T, buffer: &mut Vec<u8>) {
            value.write(buffer);
            forget(value);
        }

        #[inline]
        pub fn set(
            cur_style_mark: &mut BitArray<[u32; 3]>,
            style_index: u8,
            buffer: &Vec<u8>,
            offset: usize,
            query: &Setting,
            entity: Entity,
            is_clone: bool,
        ) {
            (STYLE_ATTR[style_index as usize].set)(cur_style_mark, unsafe { buffer.as_ptr().add(offset) }, query, entity, is_clone)
        }

        #[inline]
        pub fn to_attr(style_index: u8, buffer: &Vec<u8>, offset: usize) -> Attribute {
            (STYLE_ATTR[style_index as usize].to_attr)(unsafe { buffer.as_ptr().add(offset) })
        }

        #[inline]
        pub fn size(style_index: u8) -> usize { (STYLE_ATTR[style_index as usize].size)() }

        #[inline]
        pub fn reset(cur_style_mark: &mut BitArray<[u32; 3]>, style_index: u8, buffer: &Vec<u8>, offset: usize, query: &Setting, entity: Entity) {
            (STYLE_ATTR[style_index as usize + 91].set)(cur_style_mark, unsafe { buffer.as_ptr().add(offset) }, query, entity, false);
        }

        #[inline]
        pub fn set_default(style_index: u8, buffer: &Vec<u8>, offset: usize, world: &Setting) {
            (STYLE_ATTR[style_index as usize].set_default)(buffer, offset, world);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StyleAttribute {
    Reset(u8),
    Set(Attribute),
}

pub fn style_attr_list_to_buffer(style_buffer: &mut Vec<u8>, style_list: &mut VecDeque<StyleAttribute>, mut count: usize) -> ClassMeta {
    let start = style_buffer.len();
    let mut class_meta = ClassMeta {
        start,
        end: start,
        class_style_mark: BitArray::default(),
    };

    loop {
        if count == 0 {
            break;
        }
        let r = style_list.pop_front().unwrap();
        match r {
            StyleAttribute::Reset(r) => style_buffer.push(r),
            StyleAttribute::Set(r) => style_to_buffer(style_buffer, r, &mut class_meta),
        }

        count -= 1;
    }
    class_meta.end = style_buffer.len();

    class_meta
}


/// 样式设置
pub enum ResetAttribute {
    BackgroundRepeat, // 0 empty 占位， 无实际作用
    FontStyle,        // 2
    FontWeight,       // 3
    FontSize,         // 4
    FontFamily,       // 5
    LetterSpacing,    // 6
    WordSpacing,      // 7
    LineHeight,       // 8
    TextIndent,       // 9
    WhiteSpace,       // 10

    TextAlign,     // 11
    VerticalAlign, // 12
    Color,         // 13
    TextStroke,    // 14
    TextShadow,    // 15

    BackgroundImage,     // 16
    BackgroundImageClip, // 17
    ObjectFit,           // 18
    BackgroundColor,     // 19
    BoxShadow,           // 20
    BorderImage,         // 21
    BorderImageClip,     // 22
    BorderImageSlice,    // 23
    BorderImageRepeat,   // 24

    BorderColor, // 25

    Hsi,                 // 26
    Blur,                // 27
    MaskImage,           // 28
    MaskImageClip,       // 29
    Transform,           // 31
    TransformOrigin,     // 32
    TransformWillChange, // 33
    BorderRadius,        // 34
    ZIndex,              // 35
    Overflow,            // 36

    BlendMode,  // 37
    Display,    // 38
    Visibility, // 39
    Enable,     // 40

    Width,  // 41
    Height, // 42

    MarginTop,    // 43
    MarginRight,  // 44
    MarginBottom, // 45
    MarginLeft,   // 46

    PaddingTop,    // 47
    PaddingRight,  // 48
    PaddingBottom, // 49
    PaddingLeft,   // 50

    BorderTop,    // 51
    BorderRight,  // 52
    BorderBottom, // 53
    BorderLeft,   // 54

    PositionTop,    // 55
    PositionRight,  // 56
    PositionBottom, // 57
    PositionLeft,   // 58

    MinWidth,       // 59
    MinHeight,      // 60
    MaxHeight,      // 61
    MaxWidth,       // 62
    Direction,      // 63
    FlexDirection,  // 64
    FlexWrap,       // 65
    JustifyContent, // 66
    AlignContent,   // 67
    AlignItems,     // 68

    PositionType, // 69
    AlignSelf,    // 70
    FlexShrink,   // 71
    FlexGrow,     // 72
    AspectRatio,  // 73
    Order,        // 74
    FlexBasis,    // 75
    Opacity,      // 80

    TextContent, // 81

    VNode, // 82

    TransformFunc, // 83

    AnimationName,           // 79
    AnimationDuration,       // 80
    AnimationTimingFunction, // 81
    AnimationDelay,          // 82
    AnimationIterationCount, // 83
    AnimationDirection,      // 84
    AnimationFillMode,       // 85
    AnimationPlayState,      // 86
    ClipPath,                // 87
    Translate,               // 88
    Scale,                   // 89
    Rotate,                  // 90
    AsImage,                 // 91
}

// clone指针指向的对象（可能未对齐）
fn clone_unaligned<T: Clone>(src: *const T) -> T {
	let r = unsafe {read_unaligned(src)};
	let ret = r.clone();
	forget(r); // 这里忘记r， 是因为read_unaligned对src进行逐位读取，如果不忘记r， src指向的对象会被释放、而此函数仅仅是想拷贝src
	ret
}