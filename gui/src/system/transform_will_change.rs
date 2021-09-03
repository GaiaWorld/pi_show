// use bevy_ecs::prelude::{Changed, Commands, Entity, EventReader, Local, Query, Res, With, Without};
// use bevy_ecs::system::Command;
// /**
//  * 监听TransformWillChange的改变， 修改TransformWillChangeMatrix
//  */
// use dirty::LayerDirty;
// use hash::XHashMap;
// use map::hashmap::HashMap;

// use crate::component::user::{ Transform };
// use crate::component::calc::{ WorldMatrix, StyleMark, StyleType1, TransformWillChangeMatrix, LayoutR };

// use crate::component::user::*;
// use crate::entity::{Node};
// use crate::single::{IdTree, to_entity};
// use crate::util::event::{EntityEvent, EventType, IdTreeEvent};
// use crate::util::util::get_or_default;


// pub fn handle_transfrom_willchange(
// 	commands: Commands,
// 	query: Query<(
// 		&LayoutR,
// 		Option<&TransformWillChange>,
// 		&WorldMatrix,
// 		Option<&Transform>)>,
// 	idtree: Res<IdTree>,
// 	mut mut_query: Query<(&mut StyleMark, Option<&mut TransformWillChangeMatrix>)>,
// 	mut style_marks: Query<&mut StyleMark>,
// 	mut willchanges: Query<&TransformWillChange, With<TransformWillChange>>,
// 	mut transfrom_willchange_event_reader: EventReader<EntityEvent<TransformWillChange>>,
// 	mut idtree_event_reader: EventReader<IdTreeEvent>,
// 	local: Local<TransformWillChangeSys>,
// ) {
// 	for e in transfrom_willchange_event_reader.iter() {
// 		if let Some(r) = idtree.get(e.id.id() as usize) {
// 			if r.data != e.id.generation() { // 实体已经销毁，不处理
// 				continue;
// 			}
// 			let id = e.id.id() as usize;

// 			match e.ty {
// 				EventType::Create => {
// 					if r.layer() > 0 {
// 						local.mark_dirty(e.id, &mut style_marks);
// 						local.will_change_mark.mark(id, r.layer());
// 					} else {
// 						local.will_change_untree.push(e.id);
// 					}
// 				},
// 				EventType::Modify => {
// 					if r.layer() > 0 {
// 						local.mark_dirty(e.id, &mut style_marks);
// 					}
// 				},
// 				EventType::Delete => {
// 					local.will_change_mark.delete(id, r.layer());
// 					commands.entity(e.id).remove::<TransformWillChangeMatrix>();
// 					// 是否通知？？TODO
// 				},
// 			}
// 		}
// 	} 

// 	// 尝试将will_change_untree中的实体移入脏列表中
// 	if local.will_change_untree.len() > 0 {
// 		local.will_change_untree.retain(|entity| {
// 			let id = entity.id() as usize;
// 			let node = match idtree.get(id) {
// 				Some(r) => if r.data == entity.generation() {
// 					if r.layer() > 0 { // 已经在树上，从vec中移除，放入脏列表中
// 						if let Ok(willchange) = willchanges.get(*entity) {
// 							local.mark_dirty(*entity, &mut style_marks);
// 							local.will_change_mark.mark(id, r.layer());
// 						}
// 						return false;
// 					}
// 				} else {// 实体已经不存在，从vec中删除
// 					return false;
// 				},
// 				None => return false, // 实体已经不存在，从vec中删除
// 			};
// 			true // 实体存在，但仍然不在树上，保留在数组中
// 		})
// 	}
// }

// #[derive(Default)]
// pub struct TransformWillChangeSys{
// 	will_change_mark: LayerDirty<usize>, // 存放存在TransformWillChang的组件id， 并不是为了记脏
// 	dirty: bool,
// 	// TransformWillChang创建时， 如果该节点不在根树上， 该值会+1， 当节点树被添加到根上时， 遍历子孙节点，如果节点上存在TransformWillChang， 该值-1， 直到减为0， 会停止遍历
// 	// 因为存在TransformWillChang的节点数量应该是少量的， 记录该值， 可以减少每次节点被添加到根时的遍历
// 	create_will_change_count: usize,
// 	// 设置了willchange， 但不在树上的节点，暂时缓存在这里
// 	// 如果有节点添加到树上时，需要遍历该容器，检查容器中的实体是否已经添加到树上了。
// 	// 该容器中的实体个数很少，甚至于大部分情况为0个。 因此，几乎不需要担心遍历的性能问题
// 	// 而且，目前事件为延时处理，添加willchange的节点，通常都在树上。该字段的存在只是用于处理边缘情况
// 	will_change_untree: Vec<Entity>,
// }

