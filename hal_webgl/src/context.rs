use atom::{Atom};
use slab::{Slab};
use share::{Share};
use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::{Object};
use stdweb::unstable::TryInto;

use hal_core::*;

use buffer::{WebGLBufferImpl};
use geometry::{WebGLGeometryImpl};
use program::{WebGLProgramImpl};
use render_target::{WebGLRenderBufferImpl, WebGLRenderTargetImpl};
use sampler::{WebGLSamplerImpl};
use state::{WebGLBlendStateImpl, WebGLDepthStateImpl, WebGLStencilStateImpl, WebGLRasterStateImpl};
use texture::{WebGLTextureImpl};
use util::{convert_to_mut};
use shader_cache::{ShaderCache};
use extension::*;

pub struct WebglHalContext {
    pub default_rt: HalRenderTarget,

    // 具体实现
    pub gl: WebGLRenderingContext,
    pub caps: Capabilities,
    pub vao_extension: Option<Object>,
    pub shader_cache: ShaderCache,

    // u32代表该槽分配的次数
    pub buffer_slab: Slab<(WebGLBufferImpl, u32)>,
    pub geometry_slab: Slab<(WebGLGeometryImpl, u32)>,
    pub texture_slab: Slab<(WebGLTextureImpl, u32)>,
    pub sampler_slab: Slab<(WebGLSamplerImpl, u32)>,
    pub rt_slab: Slab<(WebGLRenderTargetImpl, u32)>,
    pub rb_slab: Slab<(WebGLRenderBufferImpl, u32)>,
    pub bs_slab: Slab<(WebGLBlendStateImpl, u32)>,
    pub ds_slab: Slab<(WebGLDepthStateImpl, u32)>,
    pub rs_slab: Slab<(WebGLRasterStateImpl, u32)>,
    pub ss_slab: Slab<(WebGLStencilStateImpl, u32)>,
    pub program_slab: Slab<(WebGLProgramImpl, u32)>,
}

