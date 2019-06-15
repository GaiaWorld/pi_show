use std::sync::{Arc};
use slab::{Slab};
use wrap::context::{WebGLContextWrap};

pub enum GLSlabType {
    Buffer,
    GeometryWrap,
    TextureWrap,
    SamplerWrap,
    RenderTargetWrap,
    RenderBufferWrap,
    BlendStateWrap,
    DepthStateWrap,
    RasterStateWrap,
    StencilState,
    Program,
}

pub struct GLSlab {
    // usize: 该slot复用的次数。
    slab_buffer: Slab<(WebGLBufferWrap, usize)>,
    slab_geometry: Slab<(WebGLGeometryWrap, usize)>,
    slab_texture: Slab<(WebGLTextureWrap, usize)>,
    slab_sampler: Slab<(WebGLSamplerWrap, usize)>,
    slab_render_target: Slab<(WebGLRenderTargetWrap, usize)>,
    slab_render_buffer: Slab<(WebGLRenderBufferWrap, usize)>,
    slab_blend_state: Slab<(WebGLBlendStateWrap, usize)>,
    slab_depth_state: Slab<(WebGLDepthStateWrap, usize)>,
    slab_raster_state: Slab<(WebGLRasterStateWrap, usize)>,
    slab_stencil_state: Slab<(WebGLStencilStateWrap, usize)>,
    slab_program: Slab<(WebGLProgramWrap, usize)>,
}

/**
 * GL上下文相关的Slab槽
 */
pub struct GLSlot {
    context: Arc<WebGLContextWrap>,
    
    slab_index: usize,    // 槽的索引
    current_count: usize, // 当前复用的次数
    
    // id，唯一的标志，高32位是current_count, 低32位是slab_index
    id: u64,
}

impl GLSlot {

    pub fn new(context: &Arc<WebGLContextWrap>) -> Self {
        Self {
            context: context.clone(),
            slab_index: 0,
            current_count: 0,
            id: 0,
        }
    }
}


