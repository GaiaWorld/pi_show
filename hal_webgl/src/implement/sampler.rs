use hal_core::{SamplerDesc};
use wrap::{WebGLContextWrap};

pub struct WebGLSamplerImpl {
}

impl WebGLSamplerImpl {

    pub fn new(context: &WebGLContextWrap, desc: &SamplerDesc) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }

    pub fn delete(&mut self) {

    }
}