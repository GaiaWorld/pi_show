pub mod constant;

use share::Share;
use std::hash::{ Hasher, Hash };
use std::mem::transmute;

use ordered_float::NotNan;
use fxhash::FxHasher32;

use ecs::{Component, MultiCaseImpl, SingleCaseImpl};
use ecs::monitor::NotifyImpl;
use hal_core::*;
use atom::Atom;
use map::vecmap::VecMap;

use component::user::*;
use component::calc::*;
// use system::util::constant::{WORLD_MATRIX, CLIP_INDICES, CLIP};
use render::engine::Engine;
use render::res_mgr::*;
use render::res::GeometryRes;
use single::*;
use entity::Node;
use system::util::constant::*;
use Z_MAX;

lazy_static! {
    // 四边形集合体的hash值
    pub static ref QUAD_GEO_HASH: u64 = 0;
}

pub fn cal_matrix(
    id: usize,
    world_matrixs: &MultiCaseImpl<Node, WorldMatrix>,
    transforms: &MultiCaseImpl<Node, Transform>,
    layouts: &MultiCaseImpl<Node, Layout>,
    transform: &Transform,
) -> Matrix4 {
    let world_matrix = unsafe { world_matrixs.get_unchecked(id) };
    let layout = unsafe { layouts.get_unchecked(id) };
    let transform = match transforms.get(id) {
        Some(r) => r,
        None => transform,
    };

    let origin = transform.origin.to_value(layout.width, layout.height);

    if origin.x != 0.0 || origin.y != 0.0 {
        return world_matrix.0 * Matrix4::from_translation(Vector3::new(-origin.x, -origin.y, 0.0));
    }
    
    world_matrix.0.clone()
}

pub fn color_is_opaque(color: &Color) -> bool{
    match &color {
        Color::RGBA(c) => {
            if c.a < 1.0 {
                return false;
            }
            return true;
        },
        Color::LinearGradient(l) => {
            for c in l.list.iter() {
                if c.rgba.a < 1.0 {
                   return false;
                }
            }
            return true;
        },
        // Color::RadialGradient(g) => {
        //     for c in g.list.iter() {
        //         if c.rgba.a < 1.0 {
        //             return false
        //         }
        //     }
        //     return true;
        // }
    }
}

#[inline]
pub fn create_geometry<C: HalContext + 'static>(gl: &C) -> HalGeometry {
    match gl.geometry_create() {
        Ok(r) => r,
        Err(e) => panic!("create_geometry error: {:?}", e),
    }
}

#[inline]
pub fn create_rs<C: HalContext + 'static>(gl: &C, rs: RasterStateDesc) -> HalRasterState {
    match gl.rs_create(rs) {
        Ok(r) => r,
        Err(e) => panic!("create_rs error: {:?}", e),
    }
}

#[inline]
pub fn create_bs<C: HalContext + 'static>(gl: &C, bs: BlendStateDesc) -> HalBlendState {
    match gl.bs_create(bs) {
        Ok(r) => r,
        Err(e) => panic!("create_bs error: {:?}", e),
    }
}

#[inline]
pub fn create_ss<C: HalContext + 'static>(gl: &C, ss: StencilStateDesc) -> HalStencilState {
    match gl.ss_create(ss) {
        Ok(r) => r,
        Err(e) => panic!("create_geometry error: {:?}", e),
    }
}

#[inline]
pub fn create_ds<C: HalContext + 'static>(gl: &C, ds: DepthStateDesc) -> HalDepthState {
    match gl.ds_create(ds) {
        Ok(r) => r,
        Err(e) => panic!("create_geometry error: {:?}", e),
    }
}

#[inline]
pub fn create_buffer<C: HalContext + 'static>(gl: &C, btype: BufferType, count: usize, data: Option<BufferData>, is_updatable: bool) -> HalBuffer {
    match gl.buffer_create(btype, count, data, is_updatable) {
        Ok(r) => r,
        Err(e) => panic!("create_buffer error: {:?}", e),
    }
}

