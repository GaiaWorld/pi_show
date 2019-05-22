pub mod constant;

use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hasher, Hash };
use std::mem::transmute;

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share};
use hal_core::{ Pipeline, RasterState, BlendState, StencilState, DepthState, Context, ShaderType, Geometry, Uniforms, SamplerDesc};
use atom::Atom;
use fnv::FnvHashMap;

use component::Matrix4;
use component::user::Transform;
use component::calc::WorldMatrix;
use component::{Vector3, Color};
use system::util::constant::{POSITION, WORLD_MATRIX, COMMON, ALPHA, CLIP_INDEICES, CLIP};
use render::engine::Engine;
use single::RenderObjs;
use layout::Layout;
use Node;

pub fn cal_matrix(
    id: usize,
    world_matrixs: &MultiCaseImpl<Node, WorldMatrix>,
    transforms: &MultiCaseImpl<Node, Transform>,
    layouts: &MultiCaseImpl<Node, Layout>,
) -> Matrix4 {
    let world_matrix = unsafe { world_matrixs.get_unchecked(id) };
    let transform = unsafe { transforms.get_unchecked(id) };
    let layout = unsafe { layouts.get_unchecked(id) };

    let origin = transform.origin.to_value(layout.width, layout.height);

    if origin.x != 0.0 || origin.y != 0.0 {
        return world_matrix.0 * Matrix4::from_translation(Vector3::new(origin.x, origin.y, 0.0));
    }
    
    world_matrix.0.clone()
}

pub fn color_is_opaque(color: &Color) -> bool{
    match &color {
        Color::RGB(c) | Color::RGBA(c) => {
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
        Color::RadialGradient(g) => {
            for c in g.list.iter() {
                if c.rgba.a < 1.0 {
                    return false
                }
            }
            return true;
        }
    }
}

pub fn create_geometry<C: Context>(gl: &mut C) -> Arc<<C as Context>::ContextGeometry> {
    match gl.create_geometry() {
        Ok(r) => r,
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
    Arc::get_mut(geometry).unwrap().set_attribute(&POSITION.clone(), 3, Some(&buffer[0..12]), false);
}

pub fn set_world_matrix_ubo<C: Context + 'static>(
    id: usize,
    index: usize,
    world_matrix: &Matrix4,
    render_objs: &mut SingleCaseImpl<RenderObjs<C>>,
){
    let mut ubos = &mut unsafe { render_objs.get_unchecked_mut(index) }.ubos;
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
    let mut ubos = &mut obj.ubos;
    if by_overflow == 0 {
        ubos.remove(&CLIP);
        if defines.get_clip() == true {
            defines.set_clip(false);
            obj.pipeline = engine.create_pipeline(start_hash, vs, fs, defines.list().as_slice(), rs, bs, ss, ds);
        }
        return;
    }
    ubos.entry(CLIP.clone()).and_modify(|by_overflow_ubo|{
        Arc::make_mut(by_overflow_ubo).set_float_1(&CLIP_INDEICES, by_overflow as f32);//裁剪属性
    }).or_insert_with(||{
        let mut by_overflow_ubo = Uniforms::new();
        by_overflow_ubo.set_float_1(&CLIP_INDEICES, by_overflow as f32); //裁剪属性
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