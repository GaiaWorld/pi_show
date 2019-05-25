/**
 *  
 */
use std::marker::PhantomData;
use std::sync::Arc;
use std::mem::transmute;

use std::collections::HashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use ecs::idtree::{ IdTree};
use map::{ vecmap::VecMap, Map } ;
use hal_core::{Context, Uniforms, RasterState, BlendState, StencilState, DepthState, BlendFunc, CullMode, ShaderType, Pipeline, Geometry, AttributeName};
use atom::Atom;
use polygon::*;

use component::user::*;
use component::calc::{Visibility, WorldMatrix, Opacity, ByOverflow, ZDepth};
use entity::{Node};
use single::{RenderObjs, RenderObjWrite, RenderObj, ViewMatrix, ProjectionMatrix, ClipUbo, ViewUbo, ProjectionUbo};
use render::engine::Engine;
use system::util::*;
use system::util::constant::{PROJECT_MATRIX, WORLD_MATRIX, VIEW_MATRIX, POSITION, COLOR, CLIP_INDEICES, ALPHA, CLIP, VIEW, PROJECT, WORLD, COMMON};
use system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};

pub struct NodeAttrSys<C: Context + Share>{
    node_render_map: VecMap<Vec<usize>>,
    marker: PhantomData<C>,
}

impl<C: Context + Share> NodeAttrSys<C> {
    pub fn new() -> Self{
        NodeAttrSys {
            node_render_map: VecMap::default(),
            marker: PhantomData,
        }
    }
}

impl<'a, C: Context + Share> EntityListener<'a, Node, CreateEvent> for NodeAttrSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        self.node_render_map.insert(event.id, Vec::new());
    }
}

impl<'a, C: Context + Share> EntityListener<'a, Node, DeleteEvent> for NodeAttrSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, _write: Self::WriteData){
        unsafe { self.node_render_map.remove_unchecked(event.id) };
    }
}

//创建索引
impl<'a, C: Context + Share> SingleCaseListener<'a, RenderObjs<C>, CreateEvent> for NodeAttrSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<ClipUbo<C>>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Layout>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (clip_ubo, world_matrixs, by_overflows, opacitys, visibilitys, transforms, layouts) = read;
        let (render_objs, engine) = write;
        let render_obj = unsafe { render_objs.get_unchecked_mut(event.id) };
        unsafe{ self.node_render_map.get_unchecked_mut(render_obj.context).push(event.id) };
        
        let mut ubos = &mut render_obj.ubos;
        // 插入世界矩阵ubo
        let mut world_matrix_ubo = engine.gl.create_uniforms();
        let world_matrix = cal_matrix(event.id, world_matrixs, transforms, layouts);
        let slice: &[f32; 16] = world_matrix.as_ref();
        world_matrix_ubo.set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
        ubos.insert(WORLD_MATRIX.clone(), Arc::new(world_matrix_ubo)); // WORLD_MATRIX

        let by_overflow = unsafe { by_overflows.get_unchecked(event.id) }.0;
        if by_overflow > 0 {
            let defines = &mut render_obj.defines;

            // 插入裁剪ubo 插入裁剪宏
            ubos.insert(CLIP.clone(), clip_ubo.0.clone());
            defines.push(CLIP.clone());
            
            // 重新创建渲染管线
            let pipeline = engine.create_pipeline(
                0,
                &render_obj.pipeline.vs,
                &render_obj.pipeline.fs,
                render_obj.defines.as_slice(),
                render_obj.pipeline.rs.clone(),
                render_obj.pipeline.bs.clone(),
                render_obj.pipeline.ss.clone(),
                render_obj.pipeline.ds.clone(),
            );
            render_obj.pipeline = pipeline;
        }

        let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
        unsafe {Arc::make_mut(ubos.get_mut(&COMMON).unwrap()).set_float_1(&ALPHA, opacity)};

        let visibility = unsafe { visibilitys.get_unchecked(event.id) }.0;
        render_obj.visibility = visibility;
    }
}

// 删除索引
impl<'a, C: Context + Share> SingleCaseListener<'a, RenderObjs<C>, DeleteEvent> for NodeAttrSys<C>{
    type ReadData = &'a SingleCaseImpl<RenderObjs<C>>;
    type WriteData = ();
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, _write: Self::WriteData){
        let render_obj = unsafe { read.get_unchecked(event.id) };
        unsafe{ self.node_render_map.get_unchecked_mut(render_obj.context).remove_item(&event.id) };
    }
}

