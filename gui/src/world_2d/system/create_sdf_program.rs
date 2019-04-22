// 监听SdfEffect的创建和删除以及SdfDefines的改变， 动态编译其对应的shader， 并初始化location

use std::rc::{Rc};
use std::cell::RefCell;

use wcs::world::{System};
use wcs::component::{ComponentHandler, ModifyFieldEvent, CreateEvent, DeleteEvent};
use vecmap::IndexMap;

use util::dirty_mark::DirtyMark;
use world_2d::system::render_util::sdf::init_location;
use world_2d::World2dMgr;
use world_2d::component::sdf::{SdfEffect, SdfDefines};

pub struct CreateSdfProgram(Rc<RefCell<DirtyMark>>);

impl CreateSdfProgram {
    pub fn init(component_mgr: &mut World2dMgr) -> Rc<CreateSdfProgram>{
        let r = Rc::new(CreateSdfProgram(Rc::new(RefCell::new(DirtyMark::new()))));
        component_mgr.sdf_effect.defines._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<SdfDefines, ModifyFieldEvent, World2dMgr>>)));
        component_mgr.sdf_effect._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<SdfEffect, DeleteEvent, World2dMgr>>)));
        component_mgr.sdf_effect._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<SdfEffect, CreateEvent, World2dMgr>>)));
        r
    }
}

impl ComponentHandler<SdfDefines, ModifyFieldEvent, World2dMgr> for CreateSdfProgram{
    fn handle(&self, event: &ModifyFieldEvent, _component_mgr: &mut World2dMgr){
        let ModifyFieldEvent{id: _, parent, field: _} = event;
        self.0.borrow_mut().marked_dirty(*parent);
    }
}

impl ComponentHandler<SdfEffect, DeleteEvent, World2dMgr> for CreateSdfProgram{
    fn handle(&self, event: &DeleteEvent, _component_mgr: &mut World2dMgr){
        let DeleteEvent{id, parent: _} = event;
        self.0.borrow_mut().delete_dirty(*id);
    }
}

impl ComponentHandler<SdfEffect, CreateEvent, World2dMgr> for CreateSdfProgram{
    fn handle(&self, event: &CreateEvent, _component_mgr: &mut World2dMgr){
        let CreateEvent{id, parent: _} = event;
        self.0.borrow_mut().dirty_mark_list.insert(*id, false);
        self.0.borrow_mut().marked_dirty(*id);
    }
}

impl System<(), World2dMgr> for CreateSdfProgram{
    fn run(&self, _e: &(), component_mgr: &mut World2dMgr){
        let mut borrow_mut = self.0.borrow_mut();
        for effect_id in borrow_mut.dirtys.iter() {
            let (defines, defines_id) = {
                let defines_id = component_mgr.sdf_effect._group.get(*effect_id).defines.clone();
                let defines = component_mgr.sdf_effect.defines._group.get(defines_id);
                (defines.list(), defines_id)
            };

            let program = component_mgr.engine.create_program(
                component_mgr.shader_store.get(&component_mgr.sdf_shader.vs).unwrap(),
                component_mgr.shader_store.get(&component_mgr.sdf_shader.fs).unwrap(),
                &defines
            );

            match program {
                Ok(v) => {
                    {
                        let effect = component_mgr.sdf_effect._group.get_mut(*effect_id);
                        effect.program = v;
                    }
                    init_location(component_mgr.sdf_effect.defines._group.get(defines_id), &mut component_mgr.engine, v);
                },
                Err(s) => {
                    js!{
                        console.log(@{s});
                    }
                },
            };
        }
        borrow_mut.dirtys.clear();
    }
}