impl WebglHalContext {
    pub fn new(gl: WebGLRenderingContext, fbo: Option<Object>) -> WebglHalContext {
        let buffer_slab = Slab::new();
        let geometry_slab = Slab::new();
        let texture_slab = Slab::new();
        let sampler_slab = Slab::new();
        let mut rt_slab = Slab::new();
        let rb_slab = Slab::new();
        let bs_slab = Slab::new();
        let ds_slab = Slab::new();
        let rs_slab = Slab::new();
        let ss_slab = Slab::new();
        let program_slab = Slab::new();
        
        let caps = WebglHalContext::create_caps(&gl);
        let vao_extension = if caps.vertex_array_object {
            match TryInto::<Object>::try_into(js! {
                var extension = @{gl.as_ref()}.getExtension("OES_vertex_array_object");
                if (!extension) { return; }
                var vaoExtensionWrap = {
                    wrap: extension
                };
                return vaoExtensionWrap;
            }) {
                Ok(object) => Some(object),
                Err(_) => None,
            }
        } else {
            None
        };

        let default_rt = WebGLRenderTargetImpl::new_default(fbo, 0, 0);
        let default_rt = create_new_slot(&mut rt_slab, default_rt);
        let default_rt = HalRenderTarget(default_rt.0, default_rt.1);

        let shader_cache = ShaderCache::new();
        
        let context = WebglHalContext {
            default_rt: default_rt,
            gl: gl,
            caps: caps,
            vao_extension,
            shader_cache,

            buffer_slab,
            geometry_slab,
            texture_slab,
            sampler_slab,
            rt_slab,
            rb_slab,
            bs_slab,
            ds_slab,
            rs_slab,
            ss_slab,
            program_slab,
        };

        context
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

impl HalContext for WebglHalContext {

    // ==================== HalBuffer
    
    fn buffer_create(&self, btype: BufferType, count: usize, data: Option<BufferData>, is_updatable: bool) -> Result<HalBuffer, String> {
        match WebGLBufferImpl::new(&self.gl, btype, count, data, is_updatable) {
            Err(s) => Err(s),
            Ok(buffer) => {
                let slab = convert_to_mut(&self.buffer_slab);
                let (index, count) = create_new_slot(slab, buffer);
                Ok(HalBuffer(index, count))
            }
        }
    }

    fn buffer_destroy(&self, buffer: &HalBuffer) {
        if get_ref(&self.buffer_slab, buffer.0, buffer.1).is_some() {
            let slab = convert_to_mut(&self.buffer_slab);
            let rimpl = slab.remove(buffer.0 as usize);
            rimpl.0.delete(&self.gl);
        }
    }

    fn buffer_update(&self, buffer: &HalBuffer, offset: usize, data: BufferData) {
        let context = convert_to_mut(self);
        match get_mut_ref(&mut context.buffer_slab, buffer.0, buffer.1) {
            None => {},
            Some(buffer) => buffer.update(&self.gl, offset, data),
        }
    }
    
    // ==================== HalGeometry

    fn geometry_create(&self) -> Result<HalGeometry, String> {
        match WebGLGeometryImpl::new(&self.gl, &self.vao_extension) {
            Err(s) => Err(s),
            Ok(geometry) => {
                let slab = convert_to_mut(&self.geometry_slab);
                let (index, count) = create_new_slot(slab, geometry);
                Ok(HalGeometry(index, count))
            }
        }
    }

    fn geometry_destroy(&self, geometry: &HalGeometry) {
        if get_ref(&self.geometry_slab, geometry.0, geometry.1).is_some() {
            let slab = convert_to_mut(&self.geometry_slab);
            let rimpl = slab.remove(geometry.0 as usize);
            rimpl.0.delete(&self.vao_extension);
        }
    }

    fn geometry_get_vertex_count(&self, geometry: &HalGeometry) -> u32 {
        match get_ref(&self.geometry_slab, geometry.0, geometry.1) {
            None => 0,
            Some(geometry) => geometry.get_vertex_count(),
        }
    }

    fn geometry_set_vertex_count(&self, geometry: &HalGeometry, count: u32) {
        let slab = convert_to_mut(&self.geometry_slab);
        match get_mut_ref(slab, geometry.0, geometry.1) {
            None => {},
            Some(geometry) => geometry.set_vertex_count(count),
        }
    }

    fn geometry_set_attribute(&self, geometry: &HalGeometry, name: &AttributeName, buffer: &HalBuffer, item_count: usize) -> Result<(), String> {
        let gslab = convert_to_mut(&self.geometry_slab);
        let bslab = convert_to_mut(&self.buffer_slab);
        match (get_mut_ref(gslab, geometry.0, geometry.1), get_mut_ref(bslab, buffer.0, buffer.1)) {
            (Some(g), Some(b)) => g.set_attribute(&self.gl, &self.vao_extension, name, b, buffer, item_count),
            _ => Err("not found".to_string()),
        }
    }

    fn geometry_set_attribute_with_offset(&self, geometry: &HalGeometry, name: &AttributeName, buffer: &HalBuffer, item_count: usize, offset: usize, count: usize, stride: usize) -> Result<(), String> {
        let gslab = convert_to_mut(&self.geometry_slab);
        let bslab = convert_to_mut(&self.buffer_slab);
        match (get_mut_ref(gslab, geometry.0, geometry.1), get_mut_ref(bslab, buffer.0, buffer.1)) {
            (Some(g), Some(b)) => g.set_attribute_with_offset(&self.gl, &self.vao_extension, name, b, buffer, item_count, offset, count, stride),
            _ => Err("not found".to_string()),
        }
    }
      
    fn geometry_remove_attribute(&self, geometry: &HalGeometry, name: &AttributeName) {
        let slab = convert_to_mut(&self.geometry_slab);
        match get_mut_ref(slab, geometry.0, geometry.1) {
            Some(g) => g.remove_attribute(&self.gl, &self.vao_extension, name),
            _ => {},
        }
    }

    fn geometry_set_indices_short(&self, geometry: &HalGeometry, buffer: &HalBuffer) -> Result<(), String> {
        let gslab = convert_to_mut(&self.geometry_slab);
        let bslab = convert_to_mut(&self.buffer_slab);
        match (get_mut_ref(gslab, geometry.0, geometry.1), get_mut_ref(bslab, buffer.0, buffer.1)) {
            (Some(g), Some(b)) => g.set_indices_short(&self.gl, &self.vao_extension, b, buffer),
            _ => Err("not found".to_string()),
        }
    }
    
    fn geometry_set_indices_short_with_offset(&self, geometry: &HalGeometry, buffer: &HalBuffer, offset: usize, count: usize) -> Result<(), String> {
        let gslab = convert_to_mut(&self.geometry_slab);
        let bslab = convert_to_mut(&self.buffer_slab);
        match (get_mut_ref(gslab, geometry.0, geometry.1), get_mut_ref(bslab, buffer.0, buffer.1)) {
            (Some(g), Some(b)) => g.set_indices_short_with_offset(&self.gl, &self.vao_extension, b, buffer, offset, count),
            _ => Err("not found".to_string()),
        }
    }

    fn geometry_remove_indices(&self, geometry: &HalGeometry) {
        let slab = convert_to_mut(&self.geometry_slab);
        match get_mut_ref(slab, geometry.0, geometry.1) {
            Some(g) => g.remove_indices(&self.gl, &self.vao_extension),
            _ => {},
        }
    }

    // ==================== HalProgram

    fn program_create_with_vs_fs(&self, vs_id: u64, fs_id: u64, vs_name: &str, vs_defines: &[Option<&str>], fs_name: &str, fs_defines: &[Option<&str>], uniform_layout: &UniformLayout) -> Result<HalProgram, String> {

        let vs_name = Atom::from(vs_name);
        let fs_name = Atom::from(fs_name);

        let shader_cache = convert_to_mut(&self.shader_cache);
        match WebGLProgramImpl::new_with_vs_fs(&self.gl, &self.caps, shader_cache, vs_id, fs_id, &vs_name, vs_defines, &fs_name, fs_defines, uniform_layout) {
            Err(s) => Err(s),
            Ok(program) => {
                let slab = convert_to_mut(&self.program_slab);
                let (index, count) = create_new_slot(slab, program);
                Ok(HalProgram(index, count))
            }
        }
    }

    fn program_destroy(&self, program: &HalProgram) {
        if get_ref(&self.program_slab, program.0, program.1).is_some() {
            let slab = convert_to_mut(&self.program_slab);
            let rimpl = slab.remove(program.0 as usize);
            rimpl.0.delete(&self.gl);
        }
    }


    // ==================== HalRenderTarget

    fn rt_create(&self, w: u32, h: u32, pformat: PixelFormat, dformat: DataFormat, has_depth: bool) -> Result<HalRenderTarget, String> {

        let texture_wrap = self.texture_create_2d(0, w, h, pformat, dformat, false, None);
        if let Err(e) = &texture_wrap {
            return Err(e.clone());
        }
        let texture_wrap = texture_wrap.unwrap();

        let rb_wrap = if has_depth {
            let rbimpl = self.rb_create(w, h, PixelFormat::DEPTH16);
            if let Err(e) = &rbimpl {
                return Err(e.clone());
            }
            Some(rbimpl.unwrap())
        } else {
            None
        };

        let texture = get_ref(&self.texture_slab, texture_wrap.0, texture_wrap.1).unwrap();

        let rb = if has_depth {
            let r = rb_wrap.as_ref().unwrap();
            Some(get_ref(&self.rb_slab, r.0, r.1).unwrap())
        } else {
            None
        };
        
        match WebGLRenderTargetImpl::new(&self.gl, w, h, texture, rb, &texture_wrap, rb_wrap.as_ref()) {
            Err(s) => Err(s),
            Ok(rt) => {
                let slab = convert_to_mut(&self.rt_slab);
                let (index, count) = create_new_slot(slab, rt);
                Ok(HalRenderTarget(index, count))
            }
        }
    }
    
    fn rt_destroy(&self, rt: &HalRenderTarget) {
        if get_ref(&self.rt_slab, rt.0, rt.1).is_some() {
            let slab = convert_to_mut(&self.rt_slab);
            let rimpl = slab.remove(rt.0 as usize);
            let rimpl = rimpl.0;
            rimpl.delete(&self.gl);
            
            if let Some(t) = &rimpl.color {
                self.texture_destroy(t);
            }
            if let Some(rb) = &rimpl.depth {
                self.rb_destroy(rb);
            }
        }
    }

    fn rt_get_size(&self, rt: &HalRenderTarget) -> Option<(u32, u32)> {
        match get_ref(&self.rt_slab, rt.0, rt.1) {
            Some(rt) => Some(rt.get_size()),
            _ => None,
        }
    }

    fn rt_get_color_texture(&self, rt: &HalRenderTarget, index: u32) -> Option<HalTexture> {
        match get_ref(&self.rt_slab, rt.0, rt.1) {
            Some(rt) => rt.get_color_texture(),
            _ => None,
        }
    }

    // ==================== HalRenderBuffer

    fn rb_create(&self, w: u32, h: u32, pformat: PixelFormat) -> Result<HalRenderBuffer, String> {
        match WebGLRenderBufferImpl::new(&self.gl, w, h, pformat) {
            Err(s) => Err(s),
            Ok(rb) => {
                let slab = convert_to_mut(&self.rb_slab);
                let (index, count) = create_new_slot(slab, rb);
                Ok(HalRenderBuffer(index, count))
            }
        }
    }
    
    fn rb_destroy(&self, rb: &HalRenderBuffer) {
        if get_ref(&self.rb_slab, rb.0, rb.1).is_some() {
            let slab = convert_to_mut(&self.rb_slab);
            let rimpl = slab.remove(rb.0 as usize);
            rimpl.0.delete(&self.gl);
        }
    }

    fn rb_get_size(&self, rb: &HalRenderBuffer) -> Option<(u32, u32)> {
        match get_ref(&self.rb_slab, rb.0, rb.1) {
            Some(rb) => Some(rb.get_size()),
            _ => None,
        }
    }


    // ==================== HalTexture

    fn texture_create_2d(&self, mipmap_level: u32, width: u32, height: u32, pformat: PixelFormat, dformat: DataFormat, is_gen_mipmap: bool, data: Option<TextureData>) -> Result<HalTexture, String> {
        match WebGLTextureImpl::new_2d(&self.gl, mipmap_level, width, height, pformat, dformat, is_gen_mipmap, data) {
            Err(s) => Err(s),
            Ok(texture) => {
                let slab = convert_to_mut(&self.texture_slab);
                let (index, count) = create_new_slot(slab, texture);
                Ok(HalTexture(index, count))
            }
        }
    }

    fn texture_destroy(&self, texture: &HalTexture) {
        if get_ref(&self.texture_slab, texture.0, texture.1).is_some() {
            let slab = convert_to_mut(&self.texture_slab);
            let rimpl = slab.remove(texture.0 as usize);
            rimpl.0.delete(&self.gl);
        }
    }

    fn texture_get_size(&self, texture: &HalTexture) -> Option<(u32, u32)> {
        match get_ref(&self.texture_slab, texture.0, texture.1) {
            Some(tex) => Some(tex.get_size()),
            _ => None,
        }
    }

    fn texture_get_render_format(&self, texture: &HalTexture) -> Option<PixelFormat> {
        match get_ref(&self.texture_slab, texture.0, texture.1) {
            Some(tex) => Some(tex.get_render_format()),
            _ => None,
        }
    }

    fn texture_is_gen_mipmap(&self, texture: &HalTexture) -> bool {
        match get_ref(&self.texture_slab, texture.0, texture.1) {
            Some(tex) => tex.is_gen_mipmap(),
            _ => false,
        }
    }

    fn texture_update(&self, texture: &HalTexture, mipmap_level: u32, data: &TextureData) {
        let slab = convert_to_mut(&self.texture_slab);
        match get_mut_ref(slab, texture.0, texture.1) {
            Some(t) => t.update(&self.gl, mipmap_level, data),
            _ => {},
        }
    }

    // ==================== HalSampler

    fn sampler_create(&self, desc: &SamplerDesc) -> Result<HalSampler, String> {
        let sampler = WebGLSamplerImpl::new(desc);
        let slab = convert_to_mut(&self.sampler_slab);
        let (index, count) = create_new_slot(slab, sampler);
        Ok(HalSampler(index, count))
    }

    fn sampler_destroy(&self, sampler: &HalSampler) {
        if get_ref(&self.sampler_slab, sampler.0, sampler.1).is_some() {
            let slab = convert_to_mut(&self.sampler_slab);
            slab.remove(sampler.0 as usize);
        }
    }

    fn sampler_get_desc(&self, sampler: &HalSampler) -> Option<&SamplerDesc> {
        match get_ref(&self.sampler_slab, sampler.0, sampler.1) {
            Some(s) => Some(&s.0),
            _ => None,
        }
    }

    // ==================== HalRasterState

    fn rs_create(&self, desc: &RasterStateDesc) -> Result<HalRasterState, String> {
        let state = WebGLRasterStateImpl::new(desc);
        let slab = convert_to_mut(&self.rs_slab);
        let (index, count) = create_new_slot(slab, state);
        Ok(HalRasterState(index, count))
    }
    
    fn rs_destroy(&self, state: &HalRasterState) {
        if get_ref(&self.rs_slab, state.0, state.1).is_some() {
            let slab = convert_to_mut(&self.rs_slab);
            slab.remove(state.0 as usize);
        }
    }

    fn rs_get_desc(&self, state: &HalRasterState) -> Option<&RasterStateDesc> {
        match get_ref(&self.rs_slab, state.0, state.1) {
            Some(state) => Some(&state.0),
            _ => None,
        }
    }

    // ==================== HalDepthState

    fn ds_create(&self, desc: &DepthStateDesc) -> Result<HalDepthState, String> {
        let state = WebGLDepthStateImpl::new(desc);
        let slab = convert_to_mut(&self.ds_slab);
        let (index, count) = create_new_slot(slab, state);
        Ok(HalDepthState(index, count))
    }
    
    fn ds_destroy(&self, state: &HalDepthState) {
        if get_ref(&self.ds_slab, state.0, state.1).is_some() {
            let slab = convert_to_mut(&self.ds_slab);
            slab.remove(state.0 as usize);
        }
    }

    fn ds_get_desc(&self, state: &HalDepthState) -> Option<&DepthStateDesc> {
        match get_ref(&self.ds_slab, state.0, state.1) {
            Some(state) => Some(&state.0),
            _ => None,
        }
    }

    // ==================== HalStencilState

    fn ss_create(&self, desc: &StencilStateDesc) -> Result<HalStencilState, String> {
        let state = WebGLStencilStateImpl::new(desc);
        let slab = convert_to_mut(&self.ss_slab);
        let (index, count) = create_new_slot(slab, state);
        Ok(HalStencilState(index, count))
    }
    
    fn ss_destroy(&self, state: &HalStencilState) {
        if get_ref(&self.ss_slab, state.0, state.1).is_some() {
            let slab = convert_to_mut(&self.ss_slab);
            slab.remove(state.0 as usize);
        }
    }

    fn ss_get_desc(&self, state: &HalStencilState) -> Option<&StencilStateDesc> {
        match get_ref(&self.ss_slab, state.0, state.1) {
            Some(state) => Some(&state.0),
            _ => None,
        }
    }

    // ==================== HalBlendState
    
    fn bs_create(&self, desc: &BlendStateDesc) -> Result<HalBlendState, String> {
        let state = WebGLBlendStateImpl::new(desc);
        let slab = convert_to_mut(&self.bs_slab);
        let (index, count) = create_new_slot(slab, state);
        Ok(HalBlendState(index, count))
    }
    
    fn bs_destroy(&self, state: &HalBlendState) {
        if get_ref(&self.bs_slab, state.0, state.1).is_some() {
            let slab = convert_to_mut(&self.bs_slab);
            slab.remove(state.0 as usize);
        }
    }

    fn bs_get_desc(&self, state: &HalBlendState) -> Option<&BlendStateDesc> {
        match get_ref(&self.bs_slab, state.0, state.1) {
            Some(state) => Some(&state.0),
            _ => None,
        }
    }

    // ==================== 上下文相关

    fn render_get_caps(&self) -> &Capabilities {
        &self.caps
    }

    fn render_get_default_target(&self) -> &HalRenderTarget {
        &self.default_rt
    }

    fn render_set_shader_code<C: AsRef<str>>(&self, name: &str, code: &C) {
        let cache = convert_to_mut(&self.shader_cache);
        cache.set_shader_code(name, code)
    }

    // TODO
    fn render_begin(&self, render_target: &HalRenderTarget, data: &RenderBeginDesc) {

    }
    
    // TODO
    fn render_end(&self) {

    }

    // TODO
    fn render_set_program(&self, program: &HalProgram) {

    }

    // TODO
    fn render_set_state(&self, bs: &HalBlendState, ds: &HalDepthState, rs: &HalRasterState, ss: &HalStencilState) {

    }

    // TODO
    fn render_draw(&self, geometry: &HalGeometry, parameter: &Share<dyn ProgramParamter>) {

    }
}

fn create_new_slot<T>(slab: &mut Slab<(T, u32)>, obj: T) -> (u32, u32) {
    let (key, v, is_first) = slab.alloc_with_is_first();
    if is_first {
        v.1 = 0;
    }
    
    v.0 = obj;
    v.1 += 1;

    (key as u32, v.1 as u32)
}

fn get_mut_ref<T>(slab: &mut Slab<(T, u32)>, key: u32, count: u32) -> Option<&mut T> {
    let key = key as usize;
    match slab.get_mut(key) {
        Some(v) => {
            if v.1 == count {
                Some(&mut v.0)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_ref<T>(slab: &Slab<(T, u32)>, key: u32, count: u32) -> Option<&T> {
    let key = key as usize;
    match slab.get(key) {
        Some(v) if v.1 == count => Some(&v.0),
        _ => None,
    }
}