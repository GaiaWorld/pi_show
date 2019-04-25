// 监听CharBlock的创建和销毁事件， 创建或销毁对应的Effect

use std::rc::{Rc};
use std::cell::RefCell;

use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent, ModifyFieldEvent};
use wcs::world::System;
use vecmap::{ VecMap};

use util::dirty_mark::DirtyMark;
use world_2d::World2dMgr;
use world_2d::component::char_block::{CharBlock, CharBlockDefines, CharBlockEffect, CharBlockEffectWriteRef};
use world_2d::system::render_util::char_block::*;
use component::color::Color;

pub struct CharBlockSys(Rc<RefCell<CharBlockSysImpl>>);

impl CharBlockSys {
    pub fn init(component_mgr: &mut World2dMgr) -> Rc<CharBlockSys>{
        let r = Rc::new(CharBlockSys(Rc::new(RefCell::new(CharBlockSysImpl::new()))));
        component_mgr.char_block._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<CharBlock, CreateEvent, World2dMgr>>)));
        component_mgr.char_block._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<CharBlock, DeleteEvent, World2dMgr>>)));
        component_mgr.char_block.by_overflow.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<CharBlock, ModifyFieldEvent, World2dMgr>>)));
        component_mgr.char_block.alpha.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<CharBlock, ModifyFieldEvent, World2dMgr>>)));
        component_mgr.char_block.color.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<CharBlock, ModifyFieldEvent, World2dMgr>>)));
        r
    }
}
 
impl ComponentHandler<CharBlock, CreateEvent, World2dMgr> for CharBlockSys{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut World2dMgr){
        println!("create charblock");
        let CreateEvent{id, parent:_} = event;
        let mut borrow_mut = self.0.borrow_mut();
        //创建effect
        let effect_id = create_effect(*id, component_mgr, 0.1, false);
        borrow_mut.char_block_effect_map.insert(*id, effect_id);

        // 标记着色器程序脏
        borrow_mut.program_dirty.dirty_mark_list.insert(effect_id, false);
        borrow_mut.program_dirty.marked_dirty(effect_id);

        // 标记buffer脏
        borrow_mut.buffer_dirty.dirty_mark_list.insert(effect_id, false);
        borrow_mut.buffer_dirty.marked_dirty(effect_id);
        
        match component_mgr.char_block._group.get(*id).shadow.clone() {
            Some(shadow) => {
                println!("shadow--------------------------------------");
                let effect_id = create_effect(*id, component_mgr, shadow.blur * 0.1, true);
                borrow_mut.char_block_shadow_effect_map.insert(*id, effect_id);

                // 标记着色器程序脏
                borrow_mut.shadow_program_dirty.dirty_mark_list.insert(effect_id, false);
                borrow_mut.shadow_program_dirty.marked_dirty(effect_id);

                // 标记buffer脏
                borrow_mut.shadow_buffer_dirty.dirty_mark_list.insert(effect_id, false);
                borrow_mut.shadow_buffer_dirty.marked_dirty(effect_id);
            },
            None => (),
        }
    }
}

impl ComponentHandler<CharBlock, DeleteEvent, World2dMgr> for CharBlockSys{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut World2dMgr){
        let DeleteEvent{id, parent:_} = event;
        let mut borrow_mut =  self.0.borrow_mut();
        let effect_id = unsafe { borrow_mut.char_block_effect_map.remove_unchecked(*id) };
        let char_block_effect = component_mgr.char_block_effect._group.remove(effect_id);

        // 删除顶点buffer. uvbuffer 和索引buffer
        component_mgr.engine.gl.delete_buffer(Some(&char_block_effect.positions_buffer));
        component_mgr.engine.gl.delete_buffer(Some(&char_block_effect.uvs_buffer));
        component_mgr.engine.gl.delete_buffer(Some(&char_block_effect.indeices_buffer));

        //删除脏标记
        borrow_mut.program_dirty.delete_dirty(*id);
        borrow_mut.buffer_dirty.delete_dirty(*id);

        match borrow_mut.char_block_shadow_effect_map.remove(*id) {
            Some(shadow_effect_id) => {
                let char_block_shadow_effect = component_mgr.char_block_effect._group.remove(shadow_effect_id);
                // 删除顶点buffer. uvbuffer 和索引buffer
                component_mgr.engine.gl.delete_buffer(Some(&char_block_shadow_effect.positions_buffer));
                component_mgr.engine.gl.delete_buffer(Some(&char_block_shadow_effect.uvs_buffer));
                component_mgr.engine.gl.delete_buffer(Some(&char_block_shadow_effect.indeices_buffer));

                //删除shadow脏标记
                borrow_mut.shadow_program_dirty.delete_dirty(*id);
                borrow_mut.shadow_buffer_dirty.delete_dirty(*id);
            }, 
            None => (),
        }

        // 发出销毁事件
        CharBlockEffectWriteRef::new(*id, component_mgr.char_block_effect.to_usize(), component_mgr).destroy();
    }
}

