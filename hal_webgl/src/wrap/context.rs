use share::Share;
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
use wrap::gl_slab::{GLSlab};

#[derive(Clone)]
pub struct WebGLContextWrap {
    pub slabs: Share<GLSlab>,
    caps: Share<Capabilities>,
    default_rt: Option<Share<WebGLRenderTargetWrap>>,
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
        &self.caps
    }

    fn get_default_target(&self) -> &Self::ContextRenderTarget {
        self.default_rt.as_ref().unwrap()
    }

    fn set_shader_code<C: AsRef<str>>(&self, name: &str, code: &C) {

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

    fn draw(&self, geometry: &Self::ContextGeometry, parameter: &Share<ProgramParamter<Self::ContextSelf>>) {

    }
}

impl WebGLContextWrap {
    pub fn new(context: WebGLRenderingContext, fbo: Option<Object>) -> Self {
        Self {
            slabs: Share::new(GLSlab::new()),
            caps: Share::new(Capabilities::new()),
            default_rt: None,
        }
    }
}