/**
 *  计算show
 *  该系统默认为所有已经创建的Entity创建Show组件， 并监听Show和Display的创建修改， 以及监听idtree上的创建事件， 计算已经在idtree上存在的实体的Enable和Visibility
 */
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl};
use ecs::idtree::{ IdTree};

use component::user::{Show};
use component::Display;
use component::calc::{Visibility as CVisibility, VisibilityWrite as CVisibilityWrite, Enable as CEnable, EnableWrite as CEnableWrite};
use entity::{Node};

#[derive(Default)]
pub struct ShowSys;

impl ShowSys {
    fn modify_show(id: usize, idtree: &SingleCaseImpl<IdTree>, show: &MultiCaseImpl<Node, Show>, visibility: &mut MultiCaseImpl<Node, CVisibility>, enable: &mut MultiCaseImpl<Node, CEnable>){
        let parent_id = match idtree.get(id) {
            Some(node) => node.parent,
            None => return,
        };
        if parent_id > 0 {
            let parent_c_visibility = unsafe { **visibility.get_unchecked(parent_id) };
            let parent_c_enable = unsafe { **enable.get_unchecked(parent_id) };
            modify_show(parent_c_visibility, parent_c_enable, id, idtree, show, visibility, enable);
        }else {
            modify_show(true, true, id, idtree, show, visibility, enable);
        }
    }
}

impl<'a> EntityListener<'a, Node, CreateEvent> for ShowSys{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, Show>, &'a mut MultiCaseImpl<Node, CVisibility>, &'a mut MultiCaseImpl<Node, CEnable>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        write.0.insert(event.id, Show::default());
        write.1.insert(event.id, CVisibility::default());
        write.2.insert(event.id, CEnable::default());
    }
}

impl<'a> MultiCaseListener<'a, Node, Show, ModifyEvent> for ShowSys{
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Show>);
    type WriteData = ( &'a mut MultiCaseImpl<Node, CVisibility>, &'a mut MultiCaseImpl<Node, CEnable>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        ShowSys::modify_show(event.id, read.0, read.1, write.0, write.1);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for ShowSys{
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Show>);
    type WriteData = ( &'a mut MultiCaseImpl<Node, CVisibility>, &'a mut MultiCaseImpl<Node, CEnable>);
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        ShowSys::modify_show(event.id, read.0, read.1, write.0, write.1);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for ShowSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ( &'a mut MultiCaseImpl<Node, CVisibility>, &'a mut MultiCaseImpl<Node, CEnable>);
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData){
        cancel_visibility(event.id, read, write.0);
        cancel_enable(event.id, read, write.1);
    }
}

fn cancel_visibility(
    id: usize,
    id_tree: &SingleCaseImpl<IdTree>,
    visibility: &mut MultiCaseImpl<Node, CVisibility>,
){
    let mut write = unsafe { visibility.get_unchecked_write(id) };
    if write.value.0 == false {
        return;
    }
    write.set_0(false);
    let first = unsafe { id_tree.get_unchecked(id).children.head };
    for child in id_tree.iter(first) {
        cancel_visibility(child.0, id_tree, visibility);
    }
}

fn cancel_enable(
    id: usize,
    id_tree: &SingleCaseImpl<IdTree>,
    enable: &mut MultiCaseImpl<Node,CEnable>,
){
    let mut write = unsafe { enable.get_unchecked_write(id) };
    if write.value.0 == false {
        return;
    }
    write.set_0(false);
    let first = unsafe { id_tree.get_unchecked(id).children.head };
    for child in id_tree.iter(first) {
        cancel_enable(child.0, id_tree, enable);
    }
}
//递归计算不透明度， 将节点最终的不透明度设置在real_show组件上
fn modify_show(
    parent_c_visibility: bool,
    parent_c_enable: bool,
    id: usize,
    id_tree: &SingleCaseImpl<IdTree>,
    show: &MultiCaseImpl<Node, Show>,
    visibility: &mut MultiCaseImpl<Node,CVisibility>,
    enable: &mut MultiCaseImpl<Node,CEnable>,
) {
    let show_value = unsafe { show.get_unchecked(id) };
    let display_value = match show_value.get_display() {
        Display::Flex => true,
        Display::None => false,
    };
    let visibility_value = show_value.get_visibility();
    let enable_value = show_value.get_enable();

    let c_visibility = display_value && visibility_value && parent_c_visibility;
    let c_enable = c_visibility && enable_value && parent_c_enable;
    let mut visibility_write = unsafe { visibility.get_unchecked_write(id) };
    let mut enable_write = unsafe { enable.get_unchecked_write(id) };
    if c_visibility == **visibility_write.value && c_enable == **enable_write.value {
        return;
    }

    visibility_write.set_0(c_visibility);
    enable_write.set_0(c_enable);

    let first = unsafe { id_tree.get_unchecked(id).children.head };
    for child_id in id_tree.iter(first) {
        modify_show(c_visibility, c_enable, child_id.0, id_tree, show, visibility, enable);
    }
}

