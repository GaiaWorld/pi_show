/**
 *  计算opacity
 *  该系统默认为所有已经创建的Entity创建Opacity组件， 并监听Opacity的创建修改， 以及监听idtree上的创建事件， 计算已经在idtree上存在的实体的Opacity
 */
use ecs::{CreateEvent, ModifyEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl};
use ecs::idtree::{ IdTree};

use component::user::{ Opacity};
use component::calc::{Opacity as COpacity, OpacityWrite as COpacityWrite};
use entity::{Node};

#[derive(Default)]
pub struct OpacitySys;

impl OpacitySys {
    fn modify_opacity(id: usize, idtree: &SingleCaseImpl<IdTree>, opacity: &MultiCaseImpl<Node, Opacity>, c_opacity: &mut MultiCaseImpl<Node, COpacity>){
        let parent_id = match idtree.get(id) {
            Some(node) => node.parent,
            None => return,
        };
        if parent_id > 0 {
            let parent_c_opacity = unsafe { **c_opacity.get_unchecked(parent_id) };
            modify_opacity(parent_c_opacity, id, idtree, opacity, c_opacity);
        }else {
            modify_opacity(1.0, id, idtree, opacity, c_opacity);
        }
    }
}

impl<'a> EntityListener<'a, Node, CreateEvent> for OpacitySys{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, Opacity>, &'a mut MultiCaseImpl<Node, COpacity>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        write.0.insert(event.id, Opacity::default());
        write.1.insert(event.id, COpacity::default());
    }
}

impl<'a> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for OpacitySys{
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Opacity>);
    type WriteData = &'a mut MultiCaseImpl<Node, COpacity>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        OpacitySys::modify_opacity(event.id, read.0, read.1, write);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for OpacitySys{
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Opacity>);
    type WriteData = &'a mut MultiCaseImpl<Node, COpacity>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        OpacitySys::modify_opacity(event.id, read.0, read.1, write);
    }
}

//递归计算不透明度， 将节点最终的不透明度设置在real_opacity组件上
fn modify_opacity(
    parent_real_opacity: f32,
    id: usize,
    id_tree: &SingleCaseImpl<IdTree>,
    opacity: &MultiCaseImpl<Node, Opacity>,
    copacity: &mut MultiCaseImpl<Node,COpacity>
) {
    let opacity_value: f32 = unsafe { **opacity.get_unchecked(id) };
    let node_real_opacity = opacity_value * parent_real_opacity;
    unsafe { copacity.get_unchecked_write(id) }.set_0(node_real_opacity);

    let first = unsafe { id_tree.get_unchecked(id).children.head };
    for child_id in id_tree.iter(first) {
        modify_opacity(node_real_opacity, child_id.0, id_tree, opacity, copacity);
    }
}

impl_system!{
    OpacitySys,
    false,
    {
        EntityListener<Node, CreateEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
        SingleCaseListener<IdTree, CreateEvent>
    }
}


#[cfg(test)]
use ecs::{World, LendMut, SeqDispatcher, Dispatcher};
#[cfg(test)]
use atom::Atom;
#[cfg(test)]
use component::user::OpacityWrite;

#[test]
fn test(){
    let world = new_world();

    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = LendMut::lend_mut(&idtree);
    let notify = idtree.get_notify();
    let opacitys = world.fetch_multi::<Node, Opacity>().unwrap();
    let opacitys = LendMut::lend_mut(&opacitys);
    let copacitys = world.fetch_multi::<Node, COpacity>().unwrap();
    let copacitys = LendMut::lend_mut(&copacitys);

    let e0 = world.create_entity::<Node>();
    
    idtree.create(e0);
    idtree.insert_child(e0, 0, 0, Some(&notify)); //根
    opacitys.insert(e0, Opacity::default());

    world.run(&Atom::from("test_opacity_sys"));
    
    let e00 = world.create_entity::<Node>();
    let e01 = world.create_entity::<Node>();
    let e02 = world.create_entity::<Node>();
    idtree.create(e00);
    idtree.insert_child(e00, e0, 1, Some(&notify));
    opacitys.insert(e00, Opacity::default());
    idtree.create(e01);
    idtree.insert_child(e01, e0, 2, Some(&notify));
    opacitys.insert(e01, Opacity::default());
    idtree.create(e02);
    idtree.insert_child(e02, e0, 3, Some(&notify));
    opacitys.insert(e02, Opacity::default());

    let e000 = world.create_entity::<Node>();
    let e001 = world.create_entity::<Node>();
    let e002 = world.create_entity::<Node>();
    idtree.create(e000);
    idtree.insert_child(e000, e00, 1, Some(&notify));
    opacitys.insert(e000, Opacity::default());
    idtree.create(e001);
    idtree.insert_child(e001, e00, 2, Some(&notify));
    opacitys.insert(e001, Opacity::default());
    idtree.create(e002);
    idtree.insert_child(e002, e00, 3, Some(&notify));
    opacitys.insert(e002, Opacity::default());

    let e010 = world.create_entity::<Node>();
    let e011 = world.create_entity::<Node>();
    let e012 = world.create_entity::<Node>();
    idtree.create(e010);
    idtree.insert_child(e010, e01, 1, Some(&notify));
    opacitys.insert(e010, Opacity::default());
    idtree.create(e011);
    idtree.insert_child(e011, e01, 2, Some(&notify));
    opacitys.insert(e011, Opacity::default());
    idtree.create(e012);
    idtree.insert_child(e012, e01, 3, Some(&notify));
    opacitys.insert(e012, Opacity::default());
    world.run(&Atom::from("test_opacity_sys"));

    unsafe { opacitys.get_unchecked_write(e0)}.set_0(0.5);
    unsafe { opacitys.get_unchecked_write(e00)}.set_0(0.5);

    world.run(&Atom::from("test_opacity_sys"));

    println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
        unsafe{copacitys.get_unchecked(e0)},
        unsafe{copacitys.get_unchecked(e00)},
        unsafe{copacitys.get_unchecked(e01)},
        unsafe{copacitys.get_unchecked(e02)},
        unsafe{copacitys.get_unchecked(e000)},
        unsafe{copacitys.get_unchecked(e001)},
        unsafe{copacitys.get_unchecked(e002)},
        unsafe{copacitys.get_unchecked(e010)},
        unsafe{copacitys.get_unchecked(e011)},
        unsafe{copacitys.get_unchecked(e012)},
    );
}

#[cfg(test)]
fn new_world() -> World {
    let mut world = World::default();

    world.register_entity::<Node>();
    world.register_multi::<Node, Opacity>();
    world.register_multi::<Node, COpacity>();
    world.register_single::<IdTree>(IdTree::default());
     
    let system = CellOpacitySys::new(OpacitySys::default());
    world.register_system(Atom::from("system"), system);

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("system".to_string(), &world);

    world.add_dispatcher( Atom::from("test_opacity_sys"), dispatch);
    world
}

