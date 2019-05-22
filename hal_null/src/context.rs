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

impl NullContextImpl {
    pub fn new() -> Self {
        NullContextImpl {
            caps: Arc::new(Capabilities::new()),

            default_rt: Arc::new(NullRenderTargetImpl {

            })
        } 
    }
}

impl Context for NullContextImpl {
    type ContextSelf = NullContextImpl;
    type ContextGeometry = NullGeometryImpl;
    type ContextTexture = NullTextureImpl;
    type ContextSampler = NullSamplerImpl;
    type ContextRenderTarget = NullRenderTargetImpl;
    type ContextRenderBuffer = NullRenderBufferImpl;
    
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

    fn create_uniforms(&mut self) -> Uniforms<Self::ContextSelf> {
        Uniforms::<Self::ContextSelf> {
            values: HashMap::new(),
        }
    }

    fn create_pipeline(&mut self, _vs_hash: u64, _fs_hash: u64, _rs: Arc<AsRef<RasterState>>, _bs: Arc<AsRef<BlendState>>, _ss: Arc<AsRef<StencilState>>, _ds: Arc<AsRef<DepthState>>) -> Result<Pipeline, String> {
        Ok(Pipeline::new())
    }

    fn create_geometry(&self) -> Result<Self::ContextGeometry, String> {
        Ok(NullGeometryImpl {

        })
    }

    fn create_texture_2d(&mut self, _width: u32, _height: u32, _pformat: PixelFormat, _dformat: DataFormat, _is_gen_mipmap: bool, _data: Option<&[u8]>) -> Result<Self::ContextTexture, String> {
        Ok(NullTextureImpl {

        })
    }

    fn create_texture_2d_with_canvas(&mut self, _width: u32, _height: u32, _pformat: PixelFormat, _dformat: DataFormat, _is_gen_mipmap: bool, _canvas: *const isize) -> Result<Self::ContextTexture, String> {
        Ok(NullTextureImpl {

        })
    }

    fn create_sampler(&mut self, _desc: Arc<AsRef<SamplerDesc>>) -> Result<Self::ContextSampler, String> {
        Ok(NullSamplerImpl {

        })
    }

    fn create_render_target(&mut self) -> Result<Self::ContextRenderTarget, String> {
        Ok(NullRenderTargetImpl {

        })
    }

    fn create_render_buffer(&mut self, _w: u32, _h: u32, _format: PixelFormat) -> Result<Self::ContextRenderBuffer, String> {
        Ok(NullRenderBufferImpl {
            
        })
    }
 
    fn begin_render(&mut self, _render_target: &Arc<AsRef<Self::ContextRenderTarget>>, _data: &Arc<AsRef<RenderBeginDesc>>) {
        
    }

    fn end_render(&mut self) {

    }

    fn set_pipeline(&mut self, _pipeline: &Arc<AsRef<Pipeline>>) {

    }

    fn draw(&mut self, _geometry: &Arc<AsRef<Self::ContextGeometry>>, _values: &HashMap<Atom, Arc<AsRef<Uniforms<Self::ContextSelf>>>>) {

    }
}