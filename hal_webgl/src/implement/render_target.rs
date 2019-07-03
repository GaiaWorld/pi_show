use share::{Share};
use hal_core::{PixelFormat, DataFormat};
use implement::context::{WebGLContextImpl}; 
use stdweb::{Object};

pub struct WebGLRenderBufferImpl {
    context: Share<WebGLContextImpl>,
}

impl WebGLRenderBufferImpl  {

    pub fn new(context: &Share<WebGLContextImpl>, w: u32, h: u32, pformat: PixelFormat) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    pub fn delete(&self) {

    }

    pub fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }
}

pub struct WebGLRenderTargetImpl {
    context: Share<WebGLContextImpl>,
}

impl WebGLRenderTargetImpl {
    pub fn new(context: &Share<WebGLContextImpl>, w: u32, h: u32, pformat: PixelFormat, dformat: DataFormat, has_depth: bool) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    pub fn new_default(context: &Share<WebGLContextImpl>, fbo: Option<Object>, w: u32, h: u32) -> Self {
        Self {
            context: context.clone(),
        }
    }
    
    pub fn delete(&self) {

    }

    pub fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }
}