// 监听Sdf, Image, 和Word的创建和销毁事件， 创建或销毁对应的Effect

use std::rc::{Rc};
use std::cell::RefCell;

use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::UnsafeTypedArray;

use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent, ModifyFieldEvent};
use wcs::world::System;
use vecmap::{ VecMap, IndexMap};

use util::dirty_mark::DirtyMark;
use world_2d::World2dMgr;
use world_2d::component::image::{Image, ImageDefines, ImageEffect, ImageEffectWriteRef};
use world_2d::system::render_util::image::*;
use render::res::Opacity;

pub struct ImageSys(Rc<RefCell<ImageSysImpl>>);

impl ImageSys {
    pub fn init(component_mgr: &mut World2dMgr) -> Rc<ImageSys>{
        let r = Rc::new(ImageSys(Rc::new(RefCell::new(ImageSysImpl::new()))));
        component_mgr.image._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Image, CreateEvent, World2dMgr>>)));
        component_mgr.image._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Image, DeleteEvent, World2dMgr>>)));
        component_mgr.image.by_overflow.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Image, ModifyFieldEvent, World2dMgr>>)));
        component_mgr.image.alpha.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Image, ModifyFieldEvent, World2dMgr>>)));
        component_mgr.image.color.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Image, ModifyFieldEvent, World2dMgr>>)));
        r
    }
}
 
impl ComponentHandler<Image, CreateEvent, World2dMgr> for ImageSys{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut World2dMgr){
        let CreateEvent{id, parent:_} = event;
        {
            let mut defines = ImageDefines::default();
            let image = &component_mgr.image._group.get(*id);
            if image.by_overflow > 0 {
                defines.clip_plane = true;
            }
            let defines_id = component_mgr.image_effect.defines._group.insert(defines, 0);
            let image_effect = ImageEffect {
                program: 0,
                defines: defines_id,
                image_id: *id,
                positions_buffer: component_mgr.engine.gl.create_buffer().unwrap(),
                indeices_buffer: component_mgr.engine.gl.create_buffer().unwrap(),
                uvs_buffer: component_mgr.engine.gl.create_buffer().unwrap(),
                positions_dirty: true,
            };
            //全局使用同一个indeices？ TODO
            let indeices: [u16; 6] = [0, 1, 2, 0, 2, 3];
            let buffer = unsafe { UnsafeTypedArray::new(&indeices) };
            component_mgr.engine.gl.bind_buffer(
                WebGLRenderingContext::ELEMENT_ARRAY_BUFFER,
                Some(&image_effect.indeices_buffer),
            );
            js! {
                @{&component_mgr.engine.gl}.bufferData(@{WebGLRenderingContext::ELEMENT_ARRAY_BUFFER}, @{&buffer}, @{WebGLRenderingContext::STATIC_DRAW});
            }

            // TODO, uvs可以根据clip属性变化
            let uvs: [f32; 8] = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0];
            #[cfg(feature = "log")]
            println!("image uv: {:?}", &uvs[0..8]);
            let buffer = unsafe { UnsafeTypedArray::new(&uvs) };
            component_mgr.engine.gl.bind_buffer(
                WebGLRenderingContext::ARRAY_BUFFER,
                Some(&image_effect.uvs_buffer),
            );
            js! {
                @{&component_mgr.engine.gl}.bufferData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{&buffer}, @{WebGLRenderingContext::STATIC_DRAW});
            }

            let effect_id = component_mgr.add_image_effect(image_effect).id;
            self.0.borrow_mut().image_effect_map.insert(*id, effect_id);
        }
        
        let is_opaque = is_opaque(*id, component_mgr);

        // 标记着色器程序脏
        self.0.borrow_mut().program_dirty.dirty_mark_list.insert(*id, false);
        self.0.borrow_mut().program_dirty.marked_dirty(*id);

        component_mgr.get_image_mut(*id).set_is_opaque(is_opaque);
    }
}

