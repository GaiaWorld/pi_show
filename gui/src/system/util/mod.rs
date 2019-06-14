pub mod constant;

use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hasher, Hash };
use std::mem::transmute;

use ecs::{Component, SingleCaseImpl, MultiCaseImpl, Share};
use hal_core::{ RasterState, BlendState, StencilState, DepthState, Context, Geometry, SamplerDesc, AttributeName};
use atom::Atom;


use component::user::*;
use component::calc::WorldMatrix;
use system::util::constant::{WORLD_MATRIX, CLIP_INDICES, CLIP};
use render::engine::Engine;
use single::{ RenderObjs, DefaultTable };
use entity::Node;

pub fn cal_matrix(
    id: usize,
    world_matrixs: &MultiCaseImpl<Node, WorldMatrix>,
    transforms: &MultiCaseImpl<Node, Transform>,
    layouts: &MultiCaseImpl<Node, Layout>,
    default_table: &SingleCaseImpl<DefaultTable>,
) -> Matrix4 {
    let world_matrix = unsafe { world_matrixs.get_unchecked(id) };
    let transform = get_or_default(id, transforms, default_table);
    let layout = unsafe { layouts.get_unchecked(id) };

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

pub fn create_geometry<C: Context>(gl: &mut C) -> Arc<<C as Context>::ContextGeometry> {
    match gl.create_geometry() {
        Ok(r) => Arc::new(r),
        Err(_) => panic!("create_geometry error"),
    }
}

pub fn set_atrribute<C: Context>(layout: &Layout, z_depth: f32, offset:(f32, f32), geometry: &mut Arc<<C as Context>::ContextGeometry>){
    let (start_x, start_y, end_x, end_y) = (offset.0, offset.1, layout.width + offset.0, layout.height + offset.1);
    let buffer = [
        start_x, start_y, z_depth, // left_top
        start_x, end_y,   z_depth, // left_bootom
        end_x,   end_y,   z_depth, // right_bootom
        end_x,   start_y, z_depth, // right_top
    ];
    Arc::get_mut(geometry).unwrap().set_attribute(&AttributeName::Position, 3, Some(&buffer[0..12]), false).unwrap();
}

pub fn set_world_matrix_ubo<C: Context + 'static>(
    _id: usize,
    index: usize,
    world_matrix: &Matrix4,
    render_objs: &mut SingleCaseImpl<RenderObjs<C>>,
){
    let ubos = &mut unsafe { render_objs.get_unchecked_mut(index) }.ubos;
    let slice: &[f32; 16] = world_matrix.as_ref();
    Arc::make_mut(ubos.get_mut(&WORLD_MATRIX).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
}

pub fn by_overflow_change<D: DefinesList + DefinesClip, C: Context + Share>(
    by_overflow: usize,
    index: usize,
    defines: &mut D,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
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
        Arc::make_mut(by_overflow_ubo).set_float_1(&CLIP_INDICES, by_overflow as f32);//裁剪属性
    }).or_insert_with(||{
        let mut by_overflow_ubo = engine.gl.create_uniforms();
        by_overflow_ubo.set_float_1(&CLIP_INDICES, by_overflow as f32); //裁剪属性
        Arc::new(by_overflow_ubo)
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
    let mut h = DefaultHasher::new();
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