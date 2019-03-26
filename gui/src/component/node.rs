use std::ops::{Deref};
use std::os::raw::{c_void};
// use web_sys::*;

use deque::deque::{Deque, Node as DeNode};
use slab::{Slab};
use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify, Builder};
use wcs::world::{ComponentMgr};
use atom::Atom;

use layout::{YgNode};
use component::math::*;
use component::style::element::{ElementId, Element, ElementReadRef, ElementWriteRef, ElementGroup};
use component::style::border::*;
use component::style::transform::*;
use component::style::generic::*;

#[allow(unused_attributes)]
#[derive(Debug, Component, Builder, Default)]
pub struct Node{
    //由外部设置的部分
    // #[builder(export)]
    // #[component(Style)]
    // pub style: usize,

    #[builder(export)]
    pub display: Option<Display>,

    #[builder(export)]
    pub class_name: Vec<Atom>,

    // #[builder(export)]
    // #[component(Layout)]
    // pub layout: usize,

    #[builder(build(Default), export)]
    #[component(LayoutChange)]
    pub layout_change: usize,

    #[builder(export)]
    #[component(Transform)]
    pub transform: usize,

     #[builder(export)]
    #[component(ClipPath)]
    pub clip: usize,

    #[builder(export)]
    #[component(Overflow)]
    pub overflow: usize,

    #[builder(export)]
    #[component(Opacity)]
    pub opacity: usize,

    #[enum_component(Element)]
    #[builder(export)]
    pub element: ElementId,

    pub childs: Deque<usize, Slab<DeNode<usize>>>,

    pub yoga: YgNode,

    #[component(ZIndex)]
    #[builder(build(Default))]
	pub zindex: usize, //zindex组件

    //以下数据由system设置
    //记录所有子节点及递归包含的子节点的数量
	pub count: usize,

    // 子节点设zindex时，将不是auto的父节点设脏
	pub z_dirty: bool,
	pub z_index: isize,

    //布局数据
    #[component(Vector3)]
    #[builder(build(Default))]
    pub position: usize, //位置（包括border）
    
    #[component(RectSize)]
    #[builder(build(Default))]
    pub extent: usize, //最大宽高（包括border）

    #[component(Border)]
    #[builder(build(Default))]
    pub border: usize, //边框

    #[builder(build(Default))]
    #[component(Opacity)]
    pub real_opacity: usize, //不透明度


    #[component(Matrix4)]
    #[builder(build(Default))]
    pub world_matrix: usize, //世界矩阵组件

    pub world_matrix_dirty: bool, //暂时将world_matrix的脏标志设置在node中
    pub bound_box_id: usize, //包围盒组件在八叉树中的id
    
    #[component(Aabb3)]
    #[builder(build(Default))]
    pub bound_box: usize, //包围盒组件

    pub bound_box_dirty: bool, //暂时将bound_box的脏标志设置在node中

    #[ignore]
    pub layer: usize,

    #[ignore]
    pub qid: usize, //在父节点中的id，即在父节点的子容器中的key， 如果没有父节点， 该值为0
}

pub trait QidContainer {
    fn get_qid_container(&mut self) -> &mut Slab<DeNode<usize>>;
}


impl<'a, M: ComponentMgr + QidContainer> NodeWriteRef<'a, M> {
    pub fn insert_child(&mut self, child: Node, ty: InsertType) -> NodeWriteRef<M> {
        let group = NodeGroup::<M>::from_usize_mut(self.groups);
        let child_id = group._group.insert(child, 0);
        self.insert_child_with_id(child_id, ty)
    }