// impl TransformWillChangeSys{

// 	#[inline]
// 	fn mark_dirty(&mut self, entity: Entity, style_marks: &mut Query<&mut StyleMark>) {
// 		self.dirty = true;
// 		let mark = style_marks.get_mut(entity).unwrap() ;
// 		mark.dirty_other |= StyleType1::TransformWillChange as usize;
// 	}

// 	fn run(
// 		&mut self,
// 		commands: &Commands,
// 		query: &Query<(
// 			&LayoutR,
// 			Option<&TransformWillChange>,
// 			&WorldMatrix,
// 			Option<&Transform>)>,
// 		idtree: &Res<IdTree>,
// 		mut willchanges_matrix: &mut Query<Option<&mut TransformWillChangeMatrix>>,
// 		mut style_marks: &mut Query<&mut StyleMark>,
// 		mut willchanges: &mut Query<&TransformWillChange, With<TransformWillChange>>,
// 		mut transfrom_willchange_event_reader: &mut EventReader<EntityEvent<TransformWillChange>>,
// 		mut idtree_event_reader: &mut EventReader<IdTreeEvent>,
// 		local: &mut Local<TransformWillChangeSys>,){

// 		if !self.dirty {
// 			return;
// 		}
// 		// let time = std::time::Instant::now();
// 		self.dirty = false;
// 		// let (idtree, layouts, transform_will_changes, world_matrixs, transforms) = ;
// 		// let (style_marks, transform_will_change_matrixs) = write;
// 		let mut count = self.will_change_mark.count();
// 		let mut deletes = Vec::default();
// 		for (id, layer) in self.will_change_mark.iter() {
// 			if count == 0 {
// 				break;
// 			}
// 			count -= 1;

// 			let node = match idtree.get(*id) {
// 				Some(r) => r,
// 				None => continue
// 			};
// 			let entity = to_entity(*id, node.data);
// 			// 不存在willchange， 不处理（因为没有关心节点销毁的事件，节点销毁，但是willchange组件删除通知没有， 就可能出现这种情况）
// 			if let Err(_) = willchanges.get(entity) {
// 				continue;
// 			}

// 			// 也可能不在树上，则将其移入will_change_untree中
// 			if node.layer() == 0 {
// 				self.will_change_untree.push(entity);
// 			}

// 			let style_mark = style_marks.get_mut(entity).unwrap();

// 			// TransformWillChange不脏
// 			// if style_mark.dirty_other | StyleType1::TransformWillChange as usize == 0 {
// 			// 	continue;
// 			// }

// 			recursive_cal_matrix(
// 				*id,
// 				node.parent(), 
// 				count + 1, 
// 				None, 
// 				idtree,
// 				willchanges, 
// 				layouts, 
// 				world_matrixs, 
// 				transforms, 
// 				style_marks,
// 				transform_will_change_matrixs);
// 		}

// 		for (id, layer) in deletes.iter() {
// 			self.will_change_mark.delete(*id, *layer);
// 		}
// 		// println!("TransformWillChangeSys run : {:?}", std::time::Instant::now() - time);
// 	}
// }

// // impl<'a> Runner<'a> for TransformWillChangeSys{
// // 	type ReadData = (
// // 		&'a SingleCaseImpl<IdTree>,
// // 		&'a MultiCaseImpl<Node, LayoutR>,
// // 		&'a MultiCaseImpl<Node, TransformWillChange>,
// // 		&'a MultiCaseImpl<Node, WorldMatrix>,
// // 		&'a MultiCaseImpl<Node, Transform>,
// // 	);
// // 	type WriteData = (
// // 		&'a mut MultiCaseImpl<Node, StyleMark>,
// // 		&'a mut MultiCaseImpl<Node, TransformWillChangeMatrix>,
// // 	);
	
// // }



