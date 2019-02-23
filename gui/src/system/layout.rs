use std::cell::RefCell;
use std::rc::{Rc};

use web_sys::*;

use wcs::world::{System};
use wcs::component::{ComponentHandler, Event};

use component::component_def::{ GuiComponentMgr, RectSize, Border, Node, Rect, Circle};
use component::math::{ Vector3 };
use layout::{Edge, YgNode};

pub struct Layout(RefCell<LayoutImpl>);

impl Layout {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<Layout>{
        let system = Rc::new(Layout(RefCell::new(LayoutImpl::new())));
        component_mgr.node._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, GuiComponentMgr>>)));
        component_mgr.node.element.rect.shape._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Rect, GuiComponentMgr>>)));
        component_mgr.node.element.circle.shape._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Circle, GuiComponentMgr>>)));
        system
    }
}

impl ComponentHandler<Node, GuiComponentMgr> for Layout{
    fn handle(&self, event: &Event, _component_mgr: &mut GuiComponentMgr){
        match event {
            Event::Create{id, parent: _} => {
                self.0.borrow_mut().list.push(*id);
            },
            Event::Delete{id, parent: _} => {
                let mut borrow_mut = self.0.borrow_mut();
                let list = &mut borrow_mut.list;
                for i in 0..list.len() {
                    if list[i] == *id{
                        list.remove(i);
                    }
                }
            },
            _ => ()
        }
    }
}

impl ComponentHandler<Rect, GuiComponentMgr> for Layout{
    fn handle(&self, event: &Event, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::Create{id, parent} => {
                let Rect{left_top, width, height, radius: _ } = component_mgr.node.element.rect.shape._group.get_mut(*id).owner;
                let parent = component_mgr.node.element.rect._group.get_mut(*parent).parent;
                let node = component_mgr.node._group.get_mut(parent);
                node.yoga_node.set_width(width);
                node.yoga_node.set_height(height);
                node.yoga_node.set_position(Edge::Left,left_top.x);
                node.yoga_node.set_position(Edge::Top, left_top.y);
                let ptr = &node.yoga_node as *const YgNode as usize;
                console::log_7(&("Rect create".into()), &(width.to_string().into()), &(height.to_string().into()), &(left_top.x.to_string().into()), &(left_top.y.to_string().into()), &(ptr.to_string().into()),  &(parent.to_string().into()));
            },
            Event::ModifyField{id, parent, field} => {
                let Rect{left_top, width, height, radius: _ } = component_mgr.node.element.rect.shape._group.get_mut(*id).owner;
                let parent = component_mgr.node.element.rect._group.get_mut(*parent).parent;
                let node = component_mgr.node._group.get_mut(parent);
                if field == &"width" {
                    node.yoga_node.set_width(width);
                }else if field == &"height"{
                    node.yoga_node.set_height(height);
                }else if field == &"left_top" {
                    node.yoga_node.set_position(Edge::Left,left_top.x);
                    node.yoga_node.set_position(Edge::Top, left_top.y);
                }
            },
            _ => ()
        }
    }
}

impl ComponentHandler<Circle, GuiComponentMgr> for Layout{
    fn handle(&self, event: &Event, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::Create{id, parent} => {
                let Circle{center, radius } = component_mgr.node.element.circle.shape._group.get_mut(*id).owner;
                let parent = component_mgr.node.element.circle._group.get_mut(*parent).parent;
                let node = component_mgr.node._group.get_mut(parent);
                let size = radius * 2.0;
                node.yoga_node.set_width(size);
                node.yoga_node.set_height(size);
                node.yoga_node.set_position(Edge::Left,center.x - radius);
                node.yoga_node.set_position(Edge::Top, center.y - radius);
                let ptr = &node.yoga_node as *const YgNode as usize;
                console::log_6(&("Circle create".into()), &(size.to_string().into()), &((center.x - radius).to_string().into()), &((center.y - radius).to_string().into()), &(ptr.to_string().into()), &(parent.to_string().into()));
            },
            Event::ModifyField{id, parent, field: _} => {
                let Circle{center, radius } = component_mgr.node.element.circle.shape._group.get_mut(*id).owner;
                let parent = component_mgr.node.element.circle._group.get_mut(*parent).parent;
                let node = component_mgr.node._group.get_mut(parent);
                let size = radius * 2.0;
                node.yoga_node.set_width(size);
                node.yoga_node.set_height(size);
                node.yoga_node.set_position(Edge::Left,center.x - radius);
                node.yoga_node.set_position(Edge::Top, center.y - radius);
            },
            _ => ()
        }
    }
}

impl System<(), GuiComponentMgr> for Layout{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        console::log_1(&("layout run".into()));
        self.0.borrow_mut().update_layout(component_mgr);
    }
}

pub struct LayoutImpl{
    list: Vec<usize> // Vec<node_id>
}

impl LayoutImpl {
    pub fn new() -> LayoutImpl{
        LayoutImpl{
            list: Vec::new()
        }
    }

    // yoga改变， 需要更新position，extend， border
    pub fn update_layout(&mut self, mgr: &mut GuiComponentMgr){
        for node_id in self.list.iter() {
            let (layout, border) = {
                let yoga = &mgr.node._group.get(*node_id).yoga_node;
                let ptr = yoga as *const YgNode as usize;
                console::log_5(&("update_layout".into()), &(yoga.get_height().to_string().into()), &(yoga.get_width().to_string().into()),  &(ptr.to_string().into()),  &(node_id.to_string().into()));
                (yoga.get_computed_layout(), yoga.get_computed_border(Edge::Left))
                
            };
            let mut node_ref = mgr.get_node_mut(*node_id);

            //修改position
            node_ref.get_position_mut().modify(|position: &mut Vector3|{
                console::log_3(&("modify_position".into()), &(layout.left.to_string().into()), &(layout.top.to_string().into()));
                if position.x == layout.left && position.y == layout.top {
                    return false;
                }
                console::log_3(&("modify_position_end".into()), &(layout.left.to_string().into()), &(layout.top.to_string().into()));
                position.x = layout.left;
                position.y = layout.top;
                true
            });

            //修改extend
            node_ref.get_extent_mut().modify(|size: &mut RectSize|{
                if size.width == layout.width && size.height == layout.height {
                    return false;
                }
                size.width = layout.width;
                size.height = layout.height;
                console::log_4(&("update_extent_end".into()), &(layout.width.to_string().into()), &(layout.height.to_string().into()), &(node_id.to_string().into()));
                true
            });

            //修改border
            node_ref.get_border_mut().modify(|b: &mut Border|{
                if b.value == border {
                    return false;
                }
                b.value = border;
                true
            });
        }
    }
}