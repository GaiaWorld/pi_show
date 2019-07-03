use hal_core::{Context, BlendState, DepthState, RasterState, StencilState, BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlot, convert_to_mut};
use implement::{WebGLBlendStateImpl, WebGLDepthStateImpl, WebGLRasterStateImpl, WebGLStencilStateImpl};

#[derive(Clone)]
pub struct WebGLBlendStateWrap {
    pub desc: BlendStateDesc,
    pub slot: GLSlot<WebGLBlendStateImpl>,
}

impl BlendState for WebGLBlendStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &BlendStateDesc) -> Result<<Self::RContext as Context>::ContextBlendState, String> {
        match WebGLBlendStateImpl::new(&context.rimpl, desc) {
            Err(s) => Err(s),
            Ok(state) => {
                Ok(Self {
                    slot: GLSlot::new(&context.blend_state, state),
                    desc: desc.clone(),
                })
            }
        }
    }
    
    fn delete(&self) {
        let slab = convert_to_mut(self.slot.slab.as_ref());
        let mut state = slab.remove(self.slot.index);
        state.delete();
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
    pub desc: DepthStateDesc,
    pub slot: GLSlot<WebGLDepthStateImpl>,
}

impl DepthState for WebGLDepthStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &DepthStateDesc) -> Result<<Self::RContext as Context>::ContextDepthState, String> {
        match WebGLDepthStateImpl::new(&context.rimpl, desc) {
            Err(s) => Err(s),
            Ok(state) => {
                Ok(Self {
                    slot: GLSlot::new(&context.depth_state, state),
                    desc: desc.clone(),
                })
            }
        }
    }
    
    fn delete(&self) {
        let slab = convert_to_mut(self.slot.slab.as_ref());
        let mut state = slab.remove(self.slot.index);
        state.delete();
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
    pub desc: RasterStateDesc,
    pub slot: GLSlot<WebGLRasterStateImpl>,
}

impl RasterState for WebGLRasterStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &RasterStateDesc) -> Result<<Self::RContext as Context>::ContextRasterState, String> {
        match WebGLRasterStateImpl::new(&context.rimpl, desc) {
            Err(s) => Err(s),
            Ok(state) => {
               Ok(Self {
                    slot: GLSlot::new(&context.raster_state, state),
                    desc: desc.clone(),
                })
            }
        }
    }
    
    fn delete(&self) {
        let slab = convert_to_mut(self.slot.slab.as_ref());
        let mut state = slab.remove(self.slot.index);
        state.delete();
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
    pub desc: StencilStateDesc,
    pub slot: GLSlot<WebGLStencilStateImpl>,
}

impl StencilState for WebGLStencilStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &StencilStateDesc) -> Result<<Self::RContext as Context>::ContextStencilState, String> {
        match WebGLStencilStateImpl::new(&context.rimpl, desc) {
            Err(s) => Err(s),
            Ok(state) => {
                Ok(Self {
                    slot: GLSlot::new(&context.stencil_state, state),
                    desc: desc.clone(),
                })
            }
        }
    }
    
    fn delete(&self) {
        let slab = convert_to_mut(self.slot.slab.as_ref());
        let mut state = slab.remove(self.slot.index);
        state.delete();
    }

    fn get_id(&self) -> u64 {
        self.slot.index as u64
    }

    fn get_desc(&self) -> &StencilStateDesc {
        &self.desc
    }
}