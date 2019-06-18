use std::sync::{Arc};

use atom::{Atom};
use webgl_rendering_context::{WebGLRenderingContext};

use stdweb::{Object};
use stdweb::unstable::TryInto;
use extension::*;
use fnv::FnvHashMap;

use hal_core::*;

use state::{State};
use geometry::{WebGLGeometryImpl};
use render_target::{WebGLRenderTargetImpl, WebGLRenderBufferImpl};
use texture::{WebGLTextureImpl};
use sampler::{WebGLSamplerImpl};
use shader::{ProgramManager};
use debug_info::*;

pub struct WebGLContextImpl {
    gl: Arc<WebGLRenderingContext>,
    caps: Arc<Capabilities>,
    default_rt: Arc<WebGLRenderTargetImpl>,
    state: State,
    vao_extension: Option<Arc<Object>>,
    program_mgr: ProgramManager,
}

unsafe impl Sync for WebGLContextImpl{}
unsafe impl Send for WebGLContextImpl{}

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

    fn set_shader_code<C: AsRef<str>>(&mut self, name: &Atom, code: &C) {
        self.program_mgr.set_shader_code(name, code);
    }

    fn compile_shader(&mut self, shader_type: ShaderType, name: &Atom, defines: &[Atom]) -> Result<u64, String> {
        self.program_mgr.compile_shader(shader_type, name, defines) 
    }

    fn create_uniforms(&mut self) -> Uniforms<Self::ContextSelf> {
        Uniforms::<Self::ContextSelf> {
            dirty_count: 0,
            values: FnvHashMap::default(),
			has_texture: false,
        }
    }

    fn create_pipeline(&mut self, vs_hash: u64, fs_hash: u64, rs: Arc<dyn AsRef<RasterState>>, bs: Arc<dyn AsRef<BlendState>>, ss: Arc<dyn AsRef<StencilState>>, ds: Arc<dyn AsRef<DepthState>>) -> Result<Pipeline, String> {
        
        // 链接Shader成Program
        if let Err(s) = self.program_mgr.get_program(vs_hash, fs_hash) {
            return Err(s);
        }
        
        Ok(Pipeline {
            vs_hash: vs_hash, 
            fs_hash: fs_hash,
            raster_state: rs,
            depth_state: ds,
            stencil_state: ss,
            blend_state: bs,
        })
    }

    fn create_geometry(&self) -> Result<Self::ContextGeometry, String> {
        let vao_extension = match &self.vao_extension {
            None => None,
            Some(extension) => Some(extension.clone()),
        };
        Ok(WebGLGeometryImpl::new(&self.gl, vao_extension))
    }

    fn create_texture_2d(&mut self, w: u32, h: u32, level: u32, pformat: &PixelFormat, dformat: &DataFormat, is_gen_mipmap: bool, data: &TextureData) -> Result<Self::ContextTexture, String> {
        WebGLTextureImpl::new_2d(&self.gl, w, h, level, pformat, dformat, is_gen_mipmap, data)
    }

    fn create_sampler(&mut self, desc: Arc<dyn AsRef<SamplerDesc>>) -> Result<Self::ContextSampler, String> {
        let desc = desc.as_ref().as_ref();
        Ok(WebGLSamplerImpl {
            min_filter: desc.min_filter,
            mag_filter: desc.mag_filter,
            mip_filter: desc.mip_filter,
            u_wrap: desc.u_wrap,
            v_wrap: desc.v_wrap,
        })
    }

    fn create_render_target(&mut self, w: u32, h: u32, pformat: &PixelFormat, dformat: &DataFormat, has_depth: bool) -> Result<Self::ContextRenderTarget, String> {
        WebGLRenderTargetImpl::new(&self.gl, w, h, pformat, dformat, has_depth)
    }

    fn restore_state(&mut self) {
        let p = self.state.pipeline.as_ref().as_ref();
        if let Ok(program) = self.program_mgr.get_program(p.vs_hash, p.fs_hash) {
            program.use_me();
        }
        State::apply_all_state(&self.gl, &mut self.state);
    }

    fn begin_render(&mut self, render_target: &Arc<dyn AsRef<Self::ContextRenderTarget>>, data: &Arc<dyn AsRef<RenderBeginDesc>>) {

        // 注：暂时在这里重置所有状态
        self.restore_state();

        self.state.set_render_target(render_target);
        let data = data.as_ref().as_ref();
        self.state.set_viewport(&data.viewport);
        self.state.set_clear(&data.clear_color, &data.clear_depth, &data.clear_stencil);
    }

    /** 
     * 对别的平台，如果RenderTarget是屏幕，就要调用swapBuffer，但是webgl不需要。
     * 注：在这里，要解除vao的绑定，否则下面更新的buffer会绑到最后一个vao上。
     */
    fn end_render(&mut self) {
        if let Some(vao_extension) = &self.vao_extension {
            let extension = vao_extension.as_ref();
            js! {
                @{&extension}.wrap.bindVertexArrayOES(null);
            }
        }
    }

    fn set_pipeline(&mut self, pipeline: &Arc<dyn AsRef<Pipeline>>) {
        
        if !self.state.set_pipeline(pipeline) {
            let p = pipeline.as_ref().as_ref();
            if let Ok(program) = self.program_mgr.get_program(p.vs_hash, p.fs_hash) {
                program.use_me();
            } else {
                debug_println!("Context set_pipeline error, no program = ({:?}, {:?})", p.vs_hash, p.fs_hash);
            }
        }
    }

    fn draw(&mut self, geometry: &Arc<dyn AsRef<Self::ContextGeometry>>, values: &FnvHashMap<Atom, Arc<dyn AsRef<Uniforms<Self::ContextSelf>>>>) {
        if let Ok(program) = self.state.get_current_program(&mut self.program_mgr) {
            program.set_uniforms(&mut self.state, values);
            self.state.draw(geometry);
        }
    }
}

