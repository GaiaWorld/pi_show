use std::sync::{Arc};
use std::collections::{HashMap};

use atom::{Atom};

use hal_core::*;

use geometry::{NullGeometryImpl};
use render_target::{NullRenderBufferImpl, NullRenderTargetImpl};
use texture::{NullTextureImpl};
use sampler::{NullSamplerImpl};

pub struct NullContextImpl {
    caps: Arc<Capabilities>,
    default_rt: Arc<NullRenderTargetImpl>,
}

pub struct NullSystemContext {

}

impl Context for NullContextImpl {
    type SystemContext = NullSystemContext;

    type ContextGeometry = NullGeometryImpl;
    type ContextTexture = NullTextureImpl;
    type ContextSampler = NullSamplerImpl;
    type ContextRenderTarget = NullRenderTargetImpl;
    type ContextRenderBuffer = NullRenderBufferImpl;

    fn new(_rimpl: Option<Arc<Self::SystemContext>>, _width: u32, _height: u32) -> Self {
        NullContextImpl {
            caps: Arc::new(Capabilities::new()),

            default_rt: Arc::new(NullRenderTargetImpl {

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

    fn create_geometry(&self) -> Result<Arc<Self::ContextGeometry>, String> {
        Ok(Arc::new(NullGeometryImpl {

        }))
    }

    fn create_texture_2d(&mut self, _width: u32, _height: u32, _pformat: PixelFormat, _dformat: DataFormat, _is_gen_mipmap: bool, _data: Option<&[u8]>) -> Result<Arc<Self::ContextTexture>, String> {
        Ok(Arc::new(NullTextureImpl {

        }))
    }

    fn create_texture_2d_with_canvas(&mut self, _width: u32, _height: u32, _pformat: PixelFormat, _dformat: DataFormat, _is_gen_mipmap: bool, _canvas: *const isize) -> Result<Arc<Self::ContextTexture>, String> {
        Ok(Arc::new(NullTextureImpl {

        }))
    }

    fn create_sampler(&mut self, _texture: Arc<Self::ContextTexture>, _desc: Arc<SamplerDesc>) -> Result<Arc<Self::ContextSampler>, String> {
        Ok(Arc::new(NullSamplerImpl {

        }))
    }

    fn create_render_target(&mut self) -> Result<Arc<Self::ContextRenderTarget>, String> {
        Ok(Arc::new(NullRenderTargetImpl {

        }))
    }

    fn create_render_buffer(&mut self, _w: u32, _h: u32, _format: PixelFormat) -> Result<Arc<Self::ContextRenderBuffer>, String> {
        Ok(Arc::new(NullRenderBufferImpl {
            
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