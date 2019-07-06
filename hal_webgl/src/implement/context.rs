use share::{Share};
use hal_core::*;
use webgl_rendering_context::{WebGLRenderingContext};

use stdweb::{Object};
use stdweb::unstable::TryInto;

use wrap::{WebGLContextWrap, convert_to_mut};

use implement::extension::*;
use implement::render_target::{WebGLRenderTargetImpl};
use implement::program::{WebGLProgramImpl};
use implement::state::{WebGLBlendStateImpl, WebGLDepthStateImpl, WebGLRasterStateImpl, WebGLStencilStateImpl};
use implement::geometry::{WebGLGeometryImpl};
use implement::shader_cache::{ShaderCache};

pub struct WebGLContextImpl {
    pub caps: Capabilities,
    pub vao_extension: Option<Object>,
    pub context: WebGLRenderingContext,
    pub shader_cache: Option<ShaderCache>,
}

impl WebGLContextImpl {
    pub fn new(context: WebGLRenderingContext) -> Share<WebGLContextImpl> {
        let caps = Self::create_caps(&context);
        let context = Share::new(Self {
            caps: caps,
            vao_extension: None,
            context: context,
            shader_cache: None,
        });

        let shader_cache = ShaderCache::new(&context);

        let cc = convert_to_mut(context.as_ref());
        cc.shader_cache = Some(shader_cache);

        context
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

    pub fn draw(&mut self, geometry: &WebGLGeometryImpl, parameter: &Share<dyn ProgramParamter<WebGLContextWrap>>) {

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
        
        let mut astc = gl.get_extension::<CompressedTextureAstc>().map_or(false, |_v| true);
        if !astc {
            astc = gl.get_extension::<WebkitCompressedTextureAstc>().map_or(false, |_v| true);
        }

        let mut s3tc = gl.get_extension::<CompressedTextureS3tc>().map_or(false, |_v| true);
        if !s3tc {
            s3tc = gl.get_extension::<WebkitCompressedTextureS3tc>().map_or(false, |_v| true);
        }

        let mut pvrtc = gl.get_extension::<CompressedTexturePvrtc>().map_or(false, |_v| true);
        if !pvrtc {
            pvrtc = gl.get_extension::<WebkitCompressedTexturePvrtc>().map_or(false, |_v| true);
        }

        let mut etc1 = gl.get_extension::<CompressedTextureEtc1>().map_or(false, |_v| true);
        if !etc1 {
            etc1 = gl.get_extension::<WebkitCompressedTextureEtc1>().map_or(false, |_v| true);
        }

        let mut etc2 = gl.get_extension::<CompressedTextureEtc2>().map_or(false, |_v| true);
        if !etc2 {
            etc2 = gl.get_extension::<WebkitCompressedTextureEtc2>().map_or(false, |_v| true);
        }
        if !etc2 {
            etc2 = gl.get_extension::<CompressedTextureEs3>().map_or(false, |_v| true);
        }
        
        Capabilities {
            astc: astc,
            s3tc: s3tc,
            pvrtc: pvrtc,
            etc1: etc1,
            etc2: etc2,
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