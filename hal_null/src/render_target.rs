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

impl RenderTarget for NullRenderTargetImpl {
    type ContextTexture = NullTextureImpl;
    type ContextRenderBuffer = NullRenderBufferImpl;

    fn attach_texture(&mut self, _attachment: RTAttachment, _texture: &Arc<Self::ContextTexture>) {
    }
    
    fn attach_render_buffer(&mut self, _attachment: RTAttachment, _buffer: &Arc<Self::ContextRenderBuffer>) {
    }
    
    fn get_texture(&self, _attachment: RTAttachment) -> Option<Arc<Self::ContextTexture>> {
        None
    }
}

impl Drop for NullRenderTargetImpl {
    fn drop(&mut self) {
    }
}
