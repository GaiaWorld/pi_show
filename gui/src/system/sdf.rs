/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use std::sync::Arc;
use std::mem::transmute;

use std::collections::HashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share};
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
use system::util::constant::{PROJECT_MATRIX, WORLD_MATRIX, VIEW_MATRIX, POSITION, COLOR, CLIP_indices, ALPHA, CLIP, VIEW, PROJECT, WORLD, COMMON};


lazy_static! {
    static ref BOX_SHADER_NAME: Atom = Atom::from("box");
    static ref BOX_FS_SHADER_NAME: Atom = Atom::from("box_fs");
    static ref BOX_VS_SHADER_NAME: Atom = Atom::from("box_vs");

    static ref STROKE: Atom = Atom::from("ATROKE");
    static ref UCOLOR: Atom = Atom::from("UCOLOR");
    static ref VERTEX_COLOR: Atom = Atom::from("VERTEX_COLOR");

    static ref BLUR: Atom = Atom::from("blur");
    static ref RADIUS: Atom = Atom::from("radius");
    static ref STROKE_SIZE: Atom = Atom::from("strokeSize");
    static ref STROKE_COLOR: Atom = Atom::from("strokeColor");
    static ref SIZE_TYPE: Atom = Atom::from("sizeType");
    static ref U_COLOR: Atom = Atom::from("uColor");
    

    // static ref CLIP_TEXTURE: Atom = Atom::from("clipTexture");
    // static ref CLIP_indices_SIZE: Atom = Atom::from("clipTextureSize");
}

pub struct SdfSys<C: Context + Share>{
    box_color_render_map: VecMap<Item>,
    box_shadow_render_map: VecMap<Item>,
    mark: PhantomData<C>,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
    pipelines: HashMap<u64, Arc<Pipeline>>,
}

impl<C: Context + Share> SdfSys<C> {
    pub fn new() -> Self{
        SdfSys {
            box_color_render_map: VecMap::default(),
            box_shadow_render_map: VecMap::default(),
            mark: PhantomData,
            rs: Arc::new(RasterState::new()),
            bs: Arc::new(BlendState::new()),
            ss: Arc::new(StencilState::new()),
            ds: Arc::new(DepthState::new()),
            pipelines: HashMap::default(),
        }
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BoxColor, CreateEvent> for SdfSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<ViewUbo<C>>,
        &'a SingleCaseImpl<ProjectionUbo<C>>,
        &'a SingleCaseImpl<ClipUbo<C>>,
        &'a MultiCaseImpl<Node, BoxColor>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, BorderRadius>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs<C>>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &CreateEvent, r: Self::ReadData, w: Self::WriteData){
        let mut defines = Defines::default();
        let mut geometry = create_geometry(&mut w.1.gl);
        let box_color = unsafe { r.3.get_unchecked(event.id) };
        let layout = unsafe { r.9.get_unchecked(event.id) };
        let z_depth = unsafe { r.4.get_unchecked(event.id) }.0 - 0.1;
        let mut ubos: HashMap<Atom, Arc<Uniforms<C>>> = HashMap::default();

        // if layout.border > 0.0 {
        //     // defines.stroke = true;
        //     let border_color = &box_color.border;
        //     let mut stroke_ubo = w.1.gl.create_uniforms();
        //     stroke_ubo.set_float_1(&STROKE_SIZE, layout.border);
        //     stroke_ubo.set_float_4(&STROKE_COLOR, border_color.r, border_color.g, border_color.b, border_color.a); // 描边属性
        //     ubos.insert(STROKE.clone(), Arc::new(stroke_ubo)); // COMMON
        // }

        // 设置u_color宏或vertex_color宏， 设置顶点流，索引流，颜色流
        Self::change_color(event.id, &box_color.background, &mut defines, &mut ubos, r.11, r.4, r.9, &mut geometry, w.1);

        let index = self.create_sdf_renderobjs(event.id, 1.0, z_depth, false, ubos, &mut defines, geometry, r.0, r.1, r.2, r.5, r.6, r.7, r.8, r.9, r.10, r.11, w.0, w.1);
        self.box_color_render_map.insert(event.id, Item{index: index, defines: defines});
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BoxShadow, CreateEvent> for SdfSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<ViewUbo<C>>,
        &'a SingleCaseImpl<ProjectionUbo<C>>,
        &'a SingleCaseImpl<ClipUbo<C>>,
        &'a MultiCaseImpl<Node, BoxShadow>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, BorderRadius>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs<C>>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &CreateEvent, r: Self::ReadData, w: Self::WriteData){
        let mut defines = Defines::default();
        let mut geometry = create_geometry(&mut w.1.gl);
        let box_shadow = unsafe { r.3.get_unchecked(event.id) };
        let layout = unsafe { r.9.get_unchecked(event.id) };
        let z_depth = unsafe { r.4.get_unchecked(event.id) }.0 - 0.2;
        let mut ubos: HashMap<Atom, Arc<Uniforms<C>>> = HashMap::default();
        
        let shadow_color = &box_shadow.color;
        let mut color_ubo = w.1.gl.create_uniforms();
        defines.u_color = true;  
        color_ubo.set_float_4(&U_COLOR, shadow_color.a, shadow_color.g, shadow_color.b, shadow_color.a);
        ubos.insert(UCOLOR.clone(), Arc::new(color_ubo)); // COLOR 属性

        //如果layout > 0.0, 表示该节点曾经布局过, 设置position
        if layout.width > 0.0 {
            let (h_offset, v_offset) = (layout.width + box_shadow.h, layout.height + box_shadow.v);
            let buffer = [
                box_shadow.h, box_shadow.v, z_depth, // left_top
                box_shadow.h, v_offset,     z_depth, // left_bootom
                h_offset,     v_offset,     z_depth, // right_bootom
                h_offset,     box_shadow.v, z_depth, // right_top
            ];
            Arc::get_mut(&mut geometry).unwrap().set_attribute(&AttributeName::Position, 3, Some(&buffer[0..12]), false);
        }

        let index = self.create_sdf_renderobjs(event.id, box_shadow.blur, z_depth, false, ubos, &mut defines, geometry, r.0, r.1, r.2, r.5, r.6, r.7, r.8, r.9, r.10, r.11, w.0, w.1);
        self.box_shadow_render_map.insert(event.id, Item{index: index, defines: defines});
    }
}

