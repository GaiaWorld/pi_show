/**
 * 监听opacity组件（该组件由外部设置本节点的不透明度， 会影响子节点的不透明度）， 递归计算最终的不透明度值， 将其记录在real_opacity组件中
 */

use std::cell::RefCell;
use std::rc::{Rc};

use wcs::world::{System};
use wcs::component::{ComponentHandler, Event};
use slab::Slab;

use component::style::generic::{Opacity};
use component::node::{Node};
use world::GuiComponentMgr;

pub struct OpacitySys(RefCell<OpacitySysImpl>);

impl OpacitySys {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<OpacitySys>{
        let system = Rc::new(OpacitySys(RefCell::new(OpacitySysImpl::new())));
        component_mgr.node.opacity._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Opacity, GuiComponentMgr>>)));
        system
    }
}

impl ComponentHandler<Opacity, GuiComponentMgr> for OpacitySys{
    fn handle(&self, event: &Event, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::ModifyField{id: _, parent, field: _} => {
                self.0.borrow_mut().marked_dirty(*parent, component_mgr);
            },
            Event::Create{id, parent} => {
                if component_mgr.node.opacity._group.get(*id).value != 1.0{
                    self.0.borrow_mut().marked_dirty(*parent, component_mgr);
                }
            },
            Event::Delete{id, parent: _} => {
                self.0.borrow_mut().delete_dirty(*id, component_mgr);
            },
            _ => ()
        }
    }
}

impl ComponentHandler<Node, GuiComponentMgr> for OpacitySys{
    fn handle(&self, event: &Event, _component_mgr: &mut GuiComponentMgr){
        match event {
            Event::Create{id, parent: _} => {
                self.0.borrow_mut().dirty_mark_list.insert_at(*id, true);
            },
            Event::Delete{id, parent: _} => {
                self.0.borrow_mut().dirty_mark_list.remove(*id);
            },
            _ => ()
        }
    }
}

impl System<(), GuiComponentMgr> for OpacitySys{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        self.0.borrow_mut().cal_opacity(component_mgr);
    }
}

pub struct OpacitySysImpl {
    dirtys: Vec<Vec<usize>>, //Vec<Vec<node_id>>
    dirty_mark_list: Slab<bool>,
}

impl OpacitySysImpl {
    pub fn new() -> OpacitySysImpl{
        OpacitySysImpl{
            dirtys: Vec::new(),
            dirty_mark_list: Slab::new(),
        }
    }

    //计算不透明度
    pub fn cal_opacity(&mut self, component_mgr: &mut GuiComponentMgr){
        for d1 in self.dirtys.iter() {
            for node_id in d1.iter() {
                //修改节点世界矩阵及子节点的世界矩阵
                modify_opacity(&mut self.dirty_mark_list, *node_id, component_mgr);
            }
        }

        self.dirtys.clear();
    }

    pub fn marked_dirty(&mut self, node_id: usize, mgr: &mut GuiComponentMgr){
        let layer = {
            let dirty_mark = unsafe{self.dirty_mark_list.get_unchecked_mut(node_id)};
            if *dirty_mark == true {
                return;
            }
            *dirty_mark = true;

            mgr.node._group.get(node_id).layer
        };

        if self.dirtys.len() <= layer{
            for _i in 0..(layer + 1 - self.dirtys.len()){
                self.dirtys.push(Vec::new());
            }
        }
        self.dirtys[layer].push(node_id);
    }

    pub fn delete_dirty(&mut self, node_id: usize, mgr: &mut GuiComponentMgr){
        let node = mgr.node._group.get_mut(node_id);
        let dirty_mark = unsafe{self.dirty_mark_list.get_unchecked_mut(node_id)};
        if *dirty_mark == true {
            let layer = node.layer;
            for i in 0..self.dirtys[layer].len() {
                if self.dirtys[layer][i] == node_id {
                    self.dirtys[layer].swap_remove(i);
                    return;
                }
            }
        }
    }
}

//递归计算不透明度， 将节点最终的不透明度设置在real_opacity组件上
fn modify_opacity(dirty_mark_list: &mut Slab<bool>, node_id: usize, component_mgr: &mut GuiComponentMgr) {
    // 设置脏标志
    {
        let dirty_mark = unsafe{dirty_mark_list.get_unchecked_mut(node_id)};
        if *dirty_mark == false {
            return;
        }
        *dirty_mark = false;
    }
    let (node_opacity_id, parent_id) = {
        let node = component_mgr.node._group.get(node_id);
        (node.opacity, node.parent)
    };

    let parent_opacity_id = component_mgr.node._group.get(parent_id).parent;

    let node_opacity;
    let parent_opacity = component_mgr.node.real_opacity._group.get(parent_opacity_id).value; //real_opacity组件一定存在
    if node_opacity_id == 0 {
        node_opacity = 1.0;
    }else {
        node_opacity = component_mgr.node.opacity._group.get(node_opacity_id).value
    }

    let value = node_opacity * parent_opacity;
    
    let mut child = {
        let mut node_ref = component_mgr.get_node_mut(node_id);
        node_ref.get_real_opacity_mut().modify(|opacity: &mut Opacity|{
            if value == opacity.value {
                false
            }else {
                opacity.value = value;
                true
            }
        });

        node_ref.get_childs_mut().get_first()
    };
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
        modify_opacity(dirty_mark_list, node_id, component_mgr);
    }
}