impl ComponentHandler<CharBlock, ModifyFieldEvent, World2dMgr> for CharBlockSys {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut World2dMgr) {
        let ModifyFieldEvent { id, parent: _, field } = event;

        if *field == "by_overflow" {
            let effect_id = *unsafe { self.0.borrow_mut().char_block_effect_map.get_unchecked(*id) };
            self.0.borrow_mut().program_dirty.marked_dirty(effect_id);
            

            let by_overflow = component_mgr.char_block._group.get(*id).by_overflow;
            if by_overflow == 0{
                if *(component_mgr.get_char_block_effect(effect_id).get_defines().get_clip_plane()) == false {
                    return;
                }
                component_mgr.get_char_block_effect_mut(effect_id).get_defines_mut().set_clip_plane(false);
            }else {
                if *(component_mgr.get_char_block_effect(effect_id).get_defines().get_clip_plane()) == true {
                    return;
                }
                component_mgr.get_char_block_effect_mut(effect_id).get_defines_mut().set_clip_plane(true);
            }
            
            let char_block = component_mgr.char_block._group.get(*id);
            match &char_block.shadow {
                Some(_) => {
                    let effect_id = *unsafe { self.0.borrow_mut().char_block_shadow_effect_map.get_unchecked(*id) };
                    if by_overflow == 0{
                        if *(component_mgr.get_char_block_effect(effect_id).get_defines().get_clip_plane()) == false {
                            return;
                        }
                        component_mgr.get_char_block_effect_mut(effect_id).get_defines_mut().set_clip_plane(false);
                    }else {
                        if *(component_mgr.get_char_block_effect(effect_id).get_defines().get_clip_plane()) == true {
                            return;
                        }
                        component_mgr.get_char_block_effect_mut(effect_id).get_defines_mut().set_clip_plane(true);
                    }
                    self.0.borrow_mut().shadow_program_dirty.marked_dirty(effect_id)
                },
                None => (),
            };
        }else if *field == "stroke_size"{
            let effect_id = *unsafe { self.0.borrow_mut().char_block_effect_map.get_unchecked(*id) };
            self.0.borrow_mut().program_dirty.marked_dirty(effect_id);
        }else if *field == "color" {
            let effect_id = *unsafe { self.0.borrow_mut().char_block_effect_map.get_unchecked(*id) };
            self.0.borrow_mut().program_dirty.marked_dirty(effect_id);
            
            {
                let mgr = unsafe {&mut *(component_mgr as *mut World2dMgr) };
                let color = &component_mgr.char_block._group.get(*id).color;
                modify_color_defines(effect_id, color, mgr);
            }
            
            //设置不透明性
            let is_opaque = is_opaque(*id, component_mgr);
            component_mgr.get_char_block_mut(*id).set_is_opaque(is_opaque);
            
        }else if *field == "alpha"  {
            //设置不透明性
            let is_opaque = is_opaque(*id, component_mgr);
            component_mgr.get_char_block_mut(*id).set_is_opaque(is_opaque);
        } else if *field == "shadow" {
            let char_block = component_mgr.char_block._group.get(*id);
            let mut borrow_mut = self.0.borrow_mut();
            match &char_block.shadow {
                Some(_shadow) => {
                    let effect_id = *unsafe { borrow_mut.char_block_shadow_effect_map.get_unchecked(*id) };
                    // 标记着色器程序脏
                    borrow_mut.shadow_program_dirty.dirty_mark_list.insert(effect_id, false);
                    borrow_mut.shadow_program_dirty.marked_dirty(effect_id);

                    // 标记buffer脏
                    borrow_mut.shadow_buffer_dirty.dirty_mark_list.insert(effect_id, false);
                    borrow_mut.shadow_buffer_dirty.marked_dirty(effect_id);
                },
                None => {
                    match borrow_mut.char_block_shadow_effect_map.remove(*id) {
                        Some(shadow_effect_id) => {
                            let char_block_shadow_effect = component_mgr.char_block_effect._group.remove(shadow_effect_id);
                            // 删除顶点buffer. uvbuffer 和索引buffer
                            component_mgr.engine.gl.delete_buffer(Some(&char_block_shadow_effect.positions_buffer));
                            component_mgr.engine.gl.delete_buffer(Some(&char_block_shadow_effect.uvs_buffer));
                            component_mgr.engine.gl.delete_buffer(Some(&char_block_shadow_effect.indeices_buffer));

                            //删除shadow脏标记
                            borrow_mut.shadow_program_dirty.delete_dirty(shadow_effect_id);
                            borrow_mut.shadow_buffer_dirty.delete_dirty(shadow_effect_id);
                        }, 
                        None => (),
                    }
                },
            }
        }
    }
}

impl System<(), World2dMgr> for CharBlockSys{
    fn run(&self, _e: &(), component_mgr: &mut World2dMgr){
        let mut borrow_mut = self.0.borrow_mut();
        println!("charblock run-----------------------------");
        borrow_mut.update_program(component_mgr);
        borrow_mut.update_buffer(component_mgr);
    }   
}

