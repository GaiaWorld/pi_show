use std::sync::{Arc};
use hal_core::{PixelFormat, DataFormat, TextureData};
use wrap::{WebGLContextWrap};

pub struct WebGLTextureImpl {

}

impl WebGLTextureImpl {
    fn new_2d(context: &Arc<WebGLContextWrap>, width: u32, height: u32, pformat: PixelFormat, dformat: DataFormat, is_gen_mipmap: bool, data: Option<TextureData<WebGLContextWrap>>) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }

    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
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

    fn update(&self, mipmap_level: u32, data: &TextureData<WebGLContextWrap>) {

    }
}