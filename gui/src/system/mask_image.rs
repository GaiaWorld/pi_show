// /**
//  * 监听TransformWillChange的改变， 修改TransformWillChangeMatrix
// 	*/

// use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner, Write};
// use single::IdTree;
// use dirty::LayerDirty;

// use component::user::{ Transform };
// use component::calc::{ WorldMatrix, StyleMark, StyleType1, TransformWillChangeMatrix, LayoutR };

// use component::user::*;
// use component::calc::{RenderContext, DirtyViewRect};
// use entity::{Node};

// #[derive(Default)]
// pub struct MaskImageSys{
// }

// impl MaskImageSys{

// }

// impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for MaskImageSys{
// 	type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, MaskImage>);
// 	type WriteData = &'a mut MultiCaseImpl<Node, RenderContext>;
// 	fn listen(&mut self, event: &CreateEvent, (idtree, mask_images): Self::ReadData, render_contexts: Self::WriteData){
// 		try_create_render_context_with_parent(event.id, idtree, mask_images, render_contexts);
// 	}
// }

// impl<'a> SingleCaseListener<'a, IdTree, ModifyEvent> for MaskImageSys{
// 	type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, MaskImage>);
// 	type WriteData = &'a mut MultiCaseImpl<Node, RenderContext>;
// 	fn listen(&mut self, event: &ModifyEvent, (idtree, mask_images): Self::ReadData, render_contexts: Self::WriteData){
// 		if event.field == "add" {
// 			try_create_render_context_with_parent(event.id, idtree, mask_images, render_contexts);
// 		}
// 	}
// }

// impl<'a> MultiCaseListener<'a, Node, MaskImage, CreateEvent> for MaskImageSys{
// 	type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, MaskImage>);
// 	type WriteData = &'a mut MultiCaseImpl<Node, RenderContext>;
// 	fn listen(&mut self, event: &CreateEvent, idtree: Self::ReadData, style_marks: Self::WriteData){
		
// 	}
// }

// impl<'a> MultiCaseListener<'a, Node, MaskImage, ModifyEvent> for MaskImageSys{
// 	type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, MaskImage>);
// 	type WriteData = &'a mut MultiCaseImpl<Node, RenderContext>;
// 	fn listen(&mut self, event: &ModifyEvent, idtree: Self::ReadData, style_marks: Self::WriteData){
// 		if event.field == "add" {
			
// 		}
// 	}
// }

// impl<'a> MultiCaseListener<'a, Node, MaskImage, DeleteEvent> for MaskImageSys{
// 	type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, MaskImage>);
// 	type WriteData = &'a mut MultiCaseImpl<Node, RenderContext>;
// 	fn listen(&mut self, event: &DeleteEvent, idtree: Self::ReadData, style_marks: Self::WriteData){
// 		if event.field == "add" {
			
// 		}
// 	}
// }

// fn try_create_render_context_with_parent(id: usize, idtree: &SingleCaseImpl<IdTree>, mask_images: &MultiCaseImpl<Node, MaskImage>,  render_contexts: &mut MultiCaseImpl<Node, RenderContext>) {
// 	let node = idtree[id];
// 	if node.count() > 1 {
// 		try_create_render_context(id, mask_images, render_contexts);
// 	}
// 	if node.parent() > 0 {
// 		try_create_render_context(node.parent(), mask_images, render_contexts);
// 	}
// }

// fn try_create_render_context(
// 	id: usize, 
// 	mask_images: &MultiCaseImpl<Node, MaskImage>,
// 	render_contexts: &mut MultiCaseImpl<Node, RenderContext>
// ) {
// 	if let Some(img) = mask_images.get(id) {
// 		if img.src.is_some() && render_contexts.get(id).is_none() {
// 			render_contexts.insert(id,RenderContext {
// 				size: (0, 0), // 大小、尺寸
// 				content_box: Aabb3::new(Point3::new(0.0, 0.0,0.0), Point3::new(0.0, 0.0,0.0)), // 内容的最大包围盒
// 				view_matrix: WorldMatrix::default(),
// 				projection_matrix: WorldMatrix::default(),
// 				render_objs: Vec::new(), // 节点本身的渲染对象
// 				dirty_view_rect: DirtyViewRect::default(),
// 				render_target: Option::None,
// 			});
// 		}
// 	}
// }





// // impl_system!{
// // 	MaskImageSys,
// // 	true,
// // 	{
// // 		MultiCaseListener<Node, TransformWillChange, CreateEvent>
// // 		MultiCaseListener<Node, TransformWillChange, ModifyEvent>
// // 		MultiCaseListener<Node, TransformWillChange, DeleteEvent>
// // 		SingleCaseListener<IdTree, CreateEvent>
// // 		// SingleCaseListener<IdTree, DeleteEvent>
// // 	}
// // }