// 删除渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BoxColor, DeleteEvent> for SdfSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData){
        let item = self.box_color_render_map.remove(event.id).unwrap();
        let notify = write.get_notify();
        write.remove(item.index, Some(notify));
    }
}

// 删除渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BoxShadow, DeleteEvent> for SdfSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData){
        let item = self.box_shadow_render_map.remove(event.id).unwrap();
        let notify = write.get_notify();
        write.remove(item.index, Some(notify));
    }
}

// BoxColor变化, 修改ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BoxColor, ModifyEvent> for SdfSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, BoxColor>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Layout>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (box_colors, border_radiuss, z_depths, layouts) = read;
        let box_color = unsafe { box_colors.get_unchecked(event.id) };
        match event.field {
            "background" => {
                let item = unsafe {self.box_color_render_map.get_unchecked_mut(event.id)};
                let render_obj = unsafe {write.0.get_unchecked_mut(item.index)};
                // 设置u_color宏或vertex_color宏， 设置顶点流，索引流，颜色流
                let defines_change = Self::change_color(event.id, &box_color.background, &mut item.defines, &mut render_obj.ubos, read.1, read.2, read.3, &mut render_obj.geometry, write.1);
                //如果宏改变，创建渲染管线
                if defines_change {
                    let pipeline = write.1.create_pipeline(0, &BOX_VS_SHADER_NAME.clone(), &BOX_FS_SHADER_NAME.clone(), item.defines.list().as_slice(), self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone());
                    render_obj.pipeline = pipeline;
                }
            },
            "border" => {
                let layout = unsafe { layouts.get_unchecked(event.id) };
                if layout.border <= 0.0 {
                    return;
                }
                let item = unsafe { self.box_color_render_map.get_unchecked(event.id) };
                let mut ubos = &mut unsafe { write.0.get_unchecked_mut(item.index) }.ubos;
                let gl = &mut write.1.gl;
                ubos.entry(STROKE.clone()).and_modify(|stroke_ubo|{
                    let border_color = &box_color.border;
                    Arc::make_mut(stroke_ubo).set_float_4(&STROKE_COLOR, border_color.r, border_color.g, border_color.b, border_color.a);
                }).or_insert_with(|| {
                    let border_color = &box_color.border;
                    let mut stroke_ubo = gl.create_uniforms();
                    stroke_ubo.set_float_1(&STROKE_SIZE, layout.border);
                    stroke_ubo.set_float_4(&STROKE_COLOR, border_color.r, border_color.g, border_color.b, border_color.a);
                    Arc::new(stroke_ubo)
                });
            },
            _ => (),
        }
    }
}

