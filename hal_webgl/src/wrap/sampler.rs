use hal_core::{Context, Sampler, SamplerDesc};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlot};
use implement::{WebGLSamplerImpl};

#[derive(Clone)]
pub struct WebGLSamplerWrap(pub GLSlot<WebGLSamplerImpl>);

impl Sampler for WebGLSamplerWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &SamplerDesc) -> Result<<Self::RContext as Context>::ContextSampler, String> {
        let rimpl = WebGLSamplerImpl(desc.clone());
        Ok(Self(GLSlot::new(&context.sampler, rimpl)))
    }

    fn delete(&self) {
    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    fn get_desc(&self) -> &SamplerDesc {
        let s = self.0.slab.get(self.0.index).unwrap();
        &s.0
    }
}