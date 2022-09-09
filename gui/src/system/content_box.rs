//八叉树系统
use ecs::{CreateEvent, Event, ModifyEvent, MultiCaseImpl, Runner, SingleCaseImpl, SingleCaseListener};


use crate::component::calc::{ ContentBox };
use crate::component::user::*;
use crate::component::calc::{NodeState, TransformWillChangeMatrix};
use crate::entity::Node;
use crate::single::oct::Oct;
use crate::single::IdTree;
use crate::util::{Dirty, DirtyMark};

#[derive(Default, Deref, DerefMut)]
pub struct ContentBoxSys(Dirty);

/// 脏类型
enum DirtyType {
	Oneself, // 自身脏
	Children, // 子节脏
}

impl<'a> Runner<'a> for ContentBoxSys {
    type ReadData = (
		&'a SingleCaseImpl<Oct>,
		&'a MultiCaseImpl<Node, NodeState>,
		&'a MultiCaseImpl<Node, TransformWillChangeMatrix>,
        &'a SingleCaseImpl<IdTree>,
	);
    type WriteData = &'a mut MultiCaseImpl<Node, ContentBox>;
    fn run(&mut self, (oct, node_states, will_change_matrixs, idtree): Self::ReadData, content_boxs: Self::WriteData) {
		if self.dirty.count() == 0 {
			return;
		}

		let dirty1 = unsafe {&mut *(&self.0 as *const Dirty as usize as *mut Dirty)};
		let d = &mut self.0;
		let dirty = &mut d.dirty;
		let dirty_mark_list = &mut d.dirty_mark_list;
		// 从叶子节点往根节点遍历
		for (id, layer) in dirty.iter_reverse() {
			// let dirty_type = dirty_mark_list[*id].clone();
			dirty_mark_list[*id] = DirtyMark::default(); // 清除标记

			let node= match idtree.get(*id){
				Some(r) => r,
				None => {
					continue},
			};
			
			if node.layer() != layer {
				continue;
			}
			let mut chilren_change = false;
			
			let mut content_box = match oct.get(*id) {
				Some(r) => r.0.clone(),
				None => {
					continue},
			}; // 
			if node.children().len > 0 {
				if node_states[node.children().head].is_rnode() {
					for (child, _child) in idtree.iter(node.children().head) {
						box_and(&mut content_box, &content_boxs[child].0);
					}
				}
			}

			// TODO
			if let Some(_will_change) = will_change_matrixs.get(*id) {

			}

			content_box.mins.x = content_box.mins.x.floor();
			content_box.mins.y = content_box.mins.y.floor();
			content_box.maxs.x = content_box.maxs.x.ceil();
			content_box.maxs.y = content_box.maxs.y.ceil();

			// log::info!("id: {}, content_box: {:?}", id, content_box);
			
			if let Some(old) = content_boxs.get(*id) {
				if old.0 != content_box {
					chilren_change = true;
				}
			} else {
				chilren_change = true;
			}
			
			// 如果内容包围盒发生改变，则重新插入内容包围盒，并标记父脏
			if chilren_change {
				content_boxs.insert(*id, ContentBox(content_box));
				if node.parent() > 0 {
					dirty1.marked_dirty(node.parent(), idtree, DirtyType::Children as usize)
				}
				
			}
		}
		self.dirty.clear();
	}
}

//监听Oct组件的修改
impl<'a> SingleCaseListener<'a, Oct, (CreateEvent, ModifyEvent)> for ContentBoxSys {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();

    fn listen(&mut self, event: &Event, idtree: Self::ReadData, _: Self::WriteData) {
		self.marked_dirty(event.id, idtree, DirtyType::Oneself as usize);
    }
}

// 两个aabb的并
fn box_and(aabb1: &mut Aabb2, aabb2: &Aabb2) {
	aabb1.mins.x = aabb1.mins.x.min(aabb2.mins.x);
	aabb1.mins.y = aabb1.mins.y.min(aabb2.mins.y);
	aabb1.maxs.x = aabb1.maxs.x.max(aabb2.maxs.x);
	aabb1.maxs.y = aabb1.maxs.y.max(aabb2.maxs.y);
}

impl_system! {
    ContentBoxSys,
    true,
    {
		SingleCaseListener<Oct, (CreateEvent, ModifyEvent)>
    }
}

// #[cfg(test)]
// use atom::Atom;
// #[cfg(test)]
// use component::calc::{ZDepth};
// #[cfg(test)]
// use component::user::{TransformFunc, TransformWrite};
// #[cfg(test)]
// use ecs::{Dispatcher, LendMut, SeqDispatcher, World};
// #[cfg(test)]
// use system::world_matrix::{CellWorldMatrixSys, WorldMatrixSys};
// #[cfg(test)]
// use flex_layout::Rect;

// #[test]
// fn test() {
//     let world = new_world();

//     let idtree = world.fetch_single::<IdTree>().unwrap();
//     let idtree = LendMut::lend_mut(&idtree);
//     let oct = world.fetch_single::<Oct>().unwrap();
//     let oct = LendMut::lend_mut(&oct);
//     let notify = idtree.get_notify();
//     let transforms = world.fetch_multi::<Node, Transform>().unwrap();
//     let transforms = LendMut::lend_mut(&transforms);
//     let layouts = world.fetch_multi::<Node, LayoutR>().unwrap();
//     let layouts = LendMut::lend_mut(&layouts);
//     let world_matrixs = world.fetch_multi::<Node, WorldMatrix>().unwrap();
//     let _world_matrixs = LendMut::lend_mut(&world_matrixs);
//     let zdepths = world.fetch_multi::<Node, ZDepth>().unwrap();
//     let zdepths = LendMut::lend_mut(&zdepths);

