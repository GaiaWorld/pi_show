use std::sync::{Arc, Weak};
use hal_core::*;
use texture::{WebGLTextureImpl};
use webgl_rendering_context::{WebGLRenderingContext, WebGLFramebuffer, WebGLRenderbuffer};
use convert::*;

#[derive(Debug)]
pub struct WebGLRenderBufferImpl {
    gl: Weak<WebGLRenderingContext>,
    width: u32, 
    height: u32,
    format: PixelFormat,
    handle: WebGLRenderbuffer,
}

#[derive(Debug)]
pub enum RenderTargetAttach {
    Texture(Arc<WebGLTextureImpl>),
    Buffer(Arc<WebGLRenderBufferImpl>),
}

#[derive(Debug)]
pub struct WebGLRenderTargetImpl {

    gl: Weak<WebGLRenderingContext>,

    pub frame_buffer: Option<WebGLFramebuffer>,
    color: Option<RenderTargetAttach>,
    depth: Option<RenderTargetAttach>,
    width: u32,
    height: u32,
}

impl RenderBuffer for WebGLRenderBufferImpl {
    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }
}

impl Drop for WebGLRenderBufferImpl {
    fn drop(&mut self) {
         if let Some(gl) = &self.gl.upgrade() {
            gl.delete_renderbuffer(Some(&self.handle));
        }
    }
}

impl AsRef<Self> for WebGLRenderBufferImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl WebGLRenderBufferImpl {
    pub fn new(gl: &Arc<WebGLRenderingContext>, w: u32, h: u32, pformat: &PixelFormat) -> Option<Self> {
    
        let r = gl.create_renderbuffer();
        if r.is_none() {
            return None;
        }
        let r = r.unwrap();
        gl.bind_renderbuffer(WebGLRenderingContext::RENDERBUFFER, Some(&r));

        let format = get_pixel_format(pformat);
        gl.renderbuffer_storage(WebGLRenderingContext::RENDERBUFFER, format, w as i32, h as i32);

        Some(WebGLRenderBufferImpl {
            gl: Arc::downgrade(gl),
            width: w,
            height: h,
            format: *pformat,
            handle: r,
        })
    }
}

impl WebGLRenderTargetImpl {
    pub fn new_default(gl: &Arc<WebGLRenderingContext>) -> Self {
        WebGLRenderTargetImpl {
            gl: Arc::downgrade(gl),
            frame_buffer: None,
            color: None,    
            depth: None,
            width: 0,
            height: 0,
        }
    }

    pub fn new(gl: &Arc<WebGLRenderingContext>, w: u32, h: u32, pformat: &PixelFormat, dformat: &DataFormat, has_depth: bool) -> Result<Self, String> {
        
        let frame_buffer = gl.create_framebuffer();
        if frame_buffer.is_none() {
            return Err("WebGLRenderTargetImpl::new failed".to_string());
        }

        let fb_type = WebGLRenderingContext::FRAMEBUFFER;

        let tex_target = WebGLRenderingContext::TEXTURE_2D;
        let color_attachment = WebGLRenderingContext::COLOR_ATTACHMENT0;
        let color = match WebGLTextureImpl::new_2d(gl, w, h, 0, pformat, dformat, false, &TextureData::None) {
            Ok(texture) => {
                gl.framebuffer_texture2_d(fb_type, color_attachment, tex_target, Some(&texture.handle), 0);
                Some(RenderTargetAttach::Texture(Arc::new(texture)))
            }
            Err(_) => None,
        };

        let depth = if has_depth {
            let rb_type = WebGLRenderingContext::RENDERBUFFER;
            let depth_attachment = WebGLRenderingContext::DEPTH_ATTACHMENT;
            match WebGLRenderBufferImpl::new(gl, w, h, &PixelFormat::DEPTH16) {
                Some(rb) => {
                    gl.framebuffer_renderbuffer(fb_type, depth_attachment, rb_type, Some(&rb.handle));
                    Some(RenderTargetAttach::Buffer(Arc::new(rb)))
                }
                None => None,
            }
        } else {
            None
        };
        
        Ok(WebGLRenderTargetImpl {
            gl: Arc::downgrade(gl),
            frame_buffer: frame_buffer,
            color: color,
            depth: depth,
            width: w,
            height: h,
        })
    }
}

impl RenderTarget for WebGLRenderTargetImpl {
    type ContextTexture = WebGLTextureImpl;
    type ContextRenderBuffer = WebGLRenderBufferImpl;

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn get_color_texture(&self, _index: u32) -> Option<Arc<Self::ContextTexture>> {
        match &self.color {
            &Some(RenderTargetAttach::Texture(ref v)) => Some(v.clone()),
            _ => None,
        }
    }
}

impl Drop for WebGLRenderTargetImpl {
    fn drop(&mut self) {
        if let Some(gl) = &self.gl.upgrade() {
            gl.delete_framebuffer(self.frame_buffer.as_ref());
        }
    }
}

impl AsRef<Self> for WebGLRenderTargetImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}