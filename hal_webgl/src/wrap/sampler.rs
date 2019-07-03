use hal_core::{Context, Sampler, SamplerDesc};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlot, convert_to_mut};
use implement::{WebGLSamplerImpl};

#[derive(Clone)]
pub struct WebGLSamplerWrap {
    slot: GLSlot<WebGLSamplerImpl>,
    desc: SamplerDesc,
}

impl Sampler for WebGLSamplerWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &SamplerDesc) -> Result<<Self::RContext as Context>::ContextSampler, String> {
         match WebGLSamplerImpl::new(&context.rimpl, desc) {
            Err(s) => Err(s),
            Ok(sampler) => {
                Ok(Self {
                    slot: GLSlot::new(&context.sampler, sampler),
                    desc: desc.clone(),
                })
            }
        }
    }

    fn delete(&self) {
        let slab = convert_to_mut(self.slot.slab.as_ref());
        let mut sampler = slab.remove(self.slot.index);
        sampler.delete();
    }

    fn get_id(&self) -> u64 {
        self.slot.index as u64
    }

    fn get_desc(&self) -> &SamplerDesc {
        &self.desc
    }
}