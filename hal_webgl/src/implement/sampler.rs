use std::sync::{Arc};
use hal_core::{SamplerDesc};
use wrap::{WebGLContextWrap};

pub struct WebGLSamplerImpl {
    desc: SamplerDesc,
}

impl WebGLSamplerImpl {

    fn new(context: &Arc<WebGLContextWrap>, desc: &SamplerDesc) -> Result<Self, String> {
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