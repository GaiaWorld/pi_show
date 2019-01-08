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


use std::sync::Arc;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::default::Default;

use cg::color::{Color};
// use cg::{Vector2 as OVector2};

use wcs::component::{EventType, ComponentGroup, Point, ComponentGroupTree};
use wcs::world::{ID, ComponentMgr};
use yoga::{YgNode};

world!(
    struct GuiComponentMgr{
        node: Node
    }
);

// type Vector2 = OVector2<f32>;

// out_component!(struct Vector2{
//     x: f32,
//     y:f32,
// });
#[derive(Component, Default)]
pub struct Vector2{
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Default)]
pub struct Text{
    pub text: String,
    pub font_family: String,
    pub font_size: usize,
    pub font_tyle: FontStyle,
    pub font_weight: usize,
    pub color: Color<usize>,
    pub spacing: usize, //	设置字符间距 
    pub line_height: usize, //	设置行高
    pub text_align: TextAlign,	//对齐元素中的文本
    pub text_indent: bool, //缩进元素中文本的首行
    pub text_shadow: String,
    pub vertical_align: VerticalAlign, //设置元素的垂直对齐
    pub direction: TextDirection,
}

#[derive(Component, Default)]
pub struct Node{
    pub yoga_node: YgNode,
    pub position: Vector2,
    pub size: Vector2,
    pub object: Object,
    pub bound_box: RectPoint,
    pub childs: Vec<NodePoint>,
}

impl NodePoint {
    pub fn add_child<M: ComponentMgr>(&mut self, node: Node, index: usize, group: &mut NodeGroup<M>) -> NodePoint {
        let node_p = group._group.insert(node, self.0);
        let n_yoga = node_p.get_yoga_node(group).clone_node();
        self.get_yoga_node_mut(group).insert_child(n_yoga, index);
        //发出事件
        node_p
    }

    pub fn remove_child<M: ComponentMgr>(&mut self, point: &NodePoint, group: &mut NodeGroup<M>) -> Node {
        let yoga_node = point.get_yoga_node(group).clone_node();
        self.get_yoga_node_mut(group).remove_child_unfree(yoga_node);
        group._group.remove(point)
    }
}

impl<M: ComponentMgr> NodeRef<M> {
    pub fn add_child(&mut self, node: Node, index: usize) -> NodeRef<M> {

        NodeRef{
            point: self.point.add_child(node, index, &mut self.groups.borrow_mut()),
            groups: self.groups.clone()
        }
    }

    pub fn remove_child(&mut self, point: &NodePoint) -> Node {
        self.point.remove_child(point, &mut self.groups.borrow_mut())
    }
}

#[derive(Default)]
pub struct Texture{
    pub image: Arc<Vec<u8>>,
    pub offset: (usize, usize),
    pub repeat: (usize, usize)
}

#[derive(Default)]
pub struct Material{
    pub vs: String,
    pub fs: String,
    pub texture: Texture,
    pub color: Color<usize>,
    pub opacity: f32,
	pub depth_test: bool,
	pub depth_write: bool,
}

#[derive(Default)]
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

// pub struct BoundBox{
//     pub dirt: bool,
//     pub aabb: Aabb2<usize>
// }

// pub struct Renderer{
//     //geometry: Layout,
//     pub material: Material, 
//     pub bound_box: BoundBox,
// }

// pub enum MaterialColor{
//     Monochrome(Color<usize>),
//     LinearGradient(LinearGradient),
//     RadialGradient(RadialGradient),
// }

#[derive(Default)]
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

#[derive(Default)]
pub struct RadialGradient{
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub fx: f32,
    pub fy: f32,
    pub colors: Vec<PointColor>
}

pub enum Display {
    None,
    Flex,
}

impl Default for Display{
    fn default() -> Display{
        Display::None
    }
}

pub enum TextAlign{
    Center,
    Left,
    Right
}

impl Default for TextAlign{
    fn default() -> TextAlign{
        TextAlign::Center
    }
}

pub enum VerticalAlign{
    Center,
    Top,
    Bootom
}

impl Default for VerticalAlign{
    fn default() -> VerticalAlign{
        VerticalAlign::Center
    }
}

pub enum TextDirection{
    Left,
    Right,
    Top,
    Bootom
}

impl Default for TextDirection{
    fn default() -> TextDirection{
        TextDirection::Left
    }
}

pub enum FontStyle{
    Normal,
    Italic,
    Oblique,
}

impl Default for FontStyle{
    fn default() -> FontStyle{
        FontStyle::Normal
    }
}

#[derive(Component, Default)]
pub struct Line{
    pub start: Vector2,
    pub end: Vector2,
}

#[derive(Component, Default)]
pub struct Rect{
    pub left_top: Vector2,
    pub width: f32,
    pub height: f32,
}

#[derive(Component, Default)]
pub struct Circle{
    pub center: Vector2,
    pub radius: f32,
}

#[derive(EnumComponent)]
pub enum Object{
    Text(Text), //文字
    Line(Line), //线
    Rect(Rect), //矩形
    Circle(Circle), //圆形
    // Ellipse, // 椭圆
    // Polyline, //折线
    // Polygon //多边形
}

impl Default for Object{
    fn default() -> Object{
        Object::Rect(Rect::default())
    }
}

