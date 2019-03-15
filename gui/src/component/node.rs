use std::ops::{Deref};

// use web_sys::*;

use deque::deque::{Deque, Node as DeNode};
use slab::{Slab};
use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify, Builder};
use wcs::world::{ComponentMgr};

use layout::{YgNode};
use component::math::*;
use component::style::element::{ElementId, Element, ElementReadRef, ElementWriteRef, ElementGroup, ElementBuilder};
use component::style::transform::*;
use component::style::border::*;
use component::style::style::*;

#[allow(unused_attributes)]
#[derive(Debug, Component, Builder)]
pub struct Node{
    //由外部设置的部分
    #[builder(export)]
    #[component(Style)]
    pub style: usize,

    #[enum_component(Element)]
    #[builder(build(Builder), export)]
    pub element: ElementId,

    #[component(Transform)]
    #[builder(build(Default), export)]
    pub transform: usize, // 几何变换组件

    pub childs: Deque<usize, Slab<DeNode<usize>>>,

    pub yoga: Option<YgNode>,

    //以下数据由system设置
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

    #[ignore]
    pub inherit_value: f32, //继承自父节点的不透明度
}

pub trait QidContainer {
    fn get_qid_container(&mut self) -> &mut Slab<DeNode<usize>>;
}


impl<'a, M: ComponentMgr + QidContainer> NodeWriteRef<'a, M> {
    pub fn insert_child(&mut self, mut child: Node, ty: InsertType) -> NodeWriteRef<M> {
        let group = NodeGroup::<M>::from_usize_mut(self.groups);
        let (handler, parent) = {
            let elem = group._group.get(self.id);
            match elem.element {
                ElementId::Text(_) | ElementId::Image(_) => panic!("leaf node can't insert child, id:{}", self.id),
                _ => (),
            };
            child.layer = elem.layer + 1; //设置layer
            (group._group.get_handlers(), elem.parent)
        };
        let (qid, child_id) = match ty {
            InsertType::Back => {
                let display = group.style._group.get_mut(child.style).display;
                match display {
                    Some(_d) => {
                        let yoga = YgNode::default();
                        {
                            let node = group._group.get_mut(self.id);
                            let index = node.childs.len();
                            node.yoga.as_ref().unwrap().insert_child(yoga.clone(), index);
                        }
                        child.yoga = Some(yoga);
                    },
                    None => ()
                }
                let child_id = group._group.insert(child, self.id); // 将节点插入容器
                let node = group._group.get_mut(self.id);
                (node.childs.push_back(child_id, &mut self.mgr.get_qid_container()), child_id)
            },
            InsertType::Front => {
                match group.style._group.get_mut(child.style).display {
                    Some(_d) => {
                        let yoga = YgNode::default();
                        {
                            let node = group._group.get_mut(self.id);
                            node.yoga.as_ref().unwrap().insert_child(yoga.clone(), 0);
                        }
                        child.yoga = Some(yoga);
                    },
                    None => ()
                }
                let child_id = group._group.insert(child, self.id); // 将节点插入容器
                let node = group._group.get_mut(self.id);
                (node.childs.push_front(child_id, &mut self.mgr.get_qid_container()), child_id)
            },
            InsertType::ToBack(yoga_index, brother_pid) => {
                match group.style._group.get_mut(child.style).display {
                    Some(_d) => {
                        let yoga = YgNode::default();
                        {
                            let node = group._group.get_mut(self.id);
                            node.yoga.as_ref().unwrap().insert_child(yoga.clone(), yoga_index);
                        }
                        child.yoga = Some(yoga);
                    },
                    None => ()
                }
                let child_id = group._group.insert(child, self.id); // 将节点插入容器
                let node = group._group.get_mut(self.id);
                (unsafe{node.childs.push_to_back(child_id, brother_pid, &mut self.mgr.get_qid_container())}, child_id)
            },
            InsertType::ToFront(yoga_index, brother_pid) => {
                match group.style._group.get_mut(child.style).display {
                    Some(_d) => {
                        let yoga = YgNode::default();
                        {
                            let node = group._group.get_mut(self.id);
                            node.yoga.as_ref().unwrap().insert_child(yoga.clone(), yoga_index);
                        }
                        child.yoga = Some(yoga);
                    },
                    None => ()
                }
                let child_id = group._group.insert(child, self.id); // 将节点插入容器
                let node = group._group.get_mut(self.id);
                (unsafe{node.childs.push_to_front(child_id, brother_pid, &mut self.mgr.get_qid_container())}, child_id)
            },
        };
        group._group.get_mut(child_id).qid = qid; //不会发出qid改变的监听， 应该通知？

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
            let child_yoga = match group._group.get(child_id).yoga.as_ref() {
                Some(child_yoga) => Some((*child_yoga).clone()),
                None => None,
            };
            match child_yoga {
                Some(child_yoga) => {
                    let node = group._group.get_mut(self.id);
                    node.yoga.as_mut().unwrap().remove_child_unfree(child_yoga);
                    node.parent
                },
                None => {
                    let node = group._group.get_mut(self.id);
                    node.parent
                },
            }
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