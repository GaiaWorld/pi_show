/**
 * 监听opacity组件（该组件由外部设置本节点的不透明度， 会影响子节点的不透明度）， 递归计算最终的不透明度值， 将其记录在real_opacity组件中
 */

use std::cell::RefCell;
use std::rc::{Rc};

use wcs::world::{System};
use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent, ModifyFieldEvent};
use vecmap::{ VecMap, IndexMap};

use world_doc::component::node::{Node};
use world_doc::WorldDocMgr;
use world_doc::system::util::layer_dirty_mark::LayerDirtyMark;

pub struct OpacitySys(RefCell<LayerDirtyMark>);

impl OpacitySys {
    pub fn init(component_mgr: &mut WorldDocMgr) -> Rc<OpacitySys>{
        let system = Rc::new(OpacitySys(RefCell::new(LayerDirtyMark::new())));
        component_mgr.node.opacity.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node._group.register_create_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, CreateEvent, WorldDocMgr>>)));
        component_mgr.node._group.register_delete_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, DeleteEvent, WorldDocMgr>>)));
        system
    }
}

//监听opacity属性的改变
impl ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr> for OpacitySys{
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr){
        let ModifyFieldEvent{id, parent: _, field: _} = event;
        self.0.borrow_mut().marked_dirty(*id, component_mgr);
    }
}

//监听Node的创建， 设置脏标志
impl ComponentHandler<Node, CreateEvent, WorldDocMgr> for OpacitySys{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut WorldDocMgr){
        let CreateEvent{id, parent: _} = event;
        let mut borrow = self.0.borrow_mut();
        borrow.dirty_mark_list.insert(*id, false);
        borrow.marked_dirty(*id, component_mgr);
    }
}

//监听Node的删除创建， 删除脏标志
impl ComponentHandler<Node, DeleteEvent, WorldDocMgr> for OpacitySys{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut WorldDocMgr){
        let DeleteEvent{id, parent: _} = event;
        let mut borrow = self.0.borrow_mut();
        borrow.delete_dirty(*id, component_mgr);
        unsafe {borrow.dirty_mark_list.remove(*id)};
    }
}

impl System<(), WorldDocMgr> for OpacitySys{
    fn run(&self, _e: &(), component_mgr: &mut WorldDocMgr){
        cal_opacity(&mut self.0.borrow_mut(), component_mgr);
    }
}



//计算不透明度
pub fn cal_opacity(dirty_marks: &mut LayerDirtyMark, component_mgr: &mut WorldDocMgr){
    for d1 in dirty_marks.dirtys.iter() {
        for node_id in d1.iter() {
            if  *unsafe{dirty_marks.dirty_mark_list.get_unchecked(*node_id)} == false {
                continue;
            }

            let parent_id = component_mgr.node._group.get(*node_id).parent;
            if parent_id > 0 {
                modify_opacity(&mut dirty_marks.dirty_mark_list, component_mgr.node._group.get(parent_id).real_opacity, *node_id, component_mgr);
            }else {
                modify_opacity(&mut dirty_marks.dirty_mark_list, 1.0, *node_id, component_mgr);
            }
        }
    }

    dirty_marks.dirtys.clear();
}
 

//递归计算不透明度， 将节点最终的不透明度设置在real_opacity组件上
fn modify_opacity(dirty_mark_list: &mut VecMap<bool>, parent_real_opacity: f32, node_id: usize, component_mgr: &mut WorldDocMgr) {
    let node_opacity = {
        let node = component_mgr.node._group.get(node_id);
        node.opacity
    };
    let node_real_opacity = node_opacity * parent_real_opacity;

    let mut child = {
        let mut node_ref = component_mgr.get_node_mut(node_id);
        node_ref.set_real_opacity(node_real_opacity);
        node_ref.get_childs_mut().get_first()
    };

    unsafe{*dirty_mark_list.get_unchecked_mut(node_id) = false}
    //递归计算子节点的世界矩阵
    loop {
        if child == 0 {
            return;
        }
        let node_id = {
            let v = unsafe{ component_mgr.node_container.get_unchecked(child) };
            child = v.next;
            v.elem.clone()
        };
        modify_opacity(dirty_mark_list, node_real_opacity, node_id, component_mgr);
    }
}

