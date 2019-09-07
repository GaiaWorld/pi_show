pub mod oct;
pub mod class;
pub mod style_parse;

use share::Share;
use std::any::{TypeId, Any};
use std::default::Default;
// use std::collections::hash_map::Entry;

use cgmath::Ortho;
use slab::Slab;
use atom::Atom;
use hal_core::*;
use ecs::{ Write };
use ecs::monitor::NotifyImpl;
use map::vecmap::VecMap;
use fx_hashmap::FxHashMap32;

use component::user::*;
use component::calc::{ClipBox, WorldMatrix};

use render::res::*;

pub use single::oct::Oct;
pub use single::class::*;

pub struct OverflowClip{
    pub id_map: FxHashMap32<usize, usize>,
    pub clip: Slab<Clip>, //[[Point2;4];16], //Vec<(clip_view, has_rotate, old_has_rotate)> 最多32个
    pub clip_map: FxHashMap32<usize, (Aabb3, Share<dyn UniformBuffer>)>,
    // pub id_vec: [usize;16],
    // pub clip: [[Point2;4];16],
}

#[derive(Debug)]
pub struct Clip {
    pub view: [Point2;4],
    pub has_rotate: bool,
    pub old_has_rotate: bool,
    pub node_id: usize,
}

impl OverflowClip {
    pub fn insert_aabb(&mut self, key: usize, value: Aabb3, view_matrix: &WorldMatrix) -> &(Aabb3, Share<dyn UniformBuffer>) {
        let min = view_matrix * Vector4::new(value.min.x, value.min.y, 0.0, 0.0);
        let max = view_matrix * Vector4::new(value.max.x, value.max.y, 0.0, 0.0);
        let (w, h) = ((max.x - min.x) / 2.0, (max.y - min.y) / 2.0 );
        let ubo = ClipBox::new(UniformValue::Float4(min.x + w, min.y + h, w, h));

        // self.clip_map.entry(key).and_modify(|e|{
        //     *e = (value, Share::new(ubo))
        // }).or {
        //     Entry::Occupied(mut r) => {
        //         r.insert((value, Share::new(ubo)));
        //         r.get()
        //     },
        //     Entry::Vacant(r) => r.insert((value, Share::new(ubo))),
        // }
        self.clip_map.insert(key, (value, Share::new(ubo)));
        self.clip_map.get(&key).unwrap()
    }
}

// pub struct ClipIndex {
    
// }

impl Default for OverflowClip {
    fn default() -> Self {
        let mut r = Self {
            id_map: FxHashMap32::default(),
            clip: Slab::default(),
            clip_map: FxHashMap32::default(),
            // id_vec: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            // clip: [
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            //     [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            // ],
        };
        r.insert_aabb(0, Aabb3::new(Point3::new(std::f32::MIN, std::f32::MIN, 0.0), Point3::new(std::f32::MAX, std::f32::MAX, 0.0)), &WorldMatrix::default());
        r
    }
}

#[derive(Debug)]
pub struct ViewMatrix(pub WorldMatrix);

#[derive(Clone)]
pub struct ProjectionMatrix(pub WorldMatrix);

impl ProjectionMatrix {
    pub fn new(width: f32, height: f32, near: f32, far: f32) -> ProjectionMatrix{
        let ortho = Ortho {
            left: 0.0,
            right: width,
            bottom: height, 
            top: 0.0,
            near: near,
            far: far,
        };
        ProjectionMatrix(WorldMatrix(Matrix4::from(ortho), false))
        // let (left, right, top, bottom, near, far) = (0.0, width, 0.0, height, -8388607.0, 8388608.0);
        // ProjectionMatrix(Matrix4::new(
        //         2.0 / (right - left),                  0.0,                               0.0,                        0.0,
        //             0.0,                     2.0 / (top - bottom),                       0.0,                        0.0,
        //             0.0,                              0.0,                       -2.0 / (far - near),   -(far + near) / (far - near),
        //     -(right + left) / (right - left), -(top + bottom) / (top - bottom),           0.0,                        1.0
        // ))
    }
}