#[allow(dead_code)]
struct CharBlockSysImpl{
    char_block_effect_map: VecMap<usize>,
    program_dirty: DirtyMark,
    buffer_dirty: DirtyMark,

    char_block_shadow_effect_map: VecMap<usize>,
    shadow_program_dirty: DirtyMark,
    shadow_buffer_dirty: DirtyMark,
}

impl CharBlockSysImpl {
    fn new () -> CharBlockSysImpl {
        CharBlockSysImpl{
            char_block_effect_map: VecMap::new(),
            program_dirty: DirtyMark::new(),
            buffer_dirty: DirtyMark::new(),

            char_block_shadow_effect_map: VecMap::new(),
            shadow_program_dirty: DirtyMark::new(),
            shadow_buffer_dirty: DirtyMark::new(),
        }
    }

    fn update_program(&mut self, component_mgr: &mut World2dMgr) {
        update_program(&mut self.program_dirty, component_mgr);
        update_program(&mut self.shadow_program_dirty, component_mgr);
    }

    fn update_buffer(&mut self, component_mgr: &mut World2dMgr) {
        update_buffer(&mut self.buffer_dirty, component_mgr);
        update_buffer(&mut self.shadow_buffer_dirty, component_mgr);
    }
}

fn is_opaque(char_block_id: usize, mgr: &mut World2dMgr) -> bool {
    let char_block = &mgr.char_block._group.get(char_block_id).owner;
    if char_block.alpha < 1.0 {
        return false;
    }

    if char_block.stroke_color.a < 1.0 {
        return false;
    }

    return char_block.color.is_opaque();
}

fn create_effect(parent: usize, component_mgr: &mut World2dMgr, blur: f32, is_shadow: bool) -> usize{
    let defines_id = component_mgr.char_block_effect.defines._group.insert(CharBlockDefines::default(), 0);
    let char_block_effect = CharBlockEffect {
        program: 0,
        defines: defines_id,
        positions_buffer: component_mgr.engine.gl.create_buffer().unwrap(),
        indeices_buffer: component_mgr.engine.gl.create_buffer().unwrap(),
        uvs_buffer: component_mgr.engine.gl.create_buffer().unwrap(),

        font_clamp: 0.55,
        smooth_range: blur,
        buffer_dirty: true,
        indeices_len: 0,
        is_shadow: is_shadow,
    };
    
    let mut effect = component_mgr.add_char_block_effect(char_block_effect);
    effect.set_parent(parent);
    effect.id
}

fn update_program(program_dirty: &mut DirtyMark, component_mgr: &mut World2dMgr) {
    for effect_id in program_dirty.dirtys.iter() {
        println!("shadow update_program--------------------------------------");
        unsafe{*program_dirty.dirty_mark_list.get_unchecked_mut(*effect_id) = false};
        let (defines, defines_id) = {
            let defines_id = component_mgr.char_block_effect._group.get(*effect_id).defines.clone();
            let defines = component_mgr.char_block_effect.defines._group.get(defines_id);
            (defines.list(), defines_id)
        };

        let program = component_mgr.engine.create_program(
            component_mgr.shader_store.get(&component_mgr.char_block_shader.vs).unwrap(),
            component_mgr.shader_store.get(&component_mgr.char_block_shader.fs).unwrap(),
            &defines
        );

        match program {
            Ok(v) => {
               
                {
                    let effect = component_mgr.char_block_effect._group.get_mut(*effect_id);
                     println!("shadow update_program1-------------------------------------effect_id:{}, program: {}, effect.program: {}", effect_id, v, effect.program);
                    if v == effect.program {
                        continue;
                    }
                    effect.program = v;
                }
                init_location(component_mgr.char_block_effect.defines._group.get(defines_id), &mut component_mgr.engine, v);
               
            },
            Err(s) => println!("{}", s),
        };
    }
    program_dirty.dirtys.clear();
}

fn update_buffer(buffer_dirty: &mut DirtyMark, component_mgr: &mut World2dMgr) {
    for effect_id in buffer_dirty.dirtys.iter() {
        unsafe{*buffer_dirty.dirty_mark_list.get_unchecked_mut(*effect_id) = false};
        update(component_mgr, *effect_id);
    }
    buffer_dirty.dirtys.clear();
}

fn modify_color_defines(effect_id: usize, color: &Color, mgr: &mut World2dMgr) {
    let mut effect_ref = mgr.get_char_block_effect_mut(effect_id);
    let mut defines_ref = effect_ref.get_defines_mut();
    match color {
        Color::RGB(_) | Color::RGBA(_) => {
            //修改COLOR宏
            defines_ref.set_color(true);
            defines_ref.set_linear_color_gradient_2(false);
            defines_ref.set_linear_color_gradient_4(false);
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
        }
        _ => ()
    }
}