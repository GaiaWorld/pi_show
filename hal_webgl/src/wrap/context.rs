use atom::Atom;
use share::Share;
use std::rc::{Rc};
use std::cell::{RefCell};
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

pub struct WebGLContextWrapImpl {
    pub caps: Capabilities,
    pub default_rt: Option<WebGLRenderTargetWrap>,
    pub slab: GLSlab,
}

#[derive(Clone)]
pub struct WebGLContextWrap(pub Rc<RefCell<WebGLContextWrapImpl>>);

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
        &self.0.try_borrow().unwrap().caps
    }

    fn get_default_target(&self) -> &Self::ContextRenderTarget {
        &self.0.try_borrow().unwrap().default_rt.as_ref().unwrap()
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

    fn draw(&self, geometry: &Self::ContextGeometry, parameter: &Share<ProgramParamter<Self::ContextSelf>>) {

    }
}

impl WebGLContextWrap {
    pub fn new(context: WebGLRenderingContext, fbo: Option<Object>) -> Self {
        let slab = GLSlab::new();

        Self(Rc::new(RefCell::new(WebGLContextWrapImpl {
            caps: Capabilities::new(),
            default_rt: None,
            slab: slab,    
        })))
    }
}