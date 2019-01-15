use std::cell::RefCell;
use std::rc::{Rc};

use wcs::world::{System};
use wcs::component::{ComponentHandler, Event};

use component::component_def::{NodePoint, GuiComponentMgr, TransformPoint};
use component::math::Matrix4;
// use alert;

pub struct WorldMatrix(RefCell<WorldMatrixImpl>);

impl WorldMatrix {
    pub fn init(_component_mgr: &mut GuiComponentMgr) -> Rc<WorldMatrix>{
        Rc::new(WorldMatrix(RefCell::new(WorldMatrixImpl::new())))
    }
}

impl ComponentHandler<TransformPoint, GuiComponentMgr> for WorldMatrix{
    fn handle(&self, event: &Event<TransformPoint>, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::ModifyField{point: _, parent, field: _} => {
                self.0.borrow_mut().marked_dirty(parent.clone(), component_mgr);
            },
            Event::Create{point: _, parent} => {
                self.0.borrow_mut().marked_dirty(parent.clone(), component_mgr);
            },
            Event::Delete{point, parent: _} => {
                self.0.borrow_mut().delete_dirty(&point);
            },
            _ => {
                unreachable!();
            }
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
        for d1 in self.dirtys.iter() {
            for node_point in d1.iter() {
                /**********************************更新世界矩阵*********************************************/
                // 设置脏标志
                {
                    let node = component_mgr.node._group.get_mut(&node_point);
                    if node.world_matrix_dirty == false {
                        continue;
                    }
                    node.world_matrix_dirty = false;
                }

                //计算世界矩阵(应该递归计算并修改子节点的世界矩阵， TODO)
                let world_matrix = {
                    let transform_point = (*component_mgr.get_node_mut(node_point).get_transform()).clone();
                    let transform = component_mgr.node.transform._group.get(&transform_point);
                    transform.matrix()
                };

                let mut node_ref = component_mgr.get_node_mut(node_point);
                //修改世界矩阵
                let mut world_matrix_ref = node_ref.get_world_matrix_mut();
                world_matrix_ref.modify(|matrix: &mut Matrix4|{
                    matrix.x = world_matrix.x;
                    matrix.y = world_matrix.y;
                    matrix.z = world_matrix.z;
                    matrix.w = world_matrix.w;
                    true
                });
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
            node.world_matrix_dirty = false;
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

