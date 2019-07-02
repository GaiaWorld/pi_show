use hal_core::{Context, Sampler, SamplerDesc};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlab, GLSlot, convert_to_mut};
use implement::{WebGLSamplerImpl};

#[derive(Clone)]
pub struct WebGLSamplerWrap {
    slot: GLSlot,
    desc: SamplerDesc,
}

impl Sampler for WebGLSamplerWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &SamplerDesc) -> Result<<Self::RContext as Context>::ContextSampler, String> {
         match WebGLSamplerImpl::new(context, desc) {
            Err(s) => Err(s),
            Ok(sampler) => {
                let slab = convert_to_mut(&context.slabs.sampler);
                let slot = GLSlab::new_slice(context, slab, sampler);
                Ok(Self {
                    slot: slot,
                    desc: desc.clone(),
                })
            }
        }
    }

    fn delete(&self) {
        let slab = convert_to_mut(&self.slot.context.slabs.sampler);
        let mut sampler = GLSlab::delete_slice(slab, &self.slot);
        sampler.delete();
    }

    fn get_id(&self) -> u64 {
        self.slot.index as u64
    }

    fn get_desc(&self) -> &SamplerDesc {
        &self.desc
    }
}