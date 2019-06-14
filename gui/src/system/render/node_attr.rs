/**
 *  
 */
use std::marker::PhantomData;
use std::sync::Arc;

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use hal_core::*;
use atom::Atom;

use component::user::*;
use component::calc::{Visibility, WorldMatrix, Opacity, ZDepth, HSV};
use entity::{Node};
use single::*;
use render::engine::Engine;
use system::util::*;
use system::util::constant::*;
use Z_MAX;

lazy_static! {
    static ref Z_DEPTH: Atom = Atom::from("zDepth");
    static ref HSV_MACRO: Atom = Atom::from("HSV");
    static ref HSV_ATTR: Atom = Atom::from("hsvValue");
}

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
        &'a MultiCaseImpl<Node, HSV>,
        &'a SingleCaseImpl<DefaultTable>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>, &'a mut SingleCaseImpl<NodeRenderMap>);
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (world_matrixs, opacitys, visibilitys, transforms, layouts, hsvs, default_table) = read;
        let (render_objs, engine, node_render_map) = write;
        let render_obj = unsafe { render_objs.get_unchecked_mut(event.id) };
        let notify = node_render_map.get_notify();
        let mut defines_change = false;
        unsafe{ node_render_map.add_unchecked(render_obj.context, event.id, &notify) };
        
        let ubos = &mut render_obj.ubos;
        // 插入世界矩阵ubo
        let mut world_matrix_ubo = engine.gl.create_uniforms();
        let world_matrix = cal_matrix(render_obj.context, world_matrixs, transforms, layouts, default_table);
        let slice: &[f32; 16] = world_matrix.as_ref();
        world_matrix_ubo.set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
        ubos.insert(WORLD.clone(), Arc::new(world_matrix_ubo)); // WORLD_MATRIX
        debug_println!("id: {}, world_matrix: {:?}", render_obj.context, &slice[0..16]);

        let mut z_depth_ubo = engine.gl.create_uniforms();
        z_depth_ubo.set_float_1(&Z_DEPTH, -render_obj.depth/Z_MAX);
        ubos.insert(Z_DEPTH.clone(), Arc::new(z_depth_ubo)); // Z_DEPTH
        debug_println!("id: {}, z_depth: {:?}", render_obj.context, -render_obj.depth/Z_MAX);

        ubos.insert(VIEW.clone(), self.view_matrix_ubo.clone().unwrap()); // VIEW_MATRIX
        ubos.insert(PROJECT.clone(), self.project_matrix_ubo.clone().unwrap()); // PROJECT_MATRIX
       

        let opacity = unsafe { opacitys.get_unchecked(render_obj.context) }.0;
        debug_println!("id: {}, alpha: {:?}", render_obj.context, opacity);
        Arc::make_mut(ubos.get_mut(&COMMON).unwrap()).set_float_1(&ALPHA, opacity);

        let visibility = unsafe { visibilitys.get_unchecked(render_obj.context) }.0;
        render_obj.visibility = visibility;
        debug_println!("id: {}, visibility: {:?}", render_obj.context, visibility);

        let hsv = unsafe { hsvs.get_unchecked(render_obj.context) };
        if !(hsv.h == 0.0 && hsv.s == 0.0 && hsv.v == 1.0) {
            defines_change = true;
            render_obj.defines.push(HSV_MACRO.clone());
            let mut hsv_ubo = engine.gl.create_uniforms();
            hsv_ubo.set_float_3(&HSV_ATTR, cal_hue(hsv.h), hsv.s, hsv.v - 1.0);
            render_obj.ubos.insert(HSV_MACRO.clone(), Arc::new(hsv_ubo));
        }
        
        let mut start_hash = 0;
        if !render_obj.is_opacity {
            start_hash = 1;
        }

        // println!("render_obj is_opacity id: {}, is_opacity: {:?}", render_obj.context, render_obj.is_opacity);
        if !render_obj.is_opacity || defines_change {   
            let pipeline = &render_obj.pipeline;
            let mut bs = pipeline.bs.clone();
            let mut ds = pipeline.ds.clone();
            Arc::make_mut(&mut bs).set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
            Arc::make_mut(&mut ds).set_write_enable(false);
            let pipeline = engine.create_pipeline(
                start_hash,
                &pipeline.vs,
                &pipeline.fs,
                render_obj.defines.as_slice(),
                pipeline.rs.clone(),
                bs,
                pipeline.ss.clone(),
                ds,
            );
            render_obj.pipeline = pipeline;
            render_objs.get_notify().modify_event(event.id, "pipeline", 0);
        }
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
                let mut ds = pipeline.ds.clone();
                if render_obj.is_opacity == false {
                    Arc::make_mut(&mut bs).set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
                    Arc::make_mut(&mut ds).set_write_enable(false);
                    // println!("is_opacity false---------------------{}", render_obj.context);
                    let pipeline = engine.create_pipeline(
                        1,
                        &pipeline.vs,
                        &pipeline.fs,
                        pipeline.defines.as_slice(),
                        pipeline.rs.clone(),
                        bs,
                        pipeline.ss.clone(),
                        ds,
                    );
                    render_obj.pipeline = pipeline;
                } else {
                    Arc::make_mut(&mut bs).set_rgb_factor(BlendFactor::One, BlendFactor::Zero);
                    Arc::make_mut(&mut ds).set_write_enable(true);
                    // println!("is_opacity true---------------------{}", render_obj.context);
                    let pipeline = engine.create_pipeline(
                        0,
                        &pipeline.vs,
                        &pipeline.fs,
                        pipeline.defines.as_slice(),
                        pipeline.rs.clone(),
                        bs,
                        pipeline.ss.clone(),
                        pipeline.ds.clone(),
                    );
                    render_obj.pipeline = pipeline;
                }
                render_objs.get_notify().modify_event(event.id, "pipeline", 0);
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
    type ReadData = (&'a MultiCaseImpl<Node, WorldMatrix>, &'a MultiCaseImpl<Node, Transform>, &'a MultiCaseImpl<Node, Layout>, &'a SingleCaseImpl<DefaultTable>);
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<NodeRenderMap>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let world_matrix = cal_matrix(event.id, read.0, read.1, read.2, read.3);
        // let world_matrix = unsafe { read.0.get_unchecked(event.id) };
        let (render_objs, node_render_map) = write;
        let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };

        for id in obj_ids.iter() {
            let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
            let ubos = &mut render_obj.ubos;
            let slice: &[f32; 16] = world_matrix.as_ref();
            Arc::make_mut(ubos.get_mut(&WORLD).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
            debug_println!("id: {}, world_matrix: {:?}", render_obj.context, &slice[0..16]);
            render_objs.get_notify().modify_event(*id, "ubos", 0);
        }
    }
}

