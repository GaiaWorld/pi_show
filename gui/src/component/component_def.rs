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
use std::default::Default;
use std::mem::uninitialized;
use std::ptr::write;
use std::ops::{Deref, DerefMut};

use cg::color::{Color};

// use fast_deque::FastDeque;
use wcs::component::{Event, ComponentGroup, Point, ComponentGroupTree, notify};
use wcs::world::{ID, ComponentMgr};
use layout::{YgNode};

use component::math::*;

out_component!(struct Aabb3{
    min: Point3,
    max: Point3,
});

out_component!(struct Vector2{
    x: f32,
    y: f32,
});

out_component!(struct Matrix4{
    x: Vector4,
    y: Vector4,
    z: Vector4,
    w: Vector4,
});

#[derive(Debug, Clone, Copy, Component)]
pub struct Size {
    pub width: f32,
    pub height: f32,
    pub dirty: bool,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Transform {
    pub scale: Vector3,
    pub position: Vector3,
    pub rotation: Quaternion,
    pub rot_ref: Vector3,
    pub dirty: bool,
}

#[derive(Component, Default, Debug, Clone)]
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

#[allow(unused_attributes)]
#[derive(Debug, Component)]
pub struct Node{
    #[Component]
    #[Must]
    pub transform: TransformPoint, // 几何变换组件
    #[Component]
    #[Must]
    pub size: SizePoint, // 几何变换组件
    #[Component]
    #[Must]
    pub world_matrix: Matrix4Point, //世界矩阵组件
    pub world_matrix_dirty: bool, //暂时将world_matrix的脏标志设置在node中
    pub bound_box: usize, //包围盒组件
    pub bound_box_dirty: bool, //暂时将bound_box的脏标志设置在node中
    pub object: Object,
    pub yoga_node: YgNode,
    pub layer: usize,
    #[Ignore]
    pub childs: Vec<NodePoint>,
}

impl NodePoint {
    pub fn create_child<M: ComponentMgr>(&mut self, index: usize, group: &mut NodeGroup<M>) -> NodePoint {
        let node_point = NodePoint::create(group, &self);
        let n_yoga = node_point.get_yoga_node(group).clone_node();
        self.get_yoga_node_mut(group).insert_child(n_yoga, index);
        node_point.set_layer(self.get_layer(group).clone(), group);
        // group._group.notify(EventType::ModifyField(self.clone(), "childs"));
        node_point
    }

    pub fn remove_child<M: ComponentMgr>(&mut self, point: &NodePoint, group: &mut NodeGroup<M>) -> Node {
        let yoga_node = point.get_yoga_node(group).clone_node();
        self.get_yoga_node_mut(group).remove_child_unfree(yoga_node);
        point.clone().set_layer(0, group);
        let node = group._group.remove(point);
        //事件处理 TODO
        node.owner
    }
}

impl<'a, M: ComponentMgr> NodeWriteRef<'a, M> {
    pub fn create_child(&mut self, index: usize) -> NodeWriteRef<M> {
        NodeWriteRef{
            point: self.point.create_child(index, unsafe{&mut *(self.groups as *mut NodeGroup<M>)}),
            groups: self.groups.clone(),
            mgr: &mut self.mgr
        }
    }

    pub fn remove_child(&mut self, point: &NodePoint) -> Node {
        self.point.remove_child(point, unsafe{&mut *(self.groups as *mut NodeGroup<M>)})
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

#[derive(Debug, Clone, Default)]
pub struct LinearGradient{
    pub x1: f32,
    pub x2: f32,
    pub x3: f32,
    pub x4: f32,
    pub colors: Vec<PointColor>
}


#[derive(Debug, Clone, Copy, Default)]
pub struct PointColor{
    pub offset: f32,
    pub color: Color<usize>
}

#[derive(Debug, Clone, Default)]
pub struct RadialGradient{
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub fx: f32,
    pub fy: f32,
    pub colors: Vec<PointColor>
}

#[derive(Debug, Clone, Copy)]
pub enum Display {
    None,
    Flex,
}

impl Default for Display{
    fn default() -> Display{
        Display::None
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Component, Debug, Clone, Copy)]
pub struct Line{
    pub start: Point2,
    pub end: Point2,
}

impl Default for Line{
    fn default() -> Line {
        Line {
            start: Point2::default(),
            end: Point2::default()
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Rect{
    pub left_top: Point2,
    pub width: f32,
    pub height: f32,
}


impl Default for Rect{
    fn default() -> Rect {
        Rect {
            left_top: Point2::default(),
            width: 1.0,
            height: 1.0
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Circle{
    pub center: Point2,
    pub radius: f32,
}

impl Default for Circle{
    fn default() -> Circle {
        Circle {
            center: Point2::default(),
            radius: 1.0,
        }
    }
}

#[derive(EnumComponent, Debug, Clone)]
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

world!(
    struct GuiComponentMgr{
        node: Node
    }
);