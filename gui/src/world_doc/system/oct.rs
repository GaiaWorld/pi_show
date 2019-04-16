//八叉树系统, 现在主要为事件做索引

use std::cell::RefCell;
use std::rc::{Rc};
use std::ops::Deref;

use wcs::world::{System};
use wcs::component::{ComponentHandler, ModifyFieldEvent, CreateEvent, DeleteEvent};
use vecmap::VecMap;

use world_doc::WorldDocMgr;
use world_doc::component::node::Node;
use component::math::{Matrix4, Aabb3 as C_Aabb3, Vector2};
use cg::{Aabb3, Vector4, Point3};

pub struct Oct(RefCell<OctImpl>);

impl Oct {
    pub fn init(component_mgr: &mut WorldDocMgr) -> Rc<Oct>{
        let r = Rc::new(Oct(RefCell::new(OctImpl::new())));
        component_mgr.node.layout.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, DeleteEvent, WorldDocMgr>>)));
        component_mgr.node._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, CreateEvent, WorldDocMgr>>)));
        component_mgr.node.world_matrix._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Matrix4, ModifyFieldEvent, WorldDocMgr>>)));
        r
    }
}

//监听layout的改变，设脏
impl ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr> for Oct{
    fn handle(&self, event: &ModifyFieldEvent, _component_mgr: &mut WorldDocMgr){
        let ModifyFieldEvent{id, parent: _, field: _} = event;
        // println!("marked_dirty111111111111111ModifyFieldEvent11{}", id);
        self.0.borrow_mut().marked_dirty(*id);
    }
}

// 监听Node的创建事件， 设脏
impl ComponentHandler<Node, CreateEvent, WorldDocMgr> for Oct{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut WorldDocMgr){
        let CreateEvent{id, parent : _} = event;
        let mut borrow = self.0.borrow_mut();
        borrow.dirty_mark_list.insert(*id, false);
        borrow.add_aabb(*id, component_mgr);
        borrow.marked_dirty(*id);
    }
}

// 监听Node的删除事件， 设脏
impl ComponentHandler<Node, DeleteEvent, WorldDocMgr> for Oct{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut WorldDocMgr){
        let DeleteEvent{id, parent : _} = event;
        let mut borrow = self.0.borrow_mut();
        borrow.remove_aabb(*id, component_mgr);
        borrow.delete_dirty(*id);
    }
}

//监听了Matrix组件的修改, 设脏
impl ComponentHandler<Matrix4, ModifyFieldEvent, WorldDocMgr> for Oct{
    fn handle(&self, event: &ModifyFieldEvent, _component_mgr: &mut WorldDocMgr){
        let ModifyFieldEvent{id: _, parent, field: _} = event;
        // println!("marked_dirty111111111111111ModifyFieldEvent11{}", parent);
        self.0.borrow_mut().marked_dirty(*parent);
    }
}

impl System<(), WorldDocMgr> for Oct{
    fn run(&self, _e: &(), mgr: &mut WorldDocMgr){
        self.0.borrow_mut().cal_bound_box(mgr);
        mgr.octree.collect();
    }
}

pub struct OctImpl {
    dirtys: Vec<usize>, //Vec<node_id>
    dirty_mark_list: VecMap<bool>,
}

impl OctImpl {
    pub fn new() -> OctImpl{
        let mut dirty_mark_list = VecMap::new();
        let mut dirtys = Vec::new();
        dirtys.push(1);
        dirty_mark_list.insert(1, true);

        OctImpl{
            dirtys,
            dirty_mark_list,
        }
    }

    //计算包围盒
    pub fn cal_bound_box(&mut self, mgr: &mut WorldDocMgr){
        for node_id in self.dirtys.iter() {
            unsafe{*self.dirty_mark_list.get_unchecked_mut(*node_id) = false};

            let (aabb, size) = {
                //计算包围盒
                let node = mgr.node._group.get(*node_id);
                let layout = &node.layout;
                let world_matrix = mgr.node.world_matrix._group.get(mgr.node._group.get(*node_id).world_matrix);
                let (aabb, size) = cal_bound_box((layout.width, layout.height), world_matrix);
                //更新八叉树
                println!("update_octree------bound_box_id: {:?}, aabb: {:?}", node.bound_box_id, aabb);
                mgr.octree.update(node.bound_box_id, aabb.clone());
                (aabb, size)
            };
            {
                //修改包围盒
                let mut node_ref = mgr.get_node_mut(*node_id);
                node_ref.get_bound_box_mut().modify(|aabb3: &mut C_Aabb3|{
                    aabb3.min = aabb.min;
                    aabb3.max = aabb.max;
                    true
                });
                node_ref.set_size(Vector2::new(size.0, size.1))
            }
        }
        self.dirtys.clear();
    }