//世界矩阵变化， 设置ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, ZDepth, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, ZDepth>;
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<NodeRenderMap>);
    fn listen(&mut self, event: &ModifyEvent, z_depths: Self::ReadData, write: Self::WriteData){
        let (render_objs, node_render_map) = write;
        let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };
        let z_depth = unsafe{ z_depths.get_unchecked(event.id) }.0;

        for id in obj_ids.iter() {
            let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
            render_obj.depth = z_depth + render_obj.depth_diff;
            let ubos = &mut render_obj.ubos;
            Arc::make_mut(ubos.get_mut(&Z_DEPTH).unwrap()).set_float_1(&Z_DEPTH, -render_obj.depth/Z_MAX);
            debug_println!("id: {}, z_depth: {:?}", render_obj.context, -render_obj.depth/Z_MAX);
            // println!("xxxxxxxxxxx, z_depth: {:?}, id: {}", render_obj.depth + render_obj.depth_diff, render_obj.context);
            render_objs.get_notify().modify_event(*id, "depth", 0);
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
            render_objs.get_notify().modify_event(*id, "ubos", 0);
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

// 设置hsv
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, HSV, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, HSV>;
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<NodeRenderMap>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, hsvs: Self::ReadData, write: Self::WriteData){
        let (render_objs, node_render_map, engine) = write;
        let hsv = unsafe { hsvs.get_unchecked(event.id) };
        let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };

        println!("HSV modify------------------------------{:?}", hsv);
        if !(hsv.h == 0.0 && hsv.s == 0.0 && hsv.v == 1.0) {
            for id in obj_ids.iter() {
                let render_obj = unsafe {render_objs.get_unchecked_mut(*id)};
                if add_or_modify_hsv(hsv, render_obj, engine) {
                    render_objs.get_notify().modify_event(*id, "pipeline", 0);
                }
                render_objs.get_notify().modify_event(*id, "ubos", 0);
            }
        } else {
            for id in obj_ids.iter() {
                // 移除宏， 移除ubo， 创建渲染管线
                let mut render_obj = unsafe {render_objs.get_unchecked_mut(*id)};
                match render_obj.ubos.remove(&HSV_MACRO) {
                    Some(_) => (),
                    None => continue,
                };

                render_obj.defines.remove_item(&HSV_MACRO);

                let pipeline = engine.create_pipeline(
                    render_obj.pipeline.start_hash,
                    &render_obj.pipeline.vs,
                    &render_obj.pipeline.fs,
                    render_obj.defines.as_slice(),
                    render_obj.pipeline.rs.clone(),
                    render_obj.pipeline.bs.clone(),
                    render_obj.pipeline.ss.clone(),
                    render_obj.pipeline.ds.clone(),
                );
                render_obj.pipeline = pipeline;
                render_objs.get_notify().modify_event(*id, "pipeline", 0);
            }
        }
        
    }
}

