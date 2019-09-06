/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;

use share::Share;
use std::hash::{ Hasher, Hash };

// use ordered_float::NotNan;
use fxhash::FxHasher32;

use ecs::{SingleCaseImpl, MultiCaseImpl, MultiCaseListener, DeleteEvent, Runner};
use map::{ vecmap::VecMap } ;
use hal_core::*;
use atom::Atom;
use polygon::*;

use component::user::*;
use component::calc::*;
use component::calc::{Opacity};
use entity::{Node};
use single::*;
use render::engine::{ Engine};
use render::res::*;
use render::res::{Opacity as ROpacity};
use system::util::*;
use system::util::constant::*;
use system::render::shaders::image::{IMAGE_FS_SHADER_NAME, IMAGE_VS_SHADER_NAME};

lazy_static! {
    static ref UV: Atom = Atom::from("UV");
    static ref POSITION: Atom = Atom::from("Position");
    static ref INDEX: Atom = Atom::from("Index");
}

const DIRTY_TY: usize = StyleType::BorderRadius as usize |
                        StyleType::Matrix as usize |
                        StyleType::Opacity as usize |
                        StyleType::Layout as usize |
                        StyleType::Image as usize |
                        StyleType::ImageClip as usize |
                        StyleType::ObjectFit as usize;

const GEO_DIRTY: usize =StyleType::BorderRadius as usize |
                        StyleType::Layout as usize |
                        StyleType::Image as usize |
                        StyleType::ImageClip as usize |
                        StyleType::ObjectFit as usize;

pub struct ImageSys<C>{
    render_map: VecMap<usize>,
    default_sampler: Share<SamplerRes>,
    unit_geo: Share<GeometryRes>, // 含uv， index， pos  
    marker: PhantomData<C>,
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for ImageSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Image>,
        &'a MultiCaseImpl<Node, ImageClip>,
        &'a MultiCaseImpl<Node, ObjectFit>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a MultiCaseImpl<Node, Culling>,

        &'a SingleCaseImpl<DefaultTable>,
        &'a SingleCaseImpl<DirtyList>,
        &'a SingleCaseImpl<DefaultState>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<Engine<C>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let (
            layouts, 
            border_radiuss, 
            z_depths, 
            images, 
            image_clips, 
            object_fits, 
            world_matrixs, 
            transforms, 
            opacitys, 
            style_marks, 
            cullings,

            default_table, 
            dirty_list, 
            default_state
        ) = read;
        let (render_objs, engine) = write;
        let default_transform = default_table.get::<Transform>().unwrap();
        let notify = render_objs.get_notify();

        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => {
                    self.remove_render_obj(*id, render_objs);
                    continue
                },
            };

            let mut dirty = style_mark.dirty;

            // 不存在Image关心的脏, 跳过
            if dirty & DIRTY_TY == 0 {
                continue;
            }

            // Image脏， 如果不存在Image的本地样式和class样式， 删除渲染对象
            let render_index = if dirty & StyleType::Image as usize != 0 {
                if style_mark.local_style & StyleType::Image as usize == 0 && style_mark.class_style & StyleType::Image as usize == 0{
                    self.remove_render_obj(*id, render_objs);
                    continue;
                }
                
                // 不存在渲染对象， 创建
                match self.render_map.get_mut(*id) {
                    Some(r) => *r,
                    None => self.create_render_obj(*id, render_objs, default_state),
                }
            } else {
                match self.render_map.get_mut(*id) {
                    Some(r) => *r,
                    None => continue,
                }
            };

            let render_obj = unsafe {render_objs.get_unchecked_mut(render_index)};

            let border_radius = border_radiuss.get(*id);
            let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;
            let layout = unsafe { layouts.get_unchecked(*id) };
            let image = unsafe { images.get_unchecked(*id) };
            let image_clip = image_clips.get(*id);
            let object_fit = object_fits.get(*id);
            let transform =  match transforms.get(*id) {
                Some(r) => r,
                None => default_transform,
            };
            let world_matrix = unsafe { world_matrixs.get_unchecked(*id) };
            
            if dirty & GEO_DIRTY != 0 {
                let (has_radius, pos) = update_geo(render_obj, border_radius, layout, image, image_clip, object_fit, engine, &self.unit_geo);

                modify_matrix(render_obj, layout, z_depth, world_matrix, transform, &pos, has_radius);
                
                // src修改， 修改texture
                if dirty & StyleType::Image as usize != 0 {
                    // 如果四边形与图片宽高一样， 使用点采样， TODO
                    render_obj.paramter.set_texture("texture", (&image.src.bind, &self.default_sampler));
                }

                notify.modify_event(render_index, "geometry", 0);
                notify.modify_event(render_index, "ubo", 0);
                dirty &= !(StyleType::Matrix as usize); // 已经计算了世界矩阵， 设置世界矩阵不脏
            }

            // 世界矩阵脏， 设置世界矩阵ubo
            if dirty & StyleType::Matrix as usize != 0 && !unsafe{cullings.get_unchecked(*id)}.0 {
                let (pos, _uv) = get_pos_uv(image, image_clip, object_fit, layout);
                let radius = cal_border_radius(border_radius, layout);
                let mut has_radius = false;
                let g_b = geo_box(layout);

                if radius.x > g_b.min.x && pos.min.x < radius.x && pos.min.y < radius.x  {
                    has_radius = true;
                }
                modify_matrix(render_obj, layout, z_depth, world_matrix, transform, &pos, has_radius);
                notify.modify_event(render_index, "ubo", 0);
            }

            // 不透明度脏或图片脏， 设置is_opacity
            if dirty & StyleType::Opacity as usize != 0 || dirty & StyleType::Image as usize != 0 {
                let opacity = unsafe {opacitys.get_unchecked(*id)}.0;
                let is_opacity = if opacity < 1.0 {
                    false
                }else if let ROpacity::Opaque = image.src.opacity{
                    true
                }else {
                    false
                };
                render_obj.is_opacity = is_opacity;
                notify.modify_event(render_index, "is_opacity", 0);
                modify_opacity(engine, render_obj);
            }
            notify.modify_event(render_index, "", 0);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Image, DeleteEvent> for ImageSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, render_objs: Self::WriteData){
        self.remove_render_obj(event.id, render_objs)
    }
}

