use std::rc::{Rc};
use std::os::raw::{c_void};
use std::mem::forget;

use wcs::world::{System};

use world_doc::component::node::{YogaContex};
use world_doc::WorldDocMgr;
use layout::{YGDirection};

pub struct Layout;

impl Layout {
    pub fn init(_component_mgr: &mut WorldDocMgr) -> Rc<Layout>{
        let system = Rc::new(Layout);
        system
    }
}

impl System<(), WorldDocMgr> for Layout{
    fn run(&self, _e: &(), component_mgr: &mut WorldDocMgr){
        let root_id = component_mgr.root_id;
        let width = component_mgr.world_2d.component_mgr.width;
        let height = component_mgr.world_2d.component_mgr.height;

        println!("calculate_layout_by_callback width:{}", width);
        println!("calculate_layout_by_callback height:{}", height);
        println!("width------------------------ width:{}", component_mgr.node._group.get(root_id).yoga.get_width());
        println!("height------------------------ height:{}", component_mgr.node._group.get(root_id).yoga.get_height());

        // component_mgr.node._group.get(root_id).yoga.calculate_layout(width, height, YGDirection::YGDirectionLTR);
        // println!("layout------------------------{:?}", component_mgr.node._group.get(root_id).yoga.get_layout());
        //计算布局，如果布局更改， 调用回调来设置layout属性
        component_mgr.node._group.get(root_id).yoga.calculate_layout_by_callback(width, height, YGDirection::YGDirectionLTR, callback, 0 as *const c_void);

        update(component_mgr, 1);
        // let yoga = &component_mgr.node._group.get(1).yoga;
        // println!("update_layout1, layout: {:?}, node_id:{}",  yoga.get_layout(), 1);
    }
}

fn update(mgr: &mut WorldDocMgr, node_id: usize) {
    let layout = {
        let yoga = &mgr.node._group.get(node_id).yoga;
        println!("update_layout, layout: {:?}, node_id:{}",  yoga.get_layout(), node_id);
        yoga.get_layout()
    };

    let mut node_ref = mgr.get_node_mut(node_id);

    //修改position
    node_ref.set_layout(layout);
}

//回调函数
#[no_mangle]
extern "C" fn callback(arg: *const c_void, context: *const c_void) {
    //更新布局
    let yoga_context = unsafe { Box::from_raw(context as usize as *mut YogaContex) };
    let component_mgr = unsafe{ &mut *(yoga_context.mgr as *mut WorldDocMgr) };
    //默认node_id为1的节点为根节点，根节点创建时不会发出创建时间， 因此也不应该处理根节点的布局变化， 否则其他某些监听Node创建事件的系统可能存在问题
    // if yoga_context.node_id == 1 {
    //     return;
    // }
    update(component_mgr, yoga_context.node_id);
    forget(yoga_context);
}