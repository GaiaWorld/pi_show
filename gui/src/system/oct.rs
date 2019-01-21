use std::cell::RefCell;
use std::rc::{Rc};

use wcs::world::{System};
use wcs::component::{ComponentHandler, Event};
use cg::octree::*;

use component::component_def::{NodePoint, GuiComponentMgr, Matrix4Point,SizePoint, Size};
use component::math::{Aabb3, Matrix4, Vector4, Point3};
// use alert;

pub struct Oct(RefCell<OctImpl>);

impl Oct {
    pub fn init(component_mgr: &mut GuiComponentMgr, size: Aabb3) -> Rc<Oct>{
        let r = Rc::new(Oct(RefCell::new(OctImpl::new(size))));
        component_mgr.node.size._group.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<SizePoint, GuiComponentMgr>>)));
        component_mgr.node.world_matrix._group.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Matrix4Point, GuiComponentMgr>>)));
        r
    }
}

impl ComponentHandler<SizePoint, GuiComponentMgr> for Oct{
    fn handle(&self, event: &Event<SizePoint>, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::Create{point:_, parent} => {
                let node = NodePoint(parent.clone());
                self.0.borrow_mut().add_aabb(&node, component_mgr);
                self.0.borrow_mut().marked_dirty(&node, component_mgr);
            },
            Event::Delete{point:_, parent} => {
                let node = NodePoint(parent.clone());
                self.0.borrow_mut().remove_aabb(&node, component_mgr);
                self.0.borrow_mut().delete_dirty(&node);
            },
            Event::ModifyField{point:_, parent, field: _} => {
                self.0.borrow_mut().marked_dirty(&NodePoint(parent.clone()), component_mgr);
            },
            _ => ()
        }
    }
}

