//! 处理MaskImage属性
//! * 当MaskImage变化时，在RenderContextMark组件上标记自己（RenderContext系统根据标记中是否存在1，来确定是否将该节点作为一个渲染上下文）
//! * 当MaskImage变化时，或删除时，取消在RenderContextMark组件上的标记
//! * 当MaskImage变化时，将该节点记录在脏列表，在帧推时，决定取消或添加`mask_texture`uniform,`mask_uv`attribute,`MASK_IMAGE`宏（之所以延迟处理，因为监听收到事件时，RenderObject可能还不存在，无法设置uniform）
//! * 另外，需要处理MaskImage为一个渐变颜色声明的情况。需要将渐变色与渲染为一张图片

use std::cell::RefCell;
use std::marker::PhantomData;

use ecs::entity::Entity;
use share::Share;
use std::hash::{Hash, Hasher};

use hash::{DefaultHasher, XHashSet};

use atom::Atom;
use ecs::{CreateEvent, DeleteEvent, ModifyEvent, EntityListener, MultiCaseImpl, MultiCaseListener, Runner, SingleCaseImpl, SingleCaseListener};
use ecs::monitor::{Event, NotifyImpl};
use hal_core::*;
use map::vecmap::VecMap;

use crate::component::calc::{LayoutR, MaskTexture};
use crate::component::calc::*;
use crate::component::user::{Point2, MaskImageClip, RenderContextMark, Aabb2};
use crate::entity::Node;
use crate::render::engine::{Engine, ShareEngine};
use crate::render::res::*;
use crate::{single::*};
use crate::single::dyn_texture::DynAtlasSet;
use crate::single::{DirtyViewRect};
use crate::system::render::shaders::image::{ FBO_VS_SHADER_NAME, FBO_FS_SHADER_NAME};
use crate::system::util::constant::*;
use crate::system::util::{*, let_top_offset_matrix as let_top_offset_matrix1};
use crate::Z_MAX;

lazy_static! {
	static ref UV: Atom = Atom::from("UV");
	static ref POSITION: Atom = Atom::from("Position");
	static ref INDEX: Atom = Atom::from("Index");
}

