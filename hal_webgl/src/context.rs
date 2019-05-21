use std::sync::{Arc};
use std::collections::{HashMap};

use atom::{Atom};

use hal_core::*;

use geometry::{WebGLGeometryImpl};
use render_target::{WebGLRenderBufferImpl, WebGLRenderTargetImpl};
use texture::{WebGLTextureImpl};
use sampler::{WebGLSamplerImpl};

pub struct WebGLContextImpl {
    caps: Arc<Capabilities>,
    default_rt: Arc<WebGLRenderTargetImpl>,
}

impl Context for WebGLContextImpl {
    type ContextGeometry = WebGLGeometryImpl;
    type ContextTexture = WebGLTextureImpl;
    type ContextSampler = WebGLSamplerImpl;
    type ContextRenderTarget = WebGLRenderTargetImpl;
    type ContextRenderBuffer = WebGLRenderBufferImpl;

    fn new(_rimpl: *const isize, _width: u32, _height: u32) -> Self {
        WebGLContextImpl {
            caps: Arc::new(Capabilities::new()),

            default_rt: Arc::new(WebGLRenderTargetImpl {

            })
        } 
    }

    fn get_caps(&self) -> Arc<Capabilities> {
        self.caps.clone()
    }

    fn get_default_render_target(&self) -> Arc<Self::ContextRenderTarget> {
        self.default_rt.clone()
    }

    fn set_shader_code<C: AsRef<str>>(&mut self, _name: &Atom, _code: &C) {

    }

    fn compile_shader(&mut self, _shader_type: ShaderType, _name: &Atom, _defines: &[Atom]) -> Result<u64, String> {
        Ok(0)
    }

    fn create_pipeline(&mut self, _vs_hash: u32, _fs_hash: u32, _rs: Arc<RasterState>, _bs: Arc<BlendState>, _ss: Arc<StencilState>, _ds: Arc<DepthState>) -> Result<Arc<Pipeline>, String> {
        Ok(Arc::new(Pipeline::new()))
    }

    fn create_geometry(&self, _vertex_count: u32) -> Result<Arc<Self::ContextGeometry>, String> {
        Ok(Arc::new(WebGLGeometryImpl {

        }))
    }

    fn create_texture_2d(&mut self, _width: u32, _height: u32, _pformat: PixelFormat, _dformat: DataFormat, _is_gen_mipmap: bool, _data: Option<&[u8]>) -> Result<Arc<Self::ContextTexture>, String> {
        Ok(Arc::new(WebGLTextureImpl {

        }))
    }

    fn create_texture_2d_with_canvas(&mut self, _width: u32, _height: u32, _pformat: PixelFormat, _dformat: DataFormat, _is_gen_mipmap: bool, _canvas: *const isize) -> Result<Arc<Self::ContextTexture>, String> {
        Ok(Arc::new(WebGLTextureImpl {

        }))
    }

    fn create_sampler(&mut self, _texture: Arc<Self::ContextTexture>, _desc: Arc<SamplerDesc>) -> Result<Arc<Self::ContextSampler>, String> {
        Ok(Arc::new(WebGLSamplerImpl {

        }))
    }

    fn create_render_target(&mut self) -> Result<Arc<Self::ContextRenderTarget>, String> {
        Ok(Arc::new(WebGLRenderTargetImpl {

        }))
    }

    fn create_render_buffer(&mut self, _w: u32, _h: u32, _format: PixelFormat) -> Result<Arc<Self::ContextRenderBuffer>, String> {
        Ok(Arc::new(WebGLRenderBufferImpl {
            
        }))
    }
 
    fn begin_render(&mut self, _render_target: &Arc<Self::ContextRenderTarget>, _data: &Arc<RenderBeginDesc>) {
        
    }

    fn end_render(&mut self) {

    }

    fn set_pipeline(&mut self, _pipeline: &Arc<Pipeline>) {

    }

    fn draw(&mut self, _geometry: &Arc<Self::ContextGeometry>, _values: &HashMap<Atom, Arc<Uniforms>>) {

    }
}