// fn recursive_cal_matrix(
// 	parent: Entity,
// 	entity: Entity,
// 	commands: &Commands,
// 	query: &Query<(
// 		&LayoutR,
// 		&WorldMatrix,
// 		Option<&Transform>)>,
// 	idtree: &IdTree,
// 	default_transform: &Transform,
// 	mut parent_will_change_matrix: Option<&WorldMatrix>,
// 	mut willchanges_matrix: &mut Query<Option<&mut TransformWillChangeMatrix>>,
// 	mut style_marks: &mut Query<&mut StyleMark>,
// 	mut willchanges: &mut Query<&TransformWillChange, With<TransformWillChange>>,
// 	mut transfrom_willchange_event_reader: &mut EventReader<EntityEvent<TransformWillChange>>,
// 	mut idtree_event_reader: &mut EventReader<IdTreeEvent>,
// 	local: &mut Local<TransformWillChangeSys>,
// ){  
// 	style_marks.get_mut(entity).unwrap().dirty_other &= !(StyleType1::TransformWillChange as usize);
// 	match willchanges.get(entity) {
// 		Ok(transform_value) => {
// 			let (layout, world_matrix, transform) = query.get(entity).unwrap();
// 			let width = layout.rect.end - layout.rect.start;
// 			let height = layout.rect.bottom - layout.rect.top;
// 			// let p_matrix = if parent == 0 {
// 			// 	WorldMatrix(Matrix4::new_translation(&Vector3::new(layout.rect.start, layout.rect.top, 0.0)), false)
// 			// } else {
// 				let (parent_layout, parent_world_matrix, parent_transform) = query.get(parent).unwrap();
// 				let parent_transform = get_or_default(parent_transform, default_transform);
// 				let parent_transform_origin = parent_transform.origin.to_value(parent_layout.rect.end - parent_layout.rect.start, parent_layout.rect.bottom - parent_layout.rect.top);
// 				let offset = get_lefttop_offset(&layout, &parent_transform_origin, &parent_layout);
// 				let p_matrix = parent_world_matrix * default_transform.matrix(width, height, &offset);
// 			// };

// 			let transform_will_change_matrix = transform_value.0.matrix(width, height, &Point2::new(-width/2.0, -height/2.0));
// 			let invert = p_matrix.invert().unwrap();
// 			let mut will_change_matrix = p_matrix * transform_will_change_matrix * invert;
// 			if let Some(parent_will_change_matrix) = parent_will_change_matrix {
// 				will_change_matrix = parent_will_change_matrix * will_change_matrix;
// 			}

// 			commands.entity(entity).insert(TransformWillChangeMatrix(will_change_matrix));
// 			// transform_will_change_matrixs.insert(id, );

// 			// TODO
// 			// parent_will_change_matrix = Some(unsafe { & (&*(transform_will_change_matrixs as *const MultiCaseImpl<Node, TransformWillChangeMatrix>))[id].0});
// 		},
// 		Err(_) => ()
// 	};

// 	let first = idtree[entity.id() as usize].children().head;
// 	for (child_id, _child) in idtree.iter(first) {
// 		if count == 0 {
// 			break;
// 		}
// 		recursive_cal_matrix(
// 			child_id, 
// 			id, 
// 			count,
// 			parent_will_change_matrix,
// 			idtree, 
// 			willchange, 
// 			layouts, 
// 			world_matrixs, 
// 			transforms, 
// 			style_marks,
// 			transform_will_change_matrixs);
// 	}
// }


// #[inline]
// fn get_lefttop_offset(layout: &LayoutR, parent_origin: &Point2, parent_layout: &LayoutR) -> Point2{
// 	Point2::new(
// 		// layout.left - parent_origin.x + parent_layout.border_left + parent_layout.padding_left,
// 		// layout.top - parent_origin.y + parent_layout.border_top + parent_layout.padding_top
// 		// 当设置宽高为auto时 可能存在bug
// 		parent_layout.border.start + parent_layout.padding.start + layout.rect.start - parent_origin.x,
// 		parent_layout.border.top + parent_layout.padding.top + layout.rect.top - parent_origin.y
// 	)  
// }

// impl_system!{
// 	TransformWillChangeSys,
// 	true,
// 	{
// 		MultiCaseListener<Node, TransformWillChange, CreateEvent>
// 		MultiCaseListener<Node, TransformWillChange, ModifyEvent>
// 		MultiCaseListener<Node, TransformWillChange, DeleteEvent>
// 		SingleCaseListener<IdTree, CreateEvent>
// 		// SingleCaseListener<IdTree, DeleteEvent>
// 	}
// }