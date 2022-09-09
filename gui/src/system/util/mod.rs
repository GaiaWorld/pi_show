pub mod constant;

use share::Share;
use std::hash::{Hash, Hasher};

use hash::DefaultHasher;
use ordered_float::NotNan;

use atom::Atom;
use ecs::monitor::NotifyImpl;
use ecs::{MultiCaseImpl, SingleCaseImpl};
use hal_core::*;
use map::vecmap::VecMap;

use crate::component::user::*;
use crate::component::{calc::LayoutR, calc::*};
use crate::entity::Node;
use crate::render::engine::Engine;
use crate::render::res::BufferRes;
use crate::single::*;
use crate::system::util::constant::*;
use crate::Z_MAX;

lazy_static! {
    // 四边形几何体的hash值
    pub static ref QUAD_GEO_HASH: u64 = 0;
    static ref UV: Atom = Atom::from("UV");
}

const EPSILON: f32 = std::f32::EPSILON * 10240.0;
#[inline]
pub fn eq_f32(v1: f32, v2: f32) -> bool { v1 == v2 || ((v2 - v1).abs() <= EPSILON) }

pub fn cal_matrix(
    id: usize,
    world_matrixs: &MultiCaseImpl<Node, WorldMatrix>,
    transforms: &MultiCaseImpl<Node, Transform>,
    layouts: &MultiCaseImpl<Node, LayoutR>,
    transform: &Transform,
) -> Matrix4 {
    let world_matrix = &world_matrixs[id];
    let layout = &layouts[id];
    let transform = match transforms.get(id) {
        Some(r) => r,
        None => transform,
    };

    let origin = transform
        .origin
        .to_value(layout.rect.end - layout.rect.start, layout.rect.bottom - layout.rect.top);

    if origin.x != 0.0 || origin.y != 0.0 {
        return world_matrix.0 * Matrix4::new_translation(&Vector3::new(-origin.x, -origin.y, 0.0));
    }

    world_matrix.0.clone()
}

pub trait DefinesList {
    fn list(&self) -> Vec<Atom>;
}

pub trait DefinesClip {
    fn set_clip(&mut self, value: bool);
    fn get_clip(&self) -> bool;
}

pub fn cal_border_radius(border_radius: Option<&BorderRadius>, layout: &LayoutR) -> Point2 {
    match border_radius {
        Some(border_radius) => Point2::new(
            match border_radius.x {
                LengthUnit::Pixel(r) => r,
                LengthUnit::Percent(r) => r * (layout.rect.end - layout.rect.start),
            },
            match border_radius.y {
                LengthUnit::Pixel(r) => r,
                LengthUnit::Percent(r) => r * (layout.rect.bottom - layout.rect.top),
            },
        ),
        None => Point2::new(0.0, 0.0),
    }
}

pub fn radius_quad_hash(hasher: &mut DefaultHasher, radius: f32, width: f32, height: f32) {
    RADIUS_QUAD_POSITION_INDEX.hash(hasher);
    NotNan::new(radius).unwrap().hash(hasher);
    NotNan::new(width).unwrap().hash(hasher);
    NotNan::new(height).unwrap().hash(hasher);
}

pub fn f32_4_hash(r: f32, g: f32, b: f32, a: f32) -> u64 {
    let mut hasher = DefaultHasher::default();
    if let Err(_r) = NotNan::new(r) {
        log::info!("r=============={}", r);
    }
    if let Err(g) = NotNan::new(g) {
        log::info!("g=============={}", g);
    }
    if let Err(r) = NotNan::new(b) {
        log::info!("b=============={}", b);
    }
    if let Err(r) = NotNan::new(a) {
        log::info!("a=============={}", a);
    }
    NotNan::new(r).unwrap().hash(&mut hasher);
    NotNan::new(g).unwrap().hash(&mut hasher);
    NotNan::new(b).unwrap().hash(&mut hasher);
    NotNan::new(a).unwrap().hash(&mut hasher);
    hasher.finish()
}

pub fn f32_4_hash_(r: f32, g: f32, b: f32, a: f32, hasher: &mut DefaultHasher) {
    if let Err(_r) = NotNan::new(r) {
        log::info!("r=============={}", r);
    }
    if let Err(g) = NotNan::new(g) {
        log::info!("g=============={}", g);
    }
    if let Err(r) = NotNan::new(b) {
        log::info!("b=============={}", b);
    }
    if let Err(r) = NotNan::new(a) {
        log::info!("a=============={}", a);
    }
    NotNan::new(r).unwrap().hash(hasher);
    NotNan::new(g).unwrap().hash(hasher);
    NotNan::new(b).unwrap().hash(hasher);
    NotNan::new(a).unwrap().hash(hasher);
}

pub fn f32_3_hash_(x: f32, y: f32, z: f32, hasher: &mut DefaultHasher) {
    NotNan::new(x).unwrap().hash(hasher);
    NotNan::new(y).unwrap().hash(hasher);
    NotNan::new(z).unwrap().hash(hasher);
}

