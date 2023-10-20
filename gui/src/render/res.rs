use std::cell::RefCell;
/**
 * 定义gl资源
*/
use std::ops::{Deref, DerefMut};

use hal_core::*;

use pi_atom::Atom;
use res::Res;
use share::Share;

use crate::{single::dyn_texture::DynAtlasSet};
use crate::component::user::Aabb2;
use crate::render::engine::UnsafeMut;

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

// 纹理的部分资源
pub struct TexturePartRes {
	index: usize,
	dyn_texture_set: Share<RefCell<DynAtlasSet>>,
}

impl TexturePartRes {
	pub fn new(index: usize, dyn_texture_set: Share<RefCell<DynAtlasSet>>) -> Self {
		TexturePartRes {
			index, dyn_texture_set
		}
	}

	pub fn cost(&self) -> usize {
		let dyn_texture_set = self.dyn_texture_set.borrow_mut();
		let rect =dyn_texture_set.get_rect(self.index).unwrap();
		(rect.maxs.y - rect.mins.y) as usize * (rect.maxs.x - rect.mins.x) as usize * 4
	}

	pub fn get_uv(&self) -> Aabb2 {
		let dyn_texture_set = self.dyn_texture_set.borrow_mut();
		return dyn_texture_set.get_uv(self.index).unwrap()
	}

	pub fn get_rect(&self) -> Aabb2 {
		let dyn_texture_set = self.dyn_texture_set.borrow_mut();
		return dyn_texture_set.get_rect(self.index).unwrap().clone()
	}

	pub fn size(&self) -> (usize, usize) {
		let set = self.dyn_texture_set.borrow_mut();
		let texture = set.get_texture(self.index).unwrap();
		return (texture.width, texture.height)
	}

	pub fn index(&self) -> usize {
		self.index
	}

	pub fn get_dyn_texture_set(&self) -> &Share<RefCell<DynAtlasSet>> {
		&self.dyn_texture_set
	}
}

impl Drop for TexturePartRes {
	fn drop(&mut self) {
		self.dyn_texture_set.borrow_mut().delete_rect(self.index);
	}
}

// impl Deref for TexturePartRes {
// 	type Target = HalTexture;
// 	fn deref(&self) -> &Self::Target {
// 		self.dyn_texture_set.borrow_mut().get_texture(self.index).unwrap()
// 	}
// }

impl Res for TexturePartRes {
    type Key = u64;
}

unsafe impl Send for TexturePartRes {}
unsafe impl Sync for TexturePartRes {}

#[derive(Deref)]
pub struct RenderBufferRes(HalRenderBuffer);

impl RenderBufferRes {
	pub fn new(res: HalRenderBuffer) -> RenderBufferRes{
		return RenderBufferRes(res);
	}
}

impl Res for RenderBufferRes {
    type Key = u64;
}

unsafe impl Send for RenderBufferRes {}
unsafe impl Sync for RenderBufferRes {}

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
    type Key = Atom;
}

unsafe impl Send for TextureRes {}
unsafe impl Sync for TextureRes {}

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
unsafe impl Send for SamplerRes {}
unsafe impl Sync for SamplerRes {}

impl Res for GeometryRes {
    type Key = u64;
}
unsafe impl Send for GeometryRes {}
unsafe impl Sync for GeometryRes {}

impl Res for RasterStateRes {
    type Key = u64;
}
unsafe impl Send for RasterStateRes {}
unsafe impl Sync for RasterStateRes {}

impl Res for BlendStateRes {
    type Key = u64;
}
unsafe impl Send for BlendStateRes {}
unsafe impl Sync for BlendStateRes {}

impl Res for StencilStateRes {
    type Key = u64;
}
unsafe impl Send for StencilStateRes {}
unsafe impl Sync for StencilStateRes {}

impl Res for DepthStateRes {
    type Key = u64;
}
unsafe impl Send for DepthStateRes {}
unsafe impl Sync for DepthStateRes {}

impl Res for BufferRes {
    type Key = u64;
}
unsafe impl Send for BufferRes {}
unsafe impl Sync for BufferRes {}
