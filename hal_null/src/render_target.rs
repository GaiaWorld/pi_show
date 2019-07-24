use share::Share;
use hal_core::{Context, RenderTarget, RenderBuffer};
use context::{NullContextImpl};
use texture::NullTextureImpl;

pub struct NullRenderBufferImpl {
    
}

pub struct NullRenderTargetImpl {
    
}

impl RenderBuffer for NullRenderBufferImpl {
    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }
}

impl Drop for NullRenderBufferImpl {
    fn drop(&mut self) {
    }
}

impl AsRef<Self> for NullRenderBufferImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl RenderTarget for NullRenderTargetImpl {
    
    type RContext = NullContextImpl;

    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }

    fn get_color_texture(&self, _index: u32) -> Option<Share<<<Self as RenderTarget>::RContext as Context>::ContextTexture>> {
        Some(Share::new(NullTextureImpl{}))
    }
}

impl Drop for NullRenderTargetImpl {
    fn drop(&mut self) {
    }
}

impl AsRef<Self> for NullRenderTargetImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}
