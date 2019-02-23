use std::cell::RefCell;
use std::rc::{Rc};
use std::ops::Deref;

use web_sys::*;

use wcs::world::{System};
use wcs::component::{ComponentHandler, Event};
use cg::octree::*;

use component::component_def::{GuiComponentMgr, RectSize};
use component::math::{Matrix4, Aabb3 as C_Aabb3};
use cg::{Aabb3, Vector4, Point3};
// use alert;

pub struct Oct(RefCell<OctImpl>);

impl Oct {
    pub fn init(component_mgr: &mut GuiComponentMgr, extent: Aabb3<f32>) -> Rc<Oct>{
        let r = Rc::new(Oct(RefCell::new(OctImpl::new(extent))));
        component_mgr.node.extent._group.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<RectSize, GuiComponentMgr>>)));
        component_mgr.node.world_matrix._group.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Matrix4, GuiComponentMgr>>)));
        r
    }
}

impl ComponentHandler<RectSize, GuiComponentMgr> for Oct{
    fn handle(&self, event: &Event, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::Create{id:_, parent} => {
                self.0.borrow_mut().add_aabb(*parent, component_mgr);
                self.0.borrow_mut().marked_dirty(*parent, component_mgr);
            },
            Event::Delete{id:_, parent} => {
                self.0.borrow_mut().remove_aabb(*parent, component_mgr);
                self.0.borrow_mut().delete_dirty(*parent);
            },
            Event::ModifyField{id:_, parent, field: _} => {
                console::log_1(&("oct_extent_modify".into()));
                self.0.borrow_mut().marked_dirty(*parent, component_mgr);
            },
            _ => ()
        }
    }
}

impl ComponentHandler<Matrix4, GuiComponentMgr> for Oct{
    fn handle(&self, event: &Event, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::ModifyField{id:_, parent, field: _} => {
                self.0.borrow_mut().marked_dirty(*parent, component_mgr);
            },
            //监听了size组件的创建和销毁， 不需要在监听Matrix4组件的创建和销毁
            _ => ()
        }
    }
}

impl System<(), GuiComponentMgr> for Oct{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        self.0.borrow_mut().cal_bound_box(component_mgr);
    }
}

pub struct OctImpl {
    dirtys: Vec<usize>, //Vec<node_id>
    octree: Tree<f32, usize>,
}

impl OctImpl {
    pub fn new(size: Aabb3<f32>) -> OctImpl{
        OctImpl{
            dirtys: Vec::new(),
            octree: Tree::new(size, 0, 0, 0, 0)
        }
    }

    //计算包围盒
    pub fn cal_bound_box(&mut self, mgr: &mut GuiComponentMgr){
        for node_id in self.dirtys.iter() {
            mgr.node._group.get_mut(*node_id).bound_box_dirty = false;

            let aabb = {
                //计算包围盒
                let node = mgr.node._group.get(*node_id);
                let extent = mgr.node.extent._group.get(node.extent);
                let world_matrix = mgr.node.world_matrix._group.get(mgr.node._group.get(*node_id).world_matrix);
                let aabb = cal_bound_box(extent, world_matrix);
                console::log_3(&("extent".into()), &(extent.width.to_string().into()), &(extent.height.to_string().into()));
                //更新八叉树
                self.octree.update(node.bound_box_id, aabb.clone());
                aabb
            };
            {
                //修改包围盒
                let mut node_ref = mgr.get_node_mut(*node_id);
                node_ref.get_bound_box_mut().modify(|aabb3: &mut C_Aabb3|{
                    console::log_5(&("oct_box".into()), &(aabb.min.x.to_string().into()), &(aabb.min.y.to_string().into()), &(aabb.max.x.to_string().into()), &(aabb.max.y.to_string().into()));
                    aabb3.min = aabb.min;
                    aabb3.max = aabb.max;
                    true
                });
            }
        }
    }

    pub fn marked_dirty(&mut self, node_id: usize, mgr: &mut GuiComponentMgr){
        {
            let node = mgr.node._group.get_mut(node_id);
            if node.bound_box_dirty == true {
                return;
            }
            node.bound_box_dirty = true;
        }

        self.dirtys.push(node_id.clone());
    }

    pub fn delete_dirty(&mut self, node_id: usize){
        for i in 0..self.dirtys.len(){
            if self.dirtys[i] == node_id{
                self.dirtys.remove(i);
                return;
            }
        }
    }

