use bevy_ecs::prelude::{Entity, In, Query, Res, ResMut};
use dirty::LayerDirty;
/**
 * 监听transform和layout组件， 利用transform和layout递归计算节点的世界矩阵（worldmatrix组件）
 */
use map::Map;

use crate::single::{IdTree, to_entity};
use crate::component::calc::{NodeState, LayoutR, WorldMatrix, StyleIndex};
use crate::component::user::Transform;
use crate::component::user::*;
use crate::util::event::{EntityEvent, ImMessenger};
use crate::util::util::get_or_default;
use crate::util::vecmap_default::VecMapWithDefault;

// 监听transform设脏
pub fn transform_listen(
	e: In<EntityEvent<Transform>>,
	mut local: ResMut<WorldMatrixSys>,
	idtree: Res<IdTree>,
) {
	local.marked_dirty(e.0.id.id() as usize, &idtree);
}

// 监听layout设脏
pub fn layout_listen(
	e: In<EntityEvent<LayoutR>>,
	mut local: ResMut<WorldMatrixSys>,
	idtree: Res<IdTree>,
) {
	local.marked_dirty(e.0.id.id() as usize, &idtree);
}

// 监听idtree设脏
pub fn idtree_listen(
	e: In<EntityEvent<IdTree>>,
	mut local: ResMut<WorldMatrixSys>,
	idtree: Res<IdTree>,
) {
	local.marked_dirty(e.0.id.id() as usize, &idtree);
}

/// 计算世界矩阵
pub fn calc_world_matrix(
	query: Query<(
		Option<&Transform>,
		&LayoutR,
		&NodeState)>,
	idtree: Res<IdTree>,
	default_transform: Res<Transform>,
	default_matrix: Res<WorldMatrix>,
	default_layout: Res<LayoutR>,
	mut world_matrix_query: Query<&mut WorldMatrix>,
	mut local: ResMut<WorldMatrixSys>,
	mut world_matrix_event_writer: ImMessenger<EntityEvent<WorldMatrix>>,
) {
	// 计算世界矩阵
	local.cal_matrix(&query, &mut world_matrix_query, &idtree, &default_transform, &default_matrix, &default_layout, &mut world_matrix_event_writer);
}

#[derive(Default)]
pub struct WorldMatrixSys {
    dirty_mark_list: VecMapWithDefault<usize>, // VecMap<layer>
    dirty: LayerDirty<usize>,
}

impl WorldMatrixSys {
	pub fn with_capacity(capacity: usize) -> WorldMatrixSys {
		WorldMatrixSys{
			dirty_mark_list: VecMapWithDefault::with_capacity(capacity), // VecMap<layer>
    		dirty: LayerDirty::default(),
		}
	}
    fn marked_dirty(&mut self, id: usize, id_tree: &IdTree) {
        match id_tree.get(id) {
            Some(r) => {
                if r.layer() != 0 {
					let d = &mut self.dirty_mark_list[id];
                    if *d != r.layer() {
                        if *d != 0 {
                            self.dirty.delete(id, *d);
                        }
                        *d = r.layer();
                        self.dirty.mark(id, r.layer());
                    }
                }
            }
            _ => (),
        };
    }

