use convert::*;
use hal_core::{HalItem, HalRenderBuffer, HalTexture, PixelFormat};
use share::Share;
use texture::WebGLTextureImpl;
use web_sys::{WebGlRenderbuffer, WebGlRenderingContext, WebGlFramebuffer};

pub struct WebGLRenderBufferImpl {
    width: u32,
    height: u32,
    _format: PixelFormat,
    handle: WebGlRenderbuffer,
}

impl WebGLRenderBufferImpl {
    pub fn new(
        gl: &WebGlRenderingContext,
        w: u32,
        h: u32,
        pformat: PixelFormat,
    ) -> Result<Self, String> {
        let r = gl.create_renderbuffer();
        if r.is_none() {
            return Err("WebGLRenderBufferImpl new failed".to_string());
        }

        let r = r.unwrap();
        gl.bind_renderbuffer(WebGlRenderingContext::RENDERBUFFER, Some(&r));

        let format = get_pixel_format(pformat);
        gl.renderbuffer_storage(
            WebGlRenderingContext::RENDERBUFFER,
            format,
            w as i32,
            h as i32,
        );

        Ok(WebGLRenderBufferImpl {
            width: w,
            height: h,
            _format: pformat,
            handle: r,
        })
    }

    pub fn delete(&self, gl: &WebGlRenderingContext) {
        gl.delete_renderbuffer(Some(&self.handle))
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

pub struct WebGLRenderTargetImpl {
    pub is_default: bool, // 注：不能从默认的渲染目标上取color depth
    
    pub handle: Option<WebGlFramebuffer>,
    
    width: u32,
    height: u32,
    
    // 不需要释放，全部由外界维护
    pub color: Option<HalTexture>,
    pub depth: Option<HalRenderBuffer>,
}

impl WebGLRenderTargetImpl {

    pub fn new_default(fbo: Option<WebGlFramebuffer>, w: u32, h: u32) -> Self {
        Self {
            is_default: true,
            handle: fbo,
            color: None,
            depth: None,
            width: w,
            height: h,
        }
    }

    pub fn new(
        gl: &WebGlRenderingContext,
        w: u32,
        h: u32,
    ) -> Result<Self, String> {
		let fbo = gl.create_framebuffer();
        
        if let None = fbo {
			return Err("WebGLRenderTargetImpl::new fail".to_string());
		}

        Ok(Self {
            is_default: false,
            handle: fbo,
            width: w,
            height: h,
            color: None,
            depth: None,
        })
    }

    pub fn delete(&self, gl: &WebGlRenderingContext) {
        if let Some(fbo) = &self.handle {
			gl.delete_framebuffer(Some(fbo));
        }
    }

    pub fn set_color(&mut self, 
        gl: &WebGlRenderingContext, 
        tex_wrap: Option<&HalTexture>, 
        tex: Option<&WebGLTextureImpl>) -> Result<(), String> {
        
        if let Some(t) = tex {
            if t.width != self.width || t.height != self.height {
                return Err(format!("WebGLRenderTargetImpl::set_color fail, w and h not match, rt: w = {}, h = {} tex: w = {}, h = {}", self.width, self.height, t.width, t.height));
            }
        }
            
        self.color = tex_wrap.map(|tex| {
            HalTexture {
                item: HalItem {index: tex.item.index, use_count: tex.item.use_count },
                destroy_func: Share::new(move |_index: u32, _use_count: u32| {
                }),
            }
        });
        
        gl.bind_framebuffer(WebGlRenderingContext::FRAMEBUFFER, self.handle.as_ref());
        
        match &self.color {
            None => {
                gl.framebuffer_texture_2d(
                    WebGlRenderingContext::FRAMEBUFFER,
                    WebGlRenderingContext::COLOR_ATTACHMENT0,
                    WebGlRenderingContext::TEXTURE_2D,
                    None,
                    0,
                );
            }
            Some(_) => {
                let tex = tex.map(|tex| &tex.handle);
                gl.framebuffer_texture_2d(
                    WebGlRenderingContext::FRAMEBUFFER,
                    WebGlRenderingContext::COLOR_ATTACHMENT0,
                    WebGlRenderingContext::TEXTURE_2D,
                    tex,
                    0,
                );
            }
        }

        gl.bind_framebuffer(WebGlRenderingContext::FRAMEBUFFER, None);

        Ok(())
    }

    pub fn set_depth(&mut self, 
        gl: &WebGlRenderingContext, 
        depth_wrap: Option<&HalRenderBuffer>,
        depth: Option<&WebGLRenderBufferImpl>) -> Result<(), String> {
 
        if let Some(d) = depth {
            if d.width != self.width || d.height != self.height {
                return Err(format!("WebGLRenderTargetImpl::set_depth fail, w and h not match, rt: w = {}, h = {} rb: w = {}, h = {}", self.width, self.height, d.width, d.height));
            }
        }

        self.depth = depth_wrap.map(|rb| {
            HalRenderBuffer {
                item: HalItem {index: rb.item.index, use_count: rb.item.use_count },
                destroy_func: Share::new(move |_index: u32, _use_count: u32| {
                }),
            }
        });
        
        gl.bind_framebuffer(WebGlRenderingContext::FRAMEBUFFER, self.handle.as_ref());

        match &self.depth {
            None => {
                gl.framebuffer_renderbuffer(
                    WebGlRenderingContext::FRAMEBUFFER,
                    WebGlRenderingContext::DEPTH_ATTACHMENT,
                    WebGlRenderingContext::RENDERBUFFER,
                    None,
                );
            }
            Some(_) => {
                let depth = depth.map(|d| &d.handle);
                gl.framebuffer_renderbuffer(
                    WebGlRenderingContext::FRAMEBUFFER,
                    WebGlRenderingContext::DEPTH_ATTACHMENT,
                    WebGlRenderingContext::RENDERBUFFER,
                    depth,
                );
            }
        }

        gl.bind_framebuffer(WebGlRenderingContext::FRAMEBUFFER, None);
        
        Ok(())
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_color(&self, _index: u32) -> Option<&HalTexture> {
        self.color.as_ref()
    }

    pub fn get_depth(&self) -> Option<&HalRenderBuffer> {
        self.depth.as_ref()
    }

}
