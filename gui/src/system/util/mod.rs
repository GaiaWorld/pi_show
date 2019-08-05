pub mod constant;

use share::Share;
use std::hash::{ Hasher, Hash };
use std::mem::transmute;

use ordered_float::NotNan;
use fxhash::FxHasher32;

use ecs::{Component, SingleCaseImpl, MultiCaseImpl};
use hal_core::{ RasterState, BlendState, StencilState, DepthState, Context, Geometry, SamplerDesc, AttributeName};
use atom::Atom;


use component::user::*;
use component::calc::WorldMatrix;
use system::util::constant::{WORLD_MATRIX, CLIP_INDICES, CLIP};
use render::engine::Engine;
use single::*;
use entity::Node;
use system::util::constant::*;

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
        return world_matrix.0 * Matrix4::from_translation(Vector3::new(-origin.x * world_matrix.0.x.x, -origin.y * world_matrix.0.y.y, 0.0));
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

pub fn create_geometry<C: Context>(gl: &mut C) -> <C as Context>::ContextGeometry {
    match gl.create_geometry() {
        Ok(r) => r,
        Err(_) => panic!("create_geometry error"),
    }
}

pub fn set_atrribute<C: Context>(layout: &Layout, z_depth: f32, offset:(f32, f32), geometry: &mut Share<<C as Context>::ContextGeometry>){
    let (start_x, start_y, end_x, end_y) = (offset.0, offset.1, layout.width + offset.0, layout.height + offset.1);
    let buffer = [
        start_x, start_y, z_depth, // left_top
        start_x, end_y,   z_depth, // left_bootom
        end_x,   end_y,   z_depth, // right_bootom
        end_x,   start_y, z_depth, // right_top
    ];
    Share::get_mut(geometry).unwrap().set_attribute(&AttributeName::Position, 3, Some(&buffer[0..12]), false).unwrap();
}

pub fn set_world_matrix_ubo<C: Context + 'static>(
    _id: usize,
    index: usize,
    world_matrix: &Matrix4,
    render_objs: &mut SingleCaseImpl<RenderObjs<C>>,
){
    let ubos = &mut unsafe { render_objs.get_unchecked_mut(index) }.ubos;
    let slice: &[f32; 16] = world_matrix.as_ref();
    Share::make_mut(ubos.get_mut(&WORLD_MATRIX).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
}

pub fn by_overflow_change<D: DefinesList + DefinesClip, C: Context + 'static>(
    by_overflow: usize,
    index: usize,
    defines: &mut D,
    rs: Share<RasterState>,
    bs: Share<BlendState>,
    ss: Share<StencilState>,
    ds: Share<DepthState>,
    start_hash: u64,
    vs: &Atom,
    fs: &Atom,
    render_objs: &mut SingleCaseImpl<RenderObjs<C>>,
    engine: &mut SingleCaseImpl<Engine<C>>,
){
    let mut obj = &mut unsafe { render_objs.get_unchecked_mut(index) };
    let ubos = &mut obj.ubos;
    if by_overflow == 0 {
        ubos.remove(&CLIP);
        if defines.get_clip() == true {
            defines.set_clip(false);
            obj.pipeline = engine.create_pipeline(start_hash, vs, fs, defines.list().as_slice(), rs, bs, ss, ds);
        }
        return;
    }
    ubos.entry(CLIP.clone()).and_modify(|by_overflow_ubo|{
        Share::make_mut(by_overflow_ubo).set_float_1(&CLIP_INDICES, by_overflow as f32);//裁剪属性
    }).or_insert_with(||{
        let mut by_overflow_ubo = engine.gl.create_uniforms();
        by_overflow_ubo.set_float_1(&CLIP_INDICES, by_overflow as f32); //裁剪属性
        Share::new(by_overflow_ubo)
    });
    if defines.get_clip() == false {
        defines.set_clip(true);
        obj.pipeline = engine.create_pipeline(start_hash, &vs, &fs, defines.list().as_slice(), rs, bs, ss, ds);
    }
}

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

pub fn cal_border_radius(border_radius: &BorderRadius,  layout: &Layout) -> Point2{
    Point2{
        x: match border_radius.x {
            LengthUnit::Pixel(r) => r,
            LengthUnit::Percent(r) => r * layout.width,
        },
        y: match border_radius.y {
            LengthUnit::Pixel(r) => r,
            LengthUnit::Percent(r) => r * layout.height,
        },
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