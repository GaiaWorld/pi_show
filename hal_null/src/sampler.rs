use hal_core::{Sampler};
use texture::{NullTextureImpl};

pub struct NullSamplerImpl {
    
}

impl Sampler for NullSamplerImpl {
    type ContextTexture = NullTextureImpl;
}