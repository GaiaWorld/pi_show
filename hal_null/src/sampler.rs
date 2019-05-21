use hal_core::{Sampler};

pub struct NullSamplerImpl {
    
}

impl Sampler for NullSamplerImpl {
}

impl Drop for NullSamplerImpl {
    fn drop(&mut self) {
    }
}
