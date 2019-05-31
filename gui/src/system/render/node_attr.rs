/**
 *  
 */
use std::marker::PhantomData;
use std::sync::Arc;

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use hal_core::{Context, Uniforms};

use component::user::*;
use component::calc::{Visibility, WorldMatrix, Opacity, ByOverflow};
use entity::{Node};
use single::*;
use render::engine::Engine;
use system::util::*;
use system::util::constant::{PROJECT_MATRIX, WORLD_MATRIX, VIEW_MATRIX, ALPHA, CLIP, VIEW, PROJECT, WORLD, COMMON};

pub struct NodeAttrSys<C: Context + Share>{
    view_matrix_ubo: Option<Arc<Uniforms<C>>>,
    project_matrix_ubo: Option<Arc<Uniforms<C>>>,
    marker: PhantomData<C>,
}

impl<C: Context + Share> NodeAttrSys<C> {
    pub fn new() -> Self{
        NodeAttrSys {
            view_matrix_ubo: None,
            project_matrix_ubo: None,
            marker: PhantomData,
        }
    }
}

impl<'a, C: Context + Share> Runner<'a> for NodeAttrSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<ViewMatrix>,
        &'a SingleCaseImpl<ProjectionMatrix>,
    );
    type WriteData =  &'a mut SingleCaseImpl<Engine<C>>;
    fn run(&mut self, _read: Self::ReadData, _write: Self::WriteData){
    }
    fn setup(&mut self, read: Self::ReadData, engine: Self::WriteData){
        let (view_matrix, projection_matrix) = read;

        let mut view_matrix_ubo = engine.gl.create_uniforms();
        let slice: &[f32; 16] = view_matrix.0.as_ref();
        view_matrix_ubo.set_mat_4v(&VIEW_MATRIX, &slice[0..16]);
        debug_println!("view_matrix: {:?}", &slice[0..16]);

        let mut project_matrix_ubo = engine.gl.create_uniforms();
        let slice: &[f32; 16] = projection_matrix.0.as_ref();
        project_matrix_ubo.set_mat_4v(&PROJECT_MATRIX, &slice[0..16]);
        debug_println!("projection_matrix: {:?}", &slice[0..16]);

        self.view_matrix_ubo = Some(Arc::new(view_matrix_ubo));
        self.project_matrix_ubo = Some(Arc::new(project_matrix_ubo));
    }
}

impl<'a, C: Context + Share> EntityListener<'a, Node, CreateEvent> for NodeAttrSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<NodeRenderMap>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, node_render_map: Self::WriteData){
        node_render_map.create(event.id);
    }
}

impl<'a, C: Context + Share> EntityListener<'a, Node, DeleteEvent> for NodeAttrSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<NodeRenderMap>;
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, node_render_map: Self::WriteData){
        unsafe { node_render_map.destroy_unchecked(event.id) };
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
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>, &'a mut SingleCaseImpl<NodeRenderMap>);
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (clip_ubo, world_matrixs, by_overflows, opacitys, visibilitys, transforms, layouts) = read;
        let (render_objs, engine, node_render_map) = write;
        let render_obj = unsafe { render_objs.get_unchecked_mut(event.id) };
        let notify = node_render_map.get_notify();
        unsafe{ node_render_map.add_unchecked(render_obj.context, event.id, &notify) };
        
        let ubos = &mut render_obj.ubos;
        // 插入世界矩阵ubo
        let mut world_matrix_ubo = engine.gl.create_uniforms();
        let world_matrix = cal_matrix(render_obj.context, world_matrixs, transforms, layouts);
        let slice: &[f32; 16] = world_matrix.as_ref();
        world_matrix_ubo.set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
        ubos.insert(WORLD.clone(), Arc::new(world_matrix_ubo)); // WORLD_MATRIX
        debug_println!("id: {}, world_matrix: {:?}", render_obj.context, &slice[0..16]);

        ubos.insert(VIEW.clone(), self.view_matrix_ubo.clone().unwrap()); // VIEW_MATRIX
        ubos.insert(PROJECT.clone(), self.project_matrix_ubo.clone().unwrap()); // PROJECT_MATRIX

        let by_overflow = unsafe { by_overflows.get_unchecked(render_obj.context) }.0;
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

        let opacity = unsafe { opacitys.get_unchecked(render_obj.context) }.0;
        debug_println!("id: {}, alpha: {:?}", render_obj.context, opacity);
        Arc::make_mut(ubos.get_mut(&COMMON).unwrap()).set_float_1(&ALPHA, opacity);

        let visibility = unsafe { visibilitys.get_unchecked(render_obj.context) }.0;
        render_obj.visibility = visibility;
        debug_println!("id: {}, visibility: {:?}", render_obj.context, visibility);
    }
}

