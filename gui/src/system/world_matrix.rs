use std::cell::RefCell;
use std::rc::{Rc};

use wcs::world::{System, ID, ComponentMgr};

use components::{NodePoint, GuiComponentMgr, NodeGroup};
use alert;

pub struct WorldMatrix(RefCell<WorldMatrixImpl>);

impl System<(), GuiComponentMgr> for WorldMatrix{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        self.0.borrow_mut().cal_matrix(&mut component_mgr.node.borrow_mut());
    }

    fn init(_component_mgr: &mut GuiComponentMgr) -> Rc<WorldMatrix>{
        Rc::new(WorldMatrix(RefCell::new(WorldMatrixImpl::new())))
    }
}

pub struct WorldMatrixImpl;

impl WorldMatrixImpl {
    pub fn new() -> WorldMatrixImpl{
        WorldMatrixImpl{
}
    }

    //计算世界矩阵
    pub fn cal_matrix<M: ComponentMgr>(&mut self, node_group: &mut NodeGroup<M>){
        for (index, node) in node_group._group.iter_mut() {
            let position = node.owner.yoga_node.get_computed_position();
            let mut p = NodePoint::default();
            p.set_id(index);

            if node.position.x != position.x || node.position.y != position.y {
                node.position = position;
                //计算世界矩阵 TODO
            }

            //更新包围盒
            {
                let size = node.owner.yoga_node.get_computed_size();
                if node.bound_box.0 != 0 {
                    let mut borrow_mut = node_group.bound_box.borrow_mut();
                    
                    if node.bound_box.get_width(&borrow_mut) != &size.x {
                        node.bound_box.set_width(size.x, &mut borrow_mut);
                    }
                    if node.bound_box.get_height(&mut borrow_mut)!= &size.y{
                        node.bound_box.set_height(size.y, &mut borrow_mut);
                    }
                }
            }
            
        }
    }
}