// #[cfg(test)]
// use wcs::world::{World};
// #[cfg(test)]
// use component::math_component::{Vector3};

// #[test]
// fn test(){
//     let mut world: World<GuiComponentMgr, ()> = World::new();
//     let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![OpacitySys::init(&mut world.component_mgr)];
//     world.set_systems(systems);
//     test_world_matrix(&mut world);
// }

// #[cfg(test)]
// fn test_world_matrix(world: &mut World<GuiComponentMgr, ()>){
//     let (root, node1, node2, node3, node4, node5) = {
//         let component_mgr = &mut world.component_mgr;
//         {
            
//             let (root, node1, node2, node3, node4, node5) = {
//                 let mut root = component_mgr.create_node(0);
//                 root.set_layer(1);
//                 (   
//                     root.id.0.clone(),
//                     root.create_child_back().id.0.clone(), 
//                     root.create_child_back().id.0.clone(),
//                     root.create_child_back().id.0.clone(),
//                     root.create_child_back().id.0.clone(),
//                     root.create_child_back().id.0.clone(),
//                 )
//            };
//             print_node(component_mgr, &node1);
//             print_node(component_mgr, &node2);
//             print_node(component_mgr, &node3);
//             print_node(component_mgr, &node4);
//             print_node(component_mgr, &node5);

//             {
//                 let mut node = component_mgr.get_node_mut(&root);
//                 let mut size = node.get_size_mut();
//                 size.set_width(500.0);
//                 size.set_height(500.0);
//             }

//             {
//                 let mut node = component_mgr.get_node_mut(&node1);
//                 let mut size = node.get_size_mut();
//                 size.set_width(100.0);
//                 size.set_height(100.0);
//             }

//             {
//                 let mut node = component_mgr.get_node_mut(&node2);
//                 {
//                     let mut size = node.get_size_mut();
//                     size.set_width(100.0);
//                     size.set_height(100.0);
//                 }
//                 {
//                     let mut transform = node.get_transform_mut();
//                     transform.set_position(Vector3::new(100.0, 0.0, 0.0));
//                 }
                
//             }

//             {
//                 let mut node = component_mgr.get_node_mut(&node3);
//                 {
//                     let mut size = node.get_size_mut();
//                     size.set_width(100.0);
//                     size.set_height(100.0);
//                 }
//                 {
//                     let mut transform = node.get_transform_mut();
//                     transform.set_position(Vector3::new(200.0, 0.0, 0.0));
//                 }
                
//             }

//             {
//                 let mut node = component_mgr.get_node_mut(&node4);
//                 {
//                     let mut size = node.get_size_mut();
//                     size.set_width(100.0);
//                     size.set_height(100.0);
//                 }
//                 {
//                     let mut transform = node.get_transform_mut();
//                     transform.set_position(Vector3::new(100.0, 0.0, 0.0));
//                     transform.set_position(Vector3::new(400.0, 0.0, 0.0));
//                 }
                
//             }

//             {
//                 let mut node = component_mgr.get_node_mut(&node5);
//                 {
//                     let mut size = node.get_size_mut();
//                     size.set_width(100.0);
//                     size.set_height(100.0);
//                 }
//                 {
//                     let mut transform = node.get_transform_mut();
//                     transform.set_position(Vector3::new(0.0, 100.0, 0.0));
//                 }
                
//             }
//             println!("modify-----------------------------------------");
//             print_node(component_mgr, &node1);
//             print_node(component_mgr, &node2);
//             print_node(component_mgr, &node3);
//             print_node(component_mgr, &node4);
//             print_node(component_mgr, &node5);

//             let node2_qid = component_mgr.get_node_mut(&node2).get_qid().clone();
//             component_mgr.get_node_mut(&root).remove_child(node2_qid);
//             (root, node1, node2, node3, node4, node5)
//         }
//     };

//     println!("modify run-----------------------------------------");
//     world.run(());
//     print_node(&world.component_mgr, &root);
//     print_node(&world.component_mgr, &node1);
//     print_node(&world.component_mgr, &node2);
//     print_node(&world.component_mgr, &node3);
//     print_node(&world.component_mgr, &node4);
//     print_node(&world.component_mgr, &node5);
// }

// #[cfg(test)]
// fn print_node(mgr: &GuiComponentMgr, id: &usize) {
//     let node = mgr.node._group.get(&id);
//     let transform = mgr.node.transform._group.get(&node.transform);
//     let matrix = mgr.node.world_matrix._group.get(&node.world_matrix);

//     println!("nodeid: {}, transform:{:?}, world_matrix: {:?}, matrix_dirty: {}", id, transform, matrix, node.world_matrix_dirty);
// }