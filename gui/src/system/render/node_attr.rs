/**
 *  
 */
use std::marker::PhantomData;
use std::sync::Arc;

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use hal_core::*;

use component::user::*;
use component::calc::{Visibility, WorldMatrix, Opacity};
use entity::{Node};
use single::*;
use render::engine::Engine;
use system::util::*;
use system::util::constant::*;

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
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Layout>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>, &'a mut SingleCaseImpl<NodeRenderMap>);
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (world_matrixs, opacitys, visibilitys, transforms, layouts) = read;
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

        let opacity = unsafe { opacitys.get_unchecked(render_obj.context) }.0;
        debug_println!("id: {}, alpha: {:?}", render_obj.context, opacity);
        Arc::make_mut(ubos.get_mut(&COMMON).unwrap()).set_float_1(&ALPHA, opacity);

        let visibility = unsafe { visibilitys.get_unchecked(render_obj.context) }.0;
        render_obj.visibility = visibility;
        debug_println!("id: {}, visibility: {:?}", render_obj.context, visibility);
    }
}

// 监听is_opacity的修改，修改渲染状态， 创建新的渲染管线
impl<'a, C: Context + Share> SingleCaseListener<'a, RenderObjs<C>, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = ();
    type WriteData = ( &'a mut SingleCaseImpl<RenderObjs<C>>,  &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        match event.field {
            "is_opacity" => {
                let (render_objs, engine) = write;
                let render_obj = unsafe { render_objs.get_unchecked_mut(event.id) };
                let pipeline = &render_obj.pipeline;
                let mut bs = pipeline.bs.clone();
                if render_obj.is_opacity == true {
                    Arc::make_mut(&mut bs).set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
                
                    let pipeline = engine.create_pipeline(
                        1,
                        &pipeline.vs,
                        &pipeline.vs,
                        pipeline.defines.as_slice(),
                        pipeline.rs.clone(),
                        bs,
                        pipeline.ss.clone(),
                        pipeline.ds.clone(),
                    );
                    render_obj.pipeline = pipeline;
                } else {
                    Arc::make_mut(&mut bs).set_rgb_factor(BlendFactor::One, BlendFactor::Zero);
                
                    let pipeline = engine.create_pipeline(
                        0,
                        &pipeline.vs,
                        &pipeline.vs,
                        pipeline.defines.as_slice(),
                        pipeline.rs.clone(),
                        bs,
                        pipeline.ss.clone(),
                        pipeline.ds.clone(),
                    );
                    render_obj.pipeline = pipeline;
                } 
            },
            _ => (),
        }
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
        // MultiCaseListener<Node, ByOverflow, ModifyEvent>
    }
}