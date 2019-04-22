// 监听Sdf, Image, 和Word的创建和销毁事件， 创建或销毁对应的Effect

use std::rc::{Rc};
use std::cell::RefCell;

use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::UnsafeTypedArray;

use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent, ModifyFieldEvent};
use wcs::world::System;
use vecmap::{ VecMap, IndexMap};

use component::color::Color;
use world_2d::World2dMgr;
use world_2d::component::sdf::{Sdf, SdfEffect, SdfEffectWriteRef, SdfDefines, SdfType};

pub struct CreateEffect(Rc<RefCell<CreateEffectImpl>>);

impl CreateEffect {
    pub fn init(component_mgr: &mut World2dMgr) -> Rc<CreateEffect>{
        let r = Rc::new(CreateEffect(Rc::new(RefCell::new(CreateEffectImpl::new()))));
        component_mgr.sdf._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Sdf, CreateEvent, World2dMgr>>)));
        component_mgr.sdf._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Sdf, DeleteEvent, World2dMgr>>)));
        component_mgr.sdf.by_overflow.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Sdf, ModifyFieldEvent, World2dMgr>>)));
        component_mgr.sdf.color.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Sdf, ModifyFieldEvent, World2dMgr>>)));
        component_mgr.sdf.border_size.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Sdf, ModifyFieldEvent, World2dMgr>>)));
        component_mgr.sdf.bound_box.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Sdf, ModifyFieldEvent, World2dMgr>>)));
        // component_mgr.render_obj.defines.sdf._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<SdfDefines, ModifyFieldEvent, World2dMgr>>)));
        // component_mgr.render_obj.defines.text._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<TextDefines, ModifyFieldEvent, World2dMgr>>)));
        // component_mgr.render_obj.defines.image._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<ImageDefines, ModifyFieldEvent, World2dMgr>>)));
        r
    }
}

impl System<(), World2dMgr> for CreateEffect{
    fn run(&self, _e: &(), _component_mgr: &mut World2dMgr){
    }
}
 
impl ComponentHandler<Sdf, CreateEvent, World2dMgr> for CreateEffect{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut World2dMgr){
        let CreateEvent{id, parent:_} = event;
        let mut defines = SdfDefines::default();
        let sdf = &component_mgr.sdf._group.get(*id);
        match sdf.ty {
            SdfType::Rect => defines.sdf_rect = true,
            _ => (),
        };
        if sdf.border_size > 0.0 {
            defines.stroke = true;
        }
        match &sdf.color {
            Color::RGB(_) | Color::RGBA(_) => {
                //修改COLOR宏
                defines.color = true;
                defines.linear_color_gradient_2 = false;
                defines.linear_color_gradient_4 = false;
                defines.ellipse_color_gradient = false;
            }
            Color::LinearGradient(v) => {
                //修改COLOR宏
                defines.color = false;
                if v.list.len() == 2 {
                    defines.linear_color_gradient_2 = true;
                    defines.linear_color_gradient_4 = false;
                } else {
                    defines.linear_color_gradient_2 = false;
                    defines.linear_color_gradient_4 = true;
                }
                defines.ellipse_color_gradient = false;
            }
            Color::RadialGradient(_) => {
                //修改COLOR宏
                defines.color = false;
                defines.linear_color_gradient_2 = false;
                defines.linear_color_gradient_4 = false;
                defines.ellipse_color_gradient = true;
            }
        };
        if sdf.by_overflow > 0 {
            defines.clip_plane = true;
        }
        let defines_id = component_mgr.sdf_effect.defines._group.insert(defines, 0);
        let sdf_effect = SdfEffect {
            program: 0,
            defines: defines_id,
            sdf_id: *id,
            positions_buffer: component_mgr.engine.gl.create_buffer().unwrap(),
            indeices_buffer: component_mgr.engine.gl.create_buffer().unwrap(),
            positions_dirty: true,
        };
        //全局使用同一个indeices？ TODO
        let indeices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let buffer = unsafe { UnsafeTypedArray::new(&indeices) };
        component_mgr.engine.gl.bind_buffer(
            WebGLRenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&sdf_effect.indeices_buffer),
        );
        js! {
            @{&component_mgr.engine.gl}.bufferData(@{WebGLRenderingContext::ELEMENT_ARRAY_BUFFER}, @{&buffer}, @{WebGLRenderingContext::STATIC_DRAW});
        }
        let effect_id = component_mgr.add_sdf_effect(sdf_effect).id;
        self.0.borrow_mut().sdf_effect_map.insert(*id, effect_id);
    }
}

