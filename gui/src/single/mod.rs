pub mod oct;

use std::sync::Arc;
use std::fmt;

use std::collections::HashMap;

use cgmath::Ortho;
use slab::Slab;
use atom::Atom;
use hal_core::{Context, Pipeline, Uniforms};
use ecs::{ Share, Write };
use ecs::monitor::NotifyImpl;

use component::user::{Point2, Matrix4};
use render::engine::{ PipelineInfo};

#[derive(Debug)]
pub struct OverflowClip{
    pub id_vec: [usize;8],
    pub clip: [[Point2;4];8],
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
pub struct RenderObj<C: Context>{
    pub depth: f32,
    pub visibility: bool,
    pub is_opacity: bool,
    pub ubos: HashMap<Atom, Arc<Uniforms<C>>>,
    pub geometry: Arc<<C as Context>::ContextGeometry>,
    pub pipeline: Arc<PipelineInfo>,
    pub context: usize,
    pub defines: Vec<Atom>,
    
    // pub shader_attr: Option<SharderAttr<C>>, 
}

// pub struct SharderAttr<C: Context>{
//     pub geometry: <C as Context>::ContextGeometry, //geometry 对象
//     pub pipeline: Arc<Pipeline>,
// }

// impl<C: Context + Debug> fmt::Debug for RenderObj<C> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // write!(f, "Point {{ x: {}, y: {} }}", self.depth, self.visibility)
//         // write!(f, "RenderObj {{ depth: {}, visibility: {}, is_opacity: {} }}", self.depth, self.visibility, self.is_opacity)
//         // write!(f, "RenderObj {{ depth: {}, visibility: {}, is_opacity: {}, geometry: {}, pipeline: {}, ubo: {} }}", self.depth, self.visibility, self.is_opacity, self.geometry, self.pipeline, self.ubo)
//     }
// }

#[derive(Deref, DerefMut)]
pub struct RenderObjs<C: Context>(pub Slab<RenderObj<C>>);

impl<C: Context> Default for RenderObjs<C> {
    fn default() -> Self {
        Self(Slab::default())
    }
}

unsafe impl<C: Context> Sync for RenderObj<C> {}
unsafe impl<C: Context> Send for RenderObj<C> {}

impl<C: Context> RenderObjs<C> {
    pub fn insert(&mut self, value: RenderObj<C>, notify: Option<NotifyImpl>) -> usize {
        let id = self.0.insert(value);
        match notify {
            Some(n) => n.modify_event(id, "", 0),
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

pub struct ClipUbo<C: Context + 'static + Sync + Send>(pub Arc<Uniforms<C>>);
pub struct ViewUbo<C: Context + 'static + Sync + Send>(pub Arc<Uniforms<C>>);
pub struct ProjectionUbo<C: Context + 'static + Sync + Send>(pub Arc<Uniforms<C>>);

unsafe impl<C: Context + 'static + Sync + Send> Sync for ClipUbo<C> {}
unsafe impl<C: Context + 'static + Sync + Send> Send for ClipUbo<C> {}

unsafe impl<C: Context + 'static + Sync + Send> Sync for ViewUbo<C> {}
unsafe impl<C: Context + 'static + Sync + Send> Send for ViewUbo<C> {}

unsafe impl<C: Context + 'static + Sync + Send> Sync for ProjectionUbo<C> {}
unsafe impl<C: Context + 'static + Sync + Send> Send for ProjectionUbo<C> {}