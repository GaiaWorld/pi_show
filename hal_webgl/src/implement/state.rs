use hal_core::{BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};
use share::{Share};
use implement::context::{WebGLContextImpl}; 

pub struct WebGLBlendStateImpl {
    context: Share<WebGLContextImpl>,
}

impl WebGLBlendStateImpl {
    
    pub fn new(context: &Share<WebGLContextImpl>, desc: &BlendStateDesc) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    pub fn delete(&mut self) {

    }
}

// ================================== 

pub struct WebGLDepthStateImpl {
    context: Share<WebGLContextImpl>,
}

impl WebGLDepthStateImpl {

    pub fn new(context: &Share<WebGLContextImpl>, desc: &DepthStateDesc) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    pub fn delete(&mut self) {

    }
}

// ================================== 

pub struct WebGLRasterStateImpl {
    context: Share<WebGLContextImpl>,
}

impl WebGLRasterStateImpl {
    
    pub fn new(context: &Share<WebGLContextImpl>, desc: &RasterStateDesc) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    pub fn delete(&mut self) {

    }
}

// ================================== 

pub struct WebGLStencilStateImpl {
    context: Share<WebGLContextImpl>,
}

impl WebGLStencilStateImpl {

    pub fn new(context: &Share<WebGLContextImpl>, desc: &StencilStateDesc) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    pub fn delete(&mut self) {

    }
}