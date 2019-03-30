use std::rc::{Rc};
use std::cell::RefCell;

use wcs::world::{System};
use wcs::component::{ComponentHandler, ModifyFieldEvent, CreateEvent, DeleteEvent};
use vecmap::VecMap;

use component::render::{SdfDefines};
use world::GuiComponentMgr;

// 监听SdfDefines的创建和变化， 从新创建program
pub struct SdfDefinesDirty(RefCell<SdfDefinesDirtyImpl>);

impl SdfDefinesDirty {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<SdfDefinesDirty>{
        let r = Rc::new(SdfDefinesDirty(RefCell::new(SdfDefinesDirtyImpl::new())));
        component_mgr.sdf_program.defines._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<SdfDefines, CreateEvent, GuiComponentMgr>>)));
        component_mgr.sdf_program.defines._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<SdfDefines, DeleteEvent, GuiComponentMgr>>)));
        component_mgr.sdf_program.defines._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<SdfDefines, ModifyFieldEvent, GuiComponentMgr>>)));
        r
    }
}
 
impl ComponentHandler<SdfDefines, CreateEvent, GuiComponentMgr> for SdfDefinesDirty{
    fn handle(&self, event: &CreateEvent, _component_mgr: &mut GuiComponentMgr){
        let CreateEvent{id, parent:_} = event;
        let mut borrow = self.0.borrow_mut();
        borrow.dirty_mark_list.insert(*id, false);
        borrow.marked_dirty(*id);
    }
}

impl ComponentHandler<SdfDefines, DeleteEvent, GuiComponentMgr> for SdfDefinesDirty{
    fn handle(&self, event: &DeleteEvent, _component_mgr: &mut GuiComponentMgr){
        let DeleteEvent{id, parent:_} = event;
        self.0.borrow_mut().delete_dirty(*id);
    }
}

 
impl ComponentHandler<SdfDefines, ModifyFieldEvent, GuiComponentMgr> for SdfDefinesDirty{
    fn handle(&self, event: &ModifyFieldEvent, _component_mgr: &mut GuiComponentMgr){
        let ModifyFieldEvent{id, parent: _, field: _} = event;
        self.0.borrow_mut().marked_dirty(*id);
    }
}

//创建program
impl System<(), GuiComponentMgr> for SdfDefinesDirty{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        let mut borrow_mut = self.0.borrow_mut();
        for defines_id in borrow_mut.dirtys.iter() {
            let (difines, parent_id) = {
                let difines = component_mgr.sdf_program.defines._group.get(*defines_id);
                (difines.to_vec(), difines.parent)
            };
            let program = component_mgr.engine.create_program(component_mgr.shader_store.get(&component_mgr.sdf_shader.vs).unwrap(), component_mgr.shader_store.get(&component_mgr.sdf_shader.fs).unwrap(), &difines);
            component_mgr.sdf_program._group.get_mut(parent_id).program = program.unwrap();
        }
        borrow_mut.dirtys.clear();
    }
}

pub struct SdfDefinesDirtyImpl {
    dirtys: Vec<usize>,
    dirty_mark_list: VecMap<bool>,
}

impl SdfDefinesDirtyImpl {
    pub fn new() -> SdfDefinesDirtyImpl{
        SdfDefinesDirtyImpl{
            dirtys: Vec::new(),
            dirty_mark_list: VecMap::new(),
        }
    }

    pub fn marked_dirty(&mut self, defines_id: usize){
        let dirty_mark = unsafe{self.dirty_mark_list.get_unchecked_mut(defines_id)};
        if *dirty_mark == true {
            return;
        }
        *dirty_mark = true;

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