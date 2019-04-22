// 监听CharBlock的创建和销毁事件， 创建或销毁对应的Effect

use std::rc::{Rc};
use std::cell::RefCell;

use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent, ModifyFieldEvent};
use wcs::world::System;
use vecmap::{ VecMap, IndexMap};

use util::dirty_mark::DirtyMark;
use world_2d::World2dMgr;
use world_2d::component::char_block::{CharBlock, CharBlockDefines, CharBlockEffect, CharBlockEffectWriteRef};
use world_2d::system::render_util::char_block::*;
use component::math::{Point2};

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
        let CreateEvent{id, parent:_} = event;
        let mut borrow_mut = self.0.borrow_mut();
        //创建effect
        let defines_id = component_mgr.char_block_effect.defines._group.insert(CharBlockDefines::default(), 0);
        let char_block_effect = CharBlockEffect {
            program: 0,
            defines: defines_id,
            positions_buffer: component_mgr.engine.gl.create_buffer().unwrap(),
            indeices_buffer: component_mgr.engine.gl.create_buffer().unwrap(),
            uvs_buffer: component_mgr.engine.gl.create_buffer().unwrap(),

            extend: Point2::default(),
            font_clamp: 0.55,
            smooth_range: 0.1,
            buffer_dirty: true,
            indeices_len: 0,
        };
        let effect_id = {
            let mut effect = component_mgr.add_char_block_effect(char_block_effect);
            effect.set_parent(*id);
            effect.id
        };
        borrow_mut.char_block_effect_map.insert(*id, effect_id);

        // 标记着色器程序脏
        borrow_mut.program_dirty.dirty_mark_list.insert(*id, false);
        borrow_mut.program_dirty.marked_dirty(*id);

        // 标记buffer脏
        borrow_mut.buffer_dirty.dirty_mark_list.insert(*id, false);
        borrow_mut.buffer_dirty.marked_dirty(*id);
    }
}

impl ComponentHandler<CharBlock, DeleteEvent, World2dMgr> for CharBlockSys{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut World2dMgr){
        let DeleteEvent{id, parent:_} = event;
        let mut borrow_mut =  self.0.borrow_mut();
        let effect_id = unsafe { borrow_mut.char_block_effect_map.remove(*id) };
        let char_block_effect = component_mgr.char_block_effect._group.remove(effect_id);

        // 删除顶点buffer. uvbuffer 和索引buffer
        component_mgr.engine.gl.delete_buffer(Some(&char_block_effect.positions_buffer));
        component_mgr.engine.gl.delete_buffer(Some(&char_block_effect.uvs_buffer));
        component_mgr.engine.gl.delete_buffer(Some(&char_block_effect.indeices_buffer));

        //删除脏标记
        borrow_mut.program_dirty.delete_dirty(*id);
        borrow_mut.buffer_dirty.delete_dirty(*id);

        // 发出销毁事件
        CharBlockEffectWriteRef::new(*id, component_mgr.char_block_effect.to_usize(), component_mgr).destroy();
    }
}

impl ComponentHandler<CharBlock, ModifyFieldEvent, World2dMgr> for CharBlockSys {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut World2dMgr) {
        let ModifyFieldEvent { id, parent: _, field } = event;
   
        if *field == "by_overflow" || *field == "stroke_size"{
            self.0.borrow_mut().program_dirty.marked_dirty(*id);
        }else if *field == "color" {
            self.0.borrow_mut().program_dirty.marked_dirty(*id);

            //设置不透明性
            let is_opaque = is_opaque(*id, component_mgr);
            component_mgr.get_char_block_mut(*id).set_is_opaque(is_opaque);
            
        }else if *field == "alpha"  {
            //设置不透明性
            let is_opaque = is_opaque(*id, component_mgr);
            component_mgr.get_char_block_mut(*id).set_is_opaque(is_opaque);
        }
    }
}

impl System<(), World2dMgr> for CharBlockSys{
    fn run(&self, _e: &(), component_mgr: &mut World2dMgr){
        let mut borrow_mut = self.0.borrow_mut();
        println!("run CharBlockSys------------------------");
        borrow_mut.update_program(component_mgr);
        borrow_mut.update_buffer(component_mgr);
    }   
}

#[allow(dead_code)]
struct CharBlockSysImpl{
    char_block_effect_map: VecMap<usize>,
    program_dirty: DirtyMark,
    buffer_dirty: DirtyMark,
}

impl CharBlockSysImpl {
    fn new () -> CharBlockSysImpl {
        CharBlockSysImpl{
            char_block_effect_map: VecMap::new(),
            program_dirty: DirtyMark::new(),
            buffer_dirty: DirtyMark::new(),
        }
    }

    fn update_program(&mut self, component_mgr: &mut World2dMgr) {
        println!("update_program create------------------------");
        for effect_id in self.program_dirty.dirtys.iter() {
            println!("update_program create1------------------------");
            unsafe{*self.program_dirty.dirty_mark_list.get_unchecked_mut(*effect_id) = false};
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
        self.program_dirty.dirtys.clear();
    }

    fn update_buffer(&mut self, component_mgr: &mut World2dMgr) {
        for effect_id in self.buffer_dirty.dirtys.iter() {
            unsafe{*self.buffer_dirty.dirty_mark_list.get_unchecked_mut(*effect_id) = false};
            update(component_mgr, *effect_id);
        }
        self.buffer_dirty.dirtys.clear();
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