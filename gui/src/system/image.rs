/**
 *  image物体（背景图片， 图片节点）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use std::sync::Arc;

use std::collections::HashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use ecs::idtree::{ IdTree};
use map::{ vecmap::VecMap, Map } ;
use hal_core::*;
use atom::Atom;
use polygon::*;

use component::user::*;
use component::calc::{Visibility, WorldMatrix, Opacity, ByOverflow, ZDepth};
use entity::{Node};
use single::{RenderObjs, RenderObjWrite, RenderObj, ViewMatrix, ProjectionMatrix, ClipUbo, ViewUbo, ProjectionUbo};
use render::engine::Engine;
use render::res::{ TextureRes, SamplerRes };
use system::util::*;
use system::util::constant::{PROJECT_MATRIX, WORLD_MATRIX, VIEW_MATRIX, POSITION, COLOR, UV, CLIP_INDEICES, COMMON, ALPHA, CLIP};


lazy_static! {
    static ref IMAGE_SHADER_NAME: Atom = Atom::from("image");
    static ref IMAGE_FS_SHADER_NAME: Atom = Atom::from("image_fs");
    static ref IMAGE_VS_SHADER_NAME: Atom = Atom::from("image_vs");

    static ref UV_OFFSET_SCALE: Atom = Atom::from("uvOffsetScale");
    static ref TEXTURE: Atom = Atom::from("texture");
}

pub struct ImageSys<C: Context + Share>{
    image_render_map: VecMap<Item>,
    br_image_render_map: VecMap<Item>,
    mark: PhantomData<C>,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
    pipelines: HashMap<u64, Arc<Pipeline>>,
    default_sampler: Option<Arc<SamplerRes<C>>>,
}

impl<C: Context + Share> ImageSys<C> {
    pub fn new() -> Self{
        ImageSys {
            image_render_map: VecMap::default(),
            br_image_render_map: VecMap::default(),
            mark: PhantomData,
            rs: Arc::new(RasterState::new()),
            bs: Arc::new(BlendState::new()),
            ss: Arc::new(StencilState::new()),
            ds: Arc::new(DepthState::new()),
            pipelines: HashMap::default(),
            default_sampler: None,
        }
    }
}

impl<'a, C: Context + Share> Runner<'a> for ImageSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<Engine<C>>;
    fn run(&mut self, _: Self::ReadData, write: Self::WriteData){
    }

    fn setup(&mut self, _: Self::ReadData, write: Self::WriteData){
        let s = SamplerDesc::default();
        let hash = sampler_desc_hash(&s);
        if write.res_mgr.samplers.get(&hash).is_none() {
            let res = SamplerRes::new(hash, write.gl.create_sampler(Arc::new(s)).unwrap());
            self.default_sampler = Some(write.res_mgr.samplers.create(res));
        }
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Image<C>, CreateEvent> for ImageSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<ViewUbo<C>>,
        &'a SingleCaseImpl<ProjectionUbo<C>>,
        &'a SingleCaseImpl<ClipUbo<C>>,
        &'a MultiCaseImpl<Node, Image<C>>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ImageClip>,
        &'a MultiCaseImpl<Node, ObjectFit>, 
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs<C>>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &CreateEvent, r: Self::ReadData, w: Self::WriteData){
        let mut defines = Defines::default();
        let mut geometry = create_geometry(&mut w.1.gl);
        let image = unsafe { r.3.get_unchecked(event.id) };
        let layout = unsafe { r.9.get_unchecked(event.id) };
        let z_depth = unsafe { r.4.get_unchecked(event.id) }.0;
        let border_radius = unsafe { r.11.get_unchecked(event.id) };
        let image_clip = unsafe { r.12.get(event.id) };
        let object_fit = unsafe { r.13.get(event.id) };
        let mut ubos: HashMap<Atom, Arc<Uniforms<C>>> = HashMap::default();
        
        //如果layout > 0.0, 表示该节点曾经布局过, 设置position
        if layout.width > 0.0 {
            // 如果存在border， 可能效果不对
            set_image_geometry(event.id, image, z_depth, &mut geometry, layout, r.11, r.12, r.13);
        }

        let index = self.create_image_renderobjs(event.id, &image.src, z_depth, false, ubos, &mut defines, geometry, r.0, r.1, r.2, r.5, r.6, r.7, r.8, r.9, r.10, r.11, w.0, w.1);
        self.image_render_map.insert(event.id, Item{index: index, defines: defines});
    }
}

// 删除渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Image<C>, DeleteEvent> for ImageSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData){
        let item = self.image_render_map.remove(event.id).unwrap();
        let notify = write.get_notify();
        write.remove(item.index, Some(notify));
    }
}

// Image变化, 修改ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Image<C>, ModifyEvent> for ImageSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, Image<C>>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let image = unsafe { read.get_unchecked(event.id) };
        match event.field {
            "src" => {
                let image = unsafe { read.get_unchecked(event.id) };
                let item = unsafe { self.image_render_map.get_unchecked(event.id) };
                let mut ubos = &mut unsafe { write.get_unchecked_mut(item.index) }.ubos;
                let common_ubo = ubos.get_mut(&COMMON).unwrap();
                Arc::make_mut(common_ubo).set_sampler(&TEXTURE, &(self.default_sampler.as_ref().unwrap().clone() as Arc<AsRef<<C as Context>::ContextSampler>>), &(image.src.clone() as Arc<AsRef<<C as Context>::ContextTexture>>));
            },
            _ => (),
        }
    }
}

//
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, ImageClip, ModifyEvent> for ImageSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Image<C>>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ImageClip>,
        &'a MultiCaseImpl<Node, ObjectFit>, 
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, r: Self::ReadData, write: Self::WriteData){
        let z_depth = unsafe { r.0.get_unchecked(event.id) }.0;
        let layout = unsafe { r.2.get_unchecked(event.id) };
        let image = unsafe { r.1.get_unchecked(event.id) };
        let item = self.image_render_map.remove(event.id).unwrap();
        let render_obj = unsafe { write.get_unchecked_mut(event.id) };
        set_image_geometry(event.id, image, z_depth, &mut render_obj.geometry, layout, r.3, r.4, r.5);
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BorderImage<C>, CreateEvent> for ImageSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<ViewUbo<C>>,
        &'a SingleCaseImpl<ProjectionUbo<C>>,
        &'a SingleCaseImpl<ClipUbo<C>>,
        &'a MultiCaseImpl<Node, BorderImage<C>>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, BorderImageClip>,
        &'a MultiCaseImpl<Node, BorderImageSlice>,
        &'a MultiCaseImpl<Node, BorderImageRepeat>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs<C>>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &CreateEvent, r: Self::ReadData, w: Self::WriteData){
        let mut defines = Defines::default();
        let mut geometry = create_geometry(&mut w.1.gl);
        let bg_image = unsafe { r.3.get_unchecked(event.id) };
        let layout = unsafe { r.9.get_unchecked(event.id) };
        let z_depth = unsafe { r.4.get_unchecked(event.id) }.0 - 0.1;
        let mut ubos: HashMap<Atom, Arc<Uniforms<C>>> = HashMap::default();
        
        //如果layout > 0.0, 表示该节点曾经布局过, 设置position
        if layout.width > 0.0 {
            set_border_image_geometry::<C>(event.id, bg_image, z_depth, &mut geometry, layout, r.12, r.13, r.14);
        }

        let index = self.create_image_renderobjs(event.id, &bg_image.src, z_depth, false, ubos, &mut defines, geometry, r.0, r.1, r.2, r.5, r.6, r.7, r.8, r.9, r.10, r.11, w.0, w.1);
        self.br_image_render_map.insert(event.id, Item{index: index, defines: defines});
    }
}

// 删除渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BorderImage<C>, DeleteEvent> for ImageSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData){
        let item = self.br_image_render_map.remove(event.id).unwrap();
        let notify = write.get_notify();
        write.remove(item.index, Some(notify));
    }
}

// Image变化, 修改ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BorderImage<C>, ModifyEvent> for ImageSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, Image<C>>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let bg_image = unsafe { read.get_unchecked(event.id) };
        let item = unsafe { self.br_image_render_map.get_unchecked(event.id) };
        let mut ubos = &mut unsafe { write.get_unchecked_mut(item.index) }.ubos;
        let common_ubo = ubos.get_mut(&COMMON).unwrap();
        Arc::make_mut(common_ubo).set_sampler(&TEXTURE, &(self.default_sampler.as_ref().unwrap().clone() as Arc<AsRef<<C as Context>::ContextSampler>>), &(bg_image.src.clone() as Arc<AsRef<<C as Context>::ContextTexture>>));
    }
}

//世界矩阵变化， 设置ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for ImageSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, WorldMatrix>, &'a MultiCaseImpl<Node, Transform>, &'a MultiCaseImpl<Node, Layout>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        match (unsafe { self.image_render_map.get(event.id) }, unsafe { self.br_image_render_map.get(event.id) }) {
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

//layout变化， 设置顶点流
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Layout, ModifyEvent> for ImageSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Image<C>>,
        &'a MultiCaseImpl<Node, ImageClip>,
        &'a MultiCaseImpl<Node, ObjectFit>,
        &'a MultiCaseImpl<Node, BorderImage<C>>,
        &'a MultiCaseImpl<Node, BorderImageClip>,
        &'a MultiCaseImpl<Node, BorderImageSlice>,
        &'a MultiCaseImpl<Node, BorderImageRepeat>,
        &'a MultiCaseImpl<Node, BorderRadius>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (layouts, depths, images, image_clips, fits, br_images, br_clips, bg_slices, br_repeats, border_radius) = read;
        let layout = unsafe { layouts.get_unchecked(event.id) };
        let z_depth = unsafe { depths.get_unchecked(event.id) }.0;
        if let Some(item) = unsafe { self.image_render_map.get(event.id) } {
            let geometry = unsafe { &mut write.get_unchecked_mut(item.index).geometry };
            let image = unsafe { &mut images.get_unchecked(event.id)};
            set_image_geometry::<C>(event.id, image, z_depth, geometry, layout, border_radius, image_clips, fits);
        }
        if let Some(item) = unsafe { self.image_render_map.get(event.id) } {
            let geometry = unsafe { &mut write.get_unchecked_mut(item.index).geometry };
            let br_image = unsafe { &mut br_images.get_unchecked(event.id)};
            set_border_image_geometry::<C>(event.id, br_image, z_depth, geometry, layout, br_clips, bg_slices, br_repeats);
        }
    }
}

//不透明度变化， 设置ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for ImageSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, Opacity>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        match unsafe { self.image_render_map.get(event.id) } {
            Some(item) => {
                let opacity = unsafe { read.get_unchecked(event.id).0 };
                let is_opacity = image_is_opacity(opacity);
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
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for ImageSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, ByOverflow>;
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        if let Some(item) = unsafe { self.image_render_map.get_mut(event.id) } {
            let by_overflow = unsafe { read.get_unchecked(event.id) }.0;
            by_overflow_change(by_overflow, item.index, &mut item.defines, self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone(), 0, &IMAGE_FS_SHADER_NAME, &IMAGE_VS_SHADER_NAME, write.0, write.1);
        }
        if let Some(item) = unsafe { self.br_image_render_map.get_mut(event.id) } {
            let by_overflow = unsafe { read.get_unchecked(event.id) }.0;
            by_overflow_change(by_overflow, item.index, &mut item.defines, self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone(), 0, &IMAGE_FS_SHADER_NAME, &IMAGE_VS_SHADER_NAME, write.0, write.1);
        }
    }
}

// 设置visibility
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Visibility, ModifyEvent> for ImageSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, Visibility>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        match self.image_render_map.get(event.id) {
            Some(item) => {
                let notify = write.get_notify();
                unsafe {write.get_unchecked_write(item.index, &notify)}.set_visibility(unsafe {read.get_unchecked(event.id).0});
            },
            None => (),
        }
    }
}

impl<C: Context + Share> ImageSys<C> {
    fn create_image_renderobjs(
        &mut self,
        id: usize,
        src: &Arc<TextureRes<C>>,
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
        common_ubo.set_sampler(&TEXTURE, &(self.default_sampler.as_ref().unwrap().clone() as Arc<AsRef<<C as Context>::ContextSampler>>), &(src.clone() as Arc<AsRef<<C as Context>::ContextTexture>>));
        common_ubo.set_float_1(&ALPHA, opacity);
        ubos.insert(COMMON.clone(), Arc::new(common_ubo)); // COMMON
        
        let by_overflow =  unsafe { by_overflow.get_unchecked(id) }.0;
        if by_overflow > 0 {
            defines.clip = true;
            let mut by_overflow_ubo = engine.gl.create_uniforms();
            by_overflow_ubo.set_float_1(&CLIP_INDEICES, by_overflow as f32); //裁剪属性，
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
}

#[derive(Default)]
pub struct Defines {
    clip: bool,
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
        arr
    }
}

fn image_is_opacity(opacity: f32) -> bool {
    if opacity < 1.0 {
        return false;
    }
    
    return true;
}

fn set_image_geometry<C: Context + Share>(
    id: usize,
    image: &Image<C>,
    z_depth: f32,
    geometry: &mut Arc<<C as Context>::ContextGeometry>,
    layout: &Layout,
    border_radiuss: &MultiCaseImpl<Node, BorderRadius>,
    image_clips: &MultiCaseImpl<Node, ImageClip>,
    object_fits: &MultiCaseImpl<Node, ObjectFit>,
){
    let border_radius = unsafe { border_radiuss.get_unchecked(id) };
    let image_clip = unsafe { image_clips.get(id) };
    let object_fit = unsafe { object_fits.get(id) };
    let radius = cal_border_radius(border_radius,  layout);
    let (pos, uv) = get_pos_uv(image, image_clip, object_fit, layout);
    let positions = positions_width_radius(border_radius, layout, z_depth, (0.0, 0.0));
    
    let (top_percent, bottom_percent, left_percent, right_percent) = (pos.0.y/layout.height, pos.1.y/layout.height, pos.0.x/layout.width, pos.3.x/layout.width);
    let (positions, _) = split_by_lg(positions, &[top_percent, bottom_percent], (pos.0.x, pos.0.y), (pos.1.x, pos.1.y));
    let (positions, indices) = split_by_lg(positions, &[left_percent, right_percent], (pos.0.x, pos.0.y), (pos.3.x, pos.3.y));
    let indices = to_triangle(indices.as_slice());
    let u = interp_by_lg(&positions, vec![LgCfg{unit: 1, data: vec![uv.0.x, uv.0.x]}], &[left_percent, right_percent], (pos.0.x, pos.0.y), (pos.3.x, pos.3.y));
    let v = interp_by_lg(&positions, vec![LgCfg{unit: 1, data: vec![uv.0.y, uv.1.y]}], &[top_percent, bottom_percent], (pos.0.x, pos.0.y), (pos.1.x, pos.1.y));
    let geo = Arc::get_mut(geometry).unwrap();
    let mut uvs = Vec::with_capacity(u[0].len());
    for i in 0..u.len() {
        uvs.push(u[0][i]);
        uvs.push(v[0][i]);
    }

    let vertex_count: u32 = (positions.len()/3) as u32;
    if vertex_count != geo.get_vertex_count() {
        geo.set_vertex_count(vertex_count);
    }
    //设置索引
    geo.set_indices_short(indices.as_slice(), false);
    geo.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false);
    geo.set_attribute(&AttributeName::UV0, 2, Some(uvs.as_slice()), false);
}

fn set_border_image_geometry<C: Context + Share>(
    id: usize,
    image: &BorderImage<C>,
    z_depth: f32,
    geometry: &mut Arc<<C as Context>::ContextGeometry>,
    layout: &Layout,
    clips: &MultiCaseImpl<Node, BorderImageClip>,
    slices: &MultiCaseImpl<Node, BorderImageSlice>,
    repeats: &MultiCaseImpl<Node, BorderImageRepeat>,
){
    let geo = Arc::get_mut(geometry).unwrap();
    let clip = unsafe { clips.get(id) };
    let slice = unsafe { slices.get_unchecked(id) };
    let repeat = unsafe { repeats.get(id) };

    let (positions, uvs, indices) = get_border_image_stream(image, clip, slice, repeat, layout, z_depth, Vec::new(), Vec::new(), Vec::new());
    
    let vertex_count: u32 = (positions.len()/3) as u32;
    if vertex_count != geo.get_vertex_count() {
        geo.set_vertex_count(vertex_count);
    }

    geo.set_indices_short(indices.as_slice(), false);
    geo.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false);
    geo.set_attribute(&AttributeName::UV0, 2, Some(uvs.as_slice()), false);
}

unsafe impl<C: Context + Share> Sync for ImageSys<C>{}
unsafe impl<C: Context + Share> Send for ImageSys<C>{}

impl_system!{
    ImageSys<C> where [C: Context + Share],
    true,
    {
        MultiCaseListener<Node, Image<C>, CreateEvent>
        MultiCaseListener<Node, Image<C>, DeleteEvent>
        MultiCaseListener<Node, Image<C>, ModifyEvent>
        MultiCaseListener<Node, BorderImage<C>, CreateEvent>
        MultiCaseListener<Node, BorderImage<C>, DeleteEvent>
        MultiCaseListener<Node, BorderImage<C>, ModifyEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
        MultiCaseListener<Node, ByOverflow, ModifyEvent>
        MultiCaseListener<Node, WorldMatrix, ModifyEvent>
        MultiCaseListener<Node, Visibility, ModifyEvent>
    }
}