//世界矩阵变化， 设置ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, WorldMatrix>, &'a MultiCaseImpl<Node, Transform>, &'a MultiCaseImpl<Node, Layout>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let world_matrix = cal_matrix(event.id, read.0, read.1, read.2);
        let obj_ids = unsafe{ self.node_render_map.get_unchecked(event.id) };

        for id in obj_ids.iter() {
            let mut ubos = &mut unsafe { render_objs.get_unchecked_mut(*id) }.ubos;
            let slice: &[f32; 16] = world_matrix.as_ref();
            Arc::make_mut(ubos.get_mut(&WORLD_MATRIX).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
        }
    }
}

//不透明度变化， 设置ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, Opacity>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, opacitys: Self::ReadData, render_objs: Self::WriteData){
        let opacity = unsafe { opacitys.get_unchecked(event.id).0 };
        let obj_ids = unsafe{ self.node_render_map.get_unchecked(event.id) };

        for id in obj_ids.iter() {
            let ubos = &mut unsafe { render_objs.get_unchecked_mut(*id) }.ubos;
            unsafe {Arc::make_mut(ubos.get_mut(&COMMON).unwrap()).set_float_1(&ALPHA, opacity)};
        }
    }
}

//by_overfolw变化， 设置ubo， 修改宏， 并重新创建渲染管线
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, ByOverflow>, &'a SingleCaseImpl<ClipUbo<C>>);
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (by_overflows, clip_ubo) = read;
        let (render_objs, engine) = write;
        let by_overflow = unsafe { by_overflows.get_unchecked(event.id).0 };
        let obj_ids = unsafe{ self.node_render_map.get_unchecked(event.id) };

        if by_overflow == 0 {
            for id in obj_ids.iter() {
                let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };

                // 移除ubo
                render_obj.ubos.remove(&CLIP);

                //移除宏
                render_obj.defines.remove_item(&CLIP);
                
                // 重新创建渲染管线
                let pipeline = engine.create_pipeline(
                    0,
                    &render_obj.pipeline.vs,
                    &render_obj.pipeline.fs,
                    render_obj.defines.as_slice(),
                    render_obj.pipeline.rs.clone(),
                    render_obj.pipeline.bs.clone(),
                    render_obj.pipeline.ss.clone(),
                    render_obj.pipeline.ds.clone(),
                );
                render_obj.pipeline = pipeline;
            }
        } else {
            for id in obj_ids.iter() {
                let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };

                let defines = &mut render_obj.defines;

                // 插入裁剪ubo 插入裁剪宏
                render_obj.ubos.entry(CLIP.clone()).or_insert_with(||{
                    defines.push(CLIP.clone());
                    clip_ubo.0.clone()
                });
                
                // 重新创建渲染管线
                let pipeline = engine.create_pipeline(
                    0,
                    &render_obj.pipeline.vs,
                    &render_obj.pipeline.fs,
                    render_obj.defines.as_slice(),
                    render_obj.pipeline.rs.clone(),
                    render_obj.pipeline.bs.clone(),
                    render_obj.pipeline.ss.clone(),
                    render_obj.pipeline.ds.clone(),
                );
                render_obj.pipeline = pipeline;
            }
        }
    }
}

// 设置visibility
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Visibility, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, Visibility>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, visibilitys: Self::ReadData, render_objs: Self::WriteData){
        let visibility = unsafe { visibilitys.get_unchecked(event.id).0 };
        let obj_ids = unsafe{ self.node_render_map.get_unchecked(event.id) };

        for id in obj_ids.iter() {
            let notify = render_objs.get_notify();
            unsafe {render_objs.get_unchecked_write(*id, &notify)}.set_visibility(visibility);
        }
    }
}

unsafe impl<C: Context + Share> Sync for NodeAttrSys<C>{}
unsafe impl<C: Context + Share> Send for NodeAttrSys<C>{}

impl_system!{
    NodeAttrSys<C> where [C: Context + Share],
    false,
    {
        EntityListener<Node, CreateEvent>
        EntityListener<Node, DeleteEvent>
        SingleCaseListener<RenderObjs<C>, CreateEvent>
        SingleCaseListener<RenderObjs<C>, DeleteEvent>
        MultiCaseListener<Node, WorldMatrix, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
        MultiCaseListener<Node, Visibility, ModifyEvent>
        MultiCaseListener<Node, ByOverflow, ModifyEvent>
    }
}