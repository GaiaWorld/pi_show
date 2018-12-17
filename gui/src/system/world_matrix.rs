use std::cell::RefCell;
use std::rc::{Rc, Weak};

use wcs::world::{System, ID};
use wcs::component::{EventType, PPoint, ComponentHandler};

use components::{Node, NodePoint, GuiComponentMgr, LayoutPoint, NodeGroups};

pub struct WorldMatrix(RefCell<WorldMatrixImpl>);

impl ComponentHandler<LayoutPoint, GuiComponentMgr> for WorldMatrix{
    fn handle(&self, event: EventType<LayoutPoint>, component_mgr: &mut GuiComponentMgr){
        println!("handle layout-----------------------------");
        // match event {
        //     EventType::ModifyField(p, _feild_name) => {
        //         let node = component_mgr.node._group.get_mut(p.parent.clone());
        //         self.0.borrow_mut().marked_dirty(node);
        //     },
        //     _ => {
        //         unreachable!();
        //     }
        // }
    }
}

impl System<(), GuiComponentMgr> for WorldMatrix{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        self.0.borrow_mut().cal_matrix(&mut component_mgr.node.borrow_mut());
    }

    fn init(component_mgr: &mut GuiComponentMgr) -> Rc<WorldMatrix>{
        let share = Rc::new(WorldMatrix(RefCell::new(WorldMatrixImpl::new())));
        let node = component_mgr.node.borrow_mut();
        node.layout.borrow_mut()._group.register_handlers(Rc::downgrade(&share) as Weak<ComponentHandler<LayoutPoint, GuiComponentMgr>>);
        share
    }
}

pub struct WorldMatrixImpl{
    dirts: Vec<Vec<NodePoint>>
}

impl WorldMatrixImpl {
    pub fn new() -> WorldMatrixImpl{
        WorldMatrixImpl{
            dirts: Vec::new()
        }
    }

    pub fn marked_dirty(&mut self, node: &mut (Node, PPoint<NodePoint>)){
        println!("marked_dirty-----------------------------");
        if !node.0.layout_dirt{
            node.0.layout_dirt = true;
            if self.dirts.len() <= node.0.layer{
                for _i in 0..node.0.layer - self.dirts.len() + 1{
                    self.dirts.push(Vec::new());
                }
            }
            unsafe{ self.dirts.get_unchecked_mut(node.0.layer).push(node.1.id.clone()) };
        }
    }

    //计算世界矩阵
    pub fn cal_matrix(&mut self, node_group: &mut NodeGroups){
        println!("cal_matrix-----------------------------");
        for layer in self.dirts.iter_mut(){
            for point in layer.iter_mut(){
                WorldMatrixImpl::recursion_cal_matrix(point, node_group);
            }
            layer.clear();
        }
    }

    //计算世界矩阵
    fn recursion_cal_matrix(node: &mut NodePoint, node_group: &mut NodeGroups){
        // if node.get_layout_dirt(node_group) == &true {
        //     node.set_layout_dirt(false, node_group);
        //     //计算， TODO
        //     for child in node.get_childs_mut(node_group).clone().iter_mut(){
        //         WorldMatrixImpl::recursion_cal_matrix(child, node_group);
        //     }
        // }
    }
}