impl ComponentHandler<Sdf, DeleteEvent, World2dMgr> for CreateEffect{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut World2dMgr){
        let DeleteEvent{id, parent:_} = event;
        let effect_id = unsafe { self.0.borrow_mut().sdf_effect_map.remove(*id) };
        {
            let (positions_buffer, indeices_buffer) = {
                let sdf_effect = component_mgr.sdf_effect._group.get(effect_id);
                (sdf_effect.positions_buffer.clone(), sdf_effect.indeices_buffer.clone())
            };
            component_mgr.engine.gl.delete_buffer(Some(&positions_buffer));
            component_mgr.engine.gl.delete_buffer(Some(&indeices_buffer));
        }
        
        SdfEffectWriteRef::new(effect_id, component_mgr.sdf_effect.to_usize(), component_mgr).destroy(); //通知组件销毁
    }
}

impl ComponentHandler<Sdf, ModifyFieldEvent, World2dMgr> for CreateEffect {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut World2dMgr) {
        let ModifyFieldEvent { id, parent: _, field } = event;
        let effect_id = unsafe { *(self.0.borrow_mut().sdf_effect_map.get_unchecked(*id)) };
        
        if *field == "by_overflow" {
            let by_overflow = component_mgr.sdf._group.get(*id).by_overflow;
            if by_overflow == 0 {
                component_mgr.get_sdf_effect_mut(effect_id).get_defines_mut().set_clip_plane(false);
            }else {
                component_mgr.get_sdf_effect_mut(effect_id).get_defines_mut().set_clip_plane(true);
            }
        }else if *field == "color" {
            let mgr = unsafe {&mut *(component_mgr as *mut World2dMgr) };
            let color = &component_mgr.sdf._group.get(*id).color;
            modify_color_defines(effect_id, color, mgr);
        }else if *field == "border_size" {
            let border_size = component_mgr.sdf._group.get(*id).border_size;
            if border_size == 0.0 {
                component_mgr.get_sdf_effect_mut(effect_id).get_defines_mut().set_stroke(false);
            } else {
                component_mgr.get_sdf_effect_mut(effect_id).get_defines_mut().set_stroke(true);
            }
        } else if *field == "bound_box" {
            component_mgr.get_sdf_effect_mut(effect_id).set_positions_dirty(true); //TODO
        }
    }
}

// fn init_defines(effect_id: usize, &mut Defines, mgr: &mut World2dMgr){
//     if 
// }

fn modify_color_defines(effect_id: usize, color: &Color, mgr: &mut World2dMgr) {
    let mut effect_ref = mgr.get_sdf_effect_mut(effect_id);
    let mut defines_ref = effect_ref.get_defines_mut();
    match color {
        Color::RGB(_) | Color::RGBA(_) => {
            //修改COLOR宏
            defines_ref.set_color(true);
            defines_ref.set_linear_color_gradient_2(false);
            defines_ref.set_linear_color_gradient_4(false);
            defines_ref.set_ellipse_color_gradient(false);
        }
        Color::LinearGradient(v) => {
            //修改COLOR宏
            defines_ref.set_color(false);
            if v.list.len() == 2 {
                defines_ref.set_linear_color_gradient_2(true);
                defines_ref.set_linear_color_gradient_4(false);
            } else {
                defines_ref.set_linear_color_gradient_2(false);
                defines_ref.set_linear_color_gradient_4(true);
            }
            defines_ref.set_ellipse_color_gradient(false);
        }
        Color::RadialGradient(_) => {
            //修改COLOR宏
            defines_ref.set_color(true);
            defines_ref.set_linear_color_gradient_2(false);
            defines_ref.set_linear_color_gradient_4(false);
            defines_ref.set_ellipse_color_gradient(true);
        }
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

 
