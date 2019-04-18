// 监听Sdf, CharBlock, 和Word的创建和销毁事件， 创建或销毁对应的Effect

use std::rc::{Rc};
use std::cell::RefCell;

use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::UnsafeTypedArray;

use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent, ModifyFieldEvent};
use wcs::world::System;
use vecmap::VecMap;
use atom::Atom;

use util::dirty_mark::DirtyMark;
use world_2d::World2dMgr;
use world_2d::component::char_block::{CharBlock, CharBlockDefines, CharBlockEffect, CharBlockEffectWriteRef};
use world_2d::system::render_util::char_block::*;
use component::math::{Point2};
use render::res::{Opacity, TextureRes};
use text_layout::font::SdfFont;

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
            char_block_id: *id,
            positions_buffer: component_mgr.engine.gl.create_buffer().unwrap(),
            indeices_buffer: component_mgr.engine.gl.create_buffer().unwrap(),
            uvs_buffer: component_mgr.engine.gl.create_buffer().unwrap(),

            positions: Vec::new(),
            uvs: Vec::new(),
            indeices: Vec::new(),
            extend: Point2::default(),
            font_clamp: 7.5,
            smooth_range: 0.3,
            buffer_dirty: true,
        };
        let effect_id = component_mgr.add_char_block_effect(char_block_effect).id;
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
        for effect_id in borrow_mut.program_dirty.dirtys.iter() {
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
                        effect.program = v;
                    }
                    init_location(component_mgr.char_block_effect.defines._group.get(defines_id), &mut component_mgr.engine, v);
                },
                Err(s) => println!("{}", s),
            };
        }
        borrow_mut.program_dirty.dirtys.clear();
    }

    // 顶点流
    
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

// 填充顶点 uv 索引
fn fill_attribute_index(char_block_id: usize, effect_id: usize, mgr: &mut World2dMgr) -> Attribute {
    let char_block = &mgr.char_block._group.get(char_block_id).owner;
    let char_block_effect_id = &mgr.char_block_effect._group.get(char_block_id).owner;
    let sdf_font = create_test_sdf_font(&mgr.engine.gl); //TODO

    let ratio = char_block.font_size/sdf_font.line_height;

    let mut positions: Vec<f32> = Vec::new();
    let mut uvs: Vec<f32> = Vec::new();
    let mut indeices: Vec<u16> = Vec::new();
    let mut i = 0;
    let line_height = sdf_font.line_height;
    for c in char_block.chars.iter() {
        let glyph = match sdf_font.get_glyph(c.value) {
            Some(r) => r,
            None => continue,
        };
        let pos = &c.pos;

        let width = ratio * glyph.width;
        let height = ratio * glyph.height;
        let half_width = width/2.0;
        let half_height = height/2.0;
        let offset_x = ratio * glyph.ox;
        let offset_y = ratio * (line_height - glyph.oy);

        positions.extend_from_slice(&[
            -half_width + pos.x + offset_x, -half_height + pos.y + offset_y, char_block.z_depth,
            -half_width + pos.x + offset_x, half_height + pos.y + offset_y,  char_block.z_depth,
            half_width + pos.x + offset_x,  half_height + pos.y + offset_y,  char_block.z_depth,
            half_width + pos.x + offset_x,  -half_height + pos.y + offset_y, char_block.z_depth,
        ]);

        let (u, v) = (glyph.x + glyph.ox, glyph.y - (line_height - glyph.oy));
        uvs.extend_from_slice(&[
            u,               v,
            u,               v + glyph.height,
            u + glyph.width, v + glyph.height,
            u + glyph.width, v,
        ]);

        indeices.extend_from_slice(&[4 * i + 0, 4 * i + 1, 4 * i + 2, 4 * i + 0, 4 * i + 2, 4 * i + 3]);
        i += 1;
    }
    
    Attribute {
        positions: positions,
        uvs: uvs,
        indeices: indeices
    }
}

struct Attribute {
    positions: Vec<f32>,
    uvs: Vec<f32>,
    indeices: Vec<u16>,
}

fn create_test_sdf_font(gl: &WebGLRenderingContext) -> Rc<SdfFont>{
    let texture = TextureRes::new(Atom::from("xxx"), 128, 128, Opacity::Translucent, 0, gl.create_texture().unwrap(), gl.clone());
    Rc::new(SdfFont::new(Rc::new(texture)))
}

pub struct Glyph {
    id: u32,
    x: f32,
    y: f32,
    ox: f32, 
    oy: f32,
    width: f32, 
    height: f32,
    advance: f32,
}