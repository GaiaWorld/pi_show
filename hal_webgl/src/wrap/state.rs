use hal_core::{Context, BlendState, DepthState, RasterState, StencilState, BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlab, GLSlot, convert_to_mut};
use implement::{WebGLBlendStateImpl, WebGLDepthStateImpl, WebGLRasterStateImpl, WebGLStencilStateImpl};

#[derive(Clone)]
pub struct WebGLBlendStateWrap {
    desc: BlendStateDesc,
    slot: GLSlot,
}

impl BlendState for WebGLBlendStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &BlendStateDesc) -> Result<<Self::RContext as Context>::ContextBlendState, String> {
        match WebGLBlendStateImpl::new(context, desc) {
            Err(s) => Err(s),
            Ok(state) => {
                let slab = convert_to_mut(&context.slabs.blend_state);
                let slot = GLSlab::new_slice(context, slab, state);
                Ok(Self {
                    slot: slot,
                    desc: desc.clone(),
                })
            }
        }
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        self.slot.index as u64
    }

    fn get_desc(&self) -> &BlendStateDesc {
        &self.desc
    }
}

// ================================== 

#[derive(Clone)]
pub struct WebGLDepthStateWrap {
    desc: DepthStateDesc,
    slot: GLSlot,
}

impl DepthState for WebGLDepthStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &DepthStateDesc) -> Result<<Self::RContext as Context>::ContextDepthState, String> {
        match WebGLDepthStateImpl::new(context, desc) {
            Err(s) => Err(s),
            Ok(state) => {
                let slab = convert_to_mut(&context.slabs.depth_state);
                let slot = GLSlab::new_slice(context, slab, state);
                Ok(Self {
                    slot: slot,
                    desc: desc.clone(),
                })
            }
        }
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        self.slot.index as u64
    }

    fn get_desc(&self) -> &DepthStateDesc {
        &self.desc
    }
}

// ================================== 

#[derive(Clone)]
pub struct WebGLRasterStateWrap {
    desc: RasterStateDesc,
    slot: GLSlot,
}

impl RasterState for WebGLRasterStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &RasterStateDesc) -> Result<<Self::RContext as Context>::ContextRasterState, String> {
        match WebGLRasterStateImpl::new(context, desc) {
            Err(s) => Err(s),
            Ok(state) => {
                let slab = convert_to_mut(&context.slabs.raster_state);
                let slot = GLSlab::new_slice(context, slab, state);
                Ok(Self {
                    slot: slot,
                    desc: desc.clone(),
                })
            }
        }
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        self.slot.index as u64
    }

    fn get_desc(&self) -> &RasterStateDesc {
        &self.desc
    }
}

// ================================== 

#[derive(Clone)]
pub struct WebGLStencilStateWrap {
    desc: StencilStateDesc,
    slot: GLSlot,
}

impl StencilState for WebGLStencilStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &StencilStateDesc) -> Result<<Self::RContext as Context>::ContextStencilState, String> {
        match WebGLStencilStateImpl::new(context, desc) {
            Err(s) => Err(s),
            Ok(state) => {
                let slab = convert_to_mut(&context.slabs.stencil_state);
                let slot = GLSlab::new_slice(context, slab, state);
                Ok(Self {
                    slot: slot,
                    desc: desc.clone(),
                })
            }
        }
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        self.slot.index as u64
    }

    fn get_desc(&self) -> &StencilStateDesc {
        &self.desc
    }
}