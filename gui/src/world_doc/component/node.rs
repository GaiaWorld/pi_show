use std::ops::{Deref};
use std::os::raw::{c_void};
// use web_sys::*;

use deque::deque::{Deque, Node as DeNode};
use slab::{Slab};
use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Builder, Handlers};
use wcs::world::{ComponentMgr};
use atom::Atom;

use layout::{YgNode, Layout};
use component::math::*;
use world_doc::component::style::element::{ElementId, Element, ElementReadRef, ElementWriteRef, ElementGroup};
use world_doc::component::style::transform::*;
use world_doc::component::style::generic::*;

#[allow(unused_attributes)]
#[derive(Debug, Component, Builder, Default)]
pub struct Node{
    //由外部设置的部分
    // #[builder(export)]
    // #[component(Style)]
    // pub style: usize,

    #[listen]
    #[builder(export)]
    pub display: Display,

    #[listen]
    #[builder(export, build(value=true))]
    pub visibility: bool,

    #[builder(export)]
    pub class_name: Vec<Atom>,

    #[builder(export)]
    #[component(Decorate)]
    pub decorate: usize,

    #[builder(export)]
    #[component(Transform)]
    pub transform: usize,

    #[builder(export)]
    #[component(ClipPath)]
    pub clip: usize,

    #[listen]
    pub overflow: bool,

    #[builder(export, build(value=1.0) )]
    #[listen]
    pub opacity: f32,

    #[enum_component(Element)]
    #[builder(export)]
    pub element: ElementId,

    pub childs: Deque<usize, Slab<DeNode<usize>>>,

    pub yoga: YgNode,

    //zindex, -1表示auto
    #[listen]
	pub zindex: isize,

    //记录所有子节点及递归包含的子节点的数量
    #[ignore]
	pub count: usize,

    // z深度
    #[listen]
    pub z_depth: f32,

    // 被裁剪
    #[listen]
    pub by_overflow: usize,

    //布局数据
    #[listen]
    pub layout: Layout,
    // #[component(Vector3)]
    // #[builder(build(Default))]
    // pub position: usize, //位置（包括border）

    //中心点的位置
    // #[listen]
    // pub center: Vector3,

    #[listen]
    pub size: Vector2, //size

    #[builder(build(value=1.0))]
    #[listen]
    pub real_opacity: f32, //不透明度

    #[listen]
    #[builder(build(value=true))]
    pub real_visibility: bool, //是否可见（参与布局， 但不响应事件）

    #[component(Matrix4)]
    #[builder(build(Default))]
    pub world_matrix: usize, //世界矩阵组件

    #[listen]
    #[builder(export, build(value=true) )]
    pub enable: bool, // 设置是否响应事件,

    #[builder(export, build(value=true) )]
    pub real_enable: bool, // 是否响应事件, 受enable， display, visibility的影响

    pub bound_box_id: usize, //包围盒组件在八叉树中的id
    
    #[component(Aabb3)]
    #[builder(build(Default))]
    pub bound_box: usize, //包围盒组件


    #[ignore]
    pub layer: usize,

    #[ignore]
    pub qid: usize, //在父节点中的id，即在父节点的子容器中的key， 如果没有父节点， 该值为0

