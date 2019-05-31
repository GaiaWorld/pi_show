/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use std::sync::Arc;

use std::collections::HashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use map::{ vecmap::VecMap } ;
use hal_core::*;
use atom::Atom;
use polygon::*;

use component::user::*;
use component::calc::{Opacity, ZDepth};
use entity::{Node};
use single::{RenderObjs, RenderObjWrite, RenderObj};
use render::engine::{ Engine};
use render::res::*;
use render::res::{Opacity as ROpacity};
use system::util::*;
use system::util::constant::*;
use system::render::shaders::image::{IMAGE_FS_SHADER_NAME, IMAGE_VS_SHADER_NAME};

pub struct ImageSys<C: Context + Share>{
    image_render_map: VecMap<Item>,
    geometry_dirtys: Vec<usize>,
    mark: PhantomData<C>,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
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
            let image_clip = image_clips.get(*id);
            let object_fit = object_fits.get(*id);
            let (positions, uvs, indices) = get_geo_flow(border_radius, layout, z_depth - 0.1, image, image_clip, object_fit);

            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
            let geometry = unsafe {&mut *(render_obj.geometry.as_ref() as *const C::ContextGeometry as usize as *mut C::ContextGeometry)};

            let vertex_count: u32 = (positions.len()/3) as u32;
            if vertex_count != geometry.get_vertex_count() {
                geometry.set_vertex_count(vertex_count);
            }
            geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false).unwrap();
            geometry.set_attribute(&AttributeName::UV0, 2, Some(uvs.as_slice()), false).unwrap();
            geometry.set_indices_short(indices.as_slice(), false).unwrap();
        }
        self.geometry_dirtys.clear();
    }

    fn setup(&mut self, _: Self::ReadData, write: Self::WriteData){
        let (_, engine) = write;
        let s = SamplerDesc::default();
        let hash = sampler_desc_hash(&s);
        match engine.res_mgr.samplers.get(&hash) {
            Some(r) => self.default_sampler = Some(r.clone()),
            None => {
                let res = SamplerRes::new(hash, engine.gl.create_sampler(Arc::new(s)).unwrap());
                self.default_sampler = Some(engine.res_mgr.samplers.create(res));
            }
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
        let _border_radius = unsafe { border_radius.get_unchecked(event.id) };
        let z_depth = unsafe { z_depths.get_unchecked(event.id) }.0;
        let _layout = unsafe { layouts.get_unchecked(event.id) };
        let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;

        let geometry = create_geometry(&mut engine.gl);
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
        if let Some(item) = self.image_render_map.get_mut(event.id) {
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
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, write: Self::WriteData){
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
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        if let Some(item) = self.image_render_map.get_mut(event.id) {
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
    fn change_is_opacity(&mut self, _id: usize, opacity: f32, image: &Image<C>, index: usize, render_objs: &mut SingleCaseImpl<RenderObjs<C>>){
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

//取几何体的顶点流、 uv流和属性流, 如果layout宽高是0， 有bug
fn get_geo_flow<C: Context + Share>(radius: &BorderRadius, layout: &Layout, z_depth: f32, image: &Image<C>, image_clip: Option<&ImageClip>, object_fit: Option<&ObjectFit>) -> (Vec<f32>, Vec<f32>, Vec<u16>) {
    let radius = cal_border_radius(radius, layout);
    let (pos, uv) = get_pos_uv(image, image_clip, object_fit, layout);
    if radius.x == 0.0 {
        use_image_pos_uv(pos, uv, z_depth)
    }else{
        if pos.min.x < radius.x && pos.min.y < radius.x {
            use_layout_pos(uv, layout, &radius, z_depth)
        }else {
            use_image_pos_uv(pos, uv, z_depth)
        }
    }

    // debug_println!("indices: {:?}", indices);
    // let (top_percent, bottom_percent, left_percent, right_percent) = (pos.0.y/layout.height, pos.1.y/layout.height, pos.0.x/layout.width, pos.3.x/layout.width);
    // debug_println!("split_by_lg,  positions:{:?}, indices:{:?}, top_percent: {}, bottom_percent: {}, start: ({}, {}) , end: ({}, {})", positions, indices, 0.0, 1.0, pos.0.x, pos.0.y, pos.1.x, pos.1.y);
    // let (positions, indices_arr) = split_by_lg(positions, indices, &[top_percent, bottom_percent], (pos.0.x, pos.0.y), (pos.1.x, pos.1.y));
    // debug_println!("split_mult_by_lg, positions: {:?}, indices_arr: {:?}, cfg: {:?}, percent: [{}, {}], start: [{}, {}], end: [{}, {}]",  &positions, indices_arr, vec![LgCfg{unit: 1, data: vec![uv.min.x, uv.max.x]}], 0.0, 1.0, pos.0.x, pos.0.y, pos.3.x, pos.3.y);
    // let (positions, indices_arr) = split_mult_by_lg(positions, indices_arr, &[0.0, 1.0, right_percent], (pos.0.x, pos.0.y), (pos.3.x, pos.3.y));
    // let indices = mult_to_triangle(&indices_arr, Vec::new());
    // debug_println!("u positions: {:?}, indices_arr: {:?}, cfg: {:?}, percent: [{}, {}], start: [{}, {}], end: [{}, {}]",  &positions, indices_arr, vec![LgCfg{unit: 1, data: vec![uv.min.x, uv.max.x]}], 0.0, 1.0, pos.0.x, pos.0.y, pos.3.x, pos.3.y);
    // let u = interp_mult_by_lg(&positions, &indices_arr, vec![Vec::new()], vec![LgCfg{unit: 1, data: vec![uv.min.x, uv.max.x]}], &[0.0, 1.0], (pos.0.x, pos.0.y), (pos.3.x, pos.3.y));
    // let v = interp_mult_by_lg(&positions, &indices_arr, vec![Vec::new()], vec![LgCfg{unit: 1, data: vec![uv.min.y, uv.max.y]}], &[0.0, 1.0], (pos.0.x, pos.0.y), (pos.1.x, pos.1.y));
    // debug_println!("v positions: {:?}, indices_arr: {:?}, cfg: {:?}, percent: [{}, {}], start: [{}, {}], end: [{}, {}]",  &positions, indices_arr, vec![LgCfg{unit: 1, data: vec![uv.min.y, uv.max.y]}], 0.0, 1.0, pos.0.x, pos.0.y, pos.1.x, pos.1.y);
    // println!("v u {:?}, {:?}", v, u);
    // let mut uvs = Vec::with_capacity(u[0].len());
    // for i in 0..u[0].len() {
    //     uvs.push(u[0][i]);
    //     uvs.push(v[0][i]);
    // }

    
}

fn use_image_pos_uv(pos: Aabb2, uv: Aabb2, z_depth: f32) -> (Vec<f32>, Vec<f32>, Vec<u16>){
    let mut uvs = Vec::new();
    let mut poss = Vec::new();

    uvs.push(uv.min.x);
    uvs.push(uv.min.y);
    uvs.push(uv.min.x);
    uvs.push(uv.max.y);
    uvs.push(uv.max.x);
    uvs.push(uv.max.y);
    uvs.push(uv.max.x);
    uvs.push(uv.min.y);

    poss.push(pos.min.x);
    poss.push(pos.min.y);
    poss.push(z_depth);
    poss.push(pos.min.x);
    poss.push(pos.max.y);
    poss.push(z_depth);
    poss.push(pos.max.x);
    poss.push(pos.max.y);
    poss.push(z_depth);
    poss.push(pos.max.x);
    poss.push(pos.min.y);
    poss.push(z_depth);
    (poss, uvs, vec![0, 1, 2, 0, 2, 3])
}

fn use_layout_pos(uv: Aabb2, layout: &Layout, radius: &Point2, z_depth: f32) -> (Vec<f32>, Vec<f32>, Vec<u16>){
    let start_x = layout.border_left;
    let start_y = layout.border_top;
    let end_x = layout.width - layout.border_right;
    let end_y = layout.height - layout.border_bottom;
    debug_println!("layout-----------------------------------{:?}", layout);
    let (positions, indices) = if radius.x == 0.0 || layout.width == 0.0 || layout.height == 0.0 {
        (
            vec![
                start_x, start_y, z_depth,
                start_x, end_y, z_depth,
                end_x, end_y, z_depth,
                end_x, start_y, z_depth,
            ],
            vec![0, 1, 2, 3],
        )
    } else {
        split_by_radius(start_x, start_y, end_x - start_x, end_y - start_y, radius.x - start_x, z_depth, None)
    };
    // debug_println!("indices: {:?}", indices);
    // debug_println!("split_by_lg,  positions:{:?}, indices:{:?}, top_percent: {}, bottom_percent: {}, start: ({}, {}) , end: ({}, {})", positions, indices, 0.0, 1.0, 0.0, 0.0, 0.0, layout.height);
    let (positions, indices_arr) = split_by_lg(positions, indices, &[0.0, 1.0], (0.0, 0.0), (0.0, layout.height));
    // debug_println!("split_mult_by_lg, positions: {:?}, indices_arr: {:?}, cfg: {:?}, percent: [{}, {}], start: [{}, {}], end: [{}, {}]",  &positions, indices_arr, vec![LgCfg{unit: 1, data: vec![uv.min.x, uv.max.x]}], 0.0, 1.0, 0.0, 0.0, layout.width, 0.0);
    let (positions, indices_arr) = split_mult_by_lg(positions, indices_arr, &[0.0, 1.0], (0.0, 0.0), (layout.width, 0.0));
    let indices = mult_to_triangle(&indices_arr, Vec::new());
    // debug_println!("u positions: {:?}, indices_arr: {:?}, cfg: {:?}, percent: [{}, {}], start: [{}, {}], end: [{}, {}]",  &positions, indices_arr, vec![LgCfg{unit: 1, data: vec![uv.min.x, uv.max.x]}], 0.0, 1.0, 0.0, 0.0, layout.width, 0.0);
    let u = interp_mult_by_lg(&positions, &indices_arr, vec![Vec::new()], vec![LgCfg{unit: 1, data: vec![uv.min.x, uv.max.x]}], &[0.0, 1.0], (0.0, 0.0), (layout.width, 0.0));
    let v = interp_mult_by_lg(&positions, &indices_arr, vec![Vec::new()], vec![LgCfg{unit: 1, data: vec![uv.min.y, uv.max.y]}], &[0.0, 1.0], (0.0, 0.0), (0.0, layout.height));
    // debug_println!("v positions: {:?}, indices_arr: {:?}, cfg: {:?}, percent: [{}, {}], start: [{}, {}], end: [{}, {}]",  &positions, indices_arr, vec![LgCfg{unit: 1, data: vec![uv.min.y, uv.max.y]}], 0.0, 1.0, 0.0, 0.0, 0.0, layout.height);
    // println!("v u {:?}, {:?}", v, u);
    let mut uvs = Vec::with_capacity(u[0].len());
    for i in 0..u[0].len() {
        uvs.push(u[0][i]);
        uvs.push(v[0][i]);
    }
    (positions, uvs, indices)
}

// 获得图片的4个点(逆时针)的坐标和uv的Aabb
fn get_pos_uv<'a, C: Context + 'static + Send + Sync> (img: &Image<C>, clip: Option<&ImageClip>, fit: Option<&ObjectFit>, layout: &Layout) -> (Aabb2, Aabb2){
    let (size, mut uv1, mut uv2) = match clip {
        Some(c) => {
            let size = Vector2::new(img.src.width as f32 * (c.max.x - c.min.x).abs(), img.src.height as f32 * (c.max.y - c.min.y).abs());
            (size, c.min, c.max)
        },
        _ => (Vector2::new(img.src.width as f32, img.src.height as f32), Point2::new(0.0,0.0), Point2::new(1.0,1.0))
    };
    let mut p1 = Point2::new(layout.left + layout.border_left + layout.padding_left, layout.top + layout.border_top + layout.padding_top);
    let mut p2 = Point2::new(layout.left + layout.width - layout.border_right - layout.padding_right, layout.top + layout.height - layout.border_bottom - layout.padding_bottom);
    let w = p2.x - p1.x;
    let h = p2.y - p1.y;
    // 如果不是填充，总是居中显示。 如果在范围内，则修改点坐标。如果超出的部分，会进行剪切，剪切会修改uv坐标。
    match fit {
      Some(f) => match f.0 {
        FitType::None => {
          // 保持原有尺寸比例。同时保持内容原始尺寸大小。 超出部分会被剪切
          if size.x <= w {
            let x = (w - size.x) / 2.0;
            p1.x += x;
            p2.x -= x;
          }else{
            let x = (size.x - w) * (uv2.x - uv1.x) * 0.5 / size.x;
            uv1.x += x; 
            uv2.x -= x; 
          }
          if size.y <= h {
            let y = (h - size.y) / 2.0;
            p1.y += y;
            p2.y -= y;
          }else{
            let y = (size.y - h) * (uv2.y - uv1.y) * 0.5 / size.y;
            uv1.y += y;
            uv2.y -= y;
          }
        },
        FitType::Contain => {
          // 保持原有尺寸比例。保证内容尺寸一定可以在容器里面放得下。因此，此参数可能会在容器内留下空白。
          fill(&size, &mut p1, &mut p2, w, h);
        },
        FitType::Cover => {
          // 保持原有尺寸比例。保证内容尺寸一定大于容器尺寸，宽度和高度至少有一个和容器一致。超出部分会被剪切
          let rw = size.x/w;
          let rh = size.y/h;
          if rw > rh {
            let x = (size.x - w*rh) * (uv2.x - uv1.x) * 0.5 / size.x;
            uv1.x += x; 
            uv2.x -= x; 
          }else{
            let y = (size.y - h*rw) * (uv2.y - uv1.y) * 0.5 / size.y;
            uv1.y += y;
            uv2.y -= y;
          }
        },
        FitType::ScaleDown => {
          // 如果内容尺寸小于容器尺寸，则直接显示None。否则就是Contain
          if size.x <= w && size.y <= h {
            let x = (w - size.x) / 2.0;
            let y = (h - size.y) / 2.0;
            p1.x += x;
            p1.y += y;
            p2.x -= x;
            p2.y -= y;
          }else{
            fill(&size, &mut p1, &mut p2, w, h);
          }
        },
        _ => () // 填充。 内容拉伸填满整个容器，不保证保持原有的比例
      },
      // 默认情况是填充
      _ => ()
    };
    (Aabb2{min:p1, max:p2}, Aabb2{min:uv1, max:uv2})
}
// 按比例缩放到容器大小，居中显示
fn fill(size: &Vector2, p1: &mut Point2, p2: &mut Point2, w: f32, h: f32){ 
    let rw = size.x/w;
    let rh = size.y/h;
    if rw > rh {
      let y = (h - size.y/rw)/2.0;
      p1.y += y;
      p2.y -= y;
    }else{
      let x = (w - size.x/rh)/2.0;
      p1.x += x;
      p2.x -= x;
    }
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