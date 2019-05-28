use std::sync::{Arc};
use hal_core::{RTAttachment, RenderTarget, RenderBuffer};
use texture::{NullTextureImpl};

pub struct NullRenderBufferImpl {
    
}

pub struct NullRenderTargetImpl {
    
}

impl RenderBuffer for NullRenderBufferImpl {
    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }
}

impl Drop for NullRenderBufferImpl {
    fn drop(&mut self) {
    }
}

impl AsRef<Self> for NullRenderBufferImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl RenderTarget for NullRenderTargetImpl {
    type ContextTexture = NullTextureImpl;
    type ContextRenderBuffer = NullRenderBufferImpl;

    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }

    fn get_color_texture(&self, index: u32) -> Option<Arc<Self::ContextTexture>> {
        None
    }
}

impl Drop for NullRenderTargetImpl {
    fn drop(&mut self) {
    }
}

impl AsRef<Self> for NullRenderTargetImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}
