/// 布局系统
/// 1.负责处理布局属性的脏，根据不同的脏，设置flex_layout节点的脏类型
/// 负责推动flex_layout节点进行布局
use single::{IdTree, DirtyList};
use ecs::{
    CreateEvent, DeleteEvent, EntityListener, ModifyEvent, MultiCaseImpl,
    SingleCaseImpl, SingleCaseListener,Runner
};
use component::user::{OtherLayoutStyle, RectLayoutStyle};
use component::calc::{LayoutR, StyleType1, StyleMark, NodeState};
use flex_layout::*;
use dirty::*;
use map::vecmap::VecMap;
use util::vecmap_default::VecMapWithDefault;

use entity::Node;

// 矩形区域脏，绝对定位下，设自身self_dirty，相对定位下，设自身self_dirty后，还要设父child_dirty
pub const RECT_DIRTY: usize = StyleType1::Create as usize
							| StyleType1::Width as usize 
							| StyleType1::Height as usize
							| StyleType1::Position as usize
							| StyleType1::Margin as usize;

// 普通脏及子节点添加或移除， 设父child_dirty
pub const NORMAL_DIRTY: usize = StyleType1::FlexBasis as usize 
							//| StyleType1::Order as usize
							| StyleType1::FlexShrink as usize
							| StyleType1::FlexGrow as usize
							| StyleType1::AlignSelf as usize
							| StyleType1::PositionType as usize;

// 自身脏， 仅设自身self_dirty
pub const SELF_DIRTY: usize = StyleType1::Padding as usize 
							| StyleType1::Border as usize;

// 子节点脏， 仅设自身child_dirty
pub const CHILD_DIRTY: usize = StyleType1::FlexDirection as usize
							| StyleType1::FlexWrap as usize
							| StyleType1::AlignItems as usize
							| StyleType1::JustifyContent as usize
							| StyleType1::AlignContent as usize;


pub const DIRTY: usize = RECT_DIRTY | NORMAL_DIRTY | SELF_DIRTY | CHILD_DIRTY | StyleType1::Display as usize;


#[derive(Default)]
pub struct LayoutSys{
	dirty: LayerDirty,
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
		if dirty_list.0.len() == 0 {
            return;
		}
		
		let flex_rect_styles = unsafe {&mut *(rect_layout_styles.get_storage() as *const VecMapWithDefault<RectLayoutStyle> as usize as *mut VecMapWithDefault<flex_layout::RectStyle>)};
		let flex_other_styles = unsafe {&mut *(other_layout_styles.get_storage() as *const VecMapWithDefault<OtherLayoutStyle> as usize as *mut VecMapWithDefault<flex_layout::OtherStyle>)};
		let flex_layouts = unsafe {&mut *(layouts.get_storage() as *const VecMap<LayoutR> as usize as *mut VecMap<flex_layout::LayoutR>)};
		let node_states = unsafe {&mut *(node_states.get_storage() as *const VecMap<NodeState> as usize as *mut VecMap<flex_layout::INode>)};
        for id in dirty_list.0.iter() {
			let style_mark = match style_marks.get_mut(*id) {
                Some(r) => r,
                None => continue,
            };
			let dirty = style_mark.dirty1;

            // 不存在LayoutTree关心的脏, 跳过
            if dirty & DIRTY == 0 {
                continue;
			}

			// println!("dirty======{:?}, {:?}", id, &flex_styles[*id]);
			let rect_style = &flex_rect_styles[*id];
			let other_style = &flex_other_styles[*id];

			if dirty & RECT_DIRTY != 0 {
				// println!("dirty rect======{:?}, parent:{}", id, tree[*id].parent());
				set_rect(tree, node_states, &mut self.dirty, *id, rect_style, other_style, true, true);
			}

			if dirty & NORMAL_DIRTY != 0 {
				// println!("dirty NORMAL_DIRTY======{:?}", id);
				set_normal_style(tree, node_states, &mut self.dirty, *id, other_style);
			}

			if dirty & SELF_DIRTY != 0 {
				// println!("dirty SELF_DIRTY======{:?}", id);
				set_self_style(tree, node_states, &mut self.dirty, *id, other_style);
			}

			if dirty & CHILD_DIRTY as usize != 0 {
				set_children_style(tree, node_states, &mut self.dirty, *id, other_style);
			}

			if dirty & StyleType1::Display as usize != 0 {
				set_display(*id, other_style.display, &mut self.dirty, tree, node_states, flex_rect_styles, flex_other_styles);
			}
			style_mark.dirty1 &= !DIRTY;
		}
		compute(&mut self.dirty, tree, node_states, flex_rect_styles, flex_other_styles, flex_layouts, notify, layouts);
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
	fn listen(&mut self, event: &CreateEvent, _tree: Self::ReadData, (rect_layout_styles, other_layout_styles, layouts, node_states): Self::WriteData) {
		// rect_layout_styles.insert(event.id, RectLayoutStyle::default());
		// other_layout_styles.insert(event.id, OtherLayoutStyle::default());
		layouts.insert(event.id, LayoutR::default());
		node_states.insert(event.id, NodeState::default());
	}
}

impl<'a> SingleCaseListener<'a, IdTree, ModifyEvent> for LayoutSys {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (&'a mut  MultiCaseImpl<Node, NodeState>, &'a mut  MultiCaseImpl<Node, RectLayoutStyle>, &'a mut  MultiCaseImpl<Node, OtherLayoutStyle>);
    fn listen(&mut self, event: &ModifyEvent, tree: Self::ReadData, (node_states, _rect_layout_styles, other_layout_styles): Self::WriteData) {
		// let flex_rect_styles = unsafe {&mut *(rect_layout_styles.get_storage() as *const VecMap<RectLayoutStyle> as usize as *mut VecMap<flex_layout::RectStyle>)};
		let flex_other_styles = unsafe {&mut *(other_layout_styles.get_storage() as *const VecMapWithDefault<OtherLayoutStyle> as usize as *mut VecMapWithDefault<flex_layout::OtherStyle>)};
		let node_states = unsafe {&mut *(node_states.get_storage() as *const VecMap<NodeState> as usize as *mut VecMap<flex_layout::INode>)};
		if event.field == "add" {
			set_normal_style(tree, node_states, &mut self.dirty, event.id, &flex_other_styles[event.id]);
		} if event.field == "remove"{
			let parent = tree[event.id].parent();
			if parent > 0 {
				mark_children_dirty(tree, node_states, &mut self.dirty, parent);
			}
		}
    }
}

impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for LayoutSys {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = &'a mut MultiCaseImpl<Node, NodeState>;
    fn listen(&mut self, event: &DeleteEvent, tree: Self::ReadData, node_states: Self::WriteData) {
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
		SingleCaseListener<IdTree, ModifyEvent>
    }
}
