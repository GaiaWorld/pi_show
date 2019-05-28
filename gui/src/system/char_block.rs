/**
 *  字符物体（背景图片， 图片节点）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use std::sync::Arc;

use std::collections::HashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use ecs::idtree::{ IdTree};
use map::{ vecmap::VecMap, Map } ;
use hal_core::{Context, Uniforms, RasterState, BlendState, StencilState, DepthState, BlendFunc, CullMode, ShaderType, Pipeline, Geometry};
use atom::Atom;

use component::user::*;
use component::calc::{Visibility, WorldMatrix, Opacity, ByOverflow, ZDepth, CharBlock};
use entity::{Node};
use single::{RenderObjs, RenderObjWrite, RenderObj, ViewMatrix, ProjectionMatrix, ClipUbo, ViewUbo, ProjectionUbo};
use render::engine::Engine;
use system::util::{cal_matrix, color_is_opaque, create_geometry, set_world_matrix_ubo, set_atrribute, by_overflow_change, DefinesList, DefinesClip};
use system::util::constant::{PROJECT_MATRIX, WORLD_MATRIX, VIEW_MATRIX, POSITION, COLOR, UV, CLIP_indices, COMMON, ALPHA, CLIP};


lazy_static! {
    static ref IMAGE_SHADER_NAME: Atom = Atom::from("char_block");
    static ref IMAGE_FS_SHADER_NAME: Atom = Atom::from("char_block_fs");
    static ref IMAGE_VS_SHADER_NAME: Atom = Atom::from("char_block_vs");

    static ref STROKE: Atom = Atom::from("ATROKE");
    static ref UCOLOR: Atom = Atom::from("UCOLOR");
    static ref VERTEX_COLOR: Atom = Atom::from("VERTEX_COLOR");

    static ref STROKE_CLAMP: Atom = Atom::from("strokeClamp");
    static ref STROKE_COLOR: Atom = Atom::from("strokeColor");
    static ref FONT_CLAMP: Atom = Atom::from("fontClamp");  // 0-1的小数，超过这个值即认为有字体，默认传0.75
    static ref SMOOT_HRANFE: Atom = Atom::from("smoothRange");
    static ref TEXTURE: Atom = Atom::from("texture");
}

pub struct CharBlockSys<C: Context + Share>{
    char_block_render_map: VecMap<Item>,
    geometry_dirtys: Vec<usize>,
    mark: PhantomData<C>,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
    pipelines: HashMap<u64, Arc<Pipeline>>,
}

impl<C: Context + Share> CharBlockSys<C> {
    fn new() -> Self{
        CharBlockSys {
            char_block_render_map: VecMap::default(),
            geometry_dirtys: Vec::new(),
            mark: PhantomData,
            rs: Arc::new(RasterState::new()),
            bs: Arc::new(BlendState::new()),
            ss: Arc::new(StencilState::new()),
            ds: Arc::new(DepthState::new()),
            pipelines: HashMap::default(),
        }
    }
}

impl<'a, C: Context + Share> Runner<'a> for CharBlockSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, CharBlock>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Layout>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let (char_blocks, z_depths, layouts) = read;
        for id in self.geometry_dirtys.iter() {
            let char_block = unsafe { char_blocks.get_unchecked(*id) };
            let z_depth = unsafe { z_depths.get_unchecked(*id) };
            let layout = unsafe { layouts.get_unchecked(*id) };
            let item = unsafe { self.char_block_render_map.get_unchecked_mut(*id) };
            //劈分顶点， TODO
        }
        self.geometry_dirtys.clear();
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock, CreateEvent> for CharBlockSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<ViewUbo<C>>,
        &'a SingleCaseImpl<ProjectionUbo<C>>,
        &'a SingleCaseImpl<ClipUbo<C>>,
        &'a MultiCaseImpl<Node, CharBlock>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, ByOverflow>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs<C>>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &CreateEvent, r: Self::ReadData, w: Self::WriteData){
        let mut defines = Defines::default();
        let mut geometry = create_geometry(&mut w.1.gl);
        let layout = unsafe { r.9.get_unchecked(event.id) };
        let z_depth = unsafe { r.4.get_unchecked(event.id) }.0;
        let mut ubos: HashMap<Atom, Arc<Uniforms<C>>> = HashMap::default();

        let index = self.create_charblock_renderobjs(event.id, z_depth, false, ubos, &mut defines, geometry, r.0, r.1, r.2, r.5, r.6, r.7, r.8, r.9, r.10, w.0, w.1);
        self.char_block_render_map.insert(event.id, Item{index: index, defines: defines, geometry_dirty: true});
    }
}

// 删除渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock, DeleteEvent> for CharBlockSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData){
        self.delete_geometry_dirty(event.id);
        let item = self.char_block_render_map.remove(event.id).unwrap();
        let notify = write.get_notify();
        write.remove(item.index, Some(notify));
    }
}

// Image变化, 修改ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock, ModifyEvent> for CharBlockSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        if event.field == "chars"{
            self.mark_geometry_dirty(event.id);
        }
    }
}

//世界矩阵变化， 设置ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for CharBlockSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, WorldMatrix>, &'a MultiCaseImpl<Node, Transform>, &'a MultiCaseImpl<Node, Layout>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        if let Some(item) = unsafe { self.char_block_render_map.get(event.id) } {   
            let world_matrix = cal_matrix(event.id, read.0, read.1, read.2);
            set_world_matrix_ubo(event.id, item.index, &world_matrix, write);
        }
    }
}

//不透明度变化， 设置ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for CharBlockSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, TextStyle>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        match unsafe { self.char_block_render_map.get(event.id) } {
            Some(item) => {
                let opacity = unsafe { read.0.get_unchecked(event.id).0 };
                let text_style = unsafe { read.1.get_unchecked(event.id) };
                let is_opacity = char_block_is_opacity(opacity, &text_style.color);
                let notify = write.get_notify();
                unsafe { write.get_unchecked_write(item.index, &notify).set_is_opacity(is_opacity) };

                let ubos = unsafe {&mut  write.get_unchecked_mut(item.index).ubos };
                unsafe {Arc::make_mut(ubos.get_mut(&COMMON).unwrap()).set_float_1(&ALPHA, opacity)};
            },
            None => return,
        };
    }
}

//by_overfolw变化， 设置ubo， 修改宏， 并重新创建渲染管线
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for CharBlockSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, ByOverflow>;
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        if let Some(item) = unsafe { self.char_block_render_map.get_mut(event.id) } {
            let by_overflow = unsafe { read.get_unchecked(event.id) }.0;
            by_overflow_change(by_overflow, item.index, &mut item.defines, self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone(), 0, &IMAGE_FS_SHADER_NAME, &IMAGE_VS_SHADER_NAME, write.0, write.1);
        }
    }
}

// 设置visibility
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Visibility, ModifyEvent> for CharBlockSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, Visibility>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        match self.char_block_render_map.get(event.id) {
            Some(item) => {
                let notify = write.get_notify();
                unsafe {write.get_unchecked_write(item.index, &notify)}.set_visibility(unsafe {read.get_unchecked(event.id).0});
            },
            None => (),
        }
    }
}

impl<C: Context + Share> CharBlockSys<C> {

    fn mark_geometry_dirty(&mut self, id: usize){
        let item = unsafe { self.char_block_render_map.get_unchecked_mut(id) };
        if item.geometry_dirty == false {
            item.geometry_dirty = true;
            self.geometry_dirtys.push(id);
        }
    }

    fn delete_geometry_dirty(&mut self, id: usize){
        let item = unsafe { self.char_block_render_map.get_unchecked_mut(id) };
        if item.geometry_dirty == true {
            item.geometry_dirty = false;
            for i in 0..self.geometry_dirtys.len() {
                if self.geometry_dirtys[i] == id {
                    self.geometry_dirtys.swap_remove(i);
                    return;
                }
            }
        }
    }

    fn create_charblock_renderobjs(
        &mut self,
        id: usize,
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
        render_objs: & mut SingleCaseImpl<RenderObjs<C>>,
        engine: & mut SingleCaseImpl<Engine<C>>,
    ) -> usize {
        let opacity = unsafe { opacity.get_unchecked(id) }.0; 
        let mut defines = Defines::default();

        let mut ubos: HashMap<Atom, Arc<Uniforms<C>>> = HashMap::default();
        ubos.insert(VIEW_MATRIX.clone(), view_ubo.0.clone());//  视图矩阵
        ubos.insert(PROJECT_MATRIX.clone(), projection_ubo.0.clone()); // 投影矩阵

        let world_matrix = cal_matrix(id, world_matrix, transform, layout);
        let world_matrix: &[f32; 16] = world_matrix.as_ref();
        let mut world_matrix_ubo = engine.gl.create_uniforms();
        world_matrix_ubo.set_mat_4v(&WORLD_MATRIX, &world_matrix[0..16]);
        ubos.insert(WORLD_MATRIX.clone(), Arc::new(world_matrix_ubo)); //世界矩阵

        let mut common_ubo = engine.gl.create_uniforms();
        let layout = unsafe { layout.get_unchecked(id) };
        common_ubo.set_float_1(&ALPHA, opacity); 
        ubos.insert(COMMON.clone(), Arc::new(common_ubo)); // COMMON

        let by_overflow =  unsafe { by_overflow.get_unchecked(id) }.0;
        if by_overflow > 0 {
            defines.clip = true;
            let mut by_overflow_ubo = engine.gl.create_uniforms();
            by_overflow_ubo.set_float_1(&CLIP_indices, by_overflow as f32); //裁剪属性，
        }

        let pipeline = engine.create_pipeline(0, &IMAGE_VS_SHADER_NAME.clone(), &IMAGE_FS_SHADER_NAME.clone(), defines.list().as_slice(), self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone());

        let render_obj: RenderObj<C> = RenderObj {
            depth: z_depth,
            visibility: unsafe { visibility.get_unchecked(id) }.0,
            is_opacity: is_opacity,
            ubos: ubos,
            geometry: geometry,
            pipeline: pipeline,
        };

        let notify = render_objs.get_notify();
        let index = render_objs.insert(render_obj, Some(notify));
        index
    }
}

pub struct Item {
    index: usize,
    defines: Defines,
    geometry_dirty: bool,
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

fn char_block_is_opacity(opacity: f32, color: &Color) -> bool {
    if opacity < 1.0 {
        return false;
    }
    
    return color_is_opaque(color);
}

// impl_system!{
//     CharBlockSys,
//     false,
//     {
//         EntityListener<Node, CreateEvent>
//         MultiCaseListener<Node, BoxColor, ModifyEvent>
//         SingleCaseListener<IdTree, CreateEvent>
//     }
// }