#[inline]
pub fn create_sampler<C: HalContext + 'static>(gl: &C, sampler: SamplerDesc) -> HalSampler{
    match gl.sampler_create(sampler) {
        Ok(r) => r,
        Err(e) => panic!("create_sampler error: {:?}", e),
    }
}


// pub fn set_atrribute(layout: &Layout, z_depth: f32, offset:(f32, f32), geometry: &mut Share<GeometryRes>){
//     let (start_x, start_y, end_x, end_y) = (offset.0, offset.1, layout.width + offset.0, layout.height + offset.1);
//     let buffer = [
//         start_x, start_y, z_depth, // left_top
//         start_x, end_y,   z_depth, // left_bootom
//         end_x,   end_y,   z_depth, // right_bootom
//         end_x,   start_y, z_depth, // right_top
//     ];
//     Share::get_mut(geometry).unwrap().set_attribute(&AttributeName::Position, 3, Some(&buffer[0..12]), false).unwrap();
// }

// pub fn set_world_matrix_ubo<C: HalContext + 'static + 'static>(
//     _id: usize,
//     index: usize,
//     world_matrix: &Matrix4,
//     render_objs: &mut SingleCaseImpl<RenderObjs<C>>,
// ){
//     let ubos = &mut unsafe { render_objs.get_unchecked_mut(index) }.ubos;
//     let slice: &[f32; 16] = world_matrix.as_ref();
//     Share::make_mut(ubos.get_mut(&WORLD_MATRIX).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
// }

// pub fn by_overflow_change<D: DefinesList + DefinesClip>(
//     by_overflow: usize,
//     index: usize,
//     defines: &mut D,
//     rs: Share<HalRasterState>,
//     bs: Share<HalBlendState>,
//     ss: Share<HalStencilState>,
//     ds: Share<HalDepthState>,
//     start_hash: u64,
//     vs: &Atom,
//     fs: &Atom,
//     render_objs: &mut SingleCaseImpl<RenderObjs<C>>,
//     engine: &mut SingleCaseImpl<Engine<C>>,
// ){
//     let mut obj = &mut unsafe { render_objs.get_unchecked_mut(index) };
//     let ubos = &mut obj.ubos;
//     if by_overflow == 0 {
//         ubos.remove(&CLIP);
//         if defines.get_clip() == true {
//             defines.set_clip(false);
//             obj.pipeline = engine.create_pipeline(start_hash, vs, fs, defines.list().as_slice(), rs, bs, ss, ds);
//         }
//         return;
//     }
//     ubos.entry(CLIP.clone()).and_modify(|by_overflow_ubo|{
//         Share::make_mut(by_overflow_ubo).set_float_1(&CLIP_INDICES, by_overflow as f32);//裁剪属性
//     }).or_insert_with(||{
//         let mut by_overflow_ubo = engine.gl.create_uniforms();
//         by_overflow_ubo.set_float_1(&CLIP_INDICES, by_overflow as f32); //裁剪属性
//         Share::new(by_overflow_ubo)
//     });
//     if defines.get_clip() == false {
//         defines.set_clip(true);
//         obj.pipeline = engine.create_pipeline(start_hash, &vs, &fs, defines.list().as_slice(), rs, bs, ss, ds);
//     }
// }

pub trait DefinesList{
    fn list(&self) -> Vec<Atom>;
}

pub trait DefinesClip{
    fn set_clip(&mut self, value: bool);
    fn get_clip(&self) -> bool;
}

pub fn sampler_desc_hash(s: &SamplerDesc) -> u64{
    let mut h = FxHasher32::default();
    unsafe { transmute::<hal_core::TextureFilterMode, u8>(s.mag_filter).hash(&mut h) };
    unsafe {  transmute::<hal_core::TextureFilterMode, u8>(s.min_filter).hash(&mut h) };
    if let Some(mip_filter) = &s.mip_filter {
        unsafe { transmute::<hal_core::TextureFilterMode, u8>(mip_filter.clone()).hash(&mut h) };
    }
    unsafe { transmute::<hal_core::TextureWrapMode, u8>(s.u_wrap).hash(&mut h) };
    unsafe { transmute::<hal_core::TextureWrapMode, u8>(s.v_wrap).hash(&mut h) };
    h.finish()
}

