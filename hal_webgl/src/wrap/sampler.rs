use hal_core::{Context, Sampler, SamplerDesc};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlot};
use implement::{WebGLSamplerImpl};

#[derive(Clone)]
pub struct WebGLSamplerWrap {
    slot: GLSlot,
    desc: SamplerDesc,
}

impl Sampler for WebGLSamplerWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &SamplerDesc) -> Result<<Self::RContext as Context>::ContextSampler, String> {
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