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