#[cfg(test)]
#[cfg(not(feature = "web"))]
mod test {
    use std::rc::Rc;

    use wcs::component::Builder;
    use wcs::world::{World, System};
    use world_doc::WorldDocMgr;

    use world_doc::component::node::{NodeBuilder, InsertType};
    use world_doc::system::opacity::OpacitySys;

    #[test]
    fn test(){
        let mut world = new_world();
        let node2 = NodeBuilder::new().build(&mut world.component_mgr.node);
        let node3 = NodeBuilder::new().build(&mut world.component_mgr.node);
        let node4 = NodeBuilder::new().build(&mut world.component_mgr.node);
        let node5 = NodeBuilder::new().build(&mut world.component_mgr.node);
        let node6 = NodeBuilder::new().build(&mut world.component_mgr.node);
        let node7 = NodeBuilder::new().build(&mut world.component_mgr.node);
        let node8 = NodeBuilder::new().build(&mut world.component_mgr.node);
        let node9 = NodeBuilder::new().build(&mut world.component_mgr.node);

        let (root, node_ids) = {
            let root = NodeBuilder::new().build(&mut world.component_mgr.node);
            let root_id = world.component_mgr.add_node(root).id;
            let mgr = &mut world.component_mgr;
            
            //root的直接子节点
            let node2 = mgr.get_node_mut(root_id).insert_child(node2, InsertType::Back).id;
            let node3 = mgr.get_node_mut(root_id).insert_child(node3, InsertType::Back).id;

            //node2的直接子节点
            let node4 = mgr.get_node_mut(node2).insert_child(node4, InsertType::Back).id;
            let node5 = mgr.get_node_mut(node2).insert_child(node5, InsertType::Back).id;

            //node3的直接子节点
            let node6 = mgr.get_node_mut(node3).insert_child(node6, InsertType::Back).id;
            let node7 = mgr.get_node_mut(node3).insert_child(node7, InsertType::Back).id;

            //node4的直接子节点
            let node8 = mgr.get_node_mut(node4).insert_child(node8, InsertType::Back).id;
            let node9 = mgr.get_node_mut(node4).insert_child(node9, InsertType::Back).id;

            (
                root_id,
                vec![node2, node3, node4, node5, node6, node7, node8, node9]
            )
        };

        //  mgr.get_node_mut(root).
        world.run(());
        for i in node_ids.iter(){
            {
                let node_ref = world.component_mgr.get_node_mut(*i);
                let real_opacity = node_ref.get_real_opacity();
                println!("test_opacity1, node{} , real_opacity:{}", i, real_opacity);
            }
        }

        world.component_mgr.get_node_mut(root).set_opacity(0.5);
        world.run(());
        println!("-----------------------------------------------------------------");
        for i in node_ids.iter(){
            {
                let node_ref = world.component_mgr.get_node_mut(*i);
                let real_opacity = node_ref.get_real_opacity();
                println!("test_opacity2, node{} , real_opacity:{}", i, real_opacity);
            }
        }

        //修改node2的opacity
        world.component_mgr.get_node_mut(node_ids[0]).set_opacity(0.5);
        world.component_mgr.get_node_mut(node_ids[2]).set_opacity(0.5);
        world.run(());
        println!("-----------------------------------------------------------------");
        for i in node_ids.iter(){
            {
                let node_ref = world.component_mgr.get_node_mut(*i);
                let real_opacity = node_ref.get_real_opacity();
                println!("test_opacity3, node{} , real_opacity:{}, opacity{}", i, real_opacity, node_ref.get_opacity());
            }
        }

        // forget(world);
    }

    fn new_world() -> World<WorldDocMgr, ()>{
        let mut world: World<WorldDocMgr, ()> = World::new(WorldDocMgr::new());
        let systems: Vec<Rc<System<(), WorldDocMgr>>> = vec![OpacitySys::init(&mut world.component_mgr)];
        world.set_systems(systems);
        world
    }
}