pub fn cal_border_radius(border_radius: Option<&BorderRadius>,  layout: &Layout) -> Point2{
    match border_radius {
        Some(border_radius) => Point2{
            x: match border_radius.x {
                LengthUnit::Pixel(r) => r,
                LengthUnit::Percent(r) => r * layout.width,
            },
            y: match border_radius.y {
                LengthUnit::Pixel(r) => r,
                LengthUnit::Percent(r) => r * layout.height,
            },
        },
        None => Point2::new(0.0, 0.0),
    }
}

// pub fn positions_width_radius(border_radius: &BorderRadius, layout: &Layout, z_depth: f32, offset:(f32, f32)) -> Vec<f32>{
//     let r = cal_border_radius(border_radius, layout);
//     if r.x == 0.0 {
//         return positions_from_layout(layout, z_depth, offset);
//     }else {
//         return split_by_radius(0.0 + offset.0, 0.0 + offset.1, layout.width + offset.0, layout.height + offset.1, r.x, z_depth, None);
//     }
// }

pub fn positions_from_layout(layout: &Layout, z_depth: f32, offset:(f32, f32)) -> Vec<f32>{
    let (start_x, start_y, end_x, end_y) = (offset.0, offset.1, layout.width + offset.0, layout.height + offset.1);
    vec!(
        start_x, start_y, z_depth, // left_top
        start_x, end_y,   z_depth, // left_bootom
        end_x,   end_y,   z_depth, // right_bootom
        end_x,   start_y, z_depth, // right_top
    )
}

pub fn create_increase_vec(count: usize) -> Vec<u16>{
    let mut arr = Vec::with_capacity(count);
    for i in 0..count{
        arr.push(i as u16);
    }
    arr
}

pub fn find_item_from_vec<T: Eq>(vec: &Vec<T>, r: &T) -> usize{
    for i in 0..vec.len() {
        if vec[i] == *r {
            return i + 1;
        }
    }
    return 0;
}

pub fn get_or_default<'a, T: Component>(id: usize, c: &'a MultiCaseImpl<Node, T>, table: &'a DefaultTable) -> &'a T{
    match c.get(id) {
        Some(r) => r,
        None => table.get_unchecked::<T>(),
    }
}


// pub fn get_or_default_value<'a, T: Component>(id: usize, c: &'a MultiCaseImpl<Node, T>, value: &'a T) -> &'a T{
//     match c.get(id) {
//         Some(r) => r,
//         None => value,
//     }
// }


pub fn radius_quad_hash(hasher: &mut FxHasher32, radius: f32, width: f32, height: f32) {
    RADIUS_QUAD_POSITION_INDEX.hash(hasher);
    unsafe { NotNan::unchecked_new(radius).hash(hasher) };
    unsafe { NotNan::unchecked_new(width).hash(hasher) };
    unsafe { NotNan::unchecked_new(height).hash(hasher) };
}

pub fn f32_4_hash(r: f32, g: f32, b: f32, a: f32) -> u64 {
    let mut hasher = FxHasher32::default();
    unsafe { NotNan::unchecked_new(r).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(g).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(b).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(a).hash(&mut hasher) };
    hasher.finish()
}

pub fn f32_4_hash_(r: f32, g: f32, b: f32, a: f32, hasher: &mut FxHasher32){
    unsafe { NotNan::unchecked_new(r).hash(hasher) };
    unsafe { NotNan::unchecked_new(g).hash(hasher) };
    unsafe { NotNan::unchecked_new(b).hash(hasher) };
    unsafe { NotNan::unchecked_new(a).hash(hasher) };
}

