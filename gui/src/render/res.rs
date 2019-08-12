
use atom::Atom;
use hal_core::*;

use share::Share;
use render::res_mgr::*;
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

pub struct TextureRes {
    pub width: usize,
    pub height: usize,
    pub opacity: Opacity,
    pub compress: Compress,
    pub bind: HalTexture,
}

// impl<> fmt::Debug for Point {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "TextureRes {{ name: {:?}, width: {}, height: {}, opacity:{:?}, compress: {:?} }}", self.name, self.width, self.height, self.opacity, self.compress)
//     }
// }

impl TextureRes {
    // 创建资源
	pub fn new(width: usize, height: usize, opacity: Opacity, compress: Compress, bind: HalTexture) -> Self{
        TextureRes {
            width: width,
            height: height,
            opacity: opacity,
            compress: compress,
            bind: bind,
        }
    }
}

impl Res for TextureRes {
    type Key = Atom;
}

pub type SamplerRes = HalSampler;

pub type ProgramRes = HalProgram;

impl Res for SamplerRes {
    type Key = u64;
}

pub struct GeometryRes {
    pub geo: HalGeometry,
    pub buffers: Vec<Share<HalBuffer>>,
}

impl Res for GeometryRes {
    type Key = u64;
}

impl Res for HalRasterState {
    type Key = u64;
}

impl Res for HalBlendState {
    type Key = u64;
}

impl Res for HalStencilState {
    type Key = u64;
}

impl Res for HalDepthState {
    type Key = u64;
}

impl Res for HalBuffer {
    type Key = u64;
}