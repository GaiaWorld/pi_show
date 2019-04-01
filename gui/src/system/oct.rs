//八叉树系统, 现在主要为事件做索引

use std::cell::RefCell;
use std::rc::{Rc};
use std::ops::Deref;

use wcs::world::{System};
use wcs::component::{ComponentHandler, ModifyFieldEvent, CreateEvent, DeleteEvent};
use vecmap::VecMap;

use component::node::{RectSize};
use world::GuiComponentMgr;
use component::math::{Matrix4, Aabb3 as C_Aabb3};
use cg::{Aabb3, Vector4, Point3};

pub struct Oct(RefCell<OctImpl>);

impl Oct {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<Oct>{
        let r = Rc::new(Oct(RefCell::new(OctImpl::new())));
        component_mgr.node.extent._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<RectSize, ModifyFieldEvent, GuiComponentMgr>>)));
        component_mgr.node.extent._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<RectSize, DeleteEvent, GuiComponentMgr>>)));
        component_mgr.node.extent._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<RectSize, CreateEvent, GuiComponentMgr>>)));
        component_mgr.node.world_matrix._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Matrix4, ModifyFieldEvent, GuiComponentMgr>>)));
        r
    }
}

impl ComponentHandler<RectSize, ModifyFieldEvent, GuiComponentMgr> for Oct{
    fn handle(&self, event: &ModifyFieldEvent, _component_mgr: &mut GuiComponentMgr){
        let ModifyFieldEvent{id: _, parent, field: _} = event;
        self.0.borrow_mut().marked_dirty(*parent);
    }
}

impl ComponentHandler<RectSize, CreateEvent, GuiComponentMgr> for Oct{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut GuiComponentMgr){
        let CreateEvent{id: _, parent} = event;
        let mut borrow = self.0.borrow_mut();
        borrow.dirty_mark_list.insert(*parent, false);
        borrow.add_aabb(*parent, component_mgr);
        borrow.marked_dirty(*parent);
    }
}

impl ComponentHandler<RectSize, DeleteEvent, GuiComponentMgr> for Oct{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut GuiComponentMgr){
        let DeleteEvent{id: _, parent} = event;
        let mut borrow = self.0.borrow_mut();
        borrow.remove_aabb(*parent, component_mgr);
        borrow.delete_dirty(*parent);
    }
}

//监听了Matrix组件的修改
impl ComponentHandler<Matrix4, ModifyFieldEvent, GuiComponentMgr> for Oct{
    fn handle(&self, event: &ModifyFieldEvent, _component_mgr: &mut GuiComponentMgr){
        let ModifyFieldEvent{id: _, parent, field: _} = event;
        self.0.borrow_mut().marked_dirty(*parent);
    }
}

impl System<(), GuiComponentMgr> for Oct{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        self.0.borrow_mut().cal_bound_box(component_mgr);
    }
}

pub struct OctImpl {
    dirtys: Vec<usize>, //Vec<node_id>
    dirty_mark_list: VecMap<bool>,
}

impl OctImpl {
    pub fn new() -> OctImpl{
        OctImpl{
            dirtys: Vec::new(),
            dirty_mark_list: VecMap::new(),
        }
    }

    //计算包围盒
    pub fn cal_bound_box(&mut self, mgr: &mut GuiComponentMgr){
        for node_id in self.dirtys.iter() {
            unsafe{*self.dirty_mark_list.get_unchecked_mut(*node_id) = false};

            let aabb = {
                //计算包围盒
                let node = mgr.node._group.get(*node_id);
                let extent = mgr.node.extent._group.get(node.extent);
                let world_matrix = mgr.node.world_matrix._group.get(mgr.node._group.get(*node_id).world_matrix);
                let aabb = cal_bound_box(extent, world_matrix);
                //更新八叉树
                mgr.octree.update(node.bound_box_id, aabb.clone());
                aabb
            };
            {
                //修改包围盒
                let mut node_ref = mgr.get_node_mut(*node_id);
                node_ref.get_bound_box_mut().modify(|aabb3: &mut C_Aabb3|{
                    aabb3.min = aabb.min;
                    aabb3.max = aabb.max;
                    true
                });
            }
        }
        self.dirtys.clear();
    }

    pub fn marked_dirty(&mut self, node_id: usize){
        let dirty_mark = unsafe{self.dirty_mark_list.get_unchecked_mut(node_id)};
        if *dirty_mark == true {
            return;
        }
        *dirty_mark = true;

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
        let node = mgr.node._group.get_mut(node_id);
        let aabb = mgr.node.bound_box._group.get_mut(node.bound_box).owner.clone();
        node.bound_box_id = mgr.octree.add(aabb.0, node_id);
    }

