use std::sync::{Arc, Weak};
use hal_core::*;
use texture::{WebGLTextureImpl};
use context::{WebGLContextImpl};

use stdweb::{Object};
use stdweb::unstable::TryInto;
use webgl_rendering_context::{WebGLRenderingContext, WebGLRenderbuffer};
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

    pub is_default: bool, // 注：不能从默认的渲染目标上取color depth
    pub frame_buffer: Option<Object>,
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
    pub fn new_default(gl: &Arc<WebGLRenderingContext>, fbo: Option<Object>, w: u32, h: u32) -> Self {

        let fbo = match fbo {
            None => None,
            Some(fbo) => match TryInto::<Object>::try_into(js! {
                var fboWrap = {
                    wrap: @{fbo},
                };
                return fboWrap;
            }) {
                Err(_) => panic!("new default rendertarget failed"),
                Ok(fbo) => Some(fbo),
            }
        };
        
        WebGLRenderTargetImpl {
            gl: Arc::downgrade(gl),
            is_default: true,
            frame_buffer: fbo,
            color: None,    
            depth: None,
            width: w,
            height: h,
        }
    }

    pub fn new(gl: &Arc<WebGLRenderingContext>, w: u32, h: u32, pformat: &PixelFormat, dformat: &DataFormat, has_depth: bool) -> Result<Self, String> {
        
        match TryInto::<Object>::try_into(js! {
            var fbo = @{gl.as_ref()}.createFramebuffer();
            var fboWrap = {
                wrap: fbo
            };
            return fboWrap;
        }) {
            Ok(fb) => {
                js! {
                    @{gl.as_ref()}.bindFramebuffer(@{WebGLRenderingContext::FRAMEBUFFER}, @{&fb}.wrap);
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
                    is_default: false,
                    frame_buffer: Some(fb),
                    color: color,
                    depth: depth,
                    width: w,
                    height: h,
                })            
            }
            Err(_) => {
                return Err("WebGLRenderTargetImpl::new failed, Convertion Object Error".to_string());
            }
        }
    }
}

impl RenderTarget for WebGLRenderTargetImpl {
    type RContext = WebGLContextImpl;

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn get_color_texture(&self, _index: u32) -> Option<Arc<<<Self as RenderTarget>::RContext as Context>::ContextTexture>> {
        match &self.color {
            &Some(RenderTargetAttach::Texture(ref v)) => Some(v.clone()),
            _ => None,
        }
    }
}

impl Drop for WebGLRenderTargetImpl {
    fn drop(&mut self) {
        if let Some(gl) = &self.gl.upgrade() {
            js! {
                @{gl.as_ref()}.deleteFramebuffer(@{&self.frame_buffer}.wrap);
            }
        }
    }
}

impl AsRef<Self> for WebGLRenderTargetImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}