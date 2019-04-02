use std::cell::RefCell;
use std::rc::{Rc};
use std::os::raw::{c_void};
use std::mem::forget;

use wcs::world::{System};
use wcs::component::{ComponentHandler, ModifyFieldEvent, DeleteEvent};

use component::style::border::Border;
use component::node::{RectSize, YogaContex, Node};
use world::GuiComponentMgr;
use component::math::{ Vector3 };
use layout::{YGEdge, YGDirection};

pub struct Layout(RefCell<LayoutImpl>);

impl Layout {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<Layout>{
        let system = Rc::new(Layout(RefCell::new(LayoutImpl::new())));
        component_mgr.node.layout_change.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, GuiComponentMgr>>)));
        component_mgr.node._group.register_delete_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Node, DeleteEvent, GuiComponentMgr>>)));
        system
    }
}

//监听LayoutChange组件的变化， 如果修改， 设置脏标志
impl ComponentHandler<Node, ModifyFieldEvent, GuiComponentMgr> for Layout{
    fn handle(&self, event: &ModifyFieldEvent, _component_mgr: &mut GuiComponentMgr){
        let ModifyFieldEvent {id, parent: _, field: _} = event;
        self.0.borrow_mut().dirtys.push(*id);//设置脏
    }
}

//监听node的删除事件，以便删除脏
impl ComponentHandler<Node, DeleteEvent, GuiComponentMgr> for Layout{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut GuiComponentMgr){
        let DeleteEvent {id, parent: _} = event;
         if component_mgr.node._group.get(*id).layout_change == true {
            let mut borrow_mut = self.0.borrow_mut();
            for i in 0..borrow_mut.dirtys.len() {
                //删除脏
                if borrow_mut.dirtys[i] == *id {
                    borrow_mut.dirtys.swap_remove(i);
                    break;
                }
            }
        }
    }
}

impl System<(), GuiComponentMgr> for Layout{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        let root_id = component_mgr.root_id;
        let width = component_mgr.root_width;
        let height = component_mgr.root_height;
        //计算布局，如果布局更改， 调用回调来设置LayoutChange组件
        component_mgr.node._group.get(root_id).yoga.calculate_layout_by_callback(width, height, YGDirection::YGDirectionLTR, callback);
        self.0.borrow_mut().update_layout(component_mgr);
    }
}

pub struct LayoutImpl{
    dirtys: Vec<usize> // Vec<node_id>
}

impl LayoutImpl {
    pub fn new() -> LayoutImpl{
        LayoutImpl{
            dirtys: Vec::new()
        }
    }

    // yoga改变， 需要更新position，extend， border
    pub fn update_layout(&mut self, mgr: &mut GuiComponentMgr){
        for node_id in self.dirtys.iter() {
            let (layout, border) = {
                let yoga = &mgr.node._group.get(*node_id).yoga;
                let layout = yoga.get_layout();
                js!{
                    console.log(@{format!("update_layout, left:{}, top:{}, width:{}, height:{}, node_id:{}", layout.left, layout.top, layout.width, layout.height, *node_id)});
                }
                (yoga.get_layout(), yoga.get_layout_border(YGEdge::YGEdgeLeft))
            };
            let mut node_ref = mgr.get_node_mut(*node_id);

            //修改position
            node_ref.get_position_mut().modify(|position: &mut Vector3|{
                if position.x == layout.left && position.y == layout.top {
                    return false;
                }
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

            node_ref.modify(|node: &mut Node| {
                node.layout_change = false;
                false //不发监听
            });
        }
        self.dirtys.clear(); //清除脏标志
    }
}

//回调函数
#[no_mangle]
extern "C" fn callback(context: *const c_void) {
    //更新布局
    let yoga_context = unsafe { Box::from_raw(context as usize as *mut YogaContex) };
    let component_mgr = unsafe{ &mut *(yoga_context.mgr as *mut GuiComponentMgr) };
    //默认node_id为1的节点为根节点，根节点创建时不会发出创建时间， 因此也不应该处理根节点的布局变化， 否则其他某些监听Node创建事件的系统可能存在问题
    if yoga_context.node_id == 1 {
        return;
    }
    let mut node_ref = component_mgr.get_node_mut(yoga_context.node_id);
    node_ref.set_layout_change(true);
    forget(yoga_context);
}