/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use std::sync::Arc;
use std::mem::transmute;

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
use render::engine::{ Engine , PipelineInfo};
use render::res::*;
use render::res::{Opacity as ROpacity};
use system::util::*;
use system::util::constant::{PROJECT_MATRIX, WORLD_MATRIX, VIEW_MATRIX, POSITION, COLOR, CLIP_INDEICES, ALPHA, CLIP, VIEW, PROJECT, WORLD, COMMON, TEXTURE};
use system::render::shaders::image::{IMAGE_FS_SHADER_NAME, IMAGE_VS_SHADER_NAME};

pub struct ImageSys<C: Context + Share>{
    image_render_map: VecMap<Item>,
    geometry_dirtys: Vec<usize>,
    mark: PhantomData<C>,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
    pipelines: HashMap<u64, Arc<PipelineInfo>>,
    default_sampler: Option<Arc<SamplerRes<C>>>,
}

impl<C: Context + Share> ImageSys<C> {
    pub fn new() -> Self{
        ImageSys {
            image_render_map: VecMap::default(),
            geometry_dirtys: Vec::new(),
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

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: Context + Share> Runner<'a> for ImageSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Image<C>>,
        &'a MultiCaseImpl<Node, ImageClip>,
        &'a MultiCaseImpl<Node, ObjectFit>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let map = &mut self.image_render_map;
        let (layouts, border_radius, z_depths, images, image_clips, object_fits) = read;
        let (render_objs, _) = write;
        for id in  self.geometry_dirtys.iter() {
            let item = unsafe { map.get_unchecked_mut(*id) };
            item.position_change = false;
            let border_radius = unsafe { border_radius.get_unchecked(*id) };
            let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;
            let layout = unsafe { layouts.get_unchecked(*id) };
            let image = unsafe { images.get_unchecked(*id) };
            let image_clip = unsafe { image_clips.get(*id) };
            let object_fit = unsafe { object_fits.get(*id) };
            let (positions, uvs, indices) = get_geo_flow(border_radius, layout, z_depth - 0.1, image, image_clip, object_fit);

            let mut render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
            let geometry = Arc::get_mut(&mut render_obj.geometry).unwrap();

            let vertex_count: u32 = (positions.len()/3) as u32;
            if vertex_count != geometry.get_vertex_count() {
                geometry.set_vertex_count(vertex_count);
            }
            geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false);
            geometry.set_attribute(&AttributeName::UV0, 2, Some(uvs.as_slice()), false);
            geometry.set_indices_short(indices.as_slice(), false);
        }
        self.geometry_dirtys.clear();
    }

    fn setup(&mut self, _: Self::ReadData, write: Self::WriteData){
        let (_, engine) = write;
        let s = SamplerDesc::default();
        let hash = sampler_desc_hash(&s);
        if engine.res_mgr.samplers.get(&hash).is_none() {
            let res = SamplerRes::new(hash, engine.gl.create_sampler(Arc::new(s)).unwrap());
            self.default_sampler = Some(engine.res_mgr.samplers.create(res));
        }
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Image<C>, CreateEvent> for ImageSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Image<C>>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, Opacity>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs<C>>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (images, border_radius, z_depths, layouts, opacitys) = read;
        let (render_objs, engine) = write;
        let image = unsafe { images.get_unchecked(event.id) };
        let border_radius = unsafe { border_radius.get_unchecked(event.id) };
        let z_depth = unsafe { z_depths.get_unchecked(event.id) }.0;
        let layout = unsafe { layouts.get_unchecked(event.id) };
        let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;

        let mut geometry = create_geometry(&mut engine.gl);
        let mut ubos: HashMap<Atom, Arc<Uniforms<C>>> = HashMap::default();
        let defines = Vec::new();

        let mut common_ubo = engine.gl.create_uniforms();
        common_ubo.set_sampler(
            &TEXTURE,
            &(self.default_sampler.as_ref().unwrap().clone() as Arc<AsRef<<C as Context>::ContextSampler>>),
            &(image.src.clone() as Arc<AsRef<<C as Context>::ContextTexture>>)
        );
        ubos.insert(COMMON.clone(), Arc::new(common_ubo)); // COMMON

        let pipeline = engine.create_pipeline(0, &IMAGE_VS_SHADER_NAME.clone(), &IMAGE_FS_SHADER_NAME.clone(), defines.as_slice(), self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone());
        
        let is_opacity = if opacity < 1.0 {
            false
        }else if let ROpacity::Opaque = image.src.opacity{
            true
        }else {
            false
        };
        let render_obj: RenderObj<C> = RenderObj {
            depth: z_depth - 1.0,
            visibility: false,
            is_opacity: is_opacity,
            ubos: ubos,
            geometry: geometry,
            pipeline: pipeline.clone(),
            context: event.id,
            defines: defines,
        };

        let notify = render_objs.get_notify();
        let index = render_objs.insert(render_obj, Some(notify));
        self.image_render_map.insert(event.id, Item{index: index, position_change: true});
        self.geometry_dirtys.push(event.id);
    }
}

