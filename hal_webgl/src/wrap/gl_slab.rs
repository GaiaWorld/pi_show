use slab::{Slab};

use wrap::context::{WebGLContextWrap};
use implement::{WebGLBufferImpl};
use implement::{WebGLGeometryImpl};
use implement::{WebGLProgramImpl};
use implement::{WebGLRenderTargetImpl, WebGLRenderBufferImpl};
use implement::{WebGLSamplerImpl};
use implement::{WebGLRasterStateImpl, WebGLDepthStateImpl, WebGLStencilStateImpl, WebGLBlendStateImpl};
use implement::{WebGLTextureImpl};

/**
 * 将不可变引用变为可变引用
 */
pub fn convert_to_mut<T>(obj: &T) -> &mut T {
    let mut_obj = obj as *const T as usize as *mut T;
	let mut_obj = unsafe { &mut *mut_obj };
    mut_obj
}
	
/**
 * Slab槽
 */
#[derive(Clone)]
pub struct GLSlot {
    pub context: WebGLContextWrap,
    pub index: usize,    // 槽的索引
}


pub struct GLSlab {
    pub buffer: Slab<WebGLBufferImpl>,
    pub geometry: Slab<WebGLGeometryImpl>,
    pub texture: Slab<WebGLTextureImpl>,
    pub sampler: Slab<WebGLSamplerImpl>,
    pub render_target: Slab<WebGLRenderTargetImpl>,
    pub render_buffer: Slab<WebGLRenderBufferImpl>,
    pub blend_state: Slab<WebGLBlendStateImpl>,
    pub depth_state: Slab<WebGLDepthStateImpl>,
    pub raster_state: Slab<WebGLRasterStateImpl>,
    pub stencil_state: Slab<WebGLStencilStateImpl>,
    pub program: Slab<WebGLProgramImpl>,
}

impl GLSlab {

    pub fn new() -> Self {
        Self {
            buffer: Slab::new(),
            geometry: Slab::new(),
            texture: Slab::new(),
            sampler: Slab::new(),
            render_target: Slab::new(),
            render_buffer: Slab::new(),
            blend_state: Slab::new(),
            depth_state: Slab::new(),
            raster_state: Slab::new(),
            stencil_state: Slab::new(),
            program: Slab::new(),
        }
    }

    pub fn new_slice<T>(context: &WebGLContextWrap, slab: &mut Slab<T>, obj: T) -> GLSlot {
        let index = slab.insert(obj);
        
        GLSlot {
            context: context.clone(),
            index: index,
        }
    }

    pub fn get_mut_slice<'a, T>(slab: &'a mut Slab<T>, slot: &GLSlot) -> Option<&'a mut T> {
        slab.get_mut(slot.index)
    }
    
    pub fn delete_slice<T>(slab: &mut Slab<T>, slot: &GLSlot) -> T {
        slab.remove(slot.index)
    }
}