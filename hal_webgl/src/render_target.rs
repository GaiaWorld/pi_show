use std::sync::{Arc, Weak};
use hal_core::{RTAttachment, RenderTarget, RenderBuffer};
use texture::{WebGLTextureImpl};
use webgl_rendering_context::{WebGLRenderingContext, WebGLFramebuffer};

pub struct WebGLRenderBufferImpl {
}

pub enum RenderTargetAttach {
    Texture(Arc<WebGLTextureImpl>),
    Buffer(Arc<WebGLRenderBufferImpl>),
}

pub struct WebGLRenderTargetImpl {

    gl: Weak<WebGLRenderingContext>,

    pub frame_buffer: Option<WebGLFramebuffer>,
    color: Option<RenderTargetAttach>,
    depth: Option<RenderTargetAttach>,
}

impl RenderBuffer for WebGLRenderBufferImpl {
    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }
}

impl Drop for WebGLRenderBufferImpl {
    fn drop(&mut self) {
    }
}

impl AsRef<Self> for WebGLRenderBufferImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl WebGLRenderTargetImpl {
    pub fn new_default(gl: &Arc<WebGLRenderingContext>) -> Self {
        WebGLRenderTargetImpl {
            gl: Arc::downgrade(gl),
            frame_buffer: None,
            color: None,    
            depth: None,
        }
    }

    pub fn new(gl: &Arc<WebGLRenderingContext>) -> Result<Self, String> {
        
        let frame_buffer = gl.create_framebuffer();
        if frame_buffer.is_none() {
            return Err("WebGLRenderTargetImpl::new failed".to_string());
        }

        Ok(WebGLRenderTargetImpl {
            gl: Arc::downgrade(gl),
            frame_buffer: frame_buffer,
            color: None,
            depth: None,
        })
    }
}

impl RenderTarget for WebGLRenderTargetImpl {
    type ContextTexture = WebGLTextureImpl;
    type ContextRenderBuffer = WebGLRenderBufferImpl;

    fn attach_texture(&mut self, _attachment: RTAttachment, _texture: &Arc<Self::ContextTexture>) {
        assert!(self.frame_buffer.is_some(), "WebGLRenderTargetImpl attach_texture failed, no fbo");
    }
    
    fn attach_render_buffer(&mut self, _attachment: RTAttachment, _buffer: &Arc<Self::ContextRenderBuffer>) {
        assert!(self.frame_buffer.is_some(), "WebGLRenderTargetImpl attach_render_buffer failed, no fbo");
    }
    
    fn get_texture(&self, _attachment: RTAttachment) -> Option<Arc<Self::ContextTexture>> {
        None
    }
}

impl Drop for WebGLRenderTargetImpl {
    fn drop(&mut self) {
    }
}

impl AsRef<Self> for WebGLRenderTargetImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}