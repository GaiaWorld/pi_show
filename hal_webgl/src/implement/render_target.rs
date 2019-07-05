use share::{Share};
use hal_core::{PixelFormat};
use implement::context::{WebGLContextImpl}; 
use implement::texture::{WebGLTextureImpl};
use stdweb::{Object};
use stdweb::unstable::TryInto;
use webgl_rendering_context::{WebGLRenderingContext, WebGLRenderbuffer};
use implement::convert::*;

use wrap::{GLSlot};

pub enum RenderTargetAttach {
    Texture(GLSlot<WebGLTextureImpl>),
    Buffer(GLSlot<WebGLRenderBufferImpl>),
}

pub struct WebGLRenderBufferImpl {
    context: Share<WebGLContextImpl>,

    width: u32, 
    height: u32,
    format: PixelFormat,
    handle: WebGLRenderbuffer,
}

impl WebGLRenderBufferImpl  {

    pub fn new(context: &Share<WebGLContextImpl>, w: u32, h: u32, pformat: PixelFormat) -> Result<Self, String> {
        let gl = &context.context;

        let r = gl.create_renderbuffer();
        if r.is_none() {
            return Err("WebGLRenderBufferImpl new failed".to_string());
        }

        let r = r.unwrap();
        gl.bind_renderbuffer(WebGLRenderingContext::RENDERBUFFER, Some(&r));

        let format = get_pixel_format(pformat);
        gl.renderbuffer_storage(WebGLRenderingContext::RENDERBUFFER, format, w as i32, h as i32);

        Ok(WebGLRenderBufferImpl {
            context: context.clone(),
            width: w,
            height: h,
            format: pformat,
            handle: r,
        })
    }
    
    pub fn delete(&self) {
        self.context.context.delete_renderbuffer(Some(&self.handle))
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

pub struct WebGLRenderTargetImpl {
    context: Share<WebGLContextImpl>,

    pub is_default: bool, // 注：不能从默认的渲染目标上取color depth
    pub handle: Option<Object>,
    width: u32,
    height: u32,
    color: Option<RenderTargetAttach>,
    depth: Option<RenderTargetAttach>,
}

impl WebGLRenderTargetImpl {
    pub fn new(context: &Share<WebGLContextImpl>, w: u32, h: u32, texture: &GLSlot<WebGLTextureImpl>, rb: Option<&GLSlot<WebGLRenderBufferImpl>>) -> Result<Self, String> {
        let gl = &context.context;

        let fbo = TryInto::<Object>::try_into(js! {
            var fbo = @{gl}.createFramebuffer();
            var fboWrap = {
                wrap: fbo
            };
            return fboWrap;
        });

        if let Err(s) = &fbo {
            return Err(s.to_string());
        }
        let fbo = fbo.unwrap();
        
        js! {
            @{gl}.bindFramebuffer(@{WebGLRenderingContext::FRAMEBUFFER}, @{&fbo}.wrap);
        }
        
        let t = texture.get_mut().unwrap();
        
        let fb_type = WebGLRenderingContext::FRAMEBUFFER;
        let tex_target = WebGLRenderingContext::TEXTURE_2D;
        let color_attachment = WebGLRenderingContext::COLOR_ATTACHMENT0;
        
        gl.framebuffer_texture2_d(fb_type, color_attachment, tex_target, Some(&t.handle), 0);
        
        if rb.is_some() {
            
            let rb = rb.unwrap().get_mut().unwrap();

            let rb_type = WebGLRenderingContext::RENDERBUFFER;
            let depth_attachment = WebGLRenderingContext::DEPTH_ATTACHMENT;
            
            gl.framebuffer_renderbuffer(fb_type, depth_attachment, rb_type, Some(&rb.handle));
        };

        js! {
            @{gl}.bindFramebuffer(@{WebGLRenderingContext::FRAMEBUFFER}, null);
        }
        
        let color = RenderTargetAttach::Texture(texture.clone());
        let depth = if rb.is_none() { None } else { Some(RenderTargetAttach::Buffer( rb.unwrap().clone() )) };

        Ok(Self {
            context: context.clone(),

            is_default: false,
            handle: Some(fbo),
            width: w,
            height: h,
            color: Some(color),
            depth: depth,
        })
    }
    
    pub fn new_default(context: &Share<WebGLContextImpl>, fbo: Option<Object>, w: u32, h: u32) -> Self {
        Self {
            context: context.clone(),
            is_default: true,
            handle: fbo,
            color: None,    
            depth: None,
            width: w,
            height: h,
        }
    }
    
    pub fn delete(&self) {
        if let Some(fbo) = &self.handle {
            let gl = &self.context.context;
            js! {
                @{gl}.deleteFramebuffer(@{fbo}.wrap);
            }
        }
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}