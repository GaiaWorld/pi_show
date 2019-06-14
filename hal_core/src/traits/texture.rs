use std::sync::{Arc};
use common::{PixelFormat, DataFormat};
use traits::context::{Context};

pub trait TextureData {
    type RContext: Context;

    fn update(&self, context: &Arc<Self::RContext>);
}

pub trait Texture {
    type RContext: Context;
    
    fn new_2d(context: &Arc<Self::RContext>, width: u32, height: u32, pformat: PixelFormat, dformat: DataFormat, is_gen_mipmap: bool, data: Option<&dyn TextureData<RContext = Self::RContext>>) -> Result<<Self::RContext as Context>::ContextTexture, String>;

    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    fn get_size(&self) -> (u32, u32);

    fn get_render_format(&self) -> PixelFormat;

    fn is_gen_mipmap(&self) -> bool;

    fn update(&self, mipmap_level: u32, data: &dyn TextureData<RContext = Self::RContext>);
}