//! 处理ClipPath属性

use std::marker::PhantomData;

use flex_layout::Rect;

use hash::XHashSet;

use ecs::{CreateEvent, DeleteEvent, ModifyEvent, MultiCaseImpl, MultiCaseListener, Runner, SingleCaseImpl};
use ecs::monitor::{Event, NotifyImpl};
use hal_core::*;
use pi_style::style::BaseShape;

use crate::component::calc::LayoutR;
use crate::component::calc::*;
use crate::component::user::*;
use crate::entity::Node;
use crate::render::engine::ShareEngine;
use crate::system::util::{cal_border_radius, let_top_offset_matrix};
use crate::single::*;

pub struct ClipPathSys<C> {
	render_mark_index: usize,
	dirty: XHashSet<usize>,
	mark: PhantomData<C>,
}

impl<C: HalContext + 'static> Default for ClipPathSys<C> {
    fn default() -> Self {
        Self { render_mark_index: Default::default(), dirty: Default::default(), mark: Default::default() }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for ClipPathSys<C> {
	type ReadData = (
		&'a MultiCaseImpl<Node, ClipPath>,
		&'a MultiCaseImpl<Node, StyleMark>,
		&'a MultiCaseImpl<Node, LayoutR>,
		&'a MultiCaseImpl<Node, WorldMatrix>,
		&'a MultiCaseImpl<Node, ContentBox>,
		&'a MultiCaseImpl<Node, Transform>,
		&'a SingleCaseImpl<RenderBegin>,
	);
	type WriteData = (
		&'a mut MultiCaseImpl<Node, RenderContext>,
		&'a mut SingleCaseImpl<RenderObjs>,
		&'a mut SingleCaseImpl<ShareEngine<C>>,
		&'a mut SingleCaseImpl<RenderContextAttrCount>,
	);
	fn run(&mut self, (clip_paths, style_marks, layouts, world_matrixs, content_boxs, transforms, render_begin): Self::ReadData, write: Self::WriteData) {
		if self.dirty.len() == 0 {
			return;
		}

		let (
			render_contexts,
			render_objs,
			_engine,
			_) = write;
		let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
		
		let mut dirty = std::mem::replace(&mut self.dirty, XHashSet::default());
		for id in dirty.iter() {
			let id = *id;
			let _style_mark = match style_marks.get(id) { // 节点已经销毁，不做处理
				Some(r) => r,
				None => continue,
			};
			let (clip_path, render_context, layout, world_matrix, content_box, w_invert) = match (
				clip_paths.get(id), 
				render_contexts.get_mut(id),
				layouts.get(id),
				world_matrixs.get(id),
				content_boxs.get(id),
			) {
				(Some(r), Some(r1),  Some(r2),  Some(r3),  Some(r4)) => {
					let transform = &transforms[id];
					let m = let_top_offset_matrix(r2, r3, transform, 0.0, 0.0);
					if let Some(invert) = m.invert() {
						(r, r1, r2, m, r4, invert)
					} else {
						continue;
					}
				},
				(_, Some(r1), _, _, _) => {

					if let (None, None, None, None, None, None) = (
						render_objs[r1.render_obj_index].fs_defines.remove("BORDER_RADIUS"), 
						render_objs[r1.render_obj_index].fs_defines.remove("RECT"),
						render_objs[r1.render_obj_index].fs_defines.remove("ELLIPSE"),
						render_objs[r1.render_obj_index].fs_defines.remove("CIRCLE"),
						render_objs[r1.render_obj_index].fs_defines.remove("SECTOR"),
						render_objs[r1.render_obj_index].vs_defines.remove("SDF_CLIP"),
					) {} else {
						notify.modify_event(r1.render_obj_index, "program_dirty", 0);
					}
					continue;
				},
				_ => continue
			};
			
			let render_obj = &mut render_objs[render_context.render_obj_index];

			let min_x = content_box.0.mins.x.max(render_begin.0.viewport.0 as f32).floor();
			let min_y = content_box.0.mins.y.max(render_begin.0.viewport.1 as f32).floor();
			let max_x = content_box.0.maxs.x.min(render_begin.0.viewport.2 as f32).ceil();
			let max_y = content_box.0.maxs.y.min(render_begin.0.viewport.3 as f32).ceil();

			render_obj.paramter.set_single_uniform("clipMatrixInvert", UniformValue::MatrixV4(w_invert.as_slice().to_vec()));
			render_obj.paramter.set_single_uniform("clipBoxRect", UniformValue::Float4(min_x, min_y, max_x - min_x, max_y - min_y));

			let vs_defines_change = render_obj.vs_defines.add("SDF_CLIP");
			let (width, height)  = (layout.rect.right - layout.rect.left, layout.rect.bottom - layout.rect.top);
			let fs_defines_change = match &clip_path.0 {
				BaseShape::Circle { radius, center } => {
					let w = f32::sqrt(width * width + height * height);
					render_obj.paramter.set_single_uniform("clipSdf", UniformValue::MatrixV4(vec![
						len_value(&center.x, width), len_value(&center.y, height), len_value(radius, w), 0.0,
						0.0, 0.0, 0.0, 0.0,
						0.0, 0.0, 0.0, 0.0,
						0.0, 0.0, 0.0, 0.0,
					]));
					render_obj.fs_defines.add("CIRCLE")
				},
				BaseShape::Ellipse { rx, ry, center } => {
					render_obj.paramter.set_single_uniform("clipSdf", UniformValue::MatrixV4(vec![
						len_value(&center.x, width), len_value(&center.y, height), len_value(rx, width), len_value(ry, height),
						0.0, 0.0, 0.0, 0.0,
						0.0, 0.0, 0.0, 0.0,
						0.0, 0.0, 0.0, 0.0,
					]));
					render_obj.fs_defines.add("ELLIPSE")
				},
				BaseShape::Inset { rect_box, border_radius } => {
					let mut rect = Rect {
						left: len_value(&rect_box[0], height),
						right: width - len_value(&rect_box[1], width),
						top: len_value(&rect_box[2], height),
						bottom: height - len_value(&rect_box[3], width),
					};
					if rect.bottom < rect.top {
						rect.bottom = rect.top;
					}
					if rect.right < rect.left {
						rect.right = rect.left;
					}
					let (width, height)  = (rect.right - rect.left, rect.bottom - rect.top);

					let border_radius = cal_border_radius(border_radius, &rect);
					
					if border_radius.x[0] <= 0.0 && border_radius.x[1] <= 0.0 && border_radius.x[2] <= 0.0 && border_radius.x[3] <= 0.0 &&
					   border_radius.y[0] <= 0.0 && border_radius.y[1] <= 0.0 && border_radius.y[2] <= 0.0 && border_radius.y[3] <= 0.0 {
						render_obj.paramter.set_single_uniform("clipSdf", UniformValue::MatrixV4(vec![
							rect.left + width/2.0, rect.top + height/2.0, 1.0, 1.0,
							width/2.0, height/2.0, 0.0, 0.0,
							0.0, 0.0, 0.0, 0.0,
							0.0, 0.0, 0.0, 0.0,
						]));
						render_obj.fs_defines.remove("BORDER_RADIUS");
						render_obj.fs_defines.add("RECT")
					} else {
						render_obj.paramter.set_single_uniform("clipSdf", UniformValue::MatrixV4(vec![
							rect.left + width/2.0, rect.top + height/2.0, 1.0, 1.0,
							width/2.0, height/2.0, 0.0, 0.0,
							border_radius.y[0], border_radius.x[0], border_radius.x[1], border_radius.y[1],
							border_radius.y[2], border_radius.x[2], border_radius.x[3], border_radius.y[3],
						]));
						render_obj.fs_defines.remove("RECT");
						render_obj.fs_defines.add("BORDER_RADIUS")
					}
				},
				BaseShape::Sector { rotate, angle, radius, center } => {
					let half_angle = angle / 2.0;
					let half_rotate = rotate + half_angle;
					let w = f32::sqrt(width * width + height * height);
					render_obj.paramter.set_single_uniform("clipSdf", UniformValue::MatrixV4(vec![
						len_value(&center.x, width), len_value(&center.y, height), len_value(radius, w), 1.0,
						half_rotate.sin(), half_rotate.cos(), half_angle.sin(), half_angle.cos(),
						0.0, 0.0, 0.0, 0.0,
						0.0, 0.0, 0.0, 0.0,
					]));

					render_obj.fs_defines.add("SECTOR")
				}
			};
			if vs_defines_change.is_none() || fs_defines_change.is_none(){
				notify.modify_event(render_context.render_obj_index, "program_dirty", 0);
			}
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

#[inline]
pub fn center_offset_matrix(layout: &LayoutR, matrix: &WorldMatrix, transform: &Transform, h: f32, v: f32) -> WorldMatrix {
    // let depth = -depth / (Z_MAX + 1.0);
    // let depth = depth1;
	if let TransformOrigin::Center = transform.origin {
		return matrix.clone();
	}

	let width = layout.rect.right - layout.rect.left;
	let height = layout.rect.bottom - layout.rect.top;
    let origin = transform
        .origin
        .to_value(width, height);
    if origin.x == 0.0 && origin.y == 0.0 && h == 0.0 && v == 0.0 {
        matrix.clone()
    } else {
        matrix * WorldMatrix(Matrix4::new_translation(&Vector3::new(-origin.x + h + width, -origin.y + v + height, 0.0)), false)
    }
}

fn len_value( v: &LengthUnit, c: f32) -> f32 {
	match v {
		LengthUnit::Pixel(r) => *r,
		LengthUnit::Percent(r) => r * c,
	}
}

// 监听创建和修改事件，对其进行标记
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ClipPath, (CreateEvent, ModifyEvent)> for ClipPathSys<C> {
	type ReadData = ();
	type WriteData = &'a mut MultiCaseImpl<Node, RenderContextMark>;
	fn listen(&mut self, event: &Event, _: Self::ReadData, marks: Self::WriteData) {
		self.dirty.insert(event.id); // 插入到脏列表中
		marks[event.id].set(self.render_mark_index, true);
		marks.get_notify().modify_event(event.id, "", 0);
	}
}

// 监听删除事件，取消标记
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ClipPath, DeleteEvent> for ClipPathSys<C> {
	type ReadData = ();
	type WriteData = &'a mut MultiCaseImpl<Node, RenderContextMark>;
	fn listen(&mut self, event: &Event, _: Self::ReadData, marks: Self::WriteData) {
		self.dirty.insert(event.id); // 插入到脏列表中

		// 取消上下标记
		marks[event.id].set(self.render_mark_index, false);
		marks.get_notify().modify_event(event.id, "", 0);
	}
}

// ContentBox修改时，标记脏
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ContentBox, (CreateEvent ,ModifyEvent)> for ClipPathSys<C> {
	type ReadData = &'a MultiCaseImpl<Node, ClipPath>;
	type WriteData = &'a mut MultiCaseImpl<Node, RenderContextMark>;
	fn listen(&mut self, event: &Event, clip_paths: Self::ReadData, marks: Self::WriteData) {
		// 没有clipPath属性，不标记脏
		if clip_paths.get(event.id).is_none() {
			return;
		}
		self.dirty.insert(event.id); // 插入到脏列表中

		// 取消上下标记
		marks[event.id].set(self.render_mark_index, false);
		marks.get_notify().modify_event(event.id, "", 0);
	}
}


impl_system! {
	ClipPathSys<C> where [C: HalContext + 'static],
	true,
	{
		MultiCaseListener<Node, ClipPath, DeleteEvent>
		MultiCaseListener<Node, ClipPath, (CreateEvent, ModifyEvent)>
		MultiCaseListener<Node, ContentBox, (CreateEvent, ModifyEvent)>
	}
}