/// 处理遮罩纹理的system
pub struct MaskTextureSys<C> {
	render_mark_index: usize,
	dirty: XHashSet<usize>,
	uv1_sampler: Share<SamplerRes>,
	mark: PhantomData<C>,
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for MaskTextureSys<C> {
	type ReadData = (
		&'a MultiCaseImpl<Node, MaskTexture>,
		&'a MultiCaseImpl<Node, MaskImageClip>,
		&'a MultiCaseImpl<Node, StyleMark>,
		&'a SingleCaseImpl<Share<RefCell<DynAtlasSet>>>,
	);
	type WriteData = (
		&'a mut MultiCaseImpl<Node, RenderContext>,
		&'a mut SingleCaseImpl<RenderObjs>,
		&'a mut SingleCaseImpl<ShareEngine<C>>,
		&'a mut SingleCaseImpl<RenderContextAttrCount>,
		&'a mut SingleCaseImpl<Oct>,
	);
	fn run(
		&mut self, 
		(mask_textures, 
		mask_image_clips,
		style_marks, 
		dyn_atlas_set): Self::ReadData, 
		write: Self::WriteData) {
		if self.dirty.len() == 0 {
			return;
		}

		let (
			render_contexts,
			render_objs,
			engine,
			_,
			octree
		) = write;
		let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
		
		let mut dirty = std::mem::replace(&mut self.dirty, XHashSet::default());
		for id in dirty.iter() {
			let id = *id;
			let style_mark = match style_marks.get(id) { // 节点已经销毁，不做处理
				Some(r) => r,
				None => continue,
			};
			let (mask_texture, render_context) = match (mask_textures.get(id), render_contexts.get_mut(id)) {
				// 没有对应的rendercontext，这种情况出现应该是逻辑问题，理论上必须存在
				(_, None) => continue,
				(Some(r1), Some(r2)) => (r1, r2), 
				(None, Some(r2)) => {
					if r2.render_obj_index == 0 {
						continue; // 根节点，什么也不做
					}
					let render_obj = &mut render_objs[r2.render_obj_index];
					render_obj.vs_defines.remove("MASK_IMAGE");
					if let Some(_r) = render_obj.fs_defines.remove("MASK_IMAGE") {
						notify.modify_event(r2.render_obj_index, "program_dirty", 0);
					}
					// mask_texture不存在，context存在，还应该从renderobj中删除maskimage设置的uv属性 TODO
					continue;
				},
			};

			let dirty1 = style_mark.dirty1;
			let render_obj = &mut render_objs[render_context.render_obj_index];

			// 设置mask_image
			if dirty1 & StyleType1::MaskTexture as usize != 0 {
				let dyn_atlas_set = dyn_atlas_set.borrow_mut();
				render_obj.vs_defines.add("MASK_IMAGE");
				if let None = render_obj.fs_defines.add("MASK_IMAGE") {
					notify.modify_event(render_context.render_obj_index, "program_dirty", 0);
				}
				let texture = match mask_texture {
					MaskTexture::All(r) => &r.bind,
					MaskTexture::Part(r) => &dyn_atlas_set.get_texture(r.index()).unwrap().bind,
				};
				render_obj.paramter.set_texture(
				"maskTexture",
				(texture, &self.uv1_sampler),
				);	
			}

			// oct发生改变时，重新设置maskRect
			if style_mark.dirty & StyleType::Oct as usize != 0 || dirty1 & StyleType1::MaskTexture as usize != 0 {
				let oct = octree.get(id).unwrap().0;
				render_obj.paramter.set_single_uniform(
				"maskRect",
				UniformValue::Float4(oct.mins.x, oct.mins.y, oct.maxs.x - oct.mins.x, oct.maxs.y - oct.mins.y),
				);
			}

			if dirty1 & StyleType1::ContentBox as usize != 0 || dirty1 & StyleType1::MaskTexture as usize != 0 {
				update_geo_quad_with_mask(render_obj, mask_texture, mask_image_clips.get(id), engine)
			}
		}

		dirty.clear(); // 清理脏
		self.dirty = dirty;
		notify.modify_event(0, "context", 0);
	}

	fn setup(&mut self, _read: Self::ReadData, write: Self::WriteData) {
		***write.3 = ***write.3 + 1;
		self.render_mark_index = ***write.3; // MaskTexture属性的rendercontext标记索引
	}
}

// 监听创建和修改事件，对其进行标记
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, MaskTexture, (CreateEvent, ModifyEvent)> for MaskTextureSys<C> {
	type ReadData = ();
	type WriteData = &'a mut MultiCaseImpl<Node, RenderContextMark>;
	fn listen(&mut self, event: &Event, _: Self::ReadData, marks: Self::WriteData) {
		self.dirty.insert(event.id); // 插入到脏列表中

		// 存在遮罩纹理，则标记在上下文标记中设置纹理为true，否则设置为false
		match marks.get(event.id) {
			Some(_r) => marks[event.id].set(self.render_mark_index, true),
			None => marks[event.id].set(self.render_mark_index, false)
		};
		marks.get_notify().modify_event(event.id, "", 0);
	}
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, Oct, (CreateEvent, ModifyEvent)> for MaskTextureSys<C> {
	type ReadData = ();
	type WriteData = &'a mut MultiCaseImpl<Node, RenderContextMark>;
	fn listen(&mut self, event: &Event, _: Self::ReadData, marks: Self::WriteData) {
		if let Some(_r) = marks.get(event.id) {
			self.dirty.insert(event.id); // 插入到脏列表中
		}
	}
}

// 监听删除事件，取消标记
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, MaskTexture, DeleteEvent> for MaskTextureSys<C> {
	type ReadData = ();
	type WriteData = &'a mut MultiCaseImpl<Node, RenderContextMark>;
	fn listen(&mut self, event: &Event, _: Self::ReadData, marks: Self::WriteData) {
		self.dirty.insert(event.id); // 插入到脏列表中

		// 取消上下标记
		marks[event.id].set(self.render_mark_index, false);
		marks.get_notify().modify_event(event.id, "", 0);
	}
}

impl<C: HalContext + 'static> MaskTextureSys<C> {
	pub fn with_capacity(engine: &mut Engine<C>, capacity: usize) -> Self {
		let mut sm1 = SamplerDesc::default();
		sm1.u_wrap = TextureWrapMode::ClampToEdge;
		sm1.v_wrap = TextureWrapMode::ClampToEdge;
		let uv1_sampler = engine.create_sampler_res(sm1);

		MaskTextureSys {
			render_mark_index: 0,
			dirty: XHashSet::default(),
			uv1_sampler,
			mark: PhantomData,
		}
	}
}

fn update_geo_quad_with_mask<C: HalContext + 'static>(
	render_obj: &mut RenderObj,
	texture: &MaskTexture,
	image_clip: Option<&MaskImageClip>,
	engine: &mut Engine<C>,
) {
	let geo = if let Some(r) = &render_obj.geometry {
		r
	} else {
		return;
	};
	let geo = unsafe { &mut *(Share::as_ptr(geo) as usize as *mut GeometryRes) };

	// 存在maskuv,不需要添加uv
	if geo.buffers.get(2).is_some() {
		return;
	}
	let clip = match texture {
		MaskTexture::All(_r) => match image_clip {
			Some(r) => r.0.clone(),
			None => Aabb2::new(Point2::new(0.0, 0.0), Point2::new(1.0, 1.0)),
		},
		MaskTexture::Part(r) => {
			let mut uv = r.get_uv();
			let size = r.size();
			uv.maxs.x = uv.maxs.x - 0.5/size.0 as f32;
			uv.mins.y = uv.mins.y - 0.5/size.1 as f32;

			uv.mins.x = uv.mins.x + 0.5/size.0 as f32;
			uv.maxs.y = uv.maxs.y + 0.5/size.1 as f32;
			uv
		},
	};
	let (uv1, uv2) =  (clip.mins, clip.maxs);
	let uv1_hash = cal_uv_hash(&uv1, &uv2);
	let uv1_buffer = create_uv_buffer(uv1_hash, &uv1, &uv2, engine);

	render_obj.paramter.set_single_uniform(
	"maskUv",
	UniformValue::Float4(uv1.x, uv2.y, uv2.x - uv1.x, uv1.y - uv2.y),
	);
	engine
		.gl
		.geometry_set_attribute(geo, &AttributeName::UV1, &uv1_buffer, 2)
		.unwrap();
	geo.buffers.insert(2, uv1_buffer);
}

