use std::rc::{Rc};
use std::cell::RefCell;

use wcs::world::{System};
use wcs::component::{ComponentHandler, ModifyFieldEvent, CreateEvent, DeleteEvent};
use vecmap::VecMap;

use document::component::render::{SdfDefines, RenderObj, TextDefines, ImageDefines, DefinesId, DefinesList};
use document::DocumentMgr;

// 监听SdfDefines的创建和变化， 从新创建program
pub struct CreateProgram(RefCell<CreateProgramImpl>);

impl CreateProgram {
    pub fn init(component_mgr: &mut DocumentMgr) -> Rc<CreateProgram>{
        let r = Rc::new(CreateProgram(RefCell::new(CreateProgramImpl::new())));
        component_mgr.render_obj._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<RenderObj, CreateEvent, DocumentMgr>>)));
        component_mgr.render_obj._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<RenderObj, DeleteEvent, DocumentMgr>>)));
        component_mgr.render_obj.defines.sdf._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<SdfDefines, ModifyFieldEvent, DocumentMgr>>)));
        component_mgr.render_obj.defines.text._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<TextDefines, ModifyFieldEvent, DocumentMgr>>)));
        component_mgr.render_obj.defines.image._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<ImageDefines, ModifyFieldEvent, DocumentMgr>>)));
        r
    }
}
 
impl ComponentHandler<RenderObj, CreateEvent, DocumentMgr> for CreateProgram{
    fn handle(&self, event: &CreateEvent, _component_mgr: &mut DocumentMgr){
        let CreateEvent{id, parent:_} = event;
        let mut borrow = self.0.borrow_mut();
        borrow.dirty_mark_list.insert(*id, false);
        borrow.marked_dirty(*id);
    }
}

impl ComponentHandler<RenderObj, DeleteEvent, DocumentMgr> for CreateProgram{
    fn handle(&self, event: &DeleteEvent, _component_mgr: &mut DocumentMgr){
        let DeleteEvent{id, parent:_} = event;
        self.0.borrow_mut().delete_dirty(*id);
    }
}

 
impl ComponentHandler<SdfDefines, ModifyFieldEvent, DocumentMgr> for CreateProgram{
    fn handle(&self, event: &ModifyFieldEvent, _component_mgr: &mut DocumentMgr){
        let ModifyFieldEvent{id: _, parent, field: _} = event;
        self.0.borrow_mut().marked_dirty(*parent);
    }
}

impl ComponentHandler<TextDefines, ModifyFieldEvent, DocumentMgr> for CreateProgram{
    fn handle(&self, event: &ModifyFieldEvent, _component_mgr: &mut DocumentMgr){
        let ModifyFieldEvent{id: _, parent, field: _} = event;
        self.0.borrow_mut().marked_dirty(*parent);
    }
}

impl ComponentHandler<ImageDefines, ModifyFieldEvent, DocumentMgr> for CreateProgram{
    fn handle(&self, event: &ModifyFieldEvent, _component_mgr: &mut DocumentMgr){
        let ModifyFieldEvent{id: _, parent, field: _} = event;
        self.0.borrow_mut().marked_dirty(*parent);
    }
}

//创建program
impl System<(), DocumentMgr> for CreateProgram{
    fn run(&self, _e: &(), component_mgr: &mut DocumentMgr){
        let mut borrow_mut = self.0.borrow_mut();
        for render_obj_id in borrow_mut.dirtys.iter() {
            let defines = {
                let defines_id = component_mgr.render_obj._group.get(*render_obj_id).defines.clone();
                let defines_list = match defines_id {
                    DefinesId::Sdf(id) => component_mgr.render_obj.defines.sdf._group.get(id).list(),
                    DefinesId::Text(id) => component_mgr.render_obj.defines.text._group.get(id).list(),
                    DefinesId::Image(id) => component_mgr.render_obj.defines.image._group.get(id).list(),
                    DefinesId::None => continue,
                };
                defines_list
            };

            let program = component_mgr.engine.create_program(component_mgr.shader_store.get(&component_mgr.sdf_shader.vs).unwrap(),
            component_mgr.shader_store.get(&component_mgr.sdf_shader.fs).unwrap(), &defines);

            match program {
                Ok(v) => {
                    let mgr_ptr = component_mgr as *const DocumentMgr as usize;
                    let render_obj = component_mgr.render_obj._group.get_mut(*render_obj_id);
                    render_obj.program = v;
                    unsafe { render_obj.bind.init_locations(mgr_ptr, *render_obj_id)}
                },
                Err(s) => {js!{
                    console.log(@{s});
                };},
            };
        }
        borrow_mut.dirtys.clear();
    }
}

pub struct CreateProgramImpl {
    dirtys: Vec<usize>,
    dirty_mark_list: VecMap<bool>,
}

impl CreateProgramImpl {
    pub fn new() -> CreateProgramImpl{
        CreateProgramImpl{
            dirtys: Vec::new(),
            dirty_mark_list: VecMap::new(),
        }
    }

    pub fn marked_dirty(&mut self, render_obj_id: usize){
        let dirty_mark = unsafe{self.dirty_mark_list.get_unchecked_mut(render_obj_id)};
        if *dirty_mark == true {
            return;
        }
        *dirty_mark = true;

        self.dirtys.push(render_obj_id.clone());
    }

    pub fn delete_dirty(&mut self, render_obj_id: usize){
        for i in 0..self.dirtys.len(){
            if self.dirtys[i] == render_obj_id{
                self.dirtys.remove(i);
                return;
            }
        }
    }
}