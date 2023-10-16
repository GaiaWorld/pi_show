/// 布局系统
/// 1.负责处理布局属性的脏，根据不同的脏，设置flex_layout节点的脏类型
/// 负责推动flex_layout节点进行布局
use ecs::{CreateEvent, DeleteEvent, EntityListener, Event, MultiCaseImpl, Runner, SingleCaseImpl, SingleCaseListener};
use flex_layout::*;
use dirty::*;
use map::vecmap::VecMap;

use crate::util::vecmap_default::VecMapWithDefault;
use crate::entity::Node;
use crate::single::{IdTree, DirtyList};
use crate::component::user::{OtherLayoutStyle, RectLayoutStyle};
use crate::component::calc::{LayoutR, StyleMark, NodeState, LAYOUT_MARGIN_MARK, LAYOUT_POSITION_MARK, LAYOUT_BORDER_MARK, LAYOUT_PADDING_MARK, CalcType, StyleBit, style_bit};
use crate::component::user::StyleType;

lazy_static! {
	// 矩形区域脏，绝对定位下，设自身self_dirty，相对定位下，设自身self_dirty后，还要设父child_dirty
	pub static ref RECT_DIRTY: StyleBit = style_bit().set_bit(StyleType::Width as usize)
	.set_bit(StyleType::Height as usize)
		| &*LAYOUT_POSITION_MARK
		| &*LAYOUT_MARGIN_MARK;

	// 普通脏及子节点添加或移除， 设父child_dirty
	pub static ref NORMAL_DIRTY: StyleBit = //StyleType::FlexBasis as usize 
		//.set_bit(StyleType::Order as usize)
		style_bit().set_bit(StyleType::FlexShrink as usize)
		.set_bit(StyleType::FlexGrow as usize)
		.set_bit(StyleType::AlignSelf as usize)
		.set_bit(StyleType::PositionType as usize);

	// 自身脏， 仅设自身self_dirty
	pub static ref SELF_DIRTY: StyleBit = LAYOUT_PADDING_MARK.clone() 
		| &*LAYOUT_BORDER_MARK;

	// 子节点脏， 仅设自身child_dirty
	pub static ref CHILD_DIRTY: StyleBit = style_bit().set_bit(StyleType::FlexDirection as usize)
		.set_bit(StyleType::FlexWrap as usize)
		.set_bit(StyleType::AlignItems as usize)
		.set_bit(StyleType::JustifyContent as usize)
		.set_bit(StyleType::AlignContent as usize)
		.set_bit(StyleType::TextContent as usize);


	pub static ref DIRTY2: StyleBit = style_bit().set_bit(StyleType::Display as usize).set_bit(StyleType::FlexBasis as usize) | &*RECT_DIRTY | &*NORMAL_DIRTY | &*SELF_DIRTY | &*CHILD_DIRTY;

}



#[derive(Default)]
pub struct LayoutSys{
	dirty: LayerDirty<usize>,
}

