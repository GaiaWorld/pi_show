//样式
// pub struct Style{
//     image: String,
//     //填充颜色
//     fill_color: Color<usize>,

//     //最大宽度和最小宽度
//     max_width: usize,
//     max_height:usize,
//     min_width: usize,
//     min_height: usize,

//     //笔触路径宽度
//     stroke_width: usize, 
//     //笔触路径颜色
//     stroke_color: Color<usize>,

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
// }



//字体属性
// Text{
//     text: String,
//     font_family: String,
//     font_size: usize,
//     font_tyle: FontStyle,
//     font_weight: usize,
//     color: Color<usize>,
//     spacing: usize, //	设置字符间距 
//     line_height: usize, //	设置行高
//     text_align: TextAlign,	//对齐元素中的文本
//     text_indent: bool, //缩进元素中文本的首行
//     text_shadow: String,
//     vertical_align: VerticalAlign, //设置元素的垂直对齐
//     direction: TextDirection
// }

use std::sync::Arc;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

use cg::aabb::Aabb2;
use cg::color::Color;

use wcs::component::{Component, EventType, ComponentGroup, Point, ComponentGroupTree};
use wcs::world::{ID, ComponentMgr};

pub enum Display {
    None,
    Flex,
}

pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse
}

pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse
}

pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround
}

pub enum AlignItems {
    Start,
    Center,
    End,
    BaseLine,
    Stretch,
}

pub enum AlignContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    Stretch
}

//align-self属性允许单个项目有与其他项目不一样的对齐方式，可覆盖align-items属性。默认值为auto，表示继承父元素的align-items属性，如果没有父元素，则等同于stretch
pub enum AlignSelf {
    Auto,
    Start,
    Center,
    End,
    BaseLine,
    Stretch,
}

pub enum PositionType {
    Relative{left: u32, right: u32, top:u32, bootom: u32},
    Absolut{left: u32, right: u32, top:u32, bootom: u32},
}

pub enum TextAlign{
    Center,
    Left,
    Right
}

pub enum VerticalAlign{
    Center,
    Top,
    Bootom
}

pub enum TextDirection{
    Left,
    Right,
    Top,
    Bootom
}

pub enum FontStyle{
    Normal,
    Italic,
    Oblique,
}

pub enum NodeType{
    Text, //文字
    Line, //线
    Rect, //矩形
    Circle, //圆形
    Ellipse, // 椭圆
    Polyline, //折线
    Polygon //多边形
}

world!(
    GuiComponentMgr{

        #[derive(Default)]
        Layout{
            left: usize,
            right: usize,
            top: usize,
            bottom: usize,
        }

        #[root=node]
        #[derive(Default)]
        Node{
            #[component]
            layout: Layout,
            //node_type: NodeType,
            // z_index: usize,
            // z: usize,
            layer: usize,
            // visibility: bool,
            layout_dirt: bool,
            childs: Vec<NodePoint>,
            //renderer: Renderer,
        }
    }, ()
);

// impl Node{
//     // pub fn set_image(&mut self, value: Color<usize>){
        
//     // }
//     pub fn set_fill_color(&mut self, value: Color<usize>, groups: NodeGroups){

//     }

//     pub fn set_stroke_width(&mut self, value: usize, groups: NodeGroups){

//     }

//     pub fn set_stroke_color(&mut self, value: Color<usize>, groups: NodeGroups){
        
//     }


//     //这里开始是布局属性
//     pub fn set_display(&mut self, value: Display, groups: NodeGroups){

//     }

//     pub fn set_flex_direction(&mut self, value: FlexDirection, groups: NodeGroups){

//     }

//     pub fn set_justify_content(&mut self, value: JustifyContent, groups: NodeGroups){

//     }

//     pub fn set_align_items(&mut self, value: AlignItems, groups: NodeGroups){

//     }

//     pub fn set_align_content(&mut self, value: AlignContent, groups: NodeGroups){

//     }

//     pub fn set_align_self(&mut self, value: AlignSelf, groups: NodeGroups){

//     }

//     pub fn set_flex_grow(&mut self, value: usize, groups: NodeGroups){

//     }

