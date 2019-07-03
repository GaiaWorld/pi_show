use common::{PixelFormat, DataFormat};
use traits::context::{Context};

pub trait CustomTextureData {
    type RContext: Context;

    fn update(&self, texture: &<Self::RContext as Context>::ContextTexture);
}

pub enum TextureData<'a, C: Context> {
    F32(u32, u32, u32, u32, &'a[f32]),  // (x, y, w, h, data)
    U8(u32, u32, u32, u32, &'a[u8]),   // (x, y, w, h, data)
    Custom(Box<dyn CustomTextureData<RContext = C>>),
}

pub trait Texture : Sized + Clone {
    type RContext: Context;
    
    fn new_2d(context: &Self::RContext, width: u32, height: u32, pformat: PixelFormat, dformat: DataFormat, is_gen_mipmap: bool, data: Option<TextureData<Self::RContext>>) -> Result<<Self::RContext as Context>::ContextTexture, String>;

    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    fn get_size(&self) -> Option<(u32, u32)>;

    fn get_render_format(&self) -> Option<PixelFormat>;

    fn is_gen_mipmap(&self) -> bool;

    fn update(&self, mipmap_level: u32, data: &TextureData<Self::RContext>);
}