// BoxShadow变化, 修改ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BoxShadow, ModifyEvent> for SdfSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, BoxShadow>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Layout>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (box_shadows, border_radiuss, z_depths, layouts) = read;
        let box_shadow= unsafe { box_shadows.get_unchecked(event.id) };
        let item = unsafe {self.box_shadow_render_map.get_unchecked_mut(event.id)};
        let render_obj = unsafe {write.0.get_unchecked_mut(item.index)};
        match event.field {
            "color" => {
                let color_ubo = render_obj.ubos.get_mut(&UCOLOR).unwrap(); //设置ubo
                Arc::make_mut(color_ubo).set_float_4(&U_COLOR, box_shadow.color.a, box_shadow.color.g, box_shadow.color.b, box_shadow.color.a);
            },
            "h" | "v" => {
                let z_depth = unsafe {z_depths.get_unchecked(event.id)}.0;
                Self::set_attr_for_cgcolor(event.id, z_depth - 0.2, border_radiuss, layouts, &mut render_obj.geometry);
            },
            "blur" => {
                let common_ubo = render_obj.ubos.get_mut(&COMMON).unwrap(); //设置ubo
                Arc::make_mut(common_ubo).set_float_1(&BLUR, box_shadow.blur);
            },
            _ => (),
        }
    }
}

//世界矩阵变化， 设置ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for SdfSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, WorldMatrix>, &'a MultiCaseImpl<Node, Transform>, &'a MultiCaseImpl<Node, Layout>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        match (unsafe { self.box_color_render_map.get(event.id) }, unsafe { self.box_shadow_render_map.get(event.id) }) {
            (Some(item), None) => {    
                let world_matrix = cal_matrix(event.id, read.0, read.1, read.2);
                set_world_matrix_ubo(event.id, item.index, &world_matrix, write);
            },
            (None, Some(item)) => {    
                let world_matrix = cal_matrix(event.id, read.0, read.1, read.2);
                set_world_matrix_ubo(event.id, item.index, &world_matrix, write);
            },
            (Some(item), Some(item1)) => {    
                let world_matrix = cal_matrix(event.id, read.0, read.1, read.2);
                set_world_matrix_ubo(event.id, item.index, &world_matrix, write);
                set_world_matrix_ubo(event.id, item1.index, &world_matrix, write);
            },
            (None, None) => return,
        };
    }
}

//世界矩阵变化， 设置ubo, 修改宏， 并重新创建渲染管线
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Layout, ModifyEvent> for SdfSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, BoxColor>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Layout>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (box_colors, border_radiuss, z_depths, layouts) = read;
        let (render_objs, engine) = write;
        if let Some(item) = unsafe { self.box_shadow_render_map.get_mut(event.id) } {
            let item = unsafe {self.box_shadow_render_map.get_unchecked_mut(event.id)};
            let render_obj = unsafe {render_objs.get_unchecked_mut(item.index)};
            let z_depth = unsafe {z_depths.get_unchecked(event.id)}.0;
            Self::set_attr_for_cgcolor(event.id, z_depth - 0.2, border_radiuss, layouts, &mut render_obj.geometry);
        };
        match unsafe { self.box_color_render_map.get_mut(event.id) } {
            Some(item) => {
                let layout = unsafe { layouts.get_unchecked(event.id) };
                let mut obj = &mut unsafe { render_objs.get_unchecked_mut(item.index) };
                let ubos = &mut obj.ubos;
                if layout.border <= 0.0 {
                    if item.defines.stroke {
                        ubos.remove(&STROKE);
                        item.defines.stroke = false;
                        obj.pipeline = engine.create_pipeline(0, &BOX_VS_SHADER_NAME.clone(), &BOX_FS_SHADER_NAME.clone(), item.defines.list().as_slice(), self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone());
                    }
                    return;
                }
                Arc::make_mut(ubos.get_mut(&STROKE).unwrap()).set_float_1(&STROKE_SIZE, layout.border);
                if item.defines.stroke == false {
                    item.defines.stroke = true;
                    obj.pipeline = engine.create_pipeline(0, &BOX_VS_SHADER_NAME.clone(), &BOX_FS_SHADER_NAME.clone(), item.defines.list().as_slice(), self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone());
                }
                let z_depth = unsafe {z_depths.get_unchecked(event.id)}.0;
                self.layout_change(event.id, z_depth - 0.1,  box_colors, border_radiuss, layouts, render_objs);
            },
            None => return,
        };
    }
}

