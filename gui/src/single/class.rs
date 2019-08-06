

use layout::*;
use slab::Slab;
use atom::Atom;

use fx_hashmap::FxHashMap32;

use component::user::*;

#[derive(Default)]
pub struct ClassSheet {
    pub transform: Slab<Transform>,
    pub layout_common: Slab<LayoutCommonUse>,
    pub layout_box_pattern: Slab<LayoutBoxPattern>,
    pub layout_adaption: Slab<LayoutAdaption>,

    pub background_color: Slab<BackgroundColorClass>,
    pub border_color: Slab<BorderColor>,
    pub image: Slab<ImageClass>,
    pub border_image: Slab<BorderImageClass>,
    pub text: Slab<TextClass>,
    pub box_shadow: Slab<BoxShadow>,

    pub class: Slab<Class>, 
    pub class_map: FxHashMap32<usize, usize>, // key对应应用层的class名称（class名称必须是一个数字），value对应class slab 的偏移量
}

#[derive(Debug, Clone, EnumDefault)]
pub enum ValueUnit {
    Undefined,
    Auto,
    Pixel(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Default)]
pub struct LayoutCommonUse {
    pub width: ValueUnit,
    pub height: ValueUnit,
    pub position_type: YGPositionType,
    pub margin_left: ValueUnit,
    pub margin_right: ValueUnit,
    pub margin_top: ValueUnit,
    pub margin_bottom: ValueUnit,
    pub flex_wrap: YGWrap,
    pub flex_direction: YGFlexDirection,

    pub align_content: YGAlign,
    pub align_items: YGAlign,
    pub justify_content: YGJustify,
    pub align_self: YGAlign,
}

#[derive(Debug, Clone, Default)]
pub struct LayoutAdaption {
    pub min_width: ValueUnit,
    pub min_height: ValueUnit,
    pub max_width: ValueUnit,
    pub max_height: ValueUnit,

    pub flex_basis: ValueUnit,
    pub flex_shrink: ValueUnit,
    pub flex_grow: ValueUnit,
}

#[derive(Debug, Clone, Default)]
pub struct LayoutBoxPattern {
    pub padding_left: ValueUnit,
    pub padding_right: ValueUnit,
    pub padding_top: ValueUnit,
    pub padding_bottom: ValueUnit,

    pub border_left: ValueUnit,
    pub border_right: ValueUnit,
    pub border_top: ValueUnit,
    pub border_bottom: ValueUnit,
}

#[derive(Debug, Clone)]
pub enum LayoutAttr {
    Width(ValueUnit),
    Height(ValueUnit),
    MarginLeft(ValueUnit),
    MarginTop(ValueUnit),
    MarginBottom(ValueUnit),
    MarginRight(ValueUnit),
    Margin(ValueUnit),
    PaddingLeft(ValueUnit),
    PaddingTop(ValueUnit),
    PaddingBottom(ValueUnit),
    PaddingRight(ValueUnit),
    Padding(ValueUnit),
    BorderLeft(ValueUnit),
    BorderTop(ValueUnit),
    BorderBottom(ValueUnit),
    BorderRight(ValueUnit),
    Border(ValueUnit),
    MinWidth(ValueUnit),
    MinHeight(ValueUnit),
    MaxHeight(ValueUnit),
    MaxWidth(ValueUnit),
    FlexBasis(ValueUnit),
    FlexShrink(f32),
    FlexGrow(f32),
    PositionType(YGPositionType),
    FlexWrap(YGWrap),
    FlexDirection(YGFlexDirection),
    AlignContent(YGAlign),
    AlignItems(YGAlign),
    AlignSelf(YGAlign),
    JustifyContent(YGJustify),
}

// 显示样式， 不包含布局
#[derive(Debug, Clone, Default)]
pub struct Class {
    // 节点属性
    pub z_index: usize,
    pub enable: EnableType,
    pub display: Display,
    pub visibility: bool,
    pub border_radius: BorderRadius,
    pub opacity: f32,
    pub transform: Transform,
    pub filter: Filter,

    // // 布局属性
    // pub layout_common: usize, // 常用布局属性
    // pub layout_box: usize,
    // pub layout_adaption: usize,
    pub layout: Vec<LayoutAttr>,

    // 显示属性
    pub background_color: usize,
    pub border_color: usize,
    pub image: usize,
    pub border_image: usize,
    pub text: usize,
    pub box_shadow: usize,

    pub class_style_mark: usize, // 标记class中的有效属性
    pub class_layout_mark: u32, // 标记class中布局的有效属性
}

// pub enum DirtyType1 {
//     BGColor = 1,
//     BorderColor = 2,
//     ImageSrc = 4,
//     ImageClip = 8,
//     ImageFitType = 0x10,
//     BorderImageSrc = 0x20,
//     BorderImageClip = 0x40,
//     BorderImageSlice= 0x80,
//     BorderImageRepeat = 0x100,
//     TextLetterSpacing = 0x200,
//     TextWordSpacing = 0x400,
//     TextLineHeight = 0x800,
//     TextIndent = 0x1000,
//     TextWhiteSpace = 0x2000,
//     TextColor = 0x4000,
//     TextStroke = 0x8000,
//     TextShadowH = 0x10000,
//     TextShadowV = 0x20000,
//     TextShadowColor = 0x40000,
//     TextShadowBlur = 0x80000,
//     TextAlign = 0x100000,
//     VerticalAlign = 0x200000,
//     FontStyle = 0x400000,
//     FontSize = 0x800000,
//     FontWeight = 0x1000000,
//     FontFamily = 0x2000000,
//     ZIndex = 0x4000000,
//     Enable = 0x8000000,
//     Display = 0x10000000,
//     Visibility = 0x20000000,
//     BorderRadius = 0x40000000,
//     Opacity = 0x80000000,
//     Transform = 0x100000000,
//     // Filter = 0x200000000,
// }

pub struct ImageClass {
    pub image: Atom,
    pub obj_fit: FitType,
    pub image_clip: ImageClip,
}

pub struct BorderImageClass {
    pub border_image: Atom,
    pub border_image_slice: BorderImageSlice,
    pub border_image_clip: BorderImageClip,
    pub border_image_repeat: BorderImageRepeat,
}

#[derive(Debug, Clone, Default)]
pub struct TextClass {
    pub style: TextStyle,
    pub font: Font,
    pub shadow: TextShadow,
}

pub type BackgroundColorClass = BackgroundColor;
pub type BorderColorClass = BorderColor;