    fn cal_matrix(
        &mut self,
		query: &Query<(
			Option<&Transform>, 
			&LayoutR, 
			&NodeState)>,
		world_matrix_query: &mut Query<&mut WorldMatrix>,
		idtree: &IdTree,
		default_transform: &Transform,
		default_matrix: &WorldMatrix,
		default_layout: &LayoutR,
		world_matrix_event_writer: &mut ImMessenger<EntityEvent<WorldMatrix>>,
    ) {
		if self.dirty.count() == 0 {
			return;
		}
		let time = cross_performance::now();
        for (id, _layer) in self.dirty.iter() {
			let node = match idtree.get(*id) {
				Some(r) => r,
				None => continue,
			};
			let entity = to_entity(*id, node.data);

			if let Ok((_transform, _layout_r, node_state)) = query.get(entity) {
				// 非真实节点，不需要计算世界矩阵 （？？？）
				if !node_state.is_rnode() {
					continue;
				}

				let dirty_mark = match self.dirty_mark_list.get_mut(id) {
					Some(r) => r,
					None => continue, //panic!("dirty_mark_list err: {}", *id),
				};
				if *dirty_mark == 0 {
					continue;
				}
				*dirty_mark = 0;
				
				// 不在树上，不处理 
				let (transform, layout, matrix) = if node.layer() > 0 {
					if node.parent() < usize::max_value() {
						let p = to_entity(node.parent(), idtree[node.parent()].data);
						let (transform, layout_r, _node_state) = unsafe { query.get_unchecked(p).unwrap() };
						let transform = get_or_default(transform, default_transform);
						let matrix_ref = unsafe { &*(&*world_matrix_query.get_unchecked(p).unwrap() as *const WorldMatrix) };
						(transform, layout_r, matrix_ref)
					} else {
						( default_transform, default_layout, default_matrix)
					}
				} else {
					continue;
				};
				recursive_cal_matrix(
					&mut self.dirty_mark_list,
					entity,
					transform,
					layout,
					matrix,
					default_transform,
					query,
					world_matrix_query,
					idtree,
					world_matrix_event_writer,
				);
			}
        }
		// if self.dirty.count() > 0 {
		// 	log::info!("worldmatrix======={:?}", cross_performance::now() - time);
		// }
		self.dirty.clear();
    }
}

//取lefttop相对于父节点的变换原点的位置
#[inline]
fn get_lefttop_offset(layout: &LayoutR, parent_origin: &Point2, parent_layout: &LayoutR) -> Point2 {
    Point2::new(
        // 当设置宽高为auto时 可能存在bug
        parent_layout.border.start + parent_layout.padding.start + layout.rect.start - parent_origin.x,
        parent_layout.border.top + parent_layout.padding.top + layout.rect.top - parent_origin.y,
    )
}

fn recursive_cal_matrix(
    dirty_mark_list: &mut VecMapWithDefault<usize>,
    entity: Entity,
    parent_transform: &Transform,
	parent_layout: &LayoutR,
	parent_world_matrix: &WorldMatrix,
    default_transform: &Transform,
	query: &Query<(
		Option<&Transform>, 
		&LayoutR, 
		&NodeState)>,
	world_matrix_query: &Query<&mut WorldMatrix>,
	idtree: &IdTree,
	world_matrix_event_writer: &mut ImMessenger<EntityEvent<WorldMatrix>>
) {

	if let Ok((transform, layout, node_state)) = query.get(entity) {
		// 虚拟节点不存在WorlMatrix组件， 不需要计算
		if !node_state.is_rnode() {
			return;
		}
		dirty_mark_list[entity.id() as usize] = 0;

		let transform = get_or_default(transform, default_transform);

		let width = layout.rect.end - layout.rect.start;
		let height = layout.rect.bottom - layout.rect.top;
		// let matrix = match parent {
		// 	None => transform.matrix(
		// 		width,
		// 		height,
		// 		&Point2::new(layout.rect.start, layout.rect.top),
		// 	),
		// 	Some(parent) => {
		// 		let parent_layout = &layouts[parent];
		// 		let parent_world_matrix = &world_matrix[parent];
		// 		let parent_transform_origin = parent_transform
		// 			.origin
		// 			.to_value(parent_layout.rect.end - parent_layout.rect.start, parent_layout.rect.bottom - parent_layout.rect.top);
		// 		let offset = get_lefttop_offset(&layout, &parent_transform_origin, &parent_layout);
		// 		parent_world_matrix
		// 			* transform_value.matrix(width, height, &offset)
		// 	}
		// }
		let parent_transform_origin = parent_transform
					.origin
					.to_value(parent_layout.rect.end - parent_layout.rect.start, parent_layout.rect.bottom - parent_layout.rect.top);
		let offset = get_lefttop_offset(layout, &parent_transform_origin, &parent_layout);
		let matrix = parent_world_matrix
			* transform.matrix(width, height, &offset);

		// world_matrix.insert(id, matrix);
		// log::info!("");
		unsafe { *world_matrix_query.get_unchecked(entity).unwrap() = matrix; }
		world_matrix_event_writer.send(EntityEvent::new_modify(entity, StyleIndex::Matrix));
		
		let first = idtree[entity.id() as usize].children().head;
		let matrix_ref = unsafe { &*(&*world_matrix_query.get_unchecked(entity).unwrap() as *const WorldMatrix) };
		for child_id in idtree.iter(first) {
			recursive_cal_matrix(
				dirty_mark_list,
				to_entity(child_id.0, child_id.1.data),
				transform,
				layout,
				matrix_ref,
				default_transform,
				query,
				world_matrix_query,
				idtree,
				world_matrix_event_writer
			);
		}
	}
	
}

