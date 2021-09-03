use std::ops::{Index, IndexMut};

use bevy_ecs::prelude::{Query, Res, ResMut, In};
/// 布局系统
/// 1.负责处理布局属性的脏，根据不同的脏，设置flex_layout节点的脏类型
/// 负责推动flex_layout节点进行布局
use flex_layout::*;
use dirty::*;

use crate::util::event::{EntityEvent, ImMessenger};
use crate::single::{IdTree, DirtyList, to_entity};
use crate::component::user::{OtherLayoutStyle, RectLayoutStyle};
use crate::component::calc::{LayoutR, StyleType1, StyleMark, NodeState, StyleType2, StyleIndex, LAYOUT_MARGIN_MARK, LAYOUT_POSITION_MARK, LAYOUT_BORDER_MARK, LAYOUT_PADDING_MARK};

// 矩形区域脏，绝对定位下，设自身self_dirty，相对定位下，设自身self_dirty后，还要设父child_dirty
pub const RECT_DIRTY: usize = StyleType2::Width as usize 
							| StyleType2::Height as usize
							| LAYOUT_POSITION_MARK
							| LAYOUT_MARGIN_MARK;

// 普通脏及子节点添加或移除， 设父child_dirty
pub const NORMAL_DIRTY: usize = //StyleType2::FlexBasis as usize 
							//| StyleType1::Order as usize
							StyleType2::FlexShrink as usize
							| StyleType2::FlexGrow as usize
							| StyleType2::AlignSelf as usize
							| StyleType2::PositionType as usize;

// 自身脏， 仅设自身self_dirty
pub const SELF_DIRTY: usize = LAYOUT_PADDING_MARK 
							| LAYOUT_BORDER_MARK;

// 子节点脏， 仅设自身child_dirty
pub const CHILD_DIRTY: usize = StyleType2::FlexDirection as usize
							| StyleType2::FlexWrap as usize
							| StyleType2::AlignItems as usize
							| StyleType2::JustifyContent as usize
							| StyleType2::AlignContent as usize;


pub const DIRTY2: usize = RECT_DIRTY | NORMAL_DIRTY | SELF_DIRTY | CHILD_DIRTY;

pub fn idtree_listen(
	e: In<EntityEvent<IdTree>>,
	other_layout_styles: Query<&OtherLayoutStyle>,
	idtree: Res<IdTree>,
	mut node_states: Query<&mut NodeState>,
	mut local: ResMut<LayoutSys>,
) {
	let e = &e.0;
	let flex_other_styles = IndexMap {idtree: unsafe {std::mem::transmute(&idtree)}, value: unsafe {std::mem::transmute(&other_layout_styles)}};
	let mut node_states = IndexMapMut {idtree: unsafe {std::mem::transmute(&idtree)}, value: unsafe {std::mem::transmute(&mut node_states)}};

	// log::info!("idtree_listen============{:?}", **e);
	match e.style_index {
		StyleIndex::Add | StyleIndex::Create => {
			set_normal_style(&**idtree, &mut node_states, &mut local.dirty, e.id.id() as usize, &flex_other_styles[e.id.id() as usize]);
		},
		StyleIndex::Delete => {
			let parent = idtree[e.id.id() as usize].parent();
			if parent < usize::max_value() {
				mark_children_dirty(&**idtree, &mut node_states, &mut local.dirty, parent);
			}
		},
		_ => (),
	}
}

