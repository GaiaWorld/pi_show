/**
 *  计算opacity
 *  该系统默认为所有已经创建的Entity创建BoxColor组件， 并监听BoxColor的创建修改， 以及监听idtree上的创建事件， 计算已经在idtree上存在的实体的BoxColor
 */
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl};
use ecs::idtree::{ IdTree};
use map::vecmap::VecMap;

use component::user::{BoxColor};
use component::calc::{BoxColor as CBoxColor, BoxColorWrite as CBoxColorWrite, RenderObj, Visibility, WorldMatrix, Transform, Opacity, ByOverflow};
use entity::{Node};

#[derive(Default)]
pub struct BoxUboSys{
    box_render_map: VecMap<usize>,
}

// 插入渲染对象
impl<'a> MultiCaseListener<'a, Node, BoxColor, CreateEvent> for BoxUboSys{
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, BoxColor>);
    type WriteData = &'a mut MultiCaseImpl<Node, RenderObj>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let render_obj = RenderObj::default();
        // 设置ubo
        let index = write.insert(RenderObj::default());
        self.box_render_map.insert(event.id, index);
    }
}

// 删除渲染对象
impl<'a> MultiCaseListener<'a, Node, BoxColor, DeleteEvent> for BoxUboSys{
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, RenderObj>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData){
        let index = self.box_render_map.remove(event.id).unwrap();
        write.remove(index);
    }
}

// BoxColor变化, 修改ubo
impl<'a> MultiCaseListener<'a, Node, BoxColor, ModifyEvent> for BoxUboSys{
    type ReadData = (&'a MultiCaseImpl<Node, BoxColor>);
    type WriteData = &'a mut MultiCaseImpl<Node, RenderObj>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        // 设置ubo TODO
    }
}

//世界矩阵变化， 设置ubo
impl<'a> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for BoxUboSys{
    type ReadData = (&'a MultiCaseImpl<Node, WorldMatrix>, &'a MultiCaseImpl<Node, Transform>,);
    type WriteData = &'a mut MultiCaseImpl<Node, RenderObj>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        // 设置ubo TODO
    }
}

//不透明度变化， 设置ubo
impl<'a> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for BoxUboSys{
    type ReadData = &'a MultiCaseImpl<Node, Opacity>;
    type WriteData = &'a mut MultiCaseImpl<Node, RenderObj>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        // 设置ubo TODO
    }
}

//by_overfolw变化， 设置ubo
impl<'a> MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for BoxUboSys{
    type ReadData = &'a MultiCaseImpl<Node, ByOverflow>;
    type WriteData = &'a mut MultiCaseImpl<Node, RenderObj>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        // 设置ubo TODO
    }
}

// 设置visibility
impl<'a> MultiCaseListener<'a, Node, Visibility, ModifyEvent> for BoxUboSys{
    type ReadData = &'a MultiCaseImpl<Node, Visibility>;
    type WriteData = &'a mut MultiCaseImpl<Node, RenderObj>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        match self.box_render_map.get(event.id); {
            Some(index) => {
                unsafe {write.get_unchecked_write(index)}.set_visibility(unsafe {read.get_unchecked(event.id).0});
            },
            None => (),
        }
    }
}

// impl_system!{
//     BoxUboSys,
//     false,
//     {
//         EntityListener<Node, CreateEvent>
//         MultiCaseListener<Node, BoxColor, ModifyEvent>
//         SingleCaseListener<IdTree, CreateEvent>
//     }
// }


// #[cfg(test)]
// use ecs::{World, BorrowMut, SeqDispatcher, Dispatcher};
// #[cfg(test)]
// use atom::Atom;
// #[cfg(test)]
// use component::user::BoxColorWrite;

// #[test]
// fn test(){
//     let world = new_world();