pub fn f32_3_hash(x: f32, y: f32, z: f32) -> u64 {
    let mut hasher = DefaultHasher::default();
    NotNan::new(x).unwrap().hash(&mut hasher);
    NotNan::new(y).unwrap().hash(&mut hasher);
    NotNan::new(z).unwrap().hash(&mut hasher);
    hasher.finish()
}

#[inline]
pub fn cal_uv_hash(uv1: &Point2, uv2: &Point2) -> u64 {
    let mut hasher = DefaultHasher::default();
    UV.hash(&mut hasher);
    f32_4_hash_(uv1.x, uv1.y, uv2.x, uv2.y, &mut hasher);
    hasher.finish()
}

pub fn create_uv_buffer<C: HalContext + 'static>(uv_hash: u64, uv1: &Point2, uv2: &Point2, engine: &mut Engine<C>) -> Share<BufferRes> {
    match engine.buffer_res_map.get(&uv_hash) {
        Some(r) => r,
        None => {
            let uvs = [uv1.x, uv1.y, uv1.x, uv2.y, uv2.x, uv2.y, uv2.x, uv1.y];
            engine.create_buffer_res(uv_hash, BufferType::Attribute, 8, Some(BufferData::Float(&uvs[..])), false)
        }
    }
}

// 计算矩阵变化， 将其变换到0~1, 以左上角为中心
pub fn create_unit_matrix_by_layout(layout: &LayoutR, matrix: &WorldMatrix, transform: &Transform, depth: f32) -> Vec<f32> {
    let width = layout.rect.end - layout.rect.start - layout.border.start - layout.border.end;
    let height = layout.rect.bottom - layout.rect.top - layout.border.top - layout.border.bottom;

    create_unit_offset_matrix(width, height, layout.border.start, layout.border.top, layout, matrix, transform, depth)
}

// 计算矩阵变化， 将其变换到0~1, 以左上角为中心
pub fn create_unit_offset_matrix(
    width: f32,
    height: f32,
    h: f32,
    v: f32,
    layout: &LayoutR,
    matrix: &WorldMatrix,
    transform: &Transform,
    depth: f32,
) -> Vec<f32> {
    let origin = transform
        .origin
        .to_value(layout.rect.end - layout.rect.start, layout.rect.bottom - layout.rect.top);

    let matrix = matrix
        * WorldMatrix(
            Matrix4::new(
                width,
                0.0,
                0.0,
                -origin.x + h,
                0.0,
                height,
                0.0,
                -origin.y + v,
                0.0,
                0.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ),
            false,
        );
    let slice: &[f32] = matrix.as_slice();
    let mut arr = Vec::from(slice);
    return arr;
}

// 将矩阵变换到布局框的左上角, 并偏移一定距离
#[inline]
pub fn create_let_top_offset_matrix(
    layout: &LayoutR,
    matrix: &WorldMatrix,
    transform: &Transform,
    h: f32,
    v: f32,
    // depth: f32,
) -> Vec<f32> {
    // let depth = -depth / (Z_MAX + 1.0);
    // let depth = depth1;
    let m = let_top_offset_matrix(layout, matrix, transform, h, v);
    let slice: &[f32] = m.as_slice();
    let mut arr = Vec::from(&slice[..]);
    return arr;
    // let origin = transform.origin.to_value(layout.rect.end - layout.rect.start, layout.rect.bottom - layout.rect.top);
    // if origin.x == 0.0 && origin.y == 0.0 && h == 0.0 && v == 0.0 {
    //     let slice: &[f32] = matrix.as_slice();
    //     let mut arr = Vec::from(slice);
    //     // arr[14] = depth;
    //     return arr;
    // } else {
    //     let matrix = matrix
    //         * WorldMatrix(
    // 			Matrix4::new_translation(&Vector3::new(-origin.x + h,
    //                 -origin.y + v,
    //                 0.0)),
    //             false,
    //         );
    //     let slice: &[f32] = matrix.as_slice();
    //     let mut arr = Vec::from(&slice[..]);
    //     // arr[14] = depth;
    //     return arr;
    // }
}

#[inline]
pub fn let_top_offset_matrix(layout: &LayoutR, matrix: &WorldMatrix, transform: &Transform, h: f32, v: f32) -> WorldMatrix {
    // let depth = -depth / (Z_MAX + 1.0);
    // let depth = depth1;

    let origin = transform
        .origin
        .to_value(layout.rect.end - layout.rect.start, layout.rect.bottom - layout.rect.top);
    if origin.x == 0.0 && origin.y == 0.0 && h == 0.0 && v == 0.0 {
        matrix.clone()
    } else {
        matrix * WorldMatrix(Matrix4::new_translation(&Vector3::new(-origin.x + h, -origin.y + v, 0.0)), false)
    }
}

#[inline]
pub fn modify_matrix(index: usize, matrix: Vec<f32>, render_obj: &mut RenderObj, notify: &NotifyImpl) {
    render_obj
        .paramter
        .set_value("worldMatrix", Share::new(WorldMatrixUbo::new(UniformValue::MatrixV4(matrix))));
    notify.modify_event(index, "ubos", 0);
}