    pub fn insert_child_with_id(&mut self, child_id: usize, ty: InsertType) -> NodeWriteRef<M> {
        let group = NodeGroup::<M>::from_usize_mut(self.groups);
        let (element, parent, handler) = {
            let node = group._group.get(self.id);
            (node.element.clone(), node.parent, group._group.get_handlers())
        };
        match element {
            ElementId::None => {
                let yoga = group._group.get(child_id).yoga;
                let yoga_context = Box::into_raw(Box::new(YogaContex {
                    node_id: child_id,
                    mgr: self.mgr as *const M as usize,
                })) as usize;
                yoga.set_context(yoga_context as *mut c_void);
                let qid = match ty {
                    InsertType::Back => {
                        let node = group._group.get_mut(self.id);
                        let index = node.childs.len();
                        node.yoga.insert_child(yoga.clone(), index as u32);
                        node.childs.push_back(child_id, &mut self.mgr.get_qid_container())
                    },
                    InsertType::Front => {
                        let node = group._group.get_mut(self.id);
                        node.yoga.insert_child(yoga.clone(), 0);
                        node.childs.push_front(child_id, &mut self.mgr.get_qid_container())
                    },
                    InsertType::ToBack(yoga_index, brother_pid) => {
                        let node = group._group.get_mut(self.id);
                        node.yoga.insert_child(yoga.clone(), yoga_index as u32);
                        unsafe{node.childs.push_to_back(child_id, brother_pid, &mut self.mgr.get_qid_container())}

                    },
                    InsertType::ToFront(yoga_index, brother_pid) => {
                        let node = group._group.get_mut(self.id);
                        node.yoga.insert_child(yoga.clone(), yoga_index as u32);
                        unsafe{node.childs.push_to_front(child_id, brother_pid, &mut self.mgr.get_qid_container())}
                    },
                };
                let child = group._group.get_mut(child_id);
                child.yoga = yoga;
                child.qid = qid; //不会发出qid改变的监听， 应该通知？
            },
            _ => panic!(format!("insert_child error, this is a leaf node")),
        };

        notify(Event::ModifyField{id: self.id, parent: parent, field: "childs"}, &handler.borrow(), &mut self.mgr);  //通知childs字段改变
        let mut child_ref = NodeWriteRef::new(child_id, self.groups, self.mgr);
        child_ref.set_parent(self.id);
        child_ref.create_notify(); //通知Node及Node子组件的创建
        child_ref
    }

    pub fn remove_child(&mut self, qid: usize) {
        if !self.mgr.get_qid_container().contains(qid){
            panic!("remove_child fail!, node is not exist, qid:{}", qid);
        }
        let group = NodeGroup::<M>::from_usize_mut(self.groups);
        let child_id = group._group.get_mut(self.id).childs.remove(qid, &mut self.mgr.get_qid_container()); //从childs移除child
        let parent = {
            let child_yoga = group._group.get(child_id).yoga;
            let node = group._group.get_mut(self.id);
            node.yoga.remove_child(child_yoga);
            node.parent
            // let child_yoga = match group._group.get(child_id).yoga.as_ref() {
            //     Some(child_yoga) => Some((*child_yoga).clone()),
            //     None => None,
            // };
            // match child_yoga {
            //     Some(child_yoga) => {
            //         let node = group._group.get_mut(self.id);
            //         node.yoga.as_mut().unwrap().remove_child_unfree(child_yoga);
            //         node.parent
            //     },
            //     None => {
            //         let node = group._group.get_mut(self.id);
            //         node.parent
            //     },
            // }
        };
        let handler = group._group.get_handlers();
        notify(Event::ModifyField{id: self.id, parent: parent, field: "childs"}, &handler.borrow(), &mut self.mgr); //通知childs字段改变
        NodeWriteRef::new(child_id, self.groups, self.mgr).destroy(); //从容器中删除child的数据， 并抛出Node及Node子组件销毁的事件
    }
}

pub enum InsertType{
    Back,
    Front,
    ToBack(usize, usize),
    ToFront(usize, usize),
}

#[derive(Debug, Component, Default)]
pub struct RectSize{
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct ZIndex {
    pub zindex: isize, // -1表示auto, 设置负zindex全部额外-1, 默认为0 
    pub pre_min_z: f32, // 预设置的节点的最小z值 // 下面4个值需要单独独立出来吗？ TODO
    pub pre_max_z: f32, // 预设置的节点的最大z值
    pub min_z: f32, // 节点的最小z值，也是节点自身的z值
    pub max_z: f32, // 节点的最大z值，z-index == -1, 则和min_z一样。
}

#[derive(Debug, Component, Default)]
pub struct LayoutChange{
    pub value: bool,
}