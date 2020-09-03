use convert::*;
use hal_core::{HalRenderBuffer, HalTexture, PixelFormat};
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
    pub is_tex_destroy: bool,
    pub color: Option<HalTexture>,
    pub depth: Option<HalRenderBuffer>,
}

impl WebGLRenderTargetImpl {
    pub fn new(
        gl: &WebGlRenderingContext,
        w: u32,
        h: u32,
        texture: &WebGLTextureImpl,
        rb: Option<&WebGLRenderBufferImpl>,
        texture_wrap: HalTexture,
        rb_wrap: Option<HalRenderBuffer>,
        is_tex_destroy: bool,
    ) -> Result<Self, String> {
		let fbo = gl.create_framebuffer();
        // let fbo = TryInto::<Object>::try_into(js! {
        //     var fbo = @{gl}.createFramebuffer();
        //     var fboWrap = {
        //         wrap: fbo
        //     };
        //     return fboWrap;
        // });
		
		if let None = fbo {
			return Err("create fbo fail".to_string())
		}

		gl.bind_framebuffer(WebGlRenderingContext::FRAMEBUFFER, fbo.as_ref());
        // js! {
        //     @{gl}.bindFramebuffer(@{WebGlRenderingContext::FRAMEBUFFER}, @{&fbo}.wrap);
        // }

        let fb_type = WebGlRenderingContext::FRAMEBUFFER;
        let tex_target = WebGlRenderingContext::TEXTURE_2D;
        let color_attachment = WebGlRenderingContext::COLOR_ATTACHMENT0;

        gl.framebuffer_texture_2d(
            fb_type,
            color_attachment,
            tex_target,
            Some(&texture.handle),
            0,
        );

        if rb.is_some() {
            let rb_type = WebGlRenderingContext::RENDERBUFFER;
            let depth_attachment = WebGlRenderingContext::DEPTH_ATTACHMENT;

            gl.framebuffer_renderbuffer(
                fb_type,
                depth_attachment,
                rb_type,
                Some(&rb.unwrap().handle),
            );
        };

		gl.bind_framebuffer(WebGlRenderingContext::FRAMEBUFFER, None);
        // js! {
        //     @{gl}.bindFramebuffer(@{WebGlRenderingContext::FRAMEBUFFER}, null);
        // }

        Ok(Self {
            is_default: false,
            is_tex_destroy: is_tex_destroy,
            handle: fbo,
            width: w,
            height: h,
            color: Some(texture_wrap),
            depth: rb_wrap,
        })
    }

    pub fn new_default(fbo: Option<WebGlFramebuffer>, w: u32, h: u32) -> Self {
        Self {
            is_tex_destroy: false,
            is_default: true,
            handle: fbo,
            color: None,
            depth: None,
            width: w,
            height: h,
        }
    }

    pub fn delete(&self, gl: &WebGlRenderingContext) {
        if let Some(fbo) = &self.handle {
			gl.delete_framebuffer(Some(fbo));
            // js! {
            //     @{gl}.deleteFramebuffer(@{fbo}.wrap);
            // }
        }
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_color_texture(&self) -> Option<&HalTexture> {
        self.color.as_ref()
    }
}
