use hal_core::*;

use std::rc::{Rc};
use std::cell::{RefCell};
use context::{RenderStats};

#[derive(Debug)]
pub struct WebGLSamplerImpl {
    pub min_filter: TextureFilterMode,
    pub mag_filter: TextureFilterMode,
    pub mip_filter: Option<TextureFilterMode>,

    pub u_wrap: TextureWrapMode,
    pub v_wrap: TextureWrapMode,

    pub stats: Rc<RefCell<RenderStats>>,
}

impl Sampler for WebGLSamplerImpl {
}

impl Drop for WebGLSamplerImpl {
    fn drop(&mut self) {
        self.stats.borrow_mut().sampler_count -= 1;
        println!("================= WebGLSamplerImpl Drop, stats = {:?}", self.stats.borrow());
    }
}

impl AsRef<Self> for WebGLSamplerImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl WebGLSamplerImpl {
    pub fn new(stats: &Rc<RefCell<RenderStats>>) -> Self {
        WebGLSamplerImpl {
            min_filter: TextureFilterMode::Linear,
            mag_filter: TextureFilterMode::Linear,
            mip_filter: None,

            u_wrap: TextureWrapMode::Repeat,
            v_wrap: TextureWrapMode::Repeat,
            stats: stats.clone(),
        }
    }
}