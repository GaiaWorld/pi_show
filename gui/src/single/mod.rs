pub mod oct;

use share::Share;
use std::any::{TypeId, Any};

use cgmath::Ortho;
use slab::Slab;
use atom::Atom;
use hal_core::{Context, Uniforms, RenderBeginDesc};
use ecs::{ Write };
use ecs::monitor::NotifyImpl;
use map::vecmap::VecMap;

use component::user::{Point2, Matrix4};
use render::engine::{ PipelineInfo};
use render::res::GeometryRes;
use util::res_mgr::Res;
use FxHashMap32;

use component::{
    user::*,
};

pub use single::oct::Oct;


/// 全局文字样式
#[derive(Default)]
pub struct TextStyleClassMap (pub FxHashMap32<usize, TextStyleClazz>);

/// 全局字符串， 缓冲渲染数据
#[derive(Default)]
pub struct StrMap (pub FxHashMap32<u64, (Atom, usize)>);

#[derive(Debug)]
pub struct OverflowClip{
    pub id_vec: [usize;16],
    pub clip: [[Point2;4];16],
}

impl Default for OverflowClip {
    fn default() -> Self {
        Self {
            id_vec: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            clip: [
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
                [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
            ],
        }
    }
}

#[derive(Debug)]
pub struct ViewMatrix(pub Matrix4);

#[derive(Clone)]
pub struct ProjectionMatrix(pub Matrix4);

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
        ProjectionMatrix(Matrix4::from(ortho))
        // let (left, right, top, bottom, near, far) = (0.0, width, 0.0, height, -8388607.0, 8388608.0);
        // ProjectionMatrix(Matrix4::new(
        //         2.0 / (right - left),                  0.0,                               0.0,                        0.0,
        //             0.0,                     2.0 / (top - bottom),                       0.0,                        0.0,
        //             0.0,                              0.0,                       -2.0 / (far - near),   -(far + near) / (far - near),
        //     -(right + left) / (right - left), -(top + bottom) / (top - bottom),           0.0,                        1.0
        // ))
    }
}

#[derive(Write)]
pub struct RenderObj<C: Context + 'static>{
    pub depth: f32,
    pub depth_diff: f32,
    pub visibility: bool,
    pub is_opacity: bool,
    pub ubos: FxHashMap32<Atom, Share<Uniforms<C>>>,
    pub geometry: Option<Res<GeometryRes<C>>>,
    pub pipeline: Share<PipelineInfo>,
    pub context: usize,
    pub defines: Vec<Atom>,
    
    // pub shader_attr: Option<SharderAttr<C>>, 
}

// pub struct SharderAttr<C: Context>{
//     pub geometry: <C as Context>::ContextGeometry, //geometry 对象
//     pub pipeline: Share<Pipeline>,
// }

// impl<C: Context + Debug> fmt::Debug for RenderObj<C> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // write!(f, "Point {{ x: {}, y: {} }}", self.depth, self.visibility)
//         // write!(f, "RenderObj {{ depth: {}, visibility: {}, is_opacity: {} }}", self.depth, self.visibility, self.is_opacity)
//         // write!(f, "RenderObj {{ depth: {}, visibility: {}, is_opacity: {}, geometry: {}, pipeline: {}, ubo: {} }}", self.depth, self.visibility, self.is_opacity, self.geometry, self.pipeline, self.ubo)
//     }
// }

#[derive(Deref, DerefMut)]
pub struct RenderObjs<C: Context + 'static>(pub Slab<RenderObj<C>>);

impl<C: Context> Default for RenderObjs<C> {
    fn default() -> Self {
        Self(Slab::default())
    }
}

unsafe impl<C: Context + 'static> Sync for RenderObj<C> {}
unsafe impl<C: Context + 'static> Send for RenderObj<C> {}

impl<C: Context + 'static> RenderObjs<C> {
    pub fn insert(&mut self, value: RenderObj<C>, notify: Option<NotifyImpl>) -> usize {
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

    pub unsafe fn get_unchecked_write<'a>(&'a mut self, id: usize, notify: &'a NotifyImpl) -> Write<RenderObj<C>>{
        Write::new(id, self.0.get_unchecked_mut(id), &notify)
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

pub struct ClipUbo<C: Context + 'static + Sync + Send>(pub Share<Uniforms<C>>);
pub struct ViewUbo<C: Context + 'static + Sync + Send>(pub Share<Uniforms<C>>);
pub struct ProjectionUbo<C: Context + 'static + Sync + Send>(pub Share<Uniforms<C>>);
pub struct RenderBegin(pub Share<RenderBeginDesc>);

unsafe impl<C: Context + 'static + Sync + Send> Sync for ClipUbo<C> {}
unsafe impl<C: Context + 'static + Sync + Send> Send for ClipUbo<C> {}

unsafe impl<C: Context + 'static + Sync + Send> Sync for ViewUbo<C> {}
unsafe impl<C: Context + 'static + Sync + Send> Send for ViewUbo<C> {}

unsafe impl<C: Context + 'static + Sync + Send> Sync for ProjectionUbo<C> {}
unsafe impl<C: Context + 'static + Sync + Send> Send for ProjectionUbo<C> {}

unsafe impl Sync for RenderBegin {}
unsafe impl Send for RenderBegin {}

pub struct DefaultTable(FxHashMap32<TypeId, Box<dyn Any>>);

impl DefaultTable {
    pub fn new() -> Self{
        Self(FxHashMap32::default())
    }

    pub fn set<T: 'static + Any + Sync + Send>(&mut self, value: T){
        self.0.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: 'static + Any + Sync + Send>(&self) -> Option<&T>{
        match self.0.get(&TypeId::of::<T>()) {
            Some(r) => r.downcast_ref::<T>(),
            None => None
        }
    }

    pub fn get_mut<T: 'static + Any + Sync + Send>(&mut self) -> Option<&mut T>{
        match self.0.get_mut(&TypeId::of::<T>()) {
            Some(r) => r.downcast_mut::<T>(),
            None => None
        }
    }

    pub fn get_unchecked<T: 'static + Any + Sync + Send>(&self) -> &T{
        self.0.get(&TypeId::of::<T>()).unwrap().downcast_ref::<T>().unwrap()
    }

    pub fn get_unchecked_mut<T: 'static + Any + Sync + Send>(&mut self) -> &mut T{
        self.0.get_mut(&TypeId::of::<T>()).unwrap().downcast_mut::<T>().unwrap()
    }

    pub fn delete<T: 'static + Any + Sync + Send>(&mut self){
        self.0.remove(&TypeId::of::<T>());
    }
}

unsafe impl Sync for DefaultTable {}
unsafe impl Send for DefaultTable {}