    pub fn remove_aabb(&mut self, node_id: usize, mgr: &mut GuiComponentMgr){
        let node = mgr.node._group.get_mut(node_id);
        if node.bound_box > 0 {
            mgr.octree.remove(node.bound_box_id);
        }
    }
}

fn cal_bound_box(size: &RectSize, matrix: &Matrix4) -> Aabb3<f32>{
    // let half_width = size.width/2.0;
    // let half_height = size.height/2.0;
    // let left_top = matrix.deref() * Vector4::new(-half_width, -half_height, 0.0, 1.0);
    // let right_top = matrix.deref() * Vector4::new(size.width-half_width, -half_height, 0.0, 1.0);
    // let left_bottom = matrix.deref() * Vector4::new(-half_width, size.height - half_height, 0.0, 1.0);
    // let right_bottom = matrix.deref() * Vector4::new(size.width - half_width, size.height - half_width, 0.0, 1.0);
    let left_top = matrix.deref() * Vector4::new(0.0, 0.0, 0.0, 1.0);
    let right_top = matrix.deref() * Vector4::new(size.width, 0.0, 0.0, 1.0);
    let left_bottom = matrix.deref() * Vector4::new(0.0, size.height, 0.0, 1.0);
    let right_bottom = matrix.deref() * Vector4::new(size.width, size.height, 0.0, 1.0);

    // let min = Point3::new(
    //     left_top.x.min(right_top.x).min(left_bottom.x).min(right_bottom.x) + half_width,
    //     left_top.y.min(right_top.y).min(left_bottom.y).min(right_bottom.y) + half_height,
    //     0.0,
    // );

    // let max = Point3::new(
    //     left_top.x.max(right_top.x).max(left_bottom.x).max(right_bottom.x) + half_width,
    //     left_top.y.max(right_top.y).max(left_bottom.y).max(right_bottom.y) + half_height,
    //     1.0,
    // );
    // let x = matrix
    // console::log_5(&("matrix".into()), );
    let min = Point3::new(
        left_top.x.min(right_top.x).min(left_bottom.x).min(right_bottom.x),
        left_top.y.min(right_top.y).min(left_bottom.y).min(right_bottom.y),
        0.0,
    );

    let max = Point3::new(
        left_top.x.max(right_top.x).max(left_bottom.x).max(right_bottom.x),
        left_top.y.max(right_top.y).max(left_bottom.y).max(right_bottom.y),
        1.0,
    );

    Aabb3::new(min, max)
}

#[cfg(test)]
#[cfg(not(feature = "web"))]
mod test {
    use std::rc::Rc;

    use wcs::component::Builder;
    use wcs::world::{World, System};
    use cg::{Aabb3, Point3};

    use component::node::{NodeBuilder, InsertType};
    use component::node::{RectSize};
    use world::GuiComponentMgr;
    use system::oct::Oct;

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

        world.component_mgr.set_size(500.0, 500.0);
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
            let bound_box_id = world.component_mgr.node._group.get(*i).bound_box;
            let bound_box = world.component_mgr.node.bound_box._group.get(bound_box_id);
            println!("test_bound_box1, node{} , bound_box:{:?}", i, bound_box);
        }

        world.component_mgr.get_node_mut(root).get_extent_mut().modify(|t: &mut RectSize| {
            t.width = 100.0;
            t.height = 100.0;
            true
        });
        
        world.run(());
        println!("-----------------------------------------------------------------");
        for i in node_ids.iter(){
            let bound_box_id = world.component_mgr.node._group.get(*i).bound_box;
            let bound_box = world.component_mgr.node.bound_box._group.get(bound_box_id);
            println!("test_bound_box2, node{} , bound_box:{:?}", i, bound_box);
        }

        //修改node2的extent
        world.component_mgr.get_node_mut(node_ids[6]).get_extent_mut().modify(|t: &mut RectSize| {
            t.width = 100.0;
            t.height = 100.0;
            true
        });
        world.run(());
        println!("-----------------------------------------------------------------");
        for i in node_ids.iter(){
            let bound_box_id = world.component_mgr.node._group.get(*i).bound_box;
            let bound_box = world.component_mgr.node.bound_box._group.get(bound_box_id);
            println!("test_bound_box3, node{} , bound_box:{:?}", i, bound_box);
        }
    }

    #[cfg(test)]
    fn new_world() -> World<GuiComponentMgr, ()>{
        let mut world: World<GuiComponentMgr, ()> = World::new(GuiComponentMgr::new());
        let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![Oct::init(&mut world.component_mgr, Aabb3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1000.0, 1000.0, 1000.0)))];
        world.set_systems(systems);
        world
    }
}