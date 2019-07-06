use hal_core::{Context, Texture, RenderBuffer, RenderTarget, PixelFormat, DataFormat};
use wrap::context::{WebGLContextWrap};
use wrap::texture::{WebGLTextureWrap};
use wrap::gl_slab::{GLSlot, convert_to_mut};
use implement::{WebGLRenderBufferImpl, WebGLRenderTargetImpl};

#[derive(Clone)]
pub struct WebGLRenderBufferWrap(pub GLSlot<WebGLRenderBufferImpl>);

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
pub struct WebGLRenderTargetWrap {
    pub slot: GLSlot<WebGLRenderTargetImpl>,
    
    is_default: bool,
    texture: Option<WebGLTextureWrap>,
    depth: Option<WebGLRenderBufferWrap>,
}

impl WebGLRenderTargetWrap {
    pub fn new_default(slot: GLSlot<WebGLRenderTargetImpl>) -> Self {
        Self {
            slot: slot,
            is_default: true,
            texture: None,
            depth: None,
        }
    }
}

impl RenderTarget for WebGLRenderTargetWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, w: u32, h: u32, pformat: PixelFormat, dformat: DataFormat, has_depth: bool) -> Result<<Self::RContext as Context>::ContextRenderTarget, String> {

        let texture = WebGLTextureWrap::new_2d(context, 0, w, h, pformat, dformat, false, None);
        if let Err(s) = texture {
            return Err(s.to_string());
        }
        let texture = texture.unwrap();

        let mut r = None;
        let rb = if has_depth {
            let temp = WebGLRenderBufferWrap::new(context, w, h, PixelFormat::DEPTH16);
            if let Err(s) = temp {
                return Err(s.to_string());
            }
            let tp = temp.unwrap();
            r = Some(tp.0.clone());
            Some(tp)
        } else {
            None
        };
        
        match WebGLRenderTargetImpl::new(&context.rimpl, w, h, &texture.0, r.as_ref()) {
            Err(s) => Err(s),
            Ok(rt) => Ok(Self {
                slot: GLSlot::new(&context.render_target, rt),
                is_default: false,
                texture: Some(texture),
                depth: rb,
            })
        }
    }
    
    fn delete(&self) {
        let slab = convert_to_mut(self.slot.slab.as_ref());
        let rt = slab.remove(self.slot.index);
        rt.delete();
    }

    fn get_id(&self) -> u64 {
        self.slot.index as u64
    }

    fn get_size(&self) -> Option<(u32, u32)> {
        match self.slot.get_mut() {
            None => None,
            Some(rt) => Some(rt.get_size()),
        }
    }

    fn get_color_texture(&self, _index: u32) -> Option<<<Self as RenderTarget>::RContext as Context>::ContextTexture> {
        match &self.texture {
            None => None,
            Some(texture) => Some(texture.clone()),
        }
    }
}