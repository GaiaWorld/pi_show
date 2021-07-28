/**
 *  计算show
 *  该系统默认为所有已经创建的Entity创建Show组件， 并监听Show和Display的创建修改， 以及监听idtree上的创建事件， 计算已经在idtree上存在的实体的Enable和Visibility
 */
use ecs::{
    CreateEvent, DeleteEvent, EntityListener, ModifyEvent, MultiCaseImpl, MultiCaseListener,
    SingleCaseImpl, SingleCaseListener,
};
use flex_layout::Display;

use crate::component::calc::{NodeState,
    Enable as CEnable, EnableWrite as CEnableWrite, Visibility as CVisibility,
    VisibilityWrite as CVisibilityWrite,
};
use crate::component::user::*;
use crate::entity::Node;
use crate::single::IdTree;

#[derive(Default)]
pub struct ShowSys;

impl ShowSys {
    fn modify_show(
        id: usize,
        idtree: &SingleCaseImpl<IdTree>,
        show: &MultiCaseImpl<Node, Show>,
        visibility: &mut MultiCaseImpl<Node, CVisibility>,
		enable: &mut MultiCaseImpl<Node, CEnable>,
		node_states: &MultiCaseImpl<Node, NodeState>,
    ) {
        let parent_id = match idtree.get(id) {
            Some(node) => {
                if node.layer() == 0 {
                    return;
                }
                node.parent()
            }
            None => return,
		};
		if !node_states[id].0.is_rnode() {
			return;
		}
        if parent_id > 0 {
            let parent_c_visibility = *visibility[parent_id];
            let parent_c_enable = *enable[parent_id];
            modify_show(
                parent_c_visibility,
                parent_c_enable,
                id,
                idtree,
                show,
                visibility,
				enable,
				node_states,
            );
        } else {
            modify_show(true, true, id, idtree, show, visibility, enable, node_states);
        }
    }
}

// impl<'a> EntityListener<'a, Node, CreateEvent> for ShowSys {
//     type ReadData = ();
//     type WriteData = (
//         &'a mut MultiCaseImpl<Node, Show>,
//         &'a mut MultiCaseImpl<Node, CVisibility>,
//         &'a mut MultiCaseImpl<Node, CEnable>,
//     );
//     fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
//         // write.0.insert(event.id, Show::default());
//         // write.1.insert(event.id, CVisibility::default());
//         // write.2.insert(event.id, CEnable::default());
//     }
// }

impl<'a> MultiCaseListener<'a, Node, Show, ModifyEvent> for ShowSys {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Show>, &'a MultiCaseImpl<Node, NodeState>);
    type WriteData = (
        &'a mut MultiCaseImpl<Node, CVisibility>,
        &'a mut MultiCaseImpl<Node, CEnable>,
    );
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
        ShowSys::modify_show(event.id, read.0, read.1, write.0, write.1, read.2);
    }
}

impl<'a> MultiCaseListener<'a, Node, Show, CreateEvent> for ShowSys {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Show>, &'a MultiCaseImpl<Node, NodeState>);
    type WriteData = (
        &'a mut MultiCaseImpl<Node, CVisibility>,
        &'a mut MultiCaseImpl<Node, CEnable>,
    );
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData) {
        ShowSys::modify_show(event.id, read.0, read.1, write.0, write.1, read.2);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for ShowSys {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Show>, &'a MultiCaseImpl<Node, NodeState>);
    type WriteData = (
        &'a mut MultiCaseImpl<Node, CVisibility>,
        &'a mut MultiCaseImpl<Node, CEnable>,
    );
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData) {
        ShowSys::modify_show(event.id, read.0, read.1, write.0, write.1, read.2);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for ShowSys {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, NodeState>);
    type WriteData = (
        &'a mut MultiCaseImpl<Node, CVisibility>,
        &'a mut MultiCaseImpl<Node, CEnable>,
    );
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData) {
        cancel_visibility(event.id, read.0, write.0, read.1);
        cancel_enable(event.id, read.0, write.1, read.1);
    }
}

