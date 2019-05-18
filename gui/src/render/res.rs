use std::ops::Drop;
use std::rc::Rc;
use std::fmt::Debug;

use atom::Atom;

use util::res_mgr::{ Res, ResMap };
// use webgl_rendering_context::{WebGLRenderingContext, WebGLTexture};

pub trait Context: Debug{}
pub trait Texture: Debug{}

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
pub struct TextureRes {
    pub name: Atom,
    pub width: usize,
    pub height: usize,
    pub opacity: Opacity,
    pub compress: Compress,
    pub bind: Rc<Texture>,
    pub gl: Rc<Context>,
}

impl TextureRes {
    // 创建资源
	pub fn new(key: Atom, width: usize,height: usize, opacity: Opacity, compress: Compress, bind: Rc<Texture>, gl: Rc<Context>) -> Self{
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
impl Res for TextureRes {
	// 创建资源
	fn name(&self) -> &Atom{
        &self.name
    }
}

impl Drop for TextureRes {
    fn drop(&mut self) {
        // self.gl.delete_texture(Some(&self.bind));
        // Unbind channels TODO
    }
}

pub struct ResMgr {
    pub textures: ResMap<TextureRes>,
}

impl ResMgr {
    pub fn new() -> ResMgr{
        ResMgr{
            textures: ResMap::new(),
        }
    }
}