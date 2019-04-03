// 监听Sdf, Image, 和Word的创建和销毁事件， 创建或销毁对应的Effect

use std::rc::{Rc};
use std::cell::RefCell;

use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent};
use vecmap::VecMap;

use object2d::Object2dMgr;
use object2d::component::sdf::{Sdf, SdfEffect, SdfEffectWriteRef, SdfDefines};

pub struct CreateEffect(Rc<RefCell<CreateEffectImpl>>);

impl CreateEffect {
    pub fn init(component_mgr: &mut Object2dMgr) -> Rc<CreateEffect>{
        let r = Rc::new(CreateEffect(Rc::new(RefCell::new(CreateEffectImpl::new()))));
        component_mgr.sdf._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Sdf, CreateEvent, Object2dMgr>>)));
        component_mgr.sdf._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Sdf, DeleteEvent, Object2dMgr>>)));
        // component_mgr.render_obj.defines.sdf._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<SdfDefines, ModifyFieldEvent, Object2dMgr>>)));
        // component_mgr.render_obj.defines.text._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<TextDefines, ModifyFieldEvent, Object2dMgr>>)));
        // component_mgr.render_obj.defines.image._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<ImageDefines, ModifyFieldEvent, Object2dMgr>>)));
        r
    }
}
 
impl ComponentHandler<Sdf, CreateEvent, Object2dMgr> for CreateEffect{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut Object2dMgr){
        let CreateEvent{id, parent:_} = event;
        let defines = SdfDefines::default();
        let defines_id = component_mgr.sdf_effect.defines._group.insert(defines, 0);
        let sdf_effect = SdfEffect {
            program: 0,
            defines: defines_id,
            sdf_id: *id,
            positions_buffer: component_mgr.engine.gl.create_buffer().unwrap(),
            indeices_buffer: component_mgr.engine.gl.create_buffer().unwrap(),
            positions_dirty: true,
        };
        let effect_id = component_mgr.add_sdf_effect(sdf_effect).id;
        self.0.borrow_mut().sdf_effect_map.insert(*id, effect_id);
    }
}

impl ComponentHandler<Sdf, DeleteEvent, Object2dMgr> for CreateEffect{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut Object2dMgr){
        let DeleteEvent{id, parent:_} = event;
        let effect_id = unsafe { self.0.borrow_mut().sdf_effect_map.remove(*id) };
        let sdf_effect = component_mgr.sdf_effect._group.remove(effect_id);
        component_mgr.engine.gl.delete_buffer(Some(&sdf_effect.positions_buffer));
        component_mgr.engine.gl.delete_buffer(Some(&sdf_effect.indeices_buffer));
        SdfEffectWriteRef::new(*id, component_mgr.sdf_effect.to_usize(), component_mgr).destroy(); //通知组件销毁
    }
}

#[allow(dead_code)]
struct CreateEffectImpl{
    sdf_effect_map: VecMap<usize>,
    image_effect_map: VecMap<usize>,
    word_effect_map: VecMap<usize>
}

impl CreateEffectImpl {
    fn new () -> CreateEffectImpl {
        CreateEffectImpl{
            sdf_effect_map: VecMap::new(),
            image_effect_map: VecMap::new(),
            word_effect_map: VecMap::new()
        }
    }
}

// Image, Word TODO

 
