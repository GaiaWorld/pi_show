use std::hash::{Hash, Hasher};
use pi_assets::asset::Handle;
use pi_assets::homogeneous::HomogeneousMgr;
use pi_atom::Atom;
use guillotiere::*;
use slab::Slab;
use hal_core::*;
use pi_share::Share;
use hash::DefaultHasher;

use crate::component::user::*;
use crate::render::asset::ShareAssetMgr;
use crate::render::res::{Opacity, TextureRes, RenderBufferRes};

lazy_static! {
    pub static ref DYN_TEXTURE: Atom = Atom::from("DYN_TEXTURE");
}
pub struct RectIndex {
	allocation: Allocation,
	allocation_index: usize,
	rect: Aabb2,
}

impl RectIndex {
	pub fn new(allocation: Allocation, allocation_index: usize, rect: Aabb2) -> Self {
		Self{
			allocation,
			allocation_index,
			rect
		}
	}
}

pub struct DynAtlas {
	allocator : AtlasAllocator,
	// allocator_index: usize, // debug
	target: HalRenderTarget,
	texture: Handle<TextureRes>,
	count: usize,
	pformat: PixelFormat,
	dformat: DataFormat,
	need_depth: bool,
	ty: usize,
	hash: usize,
	size: usize,
}

impl DynAtlas {
	pub fn size(&self) -> usize {
		self.size

	}
}

pub struct DynAtlasSet {
	dyn_atlas : Slab<DynAtlas>,
	rects: Slab<RectIndex>,
	texture_res_map: ShareAssetMgr<TextureRes>,
	render_buffer_res_map: ShareAssetMgr<RenderBufferRes>,
	texture_cur_index: usize,
	// unuse_texture: Vec<UnuseTexture>,
	unuse_textures: Share<HomogeneousMgr<UnuseTexture>>,

	// 创建的fbo的默认尺寸
	default_size: Size,

	// pub debugList: Vec<Cmd>,
	// cur_allocator_index: usize,
}

// #[derive(Serialize, Deserialize)]
// pub enum Cmd {
// 	Allocate(usize, i32, i32),
// 	Deallocate(usize, u32),
// 	Create(usize, i32, i32),
// }


// pub fn exedebug(v: &Vec<Cmd>) {
// 	let mut map: XHashMap<usize, AtlasAllocator> = XHashMap::default();

// 	for item in v.iter() {
// 		match item {
// 			Cmd::Allocate(k, w, h) => {
// 				let r = map.get_mut(k);
// 				match r {
// 					Some(r) => {
// 						r.allocate(guillotiere::Size::new(*w, *h));
// 					},
// 					None => {
// 						log::error!("Allocate AtlasAllocator 不存在！！！{:?}, w:{}, h:{}", k,w, h);
// 					}
// 				}
// 			}
// 			Cmd::Deallocate(k, id) => {
// 				let r = map.get_mut(k);
// 				match r {
// 					Some(r) => {
// 						r.deallocate(AllocId::deserialize(*id));
// 					},
// 					None => {
// 						log::error!(" Deallocate AtlasAllocator 不存在！！！{:?}, {:?}", k, id);
// 					}
// 				}
// 			},
// 			Cmd::Create(k, w, h) => {
// 				let r = AtlasAllocator::new(guillotiere::Size::new(*w, *h));
// 				map.insert(*k, r);
// 			}
// 		}
// 	}
// }
pub struct UnuseTexture {
	texture: Handle<TextureRes>,
	pformat: PixelFormat,
	dformat: DataFormat,
	width: u32,
	height: u32,
	ty: usize,
	hash: usize,
	target: HalRenderTarget,
	size: usize
}

impl pi_assets::asset::Asset for UnuseTexture {
	type Key = u64;
}

