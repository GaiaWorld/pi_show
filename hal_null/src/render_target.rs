use std::rc::{Rc};
use hal_core::{RTAttachment, RenderTarget, RenderBuffer};
use texture::{NullTextureImpl};

pub struct NullRenderBufferImpl {
    
}

pub struct NullRenderTargetImpl {
    
}

impl RenderBuffer for NullRenderBufferImpl {
    fn get_size() -> (u32, u32) {
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

impl Drop for NullRenderTargetImpl {
    fn drop(&mut self) {
    }
}
