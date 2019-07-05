use share::{Share};
use hal_core::{Capabilities, RenderBeginDesc, ProgramParamter};
use webgl_rendering_context::{WebGLRenderingContext};

use implement::render_target::{WebGLRenderTargetImpl};
use implement::program::{WebGLProgramImpl};
use implement::state::{WebGLBlendStateImpl, WebGLDepthStateImpl, WebGLRasterStateImpl, WebGLStencilStateImpl};
use implement::geometry::{WebGLGeometryImpl};

use stdweb::{Object};
use wrap::{WebGLContextWrap};

#[derive(Debug)]
pub struct WebGLContextImpl {
    pub caps: Capabilities,
    pub vao_extension: Option<Object>,
    pub context: WebGLRenderingContext,
}

impl WebGLContextImpl {
    pub fn new(context: WebGLRenderingContext) -> Self {
        let caps = Capabilities::new();
        Self {
            caps: caps,
            vao_extension: None,
            context: context,
        }
    }

    pub fn set_shader_code<C: AsRef<str>>(&mut self, name: &str, code: &C) {

    }

    pub fn restore_state(&mut self) {

    }

    pub fn begin_render(&mut self, render_target: &WebGLRenderTargetImpl, data: &RenderBeginDesc) {

    }

    pub fn end_render(&mut self) {

    }

    pub fn set_program(&mut self, program: &WebGLProgramImpl) {

    }

    pub fn set_state(&mut self, bs: &WebGLBlendStateImpl, ds: &WebGLDepthStateImpl, rs: &WebGLRasterStateImpl, ss: &WebGLStencilStateImpl) {

    }

    pub fn draw(&mut self, geometry: &WebGLGeometryImpl, parameter: &Share<ProgramParamter<WebGLContextWrap>>) {

    }
}