//     pub fn set_flex_shrink(&mut self, value: usize, groups: NodeGroups){

//     }

//     pub fn set_flex_basis(&mut self, value: usize, groups: NodeGroups){

//     }

//     pub fn set_margin_left(&mut self, value: usize, groups: NodeGroups){

//     }

//     pub fn set_margin_right(&mut self, value: usize, groups: NodeGroups){

//     }

//     pub fn set_margin_top(&mut self, value: usize, groups: NodeGroups){

//     }

//     pub fn set_margin_bottom(&mut self, value: usize, groups: NodeGroups){

//     }

//     pub fn set_padding_left(&mut self, value: usize, groups: NodeGroups){

//     }

//     pub fn set_padding_right(&mut self, value: usize, groups: NodeGroups){

//     }

//     pub fn set_padding_top(&mut self, value: usize, groups: NodeGroups){

//     }

//     pub fn set_padding_bottom(&mut self, value: usize, groups: NodeGroups){

//     }

//     pub fn set_max_width(&mut self, value: usize, groups: NodeGroups){
        
//     }
//     pub fn set_max_height(&mut self, value: usize, groups: NodeGroups){
        
//     }
//     pub fn set_min_width(&mut self, value: usize, groups: NodeGroups){
        
//     }
//     pub fn set_min_height(&mut self, value: usize, groups: NodeGroups){
        
//     }

//     pub fn set_position_type(&mut self, value: PositionType, groups: NodeGroups){

//     }

//     pub fn set_position(&mut self, value: Vector2<usize>, groups: NodeGroups){

//     }

//     //这里开始是文字的属性
//     pub fn set_text(&mut self, value: String, groups: NodeGroups){

//     }

//     pub fn set_font_family(&mut self, value: String, groups: NodeGroups){

//     }

//     pub fn set_font_size(&mut self, value: String, groups: NodeGroups){

//     }

//     pub fn set_font_tyle(&mut self, value: String, groups: NodeGroups){

//     }

//     pub fn set_font_weight(&mut self, value: String, groups: NodeGroups){

//     }

//     pub fn set_spacing(&mut self, value: usize, groups: NodeGroups){

//     }

//     pub fn set_line_height(&mut self, line_height: usize, groups: NodeGroups){

//     }

//     pub fn set_text_align(&mut self, text_align: TextAlign, groups: NodeGroups){

//     }

//     pub fn set_text_indent(&mut self, text_indent: bool, groups: NodeGroups){

//     }

//     pub fn set_text_shadow(&mut self, text_shadow: String, groups: NodeGroups){

//     }

//     pub fn set_vertical_align(&mut self, vertical_align: VerticalAlign, groups: NodeGroups){

//     }

// }

pub struct Texture{
    pub image: Arc<Vec<u8>>,
    pub offset: (usize, usize),
    pub repeat: (usize, usize)
}

pub struct Material{
    pub vs: String,
    pub fs: String,
    pub texture: Texture,
    pub color: Color<usize>,
    pub opacity: f32,
	pub depth_test: bool,
	pub depth_write: bool,
}

pub struct SDFMaterial{
    pub vs: String,
    pub fs: String,
    pub texture: Texture,
    pub color: Color<usize>,
    pub opacity: f32,
	pub depth_test: bool,
	pub depth_write: bool,
    pub stroke_width: bool,
    pub stroke_color: Color<usize>,
    pub underlay_width: bool,
    pub underlay_color: Color<usize>,
}

pub struct BoundBox{
    pub dirt: bool,
    pub aabb: Aabb2<usize>
}

pub struct Renderer{
    //geometry: Layout,
    pub material: Material, 
    pub bound_box: BoundBox,
}

pub enum MaterialColor{
    Monochrome(Color<usize>),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
}

pub struct LinearGradient{
    pub x1: f32,
    pub x2: f32,
    pub x3: f32,
    pub x4: f32,
    pub colors: Vec<PointColor>
}


pub struct PointColor{
    pub offset: f32,
    pub color: Color<usize>
}

pub struct RadialGradient{
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub fx: f32,
    pub fy: f32,
    pub colors: Vec<PointColor>
}