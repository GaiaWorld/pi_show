pub mod constant;

use share::Share;
use std::hash::{ Hasher, Hash };

use ordered_float::NotNan;
use hash::DefaultHasher;

use ecs::{MultiCaseImpl, SingleCaseImpl};
use ecs::monitor::NotifyImpl;
use hal_core::*;
use atom::Atom;
use map::vecmap::VecMap;

use component::user::*;
use component::calc::*;
use render::engine::Engine;
use single::*;
use entity::Node;
use system::util::constant::*;
use Z_MAX;

lazy_static! {
    // 四边形几何体的hash值
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

pub trait DefinesList{
    fn list(&self) -> Vec<Atom>;
}

pub trait DefinesClip{
    fn set_clip(&mut self, value: bool);
    fn get_clip(&self) -> bool;
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

pub fn radius_quad_hash(hasher: &mut DefaultHasher, radius: f32, width: f32, height: f32) {
    RADIUS_QUAD_POSITION_INDEX.hash(hasher);
    unsafe { NotNan::unchecked_new(radius).hash(hasher) };
    unsafe { NotNan::unchecked_new(width).hash(hasher) };
    unsafe { NotNan::unchecked_new(height).hash(hasher) };
}

pub fn f32_4_hash(r: f32, g: f32, b: f32, a: f32) -> u64 {
    let mut hasher = DefaultHasher::default();
    unsafe { NotNan::unchecked_new(r).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(g).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(b).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(a).hash(&mut hasher) };
    hasher.finish()
}

pub fn f32_4_hash_(r: f32, g: f32, b: f32, a: f32, hasher: &mut DefaultHasher){
    unsafe { NotNan::unchecked_new(r).hash(hasher) };
    unsafe { NotNan::unchecked_new(g).hash(hasher) };
    unsafe { NotNan::unchecked_new(b).hash(hasher) };
    unsafe { NotNan::unchecked_new(a).hash(hasher) };
}

pub fn f32_3_hash_(x: f32, y: f32, z: f32, hasher: &mut DefaultHasher) {
    unsafe { NotNan::unchecked_new(x).hash(hasher) };
    unsafe { NotNan::unchecked_new(y).hash(hasher) };
    unsafe { NotNan::unchecked_new(z).hash(hasher) };
}

pub fn f32_3_hash(x: f32, y: f32, z: f32) -> u64 {
    let mut hasher = DefaultHasher::default();
    unsafe { NotNan::unchecked_new(x).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(y).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(z).hash(&mut hasher) };
    hasher.finish()
}

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
    let depth = -depth/(Z_MAX + 1.0);
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

pub fn modify_opacity<C: HalContext + 'static>(engine: &mut Engine<C>, render_obj: &mut RenderObj) {
    let mut bs = engine.gl.bs_get_desc(render_obj.state.bs.as_ref()).clone();
    let mut ds = engine.gl.ds_get_desc(render_obj.state.ds.as_ref()).clone();
    if render_obj.is_opacity == false {
        bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        ds.set_write_enable(false);
        
        render_obj.state.bs = engine.create_bs_res(bs);
        render_obj.state.ds = engine.create_ds_res(ds);
    }else {
        bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        // bs.set_rgb_factor(BlendFactor::One, BlendFactor::Zero);
        ds.set_write_enable(true);
        
        render_obj.state.bs = engine.create_bs_res(bs);
        render_obj.state.ds = engine.create_ds_res(ds);
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