impl WebGLContextImpl {

    /** 
     * 注：fbo是WebGLFramebuffer对象，但是WebGLFramebuffer在小游戏真机上不是真正的Object对象，所以要封装成：{wrap: WebGLFramebuffer}
     */
    pub fn new(gl: Arc<WebGLRenderingContext>, fbo: Option<Object>) -> Self {
        
        let caps = Self::create_caps(gl.as_ref());
        let rt = Arc::new(WebGLRenderTargetImpl::new_default(&gl, fbo, 0, 0));

        let state = State::new(&gl, &(rt.clone() as Arc<dyn AsRef<WebGLRenderTargetImpl>>), caps.max_vertex_attribs, caps.max_textures_image_units);

        let mgr = ProgramManager::new(&gl, caps.max_vertex_attribs);

        let vao_extension = if caps.vertex_array_object {
            match TryInto::<Object>::try_into(js! {
                var extension = @{gl.as_ref()}.getExtension("OES_vertex_array_object");
                if (!extension) { return; }
                var vaoExtensionWrap = {
                    wrap: extension
                };
                return vaoExtensionWrap;
            }) {
                Ok(object) => Some(Arc::new(object)),
                Err(_) => None,
            }
        } else {
            None
        };
        
        println!("~~~~~~~~~~~~~ WebGLRenderingContext, All extensions: {:?}", gl.get_supported_extensions());
        println!("~~~~~~~~~~~~~ WebGLRenderingContext, caps {:?}", &caps);

        WebGLContextImpl {
            gl: gl,
            caps: Arc::new(caps),
            default_rt: rt,
            state: state,
            vao_extension: vao_extension,
            program_mgr: mgr,
        }
    }


    /** 
     * 注：data是Image或者是Canvas对象，但是那两个在小游戏真机上不是真正的Object对象，所以要封装成：{wrap: Image | Canvas}
     */
    pub fn create_texture_2d_webgl(&self, w: u32, h: u32, level: u32, pformat: &PixelFormat, dformat: &DataFormat, is_gen_mipmap: bool, data: &Object) -> Result<WebGLTextureImpl, String> {
        WebGLTextureImpl::new_2d_webgl(&self.gl, w, h, level, pformat, dformat, is_gen_mipmap, data)
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