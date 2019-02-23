//样式
// pub struct Style{
//     image: String,
//     //填充颜色
//     fill_color: cg::color::Color<f32>,

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
// }

// use std::sync::Arc;
use std::default::Default;
use std::ops::{Deref};

use web_sys::*;

use deque::deque::{Deque, Node as DeNode};
use slab::{Slab};
use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify, Builder};
use wcs::world::{ComponentMgr};

use render::vector_sdf::VectorSdf;
use layout::{YgNode, YgNodeP};
use component::math::*;

#[allow(unused_attributes)]
#[derive(Debug, Component, Builder)]
pub struct Node{
    //由外部设置的部分
    #[Builder(Export)]
    pub yoga_node: YgNode,
    #[EnumComponent(Element)]
    #[Builder(Build(Builder), Export)]
    pub element: ElementId,
    #[Component(Transform)]
    #[Builder(Build(Default), Export)]
    pub transform: usize, // 几何变换组件
    pub childs: Deque<usize, Slab<DeNode<usize>>>,

    //以下数据由system设置
    //布局数据
    #[Component(Vector3)]
    #[Builder(Build(Default))]
    pub position: usize, //位置（包括border）
    #[Component(RectSize)]
    #[Builder(Build(Default))]
    pub extent: usize, //最大宽高（包括border）
    #[Component(Border)]
    #[Builder(Build(Default))]
    pub border: usize, //边框

    #[Component(Matrix4)]
    #[Builder(Build(Default))]
    pub world_matrix: usize, //世界矩阵组件
    pub world_matrix_dirty: bool, //暂时将world_matrix的脏标志设置在node中
    pub bound_box_id: usize, //包围盒组件在八叉树中的id
    #[Component(Aabb3)]
    #[Builder(Build(Default))]
    pub bound_box: usize, //包围盒组件
    pub bound_box_dirty: bool, //暂时将bound_box的脏标志设置在node中

    #[ignore]
    pub layer: usize,
    #[ignore]
    pub qid: usize, //在父节点中的id，即在父节点的子容器中的key， 如果没有父节点， 该值为0

    // pub background:
    // pub texts: Vec<>
}

#[allow(unused_attributes)]
#[derive(Debug, Clone, Copy, Component, Builder, Default)]
pub struct Transform {
    #[Builder(Export)]
    pub scale: Scale,
    #[Builder(Export)]
    pub position: Vector3,
    #[Builder(Export)]
    pub rotation: Vector3,
    #[ignore]
    pub dirty: bool,
    #[ignore]
    pub quaternion: Quaternion,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct RectSize {
    pub width: f32,
    pub height: f32,
}

impl Default for RectSize {
    fn default () -> RectSize{
        RectSize{
            width: 1.0,
            height: 1.0
        }
    }
}


#[derive(Debug, Component, Default)]
pub struct Border{
    pub value: f32, //暂时只支持统一的border， 可能会分解为left， top， right， bootom
}

#[allow(unused_attributes)]
#[derive(Debug, EnumComponent, Builder)]
pub enum Element {
    Rect(#[Builder(Build(Builder))]RectElem),
    Circle(CircleElem),
    Text(Text)
}



#[allow(unused_attributes)]
#[derive(Component, Debug, Clone, Builder)]
pub struct Rect{
    #[Builder(Export)]
    pub left_top: Point2,
    #[Builder(Export)]
    pub width: f32,
    #[Builder(Export)]
    pub height: f32,
    #[Builder(Export)]
    pub radius: f32,
}

impl Default for Rect {
    fn default() -> Rect{
        Rect{
            left_top: Point2::default(),
            width: 1.0,
            height: 1.0,
            radius: 0.0
        }
    }
}

#[allow(unused_attributes)]
#[derive(Component, Debug, Clone, Builder, Default)]
pub struct Circle{
    #[Builder(Export)]
    pub center: Point2,
    #[Builder(Export)]
    pub radius: f32,
}

#[allow(unused_attributes)]
#[derive(Debug, Component, Builder)]
pub struct RectElem {
    #[Component(Rect)]
    #[Builder(Build(Default), Export)]
    pub shape: usize,
    #[Component(SdfStyle)]
    #[Builder(Build(Default), Export)]
    pub style: usize
}

#[allow(unused_attributes)]
#[derive(Debug, Component, Builder)]
pub struct CircleElem {
    #[Component(Circle)]
    #[Builder(Build(Default), Export)]
    pub shape: usize,
    #[Component(SdfStyle)]
    #[Builder(Build(Default), Export)]
    pub style: usize
}

// 矢量图形的style
#[allow(unused_attributes)]
#[derive(Debug, Component, Default, Builder)]
pub struct SdfStyle{
    #[Builder(Export)]
    pub color: cg::color::Color<f32>,

