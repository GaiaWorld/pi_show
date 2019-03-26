use std::cell::RefCell;
use std::rc::{Rc};
use std::os::raw::{c_void};
use std::mem::forget;

use wcs::world::{System};
use wcs::component::{ComponentHandler, Event};

use component::style::border::Border;
use component::node::{RectSize, YogaContex, LayoutChange};
use world::GuiComponentMgr;
use component::math::{ Vector3 };
use layout::{YGEdge, YGDirection};

pub struct Layout(RefCell<LayoutImpl>);

impl Layout {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<Layout>{
        let system = Rc::new(Layout(RefCell::new(LayoutImpl::new())));
        component_mgr.node.layout_change._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<LayoutChange, GuiComponentMgr>>)));
        system
    }
}

//监听LayoutChange组件的变化， 如果修改， 设置脏标志
impl ComponentHandler<LayoutChange, GuiComponentMgr> for Layout{
    fn handle(&self, event: &Event, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::ModifyField{id: _, parent, field: _} => {
                let mut borrow_mut = self.0.borrow_mut();
                borrow_mut.dirtys.push(*parent); //设置脏
            },
            Event::Delete{id, parent} => {
                if component_mgr.node.layout_change._group.get(*id).value == true {
                    let mut borrow_mut = self.0.borrow_mut();
                    for i in 0..borrow_mut.dirtys.len() {
                        //删除脏
                        if borrow_mut.dirtys[i] == *parent {
                            borrow_mut.dirtys.swap_remove(i);
                            break;
                        }
                    }
                }
            },
            //create事件不需要监听， node初始化就会创建LayoutChange组件， 但只有LayoutChange改变时， 布局才会发生变化
            _ => ()
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

            node_ref.get_layout_change_mut().modify(|layout_change: &mut LayoutChange| {
                layout_change.value = false;
                false //不发监听
            });
        }
        self.dirtys.clear(); //清除脏标志
    }
}

//回调函数
#[no_mangle]
extern "C" fn callback(context: *const c_void) {
    // js!{
    //     console.log("ccccccccccccccccccccccccccxxxxxxxxxxxxxx");
    // }
    //更新布局
    let yoga_context = unsafe { Box::from_raw(context as usize as *mut YogaContex) };
    let component_mgr = unsafe{ &mut *(yoga_context.mgr as *mut GuiComponentMgr) };
    let mut node_ref = component_mgr.get_node_mut(yoga_context.node_id);
    node_ref.get_layout_change_mut().modify(|change: &mut LayoutChange|{
        change.value = true;
        true
    });
    // js!{
    //     console.log("yyyyyyyyyyyyyyyyyyyyyyyy");
    // }
    forget(yoga_context);
    // js!{
    //     console.log("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    // }
}

#[cfg(test)]
mod test {
    use std::mem::{uninitialized, forget};
    use std::rc::Rc;
    use std::os::raw::{c_void};

    use stdweb::*;

    use wcs::world::{World, System};
    use wcs::component::Builder;

    use world::GuiComponentMgr;
    use system::layout::Layout;
    use component::node::{NodeBuilder, InsertType, YogaContex};
    use layout::{YGFlexDirection};

    #[test]
    pub fn test_layout_system(){
        let mut world = new_world();
        let node1 = NodeBuilder::new().build(&mut world.component_mgr.node);
        let node2 = NodeBuilder::new().build(&mut world.component_mgr.node);
        let node3 = NodeBuilder::new().build(&mut world.component_mgr.node);
        let node4 = NodeBuilder::new().build(&mut world.component_mgr.node);

        node1.yoga.set_width(100.0);
        node1.yoga.set_height(100.0);
        node2.yoga.set_width(200.0);
        node2.yoga.set_height(200.0); 
        node3.yoga.set_width(300.0);
        node3.yoga.set_height(300.0); 
        node4.yoga.set_width(400.0);
        node4.yoga.set_height(500.0);

        world.component_mgr.set_size(500.0, 500.0);
        let (root, root_yoga, node_ids) = {
            let root = NodeBuilder::new().build(&mut world.component_mgr.node);
            let root_yoga = root.yoga;
            let mut root_ref = world.component_mgr.add_node(root);
            (   
                root_ref.id,
                root_yoga,
                [
                    root_ref.insert_child(node1, InsertType::Back).id,
                    root_ref.insert_child(node2, InsertType::Back).id,
                    root_ref.insert_child(node3, InsertType::Back).id,
                    root_ref.insert_child(node4, InsertType::Back).id,
                ]
            )
        };
        let yoga_context = Box::into_raw(Box::new(YogaContex {
            node_id: root,
            mgr: &world.component_mgr as *const GuiComponentMgr as usize,
        })) as usize;
        root_yoga.set_context(yoga_context as *mut c_void);
        root_yoga.set_flex_direction(YGFlexDirection::YGFlexDirectionRow);
        world.component_mgr.set_root(root);

        // root_yoga.calculate_layout(500.0, 500.0, YGDirection::YGDirectionLTR);
        world.run(());
        for i in node_ids.iter(){
            {
                let node_ref = world.component_mgr.get_node_mut(*i);
                let width = node_ref.get_extent().get_width().clone();
                let height = node_ref.get_extent().get_height().clone();
                let x = node_ref.get_position().get_x().clone();
                let y = node_ref.get_position().get_y().clone();

                let node_s = format!("test_layout_system, node{} position_x:{:?}, position_y:{:?}, width:{:?}, heigth: {:?}", i, x, y, width, height);
                js!{
                    console.log(@{node_s} );
                }
            }
        }

        forget(world);
    }

    fn new_world() -> World<GuiComponentMgr, ()>{
        let mut world: World<GuiComponentMgr, ()> = World::new(GuiComponentMgr::new(unsafe{uninitialized()}));
        let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![Layout::init(&mut world.component_mgr)];
        world.set_systems(systems);
        world
    }
}