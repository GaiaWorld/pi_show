use std::sync::{Arc};
use hal_core::{Context, BlendState, DepthState, RasterState, StencilState, BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};
use wrap::context::{WebGLContextWrap};
use implement::{WebGLBlendStateImpl, WebGLDepthStateImpl, WebGLRasterStateImpl, WebGLStencilStateImpl};

pub struct WebGLBlendStateWrap {
    desc: BlendStateDesc,
}

impl BlendState for WebGLBlendStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Arc<Self::RContext>, desc: &BlendStateDesc) -> Result<<Self::RContext as Context>::ContextBlendState, String> {
        Err("not implmentation".to_string())
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn get_desc(&self) -> &BlendStateDesc {
        &self.desc
    }
}

impl Clone for WebGLBlendStateWrap {
    fn clone(&self) -> Self {
        Self {
            desc: self.desc.clone(),
        }
    }
}

// ================================== 

pub struct WebGLDepthStateWrap {
    desc: DepthStateDesc,
}

impl DepthState for WebGLDepthStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Arc<Self::RContext>, desc: &DepthStateDesc) -> Result<<Self::RContext as Context>::ContextDepthState, String> {
        Err("not implmentation".to_string())
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn get_desc(&self) -> &DepthStateDesc {
        &self.desc
    }
}

impl Clone for WebGLDepthStateWrap {
    fn clone(&self) -> Self {
        Self {
            desc: self.desc.clone(),
        }
    }
}

// ================================== 

pub struct WebGLRasterStateWrap {
    desc: RasterStateDesc,
}

impl RasterState for WebGLRasterStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Arc<Self::RContext>, desc: &RasterStateDesc) -> Result<<Self::RContext as Context>::ContextRasterState, String> {
        Err("not implmentation".to_string())
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn get_desc(&self) -> &RasterStateDesc {
        &self.desc
    }
}

impl Clone for WebGLRasterStateWrap {
    fn clone(&self) -> Self {
        Self {
            desc: self.desc.clone(),
        }
    }
}

// ================================== 

pub struct WebGLStencilStateWrap {
    desc: StencilStateDesc,
}

impl StencilState for WebGLStencilStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Arc<Self::RContext>, desc: &StencilStateDesc) -> Result<<Self::RContext as Context>::ContextStencilState, String> {
        Err("not implmentation".to_string())
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn get_desc(&self) -> &StencilStateDesc {
        &self.desc
    }
}

impl Clone for WebGLStencilStateWrap {
    fn clone(&self) -> Self {
        Self {
            desc: self.desc.clone(),
        }
    }
}