// 修改或添加hsv， 如果是添加， 返回true
fn add_or_modify_hsv<C: Context + Share>(hsv: &HSV, render_obj: &mut RenderObj<C>, engine: &mut SingleCaseImpl<Engine<C>>) -> bool{
    let defines = &mut render_obj.defines;
    let ubos = &mut render_obj.ubos;
    let id = render_obj.context;
    let mut define_change = false;
    // 插入裁剪ubo 插入裁剪宏
    ubos.entry(HSV_MACRO.clone()).and_modify(|hsv_ubo: &mut Arc<Uniforms<C>>|{
        Arc::make_mut(hsv_ubo).set_float_3(&HSV_ATTR, cal_hue(hsv.h), hsv.s, hsv.v - 1.0);
        println!("HSV modify1------------------------------{:?}", hsv);
        debug_println!("id: {}, hsv: {:?}", id, hsv);
    }).or_insert_with(||{
        println!("HSV modify2------------------------------{:?}", hsv);
        defines.push(HSV_MACRO.clone());
        define_change = true;
        let mut hsv_ubo = engine.gl.create_uniforms();
        hsv_ubo.set_float_3(&HSV_ATTR, cal_hue(hsv.h), hsv.s, hsv.v - 1.0);
        debug_println!("id: {}, hsv: {:?}", id, hsv);
        Arc::new(hsv_ubo)
    });

    if define_change {
        // 重新创建渲染管线
        let pipeline = engine.create_pipeline(
            render_obj.pipeline.start_hash,
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
    return define_change;
}

// 参数0-360， 返回-1~1
fn cal_hue(value : f32) -> f32{
    let v = value/180.0;
    if v > 1.0 {
        -(2.0 - v)
    } else {
        v
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
        SingleCaseListener<RenderObjs<C>, ModifyEvent>
        SingleCaseListener<RenderObjs<C>, DeleteEvent>
        MultiCaseListener<Node, WorldMatrix, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
        MultiCaseListener<Node, Visibility, ModifyEvent>
        MultiCaseListener<Node, ZDepth, ModifyEvent>
        MultiCaseListener<Node, HSV, ModifyEvent>
    }
}