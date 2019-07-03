use hal_core::{Context, RenderBuffer, RenderTarget, PixelFormat, DataFormat};
use wrap::context::{WebGLContextWrap};
use wrap::texture::{WebGLTextureWrap};
use wrap::gl_slab::{GLSlot, convert_to_mut};
use implement::{WebGLRenderBufferImpl, WebGLRenderTargetImpl};

#[derive(Clone)]
pub struct WebGLRenderBufferWrap(GLSlot<WebGLRenderBufferImpl>);

impl RenderBuffer for WebGLRenderBufferWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, w: u32, h: u32, pformat: PixelFormat) -> Result<<Self::RContext as Context>::ContextRenderBuffer, String> {
        match WebGLRenderBufferImpl::new(&context.rimpl, w, h, pformat) {
            Err(s) => Err(s),
            Ok(rb) => Ok(Self(GLSlot::new(&context.render_buffer, rb))),
        }
    }
    
    fn delete(&self) {
        let slab = convert_to_mut(self.0.slab.as_ref());
        let mut rb = slab.remove(self.0.index);
        rb.delete();
    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    fn get_size(&self) -> Option<(u32, u32)> {
         match self.0.slab.get(self.0.index) {
            None => None,
            Some(rb) => Some(rb.get_size()),
        }
    }
}

#[derive(Clone)]
pub struct WebGLRenderTargetWrap(GLSlot<WebGLRenderTargetImpl>);

impl WebGLRenderTargetWrap {
    pub fn new(slot: GLSlot<WebGLRenderTargetImpl>) -> Self {
        Self(slot)
    }
}

impl RenderTarget for WebGLRenderTargetWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, w: u32, h: u32, pformat: PixelFormat, dformat: DataFormat, has_depth: bool) -> Result<<Self::RContext as Context>::ContextRenderTarget, String> {
        match WebGLRenderTargetImpl::new(&context.rimpl, w, h, pformat, dformat, has_depth) {
            Err(s) => Err(s),
            Ok(rt) => Ok(Self(GLSlot::new(&context.render_target, rt))),
        }
    }
    
    fn delete(&self) {
        let slab = convert_to_mut(self.0.slab.as_ref());
        let mut rt = slab.remove(self.0.index);
        rt.delete();
    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    fn get_size(&self) -> Option<(u32, u32)> {
        match self.0.slab.get(self.0.index) {
            None => None,
            Some(rt) => Some(rt.get_size()),
        }
    }

    // TODO
    fn get_color_texture(&self, index: u32) -> Option<<<Self as RenderTarget>::RContext as Context>::ContextTexture> {
        None
    }
}