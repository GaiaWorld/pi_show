use std::cell::RefCell;
use std::rc::{Rc};
use std::cmp::{Ord, Ordering, Eq, PartialEq};


use wcs::world::{System};
use world_doc::WorldDocMgr;

pub struct Render(RefCell<RenderImpl>);

impl Render {
    pub fn init(_component_mgr: &mut WorldDocMgr) -> Rc<Render>{
        let system = Rc::new(Render(RefCell::new(RenderImpl::new())));
        // component_mgr.render_obj._group.register_create_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<RenderObj, CreateEvent, WorldDocMgr>>)));
        // component_mgr.render_obj._group.register_delete_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<RenderObj, DeleteEvent, WorldDocMgr>>)));
        // component_mgr.render_obj.is_opaque.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<RenderObj, ModifyFieldEvent, WorldDocMgr>>)));
        // component_mgr.render_obj.z_index.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<RenderObj, ModifyFieldEvent, WorldDocMgr>>)));
        system
    }
}

//渲染, 
impl System<(), WorldDocMgr> for Render{
    fn run(&self, _e: &(), component_mgr: &mut WorldDocMgr){
        self.0.borrow_mut().render(component_mgr);
    }
}

pub struct RenderImpl{
    //透明的渲染对象
    transparent_objs: Vec<Obj>,
    //透明的渲染对象
    // transparent_objs: SlabHeap<Obj>, // Obj.id对应RenderObj在slab中的位子
    //不透明的渲染对象
    opaque_objs: Vec<usize>, // Obj.id对应RenderObj在slab中的位子
}

impl RenderImpl {
    pub fn new() -> RenderImpl{
        RenderImpl{
            transparent_objs: Vec::new(),
            opaque_objs: Vec::new(),
        }
    }

    // pub fn marked_dirty(&mut self, defines_id: usize){
    //     let dirty_mark = unsafe{self.mark_list.get_unchecked_mut(defines_id)};
    //     if dirty_mark.dirty == true {
    //         return;
    //     }
    //     dirty_mark.dirty = true;

    //     self.dirtys.push(defines_id.clone());
    // }

    // pub fn delete_dirty(&mut self, defines_id: usize){
    //     for i in 0..self.dirtys.len(){
    //         if self.dirtys[i] == defines_id{
    //             self.dirtys.remove(i);
    //             return;
    //         }
    //     }
    // }

    pub fn render(&mut self, mgr: &mut WorldDocMgr) {
        self.list_obj(mgr);
        let mgr_ptr = mgr as *const WorldDocMgr as usize;
        for v in self.opaque_objs.iter() {
            //bind an render
            unsafe {mgr.render_obj._group.get(*v).bind.bind(mgr_ptr, *v)};
            //render
        }

        for v in self.transparent_objs.iter() {
            //bind an render
            unsafe {mgr.render_obj._group.get(v.id).bind.bind(mgr_ptr, v.id)};
            //render
        }
        self.opaque_objs.clear();
        self.transparent_objs.clear();
    }

    //对不透明物体和透明物体排序
    fn list_obj(&mut self, mgr: &mut WorldDocMgr){
        for v in mgr.render_obj._group.iter() {
            if v.1.is_opaque {
                self.opaque_objs.push(v.0);
            }else {
                self.transparent_objs.push(Obj{z: v.1.z_index, id: v.0} );
            }
        }
        self.transparent_objs.sort();
    }
}

// pub struct Mark {
//     render_type: RenderType, // RenderType::None表示不在heap中
//     index: usize,   //index==0 && render_type==RenderType::None， 表示插入， index!=0 && render_type==RenderType::None表示删除,
// }

// impl Mark {
//     pub fn new(render_type: RenderType, index: usize) -> Self {
//        Mark {
//            render_type,
//            index,
//        } 
//     }
// }

// #[derive(Clone, Copy, Debug)]
// pub enum RenderType{
//     None,
//     Transparent,
//     Opaque,
// }

// #[derive(Clone, Copy, Debug)]
// pub enum DirtyType{
//     Insert,
//     Delete,
//     Modify,
// }

pub struct Obj {
    z: f32,
    id: usize,
}

impl PartialOrd for Obj {
	fn partial_cmp(&self, other: &Obj) -> Option<Ordering> {
		self.z.partial_cmp(&other.z)
	}
}

impl PartialEq for Obj{
	 fn eq(&self, other: &Obj) -> bool {
        self.z.eq(&other.z)
    }
}

impl Eq for Obj{}

impl Ord for Obj{
	fn cmp(&self, other: &Obj) -> Ordering {
        js!{console.log("mmmmmmmmmmmmmmmmmmmmmmmmmmmm")}
        let r = self.partial_cmp(&other).unwrap();
        js!{console.log("nnnnnnnnaaaaaaaaaaaaaaaaaaaaaaaa")}
        r

    }
}