pub fn f32_3_hash_(x: f32, y: f32, z: f32, hasher: &mut FxHasher32) {
    unsafe { NotNan::unchecked_new(x).hash(hasher) };
    unsafe { NotNan::unchecked_new(y).hash(hasher) };
    unsafe { NotNan::unchecked_new(z).hash(hasher) };
}

pub fn f32_3_hash(x: f32, y: f32, z: f32) -> u64 {
    let mut hasher = FxHasher32::default();
    unsafe { NotNan::unchecked_new(x).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(y).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(z).hash(&mut hasher) };
    hasher.finish()
}

pub fn f32_1_hash(value: f32) -> u64 {
    let mut hasher = FxHasher32::default();
    unsafe { NotNan::unchecked_new(value).hash(&mut hasher) };
    hasher.finish()
}

pub fn create_hash_res<T: Res<Key=u64> + Hash + 'static, C: HalContext + 'static>( engine: &mut Engine<C>, res: T) -> Share<T> {
    let mut hasher = FxHasher32::default();
    res.hash(&mut hasher);
    let h = hasher.finish();
    match engine.res_mgr.get::<T>(&h) {
        Some(r) => r,
        None => engine.res_mgr.create(h, res),
    }
}

pub fn create_rs_res<C: HalContext + 'static>( engine: &mut Engine<C>, rs: RasterStateDesc) -> Share<HalRasterState> {
    let mut hasher = FxHasher32::default();
    rs.hash(&mut hasher);
    let h = hasher.finish();
    match engine.res_mgr.get::<HalRasterState>(&h) {
        Some(r) => r,
        None => engine.res_mgr.create(h, create_rs(&engine.gl, rs)),
    }
}

pub fn create_bs_res<C: HalContext + 'static>( engine: &mut Engine<C>, bs: BlendStateDesc) -> Share<HalBlendState> {
    let mut hasher = FxHasher32::default();
    bs.hash(&mut hasher);
    let h = hasher.finish();
    match engine.res_mgr.get::<HalBlendState>(&h) {
        Some(r) => r,
        None => engine.res_mgr.create(h, create_bs(&engine.gl, bs)),
    }
}

pub fn create_ss_res<C: HalContext + 'static>( engine: &mut Engine<C>, ss: StencilStateDesc) -> Share<HalStencilState> {
    let mut hasher = FxHasher32::default();
    ss.hash(&mut hasher);
    let h = hasher.finish();
    match engine.res_mgr.get::<HalStencilState>(&h) {
        Some(r) => r,
        None => engine.res_mgr.create(h, create_ss(&engine.gl, ss)),
    }
}

pub fn create_ds_res<C: HalContext + 'static>( engine: &mut Engine<C>, ds: DepthStateDesc) -> Share<HalDepthState> {
    let mut hasher = FxHasher32::default();
    ds.hash(&mut hasher);
    let h = hasher.finish();
    match engine.res_mgr.get::<HalDepthState>(&h) {
        Some(r) => r,
        None => engine.res_mgr.create(h, create_ds(&engine.gl, ds)),
    }
}

pub fn create_quad_geo() -> (Vec<f32>, Vec<u16>) {
    return (
        vec![
            0.0, 0.0, 0.0, // left_top
            0.0, 1.0, 0.0, // left_bootom
            1.0, 1.0, 0.0, // right_bootom
            1.0, 0.0, 0.0, // right_top
        ],
        vec![0, 1, 2, 3],
    );
}

// pub fn quad_geo_hash() -> u64 {
//     QUAD_GEO_HASH
// }

// use ordered_float::NotNan;

// 计算矩阵变化， 将其变换到0~1, 以左上角为中心
pub fn create_unit_matrix_by_layout(
    layout: &Layout,
    matrix: &WorldMatrix,
    transform: &Transform,
    depth: f32,
) -> Vec<f32> {
    let width = layout.width - layout.border_left - layout.border_right;
    let height = layout.height - layout.border_top - layout.border_bottom;

    create_unit_offset_matrix(width, height, layout.border_left, layout.border_top, layout, matrix, transform, depth)
}

