use slab::{Slab};
use share::{Share};
use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::{Object};

use hal_core::{Context, ProgramParamter, Capabilities, RenderBeginDesc};
use wrap::buffer::{WebGLBufferWrap};
use wrap::geometry::{WebGLGeometryWrap};
use wrap::program::{WebGLProgramWrap};
use wrap::render_target::{WebGLRenderTargetWrap, WebGLRenderBufferWrap};
use wrap::sampler::{WebGLSamplerWrap};
use wrap::state::{WebGLRasterStateWrap, WebGLDepthStateWrap, WebGLStencilStateWrap, WebGLBlendStateWrap};
use wrap::texture::{WebGLTextureWrap};

use wrap::gl_slab::{GLSlot};
use implement::{
    WebGLBufferImpl, 
    WebGLContextImpl,
    WebGLGeometryImpl, 
    WebGLProgramImpl,
    WebGLRenderTargetImpl, WebGLRenderBufferImpl,
    WebGLSamplerImpl,
    WebGLRasterStateImpl, WebGLDepthStateImpl, WebGLStencilStateImpl, WebGLBlendStateImpl,
    WebGLTextureImpl,
};

#[derive(Clone)]
pub struct WebGLContextWrap {
    pub rimpl: Share<WebGLContextImpl>,
    pub default_rt: WebGLRenderTargetWrap,

    pub buffer: Share<Slab<WebGLBufferImpl>>,
    pub geometry: Share<Slab<WebGLGeometryImpl>>,
    pub texture: Share<Slab<WebGLTextureImpl>>,
    pub sampler: Share<Slab<WebGLSamplerImpl>>,
    pub render_target: Share<Slab<WebGLRenderTargetImpl>>,
    pub render_buffer: Share<Slab<WebGLRenderBufferImpl>>,
    pub blend_state: Share<Slab<WebGLBlendStateImpl>>,
    pub depth_state: Share<Slab<WebGLDepthStateImpl>>,
    pub raster_state: Share<Slab<WebGLRasterStateImpl>>,
    pub stencil_state: Share<Slab<WebGLStencilStateImpl>>,
    pub program: Share<Slab<WebGLProgramImpl>>,
}

impl Context for WebGLContextWrap {
    type ContextSelf = WebGLContextWrap;
    
    type ContextBuffer = WebGLBufferWrap;
    type ContextGeometry = WebGLGeometryWrap;
    type ContextTexture = WebGLTextureWrap;
    type ContextSampler = WebGLSamplerWrap;
    type ContextRenderTarget = WebGLRenderTargetWrap;
    type ContextRenderBuffer = WebGLRenderBufferWrap;
    type ContextBlendState = WebGLBlendStateWrap;
    type ContextDepthState = WebGLDepthStateWrap;
    type ContextRasterState = WebGLRasterStateWrap;
    type ContextStencilState = WebGLStencilStateWrap;
    type ContextProgram = WebGLProgramWrap;

    fn get_caps(&self) -> &Capabilities {
        &self.rimpl.caps
    }

    fn get_default_target(&self) -> &Self::ContextRenderTarget {
        &self.default_rt
    }

    fn set_shader_code<C: AsRef<str>>(&self, name: &str, code: &C) {

    }

    fn restore_state(&self) {

    }

    fn begin_render(&self, render_target: &Self::ContextRenderTarget, data: &RenderBeginDesc) {

    }

    fn end_render(&self) {

    }

    fn set_program(&self, program: &Self::ContextProgram) {

    }

    fn set_state(&self, bs: &Self::ContextBlendState, ds: &Self::ContextDepthState, rs: &Self::ContextRasterState, ss: &Self::ContextStencilState) {

    }

    fn draw(&self, geometry: &Self::ContextGeometry, parameter: &Share<ProgramParamter<Self::ContextSelf>>) {

    }
}

impl WebGLContextWrap {
    pub fn new(context: WebGLRenderingContext, fbo: Option<Object>) -> Self {
        let buffer = Share::new(Slab::new());
        let geometry = Share::new(Slab::new());
        let texture = Share::new(Slab::new());
        let sampler = Share::new(Slab::new());
        let render_target = Share::new(Slab::new());
        let render_buffer = Share::new(Slab::new());
        let blend_state = Share::new(Slab::new());
        let depth_state = Share::new(Slab::new());
        let raster_state = Share::new(Slab::new());
        let stencil_state = Share::new(Slab::new());
        let program = Share::new(Slab::new());
        
        let rimpl = Share::new(WebGLContextImpl::new(context));
        
        let default_rt = WebGLRenderTargetImpl::new_default(&rimpl, fbo, 0, 0);
        let default_rt = GLSlot::new(&render_target, default_rt);
        let default_rt = WebGLRenderTargetWrap::new(default_rt);
        
        Self {
            
            rimpl: rimpl,
            default_rt: default_rt,

            buffer: buffer,
            geometry: geometry,
            texture: texture,
            sampler: sampler,
            render_buffer: render_buffer,
            render_target: render_target,
            blend_state: blend_state,
            depth_state: depth_state,
            raster_state: raster_state,
            stencil_state: stencil_state,
            program: program,
        }
    }
}