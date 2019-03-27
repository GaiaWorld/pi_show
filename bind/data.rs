use wasm_bindgen::prelude::*;

//定义横轴方向， 当主轴为横轴是， 会与FlexDirection的值相会影响
#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Inherit,
    LTR,
    RTL,
}

//主轴
#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum FlexDirection {
    Column, //主轴为垂直方向，起点在上沿。(默认)
    ColumnReverse,//主轴为垂直方向，起点在下沿。
    Row,//主轴为水平方向，起点在左端
    RowReverse,//主轴为水平方向，起点在右端。
}

//flex-wrap属性定义，如果一条轴线排不下，如何换行
#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum FlexWrap {
    NoWrap, //不换行
    Wrap, //下一行在下方
    WrapReverse, //下一行在上方
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum JustifyContent {
    Start, //主轴方向起点对齐
    Center, //主轴方向居中对齐对齐
    End, //主轴方向终点对齐
    SpaceBetween, // 两端对齐，项目之间的间隔都相等
    SpaceAround, // 每个项目两侧的间隔相等。所以，项目之间的间隔比项目与边框的间隔大一倍
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum AlignItems {
    Start, //交叉轴方向起点对齐
    Center, //交叉轴方向居中对齐
    End, //交叉轴方向终点对齐
    BaseLine, // 项目的第一行文字的基线对齐
    Stretch, // 如果项目未设置高度或设为auto，将占满整个容器的高度
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum AlignContent {
    Start, //与交叉轴的起点对齐
    Center, // 与交叉轴的中点对齐
    End, // 与交叉轴的终点对齐
    SpaceBetween, // 与交叉轴两端对齐，轴线之间的间隔平均分布
    SpaceAround, // 每根轴线两侧的间隔都相等。所以，轴线之间的间隔比轴线与边框的间隔大一倍
    Stretch, // 轴线占满整个交叉轴
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum AlignSelf {
    Auto,
    Start,
    Center,
    End,
    BaseLine,
    Stretch,
}

//定位类型
#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum PositionType {
    Relative,
    Absolute,
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum Display {
    Flex,
    Inline,
    None
}


#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub struct StyleUnit{
    pub ty: Option<StyleUnitType>,
    pub value : f32,
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum StyleUnitType{
    Auto,
    Length,
    Percent
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub struct LengthPercent{
    pub ty: Option<LengthPercentType>,
    pub value : f32,
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum LengthPercentType{
    Length,
    Percent
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum VerticalAlign{
    Center,
    Top,
    Bootom
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum TextDirection{
    Left,
    Right,
    Top,
    Bootom
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct LineHeight{
    pub ty: Option<LineHeightType>,
    pub value: f32,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum LineHeightType{
    Normal, //设置合理的行间距（等于font-size）
    Length, //固定像素
    Number, //设置数字，此数字会与当前的字体尺寸相乘来设置行间距。
    Percent,   //	基于当前字体尺寸的百分比行间距.
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum TextAlign{
    Left,	//把文本排列到左边。默认值：由浏览器决定。
    Right,	//把文本排列到右边。
    Center,	//把文本排列到中间。
    Justify,	//实现两端对齐文本效果。
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum WhiteSpace{
    Normal, //	默认。空白会被浏览器忽略(其实是所有的空白被合并成一个空格), 超出范围会换行。
    Nowrap, //	空白会被浏览器忽略(其实是所有的空白被合并成一个空格), 超出范围文本也不会换行，文本会在在同一行上继续，直到遇到 <br> 标签为止。
    PreWrap, //	保留所有空白符序列，超出范围会换行。
    Pre, //	保留空白符，超出范围不会换行
    PreLine, //	合并空白符序列，如果存在换行符，优先保留换行符。
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum FontStyle{
    Normal, //	默认值。标准的字体样式。
    Ttalic, //	斜体的字体样式。
    Oblique, //	倾斜的字体样式。
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum FontWeight{
    Normal, //	默认值。定义标准的字符。
    Bold, // 定义粗体字符。
    Bolder, //	定义更粗的字符。
    Lighter, //	定义更细的字符。
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine, //400 等同于 normal，而 700 等同于 bold。
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct FontSize {
    pub ty: FontSizeType,
    pub value: f32,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum FontSizeType {
    Medium,
    XXSmall,    //把字体的尺寸设置为不同的尺寸，从 xx-small 到 xx-large。
    XSmall,
    Small,
    Large,
    XLarge,
    XXLarge,
    Smaller,	//把 font-size 设置为比父元素更小的尺寸。
    Larger,	//把 font-size 设置为比父元素更大的尺寸。
    Length,	//把 font-size 设置为一个固定的值。
    Percent, //把 font-size 设置为基于父元素的一个百分比值。
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum ColorType{
    RGB,
    RGBA,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Color{
    pub ty: Option<ColorType>,
    value: Vec<u8>,
}

#[wasm_bindgen]
impl Color {
    pub fn set_value(&mut self, value: &[u8]) {
        self.value = Vec::from(value);
    }
} 

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ClipPath{
    pub ty: Option<ClipPathType>,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum ClipPathType {
    MarginBox,
    BorderBox,
    PaddingBox,
    ContentBox,
}