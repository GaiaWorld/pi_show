//! 处理Blur属性
//! Blur属性是将该节点及其递归自己点作为一个整体来添加模糊效果，会使得当前节点成为一个渲染上下文
//! * 当 Blur变化时，在RenderContextMark组件上标记自己（RenderContext系统根据标记中是否存在1，来确定是否将该节点作为一个渲染上下文）
//! * 当Blur变化时，或删除时，取消在RenderContextMark组件上的标记
//! * 当Blur变化时，将该节点记录在脏列表，在帧推时，决定为渲染上下文取消或添加`blur`后处理（之所以延迟处理，因为监听收到事件时，RenderObject可能还不存在，无法设置uniform）

use std::cell::RefCell;
use std::marker::PhantomData;

use ecs::entity::Entity;
use share::Share;
use std::hash::{Hash, Hasher};

// use ordered_float::NotNan;
use hash::{DefaultHasher, XHashSet};

use pi_atom::Atom;
use ecs::{CreateEvent, DeleteEvent, ModifyEvent, EntityListener, MultiCaseImpl, MultiCaseListener, Runner, SingleCaseImpl, SingleCaseListener};
use ecs::monitor::{Event, NotifyImpl};
use hal_core::*;
use map::vecmap::VecMap;

use crate::component::calc::{LayoutR, MaskTexture};
use crate::component::calc::*;
use crate::component::user::{ *, Opacity};
use crate::entity::Node;
use crate::render::engine::{Engine, ShareEngine};
use crate::render::res::*;
use crate::{single::*};
use crate::single::dyn_texture::DynAtlasSet;
use crate::single::{DirtyViewRect};
use crate::system::render::shaders::image::{ FBO_VS_SHADER_NAME, FBO_FS_SHADER_NAME};
use crate::system::util::constant::*;
use crate::system::util::{*, let_top_offset_matrix as let_top_offset_matrix1};

use super::gassu_blur::add_gassu_blur;


