use std::sync::{Arc, Weak};
use slab::{Slab};
use wrap::context::{WebGLContextWrap};
use wrap::buffer::{WebGLBufferWrap};
use wrap::geometry::{WebGLGeometryWrap};
use wrap::program::{WebGLProgramWrap};
use wrap::render_target::{WebGLRenderTargetWrap, WebGLRenderBufferWrap};
use wrap::sampler::{WebGLSamplerWrap};
use wrap::state::{WebGLRasterStateWrap, WebGLDepthStateWrap, WebGLStencilStateWrap, WebGLBlendStateWrap};
use wrap::texture::{WebGLTextureWrap};

/**
 * Slab槽
 */
#[derive(Clone)]
pub struct GLSlot {
    context: Weak<WebGLContextWrap>,
    
    slab_index: usize,    // 槽的索引
    current_count: usize, // 当前复用的次数

    // id，唯一的标志，高32位是current_count, 低32位是slab_index
    id: u64,
}


pub struct GLSlab {
    // usize: 该slot复用的次数。
    slab_buffer: Slab<(WebGLBufferWrap, usize)>,
    slab_geometry: Slab<(WebGLGeometryWrap, usize)>,
    slab_texture: Slab<(WebGLTextureWrap, usize)>,
    slab_sampler: Slab<(WebGLSamplerWrap, usize)>,
    slab_render_target: Slab<(WebGLRenderTargetWrap, usize)>,
    slab_render_buffer: Slab<(WebGLRenderBufferWrap, usize)>,
    slab_blend_state: Slab<(WebGLBlendStateWrap, usize)>,
    slab_depth_state: Slab<(WebGLDepthStateWrap, usize)>,
    slab_raster_state: Slab<(WebGLRasterStateWrap, usize)>,
    slab_stencil_state: Slab<(WebGLStencilStateWrap, usize)>,
    slab_program: Slab<(WebGLProgramWrap, usize)>,
}

impl GLSlot {

    pub fn new(context: &Weak<WebGLContextWrap>, index: usize, count: usize) -> Self {
        let id = count << 32 | index;
        Self {
            context: context.clone(),
            slab_index: index,
            current_count: count,
            id: id as u64,
        }
    }
}

impl GLSlab {

    pub fn new() -> Self {
        Self {
            slab_buffer: Slab::new(),
            slab_geometry: Slab::new(),
            slab_texture: Slab::new(),
            slab_sampler: Slab::new(),
            slab_render_target: Slab::new(),
            slab_render_buffer: Slab::new(),
            slab_blend_state: Slab::new(),
            slab_depth_state: Slab::new(),
            slab_raster_state: Slab::new(),
            slab_stencil_state: Slab::new(),
            slab_program: Slab::new(),
        }
    }

    pub fn get_buffer_slice(&mut self, index: usize, count: usize) -> Option<&mut WebGLBufferWrap> {
        match self.slab_buffer.get_mut(index) {
            None => None,
            Some((r, c)) => {
                debug_assert!(*c == count, "c != count");
                Some(r)
            }
        }
    }
    
    pub fn get_geometry_slice(&mut self, index: usize, count: usize) -> Option<&mut WebGLGeometryWrap> {
        match self.slab_geometry.get_mut(index) {
            None => None,
            Some((r, c)) => {
                debug_assert!(*c == count, "c != count");
                Some(r)
            }
        }
    }
    
    pub fn get_texture_slice(&mut self, index: usize, count: usize) -> Option<&mut WebGLTextureWrap> {
        match self.slab_texture.get_mut(index) {
            None => None,
            Some((r, c)) => {
                debug_assert!(*c == count, "c != count");
                Some(r)
            }
        }
    }
    
    pub fn get_sampler_slice(&mut self, index: usize, count: usize) -> Option<&mut WebGLSamplerWrap> {
        match self.slab_sampler.get_mut(index) {
            None => None,
            Some((r, c)) => {
                debug_assert!(*c == count, "c != count");
                Some(r)
            }
        }
    }
    
    pub fn get_render_target_slice(&mut self, index: usize, count: usize) -> Option<&mut WebGLRenderTargetWrap> {
        match self.slab_render_target.get_mut(index) {
            None => None,
            Some((r, c)) => {
                debug_assert!(*c == count, "c != count");
                Some(r)
            }
        }
    }
    
    pub fn get_render_buffer_slice(&mut self, index: usize, count: usize) -> Option<&mut WebGLRenderBufferWrap> {
        match self.slab_render_buffer.get_mut(index) {
            None => None,
            Some((r, c)) => {
                debug_assert!(*c == count, "c != count");
                Some(r)
            }
        }
    }
    
    pub fn get_blend_state_slice(&mut self, index: usize, count: usize) -> Option<&mut WebGLBlendStateWrap> {
        match self.slab_blend_state.get_mut(index) {
            None => None,
            Some((r, c)) => {
                debug_assert!(*c == count, "c != count");
                Some(r)
            }
        }
    }
    
    pub fn get_depth_state_slice(&mut self, index: usize, count: usize) -> Option<&mut WebGLDepthStateWrap> {
        match self.slab_depth_state.get_mut(index) {
            None => None,
            Some((r, c)) => {
                debug_assert!(*c == count, "c != count");
                Some(r)
            }
        }
    }
    
    pub fn get_raster_state_slice(&mut self, index: usize, count: usize) -> Option<&mut WebGLRasterStateWrap> {
        match self.slab_raster_state.get_mut(index) {
            None => None,
            Some((r, c)) => {
                debug_assert!(*c == count, "c != count");
                Some(r)
            }
        }
    }
    
    pub fn get_stencil_state_slice(&mut self, index: usize, count: usize) -> Option<&mut WebGLStencilStateWrap> {
        match self.slab_stencil_state.get_mut(index) {
            None => None,
            Some((r, c)) => {
                debug_assert!(*c == count, "c != count");
                Some(r)
            }
        }
    }
    
    pub fn get_program_slice(&mut self, index: usize, count: usize) -> Option<&mut WebGLProgramWrap> {
        match self.slab_program.get_mut(index) {
            None => None,
            Some((r, c)) => {
                debug_assert!(*c == count, "c != count");
                Some(r)
            }
        }
    }
}