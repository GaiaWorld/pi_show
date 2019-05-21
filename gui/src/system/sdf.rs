/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use std::sync::Arc;

use fnv::FnvHashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share};
use ecs::idtree::{ IdTree};
use map::{ vecmap::VecMap, Map } ;
use hal_core::{Context, Uniforms, RasterState, BlendState, StencilState, DepthState, BlendFunc, CullMode, ShaderType, Pipeline, Geometry};
use atom::Atom;

use component::user::{BoxColor, Transform, BorderRadius, BoxShadow};
use component::calc::{Visibility, WorldMatrix, Opacity, ByOverflow, ZDepth};
use component::{Color, CgColor, LengthUnit};
use layout::Layout;
use entity::{Node};
use single::{RenderObjs, RenderObjWrite, RenderObj, ViewMatrix, ProjectionMatrix, ClipUbo, ViewUbo, ProjectionUbo};
use render::engine::Engine;
use system::util::{cal_matrix, color_is_opaque, create_geometry, by_overflow_change, set_world_matrix_ubo, DefinesClip, DefinesList};
use system::util::constant::{PROJECT_MATRIX, WORLD_MATRIX, VIEW_MATRIX, POSITION, COLOR, CLIP_INDEICES, ALPHA, CLIP, VIEW, PROJECT, WORLD, COMMON};


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
    // static ref CLIP_INDEICES_SIZE: Atom = Atom::from("clipTextureSize");
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

pub struct SdfSys<C: Context + Share>{
    box_color_render_map: VecMap<Item>,
    box_shadow_render_map: VecMap<Item>,
    mark: PhantomData<C>,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
    pipelines: FnvHashMap<u64, Arc<Pipeline>>,
}

