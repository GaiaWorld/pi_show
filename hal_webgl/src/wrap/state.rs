use std::sync::{Arc};
use hal_core::{Context, BlendState, DepthState, RasterState, StencilState, BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlot};
use implement::{WebGLBlendStateImpl, WebGLDepthStateImpl, WebGLRasterStateImpl, WebGLStencilStateImpl};

#[derive(Clone)]
pub struct WebGLBlendStateWrap {
    desc: BlendStateDesc,
    slot: GLSlot,
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

// ================================== 

#[derive(Clone)]
pub struct WebGLDepthStateWrap {
    desc: DepthStateDesc,
    slot: GLSlot,
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

// ================================== 

#[derive(Clone)]
pub struct WebGLRasterStateWrap {
    desc: RasterStateDesc,
    slot: GLSlot,
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

// ================================== 

#[derive(Clone)]
pub struct WebGLStencilStateWrap {
    desc: StencilStateDesc,
    slot: GLSlot,
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