impl<'a> Runner<'a> for LayoutSys {
	type ReadData = ( 
		&'a MultiCaseImpl<Node, RectLayoutStyle>,
		&'a MultiCaseImpl<Node, OtherLayoutStyle>,
		&'a SingleCaseImpl<IdTree>, 
		&'a SingleCaseImpl<DirtyList>);
	type WriteData = (
		&'a mut MultiCaseImpl<Node, LayoutR>,
		&'a mut MultiCaseImpl<Node, NodeState>,
		&'a mut MultiCaseImpl<Node, StyleMark>, );
    fn run(&mut self, (rect_layout_styles, other_layout_styles, tree, dirty_list, ): Self::ReadData, (layouts, node_states, style_marks): Self::WriteData) {
		let time = cross_performance::now();
		if dirty_list.0.len() == 0 {
            return;
		}
		
		let flex_rect_styles = unsafe {&mut *(rect_layout_styles.get_storage() as *const VecMapWithDefault<RectLayoutStyle> as usize as *mut VecMapWithDefault<flex_layout::RectStyle>)};
		let flex_other_styles = unsafe {&mut *(other_layout_styles.get_storage() as *const VecMapWithDefault<OtherLayoutStyle> as usize as *mut VecMapWithDefault<flex_layout::OtherStyle>)};
		let flex_layouts = unsafe {&mut *(layouts.get_storage() as *const VecMap<LayoutR> as usize as *mut VecMap<flex_layout::LayoutR>)};
		let node_states = unsafe {&mut *(node_states.get_storage() as *const VecMap<NodeState> as usize as *mut VecMap<flex_layout::INode>)};
		
		// log::info!("dirty_list============={:?}", dirty_list.0);
		for id in dirty_list.0.iter() {
			let style_mark = match style_marks.get_mut(*id) {
                Some(r) => r,
                None => continue,
            };
			match tree.get(*id) {
                Some(r) => if r.layer() == 0 {continue},
                None => continue,
            };
			// let dirty2 = style_mark.dirty2;
			let dirty1 = style_mark.dirty1;
			let dirty = style_mark.dirty;
			// log::info!("layout dirty============={}, {}, {}", dirty2, dirty1, dirty2 & RECT_DIRTY);

            // 不存在LayoutTree关心的脏, 跳过
            if !(dirty & DIRTY2.set_bit(StyleType::FontSize as usize)).any() && dirty1 & CalcType::Create as usize == 0 {
                continue;
			}

			// log::info!("layout dirty1============={}", id);

			// println!("dirty======{:?}, {:?}", id, &flex_styles[*id]);
			let rect_style = &flex_rect_styles[*id];
			let other_style = &flex_other_styles[*id];

			if (dirty & &*RECT_DIRTY).any() || dirty1 & CalcType::Create as usize != 0 || dirty[StyleType::FontSize as usize] {
				set_rect(tree, node_states, &mut self.dirty, *id, rect_style, other_style, true, true);
			}

			if (dirty & &*NORMAL_DIRTY).any() || dirty1 & StyleType::FlexBasis as usize != 0 {
				// println!("dirty NORMAL_DIRTY======{:?}", id);
				set_normal_style(tree, node_states, &mut self.dirty, *id, other_style);
			}

			if (dirty & &*SELF_DIRTY).any() {
				// println!("dirty SELF_DIRTY======{:?}", id);
				set_self_style(tree, node_states, &mut self.dirty, *id, other_style);
			}

			if (dirty & &*CHILD_DIRTY).any(){
				set_children_style(tree, node_states, &mut self.dirty, *id, other_style);
			}

			if dirty1 & StyleType::Display as usize != 0 {
				set_display(*id, other_style.display, &mut self.dirty, tree, node_states, rect_style, other_style);
			}
			style_mark.dirty &= !*DIRTY2;
			style_mark.dirty1 &= !(CalcType::Create as usize);
		}
		let count = self.dirty.count();
		compute(&mut self.dirty, tree, node_states, flex_rect_styles, flex_other_styles, flex_layouts, notify, layouts);
		// if count > 0 {
		// 	log::warn!("layout======={:?}", cross_performance::now() - time);
		// }
	}
}


//节点创建时， 默认为节点创建LayoutStyle组件
impl<'a> EntityListener<'a, Node, CreateEvent> for LayoutSys {
	type ReadData = &'a SingleCaseImpl<IdTree>;
	type WriteData = (
		&'a mut MultiCaseImpl<Node, RectLayoutStyle>, 
		&'a mut MultiCaseImpl<Node, OtherLayoutStyle>, 
		&'a mut MultiCaseImpl<Node, LayoutR>, 
		&'a mut MultiCaseImpl<Node, NodeState>);
	fn listen(&mut self, event: &Event, _tree: Self::ReadData, (rect_layout_styles, other_layout_styles, layouts, node_states): Self::WriteData) {
		// rect_layout_styles.insert(event.id, RectLayoutStyle::default());
		// other_layout_styles.insert(event.id, OtherLayoutStyle::default());
		layouts.insert(event.id, LayoutR::default());
		node_states.insert(event.id, NodeState::default());
	}
}


pub fn aa() {
	Vec::new().push(1);
}
// impl<'a> SingleCaseListener<'a, IdTree, ModifyEvent> for LayoutSys {
//     type ReadData = &'a SingleCaseImpl<IdTree>;
//     type WriteData = (&'a mut  MultiCaseImpl<Node, NodeState>, &'a mut  MultiCaseImpl<Node, RectLayoutStyle>, &'a mut  MultiCaseImpl<Node, OtherLayoutStyle>);
//     fn listen(&mut self, event: &Event, tree: Self::ReadData, (node_states, _rect_layout_styles, other_layout_styles): Self::WriteData) {
// 		// let flex_rect_styles = unsafe {&mut *(rect_layout_styles.get_storage() as *const VecMap<RectLayoutStyle> as usize as *mut VecMap<flex_layout::RectStyle>)};
		
// 		// if event.field == "add" {
// 		// let flex_other_styles = unsafe {&mut *(other_layout_styles.get_storage() as *const VecMapWithDefault<OtherLayoutStyle> as usize as *mut VecMapWithDefault<flex_layout::OtherStyle>)};
// 		// 	set_normal_style(tree, node_states, &mut self.dirty, event.id, &flex_other_styles[event.id]);
// 		// }
// 		 if event.field == "remove"{

// 			let node_states = unsafe {&mut *(node_states.get_storage() as *const VecMap<NodeState> as usize as *mut VecMap<flex_layout::INode>)};
// 			let parent = tree[event.id].parent();
// 			if parent > 0 {
// 				mark_children_dirty(tree, node_states, &mut self.dirty, parent);
// 			}
// 		}
//     }
// }

// impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for LayoutSys {
//     type ReadData = &'a SingleCaseImpl<IdTree>;
//     type WriteData = (&'a mut  MultiCaseImpl<Node, NodeState>, &'a mut  MultiCaseImpl<Node, RectLayoutStyle>, &'a mut  MultiCaseImpl<Node, OtherLayoutStyle>);
//     fn listen(&mut self, event: &Event, tree: Self::ReadData, (node_states, _rect_layout_styles, other_layout_styles): Self::WriteData) {
// 		// log::info!("idtree create============={}", event.id);
// 		let flex_other_styles = unsafe {&mut *(other_layout_styles.get_storage() as *const VecMapWithDefault<OtherLayoutStyle> as usize as *mut VecMapWithDefault<flex_layout::OtherStyle>)};
// 		let node_states = unsafe {&mut *(node_states.get_storage() as *const VecMap<NodeState> as usize as *mut VecMap<flex_layout::INode>)};
// 		set_normal_style(tree, node_states, &mut self.dirty, event.id, &flex_other_styles[event.id]);
//     }
// }

impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for LayoutSys {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = &'a mut MultiCaseImpl<Node, NodeState>;
    fn listen(&mut self, event: &Event, tree: Self::ReadData, node_states: Self::WriteData) {
		let node_states = unsafe {&mut *(node_states.get_storage() as *const VecMap<NodeState> as usize as *mut VecMap<flex_layout::INode>)};
		let parent = tree[event.id].parent();
		if parent > 0 {
			mark_children_dirty(tree, node_states, &mut self.dirty, parent);
		}
    }
}

fn notify(context: &mut MultiCaseImpl<Node, LayoutR>, id: usize, _layout:&flex_layout::LayoutR) {
	// println!("notify======================={}, layout:{:?}", id, layout);
	context.get_notify_ref().modify_event(id, "", 0);
}


impl_system! {
    LayoutSys,
    true,
    {
        EntityListener<Node, CreateEvent>
        SingleCaseListener<IdTree, DeleteEvent>
		// SingleCaseListener<IdTree, ModifyEvent>
		// SingleCaseListener<IdTree, CreateEvent>
    }
}