impl ComponentHandler<Image, DeleteEvent, World2dMgr> for ImageSys{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut World2dMgr){
        let DeleteEvent{id, parent:_} = event;
        let effect_id = unsafe { self.0.borrow_mut().image_effect_map.remove(*id) };
        let image_effect = component_mgr.image_effect._group.remove(effect_id);
        // 删除顶点buffer和索引buffer
        component_mgr.engine.gl.delete_buffer(Some(&image_effect.positions_buffer));
        component_mgr.engine.gl.delete_buffer(Some(&image_effect.indeices_buffer));

        //删除脏标记
        self.0.borrow_mut().program_dirty.delete_dirty(*id);
        // 发出销毁事件
        ImageEffectWriteRef::new(*id, component_mgr.image_effect.to_usize(), component_mgr).destroy();
    }
}

impl ComponentHandler<Image, ModifyFieldEvent, World2dMgr> for ImageSys {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut World2dMgr) {
        let ModifyFieldEvent { id, parent: _, field } = event;
        let effect_id = unsafe { *(self.0.borrow_mut().image_effect_map.get_unchecked(*id)) };
        
        if *field == "by_overflow" {
            let by_overflow = component_mgr.image._group.get(*id).by_overflow;
            if by_overflow == 0{
                if *(component_mgr.get_image_effect(effect_id).get_defines().get_clip_plane()) == false {
                    return;
                }
                component_mgr.get_image_effect_mut(effect_id).get_defines_mut().set_clip_plane(false);
            }else {
                if *(component_mgr.get_image_effect(effect_id).get_defines().get_clip_plane()) == true {
                    return;
                }
                component_mgr.get_image_effect_mut(effect_id).get_defines_mut().set_clip_plane(true);
            }
            // 标记着色器程序脏
            self.0.borrow_mut().program_dirty.marked_dirty(*id);
        }else if *field == "color" || *field == "alpha" {
            //设置不透明性
            let is_opaque = is_opaque(*id, component_mgr);
            component_mgr.get_image_mut(*id).set_is_opaque(is_opaque);
        }
    }
}

impl System<(), World2dMgr> for ImageSys{
    fn run(&self, _e: &(), component_mgr: &mut World2dMgr){
        let mut borrow_mut = self.0.borrow_mut();
        for effect_id in borrow_mut.program_dirty.dirtys.iter() {
            let (defines, defines_id) = {
                let defines_id = component_mgr.image_effect._group.get(*effect_id).defines.clone();
                let defines = component_mgr.image_effect.defines._group.get(defines_id);
                (defines.list(), defines_id)
            };

            let program = component_mgr.engine.create_program(
                component_mgr.shader_store.get(&component_mgr.image_shader.vs).unwrap(),
                component_mgr.shader_store.get(&component_mgr.image_shader.fs).unwrap(),
                &defines
            );

            match program {
                Ok(v) => {
                    {
                        let effect = component_mgr.image_effect._group.get_mut(*effect_id);
                        effect.program = v;
                    }
                    init_location(component_mgr.image_effect.defines._group.get(defines_id), &mut component_mgr.engine, v);
                },
                Err(s) => println!("{}", s),
            };
        }
        borrow_mut.program_dirty.dirtys.clear();
    }
}

#[allow(dead_code)]
struct ImageSysImpl{
    image_effect_map: VecMap<usize>,
    program_dirty: DirtyMark,
}

impl ImageSysImpl {
    fn new () -> ImageSysImpl {
        ImageSysImpl{
            image_effect_map: VecMap::new(),
            program_dirty: DirtyMark::new(),
        }
    }
}

fn is_opaque(image_id: usize, mgr: &mut World2dMgr) -> bool {
    let image = &mgr.image._group.get(image_id).owner;
    // if image.color.a < 1.0{
    //     return false;
    // }
    if image.alpha < 1.0 {
        return false;
    }
    match image.src.opacity {
        Opacity::Translucent | Opacity::Transparent => return false,
        _ => return true,
    }
}