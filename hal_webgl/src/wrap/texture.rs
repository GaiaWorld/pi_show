use hal_core::{PixelFormat, DataFormat, Texture, TextureData};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlot, convert_to_mut};
use implement::{WebGLTextureImpl};

#[derive(Clone)]
pub struct WebGLTextureWrap(pub GLSlot<WebGLTextureImpl>);

impl Texture for WebGLTextureWrap {
    type RContext = WebGLContextWrap;
    
    fn new_2d(context: &Self::RContext, mipmap_level: u32, width: u32, height: u32, pformat: PixelFormat, dformat: DataFormat, is_gen_mipmap: bool, data: Option<TextureData<Self::RContext>>) -> Result<Self, String> {
        match WebGLTextureImpl::new_2d(&context.rimpl, mipmap_level, width, height, pformat, dformat, is_gen_mipmap, data) {
            Err(s) => Err(s),
            Ok(texture) => {
                let slot = GLSlot::new(&context.texture, texture);
                Ok(Self(slot))
            }
        }
    }

    fn delete(&self) {
        let slab = convert_to_mut(self.0.slab.as_ref());
        let mut sampler = slab.remove(self.0.index);
        sampler.delete();
    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    fn get_size(&self) -> Option<(u32, u32)> {
        match self.0.slab.get(self.0.index) {
            None => None,
            Some(texture) => Some(texture.get_size()),
        }
    }

    fn get_render_format(&self) -> Option<PixelFormat> {
        match self.0.slab.get(self.0.index) {
            None => None,
            Some(texture) => Some(texture.get_render_format()),
        }
    }

    fn is_gen_mipmap(&self) -> bool {
        match self.0.slab.get(self.0.index) {
            None => false,
            Some(texture) => texture.is_gen_mipmap(),
        }
    }

    fn update(&self, mipmap_level: u32, data: &TextureData<Self::RContext>) {
        let slab = convert_to_mut(self.0.slab.as_ref());
        match slab.get_mut(self.0.index) {
            None => {},
            Some(texture) => texture.update(mipmap_level, data),
        }
    }
}