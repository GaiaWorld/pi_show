use std::os::raw::{c_void};
use std::marker::PhantomData;

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl};
use ecs::idtree::{IdTree};
use ecs::Share;

use component::user::*;
use layout::{FlexNode, YGOverflow, YGAlign};
use entity::{Node};

pub struct LayoutSys<L: FlexNode>(PhantomData<L>);

impl<L: FlexNode> LayoutSys<L> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

//插入Layout 和 L 组件
impl<'a, L: FlexNode + Share> EntityListener<'a, Node, CreateEvent> for LayoutSys<L>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, Layout>, &'a mut MultiCaseImpl<Node, L>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
      write.0.insert(event.id, Layout::default());
			let yoga = L::default();
			yoga.set_context(event.id as *mut c_void);
			yoga.set_overflow(YGOverflow::YGOverflowVisible);
            yoga.set_align_items(YGAlign::YGAlignFlexStart);
      write.1.insert(event.id, yoga);
    }
}

impl<'a, L: FlexNode + Share> SingleCaseListener<'a, IdTree, ModifyEvent> for LayoutSys<L>{
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, L>);
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData){
			if event.index > 0 {
				if event.field == "add" {
					add_yoga(event.id, read.0, read.1);
				}else if event.field == "remove"{
					let parent_yoga = unsafe { read.1.get_unchecked(event.index)};
					parent_yoga.remove_child(unsafe { read.1.get_unchecked(event.id)}.clone());
				}
			}
    }
}

impl<'a, L: FlexNode + Share> SingleCaseListener<'a, IdTree, CreateEvent> for LayoutSys<L>{
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, L>);
    type WriteData = ();
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, _write: Self::WriteData){
			add_yoga(event.id, read.0, read.1);
    }
}

impl<'a, L: FlexNode + Share> SingleCaseListener<'a, IdTree, DeleteEvent> for LayoutSys<L>{
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, L>);
    type WriteData = ();
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, _write: Self::WriteData){
		let node = unsafe { read.0.get_unchecked(event.id) };
		if node.parent > 0 {
			let parent_yoga = unsafe { read.1.get_unchecked(node.parent)};
			parent_yoga.remove_child(unsafe { read.1.get_unchecked(event.id)}.clone());
		}
    }
}

fn add_yoga<L: FlexNode + Share>(id: usize, idtree: &SingleCaseImpl<IdTree>, yogas: &MultiCaseImpl<Node, L>){
	let node = unsafe { idtree.get_unchecked(id) };
	let yoga = unsafe { yogas.get_unchecked(id) };
	if node.parent > 0 {
		let parent_yoga = unsafe { yogas.get_unchecked(node.parent)};
		let child_count = parent_yoga.get_child_count();
		let mut index = child_count;
		if node.next > 0 {
		index -= 1;
		let next_yoga = unsafe { yogas.get_unchecked(node.next)}.clone();
		while parent_yoga.get_child(index) != next_yoga {
			index-=1;
		}
		}
		parent_yoga.insert_child(yoga.clone(), index);
	}
}


impl_system!{
    LayoutSys<L> where [L: FlexNode + Share],
    false,
    {
				EntityListener<Node, CreateEvent>
        SingleCaseListener<IdTree, CreateEvent>
        SingleCaseListener<IdTree, DeleteEvent>
				SingleCaseListener<IdTree, ModifyEvent>
    }
}