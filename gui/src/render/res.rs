/**
 * 定义gl资源
*/
use std::ops::{Deref, DerefMut};

use hal_core::*;

use res::Res;
use share::Share;
// use webgl_rendering_context::{WebGLRenderingContext, WebGLTexture};

#[derive(Debug, Clone, Copy)]
pub enum Opacity {
    Opaque,
    Translucent,
    Transparent,
}

// #[derive(Debug)]
// pub enum Compress {
//     None,
//     DXT1, // s3tc DXT1 适用于不具有透明度或者仅具有一位Alpha的贴图
//     DXT2,
//     DXT3,
//     DXT4,
//     DXT5,
//     ATCRGB,
//     ATCRGBA,
//     PVRTCRGB,
//     PVRTCRGBA,
//     ETC1, //(RGB)
//     ETC2RGB,
//     ETC2RGBA,
//     ASTC,
// }

pub struct TextureRes {
    pub width: usize,
    pub height: usize,
    pub pformat: PixelFormat,
    pub dformat: DataFormat,
    pub opacity: Opacity,
    pub compress: Option<CompressedTexFormat>,
	pub cost: Option<usize>,
    pub bind: HalTexture,
}

// 强行实现Send、Sync 修改?TODO
unsafe impl Send for TextureRes {}
unsafe impl Sync for TextureRes {}

// 强行实现Send、Sync 修改?TODO
unsafe impl Send for SamplerRes {}
unsafe impl Sync for SamplerRes {}

// impl<> fmt::Debug for Point {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "TextureRes {{ name: {:?}, width: {}, height: {}, opacity:{:?}, compress: {:?} }}", self.name, self.width, self.height, self.opacity, self.compress)
//     }
// }

impl TextureRes {
    // 创建资源
    pub fn new(
        width: usize,
        height: usize,
        pformat: PixelFormat,
        dformat: DataFormat,
        opacity: Opacity,
        compress: Option<CompressedTexFormat>,
        bind: HalTexture,
		cost: Option<usize>,
    ) -> Self {
        TextureRes {
            width,
            height,
            pformat,
            dformat,
            opacity,
            compress,
            bind,
			cost
        }
    }

    pub fn update_size(&self, width: usize, height: usize) {
        let s = unsafe { &mut *(self as *const Self as *mut Self) };
        s.width = width;
        s.height = height;
    }
}

impl Res for TextureRes {
    type Key = usize;
}

#[derive(Deref, DerefMut)]
pub struct SamplerRes(pub HalSampler);

#[derive(Deref, DerefMut)]
pub struct ProgramRes(pub HalProgram);

#[derive(Deref, DerefMut)]
pub struct RasterStateRes(pub HalRasterState);

#[derive(Deref, DerefMut)]
pub struct BlendStateRes(pub HalBlendState);

#[derive(Deref, DerefMut)]
pub struct StencilStateRes(pub HalStencilState);

#[derive(Deref, DerefMut)]
pub struct DepthStateRes(pub HalDepthState);

#[derive(Deref, DerefMut)]
pub struct BufferRes(pub HalBuffer);

pub struct GeometryRes {
    pub geo: HalGeometry,
    pub buffers: Vec<Share<BufferRes>>,
}

impl Deref for GeometryRes {
    type Target = HalGeometry;
    fn deref(&self) -> &Self::Target {
        &self.geo
    }
}

impl DerefMut for GeometryRes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.geo
    }
}

impl Res for SamplerRes {
    type Key = u64;
}

// 强行实现Send、Sync 修改?TODO
unsafe impl Send for GeometryRes {}
unsafe impl Sync for GeometryRes {}

impl Res for GeometryRes {
    type Key = u64;
}

// 强行实现Send、Sync 修改?TODO
unsafe impl Send for RasterStateRes {}
unsafe impl Sync for RasterStateRes {}
impl Res for RasterStateRes {
    type Key = u64;
}

// 强行实现Send、Sync 修改?TODO
unsafe impl Send for BlendStateRes {}
unsafe impl Sync for BlendStateRes {}
impl Res for BlendStateRes {
    type Key = u64;
}

// 强行实现Send、Sync 修改?TODO
unsafe impl Send for StencilStateRes {}
unsafe impl Sync for StencilStateRes {}
impl Res for StencilStateRes {
    type Key = u64;
}

// 强行实现Send、Sync 修改?TODO
unsafe impl Send for DepthStateRes {}
unsafe impl Sync for DepthStateRes {}
impl Res for DepthStateRes {
    type Key = u64;
}

// 强行实现Send、Sync 修改?TODO
unsafe impl Send for BufferRes {}
unsafe impl Sync for BufferRes {}
impl Res for BufferRes {
    type Key = u64;
}
