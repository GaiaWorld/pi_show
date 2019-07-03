use hal_core::{Capabilities};
use webgl_rendering_context::{WebGLRenderingContext};

pub struct WebGLContextImpl {
    pub caps: Capabilities,
    pub context: WebGLRenderingContext,
}

impl WebGLContextImpl {
    pub fn new(context: WebGLRenderingContext) -> Self {
        let caps = Capabilities::new();
        Self {
            caps: caps,
            context: context,
        }
    }
}