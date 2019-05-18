use hal_core::{Texture, PixelFormat};

pub struct WebGLTextureImpl {
    
}

impl Texture for WebGLTextureImpl {
    
    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }

    fn get_render_format(&self) -> PixelFormat {
        PixelFormat::RGB
    }

    fn is_gen_mipmap(&self) -> bool {
        false
    }

    fn update(&self, _x: u32, _y: u32, _width: u32, _height: u32, _data: &[u8]) {

    }
}