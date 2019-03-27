/**
 * 监听transform和position组件， 利用transform和position递归计算节点的世界矩阵（worldmatrix组件）
 */

use std::cell::RefCell;
use std::rc::{Rc};

use wcs::world::{System};
use wcs::component::{ComponentHandler, ModifyFieldEvent, CreateEvent, DeleteEvent};

use component::style::transform::{Transform};
use component::node::Node;
use world::GuiComponentMgr;
use component::math::{Matrix4, Vector3};
use vecmap::VecMap;
// use alert;

pub struct WorldMatrix(RefCell<WorldMatrixImpl>);

impl WorldMatrix {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<WorldMatrix>{
        let system = Rc::new(WorldMatrix(RefCell::new(WorldMatrixImpl::new())));
        component_mgr.node.transform._group.register_modify_field_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Transform, ModifyFieldEvent, GuiComponentMgr>>)));
        component_mgr.node.transform._group.register_delete_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Transform, DeleteEvent, GuiComponentMgr>>)));
        component_mgr.node.transform._group.register_create_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Transform, CreateEvent, GuiComponentMgr>>)));
        component_mgr.node.position._group.register_modify_field_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Vector3, ModifyFieldEvent, GuiComponentMgr>>)));
        component_mgr.node._group.register_create_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, CreateEvent, GuiComponentMgr>>)));
        system
    }
}

impl ComponentHandler<Transform, ModifyFieldEvent, GuiComponentMgr> for WorldMatrix{
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut GuiComponentMgr){
        let ModifyFieldEvent{id: _, parent, field: _} = event;
        self.0.borrow_mut().marked_dirty(*parent, component_mgr);
    }
}

impl ComponentHandler<Transform, DeleteEvent, GuiComponentMgr> for WorldMatrix{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut GuiComponentMgr){
        let DeleteEvent{id: _, parent} = event;
        self.0.borrow_mut().delete_dirty(*parent, component_mgr);
        //不需要从dirty_mark_list中删除
    }
}

impl ComponentHandler<Transform, CreateEvent, GuiComponentMgr> for WorldMatrix{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut GuiComponentMgr){
        let CreateEvent{id: _, parent} = event;
        let mut borrow = self.0.borrow_mut();
        borrow.marked_dirty(*parent, component_mgr);
    }
}

impl ComponentHandler<Node, CreateEvent, GuiComponentMgr> for WorldMatrix{
    fn handle(&self, event: &CreateEvent, _component_mgr: &mut GuiComponentMgr){
        let CreateEvent{id, parent: _} = event;
        self.0.borrow_mut().dirty_mark_list.insert(*id, false);
    }
}

impl ComponentHandler<Vector3, ModifyFieldEvent, GuiComponentMgr> for WorldMatrix{
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut GuiComponentMgr){
        let ModifyFieldEvent{id: _, parent, field: _} = event;
        self.0.borrow_mut().marked_dirty(*parent, component_mgr);
    }
}

impl System<(), GuiComponentMgr> for WorldMatrix{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        self.0.borrow_mut().cal_matrix(component_mgr);
    }
}

pub struct WorldMatrixImpl {
    dirtys: Vec<Vec<usize>>, //Vec<Vec<node_id>>
    dirty_mark_list: VecMap<bool>,
}

impl WorldMatrixImpl {
    pub fn new() -> WorldMatrixImpl{
        WorldMatrixImpl{
            dirtys: Vec::new(),
            dirty_mark_list: VecMap::new()
        }
    }

    //计算世界矩阵
    pub fn cal_matrix(&mut self, component_mgr: &mut GuiComponentMgr){
        for d1 in self.dirtys.iter() {
            for node_id in d1.iter() {
                let dirty_mark = unsafe{*self.dirty_mark_list.get_unchecked(*node_id)};
                if dirty_mark == false {
                    continue;
                }
                //修改节点世界矩阵及子节点的世界矩阵
                modify_matrix(&mut self.dirty_mark_list, *node_id, component_mgr);
            }
        }
        self.dirtys.clear();
    }

    pub fn marked_dirty(&mut self, node_id: usize, mgr: &mut GuiComponentMgr){
        {
            let dirty_mark = unsafe { self.dirty_mark_list.get_unchecked_mut(node_id) };
            if *dirty_mark == true {
                return;
            }
            *dirty_mark = true;
        }
        let layer = mgr.node._group.get_mut(node_id).layer;

        if self.dirtys.len() <= layer{
            for _i in 0..(layer + 1 - self.dirtys.len()){
                self.dirtys.push(Vec::new());
            }
        }
        self.dirtys[layer].push(node_id);
    }

