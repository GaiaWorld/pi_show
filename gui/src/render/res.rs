
use atom::Atom;
use hal_core::{Context};

use util::res_mgr::{ Res, ResMap};
// use webgl_rendering_context::{WebGLRenderingContext, WebGLTexture};

#[derive(Debug, Clone, Copy)]
pub enum Opacity {
    Opaque,
    Translucent,
    Transparent,
}

#[derive(Debug)]
pub enum Compress {
    None,
    DXT1, // s3tc DXT1 适用于不具有透明度或者仅具有一位Alpha的贴图
    DXT2,
    DXT3,
    DXT4,
    DXT5,
    ATCRGB,
    ATCRGBA,
    PVRTCRGB,
    PVRTCRGBA,
    ETC1,//(RGB)
    ETC2RGB,
    ETC2RGBA,
    ASTC
}

#[derive(Debug)]
pub struct TextureRes<C: Context> {
    pub name: Atom,
    pub width: usize,
    pub height: usize,
    pub opacity: Opacity,
    pub compress: Compress,
    pub bind: C::ContextTexture,
}

// impl<> fmt::Debug for Point {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "TextureRes {{ name: {:?}, width: {}, height: {}, opacity:{:?}, compress: {:?} }}", self.name, self.width, self.height, self.opacity, self.compress)
//     }
// }

impl<C: Context> TextureRes<C> {
    // 创建资源
	pub fn new(key: Atom, width: usize,height: usize, opacity: Opacity, compress: Compress, bind: C::ContextTexture) -> Self{
        TextureRes {
            name: key,
            width: width,
            height: height,
            opacity: opacity,
            compress: compress,
            bind: bind,
        }
    }
}
impl<C: Context> Res for TextureRes<C> {
    type Key = Atom;
	// 创建资源
	fn name(&self) -> &Self::Key{
        &self.name
    }
}

impl<C: Context> AsRef<<C as Context>::ContextTexture> for TextureRes<C> {
    fn as_ref(&self) -> &<C as Context>::ContextTexture{
        &self.bind
    }
}

#[derive(Debug)]
pub struct SamplerRes<C: Context> {
    pub name: u64,
    pub bind: C::ContextSampler,
}

impl<C: Context> SamplerRes<C> {
    // 创建资源
	pub fn new(key: u64, bind: C::ContextSampler) -> Self{
        SamplerRes {
            name: key,
            bind: bind,
        }
    }
}

impl<C: Context> Res for SamplerRes<C> {
    type Key = u64;
	// 创建资源
	fn name(&self) -> &Self::Key{
        &self.name
    }
}

impl<C: Context> AsRef<<C as Context>::ContextSampler> for SamplerRes<C> {
    fn as_ref(&self) -> &<C as Context>::ContextSampler{
        &self.bind
    }
}

pub struct ResMgr<C: Context> {
    pub textures: ResMap<TextureRes<C>>,
    pub samplers: ResMap<SamplerRes<C>>,
}

impl<C: Context> ResMgr<C> {
    pub fn new() -> Self{
        ResMgr{
            textures: ResMap::new(),
            samplers: ResMap::new(),
        }
    }
}

unsafe impl<C: Context + Sync> Sync for TextureRes<C> {}
unsafe impl<C: Context + Send> Send for TextureRes<C> {}