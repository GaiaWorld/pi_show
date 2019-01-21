use std::cell::RefCell;
use std::rc::{Rc};

use wcs::world::{System};
use wcs::component::{ComponentHandler, Event};

use component::component_def::{NodePoint, GuiComponentMgr, TransformPoint};
use component::math::Matrix4;
// use alert;

pub struct WorldMatrix(RefCell<WorldMatrixImpl>);

impl WorldMatrix {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<WorldMatrix>{
        let system = Rc::new(WorldMatrix(RefCell::new(WorldMatrixImpl::new())));
        component_mgr.node.transform._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<TransformPoint, GuiComponentMgr>>)));
        component_mgr.node._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<NodePoint, GuiComponentMgr>>)));
        system
    }
}

impl ComponentHandler<NodePoint, GuiComponentMgr> for WorldMatrix{
    fn handle(&self, event: &Event<NodePoint>, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::ModifyField{point, parent: _, field} => {
                if field != &"layer"{
                    return;
                }
                 println!("ModifyField layer-----------------------");
                self.0.borrow_mut().marked_dirty(point.0.clone(), component_mgr);
            },
            _ => ()
        }
    }
}

impl ComponentHandler<TransformPoint, GuiComponentMgr> for WorldMatrix{
    fn handle(&self, event: &Event<TransformPoint>, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::ModifyField{point: _, parent, field: _} => {
                println!("ModifyField TransformPoint-----------------------");
                self.0.borrow_mut().marked_dirty(parent.clone(), component_mgr);
            },
            //不监听创建， 该系统以来layer， 创建Transform可能还没有初始化layer
            // Event::Create{point: _, parent} => {
            //     println!("Create TransformPoint-----------------------");
            //     self.0.borrow_mut().marked_dirty(parent.clone(), component_mgr);
            // },
            Event::Delete{point, parent: _} => {
                println!("Delete TransformPoint-----------------------");
                self.0.borrow_mut().delete_dirty(&point);
            },
            _ => ()
        }
    }
}

impl System<(), GuiComponentMgr> for WorldMatrix{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        self.0.borrow_mut().cal_matrix(component_mgr);
    }
}

pub struct WorldMatrixImpl {
    dirtys: Vec<Vec<NodePoint>>,
}

impl WorldMatrixImpl {
    pub fn new() -> WorldMatrixImpl{
        WorldMatrixImpl{
            dirtys: Vec::new()
        }
    }

    //计算世界矩阵
    pub fn cal_matrix(&mut self, component_mgr: &mut GuiComponentMgr){
        println!("self.dirtys----------------------{:?}", self.dirtys);
        for d1 in self.dirtys.iter() {
            for node_point in d1.iter() {
                //修改节点世界矩阵及子节点的世界矩阵
                modify_matrix(&node_point, component_mgr);
            }
        }

        //处理完脏列表，需要清空， 此处暂时不清空， TODO
    }

    pub fn marked_dirty(&mut self, node_index: usize, mgr: &mut GuiComponentMgr){
        let layer = {
            let node = mgr.node._group.get_mut(&node_index);
            if node.world_matrix_dirty == true {
                return;
            }
            node.world_matrix_dirty = true;
            if node.layer == 0 {
                return;
            }
            node.layer
        };

        if self.dirtys.len() < layer{
            for _i in 0..layer - self.dirtys.len(){
                self.dirtys.push(Vec::new());
            }
        }
        self.dirtys[layer - 1].push(NodePoint(node_index));
    }

    pub fn delete_dirty(&mut self, transfrom: &TransformPoint){
        for i in 0..self.dirtys.len(){
            for j in 0..self.dirtys[i].len(){
                if self.dirtys[i][j].0 == transfrom.0{
                    self.dirtys[i].remove(j);
                    return;
                }
            }
        }
    }
}

fn modify_matrix(node_point: &NodePoint, component_mgr: &mut GuiComponentMgr) {
    // 设置脏标志
    {
        let node = component_mgr.node._group.get_mut(&node_point);
        if node.world_matrix_dirty == false {
            return;
        }
        node.world_matrix_dirty = false;
    }

    //计算世界矩阵(应该递归计算并修改子节点的世界矩阵， TODO)
    let world_matrix = {
        let transform_point = (*component_mgr.get_node_mut(node_point).get_transform()).clone();
        let transform = component_mgr.node.transform._group.get(&transform_point);
        transform.matrix()
    };

    let mut child = {
        let mut node_ref = component_mgr.get_node_mut(node_point);
        let mut world_matrix_ref = node_ref.get_world_matrix_mut();
        world_matrix_ref.modify(|matrix: &mut Matrix4|{
            matrix.x = world_matrix.x;
            matrix.y = world_matrix.y;
            matrix.z = world_matrix.z;
            matrix.w = world_matrix.w;
            true
        });

        node_ref.get_childs_mut().get_first()
    };
    //递归计算子节点的世界矩阵
    loop {
        if child == 0 {
            return;
        }
        let node_point = {
            let v = unsafe{ component_mgr.node_container.get_unchecked(child) };
            child = v.next;
            v.elem.clone()
        };
        // println!("node_point-----------------------{:?}", node_point);
        modify_matrix(&node_point, component_mgr);
    }
}

#[cfg(test)]
use wcs::world::{World};
#[cfg(test)]
use component::math::{Vector3};
#[cfg(test)]
use component::component_def::{Children};

#[test]
fn test(){
    let mut world: World<GuiComponentMgr, ()> = World::new();
    let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![WorldMatrix::init(&mut world.component_mgr)];
    world.set_systems(systems);

    let (root, node1, node2, node3, node4, node5) = {
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
    print_node(&world.component_mgr, &node1);
    print_node(&world.component_mgr, &node2);
    print_node(&world.component_mgr, &node3);
    print_node(&world.component_mgr, &node4);
    print_node(&world.component_mgr, &node5);

    // root.remove_child

    // let mut s = "{".to_string();
    // s = s + "node_group: " + format!("{:?}", world.component_mgr.node._group).as_str();
    // s = s + "transform_group:" + format!("{:?}", world.component_mgr.node.transform._group).as_str();
    // s = s + "size_group:" + format!("{:?}", world.component_mgr.node.size._group).as_str();
    // s = s + "world_matrix_group:" + format!("{:?}", world.component_mgr.node.world_matrix._group).as_str();
    // s = s + "}";

    // //let arr = Array::new();
    // //let js_value = format_args!("{}", s).to_string().into();
    // //arr.push(&js_value);
    // // log(&format_args!("{}", s).to_string());
    // //console::log(&arr);
    // world
}

#[cfg(test)]
fn print_node(mgr: &GuiComponentMgr, id: &usize) {
    let node = mgr.node._group.get(&id);
    let transform = mgr.node.transform._group.get(&node.transform);
    let matrix = mgr.node.world_matrix._group.get(&node.world_matrix);

    println!("nodeid: {}, transform:{:?}, world_matrix: {:?}, matrix_dirty: {}", id, transform, matrix, node.world_matrix_dirty);
}