pub struct BlurSys<C> {
	render_mark_index: usize,
	dirty: XHashSet<usize>,
	mark: PhantomData<C>,
	// render_map: VecMap<usize>,
	// default_sampler: Share<SamplerRes>,
	// uv1_sampler: Share<SamplerRes>,
	// unit_geo: Share<GeometryRes>, // 含uv， index， pos
	// default_paramter: FboParamter,
	// marker: PhantomData<C>,
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for BlurSys<C> {
	type ReadData = (
		&'a MultiCaseImpl<Node, Blur>,
		&'a MultiCaseImpl<Node, StyleMark>,
		&'a MultiCaseImpl<Node, ContentBox>,
		&'a SingleCaseImpl<PremultiState>,
	);
	type WriteData = (
		&'a mut MultiCaseImpl<Node, RenderContext>,
		&'a mut SingleCaseImpl<RenderObjs>,
		&'a mut SingleCaseImpl<ShareEngine<C>>,
		&'a mut SingleCaseImpl<Share<RefCell<DynAtlasSet>>>,
		&'a mut SingleCaseImpl<RenderContextAttrCount>,
	);
	fn run(&mut self, (blurs, style_marks, context_boxs, premulti_state): Self::ReadData, write: Self::WriteData) {
		if self.dirty.len() == 0 {
			return;
		}

		let (
			render_contexts,
			render_objs,
			engine,
			dyn_atlas_set, 
			_) = write;
		let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
		
		let mut dirty = std::mem::replace(&mut self.dirty, XHashSet::default());
		for id in dirty.iter() {
			let id = *id;
			let style_mark = match style_marks.get(id) { // 节点已经销毁，不做处理
				Some(r) => r,
				None => continue,
			};
			let (blur, render_context) = match (blurs.get(id), render_contexts.get_mut(id)) {
				(Some(r), Some(r1)) => if **r != 0.0{
					(**r, r1)
				} else {
					r1.set_post(None);
					return;
				},
				(Some(r), None) => return,// 不会出现这种情况， 可能是逻辑问题
				(None, Some(r1))  => {
					r1.set_post(None);
					return
				},
				_ => return,
			};
			let has_blur = add_gassu_blur(id,
				blur,
				render_context,
				engine,
				&context_boxs.get(id).unwrap().0,
				&premulti_state.0);
			// 存在blur，则为该renderContext添加高斯模糊后处理， TODO
			notify.modify_event(render_context.render_obj_index, "", 0);
		}

		dirty.clear(); // 清理脏
		self.dirty = dirty;

		notify.modify_event(0, "context", 0);
	}

	fn setup(&mut self, _read: Self::ReadData, write: Self::WriteData) {
		***write.4 = ***write.4 + 1;
		self.render_mark_index = ***write.4; // Blur属性的rendercontext标记索引
	}
}

// 监听创建和修改事件，对其进行标记
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Blur, (CreateEvent, ModifyEvent)> for BlurSys<C> {
	type ReadData = &'a MultiCaseImpl<Node, Blur>;
	type WriteData = &'a mut MultiCaseImpl<Node, RenderContextMark>;
	fn listen(&mut self, event: &Event, blurs: Self::ReadData, marks: Self::WriteData) {
		self.dirty.insert(event.id); // 插入到脏列表中

		// 变为0， 则取消标记
		if (*blurs[event.id] == 0.0) {
			marks[event.id].set(self.render_mark_index, false);
		} else {
			// 否则，添加标记
			marks[event.id].set(self.render_mark_index, true);
		}
		marks.get_notify().modify_event(event.id, "", 0);
	}
}

// 监听删除事件，取消标记
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Blur, DeleteEvent> for BlurSys<C> {
	type ReadData = ();
	type WriteData = &'a mut MultiCaseImpl<Node, RenderContextMark>;
	fn listen(&mut self, event: &Event, _: Self::ReadData, marks: Self::WriteData) {
		self.dirty.insert(event.id); // 插入到脏列表中

		// 取消上下标记
		marks[event.id].set(self.render_mark_index, false);
		marks.get_notify().modify_event(event.id, "", 0);
	}
}

impl<C: HalContext + 'static> BlurSys<C> {
	pub fn with_capacity(engine: &mut Engine<C>, capacity: usize) -> Self {
		let mut sm = SamplerDesc::default();
		// 使用点采样，因为fbo的纹理部分和渲染的实际大小一致
		sm.u_wrap = TextureWrapMode::ClampToEdge;
		sm.v_wrap = TextureWrapMode::ClampToEdge;
		sm.min_filter = TextureFilterMode::Nearest;
		sm.mag_filter = TextureFilterMode::Nearest;

		let default_sampler = engine.create_sampler_res(sm);

		let mut sm1 = SamplerDesc::default();
		sm1.u_wrap = TextureWrapMode::ClampToEdge;
		sm1.v_wrap = TextureWrapMode::ClampToEdge;
		let default_sampler1 = engine.create_sampler_res(sm1);

		let positions = engine
			.buffer_res_map
			.get(&(POSITIONUNIT.get_hash() as u64))
			.unwrap();
		let indices = engine
			.buffer_res_map
			.get(&(INDEXUNIT.get_hash() as u64))
			.unwrap();

		let geo = engine.create_geometry();
		engine
			.gl
			.geometry_set_attribute(&geo, &AttributeName::Position, &positions, 2)
			.unwrap();
		engine
			.gl
			.geometry_set_attribute(&geo, &AttributeName::UV0, &positions, 2)
			.unwrap();
		engine
			.gl
			.geometry_set_indices_short(&geo, &indices)
			.unwrap();

		BlurSys {
			render_mark_index: 0,
			dirty: XHashSet::default(),
			mark: PhantomData,
			// render_map: VecMap::with_capacity(capacity),
			// default_sampler: default_sampler,
			// uv1_sampler: default_sampler1,
			// unit_geo: Share::new(GeometryRes {
			// 	geo: geo,
			// 	buffers: vec![indices, positions.clone(), positions],
			// }),
			// default_paramter: FboParamter::default(),
			// marker: PhantomData,
		}
	}
}


impl_system! {
	BlurSys<C> where [C: HalContext + 'static],
	true,
	{
		MultiCaseListener<Node, Blur, DeleteEvent>
		MultiCaseListener<Node, Blur, (CreateEvent, ModifyEvent)>
		// MultiCaseListener<Node, Opacity, (CreateEvent, DeleteEvent, ModifyEvent)>
		// MultiCaseListener<Node, ContentBox, (CreateEvent, ModifyEvent)>
		// MultiCaseListener<Node, RenderContext, DeleteEvent>
		// EntityListener<Node, ModifyEvent>
		// SingleCaseListener<IdTree, CreateEvent>
		// SingleCaseListener<RenderBegin, ModifyEvent>
	}
}