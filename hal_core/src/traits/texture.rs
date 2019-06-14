use std::sync::{Arc};
use common::{PixelFormat};
use traits::context::{Context};

pub trait TextureData {
    type RContext: Context;

    fn update(&self, context: &Arc<Self::RContext>);
}

pub trait Texture {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>) -> Result<<Self::RContext as Context>::ContextTexture, String>;
    
    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    fn get_size(&self) -> (u32, u32);

    fn get_render_format(&self) -> PixelFormat;

    fn is_gen_mipmap(&self) -> bool;

    fn update(&self, data: &Arc<dyn TextureData<RContext = Self::RContext>>);
}