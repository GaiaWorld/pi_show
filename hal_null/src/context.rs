use share::Share;

use atom::{Atom};

use hal_core::*;

use geometry::{NullGeometryImpl};
use render_target::{NullRenderTargetImpl, NullRenderBufferImpl};
use texture::{NullTextureImpl};
use sampler::{NullSamplerImpl};

pub struct NullContextImpl {
    caps: Share<Capabilities>,
    default_rt: Share<NullRenderTargetImpl>,
}

impl NullContextImpl {
    pub fn new() -> Self {
        NullContextImpl {
            caps: Share::new(Capabilities::new()),

            default_rt: Share::new(NullRenderTargetImpl {

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
    
    fn get_caps(&self) -> Share<Capabilities> {
        self.caps.clone()
    }

    fn get_default_render_target(&self) -> Share<Self::ContextRenderTarget> {
        self.default_rt.clone()
    }

    fn set_shader_code<C: AsRef<str>>(&mut self, _name: &Atom, _code: &C) {

    }

    fn restore_state(&mut self){

    }

    fn compile_shader(&mut self, _shader_type: ShaderType, _name: &Atom, _defines: &[Atom]) -> Result<u64, String> {
        Ok(0)
    }

    fn create_uniforms(&mut self) -> Uniforms<Self::ContextSelf> {
        Uniforms::<Self::ContextSelf> {
            values: fx_hashmap::FxHashMap32::default(),
            dirty_count: 0,
            has_texture: false,
        }
    }

    fn create_pipeline(&mut self, _vs_hash: u64, _fs_hash: u64, _rs: Share<AsRef<RasterState>>, _bs: Share<AsRef<BlendState>>, _ss: Share<AsRef<StencilState>>, _ds: Share<AsRef<DepthState>>) -> Result<Pipeline, String> {
        Ok(Pipeline::new())
    }

    fn create_geometry(&self) -> Result<Self::ContextGeometry, String> {
        Ok(NullGeometryImpl {

        })
    }

    fn create_texture_2d(&mut self, _w: u32, _h: u32, _level: u32, _pformat: &PixelFormat, _dformat: &DataFormat, _is_gen_mipmap: bool, _data: &TextureData) -> Result<Self::ContextTexture, String> {
        Ok(NullTextureImpl {
            
        })
    }

    fn create_sampler(&mut self, _desc: Share<AsRef<SamplerDesc>>) -> Result<Self::ContextSampler, String> {
        Ok(NullSamplerImpl {

        })
    }

    fn create_render_target(&mut self, _w: u32, _h: u32, _pformat: &PixelFormat, _dformat: &DataFormat, _has_depth: bool) -> Result<Self::ContextRenderTarget, String> {
        Ok(NullRenderTargetImpl {

        })
    }

    fn begin_render(&mut self, _render_target: &Share<AsRef<Self::ContextRenderTarget>>, _data: &Share<AsRef<RenderBeginDesc>>) {
        
    }

    fn end_render(&mut self) {

    }

    fn set_pipeline(&mut self, _pipeline: &Share<AsRef<Pipeline>>) {

    }

    fn draw(&mut self, _geometry: &Share<AsRef<Self::ContextGeometry>>, _values: &fx_hashmap::FxHashMap32<Atom, Share<AsRef<Uniforms<Self::ContextSelf>>>>) {

    }
}