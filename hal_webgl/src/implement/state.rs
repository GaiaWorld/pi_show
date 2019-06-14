use std::sync::{Arc};
use hal_core::{BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};
use wrap::{WebGLContextWrap};

pub struct WebGLBlendStateImpl {
    desc: BlendStateDesc,
}

impl WebGLBlendStateImpl {
    
    fn new(context: &Arc<WebGLContextWrap>, desc: &BlendStateDesc) -> Result<Self, String> {
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

pub struct WebGLDepthStateImpl {
    desc: DepthStateDesc,
}

impl WebGLDepthStateImpl {

    fn new(context: &Arc<WebGLContextWrap>, desc: &DepthStateDesc) -> Result<Self, String> {
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

pub struct WebGLRasterStateImpl {
    desc: RasterStateDesc,
}

impl WebGLRasterStateImpl {
    
    fn new(context: &Arc<WebGLContextWrap>, desc: &RasterStateDesc) -> Result<Self, String> {
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

pub struct WebGLStencilStateImpl {
    desc: StencilStateDesc,
}

impl WebGLStencilStateImpl {

    fn new(context: &Arc<WebGLContextWrap>, desc: &StencilStateDesc) -> Result<Self, String> {
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