impl pi_assets::asset::Size for UnuseTexture {
	fn size(&self) -> usize {
        std::mem::size_of::<UnuseTexture>()
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Size {
	pub width: usize,
	pub height: usize,
}

const PADDING: i32 = 1;
const DOUBLE_PADDING: usize = 2;

impl DynAtlasSet {
	pub fn new(
		texture_res_map: ShareAssetMgr<TextureRes>, 
		render_buffer_res_map: ShareAssetMgr<RenderBufferRes>, 
		unuse_textures: Share<HomogeneousMgr<UnuseTexture>>, 
		default_width: usize, 
		default_height: usize
	) -> DynAtlasSet {
		DynAtlasSet {
			dyn_atlas: Slab::new(),
			rects: Slab::new(),
			texture_res_map,
			render_buffer_res_map,
			unuse_textures,
			texture_cur_index: 0,
			default_size: Size{width: default_width, height: default_height},
			// debugList: Vec::new(),
			// cur_allocator_index: 0,
		}
	}

	pub fn set_default_size(&mut self, default_width: usize, default_height: usize) {
		self.default_size = Size{width: default_width, height: default_height};
	}

	/// 添加矩形
	pub fn add_rect<C: HalContext>(&mut self, exclude: usize, width: f32, height: f32, pformat: PixelFormat, dformat: DataFormat, need_depth: bool, ty: usize/*纹理类型，不同类型的纹理，不会分配到同一张fbo上*/, mut multiple: usize/*倍数：新开纹理是矩形的多少倍 */, gl: &mut C) -> usize {
		let width = width.ceil() as usize;
		let height = height.ceil() as usize;

		for (index, dyn_atlas) in self.dyn_atlas.iter_mut() {
			if exclude == index { // 不能分配在exclude上
				continue;
			}

			if dyn_atlas.pformat != pformat || dyn_atlas.dformat != dformat || dyn_atlas.need_depth != need_depth || dyn_atlas.ty != ty{
				continue;
			}

			let (offset, width, height) = if dyn_atlas.count == 0 {
				(0.0, width, height)
			} else {
				(PADDING as f32, width + DOUBLE_PADDING, height + DOUBLE_PADDING)
			};
			
			// self.debugList.push(Cmd::Allocate(dyn_atlas.allocator_index, width as i32, height as i32));
			match dyn_atlas.allocator.allocate(guillotiere::Size::new(width as i32, height as i32)) {
				Some(allocation) => {
					dyn_atlas.count += 1;
					let rectangle = &allocation.rectangle;
					let rect = Aabb2::new(
						Point2::new(rectangle.min.x as f32 + offset, rectangle.min.y as f32 + offset),
						Point2::new(rectangle.max.x as f32 - offset, rectangle.max.y as f32 - offset)
					);
					let index = self.rects.insert(RectIndex::new(allocation, index, rect));
					return index;
				},
				None => (),
			};
		}

		let (mut w, mut h) = if !need_depth && multiple > 0 {
			while multiple > 1 {
				if width*multiple > self.default_size.width || width*multiple > self.default_size.height {
					multiple -= 1;
				} else {
					break;
				}
			}
			((width*multiple) as i32, (height*multiple) as i32)
		} else {
			(self.default_size.width.max(width) as i32, self.default_size.height.max(height) as i32)
		};

		if pformat == PixelFormat::RGB {// 目前RGB是用来绘制maskimage的渐变色，不需要很大的fbo，（改为外部选择大小？TODO）
			w = 100.max(width as i32 * 3);
			h = 100.max(height as i32 * 3);
		}

		let unuse =  self.unuse_textures.pop_by_filter(|t| {
			if t.pformat == pformat && t.dformat == dformat && t.width >= width as u32 && t.height >= height as u32 && t.ty == ty {
				return true;
			}
			// if t.hash == hash && 
			// 	(( // 只需要一张纹理，则只要该纹理的大小大于等于要求的大小即可
			// 		len == 1 &&
			// 		t.width >= width &&
			// 		t.height >= height) ||
			// 	( // 需要多张纹理，该纹理的大小必须等于要求的大小（如果大于等于就可以，后续如果找不到缓冲的纹理，则需要创建比要求的大小更大的纹理）
			// 		len > 1 && 
			// 		t.width == width &&
			// 		t.height == height)) {
			// 	return true;
			// }
			return false;
		});
		// if let Some(r) = unuse {
		// 	return (r.texture.clone(), r.width, r.height);
		// }

		// if self.unuse_texture.len() > 0 {
		// 	let mut i = 0;
		// 	while i < self.unuse_texture.len() {
		// 		let t = &self.unuse_texture[i];
		// 		if t.pformat == pformat && t.dformat == dformat && t.width >= width as u32 && t.height >= height as u32 && t.ty == ty {
		// 			let unuse_texture = self.unuse_texture.swap_remove(i);
		// 			match unuse_texture.weak.upgrade() {
		// 				Some(r) => {
		// 					w = unuse_texture.width as i32;
		// 					h = unuse_texture.height as i32;
		// 					catch_texture = Some((unuse_texture.hash, r, unuse_texture.target));
		// 					break;
		// 				},
		// 				None => {
		// 					continue
		// 				},
		// 			};
		// 		} else {
		// 			i += 1;
		// 		}
		// 	}
		// }
		let (texture_hash, texture_res, target) = match unuse {
			Some(r) => {
				w = r.width as i32;
				h = r.height as i32;
				(r.hash, r.texture, r.target)
			},
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
				log::trace!("create fbo texture, index: {}, use_count: {}, w: {}, h: {}", texture.item.index, texture.item.use_count, w, h);
				let mut hasher = DefaultHasher::default();
				DYN_TEXTURE.hash(&mut hasher);
				self.texture_cur_index.hash(&mut hasher);
				pformat.hash(&mut hasher);
				dformat.hash(&mut hasher);
				w.hash(&mut hasher);
				h.hash(&mut hasher);
				self.texture_cur_index += 1;
				let hash = hasher.finish() as usize;
				let key = Atom::from(hash.to_string()+ "_fbo");
				
				let texture = match self.texture_res_map.insert(key.clone(), TextureRes::new(w as usize, h as usize,pformat, dformat,Opacity::Transparent, None, texture, Some((w * h * 4) as usize))) {
					Ok(r) => r,
					Err(_) => self.texture_res_map.get(&key).unwrap(),
				};

				let target = gl.rt_create(
					w as u32,
					h as u32,
					
				)
				.unwrap();
				gl.rt_set_color(&target, Some(&texture.bind));
				(hash, texture, target)
			}
		};
		if need_depth {
			let mut hasher = DefaultHasher::default();
			w.hash(&mut hasher);
			h.hash(&mut hasher);
			let hash = hasher.finish();

			let rb = match self.render_buffer_res_map.get(&hash) {
				Some(r) => r,
				None => {
					let rb = gl.rb_create(
						w as u32,
						h as u32,
						PixelFormat::DEPTH16
					).unwrap();
					match self.render_buffer_res_map.insert(hash, RenderBufferRes{ value: rb, size: (w * h * 3) as usize} ) {
						Ok(r) => r,
						_ => panic!(),
					}
				}
			};

			gl.rt_set_depth(&target, Some(&rb.value));
		} else {
			gl.rt_set_depth(&target, None);
		}

		// self.debugList.push(Cmd::Create(self.cur_allocator_index, w , h));
		let mut atlas_allocator= AtlasAllocator::new(guillotiere::Size::new(w, h));
		// self.debugList.push(Cmd::Allocate(self.cur_allocator_index, width as i32 , height as i32));
		let allocation= atlas_allocator.allocate(guillotiere::Size::new(width as i32, height as i32)).unwrap();

		let size = pi_assets::asset::Size::size(&**texture_res);
		let dyn_atlas_index = self.dyn_atlas.insert(DynAtlas{
			allocator: atlas_allocator,
			target,
			count: 1,
			texture: texture_res,
			pformat,
			dformat,
			need_depth,
			ty: ty,
			hash: texture_hash,
			size,
			// allocator_index: self.cur_allocator_index,
		});
		let rectangle = &allocation.rectangle;
		let rect = Aabb2::new(
			Point2::new(rectangle.min.x as f32, rectangle.min.y as f32),
			Point2::new(rectangle.max.x as f32, rectangle.max.y as f32)
		);
		// self.cur_allocator_index += 1;
		let index = self.rects.insert(RectIndex::new(allocation.clone(),  dyn_atlas_index, rect));
		index
	}

	// 更新矩形
	// exclude为排除fbo，
	pub fn update_or_add_rect<C: HalContext>(&mut self, old: usize, exclude: usize, width: f32, height: f32, pformat: PixelFormat, dformat: DataFormat, need_depth: bool, ty: usize, multiple: usize, gl: &mut C) -> usize {
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

			// log::info!("update_or_add_rect del============={}", old);
			let rect_index = self.rects.remove(old);
			let dyn_atlas = &mut self.dyn_atlas[rect_index.allocation_index];

			// self.debugList.push(Cmd::Deallocate(dyn_atlas.allocator_index, rect_index.allocation.id.serialize()));
			dyn_atlas.allocator.deallocate(rect_index.allocation.id);
			
			dyn_atlas.count -= 1;
			rect_index.allocation_index
		} else {
			0
		};
		let index = self.add_rect(exclude_allocation_index, width, height, pformat, dformat, need_depth, ty, multiple, gl);

		// 如果释放过某个矩形，且当前存在的纹理大于一张，则释放多余的纹理，最多保留一张
		if allocation_index > 0 && self.dyn_atlas.len() > 1 {
			let dyn_atlas = &mut self.dyn_atlas[allocation_index];
			if dyn_atlas.count == 0 {
				let dyn_atla = self.dyn_atlas.remove(allocation_index);
				let (pformat, dformat, width, height, ty, target, hash, size) = (dyn_atla.texture.pformat, dyn_atla.texture.dformat, dyn_atla.texture.width, dyn_atla.texture.height, dyn_atla.ty, dyn_atla.target, dyn_atla.hash, dyn_atla.size);
				self.unuse_textures.create(UnuseTexture { 
					pformat,
					dformat,
					width: width as u32,
					height: height as u32,
					texture: dyn_atla.texture,
					target: target,
					ty,
					hash,
					size
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
		// log::info!("delete_rect============={}", index);
		let mut dyn_atlas = &mut self.dyn_atlas[r.allocation_index];
		// self.debugList.push(Cmd::Deallocate(dyn_atlas.allocator_index, r.allocation.id.serialize()));
		dyn_atlas.allocator.deallocate(r.allocation.id);

		dyn_atlas.count -= 1;

		// 将纹理缓冲起来
		if dyn_atlas.count == 0 && self.dyn_atlas.len() > 1 {
			let dyn_atla = self.dyn_atlas.remove(r.allocation_index);
			let (pformat, dformat, width, height, ty, target, hash, size) = (dyn_atla.texture.pformat, dyn_atla.texture.dformat, dyn_atla.texture.width, dyn_atla.texture.height, dyn_atla.ty, dyn_atla.target, dyn_atla.hash, dyn_atla.size);
			self.unuse_textures.create(UnuseTexture { 
				pformat,
				dformat,
				width: width as u32,
				height: height as u32,
				texture: dyn_atla.texture,
				target: target,
				ty,
				hash,
				size,
			});
		}
		let r = &r.rect;
		Some(guillotiere::Size::new((r.maxs.x - r.mins.x) as i32, (r.maxs.y - r.mins.y) as i32))
	}

	pub fn get_target(&self, index: usize) -> Option<&HalRenderTarget> {
		match self.rects.get(index) {
			Some(r) => Some(&self.dyn_atlas[r.allocation_index].target),
			None => None,
		}
	}

	pub fn get_target_size(&self, index: usize) -> Option<Size> {
		match self.rects.get(index) {
			Some(r) => {
				Some(Size {
					width: self.dyn_atlas[r.allocation_index].allocator.size().width as usize,
					height: self.dyn_atlas[r.allocation_index].allocator.size().height as usize,
				})
			},
			None => None,
		}
	}

	pub fn get_texture(&self, index: usize) -> Option<&Handle<TextureRes>> {
		match self.rects.get(index) {
			Some(r) => Some(&self.dyn_atlas[r.allocation_index].texture),
			None => None,
		}
	}

	pub fn get_rect(&self, index: usize) -> Option<&Aabb2> {
		match self.rects.get(index) {
			Some(r) => Some(&r.rect),
			None => None,
		}
	}

	pub fn get_rect_with_border(&self, index: usize) -> Option<Aabb2> {

		match self.rects.get(index) {
			Some(r) =>{
				let rectangle = &r.allocation.rectangle;
				Some( Aabb2::new(
					Point2::new(rectangle.min.x as f32, rectangle.min.y as f32),
					Point2::new(rectangle.max.x as f32, rectangle.max.y as f32)
				))
			} ,
			None => None,
		}
	}

	// 返回uv(0~1)
	pub fn get_uv(&self, index: usize) -> Option<Aabb2> {
		match self.rects.get(index) {
			Some(r) => {
				let size = self.dyn_atlas[r.allocation_index].allocator.size();
				let rectangle = &r.rect;
				Some(Aabb2::new(
					Point2::new(rectangle.mins.x as f32 / size.width as f32, rectangle.maxs.y as f32 / size.height as f32),
						Point2::new((rectangle.maxs.x as f32) / size.width as f32, rectangle.mins.y as f32 / size.height as f32)
					))
			},
			None => None,
		}
	}

	pub fn debug_rects(&self) -> Vec<DebugRect>{
		let mut l = Vec::new();
		for (i, r) in self.rects.iter() {
			let rect = r.rect.clone();
			l.push(DebugRect {
				id: i,
				rect,
				texture_id: r.allocation_index,
			});
		}
		l
	}

	pub fn debug_texture(&self) -> Vec<DebugTexture>{
		let mut l = Vec::new();
		for (i, r) in self.dyn_atlas.iter() {
			let size = r.allocator.size();
			l.push(DebugTexture {
				id: i,
				rect: Size { width: size.width as usize, height: size.height as usize },
				count: r.count,
				pformat: r.pformat,
				dformat: r.dformat,
				need_depth: r.need_depth,
				ty: r.ty,
				hash: r.hash,
			});
		}
		l
	}

	
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebugRect {
	pub id: usize,
	pub rect: Aabb2,
	pub texture_id: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebugTexture {
	pub id: usize,
	pub rect: Size,
	pub count: usize,
	pub pformat: PixelFormat,
	pub dformat: DataFormat,
	pub need_depth: bool,
	pub ty: usize,
	pub hash: usize,
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
// #[cfg(test)]
// extern crate bincode;
// #[test]


// pub fn exec_dyn_texture(bin: Vec<u8>) {
// 	match bincode::deserialize(bin.as_slice()) {
// 		Ok(r) => exedebug(&r),
// 		Err(e) => {
// 			println!("deserialize_class_map error: {:?}", e);
// 			return;
// 		}
// 	}
// }

