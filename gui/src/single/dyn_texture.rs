/// 解析字符串格式的样式
use std::mem::transmute;
use std::str::FromStr;
use std::result::Result;

use atom::Atom;
use hash::XHashMap;
use flex_layout::*;
use nalgebra::Field;
// use rectangle_pack::{
//     GroupedRectsToPlace,
//     RectToInsert,
//     pack_rects,
//     TargetBin,
//     volume_heuristic,
//     contains_smallest_box
// };
use guillotiere::*;
use slab::Slab;
use hal_core::*;

use crate::component::calc::*;
use crate::component::user::Opacity;
use crate::component::user::*;
use crate::render::engine::Engine;
use crate::render::res::TextureRes;
use crate::single::class::*;
use std::collections::BTreeMap;

// pub struct DynAtlas{
// 	atlas_allocator : AtlasAllocator,
// 	render_target: HalRenderTarget,
// 	rects: XHashMap<usize, Aabb2>,
// 	rects: XHashMap<usize, Aabb2>,
// }

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
	texture: HalRenderTarget,
	count: usize,
}

pub struct DynAtlasSet {
	dyn_atlas : Slab<DynAtlas>,
	rects:Slab<RectIndex>,
}

impl DynAtlasSet {
	pub fn new() -> DynAtlasSet {
		DynAtlasSet {
			dyn_atlas: Slab::new(),
			rects: Slab::new(),
		}
	}

	/// 添加矩形
	pub fn add_rect<C: HalContext>(&mut self, exclude: usize, width: f32, height: f32, gl: &mut C) -> usize {
		for (index, dyn_atlas) in self.dyn_atlas.iter_mut() {
			if exclude == index { // 不能分配在exclude上
				continue;
			}

			match dyn_atlas.allocator.allocate(guillotiere::Size::new(width as i32, height as i32)) {
				Some(allocation) => {
					dyn_atlas.count += 1;
					return self.rects.insert(RectIndex::new(allocation, index));
				},
				None => (),
			};
		}

		let w = u32::max(width as u32, 2048);
		let h = u32::max(height as u32, 2048);
		let target = gl.rt_create(
			None,
			w,
			h,
			PixelFormat::RGBA,
			DataFormat::UnsignedByte,
			false,
		)
		.unwrap();
		
		let mut atlas_allocator= AtlasAllocator::new(guillotiere::Size::new(w as i32, h as i32));
		let allocation= atlas_allocator.allocate(guillotiere::Size::new(width as i32, height as i32)).unwrap();

		let dyn_atlas_index = self.dyn_atlas.insert(DynAtlas{allocator: atlas_allocator, texture: target, count: 0});
		let index = self.rects.insert(RectIndex::new(allocation,  dyn_atlas_index));
		index
	}

	// 更新矩形
	// exclude为排除fbo， 
	pub fn update_or_add_rect<C: HalContext>(&mut self, old: usize, exclude: usize, width: f32, height: f32, gl: &mut C) -> usize {
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
			rect_index.allocation_index
		} else {
			0
		};
		let index = self.add_rect(exclude_allocation_index, width, height, gl);

		// 如果释放过某个矩形，且当前存在的纹理大于一张，则释放多余的纹理，最多保留一张
		if allocation_index > 0 {
			let dyn_atlas = &mut self.dyn_atlas[allocation_index];
			if dyn_atlas.count == 0 && self.dyn_atlas.len() > 2 {
				self.dyn_atlas.remove(allocation_index);
			}
		}

		index
	}

	pub fn delete_rect(&mut self, index: usize) -> Option<guillotiere::Size> {
		let r = match self.rects.get(index) {
			Some(_r) => self.rects.remove(index),
			None => return None,
		};
		let mut dyn_atlas = &mut self.dyn_atlas[r.allocation_index];
		dyn_atlas.allocator.deallocate(r.allocation.id);
		dyn_atlas.count -= 1;

		// 如果纹理为空，切当前存在的纹理大于一张，则释放多余的纹理，最多保留一张
		if dyn_atlas.count == 0 && self.dyn_atlas.len() > 1 {
			self.dyn_atlas.remove(r.allocation_index);
		}
		let r = &r.allocation.rectangle;
		Some(guillotiere::Size::new(r.max.x - r.min.x, r.max.y - r.min.y))
	}

	pub fn get_target(&self, index: usize) -> Option<&HalRenderTarget> {
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