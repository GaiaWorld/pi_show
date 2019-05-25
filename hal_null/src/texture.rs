use hal_core::{PixelFormat, TextureData, Texture};

pub struct NullTextureImpl {
    
}

impl Texture for NullTextureImpl {
    
    fn get_size(&self) -> (u32, u32) {
        (0, 0)    
    }

    fn get_render_format(&self) -> PixelFormat {
        PixelFormat::RGB
    }

    fn is_gen_mipmap(&self) -> bool {
        false
    }

    fn update(&self, _x: u32, _y: u32, _width: u32, _height: u32, _data: &TextureData) {

    }
}

impl Drop for NullTextureImpl {
    fn drop(&mut self) {
    }
}

impl AsRef<Self> for NullTextureImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}
