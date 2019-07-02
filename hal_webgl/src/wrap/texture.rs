use hal_core::{PixelFormat, DataFormat, Texture, TextureData};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlab, GLSlot, convert_to_mut};
use implement::{WebGLTextureImpl};

#[derive(Clone)]
pub struct WebGLTextureWrap(GLSlot);

impl Texture for WebGLTextureWrap {
    type RContext = WebGLContextWrap;
    
    fn new_2d(context: &Self::RContext, width: u32, height: u32, pformat: PixelFormat, dformat: DataFormat, is_gen_mipmap: bool, data: Option<TextureData<Self::RContext>>) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }

    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }

    fn get_render_format(&self) -> PixelFormat {
        PixelFormat::RGB
    }

    fn is_gen_mipmap(&self) -> bool {
        false
    }

    fn update(&self, mipmap_level: u32, data: &TextureData<Self::RContext>) {

    }
}