// 计算矩阵变化， 将其变换到0~1, 以左上角为中心
pub fn create_unit_offset_matrix(
    width: f32,
    height: f32,
    h: f32, 
    v: f32,
    layout: &Layout,
    matrix: &WorldMatrix,
    transform: &Transform,
    depth: f32,
) -> Vec<f32> {
    println!("depth----------------------:::::{}",depth );
    let depth = -depth/(Z_MAX + 1.0);
    println!("depth1----------------------::::{}",depth );
    let origin = transform.origin.to_value(layout.width, layout.height);

    let matrix = matrix * WorldMatrix(Matrix4::new(
        width,     0.0,                0.0, 0.0,
        0.0,       height,            0.0, 0.0,
        0.0,       0.0,                1.0, 0.0,
        -origin.x + h,  -origin.y + v, 0.0, 1.0,
    ), false);
    let slice: &[f32; 16] = matrix.as_ref();
    let mut arr = Vec::from(&slice[..]);
    arr[14] = depth;
    return arr;
}

// 将矩阵变换到布局框的左上角, 并偏移一定距离
#[inline]
pub fn create_let_top_offset_matrix(
    layout: &Layout,
    matrix: &WorldMatrix,
    transform: &Transform,
    h: f32, 
    v: f32,
    depth: f32,
) -> Vec<f32> {
    let depth = -depth/(Z_MAX + 1.0);
    // let depth = depth1;
    
    let origin = transform.origin.to_value(layout.width, layout.height);
    if origin.x == 0.0 && origin.y == 0.0 && h == 0.0 && v == 0.0 {
        let slice: &[f32; 16] = matrix.as_ref();
        let mut arr = Vec::from(&slice[..]);
        arr[14] = depth;
        return arr;
    } else {
       
        let matrix = matrix * WorldMatrix(Matrix4::new(
            1.0,       0.0,       0.0, 0.0,
            0.0,       1.0,       0.0, 0.0,
            0.0,       0.0,       1.0, 0.0,
            -origin.x + h, -origin.y + v, 0.0, 1.0,
        ), false);
        let slice: &[f32; 16] = matrix.as_ref();
        let mut arr = Vec::from(&slice[..]);
        arr[14] = depth;
        return arr;
    }
}

#[inline]
pub fn modify_matrix(
    index: usize,
    matrix: Vec<f32>,
    render_obj: &mut RenderObj,
    notify: &NotifyImpl,
){
    render_obj.paramter.set_value("worldMatrix", Share::new( WorldMatrixUbo::new(UniformValue::MatrixV4(matrix)) ));
    notify.modify_event(index, "ubos", 0);
}

#[inline]
pub fn geo_box(layout: &Layout) -> Aabb2{
    Aabb2::new(Point2::new(layout.border_left, layout.border_top), Point2::new(layout.width - layout.border_right, layout.height - layout.border_bottom))
}

#[inline]
pub fn to_ucolor_defines(vs_defines: &mut dyn Defines, fs_defines: &mut dyn Defines) -> bool {
    match fs_defines.add("UCOLOR") {
        Some(_) => false,
        None => {
            vs_defines.remove("VERTEX_COLOR");
            fs_defines.remove("VERTEX_COLOR");
            true
        },
    }
}

#[inline]
pub fn to_vex_color_defines(vs_defines: &mut dyn Defines, fs_defines: &mut dyn Defines) -> bool {
    match vs_defines.add("VERTEX_COLOR") {
        Some(_) => false,
        None => {
            fs_defines.add("VERTEX_COLOR");
            fs_defines.remove("UCOLOR");
            true
        }
    }
}

