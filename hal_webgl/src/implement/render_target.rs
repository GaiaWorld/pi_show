use std::sync::{Arc};
use wrap::{WebGLContextWrap};

pub struct WebGLRenderBufferImpl {
}

impl WebGLRenderBufferImpl  {

    pub fn new(context: &Arc<WebGLContextWrap>) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    pub fn delete(&self) {

    }

    pub fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }
}

pub struct WebGLRenderTargetImpl {
}

impl WebGLRenderTargetImpl {

    pub fn new(context: &Arc<WebGLContextWrap>) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    pub fn delete(&self) {

    }

    pub fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }
}