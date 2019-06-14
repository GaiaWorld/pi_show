use std::sync::{Arc};
use hal_core::{Context, Sampler, SamplerDesc};
use wrap::context::{WebGLContextWrap};
use implement::{WebGLSamplerImpl};

pub struct WebGLSamplerWrap {
    desc: SamplerDesc,
}

impl Sampler for WebGLSamplerWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Arc<Self::RContext>, desc: &SamplerDesc) -> Result<<Self::RContext as Context>::ContextSampler, String> {
        Err("not implmentation".to_string())
    }

    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn get_desc(&self) -> &SamplerDesc {
        &self.desc
    }
}

impl Clone for WebGLSamplerWrap {
    fn clone(&self) -> Self {
        Self {
            desc: self.desc.clone(),
        }
    }
}