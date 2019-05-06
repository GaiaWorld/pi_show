/**
 * 监听real_visibility, enable 组件, 修改real_enable属性
 */
use std::rc::{Rc};

use wcs::component::{ComponentHandler, CreateEvent, ModifyFieldEvent};
use wcs::world::System;

use world_doc::component::node::{Node};
use world_doc::WorldDocMgr;

pub struct EnableSys;

impl System<(), WorldDocMgr> for EnableSys{
    fn run(&self, _e: &(), _component_mgr: &mut WorldDocMgr){

    }
}

impl EnableSys {
    pub fn init(component_mgr: &mut WorldDocMgr) -> Rc<EnableSys>{
        let system = Rc::new(EnableSys);
        component_mgr.node.real_visibility.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node.enable.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node._group.register_create_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, CreateEvent, WorldDocMgr>>)));
        system
    }
}

//监听real_visibility, enable属性的改变
impl ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr> for EnableSys{
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr){
        let ModifyFieldEvent{id, parent, field: _} = event;
        let parent_real_enable = component_mgr.node._group.get(*parent).real_enable;
        modify_real_enable(parent_real_enable, *id, component_mgr);
    }
}

//监听Node的创建， 设置脏标志
impl ComponentHandler<Node, CreateEvent, WorldDocMgr> for EnableSys{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut WorldDocMgr){
        let CreateEvent{id, parent} = event;
        let parent_real_enable = component_mgr.node._group.get(*parent).real_enable;
        modify_real_enable(parent_real_enable, *id, component_mgr);
    }
}

//递归计算real_enable
fn modify_real_enable(parent_real_enable: bool, node_id: usize, component_mgr: &mut WorldDocMgr) {
    let (enable, real_visibility, real_enable) = {
        let node = component_mgr.node._group.get(node_id);
        (node.enable, node.real_visibility, node.real_enable)
    };
    let node_real_enable = parent_real_enable && enable && real_visibility;

    if node_real_enable == real_enable {
        return;
    }

    let mut child = {
        let mut node_ref = component_mgr.get_node_mut(node_id);
        node_ref.set_real_enable(node_real_enable);
        node_ref.get_childs_mut().get_first()
    };

    loop {
        if child == 0 {
            return;
        }
        let node_id = {
            let v = unsafe{ component_mgr.node_container.get_unchecked(child) };
            child = v.next;
            v.elem.clone()
        };
        modify_real_enable(node_real_enable, node_id, component_mgr);
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
    use world_doc::system::opacity::EnableSys;

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

    
        for i in node_ids.iter(){
            {
                let node_ref = world.component_mgr.get_node_mut(*i);
                debug_println!("1:{}", node_ref.get_real_enable());
            }
        }

        world.component_mgr.get_node_mut(root).set_enable(false);
        for i in node_ids.iter(){
            {
                let node_ref = world.component_mgr.get_node_mut(*i);
                debug_println!("2:{}", node_ref.get_real_enable());
            }
        }
        world.component_mgr.get_node_mut(root).set_enable(true);

        //修改node2的display
        world.component_mgr.get_node_mut(node_ids[0]).set_display(Display::None);
        for i in node_ids.iter(){
            {
                let node_ref = world.component_mgr.get_node_mut(*i);
                debug_println!("3:{}", node_ref.get_real_enable());
            }
        }
        world.component_mgr.get_node_mut(node_ids[0]).set_display(Display::Flex);

        //修改root的vis
        world.component_mgr.get_node_mut(root).set_vis(false);
        for i in node_ids.iter(){
            {
                let node_ref = world.component_mgr.get_node_mut(*i);
                debug_println!("3:{}", node_ref.get_real_enable());
            }
        }
        world.component_mgr.get_node_mut(root).set_enable(false);

        // forget(world);
    }

    fn new_world() -> World<WorldDocMgr, ()>{
        let mut world: World<WorldDocMgr, ()> = World::new(WorldDocMgr::new());
        let systems: Vec<Rc<System<(), WorldDocMgr>>> = vec![EnableSys::init(&mut world.component_mgr)];
        world.set_systems(systems);
        world
    }
}
