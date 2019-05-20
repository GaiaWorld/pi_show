use std::ops::Drop;

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
    pub gl: C,
}

// impl<> fmt::Debug for Point {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "TextureRes {{ name: {:?}, width: {}, height: {}, opacity:{:?}, compress: {:?} }}", self.name, self.width, self.height, self.opacity, self.compress)
//     }
// }

impl<C: Context> TextureRes<C> {
    // 创建资源
	pub fn new(key: Atom, width: usize,height: usize, opacity: Opacity, compress: Compress, bind: C::ContextTexture, gl: C) -> Self{
        TextureRes {
            name: key,
            width: width,
            height: height,
            opacity: opacity,
            compress: compress,
            bind: bind,
            gl: gl,
        }
    }
}
impl<C: Context> Res for TextureRes<C> {
	// 创建资源
	fn name(&self) -> &Atom{
        &self.name
    }
}

impl<C: Context> Drop for TextureRes<C> {
    fn drop(&mut self) {
        // self.gl.delete_texture(Some(&self.bind));
        // Unbind channels TODO
    }
}

pub struct ResMgr<C: Context> {
    pub textures: ResMap<TextureRes<C>>,
}

impl<C: Context> ResMgr<C> {
    pub fn new() -> Self{
        ResMgr{
            textures: ResMap::new(),
        }
    }
}

unsafe impl<C: Context + Sync> Sync for TextureRes<C> {}
unsafe impl<C: Context + Send> Send for TextureRes<C> {}