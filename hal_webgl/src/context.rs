use slab::{Slab};
use share::{Share};
use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::{Object};

use hal_core::*;

use buffer::{WebGLBufferImpl};
use geometry::{WebGLGeometryImpl};
use program::{WebGLProgramImpl};
use render_target::{WebGLRenderBufferImpl, WebGLRenderTargetImpl};
use sampler::{WebGLSamplerImpl};
use state::{WebGLBlendStateImpl, WebGLDepthStateImpl, WebGLStencilStateImpl, WebGLRasterStateImpl};
use texture::{WebGLTextureImpl};
use util::{convert_to_mut};

pub struct WebglHalContext {
    pub default_rt: HalRenderTarget,

    // 

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

impl HalContext {
    pub fn new(gl: WebGLRenderingContext, fbo: Option<Object>) -> HalContext {
        
    }
}

impl HalContext for WebglHalContext {

    // ==================== HalBuffer
    
    fn buffer_create(&self, btype: BufferType, count: usize, data: Option<BufferData>, is_updatable: bool) -> Result<HalBuffer, String> {
        match WebGLBufferImpl::new(&context.rimpl, btype, count, data, is_updatable) {
            Err(s) => Err(s),
            Ok(buffer) => Ok(HalBuffer(create_new_slot(self.buffer_slab, buffer)),
        }
    }

    fn buffer_destroy(&self, buffer: &HalBuffer) {
        let context = convert_to_mut(self);
        let rimpl = context.buffer_slab.remove(buffer.0);
        rimpl.0.delete(&mut self.rimpl);
    }

    fn buffer_update(&self, buffer: &HalBuffer, offset: usize, data: BufferData) {
        let context = convert_to_mut(self);
        match get_mut(&mut context.buffer_slab, buffer.0, buffer.1) {
            None => {},
            Some(buffer) => buffer.update(&mut self.rimpl, offset, data),
        }
    }
    
    // ==================== HalGeometry

    fn geometry_create(&self) -> Result<HalGeometry, String> {
        match WebGLGeometryImpl::new(&context.rimpl) {
            Err(s) => Err(s),
            Ok(geometry) => Ok(HalGeometry(create_new_slot(self.geometry_slab, geometry)),
        }
    }

    fn geometry_destroy(&self, geometry: &HalGeometry) {
        let context = convert_to_mut(self);
        let rimpl = context.geometry_slab.remove(geometry.0);
        rimpl.0.delete(&mut self.rimpl);
    }

    fn geometry_get_vertex_count(&self, geometry: &HalGeometry) -> u32 {
        let context = convert_to_mut(self);
        match get_mut(&mut context.geometry_slab, geometry.0, geometry.1) {
            None => 0,
            Some(geometry) => geometry.get_vertex_count(),
        }
    }

    fn geometry_set_vertex_count(&self, geometry: &HalGeometry, count: u32) {
        let slab = convert_to_mut(&self.geometry_slab);
        match get_mut(slab, geometry.0, geometry.1) {
            None => {},
            Some(geometry) => geometry.set_vertex_count(count),
        }
    }

    fn geometry_set_attribute(&self, geometry: &HalGeometry, name: &AttributeName, buffer: &HalBuffer, item_count: usize) -> Result<(), String> {
        let slab = convert_to_mut(&self.geometry_slab);
        match get_mut(slab, geometry.0, geometry.1) {
            None => Err("not found".to_string()),
            Some(geometry) => geometry.set_attribute(&mut self.rimpl, name, buffer, item_count)
        }
    }

    fn geometry_set_attribute_with_offset(&self, geometry: &HalGeometry, name: &AttributeName, buffer: &HalBuffer, item_count: usize, offset: usize, count: usize, stride: usize) -> Result<(), String> {
        let slab = convert_to_mut(&self.geometry_slab);
        match get_mut(slab, geometry.0, geometry.1) {
            None => Err("not found".to_string()),
            Some(geometry) => geometry.set_attribute_with_offset(&mut self.rimpl, name, buffer, item_count, offset, count, stride)
        }
    }
      
    fn geometry_remove_attribute(&self, geometry: &HalGeometry, name: &AttributeName) {
        let slab = convert_to_mut(&self.geometry_slab);
        match get_mut(slab, geometry.0, geometry.1) {
            None => Err("not found".to_string()),
            Some(geometry) => geometry.remove_attribute(&mut self.rimpl, name),
        }
    }

    fn geometry_set_indices_short(&self, geometry: &HalGeometry, buffer: &HalBuffer) -> Result<(), String> {
        let slab = convert_to_mut(&self.geometry_slab);
        match get_mut(slab, geometry.0, geometry.1) {
            None => Err("not found".to_string()),
            Some(geometry) => geometry.set_indices_short(&mut self.rimpl, buffer),
        }
    }
    
    fn geometry_set_indices_short_with_offset(&self, geometry: &HalGeometry, buffer: &HalBuffer, offset: usize, count: usize) -> Result<(), String>;

    fn geometry_remove_indices(&self, geometry: &HalGeometry);



    // ==================== HalProgram

    fn program_create_with_vs_fs(&self, vs_name: &Atom, vs_defines: &[Atom], fs_name: &Atom, fs_defines: &[Atom], uniform_layout: &UniformLayout) -> Result<HalProgram, String>;

    fn program_destroy(&self, program: &HalProgram);

    fn program_get_shader_info(&self, program: &HalProgram, stype: ShaderType) -> Option<(&Atom, &[Atom])>;


    // ==================== HalRenderTarget

    fn rt_create(&self, w: u32, h: u32, pformat: PixelFormat, dformat: DataFormat, has_depth: bool) -> Result<HalRenderTarget, String>;
    
    fn rt_destroy(&self, rt: &HalRenderTarget);

    fn rt_get_size(&self, rt: &HalRenderTarget) -> Option<(u32, u32)>;

    fn rt_get_color_texture(&self, rt: &HalRenderTarget, index: u32) -> Option<HalTexture>;

    // ==================== HalRenderBuffer

    fn rb_create(&self, w: u32, h: u32, pformat: PixelFormat) -> Result<HalRenderBuffer, String>;
    
    fn rb_destroy(&self, rb: &HalRenderBuffer);

    fn rb_get_size(&self, rb: &HalRenderBuffer) -> Option<(u32, u32)>;


    // ==================== HalTexture

    fn texture_create_2d(&self, mipmap_level: u32, width: u32, height: u32, pformat: PixelFormat, dformat: DataFormat, is_gen_mipmap: bool, data: Option<TextureData>) -> Result<HalTexture, String>;

    fn texture_destroy(&self, texture: &HalTexture);

    fn texture_get_size(&self, texture: &HalTexture) -> Option<(u32, u32)>;

    fn texture_get_render_format(&self, texture: &HalTexture) -> Option<PixelFormat>;

    fn texture_is_gen_mipmap(&self, texture: &HalTexture) -> bool;

    fn texture_update(&self, texture: &HalTexture, mipmap_level: u32, data: &TextureData);

    // ==================== HalSampler

    fn sampler_create(&self, desc: &SamplerDesc) -> Result<HalSampler, String>;

    fn sampler_destroy(&self, sampler: &HalSampler);

    fn sampler_get_desc(&self, sampler: &HalSampler) -> &SamplerDesc;

    // ==================== HalRasterState

    fn rs_create(&self, desc: &RasterStateDesc) -> Result<HalRasterState, String>;
    
    fn rs_destroy(&self, state: &HalRasterState);

    fn rs_get_desc(&self, state: &HalRasterState) -> &RasterStateDesc;

    // ==================== HalDepthState

    fn ds_create(&self, desc: &DepthStateDesc) -> Result<HalDepthState, String>;
    
    fn ds_destroy(&self, state: &HalDepthState);

    fn ds_get_desc(&self, state: &HalDepthState) -> &DepthStateDesc;

    // ==================== HalStencilState

    fn ss_create(&self, desc: &StencilStateDesc) -> Result<HalStencilState, String>;
    
    fn ss_destroy(&self, state: &HalStencilState);

    fn ss_get_desc(&self, state: &HalStencilState) -> &StencilStateDesc;

    // ==================== HalBlendState
    
    fn bs_create(&self, desc: &BlendStateDesc) -> Result<HalBlendState, String>;
    
    fn bs_destroy(&self, state: &HalBlendState);

    fn bs_get_desc(&self, state: &HalBlendState) -> &BlendStateDesc;

    // ==================== 上下文相关

    fn render_get_caps(&self) -> &Capabilities;

    fn render_get_default_target(&self) -> &HalRenderTarget;

    fn render_set_shader_code<C: AsRef<str>>(&self, name: &str, code: &C);

    fn render_restore_state(&self);

    fn render_begin(&self, render_target: &HalRenderTarget, data: &RenderBeginDesc);

    fn render_end(&self);

    fn render_set_program(&self, program: &HalProgram);

    fn render_set_state(&self, bs: &HalBlendState, ds: &HalDepthState, rs: &HalRasterState, ss: &HalStencilState);

    fn render_draw(&self, geometry: &HalGeometry, parameter: &Share<dyn ProgramParamter>);
}

fn create_new_slot<T>(slab: &Slab<T>, obj: T) -> (u32, u32) {
    let (key, v, is_first) = slab.alloc_with_is_first();
    if is_first {
        v.1 = 0;
    }
    
    v.0 = obj;
    v.1 += 1;

    (key as u32, v.1 as u32)
}

fn get_mut<T>(slab: &mut Slab<T>, key: u32, count: u32) -> Option<&mut T> {
    match slab.get_mut(key) {
        Some(v) if v.1 == count => Some(&v.0),
        _ => None,
    }
}