    // TODO 阴影， 颜色渐变
}

pub enum InsertType{
    Back,
    Front,
    ToBack(usize, usize),
    ToFront(usize, usize),
}

impl<'a> NodeWriteRef<'a, GuiComponentMgr> {
    pub fn insert_child(&mut self, mut child: Node, ty: InsertType) -> NodeWriteRef<GuiComponentMgr> {
        let group = NodeGroup::<GuiComponentMgr>::from_usize_mut(self.groups);
        let (handler, parent) = {
            let elem = group._group.get(self.id);
            child.layer = elem.layer + 1; //设置layer
            (group._group.get_handlers(), elem.parent)
        };
        let child_yoga: YgNodeP = child.yoga_node.clone();
        let child_id = group._group.insert(child, self.id); // 将节点插入容器
        let qid = match ty {
            InsertType::Back => {
                let node = group._group.get_mut(self.id);
                let index = node.childs.len();
                node.yoga_node.insert_child(child_yoga, index);
                console::log_2(&("insert_child".into()), &(index.to_string().into()));
                node.childs.push_back(child_id, &mut self.mgr.node_container)
            },
            InsertType::Front => {
                let node = group._group.get_mut(self.id);
                node.yoga_node.insert_child(child_yoga, 0);
                node.childs.push_front(child_id, &mut self.mgr.node_container)
            },
            InsertType::ToBack(yoga_index, brother_pid) => {
                let node = group._group.get_mut(self.id);
                node.yoga_node.insert_child(child_yoga, yoga_index);
                unsafe{node.childs.push_to_back(child_id, brother_pid, &mut self.mgr.node_container)}
            },
            InsertType::ToFront(yoga_index, brother_pid) => {
                let node = group._group.get_mut(self.id);
                node.yoga_node.insert_child(child_yoga, yoga_index);
                unsafe{node.childs.push_to_front(child_id, brother_pid, &mut self.mgr.node_container)}
            },
        };
        // console::log_2(&("mmmmmmmmmmmm".into()), &(group.transform._group.get_mut(group._group.get_mut(child_id).transform).parent.to_string().into()));
        group._group.get_mut(child_id).qid = qid; //不会发出qid改变的监听， 应该通知？

        {
            let xx = group._group.get_mut(child_id).element.clone();
            match xx {
                ElementId::Rect(v) => {
                    let xx = group.element.rect._group.get_mut(v).shape;
                    console::log_2(&("xxxxxxxxxxxxxxxxxxxx".into()), &(group.element.rect.shape._group.get_mut(xx).parent.to_string().into()));
                },
                ElementId::Circle(v) => {
                    {
                    let xx = group.element.circle._group.get_mut(v).shape;
                    console::log_2(&("xxxxxxxxxxxxxxxxxxxx".into()), &(group.element.circle.shape._group.get_mut(xx).parent.to_string().into()));
                    }
                    {
                    let xx = group.element.circle._group.get_mut(v).parent;
                    console::log_2(&("yyyyyyyyyyyyyyyyyyyyyyyyyyyy".into()), &(xx.to_string().into()));
                    }
                },
                _ => (),
            }
        }

        notify(Event::ModifyField{id: self.id, parent: parent, field: "childs"}, &handler.borrow(), &mut self.mgr);  //通知childs字段改变
        let mut child_ref = NodeWriteRef::new(child_id, self.groups, self.mgr);
        child_ref.set_parent(self.id);
        // console::log_2(&("rrrrrrrrrrrrrrrr".into()), &(group.transform._group.get_mut(group._group.get_mut(child_id).transform).parent.to_string().into()));
        child_ref.create_notify(); //通知Node及Node子组件的创建
        {
            let xx = group._group.get_mut(child_id).element.clone();
            match xx {
                ElementId::Rect(v) => {
                    let xx = group.element.rect._group.get_mut(v).shape;
                    console::log_2(&("xxxxxxxxxxxxxxxxxxxx".into()), &(group.element.rect.shape._group.get_mut(xx).parent.to_string().into()));
                },
                ElementId::Circle(v) => {
                    let xx = group.element.circle._group.get_mut(v).shape;
                    console::log_2(&("xxxxxxxxxxxxxxxxxxxx".into()), &(group.element.circle.shape._group.get_mut(xx).parent.to_string().into()));
                },
                _ => (),
            }
        }
        // console::log_2(&("xxxxxxxxxxxxxxxxxxxx".into()), &(group.transform._group.get_mut(group._group.get_mut(child_id).transform).parent.to_string().into()));
        child_ref
    }

