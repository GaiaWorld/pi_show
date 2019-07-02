use std::sync::{Arc};
use hal_core::{PixelFormat, DataFormat, TextureData};
use wrap::{WebGLContextWrap};

pub struct WebGLTextureImpl {

}

impl WebGLTextureImpl {
    pub fn new_2d(context: &Arc<WebGLContextWrap>, width: u32, height: u32, pformat: PixelFormat, dformat: DataFormat, is_gen_mipmap: bool, data: Option<TextureData<WebGLContextWrap>>) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }

    pub fn delete(&self) {

    }

    pub fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }

    pub fn get_render_format(&self) -> PixelFormat {
        PixelFormat::RGB
    }

    pub fn is_gen_mipmap(&self) -> bool {
        false
    }

    pub fn update(&self, mipmap_level: u32, data: &TextureData<WebGLContextWrap>) {

    }
}