use std::cell::RefCell;
use std::rc::{Rc};
use std::cmp::{Ord, Ordering, Eq, PartialEq
};


use wcs::world::{System};
use wcs::component::{ComponentHandler, ModifyFieldEvent, CreateEvent, DeleteEvent};
use heap::slab_heap::SlabHeap;
use vecmap::VecMap;

use component::style::transform::{Transform};
use component::render::{SdfDefines, SdfProgram};
use world::GuiComponentMgr;
use component::math::{Matrix4, Vector3};

pub struct Render(RefCell<RenderImpl>);

impl Render {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<Render>{
        let system = Rc::new(Render(RefCell::new(RenderImpl::new())));
        system
    }
}

//渲染
impl System<(), GuiComponentMgr> for Render{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        // let mut borrow_mut = self.0.borrow_mut();
        // for defines_id in borrow_mut.dirtys.iter() {
        //     let (difines, parent_id) = {
        //         let difines = component_mgr.sdf_program.defines._group.get(*defines_id);
        //         (difines.to_vec(), difines.parent)
        //     };
        //     let program = component_mgr.engine.create_program(component_mgr.shader_store.get(&component_mgr.sdf_shader.vs).unwrap(), component_mgr.shader_store.get(&component_mgr.sdf_shader.fs).unwrap(), &difines);
        //     component_mgr.sdf_program._group.get_mut(parent_id).program = program.unwrap();
        // }
        // borrow_mut.dirtys.clear();
    }
}

impl ComponentHandler<SdfProgram, CreateEvent, GuiComponentMgr> for Render{
    fn handle(&self, event: &CreateEvent, _component_mgr: &mut GuiComponentMgr){
        let CreateEvent{id, parent: _} = event;
        let mut borrow = self.0.borrow_mut();
        borrow.mark_list.insert(*id, Mark::new(false, RenderType::Opaque, 0));
        borrow.marked_dirty(*id);
    }
}

//监听is_opaque字段的变化
impl ComponentHandler<SdfProgram, ModifyFieldEvent, GuiComponentMgr> for Render{
    fn handle(&self, event: &ModifyFieldEvent, _component_mgr: &mut GuiComponentMgr){
        let ModifyFieldEvent{id, parent: _, field: _} = event;
        self.0.borrow_mut().marked_dirty(*id);
    }
}

//监听is_opaque字段的变化
// impl ComponentHandler<SdfProgram, ModifyFieldEvent, GuiComponentMgr> for Render{
//     fn handle(&self, event: &ModifyFieldEvent, _component_mgr: &mut GuiComponentMgr){
//         let ModifyFieldEvent{id, parent: _, field: _} = event;
//         self.0.borrow_mut().marked_dirty(*id);
//     }
// }

        // let ty = match component_mgr.sdf_program._group.get(*id).is_opaque {
        //     true => RenderType::Opaque,
        //     false => RenderType::Transparent,
        // };

// impl ComponentHandler<SdfDefines, DeleteEvent, GuiComponentMgr> for Render{
//     fn handle(&self, event: &DeleteEvent, _component_mgr: &mut GuiComponentMgr){
//         let DeleteEvent{id, parent:_} = event;
//         self.0.borrow_mut().delete_dirty(*id);
//     }
// }

pub struct RenderImpl{
    //透明的渲染对象
    transparent_objs: SlabHeap<Obj>,
    //不透明的渲染对象
    opaque_opaque: SlabHeap<Obj>,
    dirtys: Vec<usize>,
    mark_list: VecMap<Mark>,
}

impl RenderImpl {
    pub fn new() -> RenderImpl{
        RenderImpl{
            transparent_objs: SlabHeap::new(Ordering::Greater),
            opaque_opaque: SlabHeap::new(Ordering::Less),
            dirtys: Vec::new(),
            mark_list: VecMap::new(),
        }
    }

    pub fn marked_dirty(&mut self, defines_id: usize){
        let dirty_mark = unsafe{self.mark_list.get_unchecked_mut(defines_id)};
        if dirty_mark.dirty == true {
            return;
        }
        dirty_mark.dirty = true;

        self.dirtys.push(defines_id.clone());
    }

    pub fn delete_dirty(&mut self, defines_id: usize){
        for i in 0..self.dirtys.len(){
            if self.dirtys[i] == defines_id{
                self.dirtys.remove(i);
                return;
            }
        }
    }
}

pub struct Mark {
    dirty: bool,
    old_render_type: RenderType,
    index: usize,
}

impl Mark {
    pub fn new(dirty: bool, old_render_type: RenderType, index: usize) -> Self {
       Mark {
           dirty,
           old_render_type,
           index
       } 
    }
}

pub enum RenderType{
    Transparent,
    Opaque,
}

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
        self.partial_cmp(&other).unwrap()
    }
}