// 删除索引
impl<'a, C: Context + Share> SingleCaseListener<'a, RenderObjs<C>, DeleteEvent> for NodeAttrSys<C>{
    type ReadData = &'a SingleCaseImpl<RenderObjs<C>>;
    type WriteData = &'a mut SingleCaseImpl<NodeRenderMap>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, node_render_map: Self::WriteData){
        let render_obj = unsafe { read.get_unchecked(event.id) };
        let notify = node_render_map.get_notify();
        unsafe{ node_render_map.remove_unchecked(render_obj.context, event.id, &notify) };
    }
}

//世界矩阵变化， 设置ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, WorldMatrix>, &'a MultiCaseImpl<Node, Transform>, &'a MultiCaseImpl<Node, Layout>);
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<NodeRenderMap>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let world_matrix = cal_matrix(event.id, read.0, read.1, read.2);
        let (render_objs, node_render_map) = write;
        let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };

        for id in obj_ids.iter() {
            let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
            let ubos = &mut render_obj.ubos;
            let slice: &[f32; 16] = world_matrix.as_ref();
            Arc::make_mut(ubos.get_mut(&WORLD).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
            debug_println!("id: {}, world_matrix: {:?}", render_obj.context, &slice[0..16]);
        }
    }
}

//不透明度变化， 设置ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, Opacity>;
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<NodeRenderMap>);
    fn listen(&mut self, event: &ModifyEvent, opacitys: Self::ReadData, write: Self::WriteData){
        let opacity = unsafe { opacitys.get_unchecked(event.id).0 };
        let (render_objs, node_render_map) = write;
        let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };

        for id in obj_ids.iter() {
            let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
            let ubos = &mut render_obj.ubos;
            Arc::make_mut(ubos.get_mut(&COMMON).unwrap()).set_float_1(&ALPHA, opacity);
            debug_println!("id: {}, alpha: {:?}", render_obj.context, opacity);
        }
    }
}

//by_overfolw变化， 设置ubo， 修改宏， 并重新创建渲染管线
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, ByOverflow>, &'a SingleCaseImpl<ClipUbo<C>>);
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>, &'a mut SingleCaseImpl<NodeRenderMap>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (by_overflows, clip_ubo) = read;
        let (render_objs, engine, node_render_map) = write;
        let by_overflow = unsafe { by_overflows.get_unchecked(event.id).0 };
        let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };

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
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<NodeRenderMap>);
    fn listen(&mut self, event: &ModifyEvent, visibilitys: Self::ReadData, write: Self::WriteData){
        let (render_objs, node_render_map) = write;
        let visibility = unsafe { visibilitys.get_unchecked(event.id).0 };
        let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };

        for id in obj_ids.iter() {
            let notify = render_objs.get_notify();
            let mut render_obj = unsafe {render_objs.get_unchecked_write(*id, &notify)};
            render_obj.set_visibility(visibility);
            debug_println!("id: {}, visibility: {:?}", render_obj.value.context, visibility);
        }
    }
}

unsafe impl<C: Context + Share> Sync for NodeAttrSys<C>{}
unsafe impl<C: Context + Share> Send for NodeAttrSys<C>{}

impl_system!{
    NodeAttrSys<C> where [C: Context + Share],
    true,
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