//不透明度变化， 设置ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for SdfSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, BoxColor>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        if let Some(item) = unsafe { self.box_color_render_map.get_mut(event.id) } {
            let opacity = unsafe { read.0.get_unchecked(event.id).0 };
            let box_color = unsafe { read.1.get_unchecked(event.id) };
            let is_opacity = box_is_opacity(opacity, &box_color.background, &box_color.border);
            let notify = write.get_notify();
            unsafe { write.get_unchecked_write(item.index, &notify).set_is_opacity(is_opacity) };

            let ubos = &mut unsafe { write.get_unchecked_mut(item.index) }.ubos;
            unsafe {Arc::make_mut(ubos.get_mut(&COMMON).unwrap()).set_float_1(&ALPHA, opacity)};
        }
        if let Some(item) = unsafe { self.box_shadow_render_map.get_mut(event.id) } {
            let opacity = unsafe { read.0.get_unchecked(event.id).0 };
            let ubos = &mut unsafe { write.get_unchecked_mut(item.index) }.ubos;
            unsafe {Arc::make_mut(ubos.get_mut(&COMMON).unwrap()).set_float_1(&ALPHA, opacity)};
        }
    }
}

//by_overfolw变化， 设置ubo， 修改宏， 并重新创建渲染管线
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for SdfSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, ByOverflow>;
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        if let Some(item) = unsafe { self.box_color_render_map.get_mut(event.id) } {
            let by_overflow = unsafe { read.get_unchecked(event.id) }.0;
            by_overflow_change(by_overflow, item.index, &mut item.defines, self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone(), 0, &BOX_FS_SHADER_NAME, &BOX_VS_SHADER_NAME, write.0, write.1);
        }
        if let Some(item) = unsafe { self.box_shadow_render_map.get_mut(event.id) } {
            let by_overflow = unsafe { read.get_unchecked(event.id) }.0;
            by_overflow_change(by_overflow, item.index, &mut item.defines, self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone(), 0, &BOX_FS_SHADER_NAME, &BOX_VS_SHADER_NAME, write.0, write.1);
        }
    }
}

// 设置visibility
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Visibility, ModifyEvent> for SdfSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, Visibility>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        if let Some(item) = unsafe { self.box_color_render_map.get_mut(event.id) } {
            let notify = write.get_notify();
            unsafe {write.get_unchecked_write(item.index, &notify)}.set_visibility(unsafe {read.get_unchecked(event.id).0});
        }
        if let Some(item) = unsafe { self.box_shadow_render_map.get_mut(event.id) } {
            let notify = write.get_notify();
            unsafe {write.get_unchecked_write(item.index, &notify)}.set_visibility(unsafe {read.get_unchecked(event.id).0});
        }
    }
}

impl<C: Context + Share> SdfSys<C> {
    fn create_sdf_renderobjs(
        &mut self,
        id: usize,
        blur: f32,
        z_depth: f32,
        is_opacity: bool,
        mut ubos: HashMap<Atom, Arc<Uniforms<C>>>,
        defines: &mut Defines,
        mut geometry: Arc<<C as Context>::ContextGeometry>,
        view_ubo: & SingleCaseImpl<ViewUbo<C>>,
        projection_ubo: & SingleCaseImpl<ProjectionUbo<C>>,
        clip_ubo: & SingleCaseImpl<ClipUbo<C>>,
        visibility: & MultiCaseImpl<Node, Visibility>,
        opacity: & MultiCaseImpl<Node, Opacity>,
        world_matrix: & MultiCaseImpl<Node, WorldMatrix>,
        transform: & MultiCaseImpl<Node, Transform>,
        layout: & MultiCaseImpl<Node, Layout>,
        by_overflow: & MultiCaseImpl<Node, ByOverflow>,
        border_radius: & MultiCaseImpl<Node, BorderRadius>,
        render_objs: & mut SingleCaseImpl<RenderObjs<C>>,
        engine: & mut SingleCaseImpl<Engine<C>>,
    ) -> usize {
        let opacity = unsafe { opacity.get_unchecked(id) }.0; 

        ubos.insert(VIEW_MATRIX.clone(), view_ubo.0.clone());//  视图矩阵
        ubos.insert(PROJECT_MATRIX.clone(), projection_ubo.0.clone()); // 投影矩阵

        let world_matrix = cal_matrix(id, world_matrix, transform, layout);
        let world_matrix: &[f32; 16] = world_matrix.as_ref();
        let mut world_matrix_ubo = engine.gl.create_uniforms();
        world_matrix_ubo.set_mat_4v(&WORLD_MATRIX, &world_matrix[0..16]);
        ubos.insert(WORLD_MATRIX.clone(), Arc::new(world_matrix_ubo)); //世界矩阵

        let mut common_ubo = engine.gl.create_uniforms();
        common_ubo.set_float_1(&BLUR, blur);
        common_ubo.set_float_1(&ALPHA, opacity);
        ubos.insert(COMMON.clone(), Arc::new(common_ubo)); // COMMON

        let by_overflow =  unsafe { by_overflow.get_unchecked(id) }.0;
        if by_overflow > 0 {
            defines.clip = true;
            let mut by_overflow_ubo = engine.gl.create_uniforms();
            by_overflow_ubo.set_float_1(&CLIP_indices, by_overflow as f32); //裁剪属性，
        }

        let pipeline = engine.create_pipeline(0, &BOX_VS_SHADER_NAME.clone(), &BOX_FS_SHADER_NAME.clone(), defines.list().as_slice(), self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone());

        let render_obj: RenderObj<C> = RenderObj {
            depth: z_depth,
            visibility: unsafe { visibility.get_unchecked(id) }.0,
            is_opacity: is_opacity,
            ubos: ubos,
            geometry: geometry,
            pipeline: pipeline.clone(),
        };

        let notify = render_objs.get_notify();
        let index = render_objs.insert(render_obj, Some(notify));
        index
    }

