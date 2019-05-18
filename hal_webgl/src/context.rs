use std::rc::{Rc};
use atom::{Atom};

use hal_core::*;

use geometry::{WebGLGeometryImpl};
use render_target::{WebGLRenderBufferImpl, WebGLRenderTargetImpl};
use texture::{WebGLTextureImpl};
use sampler::{WebGLSamplerImpl};

pub struct WebGLContextImpl {
    caps: Rc<Capabilities>,
    default_rt: Rc<WebGLRenderTargetImpl>,
}

impl Context for WebGLContextImpl {
    type ContextGeometry = WebGLGeometryImpl;
    type ContextTexture = WebGLTextureImpl;
    type ContextSampler = WebGLSamplerImpl;
    type ContextRenderTarget = WebGLRenderTargetImpl;
    type ContextRenderBuffer = WebGLRenderBufferImpl;

    fn new(_rimpl: *const isize, _width: u32, _height: u32) -> Self {
        WebGLContextImpl {
            caps: Rc::new(Capabilities::new()),

            default_rt: Rc::new(WebGLRenderTargetImpl {

            })
        } 
    }

    fn get_caps(&self) -> Rc<Capabilities> {
        self.caps.clone()
    }

    fn get_default_render_target(&self) -> Rc<Self::ContextRenderTarget> {
        self.default_rt.clone()
    }

    fn set_shader_code<C: AsRef<str>>(&mut self, _name: &Atom, _code: &C) {

    }

    fn compile_shader(&mut self, _shader_type: ShaderType, _name: &Atom, _defines: &[Atom]) -> Result<u64, String> {
        Ok(0)
    }

    fn create_pipeline(&mut self, _vs_hash: u32, _fs_hash: u32, _rs: Rc<RasterState>, _bs: Rc<BlendState>, _ss: Rc<StencilState>, _ds: Rc<DepthState>) -> Result<Rc<Pipeline>, String> {
        Ok(Rc::new(Pipeline::new()))
    }

    fn create_geometry(&self, _vertex_count: u32) -> Result<Rc<Self::ContextGeometry>, String> {
        Ok(Rc::new(WebGLGeometryImpl {

        }))
    }

    fn create_texture_2d(&mut self, _width: u32, _height: u32, _pixel: PixelFormat, _data: DataFormat, _is_gen_mipmap: bool, _data: Option<&[u8]>) -> Result<Rc<Self::ContextTexture>, String> {
        Ok(Rc::new(WebGLTextureImpl {

        }))
    }

    fn create_texture_2d_with_canvas(&mut self, _width: u32, _height: u32, _pixel: PixelFormat, _data: DataFormat, _is_gen_mipmap: bool, _canvas: *const isize) -> Result<Rc<Self::ContextTexture>, String> {
        Ok(Rc::new(WebGLTextureImpl {

        }))
    }

    fn create_sampler(&mut self, _texture: Rc<Self::ContextTexture>, _desc: Rc<SamplerDesc>) -> Result<Rc<Self::ContextSampler>, String> {
        Ok(Rc::new(WebGLSamplerImpl {

        }))
    }

    fn create_render_target(&mut self) -> Result<Rc<Self::ContextRenderTarget>, String> {
        Ok(Rc::new(WebGLRenderTargetImpl {

        }))
    }

    fn create_render_buffer(&mut self, _w: u32, _h: u32, _format: PixelFormat) -> Result<Rc<Self::ContextRenderBuffer>, String> {
        Ok(Rc::new(WebGLRenderBufferImpl {
            
        }))
    }
 
    fn begin_render(&mut self, _render_target: Rc<Self::ContextRenderTarget>, _data: Rc<RenderBeginDesc>) {
        
    }

    fn end_render(&mut self) {

    }

    fn set_pipeline(&mut self, _pipeline: Rc<Pipeline>) {

    }

    fn draw(&mut self, _geometry: Rc<Self::ContextGeometry>, _values: &[Rc<Uniforms>]) {

    }
}