    pub fn delete_dirty(&mut self, node_id: usize, mgr: &mut GuiComponentMgr){
        let dirty_mark = *unsafe{self.dirty_mark_list.get_unchecked_mut(node_id)};
        if dirty_mark == true {
            let layer = mgr.node._group.get_mut(node_id).layer;
            for i in 0..self.dirtys[layer].len() {
                if self.dirtys[layer][i] == node_id {
                    self.dirtys[layer].swap_remove(i);
                    return;
                }
            }
        }
    }
}

//计算世界矩阵
fn modify_matrix(dirty_mark_list: &mut VecMap<bool>, node_id: usize, component_mgr: &mut GuiComponentMgr) {
    let mut world_matrix = {
        let (transform_id, position_id) = {
            let node = component_mgr.node._group.get(node_id);
            (node.transform, node.position)
        };
        let transform = match transform_id == 0 {
            true => Transform::default().matrix(),
            false => component_mgr.node.transform._group.get(transform_id).matrix(),
        };
        let p = component_mgr.node.position._group.get(position_id).owner.0;
        let position = cg::Matrix4::from_translation(p.clone());
        (transform * position)
    };

    let parent_id = component_mgr.node._group.get(node_id).parent;

    if parent_id != 0 {
        let parent_world_matrix = {
            let parent_world_matrix_id = component_mgr.node._group.get(parent_id).world_matrix;
            ***component_mgr.node.world_matrix._group.get(parent_world_matrix_id)
        };
        world_matrix = parent_world_matrix * world_matrix
    }

    let mut child = {
        let mut node_ref = component_mgr.get_node_mut(node_id);
        node_ref.get_world_matrix_mut().modify(|matrix: &mut Matrix4|{
            matrix.x = world_matrix.x;
            matrix.y = world_matrix.y;
            matrix.z = world_matrix.z;
            matrix.w = world_matrix.w;
            true
        });

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
        modify_matrix(dirty_mark_list, node_id, component_mgr);
    }
}

#[cfg(test)]
use wcs::component::Builder;
#[cfg(test)]
use wcs::world::{World};

#[cfg(test)]
use component::node::{NodeBuilder, InsertType};

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
        {
            let world_matrix_id = world.component_mgr.node._group.get(*i).world_matrix;
            let world_matrix = world.component_mgr.node.world_matrix._group.get(world_matrix_id);
            println!("test_world_matrix1, node{} , world_matrix:{:?}", i, world_matrix);
        }
    }
    {
        world.component_mgr.get_node(root);
        let transform_id = *(world.component_mgr.get_node(root).get_transform());
        if transform_id == 0 {
            let mut transform = Transform::default();
            transform.position = Vector3(cg::Vector3::new(1.0, 2.0, 3.0));
            world.component_mgr.get_node_mut(root).set_transform(transform);
        }else {
            world.component_mgr.get_node_mut(root).get_transform_mut().modify(|t: &mut Transform| {
                t.position = Vector3(cg::Vector3::new(1.0, 2.0, 3.0));
                true
            });
        }
    }
    
    world.run(());
    println!("-----------------------------------------------------------------");
    for i in node_ids.iter(){
        {
            let world_matrix_id = world.component_mgr.node._group.get(*i).world_matrix;
            let world_matrix = world.component_mgr.node.world_matrix._group.get(world_matrix_id);
            println!("test_world_matrix2, node{} , world_matrix:{:?}", i, world_matrix);
        }
    }

    //修改node2的position
    world.component_mgr.get_node_mut(node_ids[0]).get_position_mut().modify(|t: &mut Vector3| {
        t.x = 1.0;
        t.y = 2.0;
        t.z = 3.0;
        true
    });
    world.run(());
    println!("-----------------------------------------------------------------");
    for i in node_ids.iter(){
        {
            let world_matrix_id = world.component_mgr.node._group.get(*i).world_matrix;
            let world_matrix = world.component_mgr.node.world_matrix._group.get(world_matrix_id);
            println!("test_world_matrix3, node{} , world_matrix:{:?}", i, world_matrix);
        }
    }
}

#[cfg(test)]
fn new_world() -> World<GuiComponentMgr, ()>{
    let mut world: World<GuiComponentMgr, ()> = World::new(GuiComponentMgr::new());
    let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![WorldMatrix::init(&mut world.component_mgr)];
    world.set_systems(systems);
    world
}