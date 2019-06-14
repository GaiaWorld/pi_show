use std::sync::{Arc};
use wrap::{WebGLContextWrap};

pub struct WebGLRenderBufferImpl {
}

impl WebGLRenderBufferImpl  {

    fn new(context: &Arc<WebGLContextWrap>) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }
}

pub struct WebGLRenderTargetImpl {
}

impl WebGLRenderTargetImpl {

    fn new(context: &Arc<WebGLContextWrap>) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }
}