use single::IdTree;
/**
 * 监听transform和layout组件， 利用transform和layout递归计算节点的世界矩阵（worldmatrix组件）
 */
use ecs::{
    CreateEvent, EntityListener, ModifyEvent, MultiCaseImpl, MultiCaseListener, SingleCaseImpl,
    SingleCaseListener,
};

use component::calc::HSV;
use component::user::Filter;
use entity::Node;

#[derive(Default)]
pub struct FilterSys;

// impl<'a> EntityListener<'a, Node, CreateEvent> for FilterSys {
//     type ReadData = ();
//     type WriteData = &'a mut MultiCaseImpl<Node, HSV>;
//     fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, hsvs: Self::WriteData) {
//         hsvs.insert(event.id, HSV::default());
//     }
// }

impl<'a> MultiCaseListener<'a, Node, Filter, CreateEvent> for FilterSys {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Filter>);
    type WriteData = &'a mut MultiCaseImpl<Node, HSV>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, hsvs: Self::WriteData) {
        let (idtree, filters) = read;
        cal_hsv(event.id, idtree, filters, hsvs);
    }
}

impl<'a> MultiCaseListener<'a, Node, Filter, ModifyEvent> for FilterSys {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Filter>);
    type WriteData = &'a mut MultiCaseImpl<Node, HSV>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, hsvs: Self::WriteData) {
        let (idtree, filters) = read;
        cal_hsv(event.id, idtree, filters, hsvs);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for FilterSys {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Filter>);
    type WriteData = &'a mut MultiCaseImpl<Node, HSV>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, hsvs: Self::WriteData) {
        let (idtree, filters) = read;
        cal_hsv(event.id, idtree, filters, hsvs);
    }
}

#[inline]
fn cal_hsv(
    id: usize,
    idtree: &SingleCaseImpl<IdTree>,
    filters: &MultiCaseImpl<Node, Filter>,
    hsvs: &mut MultiCaseImpl<Node, HSV>,
) {
    let parent_id = match idtree.get(id) {
        Some(node) => {
            if node.layer() != 0 {
                node.parent()
            } else {
                return;
            }
        }
        None => return,
    };
    let hsv = hsvs[parent_id].clone();

    recursive_cal_hsv(id, idtree, &hsv, filters, hsvs)
}

#[inline]
fn recursive_cal_hsv(
    id: usize,
    idtree: &SingleCaseImpl<IdTree>,
    parent_hsv: &HSV,
    filters: &MultiCaseImpl<Node, Filter>,
    hsvs: &mut MultiCaseImpl<Node, HSV>,
) {
    let old_hsv =  hsvs[id].clone();
    let hsv = match filters.get(id) {
        Some(filter) => {
            let hsv = HSV {
                h: cal_h_from_hue(filter.hue_rotate + parent_hsv.h),
                s: cal_range(filter.saturate + parent_hsv.s, -1.0, 1.0),
                v: cal_range(filter.bright_ness + parent_hsv.v, -1.0, 1.0),
            };
            if hsv.h != old_hsv.h || hsv.s != old_hsv.s || hsv.v != old_hsv.v {
                hsvs.insert(id, hsv.clone());
            }
            hsv
        }
        None => {
            if parent_hsv.h != old_hsv.h || parent_hsv.s != old_hsv.s || parent_hsv.v != old_hsv.v {
                hsvs.insert(id, parent_hsv.clone());
            }
            parent_hsv.clone()
        }
    };
    let first = idtree[id].children().head;
    for child_id in idtree.iter(first) {
        recursive_cal_hsv(child_id.0, idtree, &hsv, filters, hsvs);
    }
}

// 计算hue， hue的值在-180 ~ 180 度范围内
fn cal_h_from_hue(mut hue_rotate: f32) -> f32 {
    if hue_rotate > 0.5 {
        loop {
            if hue_rotate <= 0.5 {
                return hue_rotate;
            }
            hue_rotate -= 1.0;
        }
    } else {
        loop {
            if hue_rotate >= -0.5 {
                return hue_rotate;
            }
            hue_rotate += 1.0;
        }
    }
}

