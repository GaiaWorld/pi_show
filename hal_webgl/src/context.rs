use std::sync::{Arc};
use std::collections::{HashMap};

use atom::{Atom};
use webgl_rendering_context::{WebGLRenderingContext};

use stdweb::unstable::TryInto;
use extension::*;

use hal_core::*;

use geometry::{WebGLGeometryImpl};
use render_target::{WebGLRenderBufferImpl, WebGLRenderTargetImpl};
use texture::{WebGLTextureImpl};
use sampler::{WebGLSamplerImpl};

pub struct WebGLContextImpl {
    gl: Arc<WebGLRenderingContext>,
    caps: Arc<Capabilities>,
    default_rt: Arc<WebGLRenderTargetImpl>,
}

impl Context for WebGLContextImpl {
    type ContextSelf = WebGLContextImpl;
    type ContextGeometry = WebGLGeometryImpl;
    type ContextTexture = WebGLTextureImpl;
    type ContextSampler = WebGLSamplerImpl;
    type ContextRenderTarget = WebGLRenderTargetImpl;
    type ContextRenderBuffer = WebGLRenderBufferImpl;

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
        Ok(WebGLGeometryImpl::new(&self.gl))
    }

    fn create_texture_2d(&mut self, _width: u32, _height: u32, _pformat: PixelFormat, _dformat: DataFormat, _is_gen_mipmap: bool, _data: Option<&[u8]>) -> Result<Self::ContextTexture, String> {
        Ok(WebGLTextureImpl {

        })
    }

    fn create_texture_2d_with_canvas(&mut self, _width: u32, _height: u32, _pformat: PixelFormat, _dformat: DataFormat, _is_gen_mipmap: bool, _canvas: *const isize) -> Result<Self::ContextTexture, String> {
        Ok(WebGLTextureImpl {

        })
    }

    fn create_sampler(&mut self, _desc: Arc<AsRef<SamplerDesc>>) -> Result<Self::ContextSampler, String> {
        Ok(WebGLSamplerImpl {

        })
    }

    fn create_render_target(&mut self) -> Result<Self::ContextRenderTarget, String> {
        Err("no impl".to_string())
    }

    fn create_render_buffer(&mut self, _w: u32, _h: u32, _format: PixelFormat) -> Result<Self::ContextRenderBuffer, String> {
        Ok(WebGLRenderBufferImpl {
            
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

impl WebGLContextImpl {

    pub fn new(gl: Arc<WebGLRenderingContext>) -> Self {
        
        let caps = Self::create_caps(gl.as_ref());
        let rt = WebGLRenderTargetImpl::new_default(&gl);

        WebGLContextImpl {
            gl: gl,
            caps: Arc::new(caps),
            default_rt: Arc::new(rt),
        }
    }

    fn create_caps(gl: &WebGLRenderingContext) -> Capabilities {
        
        let max_textures_image_units = gl.get_parameter(WebGLRenderingContext::MAX_TEXTURE_IMAGE_UNITS).try_into().unwrap();
        let max_vertex_texture_image_units= gl.get_parameter(WebGLRenderingContext::MAX_VERTEX_TEXTURE_IMAGE_UNITS).try_into().unwrap();
        let max_combined_textures_image_units = gl.get_parameter(WebGLRenderingContext::MAX_COMBINED_TEXTURE_IMAGE_UNITS).try_into().unwrap();
        let max_texture_size = gl.get_parameter(WebGLRenderingContext::MAX_TEXTURE_SIZE).try_into().unwrap();
        let max_render_texture_size = gl.get_parameter(WebGLRenderingContext::MAX_RENDERBUFFER_SIZE).try_into().unwrap();
        let max_vertex_attribs = gl.get_parameter(WebGLRenderingContext::MAX_VERTEX_ATTRIBS).try_into().unwrap();
        let max_varying_vectors = gl.get_parameter(WebGLRenderingContext::MAX_VARYING_VECTORS).try_into().unwrap();
        let max_vertex_uniform_vectors = gl.get_parameter(WebGLRenderingContext::MAX_VERTEX_UNIFORM_VECTORS).try_into().unwrap();
        let max_fragment_uniform_vectors = gl.get_parameter(WebGLRenderingContext::MAX_FRAGMENT_UNIFORM_VECTORS).try_into().unwrap();

        let standard_derivatives = gl.get_extension::<OESStandardDerivatives>().map_or(false, |_v| true);
        let uint_indices = gl.get_extension::<OESElementIndexUint>().map_or(false, |_v| true);

        let fragment_depth_supported = gl.get_extension::<EXTFragDepth>().map_or(false, |_v| true);

        let texture_float = gl.get_extension::<OESTextureFloat>().map_or(false, |_v| true);
        let texture_float_linear_filtering = texture_float && gl.get_extension::<OESTextureFloatLinear>().map_or(false, |_v| true);

        let texture_lod = gl.get_extension::<EXTShaderTextureLod>().map_or(false, |_v| true);
        let color_buffer_float = gl.get_extension::<WEBGLColorBufferFloat>().map_or(false, |_v| true);

        let depth_texture_extension = gl.get_extension::<WEBGLDepthTexture>().map_or(false, |_v| true);
        // depth_texture_extension.UNSIGNED_INT_24_8_WEBGL;
        
        let vertex_array_object = gl.get_extension::<OESVertexArrayObject>().map_or(false, |_v| true);
        let instanced_arrays = gl.get_extension::<ANGLEInstancedArrays>().map_or(false, |_v| true);
        
        Capabilities {
            max_textures_image_units: max_textures_image_units,
            max_vertex_texture_image_units: max_vertex_texture_image_units,
            max_combined_textures_image_units: max_combined_textures_image_units,
            max_texture_size: max_texture_size,
            max_render_texture_size: max_render_texture_size,
            max_vertex_attribs: max_vertex_attribs,
            max_varying_vectors: max_varying_vectors,
            max_vertex_uniform_vectors: max_vertex_uniform_vectors,
            max_fragment_uniform_vectors: max_fragment_uniform_vectors,
            standard_derivatives: standard_derivatives,
            uint_indices: uint_indices,
            fragment_depth_supported: fragment_depth_supported,
            texture_float: texture_float,
            texture_float_linear_filtering: texture_float_linear_filtering,
            texture_lod: texture_lod,
            color_buffer_float: color_buffer_float,
            depth_texture_extension: depth_texture_extension,
            vertex_array_object: vertex_array_object,
            instanced_arrays: instanced_arrays,
        }
    }
}