    fn layout_change(
        &mut self,
        id: usize,
        z_depth: f32,
        box_colors: &MultiCaseImpl<Node, BoxColor>,
        border_radiuss: &MultiCaseImpl<Node, BorderRadius>,
        layouts: &MultiCaseImpl<Node, Layout>,
        render_objs: &mut SingleCaseImpl<RenderObjs<C>>
    ){
        let box_color = unsafe { box_colors.get_unchecked(id) };
        let layout = unsafe { layouts.get_unchecked(id) };
        let item = unsafe { self.box_color_render_map.get_unchecked_mut(id) };
        let defines = &mut item.defines;
        let geometry = &mut unsafe { render_objs.get_unchecked_mut(item.index) }.geometry;

        match &box_color.background {
            Color::RGBA(r) => {
                if defines.vertex_color == true {
                    Self::set_attr_for_cgcolor(id, z_depth, border_radiuss, layouts, geometry);
                }
            },
            Color::LinearGradient(r) => {
                if defines.u_color == true {
                    Self::set_attr_for_linear_gradient(id, z_depth, r, border_radiuss, layouts, geometry);
                }
            },
            Color::RadialGradient(r) => {
                panic!("error, RadialGradient is not suported");
            }
        }
    }

    fn set_attr_for_cgcolor(
        id: usize,
        z_depth: f32,
        border_radiuss: &MultiCaseImpl<Node, BorderRadius>,
        layouts: &MultiCaseImpl<Node, Layout>,
        geometry: &mut Arc<<C as Context>::ContextGeometry>,
    ){
        let border_radius = unsafe { border_radiuss.get_unchecked(id) };
        let layout = unsafe { layouts.get_unchecked(id) };
        let geometry = Arc::get_mut(geometry).unwrap();

        let radius = cal_border_radius(border_radius, layout);
        let position = split_by_radius(0.0, 0.0, layout.width, layout.height, radius.x, z_depth);

        let vertex_count: u32 = (position.len()/3) as u32;
        if vertex_count != geometry.get_vertex_count() {
            geometry.set_vertex_count(vertex_count);
        }
        geometry.set_attribute(&AttributeName::Position, 3, Some(position.as_slice()), false);
    }