impl<C: HalContext + 'static> ImageSys<C> {
    pub fn new(engine: &mut Engine<C>) -> Self{
        let default_sampler = SamplerDesc::default();
        let mut hasher = FxHasher32::default();
        default_sampler.hash(&mut hasher);
        let hash = hasher.finish();
        let default_sampler = match engine.res_mgr.get::<HalSampler>(&hash) {
            Some(r) => r,
            None => engine.res_mgr.create(hash, create_sampler(&engine.gl, default_sampler)),
        };

        let positions = engine.res_mgr.get::<HalBuffer>(&POSITIONUNIT.get_hash()).unwrap();
        let indices = engine.res_mgr.get::<HalBuffer>(&INDEXUNIT.get_hash()).unwrap();

        let geo = create_geometry(&engine.gl);
        engine.gl.geometry_set_attribute(&geo, &AttributeName::Position, &positions, 2).unwrap();
        engine.gl.geometry_set_attribute(&geo, &AttributeName::UV0, &positions, 2).unwrap();
        engine.gl.geometry_set_indices_short(&geo, &indices).unwrap();

        ImageSys {
            render_map: VecMap::default(),
            default_sampler: default_sampler,
            unit_geo: Share::new(GeometryRes{geo: geo, buffers: vec![positions.clone(), indices, positions]}),  
            marker: PhantomData,
        }
    }

    #[inline]
    fn remove_render_obj(&mut self, id: usize, render_objs: &mut SingleCaseImpl<RenderObjs>) {
        match self.render_map.remove(id) {
            Some(index) => {
                let notify = render_objs.get_notify();
                render_objs.remove(index, Some(notify));
            },
            None => ()
        };
    }

    #[inline]
    fn create_render_obj(
        &mut self,
        id: usize,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &DefaultState,
    ) -> usize{    
        create_render_obj(
            id,
            -0.1,
            true,
            IMAGE_VS_SHADER_NAME.clone(),
            IMAGE_FS_SHADER_NAME.clone(),
            Share::new(ImageParamter::default()),
            default_state, render_objs,
            &mut self.render_map
        )
    }
}

