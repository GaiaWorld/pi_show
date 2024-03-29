/**
 * 监听TransformWillChange的改变， 修改TransformWillChangeMatrix
	*/

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
use single::IdTree;
use dirty::LayerDirty;

use component::user::{ Transform };
use component::calc::{ WorldMatrix, StyleMark, StyleType1, TransformWillChangeMatrix, LayoutR };
use single::DefaultTable;

use component::user::*;
use entity::{Node};

#[derive(Default)]
pub struct TransformWillChangeSys{
	will_change_mark: LayerDirty, // 存放存在TransformWillChang的组件id， 并不是为了记脏
	dirty: bool,
	// TransformWillChang创建时， 如果该节点不在根树上， 该值会+1， 当节点树被添加到根上时， 遍历子孙节点，如果节点上存在TransformWillChang， 该值-1， 直到减为0， 会停止遍历
	// 因为存在TransformWillChang的节点数量应该是少量的， 记录该值， 可以减少每次节点被添加到根时的遍历
	create_will_change_count: usize, 
}

impl TransformWillChangeSys{

	#[inline]
	fn mark_dirty(&mut self, id: usize, style_marks: &mut MultiCaseImpl<Node, StyleMark>) {
		self.dirty = true;
		let mark = &mut style_marks[id];
		mark.dirty_other |= StyleType1::TransformWillChange as usize;
	}
}

impl<'a> Runner<'a> for TransformWillChangeSys{
	type ReadData = (
		&'a SingleCaseImpl<DefaultTable>,
		&'a SingleCaseImpl<IdTree>,
		&'a MultiCaseImpl<Node, LayoutR>,
		&'a MultiCaseImpl<Node, TransformWillChange>,
		&'a MultiCaseImpl<Node, WorldMatrix>,
		&'a MultiCaseImpl<Node, Transform>,
	);
	type WriteData = (
		&'a mut MultiCaseImpl<Node, StyleMark>,
		&'a mut MultiCaseImpl<Node, TransformWillChangeMatrix>,
	);
	fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
		if !self.dirty {
			return;
		}
		// let time = std::time::Instant::now();
		self.dirty = false;
		let (default_table, idtree, layouts, transform_will_changes, world_matrixs, transforms) = read;
		let (style_marks, transform_will_change_matrixs) = write;
		let mut count = self.will_change_mark.count();
		let default_transform = default_table.get::<Transform>().unwrap();
		let mut deletes = Vec::default();
		for (id, layer) in self.will_change_mark.iter() {
			if count == 0 {
				break;
			}
			count -= 1;
			// 节点已经不存在， 应该删除willchange标记
			let style_mark = match style_marks.get_mut(*id) {
				Some(r) => r,
				None => {
					deletes.push((*id, layer));
					continue;
				},
			};

			// 不存在willchange， 应该删除willchange标记
			if let None = transform_will_changes.get(*id) {
				deletes.push((*id, layer));
				continue;
			}

			// 节点不在树上， 应该删除willchange标记
			let node = &idtree[*id];
			if node.layer() == 0 {
				deletes.push((*id, layer));
				continue;
			}

			// TransformWillChange不脏
			if style_mark.dirty_other | StyleType1::TransformWillChange as usize == 0 {
				continue;
			}

			recursive_cal_matrix(
				*id,
				node.parent(), 
				count + 1, 
				None, 
				default_transform, 
				idtree,
				transform_will_changes, 
				layouts, 
				world_matrixs, 
				transforms, 
				style_marks,
				transform_will_change_matrixs);
		}

		for (id, layer) in deletes.iter() {
			self.will_change_mark.delete(*id, *layer);
		}
		// println!("TransformWillChangeSys run : {:?}", std::time::Instant::now() - time);
	}
}

impl<'a> MultiCaseListener<'a, Node, TransformWillChange, CreateEvent> for TransformWillChangeSys{
	type ReadData = &'a SingleCaseImpl<IdTree>;
	type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
	fn listen(&mut self, event: &CreateEvent, idtree: Self::ReadData, style_marks: Self::WriteData){
		if let Some(r) = idtree.get(event.id) {
			if r.layer() > 0 {
				self.mark_dirty(event.id, style_marks);
				self.will_change_mark.mark(event.id, r.layer());
			}
		} else {
			self.create_will_change_count += 1;
		} 
	}
}

impl<'a> MultiCaseListener<'a, Node, TransformWillChange, ModifyEvent> for TransformWillChangeSys{
	type ReadData = &'a SingleCaseImpl<IdTree>;
	type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
	fn listen(&mut self, event: &ModifyEvent, idtree: Self::ReadData, style_marks: Self::WriteData){
		if let Some(r) = idtree.get(event.id) {
			if r.layer() > 0 {
				self.mark_dirty(event.id, style_marks);
			}
		}
	}
}

// 删除TransformWillChange组件， 标记脏
impl<'a> MultiCaseListener<'a, Node, TransformWillChange, DeleteEvent> for TransformWillChangeSys{
	type ReadData = &'a SingleCaseImpl<IdTree>;
	type WriteData = &'a mut MultiCaseImpl<Node, TransformWillChangeMatrix>;
	fn listen(&mut self, event: &DeleteEvent, idtree: Self::ReadData, transform_will_change_matrix: Self::WriteData){
		if let Some(r) = idtree.get(event.id) {
			self.will_change_mark.delete(event.id, r.layer());
			if transform_will_change_matrix.get(event.id).is_some() {
				transform_will_change_matrix.delete(event.id);
			}
			// self.mark_dirty(event.id, style_marks);
		}
	}
}

