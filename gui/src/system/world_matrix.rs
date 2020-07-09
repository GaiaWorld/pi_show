use dirty::LayerDirty;
use ecs::idtree::IdTree;
/**
 * 监听transform和layout组件， 利用transform和layout递归计算节点的世界矩阵（worldmatrix组件）
	*/
use ecs::{
    CreateEvent, DeleteEvent, EntityListener, ModifyEvent, MultiCaseImpl, MultiCaseListener,
    Runner, SingleCaseImpl, SingleCaseListener,
};

use component::calc::{Layout, WorldMatrix, WorldMatrixWrite};
use component::user::Transform;
use map::vecmap::VecMap;
use single::DefaultTable;

use component::user::*;
use entity::Node;

#[derive(Default)]
pub struct WorldMatrixSys {
    dirty_mark_list: VecMap<usize>, // VecMap<layer>
    dirty: LayerDirty,
}

impl WorldMatrixSys {
    fn marked_dirty(&mut self, id: usize, id_tree: &SingleCaseImpl<IdTree>) {
        match id_tree.get(id) {
            Some(r) => {
                if r.layer != 0 {
                    let d = &mut self.dirty_mark_list[id];
                    if *d != r.layer {
                        if *d != 0 {
                            self.dirty.delete(id, *d);
                        }
                        *d = r.layer;
                        self.dirty.mark(id, r.layer);
                    }
                }
            }
            _ => (),
        };
    }

    fn cal_matrix(
        &mut self,
        idtree: &SingleCaseImpl<IdTree>,
        transform: &MultiCaseImpl<Node, Transform>,
        layout: &MultiCaseImpl<Node, Layout>,
        world_matrix: &mut MultiCaseImpl<Node, WorldMatrix>,
        default_table: &SingleCaseImpl<DefaultTable>,
    ) {
        let mut count = 0;
        // let time = std::time::Instant::now();
        let default_transform = default_table.get_unchecked::<Transform>();
        for (id, layer) in self.dirty.iter() {
            {
                let dirty_mark = match self.dirty_mark_list.get_mut(*id) {
                    Some(r) => r,
                    None => continue, //panic!("dirty_mark_list err: {}", *id),
                };
                if *dirty_mark == 0 {
                    continue;
                }
                *dirty_mark = 0;
            }

            let parent_id = match idtree.get(*id) {
                Some(r) => {
                    if layer == r.layer {
                        r.parent
                    } else {
                        continue;
                    }
                }
                None => continue, //panic!("cal_matrix error, idtree is not exist, id: {}", *id),
            };
            // let transform_value = get_or_default(parent_id, transform, default_table);
            let transform_value = match transform.get(parent_id) {
                Some(r) => r,
                None => default_transform,
            };
            recursive_cal_matrix(
                &mut self.dirty_mark_list,
                parent_id,
                *id,
                transform_value,
                idtree,
                transform,
                layout,
                world_matrix,
                default_transform,
                &mut count,
            );
        }
        self.dirty.clear();
    }
}

impl<'a> Runner<'a> for WorldMatrixSys {
    type ReadData = (
        &'a SingleCaseImpl<IdTree>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a SingleCaseImpl<DefaultTable>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, WorldMatrix>;
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        self.cal_matrix(read.0, read.1, read.2, write, read.3);
    }
}

impl<'a> EntityListener<'a, Node, CreateEvent> for WorldMatrixSys {
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, Transform>,
        &'a mut MultiCaseImpl<Node, WorldMatrix>,
    );
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        write.1.insert(event.id, WorldMatrix::default());
        match self.dirty_mark_list.get_mut(event.id) {
            None => {
                self.dirty_mark_list.insert(event.id, 0);
            }
            _ => (),
        };
    }
}

impl<'a> MultiCaseListener<'a, Node, Transform, ModifyEvent> for WorldMatrixSys {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData) {
        self.marked_dirty(event.id, read);
    }
}

impl<'a> MultiCaseListener<'a, Node, Transform, CreateEvent> for WorldMatrixSys {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, _write: Self::WriteData) {
        self.marked_dirty(event.id, read);
    }
}

impl<'a> MultiCaseListener<'a, Node, Transform, DeleteEvent> for WorldMatrixSys {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, _write: Self::WriteData) {
        self.marked_dirty(event.id, read);
    }
}

impl<'a> MultiCaseListener<'a, Node, Layout, ModifyEvent> for WorldMatrixSys {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData) {
        self.marked_dirty(event.id, read);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for WorldMatrixSys {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, _write: Self::WriteData) {
        self.marked_dirty(event.id, read);
    }
}