// 图片等待表
#[derive(Default)]
pub struct ImageWaitSheet {
    pub wait: FxHashMap32<Atom, Vec<ImageWait>>,
    pub finish: Vec<(Atom, Share<TextureRes>, Vec<ImageWait>)>,
    pub loads: Vec<Atom>,
}

impl ImageWaitSheet {
    pub fn add(&mut self, name: &Atom, wait: ImageWait) {
        let loads = &mut self.loads;
        self.wait.entry(name.clone()).or_insert_with(||{
            loads.push(name.clone());
            Vec::with_capacity(1)
        }).push(wait);
    }
}

#[derive(Debug)]
pub enum ImageType {
    ImageClass,
    ImageLocal,
    BorderImageClass,
    BorderImageLocal,
}

#[derive(Debug)]
pub struct ImageWait {
    pub ty: ImageType,
    pub id: usize,
}

pub struct UnitQuad(pub Share<GeometryRes>);

#[derive(Default)]
pub struct DirtyList(pub Vec<usize>);

pub struct DefaultState{
    pub df_rs: Share<HalRasterState>,
    pub df_bs: Share<HalBlendState>,
    pub df_ss: Share<HalStencilState>,
    pub df_ds: Share<HalDepthState>,

    pub tarns_bs: Share<HalBlendState>,
    pub tarns_ds: Share<HalDepthState>,
}

impl DefaultState {
    pub fn new<C: HalContext + 'static>(gl: &C) -> Self {
        let df_rs = RasterStateDesc::default();
        let mut df_bs = BlendStateDesc::default();
        let df_ss = StencilStateDesc::default();
        let df_ds = DepthStateDesc::default();

        df_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);

        let mut tarns_bs = BlendStateDesc::default();
        tarns_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);

        let mut tarns_ds = DepthStateDesc::default();
        tarns_ds.set_write_enable(false);

        Self {
            df_rs: Share::new(gl.rs_create(df_rs).unwrap()),
            df_bs: Share::new(gl.bs_create(df_bs).unwrap()),
            df_ss: Share::new(gl.ss_create(df_ss).unwrap()),
            df_ds: Share::new(gl.ds_create(df_ds).unwrap()),
            tarns_bs: Share::new(gl.bs_create(tarns_bs).unwrap()),
            tarns_ds: Share::new(gl.ds_create(tarns_ds).unwrap()),
        }
    }
} 

pub struct Data<C>{
    map: Slab<C>,
    notify: NotifyImpl,
}

impl<C> Default for Data<C> {
    fn default() -> Self {
        Self{
            map: Slab::default(),
            notify: NotifyImpl::default(),
        }
    }
}