    pub fn marked_dirty(&mut self, node_id: usize){
        // println!("marked_dirty11111111111111111111111{}", node_id);
        let dirty_mark = unsafe{self.dirty_mark_list.get_unchecked_mut(node_id)};
        // println!("marked_dirty11111111111111111111111c");
        if *dirty_mark == true {
            return;
        }
        *dirty_mark = true;

        self.dirtys.push(node_id);
        // println!("marked_dirty11111111111111111111111e");
    }

    pub fn delete_dirty(&mut self, node_id: usize){
        for i in 0..self.dirtys.len(){
            if self.dirtys[i] == node_id{
                self.dirtys.remove(i);
                return;
            }
        }
    }

    pub fn add_aabb(&mut self, node_id: usize, mgr: &mut WorldDocMgr){
        let node = mgr.node._group.get_mut(node_id);
        let aabb = mgr.node.bound_box._group.get_mut(node.bound_box).owner.clone();
        println!("add_octree------{:?}, node_id: {}", aabb.0, node_id);
        node.bound_box_id = mgr.octree.add(aabb.0, node_id);
        
    }

    pub fn remove_aabb(&mut self, node_id: usize, mgr: &mut WorldDocMgr){
        let node = mgr.node._group.get_mut(node_id);
        if node.bound_box > 0 {
            mgr.octree.remove(node.bound_box_id);
        }
    }
}

fn cal_bound_box(size: (f32, f32), matrix: &Matrix4) -> (Aabb3<f32>, (f32, f32)){
    let half_width = size.0/2.0;
    let half_height = size.1/2.0;
    // let left_top = matrix.deref() * Vector4::new(-half_width, -half_height, 0.0, 1.0);
    // let right_top = matrix.deref() * Vector4::new(size.width-half_width, -half_height, 0.0, 1.0);
    // let left_bottom = matrix.deref() * Vector4::new(-half_width, size.height - half_height, 0.0, 1.0);
    // let right_bottom = matrix.deref() * Vector4::new(size.width - half_width, size.height - half_width, 0.0, 1.0);
    let left_top = matrix.deref() * Vector4::new(-half_width, -half_height, 0.0, 1.0);
    let right_top = matrix.deref() * Vector4::new(half_width, -half_height, 0.0, 1.0);
    let left_bottom = matrix.deref() * Vector4::new(-half_width, half_height, 0.0, 1.0);
    let right_bottom = matrix.deref() * Vector4::new(half_width, half_height, 0.0, 1.0);

    let size_x = right_top.x - left_top.x;
    let size_y = left_bottom.y - left_top.y;

    // println!("box--------------------------size_x: {}, size_y: {}, width: {}, heigth: {}", size_x, size_y, size.0, size.1);

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

    (Aabb3::new(min, max), (size_x, size_y))
}

#[cfg(test)]
#[cfg(not(feature = "web"))]
mod test {
    use std::rc::Rc;

    use wcs::component::Builder;
    use wcs::world::{World, System};

    use world_doc::component::node::{NodeBuilder, InsertType};
    use world_doc::component::node::{RectSize};
    use world_doc::WorldDocMgr;
    use world_doc::system::oct::Oct;

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
            // println!("test_bound_box1, node{} , bound_box:{:?}", i, bound_box);
        }

        world.component_mgr.get_node_mut(root).get_extend_mut().modify(|t: &mut RectSize| {
            t.width = 100.0;
            t.height = 100.0;
            true
        });
        
        world.run(());
        for i in node_ids.iter(){
            let bound_box_id = world.component_mgr.node._group.get(*i).bound_box;
            let bound_box = world.component_mgr.node.bound_box._group.get(bound_box_id);
            // println!("test_bound_box2, node{} , bound_box:{:?}", i, bound_box);
        }

        //修改node2的extend
        world.component_mgr.get_node_mut(node_ids[6]).get_extend_mut().modify(|t: &mut RectSize| {
            t.width = 100.0;
            t.height = 100.0;
            true
        });
        world.run(());
        // println!("-----------------------------------------------------------------");
        for i in node_ids.iter(){
            let bound_box_id = world.component_mgr.node._group.get(*i).bound_box;
            let bound_box = world.component_mgr.node.bound_box._group.get(bound_box_id);
            // println!("test_bound_box3, node{} , bound_box:{:?}", i, bound_box);
        }
    }

    #[cfg(test)]
    fn new_world() -> World<WorldDocMgr, ()>{
        let mut world: World<WorldDocMgr, ()> = World::new(WorldDocMgr::new());
        let systems: Vec<Rc<System<(), WorldDocMgr>>> = vec![Oct::init(&mut world.component_mgr)];
        world.set_systems(systems);
        world
    }
}