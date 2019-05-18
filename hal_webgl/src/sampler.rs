use hal_core::{Sampler};
use texture::{WebGLTextureImpl};

pub struct WebGLSamplerImpl {
    
}

impl Sampler for WebGLSamplerImpl {
    type ContextTexture = WebGLTextureImpl;
}

impl Drop for WebGLSamplerImpl {
    fn drop(&mut self) {
    }
}