//取lefttop相对于父节点的变换原点的位置
#[inline]
fn get_lefttop_offset(layout: &Layout, parent_origin: &Point2, _parent_layout: &Layout) -> Point2 {
    Point2::new(
        // 当设置宽高为auto时 可能存在bug
        layout.left - parent_origin.x,
        layout.top - parent_origin.y,
    )
}

fn recursive_cal_matrix(
    dirty_mark_list: &mut VecMap<usize>,
    parent: usize,
    id: usize,
    parent_transform: &Transform,
    idtree: &SingleCaseImpl<IdTree>,
    transform: &MultiCaseImpl<Node, Transform>,
    layout: &MultiCaseImpl<Node, Layout>,
    world_matrix: &mut MultiCaseImpl<Node, WorldMatrix>,
    default_transform: &Transform,
    count: &mut usize,
) {
    // *count = 1 + *count;
    // match dirty_mark_list.get_mut(id) {
    //     Some(r) => *r = false,
    //     None => panic!("dirty_mark_list is no exist, id: {}", id),
    // }
    dirty_mark_list[id] = 0;

    let layout_value = &layout[id];
    let transform_value = match transform.get(id) {
        Some(r) => r,
        None => default_transform,
    };

    let matrix = if parent == 0 {
        transform_value.matrix(
            layout_value.width,
            layout_value.height,
            &Point2::new(layout_value.left, layout_value.top),
        )
    } else {
        let parent_layout = &layout[parent];
        let parent_world_matrix = &world_matrix[parent];
        let parent_transform_origin = parent_transform
            .origin
            .to_value(parent_layout.width, parent_layout.height);
        let offset = get_lefttop_offset(&layout_value, &parent_transform_origin, &parent_layout);
        parent_world_matrix
            * transform_value.matrix(layout_value.width, layout_value.height, &offset)
    };
    world_matrix
		.get_write(id).unwrap()
		.modify(|w: &mut WorldMatrix| {
			*w = matrix;
			true
		});
    let first = idtree[id].children.head;
    for child_id in idtree.iter(first) {
        recursive_cal_matrix(
            dirty_mark_list,
            id,
            child_id.0,
            transform_value,
            idtree,
            transform,
            layout,
            world_matrix,
            default_transform,
            count,
        );
    }
}

impl_system! {
    WorldMatrixSys,
    true,
    {
        EntityListener<Node, CreateEvent>
        MultiCaseListener<Node, Transform, ModifyEvent>
        MultiCaseListener<Node, Transform, CreateEvent>
        MultiCaseListener<Node, Transform, DeleteEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        SingleCaseListener<IdTree, CreateEvent>
    }
}

#[cfg(test)]
use atom::Atom;
#[cfg(test)]
use component::calc::ZDepth;
#[cfg(test)]
use component::user::{TransformFunc, TransformWrite};
#[cfg(test)]
use ecs::{Dispatcher, LendMut, SeqDispatcher, World};

