/**
 * 监听visibility组件， 递归设置子节点的real_visibility
 */

use std::cell::RefCell;
use std::rc::{Rc};

use wcs::world::{System};
use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent, ModifyFieldEvent};
use vecmap::VecMap;

use world_doc::component::node::{Node};
use world_doc::WorldDocMgr;
use world_doc::system::util::layer_dirty_mark::LayerDirtyMark;

pub struct VisibilitySys(RefCell<LayerDirtyMark>);

impl VisibilitySys {
    pub fn init(component_mgr: &mut WorldDocMgr) -> Rc<VisibilitySys>{
        let system = Rc::new(VisibilitySys(RefCell::new(LayerDirtyMark::new())));
        component_mgr.node.visibility.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node._group.register_create_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, CreateEvent, WorldDocMgr>>)));
        component_mgr.node._group.register_delete_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, DeleteEvent, WorldDocMgr>>)));
        system
    }
}

//监听visibility属性的改变
impl ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr> for VisibilitySys{
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr){
        let ModifyFieldEvent{id, parent: _, field: _} = event;
        self.0.borrow_mut().marked_dirty(*id, component_mgr);
    }
}

//监听Node的创建， 设置脏标志
impl ComponentHandler<Node, CreateEvent, WorldDocMgr> for VisibilitySys{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut WorldDocMgr){
        let CreateEvent{id, parent: _} = event;
        let mut borrow = self.0.borrow_mut();
        borrow.dirty_mark_list.insert(*id, false);
        borrow.marked_dirty(*id, component_mgr);
    }
}

//监听Node的删除创建， 删除脏标志
impl ComponentHandler<Node, DeleteEvent, WorldDocMgr> for VisibilitySys{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut WorldDocMgr){
        let DeleteEvent{id, parent: _} = event;
        let mut borrow = self.0.borrow_mut();
        borrow.delete_dirty(*id, component_mgr);
        unsafe {borrow.dirty_mark_list.remove(*id)};
    }
}

impl System<(), WorldDocMgr> for VisibilitySys{
    fn run(&self, _e: &(), component_mgr: &mut WorldDocMgr){
        cal_visibility(&mut self.0.borrow_mut(), component_mgr);
    }
}

//循环脏标记， 递归设置real_visibility
fn cal_visibility(dirty_marks: &mut LayerDirtyMark, component_mgr: &mut WorldDocMgr){
    for d1 in dirty_marks.dirtys.iter() {
        for node_id in d1.iter() {
            if  *unsafe{dirty_marks.dirty_mark_list.get_unchecked(*node_id)} == false {
                continue;
            }

            let parent_id = component_mgr.node._group.get(*node_id).parent;
            if parent_id > 0 {
                modify_visibility(&mut dirty_marks.dirty_mark_list, component_mgr.node._group.get(parent_id).real_visibility, *node_id, component_mgr);
            }else {
                modify_visibility(&mut dirty_marks.dirty_mark_list, true, *node_id, component_mgr);
            }
        }
    }

    dirty_marks.dirtys.clear();
}

//递归计算visibility， 将节点最终的visibility设置在real_visibility属性上
fn modify_visibility(dirty_mark_list: &mut VecMap<bool>, parent_real_visibility: bool, node_id: usize, component_mgr: &mut WorldDocMgr) {
    let (node_visibility, old_node_real_visibility) = {
        let node = component_mgr.node._group.get(node_id);
        (node.visibility, node.real_visibility)
    };
    let node_real_visibility = node_visibility && parent_real_visibility;
    //如果real_visibility值没有改变， 不需要递归设置子节点的real_visibility值
    if old_node_real_visibility == node_real_visibility {
        return;
    }

    let mut child = {
        let mut node_ref = component_mgr.get_node_mut(node_id);
        //设置real_visibility
        node_ref.set_real_visibility(node_real_visibility);
        node_ref.get_childs_mut().get_first()
    };

    unsafe{*dirty_mark_list.get_unchecked_mut(node_id) = false}
    //递归设置子节点的real_visibility
    loop {
        if child == 0 {
            return;
        }
        let node_id = {
            let v = unsafe{ component_mgr.node_container.get_unchecked(child) };
            child = v.next;
            v.elem
        };
        modify_visibility(dirty_mark_list, node_real_visibility, node_id, component_mgr);
    }
}