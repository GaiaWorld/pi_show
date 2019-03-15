use std::ops::{Deref};

use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify, Builder};
use wcs::world::{ComponentMgr};

use layout::*;
use component::style::generic::StyleUnit;

//     //最大宽度和最小宽度
//     max_width: usize,
//     max_height:usize,
//     min_width: usize,
//     min_height: usize,

//     //笔触路径宽度
//     stroke_width: usize, 
//     //笔触路径颜色
//     stroke_color: cg::color::Color<f32>,

//     //布局属性
//     display: Display,
//     flex_direction: FlexDirection,
//     justify_content: JustifyContent,
//     align_items: AlignItems,
//     align_content: AlignContent,
//     flex_grow: usize, //放大比例  默认是0   当有放大空间的时候，值越大，放大的比例越大
//     flex_shrink: usize, //缩小比例  默认是1  值越大，缩小的时候比例越小
//     flex_basis: usize, //属性定义了在分配多余空间之前，项目占据的主轴空间（main size）。浏览器根据这个属性，计算主轴是否有多余空间。它的默认值为auto，即项目的本来大小
//     margin_left: usize,
//     margin_top: usize,
//     margin_right: usize,
//     margin_bottom: usize,
//     padding_left: usize,
//     padding_top: usize,
//     padding_right: usize,
//     padding_bottom: usize,
//     align_self: AlignSelf,

//     position_type: PositionType,
//     position: Vector2<usize>,

//flex container 的设置
#[derive(Debug, Component, Default)]
pub struct FlexContainer {
    align_content: Option<AlignContent>,
    align_items: Option<AlignItems>,
    justify_ontent: Option<JustifyContent>,
    // aspect_ratio:f32,
    flex_direction: Option<FlexDirection>,
    flex_wrap: Option<FlexWrap>,
}

//flex item 的设置
#[derive(Debug, Component, Default)]
pub struct FlexItem{
    flex_grow: Option<f32>,
    flex_shrink: Option<f32>,
    flex_basis: Option<StyleUnit>,
    align_self: Option<AlignSelf>,
}

#[derive(Debug, Component, Default)]
pub struct Boundary{
    bottom: Option<StyleUnit>,
    left: Option<StyleUnit>,
    right: Option<StyleUnit>,
    top: Option<StyleUnit>,
}

#[derive(Debug, Component, Default)]
pub struct Rect {
    width: Option<StyleUnit>,
    height: Option<StyleUnit>,
}


#[derive(Debug, Component, Default)]
pub struct MinMax {
    max_height: Option<StyleUnit>,
    max_width: Option<StyleUnit>,
    min_height: Option<StyleUnit>,
    min_hidth: Option<StyleUnit>,
}

#[derive(Debug, Component, Default)]
pub struct Position {
    ty: Option<PositionType>,
    bottom: Option<StyleUnit>,
    left: Option<StyleUnit>,
    right: Option<StyleUnit>,
    top: Option<StyleUnit>,
}

#[allow(unused_attributes)]
#[derive(Debug, Component, Builder)]
pub struct Layout{
    #[component(FlexContainer)]
    pub container: usize,

    #[component(FlexItem)]
    pub item: usize,

    #[component(Boundary)]
    pub paddind: usize,

    #[component(Boundary)]
    pub margin: usize,

    #[component(Boundary)]
    pub border: usize,

    #[component(Boundary)]
    pub padding: usize,

    #[component(Rect)]
    pub wh: usize,

    #[component(Position)]
    pub position: usize,
}