    pub fn add_aabb(&mut self, node_id: usize, mgr: &mut GuiComponentMgr){
        let aabb = mgr.node.bound_box._group.get_mut(mgr.node._group.get(node_id).bound_box).owner.clone();
        let oct_id = self.octree.add(aabb.0, node_id);
        console::log_1(&("set_bound_box_id start".into()));
        mgr.get_node_mut(node_id).set_bound_box_id(oct_id);
        console::log_1(&("set_bound_box_id end".into()));
    }

    pub fn remove_aabb(&mut self, node_id: usize, mgr: &mut GuiComponentMgr){
        let node = mgr.node._group.get_mut(node_id);
        if node.bound_box > 0 {
            self.octree.remove(node.bound_box);
        }
    }
}

fn cal_bound_box(size: &RectSize, matrix: &Matrix4) -> Aabb3<f32>{
    // let half_width = size.width/2.0;
    // let half_height = size.height/2.0;
    // let let_top = matrix.deref() * Vector4::new(-half_width, -half_height, 0.0, 1.0);
    // let right_top = matrix.deref() * Vector4::new(size.width-half_width, -half_height, 0.0, 1.0);
    // let left_bottom = matrix.deref() * Vector4::new(-half_width, size.height - half_height, 0.0, 1.0);
    // let right_bottom = matrix.deref() * Vector4::new(size.width - half_width, size.height - half_width, 0.0, 1.0);
    let let_top = matrix.deref() * Vector4::new(0.0, 0.0, 0.0, 1.0);
    let right_top = matrix.deref() * Vector4::new(size.width, 0.0, 0.0, 1.0);
    let left_bottom = matrix.deref() * Vector4::new(size.height, 0.0, 0.0, 1.0);
    let right_bottom = matrix.deref() * Vector4::new(size.width, size.height, 0.0, 1.0);

    // let min = Point3::new(
    //     let_top.x.min(right_top.x).min(left_bottom.x).min(right_bottom.x) + half_width,
    //     let_top.y.min(right_top.y).min(left_bottom.y).min(right_bottom.y) + half_height,
    //     0.0,
    // );

    // let max = Point3::new(
    //     let_top.x.max(right_top.x).max(left_bottom.x).max(right_bottom.x) + half_width,
    //     let_top.y.max(right_top.y).max(left_bottom.y).max(right_bottom.y) + half_height,
    //     1.0,
    // );
    // let x = matrix
    // console::log_5(&("matrix".into()), );
    let min = Point3::new(
        let_top.x.min(right_top.x).min(left_bottom.x).min(right_bottom.x),
        let_top.y.min(right_top.y).min(left_bottom.y).min(right_bottom.y),
        0.0,
    );

    let max = Point3::new(
        let_top.x.max(right_top.x).max(left_bottom.x).max(right_bottom.x),
        let_top.y.max(right_top.y).max(left_bottom.y).max(right_bottom.y),
        1.0,
    );

    Aabb3::new(min, max)
}

// #[cfg(test)]
// use wcs::world::{World};
// #[cfg(test)]
// use component::math_component::{Vector3};
// #[cfg(test)]
// use system::world_matrix::{WorldMatrix};

// #[test]
// fn test(){
//     let mut world: World<GuiComponentMgr, ()> = World::new();
//     let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![WorldMatrix::init(&mut world.component_mgr), Oct::init(&mut world.component_mgr, Aabb3::new(id3::new(2000.0, 2000.0, 2000.0), id3::new(2000.0, 2000.0, 2000.0)))];
//     world.set_systems(systems);

//     let (root, node1, _node2, node3, node4, node5) = {
//         let component_mgr = &mut world.component_mgr;
//         {
            
//             let (root, node1, node2, node3, node4, node5) = {
//                 let mut root = component_mgr.create_node(&0);
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
//     print_node(&world.component_mgr, &node3);
//     print_node(&world.component_mgr, &node4);
//     print_node(&world.component_mgr, &node5);
// }

// #[cfg(test)]
// fn print_node(mgr: &GuiComponentMgr, id: &usize) {
//     let node = mgr.node._group.get(&id);
//     let bound_box_data = mgr.node.bound_box_data._group.get(&node.bound_box_data);
//     println!("nodeid: {}, bound_box_data:{:?}, bound_box_dirty: {}, bound_box: {}", id, bound_box_data, node.bound_box_dirty, node.bound_box);
// }