//     let idtree = world.fetch_single::<IdTree>().unwrap();
//     let idtree = BorrowMut::borrow_mut(&idtree);
//     let notify = idtree.get_notify();
//     let opacitys = world.fetch_multi::<Node, BoxColor>().unwrap();
//     let opacitys = BorrowMut::borrow_mut(&opacitys);
//     let copacitys = world.fetch_multi::<Node, CBoxColor>().unwrap();
//     let copacitys = BorrowMut::borrow_mut(&copacitys);

//     let e0 = world.create_entity::<Node>();
    
//     idtree.create(e0);
//     idtree.insert_child(e0, 0, 0, Some(&notify)); //根
//     opacitys.insert(e0, BoxColor::default());

//     world.run(&Atom::from("test_opacity_sys"));
    
//     let e00 = world.create_entity::<Node>();
//     let e01 = world.create_entity::<Node>();
//     let e02 = world.create_entity::<Node>();
//     idtree.create(e00);
//     idtree.insert_child(e00, e0, 1, Some(&notify));
//     opacitys.insert(e00, BoxColor::default());
//     idtree.create(e01);
//     idtree.insert_child(e01, e0, 2, Some(&notify));
//     opacitys.insert(e01, BoxColor::default());
//     idtree.create(e02);
//     idtree.insert_child(e02, e0, 3, Some(&notify));
//     opacitys.insert(e02, BoxColor::default());

//     let e000 = world.create_entity::<Node>();
//     let e001 = world.create_entity::<Node>();
//     let e002 = world.create_entity::<Node>();
//     idtree.create(e000);
//     idtree.insert_child(e000, e00, 1, Some(&notify));
//     opacitys.insert(e000, BoxColor::default());
//     idtree.create(e001);
//     idtree.insert_child(e001, e00, 2, Some(&notify));
//     opacitys.insert(e001, BoxColor::default());
//     idtree.create(e002);
//     idtree.insert_child(e002, e00, 3, Some(&notify));
//     opacitys.insert(e002, BoxColor::default());

//     let e010 = world.create_entity::<Node>();
//     let e011 = world.create_entity::<Node>();
//     let e012 = world.create_entity::<Node>();
//     idtree.create(e010);
//     idtree.insert_child(e010, e01, 1, Some(&notify));
//     opacitys.insert(e010, BoxColor::default());
//     idtree.create(e011);
//     idtree.insert_child(e011, e01, 2, Some(&notify));
//     opacitys.insert(e011, BoxColor::default());
//     idtree.create(e012);
//     idtree.insert_child(e012, e01, 3, Some(&notify));
//     opacitys.insert(e012, BoxColor::default());
//     world.run(&Atom::from("test_opacity_sys"));

//     unsafe { opacitys.get_unchecked_write(e0)}.set_0(0.5);
//     unsafe { opacitys.get_unchecked_write(e00)}.set_0(0.5);

//     world.run(&Atom::from("test_opacity_sys"));

//     println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
//         unsafe{copacitys.get_unchecked(e0)},
//         unsafe{copacitys.get_unchecked(e00)},
//         unsafe{copacitys.get_unchecked(e01)},
//         unsafe{copacitys.get_unchecked(e02)},
//         unsafe{copacitys.get_unchecked(e000)},
//         unsafe{copacitys.get_unchecked(e001)},
//         unsafe{copacitys.get_unchecked(e002)},
//         unsafe{copacitys.get_unchecked(e010)},
//         unsafe{copacitys.get_unchecked(e011)},
//         unsafe{copacitys.get_unchecked(e012)},
//     );
// }

// #[cfg(test)]
// fn new_world() -> World {
//     let mut world = World::default();

//     world.register_entity::<Node>();
//     world.register_multi::<Node, BoxColor>();
//     world.register_multi::<Node, CBoxColor>();
//     world.register_single::<IdTree>(IdTree::default());
     
//     let system = CellBoxUboSys::new(BoxUboSys::default());
//     world.register_system(Atom::from("system"), system);

//     let mut dispatch = SeqDispatcher::default();
//     dispatch.build("system".to_string(), &world);

//     world.add_dispatcher( Atom::from("test_opacity_sys"), dispatch);
//     world
// }