fn cancel_visibility(
    id: usize,
    id_tree: &SingleCaseImpl<IdTree>,
	visibility: &mut MultiCaseImpl<Node, CVisibility>,
	node_states: &MultiCaseImpl<Node, NodeState>,
) {
	if !node_states[id].0.is_rnode(){
		return;
	}
    let mut write = unsafe{ visibility.get_unchecked_write(id) };
    if write.value.0 == false {
        return;
    }
    write.set_0(false);
    let first = id_tree[id].children().head;
    for child in id_tree.iter(first) {
        cancel_visibility(child.0, id_tree, visibility, node_states);
    }
}

//计算enable
fn cancel_enable(
    id: usize,
    id_tree: &SingleCaseImpl<IdTree>,
	enable: &mut MultiCaseImpl<Node, CEnable>,
	node_states: &MultiCaseImpl<Node, NodeState>,
) {
	if !node_states[id].0.is_rnode(){
		return;
	}
    let mut write = unsafe{ enable.get_unchecked_write(id) };
    if write.value.0 == false {
        return;
    }
    write.set_0(false);
    let first = id_tree[id].children().head;
    for child in id_tree.iter(first) {
        cancel_enable(child.0, id_tree, enable, node_states);
    }
}
//递归计算不透明度， 将节点最终的不透明度设置在real_show组件上
fn modify_show(
    parent_c_visibility: bool,
    parent_c_enable: bool,
    id: usize,
    id_tree: &SingleCaseImpl<IdTree>,
    show: &MultiCaseImpl<Node, Show>,
    visibility: &mut MultiCaseImpl<Node, CVisibility>,
	enable: &mut MultiCaseImpl<Node, CEnable>,
	node_states: &MultiCaseImpl<Node, NodeState>,
) {
	if !node_states[id].0.is_rnode() {
		return;
	}
    let show_value = &show[id];
    let display_value = match show_value.get_display() {
        Display::Flex => true,
        Display::None => false,
    };
    let visibility_value = show_value.get_visibility();
    let enable_value = show_value.get_enable();

    let c_visibility = display_value && visibility_value && parent_c_visibility;
    let c_enable = match enable_value {
        EnableType::Visible => true,
        EnableType::Auto => parent_c_enable,
        EnableType::None => false,
    };
    let c_enable = c_visibility && c_enable;
    let mut visibility_write = unsafe { visibility.get_unchecked_write(id) };
    let mut enable_write = unsafe {  enable.get_unchecked_write(id)};
    // if c_visibility == **visibility_write.value && c_enable == **enable_write.value {
    //     println!("c_visibility1-------------------{}, {}, {}, {}", c_visibility, **visibility_write.value, c_enable, **enable_write.value);
    //     return;
    // }

    visibility_write.set_0(c_visibility);
    enable_write.set_0(c_enable);

    let first = id_tree[id].children().head;
    for child_id in id_tree.iter(first) {
        modify_show(
            c_visibility,
            c_enable,
            child_id.0,
            id_tree,
            show,
            visibility,
			enable,
			node_states,
        );
    }
}

impl_system! {
    ShowSys,
    false,
    {
        // EntityListener<Node, CreateEvent>
		MultiCaseListener<Node, Show, ModifyEvent>
		MultiCaseListener<Node, Show, CreateEvent>
        SingleCaseListener<IdTree, CreateEvent>
        SingleCaseListener<IdTree, DeleteEvent>
    }
}

#[cfg(test)]
use atom::Atom;
#[cfg(test)]
use crate::component::user::ShowWrite;
#[cfg(test)]
use ecs::{Dispatcher, LendMut, SeqDispatcher, World};

