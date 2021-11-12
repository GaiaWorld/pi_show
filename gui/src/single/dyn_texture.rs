use std::hash::{Hash, Hasher};
use std::cell::RefCell;

use atom::Atom;
use guillotiere::*;
use slab::Slab;
use hal_core::*;
use res::{ResMap, ResMgr};
use share::{Share, ShareWeak};
use hash::DefaultHasher;

use crate::component::user::*;
use crate::render::engine::UnsafeMut;
use crate::render::res::{Opacity, TextureRes};
use crate::single::class::*;
use std::collections::BTreeMap;

lazy_static! {
    pub static ref DYN_TEXTURE: Atom = Atom::from("DYN_TEXTURE");
}
pub struct RectIndex {
	allocation: Allocation,
	allocation_index: usize,
}

impl RectIndex {
	pub fn new(allocation: Allocation, allocation_index: usize) -> Self {
		Self{
			allocation,
			allocation_index,
		}
	}
}

pub struct DynAtlas {
	allocator : AtlasAllocator,
	target: HalRenderTarget,
	texture: Share<TextureRes>,
	count: usize,
	pformat: PixelFormat,
	dformat: DataFormat,
}

pub struct DynAtlasSet {
	dyn_atlas : Slab<DynAtlas>,
	rects: Slab<RectIndex>,
	texture_res_map: UnsafeMut<ResMap<TextureRes>>,
	texture_cur_index: usize,
	unuse_texture: Vec<UnuseTexture>
}

struct UnuseTexture {
	weak: ShareWeak<TextureRes>,
	pformat: PixelFormat,
	dformat: DataFormat,
	width: u32,
	height: u32,
}

impl DynAtlasSet {
	pub fn new(res_mgr: Share<RefCell<ResMgr>>) -> DynAtlasSet {
		let res_mgr_ref = res_mgr.borrow();
		let texture_res_map = UnsafeMut::new(res_mgr_ref.fetch_map::<TextureRes>(0).unwrap());
		DynAtlasSet {
			dyn_atlas: Slab::new(),
			rects: Slab::new(),
			texture_res_map: texture_res_map,
			texture_cur_index: 0,
			unuse_texture: Vec::new(),
		}
	}

	/// 添加矩形
	pub fn add_rect<C: HalContext>(&mut self, exclude: usize, width: f32, height: f32, pformat: PixelFormat, dformat: DataFormat, gl: &mut C) -> usize {
		for (index, dyn_atlas) in self.dyn_atlas.iter_mut() {
			if exclude == index { // 不能分配在exclude上
				continue;
			}

			if dyn_atlas.pformat != pformat || dyn_atlas.dformat != dformat {
				continue;
			}

			match dyn_atlas.allocator.allocate(guillotiere::Size::new(width as i32, height as i32)) {
				Some(allocation) => {
					dyn_atlas.count += 1;
					let index = self.rects.insert(RectIndex::new(allocation, index));
					return index;
				},
				None => (),
			};
		}

		let mut w: i32 = 1024.max(width.ceil() as i32);
		let mut h: i32 = 1024.max(height.ceil() as i32);

		if pformat == PixelFormat::RGB {// 目前RGB是用来绘制maskimage的渐变色，不需要很大的fbo，（改为外部选择大小？TODO）
			w = 100.max(width as i32 * 3);
			h = 100.max(height as i32 * 3);
		}

		// 从缓冲上取到纹理
		let mut catch_texture = None;
		if self.unuse_texture.len() > 0 {
			for i in 0..self.unuse_texture.len() {
				let t = &self.unuse_texture[i];
				if t.pformat == pformat && t.dformat == dformat && t.width >= width.ceil() as u32 && t.height >= height.ceil() as u32 {
					let unuse_texture = self.unuse_texture.swap_remove(i);
					catch_texture = Some(match unuse_texture.weak.upgrade() {
						Some(r) => {
							w = unuse_texture.width as i32;
							h = unuse_texture.height as i32;
							r
						},
						None => continue,
					});
				}
				break;
			}
		}

		let texture_res = match catch_texture {
			Some(r) => r,
			None => {
				// 如果缓冲上不存在纹理，则重新创建纹理
				let texture = gl.texture_create_2d(
					0, 
					w as u32, 
					h as u32, 
					pformat,
					dformat,
					false, 
					None
				).unwrap();
				log::info!("create fbo texture, index: {}, use_count: {}, w: {}, h: {}", texture.item.index, texture.item.use_count, w, h);
				let mut hasher = DefaultHasher::default();
				DYN_TEXTURE.hash(&mut hasher);
				self.texture_cur_index.hash(&mut hasher);
				pformat.hash(&mut hasher);
				dformat.hash(&mut hasher);
				w.hash(&mut hasher);
				h.hash(&mut hasher);
				self.texture_cur_index += 1;
				let hash = hasher.finish() as usize;
				
				self.texture_res_map.create(hash, TextureRes::new(w as usize, h as usize,pformat, dformat,Opacity::Transparent, None, texture, Some((w * h * 4) as usize)), (w * h * 4) as usize, 0)
			}
		};
		let target = gl.rt_create(
			Some(&texture_res.bind),
			w as u32,
			h as u32,
			pformat,
			dformat,
			true,
		)
		.unwrap();

		let mut atlas_allocator= AtlasAllocator::new(guillotiere::Size::new(w, h));
		let allocation= atlas_allocator.allocate(guillotiere::Size::new(width as i32, height as i32)).unwrap();

		let dyn_atlas_index = self.dyn_atlas.insert(DynAtlas{
			allocator: atlas_allocator,
			target,
			count: 1,
			texture: texture_res,
			pformat: pformat,
			dformat: dformat,
		});
		let index = self.rects.insert(RectIndex::new(allocation.clone(),  dyn_atlas_index));
		index
	}

