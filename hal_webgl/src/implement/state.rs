use hal_core::{BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};
use wrap::{WebGLContextWrap};

pub struct WebGLBlendStateImpl {
}

impl WebGLBlendStateImpl {
    
    pub fn new(context: &WebGLContextWrap, desc: &BlendStateDesc) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    pub fn delete(&mut self) {

    }
}

// ================================== 

pub struct WebGLDepthStateImpl {
}

impl WebGLDepthStateImpl {

    pub fn new(context: &WebGLContextWrap, desc: &DepthStateDesc) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    pub fn delete(&mut self) {

    }
}

// ================================== 

pub struct WebGLRasterStateImpl {
}

impl WebGLRasterStateImpl {
    
    pub fn new(context: &WebGLContextWrap, desc: &RasterStateDesc) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    pub fn delete(&mut self) {

    }
}

// ================================== 

pub struct WebGLStencilStateImpl {
}

impl WebGLStencilStateImpl {

    pub fn new(context: &WebGLContextWrap, desc: &StencilStateDesc) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    pub fn delete(&mut self) {

    }
}