#[test]
fn test() {
    let world = new_world();

    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = LendMut::lend_mut(&idtree);
    // let notify = idtree.get_notify();
    let shows = world.fetch_multi::<Node, Show>().unwrap();
    let shows = LendMut::lend_mut(&shows);
    let cvisibilitys = world.fetch_multi::<Node, CVisibility>().unwrap();
    let cvisibilitys = LendMut::lend_mut(&cvisibilitys);
    let cenables = world.fetch_multi::<Node, CEnable>().unwrap();
    let cenables = LendMut::lend_mut(&cenables);

    let e0 = world.create_entity::<Node>();

    idtree.create(e0);
    idtree.insert_child(e0, 0, 0); //根
    shows.insert(e0, Show::default());

    world.run(&Atom::from("test_show_sys"));

    let e00 = world.create_entity::<Node>();
    let e01 = world.create_entity::<Node>();
    let e02 = world.create_entity::<Node>();
    idtree.create(e00);
    idtree.insert_child(e00, e0, 1);
    shows.insert(e00, Show::default());
    idtree.create(e01);
    idtree.insert_child(e01, e0, 2);
    shows.insert(e01, Show::default());
    idtree.create(e02);
    idtree.insert_child(e02, e0, 3);
    shows.insert(e02, Show::default());

    let e000 = world.create_entity::<Node>();
    let e001 = world.create_entity::<Node>();
    let e002 = world.create_entity::<Node>();
    idtree.create(e000);
    idtree.insert_child(e000, e00, 1);
    shows.insert(e000, Show::default());
    idtree.create(e001);
    idtree.insert_child(e001, e00, 2);
    shows.insert(e001, Show::default());
    idtree.create(e002);
    idtree.insert_child(e002, e00, 3);
    shows.insert(e002, Show::default());

    let e010 = world.create_entity::<Node>();
    let e011 = world.create_entity::<Node>();
    let e012 = world.create_entity::<Node>();
    idtree.create(e010);
    idtree.insert_child(e010, e01, 1);
    shows.insert(e010, Show::default());
    idtree.create(e011);
    idtree.insert_child(e011, e01, 2);
    shows.insert(e011, Show::default());
    idtree.create(e012);
    idtree.insert_child(e012, e01, 3);
    shows.insert(e012, Show::default());
    world.run(&Atom::from("test_show_sys"));

    unsafe { shows.get_unchecked_write(e00)}.modify(|show: &mut Show| {
        show.set_visibility(false);
        true
    });

    unsafe { shows.get_unchecked_write(e01)}.modify(|show: &mut Show| {
        show.set_enable(EnableType::None);
        true
    });

    unsafe { shows.get_unchecked_write(e02)}.modify(|show: &mut Show| {
        show.set_display(Display::None);
        true
    });

    unsafe { shows.get_unchecked_write(e010)}.modify(|show: &mut Show| {
        show.set_enable(EnableType::Visible);
        true
    });

    world.run(&Atom::from("test_show_sys"));

    debug_println!("cvisibilitys, e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
        &cvisibilitys[e0],
        &cvisibilitys[e00],
        &cvisibilitys[e01],
        &cvisibilitys[e02],
        &cvisibilitys[e000],
        &cvisibilitys[e001],
        &cvisibilitys[e002],
        &cvisibilitys[e010],
        &cvisibilitys[e011],
        &cvisibilitys[e012],
    );

    debug_println!("cenables, e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
        &cenables[e0],
        &cenables[e00],
        &cenables[e01],
        &cenables[e02],
        &cenables[e000],
        &cenables[e001],
        &cenables[e002],
        &cenables[e010],
        &cenables[e011],
        &cenables[e012],
    );
}

#[cfg(test)]
fn new_world() -> World {
    let mut world = World::default();

    world.register_entity::<Node>();
    world.register_multi::<Node, Show>();
    world.register_multi::<Node, CVisibility>();
    world.register_multi::<Node, CEnable>();
    world.register_single::<IdTree>(IdTree::default());

    let system = CellShowSys::new(ShowSys::default());
    world.register_system(Atom::from("system"), system);

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("system".to_string(), &world);

    world.add_dispatcher(Atom::from("test_show_sys"), dispatch);
    world
}