pub fn calc_layout(
	rect_layout_styles: Query<&RectLayoutStyle>,
	other_layout_styles: Query<&OtherLayoutStyle>,
	mut layout_r: Query<&mut LayoutR>,
	mut node_states: Query<&mut NodeState>,
	mut style_marks: Query<&mut StyleMark>,
	idtree: Res<IdTree>,
	dirty_list: Res<DirtyList>,
	mut local: ResMut<LayoutSys>,

	mut layout_event_writer: ImMessenger<EntityEvent<LayoutR>>,
) {
	let time = cross_performance::now();
	let flex_rect_styles = IndexMap {idtree: unsafe {std::mem::transmute(&idtree)} , value: unsafe {std::mem::transmute(&rect_layout_styles)}};
	let flex_other_styles = IndexMap {idtree: unsafe {std::mem::transmute(&idtree)}, value: unsafe {std::mem::transmute(&other_layout_styles)}};
	let mut node_states = IndexMapMut {idtree: unsafe {std::mem::transmute(&idtree)}, value: unsafe {std::mem::transmute(&mut node_states)}};
	// 迭代idtree的事件，设置脏

	// log::info!("calc_layout============{:?}", dirty_list.0.len());
	// 遍历脏节点，设置脏
	if dirty_list.0.len() == 0 {
		return;
	}
	
	for id in dirty_list.0.iter() {
		let mut style_mark = match style_marks.get_mut(*id) {
			Ok(r) => r,
			Err(_r) => continue,
		};
		let dirty = style_mark.dirty[2];
		let dirty1 = style_mark.dirty[1];

		// 不存在LayoutTree关心的脏, 跳过
		if dirty & DIRTY2 == 0 && dirty1 & StyleType1::Display as usize == 0 && dirty1 & StyleType1::FlexBasis as usize == 0 && dirty1 & StyleType1::Create as usize == 0 {
			continue;
		}

		// println!("dirty======{:?}, {:?}", id, &flex_styles[*id]);
		let id = id.id() as usize;
		let rect_style = &flex_rect_styles[id];
		let other_style = &flex_other_styles[id];

		if dirty & RECT_DIRTY != 0 || dirty1 & StyleType1::Create as usize != 0 {
			set_rect(&**idtree, &mut node_states, &mut local.dirty, id, rect_style, other_style, true, true);
		}

		if dirty & NORMAL_DIRTY != 0 || dirty1 & StyleType1::FlexBasis as usize != 0 {
			// println!("dirty NORMAL_DIRTY======{:?}", id);
			set_normal_style(&**idtree, &mut node_states, &mut local.dirty, id, other_style);
		}

		if dirty & SELF_DIRTY != 0 {
			// println!("dirty SELF_DIRTY======{:?}", id);
			set_self_style(&**idtree, &mut node_states, &mut local.dirty, id, other_style);
		}

		if dirty & CHILD_DIRTY as usize != 0 {
			set_children_style(&**idtree, &mut node_states, &mut local.dirty, id, other_style);
		}

		if dirty1 & StyleType1::Display as usize != 0 {
			set_display(id, other_style.display, &mut local.dirty, &**idtree, &mut node_states, rect_style, other_style);
		}
		style_mark.dirty[2] &= !DIRTY2;
		style_mark.dirty[1] &= !(StyleType1::Display as usize | StyleType1::FlexBasis as usize | StyleType1::Create as usize);
	}

	let flex_layouts = IndexMapMut {idtree: unsafe {std::mem::transmute(&idtree)}, value: unsafe {std::mem::transmute(&mut layout_r)}};
	let flex_layouts = unsafe {&mut *(&flex_layouts  as *const IndexMapMut<LayoutR> as usize as *mut IndexMapMut<flex_layout::LayoutR>)};
	
	let mut event_sender = EventSender {
		idtree: &idtree,
		sender: &mut layout_event_writer,
	};
	compute(&mut local.dirty, &**idtree, &mut node_states, &flex_rect_styles, &flex_other_styles, flex_layouts, notify, &mut event_sender);
	// if local.dirty.count() > 0 {
	// 	log::info!("layout======={:?}", cross_performance::now() - time);
	// }
}

fn notify(event_sender: &mut EventSender, id: usize, _layout:&flex_layout::LayoutR) {
	let version = event_sender.idtree[id].data;
	event_sender.sender.send(EntityEvent::new_modify(to_entity(id, version), StyleIndex::Layout));
}

struct EventSender<'a,'b> {
	idtree: &'a IdTree,
	sender: &'a mut ImMessenger<'b, EntityEvent<LayoutR>>
}

#[derive(Default)]
pub struct LayoutSys{
	pub dirty: LayerDirty<usize>,
}

pub struct IndexMap<'a, 'b, 'c, T: Send + Sync + 'static> {
	idtree: &'a Res<'b, IdTree>,
	value: &'a Query<'b, &'c T>,
}

pub struct IndexMapMut<'a, 'b, 'c, T: Send + Sync + 'static> {
	idtree: &'a Res<'b, IdTree>,
	value: &'a mut Query<'b, &'c mut T>,
}

impl<'a, 'b, 'c, T: Send + Sync + 'static> Index<usize> for IndexMap<'a, 'b, 'c, T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
		let n = &self.idtree[index];
		unsafe { self.value.get_unchecked(to_entity(index, n.data)).unwrap() }
	}
}

impl<'a, 'b, 'c, T: Send + Sync + 'static> Index<usize> for IndexMapMut<'a, 'b, 'c, T> {
	type Output = T;
    fn index(&self, index: usize) -> &T {
		let n = &self.idtree[index];
		unsafe { &*(&*self.value.get_unchecked(to_entity(index, n.data)).unwrap() as *const T) }
	}
}

impl<'a, 'b, 'c, T: Send + Sync + 'static> IndexMut<usize> for IndexMapMut<'a, 'b, 'c, T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
		let n = &self.idtree[index];
		unsafe { &mut *(&mut *self.value.get_unchecked(to_entity(index, n.data)).unwrap() as *mut T) }
	}
}


// impl_system! {
//     LayoutSys,
//     true,
//     {
//         EntityListener<Node, CreateEvent>
//         SingleCaseListener<IdTree, DeleteEvent>
// 		SingleCaseListener<IdTree, ModifyEvent>
// 		SingleCaseListener<IdTree, CreateEvent>
//     }
// }