    pub fn remove_child(&mut self, qid: usize) {
        if !self.mgr.node_container.contains(qid){
            panic!("remove_child fail!, node is not exist, qid:{}", qid);
        }
        let group = NodeGroup::<GuiComponentMgr>::from_usize_mut(self.groups);
        let child_id = group._group.get_mut(self.id).childs.remove(qid, &mut self.mgr.node_container); //从childs移除child
        let child_yoga = group._group.get(child_id).yoga_node.clone();
        let parent = {
            let node = group._group.get_mut(self.id);
            node.yoga_node.remove_child_unfree(child_yoga);
            node.parent
        };
        let handler = group._group.get_handlers();
        notify(Event::ModifyField{id: self.id, parent: parent, field: "childs"}, &handler.borrow(), &mut self.mgr); //通知childs字段改变
        NodeWriteRef::new(child_id, self.groups, self.mgr).destroy(); //从容器中删除child的数据， 并抛出Node及Node子组件销毁的事件
    }
}

// #[derive(Default)]
// pub struct Texture{
//     pub image: Arc<Vec<u8>>,
//     pub offset: (usize, usize),
//     pub repeat: (usize, usize)
// }

// #[derive(Default)]
// pub struct Material{
//     pub vs: String,
//     pub fs: String,
//     pub texture: Texture,
//     pub color: cg::color::Color<f32>,
//     pub opacity: f32,
// 	pub depth_test: bool,
// 	pub depth_write: bool,
// }

// #[derive(Default)]
// pub struct SDFMaterial{
//     pub vs: String,
//     pub fs: String,
//     pub texture: Texture,
//     pub color: cg::color::Color<f32>,
//     pub opacity: f32,
// 	pub depth_test: bool,
// 	pub depth_write: bool,
//     pub stroke_width: bool,
//     pub stroke_color: cg::color::Color<f32>,
//     pub underlay_width: bool,
//     pub underlay_color: cg::color::Color<f32>,
// }

// // pub struct BoundBox{
// //     pub dirt: bool,
// //     pub aabb: Aabb2<usize>
// // }

// // pub struct Renderer{
// //     //geometry: Layout,
// //     pub material: Material, 
// //     pub bound_box: BoundBox,
// // }

// // pub enum MaterialColor{
// //     Monochrome(cg::color::Color<f32>),
// //     LinearGradient(LinearGradient),
// //     RadialGradient(RadialGradient),
// // }

// #[derive(Debug, Clone, Default)]
// pub struct LinearGradient{
//     pub x1: f32,
//     pub x2: f32,
//     pub x3: f32,
//     pub x4: f32,
//     pub colors: Vec<PointColor>
// }

// #[derive(Debug, Clone, Default)]
// pub struct PointColor{
//     pub offset: f32,
//     pub color: cg::color::Color<f32>
// }

// #[derive(Debug, Clone, Default)]
// pub struct RadialGradient{
//     pub cx: f32,
//     pub cy: f32,
//     pub r: f32,
//     pub fx: f32,
//     pub fy: f32,
//     pub colors: Vec<PointColor>
// }

// #[derive(Debug, Clone, Copy)]
// pub enum Display {
//     None,
//     Flex,
// }

// impl Default for Display{
//     fn default() -> Display{
//         Display::None
//     }
// }


// #[derive(Component, Debug, Clone, Copy)]
// pub struct Line{
//     pub start: cg::Point2<f32>,
//     pub end: cg::Point2<f32>,
// }

// impl Default for Line{
//     fn default() -> Line {
//         Line {
//             start: cg::Point2::new(0.0,0.0),
//             end: cg::Point2::new(0.0,0.0)
//         }
//     }
// }

// // #[derive(Component, Debug, Clone, Default)]
// // pub struct SdfStyle {
// //     pub color: cg::color::Color<f32>,
// //     pub stroke_color: cg::color::Color<f32>,
// //     pub stroke_size: usize,
// //     pub shadow_size: usize,
// //     pub shadow_color: cg::color::Color<f32>,
// // }

// #[derive(EnumComponent, Debug, Clone)]
// pub enum Shape{
//     Text(Text),
//     Line(Line), //线
//     Rect(Rect), //矩形
//     Circle(Circle), //圆形
//     // Ellipse, // 椭圆
//     // Polyline, //折线
//     // Polygon //多边形
// }

// // #[derive(EnumComponent, Debug, Clone)]
// // pub enum Style{
// //     Vector(SdfStyle),
// //     Text(SdfStyle), //线
// //     // Circle(Circle), //圆形
// //     // Image(ImageStyle), //矩形
// //     // Ellipse, // 椭圆
// //     // Polyline, //折线
// //     // Polygon //多边形
// // }

// impl Default for Shape{
//     fn default() -> Shape{
//         Shape::Rect(Rect::default())
//     }
// }

#[allow(unused_attributes)]
#[derive(Component, Default, Debug, Clone, Builder)]
pub struct Text{
    #[Builder(Export)]
    pub text: String,
    #[Builder(Export)]
    pub font_family: String,
    #[Builder(Export)]
    pub font_size: usize,
    #[Builder(Export)]
    pub font_tyle: FontStyle,
    #[Builder(Export)]
    pub font_weight: usize,
    #[Builder(Export)]
    pub color: cg::color::Color<f32>,
    #[Builder(Export)]
    pub spacing: usize, //	设置字符间距 
    #[Builder(Export)]
    pub line_height: usize, //	设置行高
    #[Builder(Export)]
    pub text_align: TextAlign,	//对齐元素中的文本
    #[Builder(Export)]
    pub text_indent: bool, //缩进元素中文本的首行
    #[Builder(Export)]
    pub text_shadow: String,
    #[Builder(Export)]
    pub vertical_align: VerticalAlign, //设置元素的垂直对齐
    #[Builder(Export)]
    pub direction: TextDirection,
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum TextAlign{
    Center,
    Left,
    Right
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum VerticalAlign{
    Center,
    Top,
    Bootom
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum TextDirection{
    Left,
    Right,
    Top,
    Bootom
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum FontStyle{
    Normal,
    Italic,
    Oblique,
}


#[derive(Component, Debug, Clone)]
pub struct RenderSize{
    size: (f32, f32),
}

world!(
    struct GuiComponentMgr{
        #[Component]
        node: Node,
        node_container: Slab<DeNode<usize>>,
        opaque_vector: VectorSdf,    //不透明渲染对象列表
        #[Component]
        world_view: RenderSize,
        // transparent_vector: VectorSdf,    //透明的矢量图形
    } 
);