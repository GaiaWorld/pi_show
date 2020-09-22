use std::marker::PhantomData;
use std::os::raw::c_void;

use ecs::idtree::IdTree;
use ecs::{
    CreateEvent, DeleteEvent, EntityListener, ModifyEvent, MultiCaseImpl, MultiCaseListener,
    SingleCaseImpl, SingleCaseListener,
};

use component::calc::Layout;
use entity::Node;
use layout::{FlexNode, YGAlign, YGOverflow, YGWrap};

pub struct LayoutSys<L: FlexNode>(PhantomData<L>);

impl<L: FlexNode> LayoutSys<L> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

//插入Layout 和 L 组件
impl<'a, L: FlexNode> EntityListener<'a, Node, CreateEvent> for LayoutSys<L> {
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, Layout>,
        &'a mut MultiCaseImpl<Node, L>,
    );
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        write.0.insert(event.id, Layout::default());
        // let yoga = if event.id == 1 {
        // 	let config = L::C::new(); // 内存泄漏
        // 	config.set_point_scale_factor(0.0);
        // 	L::new_with_config(config)
        // }else {
        // 	L::default()
        // };
        let yoga = L::default();
        yoga.set_context(event.id as *mut c_void);
        yoga.set_overflow(YGOverflow::YGOverflowVisible);
        yoga.set_align_items(YGAlign::YGAlignStretch);
        yoga.set_flex_wrap(YGWrap::YGWrapWrap);

        // if event.id == 1 {
        // 	let config = L::C::new(); // 内存泄漏
        // 	config.set_point_scale_factor(0.0);

        // }
		write.1.insert(event.id, yoga);
    }
}

//插入Layout 和 L 组件
impl<'a, L: FlexNode> MultiCaseListener<'a, Node, L, DeleteEvent> for LayoutSys<L> {
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, Layout>,
        &'a mut MultiCaseImpl<Node, L>,
    );
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, write: Self::WriteData) {
        let yoga = &write.1[event.id];
        let p = yoga.get_parent();
        if !p.is_null() {
            p.remove_child(*yoga);
		}
		yoga.free();
    }
}

impl<'a, L: FlexNode> SingleCaseListener<'a, IdTree, ModifyEvent> for LayoutSys<L> {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, L>);
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData) {
        if event.index > 0 {
            if event.field == "add" {
                add_yoga(event.id, read.0, read.1);
            } else if event.field == "remove" {
                let parent_yoga = &read.1[event.index];
                parent_yoga.remove_child(read.1[event.id].clone());
            }
        }
    }
}

impl<'a, L: FlexNode> SingleCaseListener<'a, IdTree, CreateEvent> for LayoutSys<L> {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, L>);
    type WriteData = ();
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, _write: Self::WriteData) {
        add_yoga(event.id, read.0, read.1);
    }
}

impl<'a, L: FlexNode> SingleCaseListener<'a, IdTree, DeleteEvent> for LayoutSys<L> {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, L>);
    type WriteData = ();
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, _write: Self::WriteData) {
		let node = &read.0[event.id];
        if node.parent > 0 {
            let parent_yoga = &read.1[node.parent];
            parent_yoga.remove_child(read.1[event.id].clone());
        }
    }
}

fn add_yoga<L: FlexNode>(
    id: usize,
    idtree: &SingleCaseImpl<IdTree>,
    yogas: &MultiCaseImpl<Node, L>,
) {
    let node = &idtree[id];
	let yoga = &yogas[id];
	let parent_yoga = yoga.get_parent();
	if !parent_yoga.is_null() {
		parent_yoga.remove_child(yoga.clone());
	}
    if node.parent > 0 {
        let parent_yoga = &yogas[node.parent];
        let child_count = parent_yoga.get_child_count();
        let mut index = child_count;
        if node.next > 0 {
            index -= 1;
            let next_yoga = yogas[node.next].clone();
            while parent_yoga.get_child(index) != next_yoga {
                index -= 1;
            }
        }
        parent_yoga.insert_child(yoga.clone(), index);
    }
}

impl_system! {
    LayoutSys<L> where [L: FlexNode],
    false,
    {
        EntityListener<Node, CreateEvent>
        MultiCaseListener<Node, L, DeleteEvent>
        SingleCaseListener<IdTree, CreateEvent>
        SingleCaseListener<IdTree, DeleteEvent>
        SingleCaseListener<IdTree, ModifyEvent>
    }
}
