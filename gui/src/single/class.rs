/**
 * 定义全局Class
*/

use layout::*;
use atom::Atom;

use hash::XHashMap;

use component::user::*;

// 显示样式， 不包含布局
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Class {
	// 将style属性按照内存占用大小划分为三种枚举
    pub attrs1: Vec<Attribute1>,
    pub attrs2: Vec<Attribute2>,
    pub attrs3: Vec<Attribute3>,

    pub class_style_mark: usize, // 标记class中的有效属性
    pub class_style_mark1: usize, // 标记class中布局的有效属性
}

// 全局Class表
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClassSheet {
    pub class_map: XHashMap<usize, Class>,
}

// 最小尺寸的style属性
#[derive(Debug, Serialize, Deserialize)]
pub enum Attribute1 {
    PositionType(YGPositionType),
    FlexWrap(YGWrap),
    FlexDirection(YGFlexDirection),
    AlignContent(YGAlign),
    AlignItems(YGAlign),
    AlignSelf(YGAlign),
    JustifyContent(YGJustify),

    ObjectFit(ObjectFit),
    TextAlign(TextAlign),
    VerticalAlign(VerticalAlign),
    WhiteSpace(WhiteSpace),
    FontStyle(FontStyle),
    Enable(EnableType),
    Display(Display),
    Visibility(bool),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Attribute2 {
    LetterSpacing(f32),
    LineHeight(LineHeight),
    TextIndent(f32),
    WordSpacing(f32),
    FontWeight(f32),
    FontSize(FontSize),
    FontFamily(Atom),
    ZIndex(isize),
    Opacity(Opacity),
    BorderImageRepeat(BorderImageRepeat),

    ImageUrl(Atom),
    BorderImageUrl(Atom),

    FlexShrink(f32),
    FlexGrow(f32),
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
    PositionLeft(ValueUnit),
    PositionTop(ValueUnit),
    PositionRight(ValueUnit),
    PositionBottom(ValueUnit),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Attribute3 {
    BGColor(BackgroundColor),
    BorderColor(BorderColor),
    BoxShadow(BoxShadow),

    ImageClip(ImageClip),

    BorderImageClip(BorderImageClip),
    BorderImageSlice(BorderImageSlice),

    Color(Color),
    TextShadow(TextShadow),
    TextStroke(Stroke),

    BorderRadius(BorderRadius),
    TransformFunc(Vec<TransformFunc>),
    TransformOrigin(TransformOrigin),
    Filter(Filter),
}

#[derive(Debug, Clone, EnumDefault, Serialize, Deserialize)]
pub enum ValueUnit {
    Undefined,
    Auto,
    Pixel(f32),
    Percent(f32),
}