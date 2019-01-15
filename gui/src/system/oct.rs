use std::cell::RefCell;
use std::rc::{Rc};

use wcs::world::{System};
use wcs::component::{ComponentHandler, Event, notify};
use cg::octree::*;

use component::component_def::{NodePoint, GuiComponentMgr, Matrix4Point,SizePoint, Size};
use component::math::{Aabb3, Matrix4, Vector4, Point3};
// use alert;

pub struct Oct(RefCell<OctImpl>);

impl Oct {
    pub fn init(component_mgr: &mut GuiComponentMgr, size: Aabb3) -> Rc<Oct>{
        let r = Rc::new(Oct(RefCell::new(OctImpl::new(size))));
        component_mgr.node.size._group.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<SizePoint, GuiComponentMgr>>)));
        r
    }
}

impl ComponentHandler<SizePoint, GuiComponentMgr> for Oct{
    fn handle(&self, event: &Event<SizePoint>, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::ModifyField{point, parent: _, field: _} => {
                self.0.borrow_mut().marked_dirty_by_size(&point, component_mgr);
            },
            _ => ()
        }
    }
}

impl ComponentHandler<Matrix4Point, GuiComponentMgr> for Oct{
    fn handle(&self, event: &Event<Matrix4Point>, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::ModifyField{point, parent: _, field: _} => {
                self.0.borrow_mut().marked_dirty_by_matrix(&point, component_mgr);
            },
            _ => ()
        }
    }
}

impl ComponentHandler<NodePoint, GuiComponentMgr> for Oct{
    fn handle(&self, event: &Event<NodePoint>, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::Create{point, parent: _} => {
                self.0.borrow_mut().add_aabb(&point, component_mgr);
            },
            Event::Delete{point, parent: _} => {
                self.0.borrow_mut().remove_aabb(&point, component_mgr);
                self.0.borrow_mut().delete_dirty(&point);
            },
            _ => {
                unreachable!();
            }
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

            {
                //计算包围盒
                let size = mgr.node.size._group.get(&mgr.node._group.get(node_point).size);
                let world_matrix = mgr.node.world_matrix._group.get(&mgr.node._group.get(node_point).world_matrix);
                let aabb = cal_bound_box(size, world_matrix);

                //更新八叉树
                self.octree.update(node_point.get_bound_box(&mgr.node).clone(), aabb);
            }

            let handlers = mgr.node._group.get_handlers();
            //通知包围盒发生改变
            notify(Event::ModifyField{point: node_point.clone(), parent:0, field: "bound_box"}, &handlers.borrow(), mgr);
        }
    }

    pub fn marked_dirty_by_size(&mut self, size: &SizePoint, mgr: &mut GuiComponentMgr){
        let node_point = {
            let size = mgr.node.size._group.get_mut(size);
            NodePoint(size.parent)
        };

        {
            let node = mgr.node._group.get_mut(&node_point);
            if node.bound_box_dirty == true {
                return;
            }
            node.bound_box_dirty = true;
        }

        self.dirtys.push(node_point.clone());
    }

    pub fn marked_dirty_by_matrix(&mut self, world_matrix: &Matrix4Point, mgr: &mut GuiComponentMgr){
        let node_point = NodePoint(mgr.node.world_matrix._group.get_mut(world_matrix).parent);

        {
            let node = mgr.node._group.get_mut(&node_point);
            if node.bound_box_dirty == true {
                return;
            }
            node.bound_box_dirty = true;
        }

        self.dirtys.push(node_point);
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
        let aabb_index = {
            let (size_point, matrix_point) = {
                let node = mgr.node._group.get_mut(node_point);
                (node.size.clone(), node.world_matrix.clone())
            };
            let aabb = cal_bound_box(&mgr.node.size._group.get_mut(&size_point).owner, &mgr.node.world_matrix._group.get_mut(&matrix_point).owner);
            self.octree.add(aabb, node_point.0)
        };
        mgr.get_node_mut(node_point).set_bound_box(aabb_index);
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