impl<C> Data<C> {
    pub fn get(&self, id: usize) -> Option<&C> {
        self.map.get(id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut C> {
        self.map.get_mut(id)
    }

    pub unsafe fn get_unchecked(&self, id: usize) -> &C {
        self.map.get_unchecked(id)
    }

    pub unsafe fn get_unchecked_mut(&mut self, id: usize) -> &mut C {
        self.map.get_unchecked_mut(id)
    }

    pub fn get_write(&mut self, id: usize) -> Option<Write<C>> {
        match self.map.get_mut(id) {
            Some(r) => Some(Write::new(id, r, &self.notify)),
            None => None,
        }
    }

    pub unsafe fn get_unchecked_write(&mut self, id: usize) -> Write<C> {
        Write::new(id, self.map.get_unchecked_mut(id), &self.notify)
    }

    pub fn create(&mut self, c: C) -> usize {
        let r = self.map.insert(c);
        self.notify.create_event(r);
        r
    }
    
    pub fn delete(&mut self, id: usize) {
        self.notify.delete_event(id);
        self.map.remove(id);
    }

    pub fn get_notify(&self) -> NotifyImpl{
        self.notify.clone()
    }
}

#[derive(Clone)]
pub struct State{
    pub rs: Share<HalRasterState>,
    pub bs: Share<HalBlendState>,
    pub ss: Share<HalStencilState>,
    pub ds: Share<HalDepthState>,
}

#[derive(Write)]
pub struct RenderObj{
    pub depth: f32,
    pub depth_diff: f32,
    pub visibility: bool,
    pub is_opacity: bool,
    pub vs_name: Atom,
    pub fs_name: Atom,
    pub vs_defines: Box<dyn Defines>,
    pub fs_defines: Box<dyn Defines>,
    pub paramter: Share<dyn ProgramParamter>,
    pub program_dirty: bool,

    pub program: Option<Share<ProgramRes>>,
    pub geometry: Option<Share<GeometryRes>>,
    pub state: State,

    pub context: usize,
}

#[derive(Deref, DerefMut)]
pub struct RenderObjs(pub Slab<RenderObj>);

impl Default for RenderObjs {
    fn default() -> Self {
        Self(Slab::default())
    }
}

impl RenderObjs {
    pub fn insert(&mut self, value: RenderObj, notify: Option<NotifyImpl>) -> usize {
        let id = self.0.insert(value);
        match notify {
            Some(n) => n.create_event(id),
            _ =>()
        };
        id
    }

    pub unsafe fn remove_unchecked(&mut self, id: usize, notify: Option<NotifyImpl>){
        self.0.remove(id);
        match notify {
            Some(n) => n.delete_event(id),
            _ =>()
        };
    }

    pub fn remove(&mut self, id: usize, notify: Option<NotifyImpl>){
        if self.0.contains(id) {
            self.0.remove(id);
            match notify {
                Some(n) => n.delete_event(id),
                _ =>()
            };
        }
    }

    pub unsafe fn get_unchecked_write<'a>(&'a mut self, id: usize, notify: &'a NotifyImpl) -> Write<RenderObj>{
        Write::new(id, self.0.get_unchecked_mut(id), &notify)
    }

    pub unsafe fn get_unchecked_mut(&mut self, id: usize) -> &mut RenderObj{
        self.0.get_unchecked_mut(id)
    }
}

pub struct NodeRenderMap(VecMap<Vec<usize>>);

impl NodeRenderMap {
    pub fn new () -> Self{
        Self (VecMap::default())
    }

    pub unsafe fn add_unchecked(&mut self, node_id: usize, render_id: usize, notify: &NotifyImpl) {
        let arr = self.0.get_unchecked_mut(node_id);
        arr.push(render_id);
        notify.modify_event(node_id, "add", render_id);
    }

    pub unsafe fn remove_unchecked(&mut self, node_id: usize, render_id: usize, notify: &NotifyImpl) {
        notify.modify_event(node_id, "remove", render_id);
        let arr = self.0.get_unchecked_mut(node_id);
        arr.remove_item(&render_id);
    }

    pub fn create(&mut self, node_id: usize) {
        self.0.insert(node_id, Vec::new());
    }

    pub unsafe fn destroy_unchecked(&mut self, node_id: usize) {
        self.0.remove_unchecked(node_id);
    }

    pub unsafe fn get_unchecked(&self, node_id: usize) -> &Vec<usize> {
        self.0.get_unchecked(node_id)
    }
}

pub struct RenderBegin(pub Share<RenderBeginDesc>);

pub struct DefaultTable(FxHashMap32<TypeId, Box<dyn Any>>);

impl DefaultTable {
    pub fn new() -> Self{
        Self(FxHashMap32::default())
    }

    pub fn set<T: 'static + Any>(&mut self, value: T){
        self.0.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: 'static + Any>(&self) -> Option<&T>{
        match self.0.get(&TypeId::of::<T>()) {
            Some(r) => r.downcast_ref::<T>(),
            None => None
        }
    }

    pub fn get_mut<T: 'static + Any>(&mut self) -> Option<&mut T>{
        match self.0.get_mut(&TypeId::of::<T>()) {
            Some(r) => r.downcast_mut::<T>(),
            None => None
        }
    }

    pub fn get_unchecked<T: 'static + Any>(&self) -> &T{
        self.0.get(&TypeId::of::<T>()).unwrap().downcast_ref::<T>().unwrap()
    }

    pub fn get_unchecked_mut<T: 'static + Any>(&mut self) -> &mut T{
        self.0.get_mut(&TypeId::of::<T>()).unwrap().downcast_mut::<T>().unwrap()
    }

    pub fn delete<T: 'static + Any>(&mut self){
        self.0.remove(&TypeId::of::<T>());
    }
}