impl<C: Context + Share> SdfSys<C> {
    fn new() -> Self{
        SdfSys {
            box_color_render_map: VecMap::default(),
            box_shadow_render_map: VecMap::default(),
            mark: PhantomData,
            rs: Arc::new(RasterState::new()),
            bs: Arc::new(BlendState::new()),
            ss: Arc::new(StencilState::new()),
            ds: Arc::new(DepthState::new()),
            pipelines: FnvHashMap::default(),
        }
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BoxColor, CreateEvent> for SdfSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<ViewUbo>,
        &'a SingleCaseImpl<ProjectionUbo>,
        &'a SingleCaseImpl<ClipUbo>,
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
        let mut geometry;
        let box_color = unsafe { r.3.get_unchecked(event.id) };
        let layout = unsafe { r.9.get_unchecked(event.id) };
        let z_depth = unsafe { r.4.get_unchecked(event.id) }.0 - 0.1;
        let mut ubos: FnvHashMap<Atom, Arc<Uniforms>> = FnvHashMap::default();
        
        match &box_color.background {
            Color::RGB(r) | Color::RGBA(r) => {
                defines.u_color = true;
                let mut color_ubo = Uniforms::new();
                color_ubo.set_float_4(&U_COLOR, r.a, r.g, r.b, r.a);
                ubos.insert(UCOLOR.clone(), Arc::new(color_ubo)); // COLOR 属性
                geometry = create_geometry(&mut w.1.gl, 4);
                //如果layout > 0.0, 表示该节点曾经布局过, 设置position
                if layout.width > 0.0 {
                    let buffer = [
                        0.0,          0.0,           z_depth, // left_top
                        0.0,          layout.height, z_depth, // left_bootom
                        layout.width, layout.height, z_depth, // right_bootom
                        layout.width, 0.0,           z_depth, // right_top
                    ];
                    Arc::get_mut(&mut geometry).unwrap().set_attribute(&POSITION.clone(), 3, &buffer[0..12], false);
                }
            },
            Color::LinearGradient(r) => {
                geometry = create_geometry(&mut w.1.gl, 4);
                // defines.vertex_color = true;
                // TODO
            }
            Color::RadialGradient(r) => {
                geometry = create_geometry(&mut w.1.gl, 4);
                // defines.vertex_color = true;
                // TODO
            }
        }

        if layout.border > 0.0 {
            defines.stroke = true;
            let border_color = &box_color.border;
            let mut stroke_ubo = Uniforms::new();
            stroke_ubo.set_float_1(&STROKE_SIZE, layout.border);
            stroke_ubo.set_float_4(&STROKE_COLOR, border_color.r, border_color.g, border_color.b, border_color.a); // 描边属性
            ubos.insert(STROKE.clone(), Arc::new(stroke_ubo)); // COMMON
        }

        let index = self.create_sdf_renderobjs(event.id, 1.0, z_depth, false, ubos, &mut defines, geometry, r.0, r.1, r.2, r.5, r.6, r.7, r.8, r.9, r.10, r.11, w.0, w.1);
        self.box_shadow_render_map.insert(event.id, Item{index: index, defines: defines});
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BoxShadow, CreateEvent> for SdfSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<ViewUbo>,
        &'a SingleCaseImpl<ProjectionUbo>,
        &'a SingleCaseImpl<ClipUbo>,
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
        let mut geometry = create_geometry(&mut w.1.gl, 4);
        let box_shadow = unsafe { r.3.get_unchecked(event.id) };
        let layout = unsafe { r.9.get_unchecked(event.id) };
        let z_depth = unsafe { r.4.get_unchecked(event.id) }.0 - 0.2;
        let mut ubos: FnvHashMap<Atom, Arc<Uniforms>> = FnvHashMap::default();
        
        let shadow_color = &box_shadow.color;
        let mut color_ubo = Uniforms::new();
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
            Arc::get_mut(&mut geometry).unwrap().set_attribute(&POSITION.clone(), 3, &buffer[0..12], false);
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
    type ReadData = (&'a MultiCaseImpl<Node, BoxColor>,  &'a MultiCaseImpl<Node, Layout>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let box_color = unsafe { read.0.get_unchecked(event.id) };
        match event.field {
            "background" => {

            },
            "border" => {
                let layout = unsafe { read.1.get_unchecked(event.id) };
                if layout.border <= 0.0 {
                    return;
                }
                let item = unsafe { self.box_color_render_map.get_unchecked(event.id) };
                let mut ubos = &mut unsafe { write.get_unchecked_mut(item.index) }.ubos;
                ubos.entry(STROKE.clone()).and_modify(|stroke_ubo|{
                    let border_color = &box_color.border;
                    Arc::make_mut(stroke_ubo).set_float_4(&STROKE_COLOR, border_color.r, border_color.g, border_color.b, border_color.a);
                }).or_insert_with(|| {
                    let border_color = &box_color.border;
                    let mut stroke_ubo = Uniforms::new();
                    stroke_ubo.set_float_1(&STROKE_SIZE, layout.border);
                    stroke_ubo.set_float_4(&STROKE_COLOR, border_color.r, border_color.g, border_color.b, border_color.a);
                    Arc::new(stroke_ubo)
                });
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
    type ReadData = &'a MultiCaseImpl<Node, Layout>;
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (render_objs, engine) = write;
        let layout = unsafe { read.get_unchecked(event.id) };
        match unsafe { self.box_color_render_map.get_mut(event.id) } {
            Some(item) => {
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
        mut ubos: FnvHashMap<Atom, Arc<Uniforms>>,
        defines: &mut Defines,
        mut geometry: Arc<<C as Context>::ContextGeometry>,
        view_ubo: & SingleCaseImpl<ViewUbo>,
        projection_ubo: & SingleCaseImpl<ProjectionUbo>,
        clip_ubo: & SingleCaseImpl<ClipUbo>,
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
        let mut defines = Defines::default();

        let mut ubos: FnvHashMap<Atom, Arc<Uniforms>> = FnvHashMap::default();
        ubos.insert(VIEW_MATRIX.clone(), view_ubo.0.clone());//  视图矩阵
        ubos.insert(PROJECT_MATRIX.clone(), projection_ubo.0.clone()); // 投影矩阵

        let world_matrix = cal_matrix(id, world_matrix, transform, layout);
        let world_matrix: &[f32; 16] = world_matrix.as_ref();
        let mut world_matrix_ubo = Uniforms::new();
        world_matrix_ubo.set_mat_4v(&WORLD_MATRIX, &world_matrix[0..16]);
        ubos.insert(WORLD_MATRIX.clone(), Arc::new(world_matrix_ubo)); //世界矩阵

        let mut common_ubo = Uniforms::new();
        common_ubo.set_float_1(&BLUR, blur);
        common_ubo.set_float_1(&ALPHA, opacity);
        let layout = unsafe { layout.get_unchecked(id) };  
        let border_radius = match unsafe { border_radius.get_unchecked(id) }.0 {
            LengthUnit::Pixel(r) => r,
            LengthUnit::Percent(r) => {
                r * layout.width
            }
        };
        common_ubo.set_float_1(&RADIUS, border_radius);
        ubos.insert(COMMON.clone(), Arc::new(common_ubo)); // COMMON

        let by_overflow =  unsafe { by_overflow.get_unchecked(id) }.0;
        if by_overflow > 0 {
            defines.clip = true;
            let mut by_overflow_ubo = Uniforms::new();
            by_overflow_ubo.set_float_1(&CLIP_INDEICES, by_overflow as f32); //裁剪属性，
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

// impl_system!{
//     SdfSys,
//     false,
//     {
//         EntityListener<Node, CreateEvent>
//         MultiCaseListener<Node, BoxColor, ModifyEvent>
//         SingleCaseListener<IdTree, CreateEvent>
//     }
// }


// #[cfg(test)]
// use ecs::{World, BorrowMut, SeqDispatcher, Dispatcher};
// #[cfg(test)]
// use atom::Atom;
// #[cfg(test)]
// use component::user::BoxColorWrite;

// #[test]
// fn test(){
//     let world = new_world();

//     let idtree = world.fetch_single::<IdTree>().unwrap();
//     let idtree = BorrowMut::borrow_mut(&idtree);
//     let notify = idtree.get_notify();
//     let opacitys = world.fetch_multi::<Node, BoxColor>().unwrap();
//     let opacitys = BorrowMut::borrow_mut(&opacitys);
//     let copacitys = world.fetch_multi::<Node, CBoxColor>().unwrap();
//     let copacitys = BorrowMut::borrow_mut(&copacitys);

//     let e0 = world.create_entity::<Node>();
    
//     idtree.create(e0);
//     idtree.insert_child(e0, 0, 0, Some(&notify)); //根
//     opacitys.insert(e0, BoxColor::default());

//     world.run(&Atom::from("test_opacity_sys"));
    
//     let e00 = world.create_entity::<Node>();
//     let e01 = world.create_entity::<Node>();
//     let e02 = world.create_entity::<Node>();
//     idtree.create(e00);
//     idtree.insert_child(e00, e0, 1, Some(&notify));
//     opacitys.insert(e00, BoxColor::default());
//     idtree.create(e01);
//     idtree.insert_child(e01, e0, 2, Some(&notify));
//     opacitys.insert(e01, BoxColor::default());
//     idtree.create(e02);
//     idtree.insert_child(e02, e0, 3, Some(&notify));
//     opacitys.insert(e02, BoxColor::default());

//     let e000 = world.create_entity::<Node>();
//     let e001 = world.create_entity::<Node>();
//     let e002 = world.create_entity::<Node>();
//     idtree.create(e000);
//     idtree.insert_child(e000, e00, 1, Some(&notify));
//     opacitys.insert(e000, BoxColor::default());
//     idtree.create(e001);
//     idtree.insert_child(e001, e00, 2, Some(&notify));
//     opacitys.insert(e001, BoxColor::default());
//     idtree.create(e002);
//     idtree.insert_child(e002, e00, 3, Some(&notify));
//     opacitys.insert(e002, BoxColor::default());

//     let e010 = world.create_entity::<Node>();
//     let e011 = world.create_entity::<Node>();
//     let e012 = world.create_entity::<Node>();
//     idtree.create(e010);
//     idtree.insert_child(e010, e01, 1, Some(&notify));
//     opacitys.insert(e010, BoxColor::default());
//     idtree.create(e011);
//     idtree.insert_child(e011, e01, 2, Some(&notify));
//     opacitys.insert(e011, BoxColor::default());
//     idtree.create(e012);
//     idtree.insert_child(e012, e01, 3, Some(&notify));
//     opacitys.insert(e012, BoxColor::default());
//     world.run(&Atom::from("test_opacity_sys"));

//     unsafe { opacitys.get_unchecked_write(e0)}.set_0(0.5);
//     unsafe { opacitys.get_unchecked_write(e00)}.set_0(0.5);

//     world.run(&Atom::from("test_opacity_sys"));

//     println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
//         unsafe{copacitys.get_unchecked(e0)},
//         unsafe{copacitys.get_unchecked(e00)},
//         unsafe{copacitys.get_unchecked(e01)},
//         unsafe{copacitys.get_unchecked(e02)},
//         unsafe{copacitys.get_unchecked(e000)},
//         unsafe{copacitys.get_unchecked(e001)},
//         unsafe{copacitys.get_unchecked(e002)},
//         unsafe{copacitys.get_unchecked(e010)},
//         unsafe{copacitys.get_unchecked(e011)},
//         unsafe{copacitys.get_unchecked(e012)},
//     );
// }

// #[cfg(test)]
// fn new_world() -> World {
//     let mut world = World::default();

//     world.register_entity::<Node>();
//     world.register_multi::<Node, BoxColor>();
//     world.register_multi::<Node, CBoxColor>();
//     world.register_single::<IdTree>(IdTree::default());
     
//     let system = CellSdfSys::new(SdfSys::default());
//     world.register_system(Atom::from("system"), system);

//     let mut dispatch = SeqDispatcher::default();
//     dispatch.build("system".to_string(), &world);

//     world.add_dispatcher( Atom::from("test_opacity_sys"), dispatch);
//     world
// }

