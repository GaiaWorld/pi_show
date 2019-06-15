use hal_core::{Context, RenderBuffer, RenderTarget};
use wrap::context::{WebGLContextWrap};
use wrap::texture::{WebGLTextureWrap};
use wrap::gl_slab::{GLSlot};
use implement::{WebGLRenderBufferImpl, WebGLRenderTargetImpl};

#[derive(Clone)]
pub struct WebGLRenderBufferWrap(GLSlot);

impl RenderBuffer for WebGLRenderBufferWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext) -> Result<<Self::RContext as Context>::ContextRenderBuffer, String> {
        Err("not implmentation".to_string())
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }
}

#[derive(Clone)]
pub struct WebGLRenderTargetWrap(GLSlot);

impl WebGLRenderTargetWrap {
    pub fn new(slot: GLSlot) -> Self {
        Self(slot)
    }
}

impl RenderTarget for WebGLRenderTargetWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext) -> Result<<Self::RContext as Context>::ContextRenderTarget, String> {
        Err("not implmentation".to_string())
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }

    fn get_color_texture(&self, index: u32) -> Option<<<Self as RenderTarget>::RContext as Context>::ContextTexture> {
        None
    }
}