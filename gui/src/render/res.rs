use std::ops::Drop;

use atom::Atom;

use util::res_mgr::{ Res, ResMap };
use webgl_rendering_context::{WebGLRenderingContext, WebGLTexture};

#[derive(Debug, Clone, Copy)]
pub enum Opacity {
    Opaque,
    Translucent,
    Transparent,

}

#[derive(Debug)]
pub struct TextureRes {
    pub name: Atom,
    pub width: usize,
    pub height: usize,
    pub opacity: Opacity,
    pub compress: usize,
    pub bind: WebGLTexture,
    pub gl: WebGLRenderingContext,
}

impl TextureRes {
    // 创建资源
	pub fn new(key: Atom, width: usize,height: usize, opacity: Opacity, compress: usize, bind: WebGLTexture, gl: WebGLRenderingContext) -> Self{
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
        self.gl.delete_texture(Some(&self.bind));
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