    text_child_count: usize,
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
                yoga.set_context(child_id as *mut c_void);
                let qid = match ty {
                    InsertType::Back => {
                        let node = group._group.get_mut(self.id);
                        let index = node.yoga.get_child_count();
                        node.yoga.insert_child(yoga.clone(), index as u32);
                        node.childs.push_back(child_id, &mut self.mgr.get_qid_container())
                    },
                    InsertType::Front => {
                        let node = group._group.get_mut(self.id);
                        node.yoga.insert_child(yoga.clone(), 0);
                        node.childs.push_front(child_id, &mut self.mgr.get_qid_container())
                    },
                    InsertType::ToBack(yoga_index, brother_qid) => {
                        let node = group._group.get_mut(self.id);
                        //TODO, node中可能存在文字节点， 无法使用yoga_index插入yoga
                        node.yoga.insert_child(yoga.clone(), yoga_index as u32);
                        unsafe{node.childs.push_to_back(child_id, brother_qid, &mut self.mgr.get_qid_container())}

                    },
                    InsertType::ToFront(mut yoga_index, brother_qid) => {
                        let brother_id = unsafe { self.mgr.get_qid_container().get_unchecked(brother_qid) }.elem;
                        let brother_yoga = group._group.get(brother_id).yoga;
                        let node = group._group.get_mut(self.id);                      
                        
                        if node.text_child_count > 0 {
                            yoga_index = brother_yoga.get_index();
                        }
                        node.yoga.insert_child(yoga.clone(), yoga_index as u32);
                        unsafe{node.childs.push_to_front(child_id, brother_qid, &mut self.mgr.get_qid_container())}
                    },
                };
                let child = group._group.get_mut(child_id);
                child.yoga = yoga;
                child.qid = qid; //不会发出qid改变的监听， 应该通知？
            },
            _ => panic!(format!("insert_child error, this is a leaf node")),
        };

        match group._group.get(child_id).element.clone() {
            ElementId::Text(_) => group._group.get_mut(self.id).text_child_count += 1,
            _ => (),
        };

        //如果节点不是根节点， 并且还没有父节点(不在节点树中)， 不需要发出修改事件和子节点创建事件
        if self.id != 1 && parent == 0 {
            return NodeWriteRef::new(child_id, self.groups, self.mgr);
        } 
        //根結點的修改事件不通知
        if self.id > 1 {
            handler.notify_modify_field(ModifyFieldEvent{id: self.id, parent: parent, field: "childs"}, &mut self.mgr);  //通知childs字段改变
        }
        let mut child_ref = NodeWriteRef::new(child_id, self.groups, self.mgr);
        child_ref._create(self.id);//通知Node及Node子组件的创建
        child_ref
    }

    pub fn remove_child(&mut self, child_id: usize) {
        let group = NodeGroup::<M>::from_usize_mut(self.groups);
        let qid = group._group.get_mut(child_id).qid;
        if !self.mgr.get_qid_container().contains(qid){
            panic!("remove_child fail!, node is not exist, qid:{}", qid);
        }
        let child_id = group._group.get_mut(self.id).childs.remove(qid, &mut self.mgr.get_qid_container()); //从childs移除child
        let (parent, child_yoga) = {
            let (child_yoga, is_text) = {
                let child = group._group.get(child_id);
                (child.yoga, match child.element {
                    ElementId::Text(_) => true,
                    _ => false,
                })
            };
            let node = group._group.get_mut(self.id);
            node.yoga.remove_child(child_yoga);

            if is_text {
                node.text_child_count -= 1;
            }

            (node.parent, child_yoga)
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
        handler.notify_modify_field(ModifyFieldEvent{id: self.id, parent: parent, field: "childs"}, &mut self.mgr); //通知childs字段改变
        NodeWriteRef::new(child_id, self.groups, self.mgr)._destroy(); //从容器中删除child的数据， 并抛出Node, 及Node子组件销毁的事件,
        child_yoga.free_recursive(); //递归释放子节点的yoga
    }

    fn _create(&mut self, parent: usize) {
        self.set_parent(parent);
        self.create_notify(); //通知Node及Node子组件的创建

        let mut child = self.get_childs().get_first();
        loop {
            if child == 0 {
                break;
            }
            let node_id = {
                let v = unsafe{ self.mgr.get_qid_container().get_unchecked(child) };
                child = v.next;
                v.elem.clone()
            };
            let mut child_ref = NodeWriteRef::new(node_id, self.groups, self.mgr);
            child_ref._create(self.id);
        }
    }

    fn _destroy(&mut self) {
        let mut child = self.get_childs().get_first();
        loop {
            if child == 0 {
                break;
            }
            let node_id = {
                let v = unsafe{ self.mgr.get_qid_container().get_unchecked(child) };
                child = v.next;
                v.elem.clone()
            };;
            NodeWriteRef::new(node_id, self.groups, self.mgr)._destroy();
        }
        self.destroy(); 
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