#[inline]
fn update_geo<C: HalContext + 'static>(
    render_obj: &mut RenderObj,
    border_radius: Option<&BorderRadius>,
    layout: &Layout,
    image: &Image,
    image_clip: Option<&ImageClip>,
    object_fit: Option<&ObjectFit>,
    engine: &mut Engine<C>,
    unit_geo: &Share<GeometryRes>,
) -> (bool, Aabb2)  {
    let (pos, uv) = get_pos_uv(image, image_clip, object_fit, layout);
    let radius = cal_border_radius(border_radius, layout);
    let g_b = geo_box(layout);

    if radius.x > g_b.min.x && pos.min.x < radius.x && pos.min.y < radius.x  {
        use_layout_pos(render_obj, uv, layout, &radius, engine); // 有圆角
        (true, pos)
    }else{
        update_geo_quad(render_obj, &uv, image_clip, engine, unit_geo); // 没有圆角
        (false, pos)
    } 
}

fn update_geo_quad<C: HalContext + 'static>(
    render_obj: &mut RenderObj,
    uv: &Aabb2,
    image_clip: Option<&ImageClip>,
    engine: &mut Engine<C>,
    unit_geo: &Share<GeometryRes>,
) {
    match image_clip {
        Some(_clip) => {
            let uv_hash = cal_uv_hash(uv);
            let uv_buffer = create_uv_buffer(uv_hash, &uv, engine);
            let geo = create_geometry(&engine.gl);
            engine.gl.geometry_set_attribute(&geo, &AttributeName::Position, &unit_geo.buffers[0], 2).unwrap();
            engine.gl.geometry_set_attribute(&geo, &AttributeName::UV0, &uv_buffer, 2).unwrap();
            engine.gl.geometry_set_indices_short(&geo, &unit_geo.buffers[1]).unwrap();
            let geo_res = GeometryRes{geo: geo, buffers: vec![unit_geo.buffers[0].clone(), unit_geo.buffers[1].clone(), uv_buffer]};
            render_obj.geometry = Some(create_geo(&uv_hash, geo_res, engine));
        },
        None => render_obj.geometry = Some(unit_geo.clone()),
    }

    // 修改世界矩阵 TODO
}

#[inline]
fn cal_uv_hash(uv: &Aabb2) -> u64 {
    let mut hasher = FxHasher32::default();
    UV.hash(&mut hasher);
    f32_4_hash_(uv.min.x, uv.min.y, uv.max.x, uv.max.y, &mut hasher);
    hasher.finish()
}

fn create_uv_buffer<C: HalContext + 'static>(uv_hash: u64, uv: &Aabb2, engine: &mut Engine<C>) -> Share<HalBuffer> {
    match engine.res_mgr.get::<HalBuffer>(&uv_hash) {
        Some(r) => r,
        None => {
            let uvs = [uv.min.x, uv.min.y, uv.min.x, uv.max.y, uv.max.x, uv.max.y, uv.max.x, uv.min.y];
            let buffer = create_buffer(&engine.gl, BufferType::Attribute, 8, Some(BufferData::Float(&uvs[..])), false);
            engine.res_mgr.create(uv_hash, buffer)
        }
    }
}

fn create_geo<C: HalContext + 'static>(uv_hash: &u64, geo_res: GeometryRes, engine: &mut Engine<C>) -> Share<GeometryRes> {
    let mut hasher = FxHasher32::default();
    uv_hash.hash(&mut hasher);
    POSITIONUNIT.hash(&mut hasher);
    INDEXUNIT.hash(&mut hasher);
    let hash = hasher.finish();

    match engine.res_mgr.get::<GeometryRes>(&hash) {
        Some(r) => r,
        None => engine.res_mgr.create(hash, geo_res)
    }
}

#[inline]
fn modify_matrix(
    render_obj: &mut RenderObj,
    layout: &Layout,
    depth: f32,
    world_matrix: &WorldMatrix,
    transform: &Transform,
    pos: &Aabb2,
    hash_radius: bool,
){
    if hash_radius {
        let arr = create_let_top_offset_matrix(layout, world_matrix, transform, 0.0, 0.0, depth);
        render_obj.paramter.set_value("worldMatrix", Share::new(  WorldMatrixUbo::new(UniformValue::MatrixV4(arr)) ));
    } else {
        let arr = create_unit_offset_matrix(pos.max.x - pos.min.x, pos.max.y - pos.min.y, 0.0, 0.0, layout, world_matrix, transform, depth);
        render_obj.paramter.set_value("worldMatrix", Share::new( WorldMatrixUbo::new(UniformValue::MatrixV4(arr)) ));
    }
}

