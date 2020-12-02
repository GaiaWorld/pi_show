//八叉树系统
use single::IdTree;
use ecs::{CreateEvent, DeleteEvent, EntityListener, MultiCaseImpl, Runner, SingleCaseImpl, MultiCaseListener, ModifyEvent};

use component::calc::{LayoutR, StyleMark, WorldMatrix};
use component::user::*;
use entity::Node;
use single::oct::Oct;
use single::*;
use ecs::monitor::NotifyImpl;
use Z_MAX;

#[derive(Default)]
pub struct OctSys;

impl<'a> Runner<'a> for OctSys {
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<Oct>;
    fn run(&mut self, _read: Self::ReadData, oct: Self::WriteData) {
        oct.collect();
    }
}

impl<'a> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for OctSys {
    type ReadData = (
		&'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, LayoutR>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a SingleCaseImpl<DefaultTable>,
        &'a SingleCaseImpl<IdTree>,
        &'a SingleCaseImpl<DirtyList>,
	);
    type WriteData = &'a mut SingleCaseImpl<Oct>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, oct: Self::WriteData) {
		let (world_matrixs, layouts, transforms, _style_marks, default_table, id_tree, _dirty_list) =
            read;
		let default_transform = default_table.get_unchecked::<Transform>();
        OctSys::modify_oct(
			event.id,
			id_tree,
			world_matrixs,
			layouts,
			transforms,
			default_transform,
			oct,
		);
    }
}

impl<'a> MultiCaseListener<'a, Node, WorldMatrix, CreateEvent> for OctSys {
    type ReadData = (
		&'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, LayoutR>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a SingleCaseImpl<DefaultTable>,
        &'a SingleCaseImpl<IdTree>,
        &'a SingleCaseImpl<DirtyList>,
	);
    type WriteData = &'a mut SingleCaseImpl<Oct>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, oct: Self::WriteData) {
        let (world_matrixs, layouts, transforms, _style_marks, default_table, id_tree, _dirty_list) =
            read;
		let default_transform = default_table.get_unchecked::<Transform>();
        OctSys::modify_oct(
			event.id,
			id_tree,
			world_matrixs,
			layouts,
			transforms,
			default_transform,
			oct,
		);
    }
}

impl OctSys {
    fn modify_oct(
        id: usize,
        idtree: &SingleCaseImpl<IdTree>,
        world_matrixs: &MultiCaseImpl<Node, WorldMatrix>,
        layouts: &MultiCaseImpl<Node, LayoutR>,
        transforms: &MultiCaseImpl<Node, Transform>,
        default_transform: &Transform,
        octree: &mut SingleCaseImpl<Oct>,
    ) {
        match idtree.get(id) {
            Some(r) => {
                if r.layer() == 0 {
                    return;
                }
            }
            None => return,
        };

        let transform = match transforms.get(id) {
            Some(r) => r,
            None => default_transform,
        };

        let world_matrix = &world_matrixs[id];
        let layout = &layouts[id];
        // let transform = get_or_default(id, transforms, default_table);

		let width = layout.rect.end - layout.rect.start;
		let height = layout.rect.bottom - layout.rect.top;
        let origin = transform.origin.to_value(width, height);
        let aabb = cal_bound_box((width, height), world_matrix, &origin);

		let notify = unsafe { &*(octree.get_notify_ref() as * const NotifyImpl) };
		if let Some(_r) = octree.get(id) {
			octree.update(id, aabb, Some(notify));
		} else {
			octree.add(id, aabb, id, Some(notify));
		}
    }
}

// impl<'a> EntityListener<'a, Node, CreateEvent> for OctSys {
//     type ReadData = ();
//     type WriteData = &'a mut SingleCaseImpl<Oct>;
//     fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
//         let notify = write.get_notify();
//         write.add(
//             event.id,
//             Aabb3::new(
//                 Point3::new(-1024f32, -1024f32, -Z_MAX),
//                 Point3::new(3072f32, 3072f32, Z_MAX),
//             ),
//             event.id,
//             Some(notify),
//         );
//     }
// }

impl<'a> EntityListener<'a, Node, DeleteEvent> for OctSys {
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<Oct>;
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, write: Self::WriteData) {
        let notify = unsafe { &* (write.get_notify_ref() as *const NotifyImpl)} ;
        write.remove(event.id, Some(notify));
    }
}

fn cal_bound_box(size: (f32, f32), matrix: &WorldMatrix, origin: &Point2) -> Aabb3 {
    let start = (-origin.x, -origin.y);
    let left_top = matrix * Vector4::new(start.0, start.1, 0.0, 1.0);
    let right_top = matrix * Vector4::new(start.0 + size.0, start.1, 0.0, 1.0);
    let left_bottom = matrix * Vector4::new(start.0, start.1 + size.1, 0.0, 1.0);
    let right_bottom = matrix * Vector4::new(start.0 + size.0, start.1 + size.1, 0.0, 1.0);

    let min = Point3::new(
        left_top
            .x
            .min(right_top.x)
            .min(left_bottom.x)
            .min(right_bottom.x),
        left_top
            .y
            .min(right_top.y)
            .min(left_bottom.y)
            .min(right_bottom.y),
        0.0,
    );

    let max = Point3::new(
        left_top
            .x
            .max(right_top.x)
            .max(left_bottom.x)
            .max(right_bottom.x),
        left_top
            .y
            .max(right_top.y)
            .max(left_bottom.y)
            .max(right_bottom.y),
        1.0,
    );

    Aabb3::new(min, max)
}

impl_system! {
    OctSys,
    true,
    {
        // EntityListener<Node, CreateEvent>
		EntityListener<Node, DeleteEvent>
		MultiCaseListener<Node, WorldMatrix, ModifyEvent>
		MultiCaseListener<Node, WorldMatrix, CreateEvent>
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

//     let system = CellOctSys::new(OctSys::default());
//     world.register_system(Atom::from("oct_system"), system);
//     let system = CellWorldMatrixSys::new(WorldMatrixSys::default());
//     world.register_system(Atom::from("world_matrix_system"), system);

//     let mut dispatch = SeqDispatcher::default();
//     dispatch.build("oct_system, world_matrix_system".to_string(), &world);

//     world.add_dispatcher(Atom::from("test_oct_sys"), dispatch);
//     world
// }
