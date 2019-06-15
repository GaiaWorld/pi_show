use atom::Atom;

use std::sync::{Arc, Weak};
use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::{Object};

use fnv::FnvHashMap;

use hal_core::{Context, Uniforms, Capabilities, RenderBeginDesc};
use wrap::buffer::{WebGLBufferWrap};
use wrap::geometry::{WebGLGeometryWrap};
use wrap::program::{WebGLProgramWrap};
use wrap::render_target::{WebGLRenderTargetWrap, WebGLRenderBufferWrap};
use wrap::sampler::{WebGLSamplerWrap};
use wrap::state::{WebGLRasterStateWrap, WebGLDepthStateWrap, WebGLStencilStateWrap, WebGLBlendStateWrap};
use wrap::texture::{WebGLTextureWrap};
use wrap::gl_slab::{GLSlab, GLSlot};

pub struct WebGLContextWrapImpl {
    caps: Capabilities,
    default_rt: WebGLRenderTargetWrap,
    slab: GLSlab,
}

#[derive(Clone)]
pub struct WebGLContextWrap(Arc<WebGLContextWrapImpl>);

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
        &self.0.caps
    }

    fn get_default_target(&self) -> &Self::ContextRenderTarget {
        &self.0.default_rt
    }

    fn set_shader_code<C: AsRef<str>>(&self, name: &Atom, code: &C) {

    }

    fn restore_state(&mut self) {

    }

    fn begin_render(&self, render_target: &Self::ContextRenderTarget, data: &RenderBeginDesc) {

    }

    fn end_render(&self) {

    }

    fn set_program(&self, program: &Self::ContextProgram) {

    }

    fn set_state(&self, bs: &Self::ContextBlendState, ds: &Self::ContextDepthState, rs: &Self::ContextRasterState, ss: &Self::ContextStencilState) {

    }

    fn draw(&self, geometry: &Self::ContextGeometry, values: &FnvHashMap<Atom, Uniforms>, samplers: &FnvHashMap<Atom, (Self::ContextSampler, Self::ContextTexture)>) {

    }
}

impl WebGLContextWrap {
    pub fn new(context: Arc<WebGLRenderingContext>, fbo: Option<Object>) -> Self {
        let slab = GLSlab::new();

        let default_rt = WebGLRenderTargetWrap::new(GLSlot::new(&Weak::new(), 0, 0));

        Self(Arc::new(WebGLContextWrapImpl {
            caps: Capabilities::new(),
            default_rt: default_rt,
            slab: slab,    
        }))
    }
}