	// 更新矩形
	// exclude为排除fbo，
	pub fn update_or_add_rect<C: HalContext>(&mut self, old: usize, exclude: usize, width: f32, height: f32, pformat: PixelFormat, dformat: DataFormat, gl: &mut C) -> usize {
		let exclude_allocation_index = match self.rects.get(exclude) {
			Some(r) => r.allocation_index,
			None => 0,
		};
		let allocation_index = if let Some(size) = self.get_rect(old) {
			let old_index = self.rects.get(old).unwrap();
			// 如果大小相同，并且分配的纹理id不等于需要排除的纹理id，则返回0， 表示未更新分配
			
			if exclude_allocation_index != old_index.allocation_index && size.maxs.x - size.mins.x == width && size.maxs.y - size.mins.y == height {
				return 0;
			}

			let rect_index = self.rects.remove(old);
			let dyn_atlas = &mut self.dyn_atlas[rect_index.allocation_index];

			dyn_atlas.allocator.deallocate(rect_index.allocation.id);
			dyn_atlas.count -= 1;
			rect_index.allocation_index
		} else {
			0
		};
		let index = self.add_rect(exclude_allocation_index, width, height, pformat, dformat, gl);

		// 如果释放过某个矩形，且当前存在的纹理大于一张，则释放多余的纹理，最多保留一张
		if allocation_index > 0 && self.dyn_atlas.len() > 1 {
			let dyn_atlas = &mut self.dyn_atlas[allocation_index];
			if dyn_atlas.count == 0 {
				let dyn_atla = self.dyn_atlas.remove(allocation_index);
				let (pformat, dformat, width, height) = (dyn_atla.texture.pformat, dyn_atla.texture.dformat, dyn_atla.texture.width, dyn_atla.texture.height);
				self.unuse_texture.push(UnuseTexture{
					pformat,
					dformat,
					width: width as u32,
					height: height as u32,
					weak: Share::downgrade(&dyn_atla.texture),
				});
			}
		}

		index
	}

	pub fn delete_rect(&mut self, index: usize) -> Option<guillotiere::Size> {
		let r = match self.rects.get(index) {
			Some(_r) => {
				self.rects.remove(index)
			},
			None => return None,
		};
		
		let mut dyn_atlas = &mut self.dyn_atlas[r.allocation_index];
		dyn_atlas.allocator.deallocate(r.allocation.id);
		dyn_atlas.count -= 1;

		// 将纹理缓冲起来
		if dyn_atlas.count == 0 && self.dyn_atlas.len() > 1 {
			let dyn_atla = self.dyn_atlas.remove(r.allocation_index);
			let (pformat, dformat, width, height) = (dyn_atla.texture.pformat, dyn_atla.texture.dformat, dyn_atla.texture.width, dyn_atla.texture.height);
			self.unuse_texture.push(UnuseTexture{
				pformat,
				dformat,
				width: width as u32,
				height: height as u32,
				weak: Share::downgrade(&dyn_atla.texture),
			});
		}
		let r = &r.allocation.rectangle;
		Some(guillotiere::Size::new(r.max.x - r.min.x, r.max.y - r.min.y))
	}

	pub fn get_target(&self, index: usize) -> Option<&HalRenderTarget> {
		match self.rects.get(index) {
			Some(r) => Some(&self.dyn_atlas[r.allocation_index].target),
			None => None,
		}
	}

	pub fn get_texture(&self, index: usize) -> Option<&Share<TextureRes>> {
		match self.rects.get(index) {
			Some(r) => Some(&self.dyn_atlas[r.allocation_index].texture),
			None => None,
		}
	}

	pub fn get_rect(&self, index: usize) -> Option<Aabb2> {
		match self.rects.get(index) {
			Some(r) => {
				let rectangle = &r.allocation.rectangle;
				Some(Aabb2::new(
					Point2::new(rectangle.min.x as f32 as f32, rectangle.min.y as f32),
						Point2::new(rectangle.max.x as f32, rectangle.max.y as f32)
					))
			},
			None => None,
		}
	}
	// 返回uv(0~1)
	pub fn get_uv(&self, index: usize) -> Option<Aabb2> {
		match self.rects.get(index) {
			Some(r) => {
				let size = self.dyn_atlas[r.allocation_index].allocator.size();
				let rectangle = &r.allocation.rectangle;
				Some(Aabb2::new(
					Point2::new(rectangle.min.x as f32 / size.width as f32, rectangle.max.y as f32 / size.height as f32),
						Point2::new(rectangle.max.x as f32 / size.width as f32, rectangle.min.y as f32 / size.height as f32)
					))
			},
			None => None,
		}
	}
}

#[test]
fn test () {
	// let width = 2048;
	// let height = 2048;
	
	// let mut atlas_allocator= AtlasAllocator::new(guillotiere::Size::new(width as i32, height as i32));
	// let allocation= atlas_allocator.allocate(guillotiere::Size::new(width as i32, height as i32)).unwrap();
	// println!("allocation: {:?}", allocation);
	// let dyn_atlas_index = self.dyn_atlas.insert(DynAtlas{allocator: atlas_allocator, texture: target, count: 0});
	// let index = self.rects.insert(RectIndex::new(allocation,  dyn_atlas_index));
	// index
}