//     let e0 = world.create_entity::<Node>();

//     idtree.create(e0);
//     transforms.insert(e0, Transform::default());
//     zdepths.insert(e0, ZDepth::default());
//     layouts.insert(
//         e0,
//         LayoutR {
// 			rect: Rect{start: 0.0, end: 900.0, top: 0.0, bottom: 900.0},
// 			border: Rect{start: 0.0, end: 900.0, top: 0.0, bottom: 900.0},
// 			padding: Rect{start: 0.0, end: 900.0, top: 0.0, bottom: 900.0},
//         },
//     );
//     idtree.insert_child(e0, 0, 0); //根

//     let e00 = world.create_entity::<Node>();
//     let e01 = world.create_entity::<Node>();
//     let e02 = world.create_entity::<Node>();
//     idtree.create(e00);
//     transforms.insert(e00, Transform::default());
//     zdepths.insert(e00, ZDepth::default());
//     layouts.insert(
//         e00,
//         LayoutR {
// 			rect: Rect{start: 0.0, end: 300.0, top: 0.0, bottom: 900.0},
// 			border: Rect{start: 0.0, end: 300.0, top: 0.0, bottom: 900.0},
// 			padding: Rect{start: 0.0, end: 300.0, top: 0.0, bottom: 900.0},
//         },
//     );
//     idtree.insert_child(e00, e0, 1);

//     idtree.create(e01);
//     layouts.insert(
//         e01,
//         Layout {
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
//     idtree.insert_child(e01, e0, 2);

//     idtree.create(e02);
//     transforms.insert(e02, Transform::default());
//     zdepths.insert(e02, ZDepth::default());
//     layouts.insert(
//         e02,
//         Layout {
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
//     idtree.insert_child(e02, e0, 3);

//     let e000 = world.create_entity::<Node>();
//     let e001 = world.create_entity::<Node>();
//     let e002 = world.create_entity::<Node>();
//     idtree.create(e000);
//     layouts.insert(
//         e000,
//         Layout {
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
//     idtree.insert_child(e000, e00, 1);

//     idtree.create(e001);
//     transforms.insert(e001, Transform::default());
//     zdepths.insert(e001, ZDepth::default());
//     layouts.insert(
//         e001,
//         Layout {
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
//     idtree.insert_child(e001, e00, 2);

//     idtree.create(e002);
//     transforms.insert(e002, Transform::default());
//     zdepths.insert(e002, ZDepth::default());
//     layouts.insert(
//         e002,
//         Layout {
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
//     idtree.insert_child(e002, e00, 3);

//     let e010 = world.create_entity::<Node>();
//     let e011 = world.create_entity::<Node>();
//     let e012 = world.create_entity::<Node>();
//     idtree.create(e010);
//     layouts.insert(
//         e010,
//         Layout {
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
//     idtree.insert_child(e010, e01, 1);

//     idtree.create(e011);
//     transforms.insert(e011, Transform::default());
//     zdepths.insert(e011, ZDepth::default());
//     layouts.insert(
//         e011,
//         Layout {
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
//     idtree.insert_child(e011, e01, 2);

//     idtree.create(e012);
//     transforms.insert(e012, Transform::default());
//     zdepths.insert(e012, ZDepth::default());
//     layouts.insert(
//         e012,
//         Layout {
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
//     idtree.insert_child(e012, e01, 3);

//     transforms.get_write(e0).unwrap().modify(|transform: &mut Transform| {
//         transform.funcs.push(TransformFunc::TranslateX(50.0));
//         true
//     });
//     world.run(&Atom::from("test_oct_sys"));
//     debug_println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
//         oct.get(e0).unwrap(),
//         oct.get(e00).unwrap(),
//         oct.get(e01).unwrap(),
//         oct.get(e02).unwrap(),
//         oct.get(e000).unwrap(),
//         oct.get(e001).unwrap(),
//         oct.get(e002).unwrap(),
//         oct.get(e010).unwrap(),
//         oct.get(e011).unwrap(),
//         oct.get(e012).unwrap(),
//     );
// }

// #[cfg(test)]
// fn new_world() -> World {
//     let mut world = World::default();

//     world.register_entity::<Node>();
//     world.register_multi::<Node, Layout>();
//     world.register_multi::<Node, Transform>();
//     world.register_multi::<Node, ZDepth>();
//     world.register_multi::<Node, WorldMatrix>();
//     world.register_single::<IdTree>(IdTree::default());
//     world.register_single::<Oct>(Oct::new());

//     let system = CellContentBoxSys::new(ContentBoxSys::default());
//     world.register_system(Atom::from("oct_system"), system);
//     let system = CellWorldMatrixSys::new(WorldMatrixSys::default());
//     world.register_system(Atom::from("world_matrix_system"), system);

//     let mut dispatch = SeqDispatcher::default();
//     dispatch.build("oct_system, world_matrix_system".to_string(), &world);

//     world.add_dispatcher(Atom::from("test_oct_sys"), dispatch);
//     world
// }