fn cal_range(value: f32, min: f32, max: f32) -> f32 {
    if value >= max {
        return max;
    } else if value <= min {
        return min;
    } else {
        return value;
    }
}

impl_system! {
    FilterSys,
    false,
    {
        // EntityListener<Node, CreateEvent>
        MultiCaseListener<Node, Filter, CreateEvent>
        MultiCaseListener<Node, Filter, ModifyEvent>
        SingleCaseListener<IdTree, CreateEvent>
    }
}

#[cfg(test)]
use atom::Atom;
#[cfg(test)]
use ecs::{Dispatcher, LendMut, SeqDispatcher, World};

#[test]
fn test() {
    let world = new_world();

    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = LendMut::lend_mut(&idtree);
    // let notify = idtree.get_notify();
    let filters = world.fetch_multi::<Node, Filter>().unwrap();
    let filters = LendMut::lend_mut(&filters);
    let hsvs = world.fetch_multi::<Node, HSV>().unwrap();
    let hsvs = LendMut::lend_mut(&hsvs);

    let e0 = world.create_entity::<Node>();

    idtree.create(e0);
    idtree.insert_child(e0, 0, 0); //根
    let filter = Filter {
        hue_rotate: 380.0,
        bright_ness: 0.5,
        saturate: 0.3,
    };
    filters.insert(e0, filter.clone());

    world.run(&Atom::from("test_filter_sys"));

    let e00 = world.create_entity::<Node>();
    let e01 = world.create_entity::<Node>();
    let e02 = world.create_entity::<Node>();
    idtree.create(e00);
    idtree.insert_child(e00, e0, 1);
    filters.insert(e00, filter.clone());
    idtree.create(e01);
    idtree.insert_child(e01, e0, 2);
    filters.insert(e01, filter.clone());
    idtree.create(e02);
    idtree.insert_child(e02, e0, 3);
    filters.insert(e02, filter.clone());

    let e000 = world.create_entity::<Node>();
    let e001 = world.create_entity::<Node>();
    let e002 = world.create_entity::<Node>();
    idtree.create(e000);
    idtree.insert_child(e000, e00, 1);
    idtree.create(e001);
    idtree.insert_child(e001, e00, 2);
    idtree.create(e002);
    idtree.insert_child(e002, e00, 3);

    let e010 = world.create_entity::<Node>();
    let e011 = world.create_entity::<Node>();
    let e012 = world.create_entity::<Node>();
    idtree.create(e010);
    idtree.insert_child(e010, e01, 1);
    filters.insert(e010, Filter::default());
    idtree.create(e011);
    idtree.insert_child(e011, e01, 2);
    filters.insert(e011, Filter::default());
    idtree.create(e012);
    idtree.insert_child(e012, e01, 3);
    filters.insert(e012, Filter::default());
    world.run(&Atom::from("test_opacity_sys"));

    // world.run(&Atom::from("test_opacity_sys"));

    debug_println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
        hsvs[e0],
        hsvs[e00],
        hsvs[e01],
        hsvs[e02],
        hsvs[e000],
        hsvs[e001],
        hsvs[e002],
        hsvs[e010],
        hsvs[e011],
        hsvs[e012],
    );
}

#[cfg(test)]
fn new_world() -> World {
    let mut world = World::default();

    world.register_entity::<Node>();
    world.register_multi::<Node, Filter>();
    world.register_multi::<Node, HSV>();
    world.register_single::<IdTree>(IdTree::default());

    let system = CellFilterSys::new(FilterSys::default());
    world.register_system(Atom::from("system"), system);

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("system".to_string(), &world);

    world.add_dispatcher(Atom::from("test_filter_sys"), dispatch);
    world
}
