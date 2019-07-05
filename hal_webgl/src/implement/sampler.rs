use hal_core::{SamplerDesc};
use share::{Share};
use implement::context::{WebGLContextImpl}; 

pub struct WebGLSamplerImpl {
    context: Share<WebGLContextImpl>,
    desc: SamplerDesc,
}

impl WebGLSamplerImpl {

    pub fn new(context: &Share<WebGLContextImpl>, desc: &SamplerDesc) -> Result<Self, String> {
        Ok(Self {
            context: context.clone(),
            desc: desc.clone(),
        })
    }

    pub fn delete(&mut self) {

    }
}