    fn set_attr_for_linear_gradient(
        id: usize,
        z_depth: f32,
        bg_colors: &LinearGradientColor,
        border_radiuss: &BorderRadius,
        layouts: &Layout,
        geometry: &mut Arc<<C as Context>::ContextGeometry>,
    ){
        let geometry = Arc::get_mut(geometry).unwrap();

        let mut lg_pos = Vec::with_capacity(bg_colors.list.len());
        let mut color = Vec::with_capacity(bg_colors.list.len() * 4);
        for v in bg_colors.list.iter() {
            lg_pos.push(v.position);
            color.extend_from_slice(&[v.rgba.r, v.rgba.g, v.rgba.b, v.rgba.a]);
        }

        let radius = cal_border_radius(border_radius, layout);
        let position = split_by_radius(0.0, 0.0, layout.width, layout.height, radius.x, z_depth);
        let (position, indices) = split_by_lg(position, lg_pos.as_slice(), (0.0, 0.0), (layout.width, layout.height)); // 计算end TODO
        let colors = interp_by_lg(position.as_slice(), vec![LgCfg{unit:4, data: color}], lg_pos.as_slice(), (0.0, 0.0), (layout.width, layout.height));// 计算end TODO

        let vertex_count: u32 = (position.len()/3) as u32;
        if vertex_count != geometry.get_vertex_count() {
            geometry.set_vertex_count(vertex_count);
        }
        geometry.set_attribute(&AttributeName::Position, 3, Some(position.as_slice()), false);
        geometry.set_attribute(&AttributeName::Color, 4, Some(colors[0].as_slice()), false);
        geometry.set_indices_short(indices.as_slice(), false);
    }

    fn change_color(
        id: usize,
        bg_color: &Color,
        defines: &mut Defines,
        ubos: &mut HashMap<Atom, Arc<Uniforms<C>>>,
        border_radiuss: &MultiCaseImpl<Node, BorderRadius>,
        z_depths: &MultiCaseImpl<Node, ZDepth>,
        layouts: &MultiCaseImpl<Node, Layout>,
        geometry: &mut Arc<<C as Context>::ContextGeometry>,
        engine: &mut Engine<C>,
    ) -> bool{
        match (bg_color, defines.vertex_color, defines.u_color) {
            (Color::RGBA(r), _, false) => {
                let z_depth = unsafe { z_depths.get_unchecked(id) }.0;
                Self::set_attr_for_cgcolor(id, z_depth - 0.1, border_radiuss, layouts, geometry);
                defines.u_color = true;
                defines.vertex_color = false;
                let mut color_ubo = engine.gl.create_uniforms();
                color_ubo.set_float_4(&U_COLOR, r.a, r.g, r.b, r.a);
                ubos.insert(UCOLOR.clone(), Arc::new(color_ubo)); // COLOR 属性
            },
            (Color::LinearGradient(r), false, true) => {
                defines.u_color = false;
                defines.vertex_color = true;
                ubos.remove(&UCOLOR);
            },
            (Color::LinearGradient(r), false, false) => {
                defines.vertex_color = true;
            },
            (Color::RadialGradient(r), _, _) => {
                panic!("error, RadialGradient is not suported");
            },
            _ => return false
        }
        return true;
    }
}

pub struct Item {
    index: usize,
    defines: Defines,
}

#[derive(Default)]
pub struct Defines {
    clip: bool,
    stroke: bool,
    u_color: bool,
    vertex_color: bool,
}

impl DefinesClip for Defines {
    fn set_clip(&mut self, value: bool){self.clip = value}
    fn get_clip(&self) -> bool {self.clip}
}

impl DefinesList for Defines {
    fn list(&self) -> Vec<Atom> {
        let mut arr = Vec::new();
        if self.clip {
            arr.push(CLIP.clone());
        }
        if self.stroke {
            arr.push(STROKE.clone());
        }
        if self.u_color {
            arr.push(UCOLOR.clone());
        }else if self.vertex_color {
            arr.push(VERTEX_COLOR.clone());
        }
        arr
    }
}

fn box_is_opacity(opacity: f32, backgroud_color: &Color, border_color: &CgColor) -> bool {
    if opacity < 1.0 {
        return false;
    }
    
    if border_color.a < 1.0 {
        return false;
    }

    return color_is_opaque(backgroud_color);
}

unsafe impl<C: Context + Share> Sync for SdfSys<C>{}
unsafe impl<C: Context + Share> Send for SdfSys<C>{}

impl_system!{
    SdfSys<C> where [C: Context + Share],
    false,
    {
        MultiCaseListener<Node, BoxColor, CreateEvent>
        MultiCaseListener<Node, BoxShadow, CreateEvent>
        MultiCaseListener<Node, BoxColor, DeleteEvent>
        MultiCaseListener<Node, BoxShadow, DeleteEvent>
        MultiCaseListener<Node, BoxColor, ModifyEvent>
        MultiCaseListener<Node, BoxShadow, ModifyEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
        MultiCaseListener<Node, ByOverflow, ModifyEvent>
        MultiCaseListener<Node, WorldMatrix, ModifyEvent>
        MultiCaseListener<Node, Visibility, ModifyEvent>
    }
}