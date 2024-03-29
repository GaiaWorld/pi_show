use convert::*;
use hal_core::{HalRenderBuffer, HalTexture, PixelFormat};
use stdweb::unstable::TryInto;
use stdweb::Object;
use texture::WebGLTextureImpl;
use webgl_rendering_context::{WebGLRenderbuffer, WebGLRenderingContext};

pub struct WebGLRenderBufferImpl {
    width: u32,
    height: u32,
    _format: PixelFormat,
    handle: WebGLRenderbuffer,
}

impl WebGLRenderBufferImpl {
    pub fn new(
        gl: &WebGLRenderingContext,
        w: u32,
        h: u32,
        pformat: PixelFormat,
    ) -> Result<Self, String> {
        let r = gl.create_renderbuffer();
        if r.is_none() {
            return Err("WebGLRenderBufferImpl new failed".to_string());
        }

        let r = r.unwrap();
        gl.bind_renderbuffer(WebGLRenderingContext::RENDERBUFFER, Some(&r));

        let format = get_pixel_format(pformat);
        gl.renderbuffer_storage(
            WebGLRenderingContext::RENDERBUFFER,
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

    pub fn delete(&self, gl: &WebGLRenderingContext) {
        gl.delete_renderbuffer(Some(&self.handle))
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

pub struct WebGLRenderTargetImpl {
    pub is_default: bool, // 注：不能从默认的渲染目标上取color depth
    pub handle: Option<Object>,
    width: u32,
    height: u32,
    pub is_tex_destroy: bool,
    pub color: Option<HalTexture>,
    pub depth: Option<HalRenderBuffer>,
}

impl WebGLRenderTargetImpl {
    pub fn new(
        gl: &WebGLRenderingContext,
        w: u32,
        h: u32,
        texture: &WebGLTextureImpl,
        rb: Option<&WebGLRenderBufferImpl>,
        texture_wrap: HalTexture,
        rb_wrap: Option<HalRenderBuffer>,
        is_tex_destroy: bool,
    ) -> Result<Self, String> {
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

        let fb_type = WebGLRenderingContext::FRAMEBUFFER;
        let tex_target = WebGLRenderingContext::TEXTURE_2D;
        let color_attachment = WebGLRenderingContext::COLOR_ATTACHMENT0;

        gl.framebuffer_texture2_d(
            fb_type,
            color_attachment,
            tex_target,
            Some(&texture.handle),
            0,
        );

        if rb.is_some() {
            let rb_type = WebGLRenderingContext::RENDERBUFFER;
            let depth_attachment = WebGLRenderingContext::DEPTH_ATTACHMENT;

            gl.framebuffer_renderbuffer(
                fb_type,
                depth_attachment,
                rb_type,
                Some(&rb.unwrap().handle),
            );
        };

        js! {
            @{gl}.bindFramebuffer(@{WebGLRenderingContext::FRAMEBUFFER}, null);
        }

        Ok(Self {
            is_default: false,
            is_tex_destroy: is_tex_destroy,
            handle: Some(fbo),
            width: w,
            height: h,
            color: Some(texture_wrap),
            depth: rb_wrap,
        })
    }

    pub fn new_default(fbo: Option<Object>, w: u32, h: u32) -> Self {
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

    pub fn delete(&self, gl: &WebGLRenderingContext) {
        if let Some(fbo) = &self.handle {
            js! {
                @{gl}.deleteFramebuffer(@{fbo}.wrap);
            }
        }
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_color_texture(&self) -> Option<&HalTexture> {
        self.color.as_ref()
    }
}
