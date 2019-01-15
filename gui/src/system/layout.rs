use std::cell::RefCell;
use std::rc::{Rc};

use wcs::world::{System};
use wcs::component::{ComponentHandler, Event};

use component::component_def::{NodePoint, GuiComponentMgr, Transform, Size };
// use alert;

pub struct Layout(RefCell<LayoutImpl>);

impl Layout {
    pub fn init(_component_mgr: &mut GuiComponentMgr) -> Rc<Layout>{
        Rc::new(Layout(RefCell::new(LayoutImpl::new())))
    }
}

impl ComponentHandler<NodePoint, GuiComponentMgr> for Layout{
    fn handle(&self, event: &Event<NodePoint>, _component_mgr: &mut GuiComponentMgr){
        match event {
            Event::Create{point, parent: _} => {
                self.0.borrow_mut().list.push(point.clone());
            },
            Event::Delete{point, parent: _} => {
                let mut borrow_mut = self.0.borrow_mut();
                let list = &mut borrow_mut.list;
                for i in 0..list.len() {
                    if list[i].0 == point.0{
                        list.remove(i);
                    }
                }
            },
            _ => {
                unreachable!();
            }
        }
    }
}

impl System<(), GuiComponentMgr> for Layout{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        self.0.borrow_mut().update_layout(component_mgr);
    }
}

pub struct LayoutImpl{
    list: Vec<NodePoint>
}

impl LayoutImpl {
    pub fn new() -> LayoutImpl{
        LayoutImpl{
            list: Vec::new()
        }
    }

    //计算世界矩阵
    pub fn update_layout(&mut self, mgr: &mut GuiComponentMgr){
        for node_point in self.list.iter() {
            let layout = node_point.get_yoga_node(&mgr.node).get_computed_layout();
            let mut node_ref = mgr.get_node_mut(node_point);

            //修改transform
            node_ref.get_transform_mut().modify(|transform: &mut Transform|{
                if transform.position.x == layout.left && transform.position.y == layout.top {
                    return false;
                }
                transform.position.x = layout.left;
                transform.position.y = layout.top;
                true
            });

            //修改size
            node_ref.get_size_mut().modify(|size: &mut Size|{
                if size.width == layout.width && size.height == layout.height {
                    return false;
                }
                size.width = layout.width;
                size.height = layout.height;
                true
            });
        }
    }
}