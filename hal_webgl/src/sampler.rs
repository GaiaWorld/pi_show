use hal_core::{Sampler};

pub struct WebGLSamplerImpl {
    
}

impl Sampler for WebGLSamplerImpl {
}

impl Drop for WebGLSamplerImpl {
    fn drop(&mut self) {
    }
}

impl AsRef<Self> for WebGLSamplerImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}