// impl_system! {
//     WorldMatrixSys,
//     true,
//     {
//         // EntityListener<Node, CreateEvent>
//         MultiCaseListener<Node, Transform, ModifyEvent>
//         MultiCaseListener<Node, Transform, CreateEvent>
//         MultiCaseListener<Node, Transform, DeleteEvent>
//         MultiCaseListener<Node, LayoutR, ModifyEvent>
// 		SingleCaseListener<IdTree, CreateEvent>
// 		// EntityListener<Node, DeleteEvent>
//     }
// }

// #[cfg(test)]
// use atom::Atom;
// #[cfg(test)]
// use component::calc::ZDepth;
// #[cfg(test)]
// use component::user::{TransformFunc, TransformWrite};
// #[cfg(test)]
// use ecs::{Dispatcher, LendMut, SeqDispatcher, World};

// #[test]
// fn test() {
//     let world = new_world();

//     let idtree = world.fetch_single::<IdTree>().unwrap();
//     let idtree = LendMut::lend_mut(&idtree);
//     let notify = idtree.get_notify();
//     let transforms = world.fetch_multi::<Node, Transform>().unwrap();
//     let transforms = LendMut::lend_mut(&transforms);
//     let layouts = world.fetch_multi::<Node, LayoutR>().unwrap();
//     let layouts = LendMut::lend_mut(&layouts);
//     let world_matrixs = world.fetch_multi::<Node, WorldMatrix>().unwrap();
//     let world_matrixs = LendMut::lend_mut(&world_matrixs);
//     let zdepths = world.fetch_multi::<Node, ZDepth>().unwrap();
//     let zdepths = LendMut::lend_mut(&zdepths);

//     let e0 = world.create_entity::<Node>();

//     idtree.create(e0);
//     idtree.insert_child(e0, 0, 0); //根
//     transforms.insert(e0, Transform::default());
//     zdepths.insert(e0, ZDepth::default());
//     layouts.insert(
//         e0,
//         LayoutR {
//             left: 0.0,
//             top: 0.0,
//             width: 900.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );

//     world.run(&Atom::from("test_transform_sys"));

//     let e00 = world.create_entity::<Node>();
//     let e01 = world.create_entity::<Node>();
//     let e02 = world.create_entity::<Node>();
//     idtree.create(e00);
//     idtree.insert_child(e00, e0, 1);
//     transforms.insert(e00, Transform::default());
//     zdepths.insert(e00, ZDepth::default());
//     layouts.insert(
//         e00,
//         LayoutR {
//             left: 0.0,
//             top: 0.0,
//             width: 300.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     idtree.create(e01);
//     idtree.insert_child(e01, e0, 2);
//     layouts.insert(
//         e01,
//         LayoutR {
//             left: 300.0,
//             top: 0.0,
//             width: 300.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     transforms.insert(e01, Transform::default());
//     zdepths.insert(e01, ZDepth::default());
//     idtree.create(e02);
//     idtree.insert_child(e02, e0, 3);
//     transforms.insert(e02, Transform::default());
//     zdepths.insert(e02, ZDepth::default());
//     layouts.insert(
//         e02,
//         LayoutR {
//             left: 600.0,
//             top: 0.0,
//             width: 300.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );

