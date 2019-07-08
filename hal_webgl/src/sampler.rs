use hal_core::{SamplerDesc};

pub struct WebGLSamplerImpl(pub SamplerDesc);

impl WebGLSamplerImpl {
    pub fn new(desc: &SamplerDesc) -> Self {
        Self(desc.clone())
    }
}