#[inline]
pub fn geo_box(layout: &LayoutR) -> Aabb2 {
    Aabb2::new(
        Point2::new(layout.border.start, layout.border.top),
        Point2::new(
            layout.rect.end - layout.rect.start - layout.border.end,
            layout.rect.bottom - layout.rect.top - layout.border.bottom,
        ),
    )
}

#[inline]
pub fn to_ucolor_defines(vs_defines: &mut dyn Defines, fs_defines: &mut dyn Defines) -> bool {
    match fs_defines.add("UCOLOR") {
        Some(_) => false,
        None => {
            vs_defines.remove("VERTEX_COLOR");
            fs_defines.remove("VERTEX_COLOR");
            true
        }
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

pub fn modify_opacity<C: HalContext + 'static>(_engine: &mut Engine<C>, render_obj: &mut RenderObj, default_state: &CommonState) {
    if render_obj.is_opacity == false {
        // render_obj.state.bs = default_state.df_bs.clone();
        render_obj.state.ds = default_state.df_ds.clone();
    } else {
        // render_obj.state.bs = default_state.tarns_bs.clone();
        render_obj.state.ds = default_state.tarns_ds.clone();
    }
    // let mut bs = engine.gl.bs_get_desc(render_obj.state.bs.as_ref()).clone();
    // let mut ds = engine.gl.ds_get_desc(render_obj.state.ds.as_ref()).clone();
    // if render_obj.is_opacity == false {
    //     bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
    //     ds.set_write_enable(false);

    //     render_obj.state.bs = engine.create_bs_res(bs);
    //     render_obj.state.ds = engine.create_ds_res(ds);
    // }else {
    //     bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
    //     // bs.set_rgb_factor(BlendFactor::One, BlendFactor::Zero);
    //     ds.set_write_enable(true);

    //     render_obj.state.bs = engine.create_bs_res(bs);
    //     render_obj.state.ds = engine.create_ds_res(ds);
    // }
}

// 计算两个aabb的交集
#[inline]
pub fn intersect(a: &Aabb2, b: &Aabb2) -> Option<Aabb2> {
    let r = Aabb2::new(
        Point2::new(a.mins.x.max(b.mins.x), a.mins.y.max(b.mins.y)),
        Point2::new(a.maxs.x.min(b.maxs.x), a.maxs.y.min(b.maxs.y)),
    );
    if r.maxs.x <= r.mins.x || r.maxs.y <= r.mins.y {
        return None;
    }
    Some(r)
}
#[inline]
pub fn new_render_obj(
    context: usize,
    depth_diff: f32,
    is_opacity: bool,
    vs_name: Atom,
    fs_name: Atom,
    paramter: Share<dyn ProgramParamter>,
    state: State,
) -> RenderObj {
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
        post_process: None,
    }
}

// #[inline]
// pub fn create_render_obj1(
//     context: usize,
//     depth_diff: f32,
//     is_opacity: bool,
//     vs_name: Atom,
//     fs_name: Atom,
//     paramter: Share<dyn ProgramParamter>,
//     default_state: &DefaultState,
//     render_objs: &mut SingleCaseImpl<RenderObjs>,
//     render_map: &mut VecMap<usize>,
// ) -> usize{
//     let state = State {
//         bs: default_state.df_bs.clone(),
//         rs: default_state.df_rs.clone(),
//         ss: default_state.df_ss.clone(),
//         ds: default_state.df_ds.clone(),
//     };
//     let notify = render_objs.get_notify();
//     let render_index = render_objs.insert(
//         new_render_obj(context, depth_diff, is_opacity, vs_name, fs_name, paramter, state),
//         Some(notify)
//     );
//     render_map.insert(context, render_index);
//     render_index
// }

#[inline]
pub fn create_render_obj(
    context: usize,
    depth_diff: f32,
    is_opacity: bool,
    vs_name: Atom,
    fs_name: Atom,
    paramter: Share<dyn ProgramParamter>,
    default_state: &CommonState,
    render_objs: &mut SingleCaseImpl<RenderObjs>,
    render_map: &mut VecMap<usize>,
) -> usize {
    let state = State {
        bs: default_state.df_bs.clone(),
        rs: default_state.df_rs.clone(),
        ss: default_state.df_ss.clone(),
        ds: default_state.df_ds.clone(),
    };
    let notify = unsafe { &*(render_objs.get_notify_ref() as *const NotifyImpl) };
    let render_index = render_objs.insert(
        new_render_obj(context, depth_diff, is_opacity, vs_name, fs_name, paramter, state),
        Some(notify),
    );
    render_map.insert(context, render_index);
    render_index
}

#[inline]
pub fn new_render_obj1(
    context: usize,
    depth_diff: f32,
    is_opacity: bool,
    vs_name: Atom,
    fs_name: Atom,
    paramter: Share<dyn ProgramParamter>,
    default_state: &CommonState,
) -> RenderObj {
    let state = State {
        bs: default_state.df_bs.clone(),
        rs: default_state.df_rs.clone(),
        ss: default_state.df_ss.clone(),
        ds: default_state.df_ds.clone(),
    };
    new_render_obj(context, depth_diff, is_opacity, vs_name, fs_name, paramter, state)
}
