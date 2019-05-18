use std::rc::{Rc};
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

impl RenderTarget for WebGLRenderTargetImpl {
    type ContextTexture = WebGLTextureImpl;
    type ContextRenderBuffer = WebGLRenderBufferImpl;

    fn get_size() -> (u32, u32) {
        (0, 0)
    }

    fn attach_texture(_attachment: RTAttachment, _texture: Rc<Self::ContextTexture>) {

    }
    
    fn attach_render_buffer(_attachment: RTAttachment, _buffer: Rc<Self::ContextRenderBuffer>) {

    }
    
    fn get_texture(_attachment: RTAttachment) -> Option<Self::ContextTexture> {
        None
    }
}