#[test]
fn test() {
    let world = new_world();

    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = LendMut::lend_mut(&idtree);
    let notify = idtree.get_notify();
    let transforms = world.fetch_multi::<Node, Transform>().unwrap();
    let transforms = LendMut::lend_mut(&transforms);
    let layouts = world.fetch_multi::<Node, Layout>().unwrap();
    let layouts = LendMut::lend_mut(&layouts);
    let world_matrixs = world.fetch_multi::<Node, WorldMatrix>().unwrap();
    let world_matrixs = LendMut::lend_mut(&world_matrixs);
    let zdepths = world.fetch_multi::<Node, ZDepth>().unwrap();
    let zdepths = LendMut::lend_mut(&zdepths);

    let e0 = world.create_entity::<Node>();

    idtree.create(e0);
    idtree.insert_child(e0, 0, 0, Some(&notify)); //根
    transforms.insert(e0, Transform::default());
    zdepths.insert(e0, ZDepth::default());
    layouts.insert(
        e0,
        Layout {
            left: 0.0,
            top: 0.0,
            width: 900.0,
            height: 900.0,
            border_left: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
        },
    );

    world.run(&Atom::from("test_transform_sys"));

    let e00 = world.create_entity::<Node>();
    let e01 = world.create_entity::<Node>();
    let e02 = world.create_entity::<Node>();
    idtree.create(e00);
    idtree.insert_child(e00, e0, 1, Some(&notify));
    transforms.insert(e00, Transform::default());
    zdepths.insert(e00, ZDepth::default());
    layouts.insert(
        e00,
        Layout {
            left: 0.0,
            top: 0.0,
            width: 300.0,
            height: 900.0,
            border_left: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
        },
    );
    idtree.create(e01);
    idtree.insert_child(e01, e0, 2, Some(&notify));
    layouts.insert(
        e01,
        Layout {
            left: 300.0,
            top: 0.0,
            width: 300.0,
            height: 900.0,
            border_left: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
        },
    );
    transforms.insert(e01, Transform::default());
    zdepths.insert(e01, ZDepth::default());
    idtree.create(e02);
    idtree.insert_child(e02, e0, 3, Some(&notify));
    transforms.insert(e02, Transform::default());
    zdepths.insert(e02, ZDepth::default());
    layouts.insert(
        e02,
        Layout {
            left: 600.0,
            top: 0.0,
            width: 300.0,
            height: 900.0,
            border_left: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
        },
    );

    let e000 = world.create_entity::<Node>();
    let e001 = world.create_entity::<Node>();
    let e002 = world.create_entity::<Node>();
    idtree.create(e000);
    idtree.insert_child(e000, e00, 1, Some(&notify));
    layouts.insert(
        e000,
        Layout {
            left: 0.0,
            top: 0.0,
            width: 100.0,
            height: 900.0,
            border_left: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
        },
    );
    transforms.insert(e000, Transform::default());
    zdepths.insert(e000, ZDepth::default());
    idtree.create(e001);
    idtree.insert_child(e001, e00, 2, Some(&notify));
    transforms.insert(e001, Transform::default());
    zdepths.insert(e001, ZDepth::default());
    layouts.insert(
        e001,
        Layout {
            left: 100.0,
            top: 0.0,
            width: 100.0,
            height: 900.0,
            border_left: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
        },
    );
    idtree.create(e002);
    idtree.insert_child(e002, e00, 3, Some(&notify));
    transforms.insert(e002, Transform::default());
    zdepths.insert(e002, ZDepth::default());
    layouts.insert(
        e002,
        Layout {
            left: 200.0,
            top: 0.0,
            width: 100.0,
            height: 900.0,
            border_left: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
        },
    );

    let e010 = world.create_entity::<Node>();
    let e011 = world.create_entity::<Node>();
    let e012 = world.create_entity::<Node>();
    idtree.create(e010);
    idtree.insert_child(e010, e01, 1, Some(&notify));
    layouts.insert(
        e010,
        Layout {
            left: 0.0,
            top: 0.0,
            width: 100.0,
            height: 900.0,
            border_left: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
        },
    );
    transforms.insert(e010, Transform::default());
    zdepths.insert(e010, ZDepth::default());
    idtree.create(e011);
    idtree.insert_child(e011, e01, 2, Some(&notify));
    transforms.insert(e011, Transform::default());
    zdepths.insert(e011, ZDepth::default());
    layouts.insert(
        e011,
        Layout {
            left: 100.0,
            top: 0.0,
            width: 100.0,
            height: 900.0,
            border_left: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
        },
    );
    idtree.create(e012);
    idtree.insert_child(e012, e01, 3, Some(&notify));
    transforms.insert(e012, Transform::default());
    zdepths.insert(e012, ZDepth::default());
    layouts.insert(
        e012,
        Layout {
            left: 200.0,
            top: 0.0,
            width: 100.0,
            height: 900.0,
            border_left: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
        },
    );
    world.run(&Atom::from("test_world_matrix_sys"));

    transforms.get_write(e0).unwrap().modify(|transform: &mut Transform| {
        transform.funcs.push(TransformFunc::TranslateX(50.0));
        true
    });

    world.run(&Atom::from("test_transform_sys"));

    debug_println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
		&world_matrixs[e0],
		&world_matrixs[e00],
		&world_matrixs[e01],
		&world_matrixs[e02],
		&world_matrixs[e000],
		&world_matrixs[e001],
		&world_matrixs[e002],
		&world_matrixs[e010],
		&world_matrixs[e011],
		&world_matrixs[e012],
	);
}

#[cfg(test)]
fn new_world() -> World {
    let mut world = World::default();

    world.register_entity::<Node>();
    world.register_multi::<Node, Transform>();
    world.register_multi::<Node, Layout>();
    world.register_multi::<Node, WorldMatrix>();
    world.register_multi::<Node, ZDepth>();
    world.register_single::<IdTree>(IdTree::default());

    let system = CellWorldMatrixSys::new(WorldMatrixSys::default());
    world.register_system(Atom::from("system"), system);

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("system".to_string(), &world);

    world.add_dispatcher(Atom::from("test_world_matrix_sys"), dispatch);
    world
}