//     let e000 = world.create_entity::<Node>();
//     let e001 = world.create_entity::<Node>();
//     let e002 = world.create_entity::<Node>();
//     idtree.create(e000);
//     idtree.insert_child(e000, e00, 1);
//     layouts.insert(
//         e000,
//         LayoutR {
//             left: 0.0,
//             top: 0.0,
//             width: 100.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     transforms.insert(e000, Transform::default());
//     zdepths.insert(e000, ZDepth::default());
//     idtree.create(e001);
//     idtree.insert_child(e001, e00, 2);
//     transforms.insert(e001, Transform::default());
//     zdepths.insert(e001, ZDepth::default());
//     layouts.insert(
//         e001,
//         LayoutR {
//             left: 100.0,
//             top: 0.0,
//             width: 100.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     idtree.create(e002);
//     idtree.insert_child(e002, e00, 3);
//     transforms.insert(e002, Transform::default());
//     zdepths.insert(e002, ZDepth::default());
//     layouts.insert(
//         e002,
//         LayoutR {
//             left: 200.0,
//             top: 0.0,
//             width: 100.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );

//     let e010 = world.create_entity::<Node>();
//     let e011 = world.create_entity::<Node>();
//     let e012 = world.create_entity::<Node>();
//     idtree.create(e010);
//     idtree.insert_child(e010, e01, 1);
//     layouts.insert(
//         e010,
//         LayoutR {
//             left: 0.0,
//             top: 0.0,
//             width: 100.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     transforms.insert(e010, Transform::default());
//     zdepths.insert(e010, ZDepth::default());
//     idtree.create(e011);
//     idtree.insert_child(e011, e01, 2);
//     transforms.insert(e011, Transform::default());
//     zdepths.insert(e011, ZDepth::default());
//     layouts.insert(
//         e011,
//         LayoutR {
//             left: 100.0,
//             top: 0.0,
//             width: 100.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     idtree.create(e012);
//     idtree.insert_child(e012, e01, 3);
//     transforms.insert(e012, Transform::default());
//     zdepths.insert(e012, ZDepth::default());
//     layouts.insert(
//         e012,
//         LayoutR {
//             left: 200.0,
//             top: 0.0,
//             width: 100.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     world.run(&Atom::from("test_world_matrix_sys"));

//     transforms.get_write(e0).unwrap().modify(|transform: &mut Transform| {
//         transform.funcs.push(TransformFunc::TranslateX(50.0));
//         true
//     });

//     world.run(&Atom::from("test_transform_sys"));

//     debug_println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
// 		&world_matrixs[e0],
// 		&world_matrixs[e00],
// 		&world_matrixs[e01],
// 		&world_matrixs[e02],
// 		&world_matrixs[e000],
// 		&world_matrixs[e001],
// 		&world_matrixs[e002],
// 		&world_matrixs[e010],
// 		&world_matrixs[e011],
// 		&world_matrixs[e012],
// 	);
// }

// #[cfg(test)]
// fn new_world() -> World {
//     let mut world = World::default();

//     world.register_entity::<Node>();
//     world.register_multi::<Node, Transform>();
//     world.register_multi::<Node, LayoutR>();
//     world.register_multi::<Node, WorldMatrix>();
//     world.register_multi::<Node, ZDepth>();
//     world.register_single::<IdTree>(IdTree::default());

//     let system = CellWorldMatrixSys::new(WorldMatrixSys::default());
//     world.register_system(Atom::from("system"), system);

//     let mut dispatch = SeqDispatcher::default();
//     dispatch.build("system".to_string(), &world);

//     world.add_dispatcher(Atom::from("test_world_matrix_sys"), dispatch);
//     world
// }
