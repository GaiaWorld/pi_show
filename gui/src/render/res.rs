
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

#[derive(Debug)]
pub struct TextureRes {
    pub width: usize,
    pub height: usize,
    pub opacity: Opacity,
    pub compress: Compress,
    pub bind: Share<HalTexture>,
}

// impl<> fmt::Debug for Point {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "TextureRes {{ name: {:?}, width: {}, height: {}, opacity:{:?}, compress: {:?} }}", self.name, self.width, self.height, self.opacity, self.compress)
//     }
// }

impl TextureRes {
    // 创建资源
	pub fn new(width: usize,height: usize, opacity: Opacity, compress: Compress, bind: Share<HalTexture>) -> Self{
        TextureRes {
            width: width,
            height: height,
            opacity: opacity,
            compress: compress,
            bind: bind,
        }
    }
}

impl<C: HalContext + 'static> Res<C> for TextureRes {
    type Key = Atom;

    fn destroy(&self, gl: &C){
        gl.texture_destroy(HalTexture(self.bind.0, self.bind.1));
    }
}

pub type SamplerRes = HalSampler;

pub type ProgramRes = HalProgram;

impl<C: HalContext + 'static> Res<C> for SamplerRes {
    type Key = u64;

    fn destroy(&self, gl: &C){
        gl.sampler_destroy(HalSampler(self.0, self.1));
    }
}

pub struct GeometryRes {
    pub geo: HalGeometry,
    pub buffers: Vec<Share<HalBuffer>>,
}

impl<C: HalContext + 'static> Res<C> for GeometryRes {
    type Key = u64;

    fn destroy(&self, gl: &C){
        gl.geometry_destroy(HalGeometry(self.geo.0, self.geo.1));
    }
}

impl<C: HalContext + 'static> Res<C> for HalRasterState {
    type Key = u64;

    fn destroy(&self, gl: &C){
        gl.rs_destroy(HalRasterState(self.0, self.1));
    }
}

impl<C: HalContext + 'static> Res<C> for HalBlendState {
    type Key = u64;

    fn destroy(&self, gl: &C){
        gl.bs_destroy(HalBlendState(self.0, self.1));
    }
}

impl<C: HalContext + 'static> Res<C> for HalStencilState {
    type Key = u64;

    fn destroy(&self, gl: &C){
        gl.ss_destroy(HalStencilState(self.0, self.1));
    }
}

impl<C: HalContext + 'static> Res<C> for HalDepthState {
    type Key = u64;

    fn destroy(&self, gl: &C){
        gl.ds_destroy(HalDepthState(self.0, self.1));
    }
}

impl<C: HalContext + 'static> Res<C> for HalBuffer {
    type Key = u64;

    fn destroy(&self, gl: &C){
        gl.buffer_destroy(HalBuffer(self.0, self.1));
    }
}