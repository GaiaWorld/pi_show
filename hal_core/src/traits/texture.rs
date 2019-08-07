use common::{PixelFormat};

/** 
 * 纹理
 */

pub enum TextureData<'a> {
    None,
    U8(&'a [u8]),
    F32(&'a [f32]),
}

pub trait Texture: Drop + AsRef<Self> {

    fn extend(&self, width: u32, height: u32) -> bool;

    fn get_size(&self) -> (u32, u32);

    fn get_render_format(&self) -> PixelFormat;

    fn is_gen_mipmap(&self) -> bool;

    fn update(&self, x: u32, y: u32, width: u32, height: u32, data: &TextureData);
}