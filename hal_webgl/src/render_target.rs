use std::sync::{Arc};
use hal_core::{RTAttachment, RenderTarget, RenderBuffer};
use texture::{WebGLTextureImpl};

pub struct WebGLRenderBufferImpl {
    
}

pub struct WebGLRenderTargetImpl {
    
}

impl RenderBuffer for WebGLRenderBufferImpl {
    fn get_size() -> (u32, u32) {
        (0, 0)
    }

}

impl Drop for WebGLRenderBufferImpl {
    fn drop(&mut self) {
    }
}

impl RenderTarget for WebGLRenderTargetImpl {
    type ContextTexture = WebGLTextureImpl;
    type ContextRenderBuffer = WebGLRenderBufferImpl;

    fn get_size() -> (u32, u32) {
        (0, 0)
    }

    fn attach_texture(_attachment: RTAttachment, _texture: &Arc<Self::ContextTexture>) {

    }
    
    fn attach_render_buffer(_attachment: RTAttachment, _buffer: &Arc<Self::ContextRenderBuffer>) {

    }
    
    fn get_texture(_attachment: RTAttachment) -> Option<Self::ContextTexture> {
        None
    }
}

impl Drop for WebGLRenderTargetImpl {
    fn drop(&mut self) {
    }
}