impl ComponentHandler<Matrix4Point, GuiComponentMgr> for Oct{
    fn handle(&self, event: &Event<Matrix4Point>, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::ModifyField{point:_, parent, field: _} => {
                self.0.borrow_mut().marked_dirty(&NodePoint(parent.clone()), component_mgr);
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
    dirtys: Vec<NodePoint>,
    octree: Tree<f32, usize>,
}

impl OctImpl {
    pub fn new(size: Aabb3) -> OctImpl{
        OctImpl{
            dirtys: Vec::new(),
            octree: Tree::new(size, 0, 0, 0, 0)
        }
    }

    //计算包围盒
    pub fn cal_bound_box(&mut self, mgr: &mut GuiComponentMgr){
        for node_point in self.dirtys.iter() {
            mgr.node._group.get_mut(&node_point).bound_box_dirty = false;

            let aabb = {
                //计算包围盒
                let size = mgr.node.size._group.get(&mgr.node._group.get(node_point).size);
                let world_matrix = mgr.node.world_matrix._group.get(&mgr.node._group.get(node_point).world_matrix);
                let aabb = cal_bound_box(size, world_matrix);
                //更新八叉树
                self.octree.update(node_point.get_bound_box(&mgr.node).clone(), aabb.clone());
                aabb
            };
            {
                //修改包围盒
                let mut node_ref = mgr.get_node_mut(node_point);
                node_ref.get_bound_box_data_mut().modify(|aabb3: &mut Aabb3|{
                    aabb3.min = aabb.min;
                    aabb3.max = aabb.max;
                    true
                });
            }
        }
    }

    pub fn marked_dirty(&mut self, node_point: &NodePoint, mgr: &mut GuiComponentMgr){
        {
            let node = mgr.node._group.get_mut(&node_point);
            if node.bound_box_dirty == true {
                return;
            }
            node.bound_box_dirty = true;
        }

        self.dirtys.push(node_point.clone());
    }

    pub fn delete_dirty(&mut self, node: &NodePoint){
        for i in 0..self.dirtys.len(){
            if self.dirtys[i].0 == node.0{
                self.dirtys.remove(i);
                return;
            }
        }
    }

    pub fn add_aabb(&mut self, node_point: &NodePoint, mgr: &mut GuiComponentMgr){
        let aabb = mgr.node.bound_box_data._group.get_mut(&mgr.node._group.get(node_point).bound_box_data).owner.clone();
        let index = self.octree.add(aabb, node_point.0);
        mgr.get_node_mut(node_point).set_bound_box(index);
    }

    pub fn remove_aabb(&mut self, node: &NodePoint, mgr: &mut GuiComponentMgr){
        let node = mgr.node._group.get_mut(node);
        if node.bound_box > 0 {
            self.octree.remove(node.bound_box);
        }
    }
}

fn cal_bound_box(size: &Size, matrix: &Matrix4) -> Aabb3{
    let half_width = size.width/2.0;
    let half_height = size.height/2.0;
    let let_top = matrix * Vector4::new(-half_width, -half_height, 0.0, 1.0);
    let right_top = matrix * Vector4::new(size.width-half_width, -half_height, 0.0, 1.0);
    let left_bottom = matrix * Vector4::new(-half_width, size.height - half_height, 0.0, 1.0);
    let right_bottom = matrix * Vector4::new(size.width - half_width, size.height - half_width, 0.0, 1.0);

    let min = Point3::new(
        let_top.x.min(right_top.x).min(left_bottom.x).min(right_bottom.x) + half_width,
        let_top.y.min(right_top.y).min(left_bottom.y).min(right_bottom.y) + half_height,
        0.0,
    );

    let max = Point3::new(
        let_top.x.max(right_top.x).max(left_bottom.x).max(right_bottom.x) + half_width,
        let_top.y.max(right_top.y).max(left_bottom.y).max(right_bottom.y) + half_height,
        1.0,
    );

    Aabb3::new(min, max)
}

#[cfg(test)]
use wcs::world::{World};
#[cfg(test)]
use component::math::{Vector3};
#[cfg(test)]
use component::component_def::{Children};
#[cfg(test)]
use system::world_matrix::{WorldMatrix};

#[test]
fn test(){
    let mut world: World<GuiComponentMgr, ()> = World::new();
    let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![WorldMatrix::init(&mut world.component_mgr), Oct::init(&mut world.component_mgr, Aabb3::new(Point3::new(2000.0, 2000.0, 2000.0), Point3::new(2000.0, 2000.0, 2000.0)))];
    world.set_systems(systems);

    let (root, node1, _node2, node3, node4, node5) = {
        let component_mgr = &mut world.component_mgr;
        {
            
            let (root, node1, node2, node3, node4, node5) = {
                let mut root = component_mgr.create_node(&0);
                root.set_layer(1);
                (   
                    root.point.0.clone(),
                    root.create_child_back().point.0.clone(), 
                    root.create_child_back().point.0.clone(),
                    root.create_child_back().point.0.clone(),
                    root.create_child_back().point.0.clone(),
                    root.create_child_back().point.0.clone(),
                )
           };
            print_node(component_mgr, &node1);
            print_node(component_mgr, &node2);
            print_node(component_mgr, &node3);
            print_node(component_mgr, &node4);
            print_node(component_mgr, &node5);

            {
                let mut node = component_mgr.get_node_mut(&root);
                let mut size = node.get_size_mut();
                size.set_width(500.0);
                size.set_height(500.0);
            }

            {
                let mut node = component_mgr.get_node_mut(&node1);
                let mut size = node.get_size_mut();
                size.set_width(100.0);
                size.set_height(100.0);
            }

            {
                let mut node = component_mgr.get_node_mut(&node2);
                {
                    let mut size = node.get_size_mut();
                    size.set_width(100.0);
                    size.set_height(100.0);
                }
                {
                    let mut transform = node.get_transform_mut();
                    transform.set_position(Vector3::new(100.0, 0.0, 0.0));
                }
                
            }

            {
                let mut node = component_mgr.get_node_mut(&node3);
                {
                    let mut size = node.get_size_mut();
                    size.set_width(100.0);
                    size.set_height(100.0);
                }
                {
                    let mut transform = node.get_transform_mut();
                    transform.set_position(Vector3::new(200.0, 0.0, 0.0));
                }
                
            }

            {
                let mut node = component_mgr.get_node_mut(&node4);
                {
                    let mut size = node.get_size_mut();
                    size.set_width(100.0);
                    size.set_height(100.0);
                }
                {
                    let mut transform = node.get_transform_mut();
                    transform.set_position(Vector3::new(100.0, 0.0, 0.0));
                    transform.set_position(Vector3::new(400.0, 0.0, 0.0));
                }
                
            }

            {
                let mut node = component_mgr.get_node_mut(&node5);
                {
                    let mut size = node.get_size_mut();
                    size.set_width(100.0);
                    size.set_height(100.0);
                }
                {
                    let mut transform = node.get_transform_mut();
                    transform.set_position(Vector3::new(0.0, 100.0, 0.0));
                }
                
            }
            println!("modify-----------------------------------------");
            print_node(component_mgr, &node1);
            print_node(component_mgr, &node2);
            print_node(component_mgr, &node3);
            print_node(component_mgr, &node4);
            print_node(component_mgr, &node5);

            let node2_qid = component_mgr.get_node_mut(&node2).get_qid().clone();
            component_mgr.get_node_mut(&root).remove_child(node2_qid);
            (root, node1, node2, node3, node4, node5)
        }
    };

    println!("modify run-----------------------------------------");
    world.run(());
    print_node(&world.component_mgr, &root);
    print_node(&world.component_mgr, &node1);
    print_node(&world.component_mgr, &node3);
    print_node(&world.component_mgr, &node4);
    print_node(&world.component_mgr, &node5);
}

#[cfg(test)]
fn print_node(mgr: &GuiComponentMgr, id: &usize) {
    let node = mgr.node._group.get(&id);
    let bound_box_data = mgr.node.bound_box_data._group.get(&node.bound_box_data);
    println!("nodeid: {}, bound_box_data:{:?}, bound_box_dirty: {}, bound_box: {}", id, bound_box_data, node.bound_box_dirty, node.bound_box);
}