#[inline]
pub fn create_u_color_ubo<C: HalContext + 'static>(c: &CgColor, engine: &mut Engine<C>) -> Share<dyn UniformBuffer> {
    let h = f32_4_hash(c.r, c.g, c.b, c.a);
    match engine.res_mgr.get::<UColorUbo>(&h) {
        Some(r) => r,
        None => engine.res_mgr.create(h, UColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a))),
    }
}

pub fn create_default_sampler<C: HalContext + 'static>(engine: &mut Engine<C>) -> Share<HalSampler> {
    let default_sampler = SamplerDesc::default();
    let mut hasher = FxHasher32::default();
    default_sampler.hash(&mut hasher);
    let hash = hasher.finish();
    match engine.res_mgr.get::<HalSampler>(&hash) {
        Some(r) => r,
        None => engine.res_mgr.create(hash, create_sampler(&engine.gl, default_sampler)),
    }
}

// 用位置流和索引流创建一个geometry
pub fn create_p_i_geometry<C: HalContext + 'static>(positions: &[f32], indices: &[u16], engine: &mut Engine<C>) -> GeometryRes {
    let p_buffer = create_buffer(&engine.gl, BufferType::Attribute, positions.len(), Some(BufferData::Float(positions)), false);
    let i_buffer = create_buffer(&engine.gl, BufferType::Indices, indices.len(), Some(BufferData::Short(indices)), false);
    let geo = create_geometry(&engine.gl);
    engine.gl.geometry_set_vertex_count(&geo, (positions.len()/2) as u32);
    engine.gl.geometry_set_attribute(&geo, &AttributeName::Position, &p_buffer, 2).unwrap();
    engine.gl.geometry_set_indices_short(&geo, &i_buffer).unwrap();
    GeometryRes{geo: geo, buffers: vec![Share::new(p_buffer), Share::new(i_buffer)]}
}

pub fn modify_opacity<C: HalContext + 'static>(engine: &mut Engine<C>, render_obj: &mut RenderObj) {
    let mut bs = engine.gl.bs_get_desc(render_obj.state.bs.as_ref()).clone();
    let mut ds = engine.gl.ds_get_desc(render_obj.state.ds.as_ref()).clone();
    if render_obj.is_opacity == false {
        bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        ds.set_write_enable(false);
        
        render_obj.state.bs = create_bs_res(engine, bs);
        render_obj.state.ds = create_ds_res(engine, ds);
    }else {
        bs.set_rgb_factor(BlendFactor::One, BlendFactor::Zero);
        ds.set_write_enable(true);
        
        render_obj.state.bs = create_bs_res(engine, bs);
        render_obj.state.ds = create_ds_res(engine, ds);
    }
    
}

#[inline]
pub fn new_render_obj(context: usize, depth_diff: f32, is_opacity: bool, vs_name: Atom, fs_name: Atom, paramter: Share<dyn ProgramParamter>, state: State) -> RenderObj{
    RenderObj {
        depth: 0.0,
        program_dirty: true,
        visibility: false,
        vs_defines: Box::new(VsDefines::default()),
        fs_defines: Box::new(FsDefines::default()),
        program: None,
        geometry: None,
        depth_diff,
        is_opacity,
        vs_name,
        fs_name,
        paramter,
        state,
        context,       
    }
}

#[inline]
pub fn create_render_obj(
    context: usize,
    depth_diff: f32,
    is_opacity: bool,
    vs_name: Atom,
    fs_name: Atom,
    paramter: Share<dyn ProgramParamter>,
    default_state: &DefaultState,
    render_objs: &mut SingleCaseImpl<RenderObjs>,
    render_map: &mut VecMap<usize>,
) -> usize{
    let state = State {
        bs: default_state.df_bs.clone(),
        rs: default_state.df_rs.clone(),
        ss: default_state.df_ss.clone(),
        ds: default_state.df_ds.clone(),
    };
    let notify = render_objs.get_notify();
    let render_index = render_objs.insert(
        new_render_obj(context, depth_diff, is_opacity, vs_name, fs_name, paramter, state),
        Some(notify)
    );
    render_map.insert(context, render_index);
    render_index
}