#[inline]
fn cal_uv_hash(uv1: &Point2, uv2: &Point2) -> u64 {
	let mut hasher = DefaultHasher::default();
	UV.hash(&mut hasher);
	f32_4_hash_(uv1.x, uv1.y, uv2.x, uv2.y, &mut hasher);
	hasher.finish()
}

fn create_uv_buffer<C: HalContext + 'static>(
	uv_hash: u64,
	uv1: &Point2,
	uv2: &Point2,
	engine: &mut Engine<C>,
) -> Share<BufferRes> {
	match engine.buffer_res_map.get(&uv_hash) {
		Some(r) => r,
		None => {
			let uvs = [uv1.x, uv1.y, uv1.x, uv2.y, uv2.x, uv2.y, uv2.x, uv1.y];
			engine.create_buffer_res(
				uv_hash,
				BufferType::Attribute,
				8,
				Some(BufferData::Float(&uvs[..])),
				false,
			)
		}
	}
}


impl_system! {
	MaskTextureSys<C> where [C: HalContext + 'static],
	true,
	{
		MultiCaseListener<Node, MaskTexture, DeleteEvent>
		MultiCaseListener<Node, MaskTexture, (CreateEvent, ModifyEvent)>
		SingleCaseListener<Oct, (CreateEvent, ModifyEvent)>
	}
}