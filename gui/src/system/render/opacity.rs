//! 处理Opacity属性
//! * 当Opacity变化时，在RenderContextMark组件上标记自己（RenderContext系统根据标记中是否存在1，来确定是否将该节点作为一个渲染上下文）
//! * 当Opacity变化时，或删除时，取消在RenderContextMark组件上的标记
//! * 当Opacity变化时，将该节点记录在脏列表，在帧推时，决定取消或添加`alpha`uniform（之所以延迟处理，因为监听收到事件时，RenderObject可能还不存在，无法设置uniform）

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
use crate::Z_MAX;

pub struct OpacitySys<C> {
	render_mark_index: usize,
	dirty: XHashSet<usize>,
	mark: PhantomData<C>,
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for OpacitySys<C> {
	type ReadData = (
		&'a MultiCaseImpl<Node, Opacity>,
		&'a MultiCaseImpl<Node, StyleMark>,
	);
	type WriteData = (
		&'a mut MultiCaseImpl<Node, RenderContext>,
		&'a mut SingleCaseImpl<RenderObjs>,
		&'a mut SingleCaseImpl<ShareEngine<C>>,
		&'a mut SingleCaseImpl<RenderContextAttrCount>,
	);
	fn run(&mut self, (opacitys, style_marks): Self::ReadData, write: Self::WriteData) {
		if self.dirty.len() == 0 {
			return;
		}

		let (
			render_contexts,
			render_objs,
			engine,
			_) = write;
		let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
		
		let mut dirty = std::mem::replace(&mut self.dirty, XHashSet::default());
		for id in dirty.iter() {
			let id = *id;
			let style_mark = match style_marks.get(id) { // 节点已经销毁，不做处理
				Some(r) => r,
				None => continue,
			};
			let (opacity, render_context) = match (opacitys.get(id), render_contexts.get_mut(id)) {
				(Some(r), Some(r1)) if **r < 1.0 => {
					(**r, r1)
				},
				(_, Some(r1)) => {
					if let Some(r) = render_objs[r1.render_obj_index].fs_defines.remove("OPACITY") {
						notify.modify_event(r1.render_obj_index, "program_dirty", 0);
					}
					continue;
				},
				_ => continue
			};
			
			let render_obj = &mut render_objs[render_context.render_obj_index];

			if let None = render_obj.fs_defines.add("OPACITY") {;
				notify.modify_event(render_context.render_obj_index, "program_dirty", 0);
			}

			// 设置uniform
			render_obj.paramter.set_single_uniform("alpha", UniformValue::Float1(opacity));
		}

		dirty.clear(); // 清理脏
		self.dirty = dirty;

		notify.modify_event(0, "context", 0);
	}

	fn setup(&mut self, _read: Self::ReadData, write: Self::WriteData) {
		***write.3 = ***write.3 + 1;
		self.render_mark_index = ***write.3; // Opacity属性的rendercontext标记索引
	}
}

// 监听创建和修改事件，对其进行标记
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Opacity, (CreateEvent, ModifyEvent)> for OpacitySys<C> {
	type ReadData = &'a MultiCaseImpl<Node, Opacity>;
	type WriteData = &'a mut MultiCaseImpl<Node, RenderContextMark>;
	fn listen(&mut self, event: &Event, opacitys: Self::ReadData, marks: Self::WriteData) {
		self.dirty.insert(event.id); // 插入到脏列表中
		
		// opacity小于1.0,添加标记
		if (opacitys[event.id].0 < 1.0) {
			marks[event.id].set(self.render_mark_index, true);
		} else {
			// 否则取消标记
			marks[event.id].set(self.render_mark_index, false);
		}
		marks.get_notify().modify_event(event.id, "", 0);
	}
}

// 监听删除事件，取消标记
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Opacity, DeleteEvent> for OpacitySys<C> {
	type ReadData = ();
	type WriteData = &'a mut MultiCaseImpl<Node, RenderContextMark>;
	fn listen(&mut self, event: &Event, _: Self::ReadData, marks: Self::WriteData) {
		self.dirty.insert(event.id); // 插入到脏列表中

		// 取消上下标记
		marks[event.id].set(self.render_mark_index, false);
		marks.get_notify().modify_event(event.id, "", 0);
	}
}

impl<C: HalContext + 'static> OpacitySys<C> {
	pub fn with_capacity(engine: &mut Engine<C>, capacity: usize) -> Self {
		OpacitySys {
			render_mark_index: 0,
			dirty: XHashSet::default(),
			mark: PhantomData,
		}
	}
}


impl_system! {
	OpacitySys<C> where [C: HalContext + 'static],
	true,
	{
		MultiCaseListener<Node, Opacity, DeleteEvent>
		MultiCaseListener<Node, Opacity, (CreateEvent, ModifyEvent)>
	}
}