fn use_layout_pos<C: HalContext + 'static>(render_obj: &mut RenderObj, uv: Aabb2, layout: &Layout, radius: &Point2, engine: &mut Engine<C>){
    let start_x = layout.border_left;
    let start_y = layout.border_top;
    let end_x = layout.width - layout.border_right;
    let end_y = layout.height - layout.border_bottom;
    let (positions, indices) = if radius.x == 0.0 || layout.width == 0.0 || layout.height == 0.0 {
        (
            vec![
                start_x, start_y,
                start_x, end_y,
                end_x, end_y,
                end_x, start_y,
            ],
            vec![0, 1, 2, 3],
        )
    } else {
        split_by_radius(start_x, start_y, end_x - start_x, end_y - start_y, radius.x - start_x, None)
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
    // debug_println!("v positions: {:?}, indices_arr: {:?}, cfg: {:?}, percent: [}, {}], start: [{}, {}], end: [{}, {}]",  &positions, indices_arr, vec![LgCfg{unit: 1, data: vec![uv.min.y, uv.max.y]}], 0.0, 1.0, 0.0, 0.0, 0.0, layout.height);
    let mut uvs = Vec::with_capacity(u[0].len());
    for i in 0..u[0].len() {
        uvs.push(u[0][i]);
        uvs.push(v[0][i]);
    }

    let pos_buffer = create_buffer(&engine.gl, BufferType::Attribute, positions.len(), Some(BufferData::Float(positions.as_slice())), false);
    let uv_buffer = create_buffer(&engine.gl, BufferType::Attribute, uvs.len(), Some(BufferData::Float(uvs.as_slice())), false);
    let indices_buffer = create_buffer(&engine.gl, BufferType::Indices, indices.len(), Some(BufferData::Short(indices.as_slice())), false);

    let geo = create_geometry(&engine.gl);
    engine.gl.geometry_set_attribute(&geo, &AttributeName::Position, &pos_buffer, 2).unwrap();
    engine.gl.geometry_set_attribute(&geo, &AttributeName::UV0, &uv_buffer, 2).unwrap();
    engine.gl.geometry_set_indices_short(&geo, &indices_buffer).unwrap();

    let geo_res = GeometryRes{geo: geo, buffers: vec![Share::new(pos_buffer), Share::new(indices_buffer), Share::new(uv_buffer)]};
    render_obj.geometry = Some(Share::new(geo_res));
}

// 获得图片的4个点(逆时针)的坐标和uv的Aabb
fn get_pos_uv(img: &Image, clip: Option<&ImageClip>, fit: Option<&ObjectFit>, layout: &Layout) -> (Aabb2, Aabb2){
    let (size, mut uv1, mut uv2) = match clip {
        Some(c) => {
            let size = Vector2::new(img.src.width as f32 * (c.max.x - c.min.x).abs(), img.src.height as f32 * (c.max.y - c.min.y).abs());
            (size, c.min, c.max)
        },
        _ => (Vector2::new(img.src.width as f32, img.src.height as f32), Point2::new(0.0,0.0), Point2::new(1.0,1.0))
    };
    let mut p1 = Point2::new(layout.border_left + layout.padding_left, layout.border_top + layout.padding_top);
    let mut p2 = Point2::new(layout.width - layout.border_right - layout.padding_right, layout.height - layout.border_bottom - layout.padding_bottom);
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
        FitType::Repeat => panic!("TODO"), // TODO
        FitType::RepeatX => panic!("TODO"), // TODO
        FitType::RepeatY => panic!("TODO"), // TODO
        FitType::Fill => () // 填充。 内容拉伸填满整个容器，不保证保持原有的比例
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

impl_system!{
    ImageSys<C> where [C: HalContext + 'static],
    true,
    {
        // MultiCaseListener<Node, Image<C>, CreateEvent>
        // MultiCaseListener<Node, Image<C>, ModifyEvent>
        // MultiCaseListener<Node, Image<C>, DeleteEvent>
        // MultiCaseListener<Node, Layout, ModifyEvent>
        // MultiCaseListener<Node, Opacity, ModifyEvent>
        // MultiCaseListener<Node, WorldMatrixRender, CreateEvent>
        // MultiCaseListener<Node, WorldMatrixRender, ModifyEvent>
        // MultiCaseListener<Node, ImageClip, CreateEvent>
        // MultiCaseListener<Node, ImageClip, ModifyEvent>
        // MultiCaseListener<Node, ObjectFit, CreateEvent>
        // MultiCaseListener<Node, ObjectFit, ModifyEvent>
        MultiCaseListener<Node, Image, DeleteEvent>
    }
}