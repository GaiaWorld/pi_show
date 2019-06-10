use std::sync::{Arc};
use common::{PixelFormat};
use traits::context::{Context};

/** 
 * 纹理
 */

pub enum TextureData<'a> {
    None,
    U8(&'a [u8]),
    F32(&'a [f32]),
}

pub trait Texture {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>) -> Self;
    fn delete(&self);

    fn get_size(&self) -> (u32, u32);

    fn get_render_format(&self) -> PixelFormat;

    fn is_gen_mipmap(&self) -> bool;

    fn update(&self, x: u32, y: u32, width: u32, height: u32, data: &TextureData);
}