//  IdTree创建， 递归遍历所有子节点， 如果存在TransformWillChange组件， 在will_change_mark中添加一个标记
impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for TransformWillChangeSys{
	type ReadData = &'a SingleCaseImpl<IdTree>;
	type WriteData = (
		&'a mut MultiCaseImpl<Node, TransformWillChange>,
		&'a mut MultiCaseImpl<Node, StyleMark>,
	);
	fn listen(&mut self, event: &CreateEvent, idtree: Self::ReadData, write: Self::WriteData){
		if self.create_will_change_count == 0 {
			return;
		}
		let (willchanges, style_marks) = write;
		let node = &idtree[event.id];
		if let Some(_willchange) = willchanges.get_mut(event.id) {
			self.mark_dirty(event.id, style_marks);
			self.will_change_mark.mark(event.id, node.layer());
			self.create_will_change_count -= 1;
			if self.create_will_change_count == 0 {
				return;
			}
		}

		let first = node.children().head;
		for (child_id, child) in idtree.recursive_iter(first) {
			if let Some(_willchange) = willchanges.get_mut(child_id) {
				self.mark_dirty(event.id, style_marks);
				self.will_change_mark.mark(child_id, child.layer());
				self.create_will_change_count -= 1;
				if self.create_will_change_count == 0 {
					break;
				}
			}
		} 
	}
}

fn recursive_cal_matrix(
	id: usize,
	parent: usize,
	mut count: usize,
	parent_will_change_matrix: Option<&WorldMatrix>,
	default_transform: &Transform,
	idtree: &IdTree,
	willchange:  &MultiCaseImpl<Node, TransformWillChange>,
	layouts: &MultiCaseImpl<Node, LayoutR>,
	world_matrixs: &MultiCaseImpl<Node, WorldMatrix>,
	transforms: &MultiCaseImpl<Node, Transform>,
	style_marks: &mut MultiCaseImpl<Node, StyleMark>,
	transform_will_change_matrixs: &mut MultiCaseImpl<Node, TransformWillChangeMatrix>,
){  
	let mut parent_will_change_matrix = parent_will_change_matrix;
	style_marks[id].dirty_other &= !(StyleType1::TransformWillChange as usize);
	match willchange.get(id) {
		Some(transform_value) => {
			let layout = &layouts[id];
			let width = layout.rect.end - layout.rect.start;
			let height = layout.rect.bottom - layout.rect.top;
			let p_matrix = if parent == 0 {
				WorldMatrix(Matrix4::from_translation(Vector3::new(layout.rect.start, layout.rect.top, 0.0)), false)
			} else {
				let parent_layout = &layouts[parent];
				let parent_world_matrix = &world_matrixs[parent];
				let parent_transform = match transforms.get(parent) {
					Some(r) => r,
					None => default_transform,
				};
				let parent_transform_origin = parent_transform.origin.to_value(parent_layout.rect.end - parent_layout.rect.start, parent_layout.rect.bottom - parent_layout.rect.top);
				let offset = get_lefttop_offset(&layout, &parent_transform_origin, &parent_layout);
				parent_world_matrix * default_transform.matrix(width, height, &offset)
			};

			let transform_will_change_matrix = transform_value.0.matrix(width, height, &Point2::new(-width/2.0, -height/2.0));
			let invert = p_matrix.invert().unwrap();
			let mut will_change_matrix = p_matrix * transform_will_change_matrix * invert;
			if let Some(parent_will_change_matrix) = parent_will_change_matrix {
				will_change_matrix = parent_will_change_matrix * will_change_matrix;
			}

			transform_will_change_matrixs.insert(id, TransformWillChangeMatrix(will_change_matrix));
			parent_will_change_matrix = Some(unsafe { & (&*(transform_will_change_matrixs as *const MultiCaseImpl<Node, TransformWillChangeMatrix>))[id].0});
			count -= 1;
		},
		None => ()
	};

	let first = idtree[id].children().head;
	for (child_id, _child) in idtree.iter(first) {
		if count == 0 {
			break;
		}
		recursive_cal_matrix(
			child_id, 
			id, 
			count,
			parent_will_change_matrix, 
			default_transform,
			idtree, 
			willchange, 
			layouts, 
			world_matrixs, 
			transforms, 
			style_marks,
			transform_will_change_matrixs);
	}
}


#[inline]
fn get_lefttop_offset(layout: &LayoutR, parent_origin: &Point2, parent_layout: &LayoutR) -> Point2{
	Point2::new(
		// layout.left - parent_origin.x + parent_layout.border_left + parent_layout.padding_left,
		// layout.top - parent_origin.y + parent_layout.border_top + parent_layout.padding_top
		// 当设置宽高为auto时 可能存在bug
		parent_layout.border.start + parent_layout.padding.start + layout.rect.start - parent_origin.x,
		parent_layout.border.top + parent_layout.padding.top + layout.rect.top - parent_origin.y
	)  
}

impl_system!{
	TransformWillChangeSys,
	true,
	{
		MultiCaseListener<Node, TransformWillChange, CreateEvent>
		MultiCaseListener<Node, TransformWillChange, ModifyEvent>
		MultiCaseListener<Node, TransformWillChange, DeleteEvent>
		SingleCaseListener<IdTree, CreateEvent>
		// SingleCaseListener<IdTree, DeleteEvent>
	}
}