impl_system!{
    ShowSys,
    false,
    {
        EntityListener<Node, CreateEvent>
        MultiCaseListener<Node, Show, ModifyEvent>
        SingleCaseListener<IdTree, CreateEvent>
    }
}


#[cfg(test)]
use ecs::{World, BorrowMut, SeqDispatcher, Dispatcher};
#[cfg(test)]
use atom::Atom;
#[cfg(test)]
use component::user::ShowWrite;

#[test]
fn test(){
    let world = new_world();

    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = BorrowMut::borrow_mut(&idtree);
    let notify = idtree.get_notify();
    let shows = world.fetch_multi::<Node, Show>().unwrap();
    let shows = BorrowMut::borrow_mut(&shows);
    let cvisibilitys = world.fetch_multi::<Node, CVisibility>().unwrap();
    let cvisibilitys = BorrowMut::borrow_mut(&cvisibilitys);
    let cenables = world.fetch_multi::<Node, CEnable>().unwrap();
    let cenables = BorrowMut::borrow_mut(&cenables);

    let e0 = world.create_entity::<Node>();
    
    idtree.create(e0);
    idtree.insert_child(e0, 0, 0, Some(&notify)); //根
    shows.insert(e0, Show::default());

    world.run(&Atom::from("test_show_sys"));
    
    let e00 = world.create_entity::<Node>();
    let e01 = world.create_entity::<Node>();
    let e02 = world.create_entity::<Node>();
    idtree.create(e00);
    idtree.insert_child(e00, e0, 1, Some(&notify));
    shows.insert(e00, Show::default());
    idtree.create(e01);
    idtree.insert_child(e01, e0, 2, Some(&notify));
    shows.insert(e01, Show::default());
    idtree.create(e02);
    idtree.insert_child(e02, e0, 3, Some(&notify));
    shows.insert(e02, Show::default());

    let e000 = world.create_entity::<Node>();
    let e001 = world.create_entity::<Node>();
    let e002 = world.create_entity::<Node>();
    idtree.create(e000);
    idtree.insert_child(e000, e00, 1, Some(&notify));
    shows.insert(e000, Show::default());
    idtree.create(e001);
    idtree.insert_child(e001, e00, 2, Some(&notify));
    shows.insert(e001, Show::default());
    idtree.create(e002);
    idtree.insert_child(e002, e00, 3, Some(&notify));
    shows.insert(e002, Show::default());

    let e010 = world.create_entity::<Node>();
    let e011 = world.create_entity::<Node>();
    let e012 = world.create_entity::<Node>();
    idtree.create(e010);
    idtree.insert_child(e010, e01, 1, Some(&notify));
    shows.insert(e010, Show::default());
    idtree.create(e011);
    idtree.insert_child(e011, e01, 2, Some(&notify));
    shows.insert(e011, Show::default());
    idtree.create(e012);
    idtree.insert_child(e012, e01, 3, Some(&notify));
    shows.insert(e012, Show::default());
    world.run(&Atom::from("test_show_sys"));

    unsafe { shows.get_unchecked_write(e00)}.modify(|show: &mut Show|{
        show.set_visibility(false);
        true
    });

    unsafe { shows.get_unchecked_write(e01)}.modify(|show: &mut Show|{
        show.set_enable(false);
        true
    });

    unsafe { shows.get_unchecked_write(e02)}.modify(|show: &mut Show|{
        show.set_display(Display::None);
        true
    });

    world.run(&Atom::from("test_show_sys"));

    println!("cvisibilitys, e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
        unsafe{cvisibilitys.get_unchecked(e0)},
        unsafe{cvisibilitys.get_unchecked(e00)},
        unsafe{cvisibilitys.get_unchecked(e01)},
        unsafe{cvisibilitys.get_unchecked(e02)},
        unsafe{cvisibilitys.get_unchecked(e000)},
        unsafe{cvisibilitys.get_unchecked(e001)},
        unsafe{cvisibilitys.get_unchecked(e002)},
        unsafe{cvisibilitys.get_unchecked(e010)},
        unsafe{cvisibilitys.get_unchecked(e011)},
        unsafe{cvisibilitys.get_unchecked(e012)},
    );

    println!("cenables, e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
        unsafe{cenables.get_unchecked(e0)},
        unsafe{cenables.get_unchecked(e00)},
        unsafe{cenables.get_unchecked(e01)},
        unsafe{cenables.get_unchecked(e02)},
        unsafe{cenables.get_unchecked(e000)},
        unsafe{cenables.get_unchecked(e001)},
        unsafe{cenables.get_unchecked(e002)},
        unsafe{cenables.get_unchecked(e010)},
        unsafe{cenables.get_unchecked(e011)},
        unsafe{cenables.get_unchecked(e012)},
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

    world.add_dispatcher( Atom::from("test_show_sys"), dispatch);
    world
}

