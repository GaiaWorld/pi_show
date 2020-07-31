use atom::Atom;
/**
 * 定义全局Class
*/
use flex_layout::*;

use hash::XHashMap;

use component::user::*;

// 显示样式， 不包含布局
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Class {
    // 将style属性按照内存占用大小划分为三种枚举
    pub attrs1: Vec<Attribute1>,
    pub attrs2: Vec<Attribute2>,
    pub attrs3: Vec<Attribute3>,

    pub class_style_mark: usize,  // 标记class中的有效属性
    pub class_style_mark1: usize, // 标记class中布局的有效属性
}

// 全局Class表
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClassSheet {
    pub class_map: XHashMap<usize, Class>,
}

impl ClassSheet {
    pub fn mem_size(&self) -> usize {
        let mut r = 0;
        for (_, v) in self.class_map.iter() {
            r += v.attrs1.capacity() * std::mem::size_of::<Attribute1>();
            r += v.attrs2.capacity() * std::mem::size_of::<Attribute2>();
            r += v.attrs3.capacity() * std::mem::size_of::<Attribute3>();
        }
        r
    }
}

// 最小尺寸的style属性
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Attribute1 {
    PositionType(PositionType),
    FlexWrap(FlexWrap),
    FlexDirection(FlexDirection),
    AlignContent(AlignContent),
    AlignItems(AlignItems),
    AlignSelf(AlignSelf),
    JustifyContent(JustifyContent),

    ObjectFit(ObjectFit),
    TextAlign(TextAlign),
    VerticalAlign(VerticalAlign),
    WhiteSpace(WhiteSpace),
    FontStyle(FontStyle),
    Enable(EnableType),
    Display(Display),
    Visibility(bool),
    Overflow(bool),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    Width(Dimension),
    Height(Dimension),
    MarginLeft(Dimension),
    MarginTop(Dimension),
    MarginBottom(Dimension),
    MarginRight(Dimension),
    Margin(Dimension),
    PaddingLeft(Dimension),
    PaddingTop(Dimension),
    PaddingBottom(Dimension),
    PaddingRight(Dimension),
    Padding(Dimension),
    BorderLeft(Dimension),
    BorderTop(Dimension),
    BorderBottom(Dimension),
    BorderRight(Dimension),
    Border(Dimension),
    MinWidth(Dimension),
    MinHeight(Dimension),
    MaxHeight(Dimension),
    MaxWidth(Dimension),
    FlexBasis(Dimension),
    PositionLeft(Dimension),
    PositionTop(Dimension),
    PositionRight(Dimension),
    PositionBottom(Dimension),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

// #[derive(Debug, Clone, EnumDefault, Serialize, Deserialize)]
// pub enum Dimension {
//     Undefined,
//     Auto,
//     Pixel(f32),
//     Percent(f32),
// }
