use std::rc::{Rc};
use std::os::raw::{c_void};
use std::mem::forget;

use wcs::world::{System};

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

        let context = Box::into_raw(Box::new(CallbackContext {
            _sys: self as *const Layout as usize,
            mgr: component_mgr as *mut WorldDocMgr as usize,
        }));
        // component_mgr.node._group.get(root_id).yoga.calculate_layout(width, height, YGDirection::YGDirectionLTR);
        // println!("layout------------------------{:?}", component_mgr.node._group.get(root_id).yoga.get_layout());
        //计算布局，如果布局更改， 调用回调来设置layout属性
        component_mgr.node._group.get(root_id).yoga.calculate_layout_by_callback(width, height, YGDirection::YGDirectionLTR, callback, context as *const c_void);

        update(component_mgr, 1);
        // let yoga = &component_mgr.node._group.get(1).yoga;
        // println!("update_layout1, layout: {:?}, node_id:{}",  yoga.get_layout(), 1);
    }
}

struct CallbackContext {
    _sys: usize,
    mgr: usize,
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
extern "C" fn callback(callback_context: *const c_void, context: *const c_void) {
    let node_id = context as usize;
    let callback_context = unsafe { Box::from_raw(callback_context as usize as *mut CallbackContext)};
    let component_mgr = unsafe {&mut *(callback_context.mgr as *mut WorldDocMgr)};
    //更新布局
    update(component_mgr, node_id);
    forget(callback_context);
}