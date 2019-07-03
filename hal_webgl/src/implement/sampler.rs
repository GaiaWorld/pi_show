use hal_core::{SamplerDesc};
use share::{Share};
use implement::context::{WebGLContextImpl}; 

pub struct WebGLSamplerImpl {
    context: Share<WebGLContextImpl>,
}

impl WebGLSamplerImpl {

    pub fn new(context: &Share<WebGLContextImpl>, desc: &SamplerDesc) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }

    pub fn delete(&mut self) {

    }
}