// 修改渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Image<C>, ModifyEvent> for ImageSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, Image<C>>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (opacitys, images) = read;
        if let Some(item) = unsafe { self.image_render_map.get_mut(event.id) } {
            let opacity = unsafe { opacitys.get_unchecked(event.id).0 };
            let image = unsafe { images.get_unchecked(event.id) };
            let index = item.index;

            // 图片改变， 修改渲染对象的不透明属性
            self.change_is_opacity(event.id, opacity, image, index, render_objs);

            // 图片改变， 更新common_ubo中的纹理
            let render_obj = unsafe { render_objs.get_unchecked_mut(event.id) };
            let common_ubo = render_obj.ubos.get_mut(&COMMON).unwrap();
            let common_ubo = Arc::make_mut(common_ubo);
            common_ubo.set_sampler(
                &TEXTURE,
                &(self.default_sampler.as_ref().unwrap().clone() as Arc<AsRef<<C as Context>::ContextSampler>>),
                &(image.src.clone() as Arc<AsRef<<C as Context>::ContextTexture>>)
            );
        }
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
        if item.position_change == true {
            self.geometry_dirtys.remove_item(&event.id);
        }
    }
}

//布局修改， 需要重新计算顶点
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Layout, ModifyEvent> for ImageSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        if let Some(item) = unsafe { self.image_render_map.get_mut(event.id) } {
            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(event.id);
            }
        };
    }
}

//不透明度变化， 修改渲染对象的is_opacity属性
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for ImageSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, Image<C>>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (opacitys, images) = read;
        if let Some(item) = self.image_render_map.get(event.id) {
            let opacity = unsafe { opacitys.get_unchecked(event.id).0 };
            let image = unsafe { images.get_unchecked(event.id) };
            let index = item.index;
            self.change_is_opacity(event.id, opacity, image, index, write);
        }
    }
}

impl<'a, C: Context + Share> ImageSys<C> {
    fn change_is_opacity(&mut self, id: usize, opacity: f32, image: &Image<C>, index: usize, render_objs: &mut SingleCaseImpl<RenderObjs<C>>){
        let is_opacity = if opacity < 1.0 {
            false
        }else if let ROpacity::Opaque = image.src.opacity{
            true
        }else {
            false
        };

        let notify = render_objs.get_notify();
        unsafe { render_objs.get_unchecked_write(index, &notify).set_is_opacity(is_opacity)};
    }
}

struct Item {
    index: usize,
    position_change: bool,
}

//取几何体的顶点流、 uv流和属性流
fn get_geo_flow<C: Context + Share>(radius: &BorderRadius, layout: &Layout, z_depth: f32, image: &Image<C>, image_clip: Option<&ImageClip>, object_fit: Option<&ObjectFit>) -> (Vec<f32>, Vec<f32>, Vec<u16>) {
    let radius = cal_border_radius(radius, layout);
    let (pos, uv) = get_pos_uv(image, image_clip, object_fit, layout);
    let start = layout.border;
    let end_x = layout.width - layout.border;
    let end_y = layout.height - layout.border;
    let positions = if radius.x == 0.0 {
        vec![
            start, start, z_depth,
            start, end_y, z_depth,
            end_x, end_y, z_depth,
            end_x, start, z_depth,
        ]
    } else {
        split_by_radius(start, start, end_x - layout.border, end_y - layout.border, radius.x - layout.border, z_depth)
    };
    
    let (top_percent, bottom_percent, left_percent, right_percent) = (pos.0.y/layout.height, pos.1.y/layout.height, pos.0.x/layout.width, pos.3.x/layout.width);
    let (positions, _) = split_by_lg(positions, &[top_percent, bottom_percent], (pos.0.x, pos.0.y), (pos.1.x, pos.1.y));
    let (positions, indices) = split_by_lg(positions, &[left_percent, right_percent], (pos.0.x, pos.0.y), (pos.3.x, pos.3.y));
    let indices = to_triangle(indices.as_slice());
    let u = interp_by_lg(&positions, vec![LgCfg{unit: 1, data: vec![uv.0.x, uv.0.x]}], &[left_percent, right_percent], (pos.0.x, pos.0.y), (pos.3.x, pos.3.y));
    let v = interp_by_lg(&positions, vec![LgCfg{unit: 1, data: vec![uv.0.y, uv.1.y]}], &[top_percent, bottom_percent], (pos.0.x, pos.0.y), (pos.1.x, pos.1.y));
    let mut uvs = Vec::with_capacity(u[0].len());
    for i in 0..u.len() {
        uvs.push(u[0][i]);
        uvs.push(v[0][i]);
    }

    (positions, uvs, indices)
}

unsafe impl<C: Context + Share> Sync for ImageSys<C>{}
unsafe impl<C: Context + Share> Send for ImageSys<C>{}

impl_system!{
    ImageSys<C> where [C: Context + Share],
    true,
    {
        MultiCaseListener<Node, Image<C>, CreateEvent>
        MultiCaseListener<Node, Image<C>, ModifyEvent>
        MultiCaseListener<Node, Image<C>, DeleteEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
    }
}