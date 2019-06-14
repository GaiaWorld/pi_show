use std::sync::{Arc};
use hal_core::{Context, RenderBuffer, RenderTarget};
use wrap::context::{WebGLContextWrap};
use wrap::texture::{WebGLTextureWrap};
use implement::{WebGLRenderBufferImpl, WebGLRenderTargetImpl};

pub struct WebGLRenderBufferWrap {
}

impl RenderBuffer for WebGLRenderBufferWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Arc<Self::RContext>) -> Result<<Self::RContext as Context>::ContextRenderBuffer, String> {
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

impl Clone for WebGLRenderBufferWrap {
    fn clone(&self) -> Self {
        Self {
            
        }
    }
}

pub struct WebGLRenderTargetWrap {
}

impl RenderTarget for WebGLRenderTargetWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Arc<Self::RContext>) -> Result<<Self::RContext as Context>::ContextRenderTarget, String> {
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

impl Clone for WebGLRenderTargetWrap {
    fn clone(&self) -> Self {
        Self {
            
        }
    }
}