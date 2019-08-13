/**
 * 监听transform和layout组件， 利用transform和layout递归计算节点的世界矩阵（worldmatrix组件）
 */

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
use ecs::idtree::{ IdTree};
use dirty::LayerDirty;

use component::user::{ Transform };
use component::calc::{ WorldMatrix, WorldMatrixWrite };
use single::DefaultTable;
use system::util::get_or_default;
use map::vecmap::{VecMap};

use component::user::*;
use entity::{Node};

#[derive(Default)]
pub struct WorldMatrixSys{
    dirty_mark_list: VecMap<bool>,
    dirty: LayerDirty,
}

impl WorldMatrixSys{
    fn marked_dirty(&mut self, id: usize, id_tree: &SingleCaseImpl<IdTree>){
        match id_tree.get(id) {
            Some(r) => {
                *unsafe {self.dirty_mark_list.get_unchecked_mut(id)} = true;
                self.dirty.mark(id, r.layer)
            },
            _ => ()
        };
    }

    // fn recursive_delete_dirty(&mut self, id: usize, node: &IdTreeNode, id_tree: &SingleCaseImpl<IdTree>){
    //     unsafe { *self.dirty_mark_list.get_unchecked_mut(id) = false };
    //     // if *unsafe {self.dirty_mark_list.get_unchecked(id)} {
    //     //     self.dirty.delete(id, node.layer)
    //     // }

    //     let first = unsafe { id_tree.get_unchecked(id).children.head };
    //     for child in id_tree.iter(first) {
    //         self.recursive_delete_dirty(child.0, &child.1, id_tree);
    //     }
    // }

    fn cal_matrix(
        &mut self,
        idtree: &SingleCaseImpl<IdTree>,
        transform: &MultiCaseImpl<Node, Transform>,
        layout: &MultiCaseImpl<Node, Layout>,
        world_matrix: &mut MultiCaseImpl<Node, WorldMatrix>,
        default_table: &SingleCaseImpl<DefaultTable>,
    ){  
        let mut count = 0;
        // let time = std::time::Instant::now();
        for id in self.dirty.iter() {
            {
                let dirty_mark = match self.dirty_mark_list.get_mut(*id) {
                    Some(r) => r,
                    None => continue, //panic!("dirty_mark_list err: {}", *id),
                };
                if  *dirty_mark == false {
                    continue;
                }
                *dirty_mark = false;
            }

            let parent_id = match idtree.get(*id) {
                Some(r) => r.parent,
                None => continue, //panic!("cal_matrix error, idtree is not exist, id: {}", *id),
            };
            // let parent_id = unsafe { idtree.get_unchecked(*id).parent };
            let transform_value = get_or_default(parent_id, transform, default_table);
            recursive_cal_matrix(&mut self.dirty_mark_list, parent_id, *id, transform_value, idtree, transform, layout, world_matrix, default_table.get_unchecked(), &mut count);
        }
        self.dirty.clear();
    }
}

impl<'a> Runner<'a> for WorldMatrixSys{
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Transform>, &'a MultiCaseImpl<Node, Layout>, &'a SingleCaseImpl<DefaultTable>);
    type WriteData = &'a mut MultiCaseImpl<Node, WorldMatrix>;
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        self.cal_matrix(read.0, read.1, read.2, write, read.3);
    }
}

impl<'a> EntityListener<'a, Node, CreateEvent> for WorldMatrixSys{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, Transform>, &'a mut MultiCaseImpl<Node, WorldMatrix>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        write.1.insert(event.id, WorldMatrix::default());
        self.dirty_mark_list.insert(event.id, false);
    }
}

impl<'a> EntityListener<'a, Node, DeleteEvent> for WorldMatrixSys{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, Transform>, &'a mut MultiCaseImpl<Node, Layout>);
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, _write: Self::WriteData){
        unsafe { self.dirty_mark_list.remove_unchecked(event.id) };
    }
}

impl<'a> MultiCaseListener<'a, Node, Transform, ModifyEvent> for WorldMatrixSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData){
        self.marked_dirty(event.id, read);
    }
}

impl<'a> MultiCaseListener<'a, Node, Layout, ModifyEvent> for WorldMatrixSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData){
        self.marked_dirty(event.id, read);
    }
}

// impl<'a> MultiCaseListener<'a, Node, ZDepth, ModifyEvent> for WorldMatrixSys{
//     type ReadData = &'a SingleCaseImpl<IdTree>;
//     type WriteData = ();
//     fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData){
//         self.marked_dirty(event.id, read);
//     }
// }

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for WorldMatrixSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, _write: Self::WriteData){
        self.marked_dirty(event.id, read);
    }
}

// impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for WorldMatrixSys{
//     type ReadData = &'a SingleCaseImpl<IdTree>;
//     type WriteData = ();
//     fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, _write: Self::WriteData){
//         let node = unsafe { read.get_unchecked(event.id) };
//         self.recursive_delete_dirty(event.id, &node, read);
//     }
// }

//取lefttop相对于父节点的变换原点的位置
#[inline]
fn get_lefttop_offset(layout: &Layout, parent_origin: &Point2, _parent_layout: &Layout) -> Point2{
    Point2::new(
        // layout.left - parent_origin.x + parent_layout.border_left + parent_layout.padding_left,
        // layout.top - parent_origin.y + parent_layout.border_top + parent_layout.padding_top
        // 当设置宽高为auto时 可能存在bug
        layout.left - parent_origin.x,
        layout.top - parent_origin.y
    )  
}

fn recursive_cal_matrix(
    dirty_mark_list: &mut VecMap<bool>,
    parent: usize,
    id: usize,
    parent_transform: &Transform,
    idtree: &SingleCaseImpl<IdTree>,
    transform: &MultiCaseImpl<Node, Transform>,
    layout: &MultiCaseImpl<Node, Layout>,
    world_matrix: &mut MultiCaseImpl<Node, WorldMatrix>,
    default_transform: &Transform,
    count: &mut usize,
){
    *count = 1 + *count;
    // match dirty_mark_list.get_mut(id) {
    //     Some(r) => *r = false,
    //     None => panic!("dirty_mark_list is no exist, id: {}", id),
    // }
    unsafe{*dirty_mark_list.get_unchecked_mut(id) = false};

    let layout_value = unsafe { layout.get_unchecked(id) };
    // let layout_value = match layout.get(id) {
    //     Some(r) => r,
    //     None => panic!("layout is no exist, id: {}", id)
    // };
    let transform_value = match transform.get(id) {
        Some(r) => r,
        None => default_transform,
    };

    let matrix = if parent == 0 {
        transform_value.matrix(layout_value.width, layout_value.height, &Point2::new(layout_value.left, layout_value.top))
    }else {
        let parent_layout = unsafe { layout.get_unchecked(parent) };
        let parent_world_matrix = unsafe { world_matrix.get_unchecked(parent) };
        let parent_transform_origin = parent_transform.origin.to_value(parent_layout.width, parent_layout.height);
        let offset = get_lefttop_offset(&layout_value, &parent_transform_origin, &parent_layout);
        parent_world_matrix * transform_value.matrix(layout_value.width, layout_value.height, &offset)
    };
    unsafe { world_matrix.get_unchecked_write(id).modify(|w: &mut WorldMatrix| {
        *w = matrix;
        true
    })};

    let first = unsafe { idtree.get_unchecked(id).children.head };
    for child_id in idtree.iter(first) {
        recursive_cal_matrix(dirty_mark_list, id, child_id.0, transform_value, idtree, transform, layout, world_matrix, default_transform, count);
    }
}

impl_system!{
    WorldMatrixSys,
    true,
    {
        EntityListener<Node, CreateEvent>
        EntityListener<Node, DeleteEvent>
        MultiCaseListener<Node, Transform, ModifyEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        SingleCaseListener<IdTree, CreateEvent>
        // SingleCaseListener<IdTree, DeleteEvent>
    }
}


#[cfg(test)]
use ecs::{World, LendMut, SeqDispatcher, Dispatcher};
#[cfg(test)]
use atom::Atom;
#[cfg(test)]
use component::user::{TransformWrite, TransformFunc};
#[cfg(test)]
use component::calc::ZDepth;


#[test]
fn test(){
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
    layouts.insert(e0, Layout{
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
    });

    world.run(&Atom::from("test_transform_sys"));
    
    let e00 = world.create_entity::<Node>();
    let e01 = world.create_entity::<Node>();
    let e02 = world.create_entity::<Node>();
    idtree.create(e00);
    idtree.insert_child(e00, e0, 1, Some(&notify));
    transforms.insert(e00, Transform::default());
    zdepths.insert(e00, ZDepth::default());
    layouts.insert(e00, Layout{
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
    });
    idtree.create(e01);
    idtree.insert_child(e01, e0, 2, Some(&notify));
    layouts.insert(e01, Layout{
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
    });
    transforms.insert(e01, Transform::default());
    zdepths.insert(e01, ZDepth::default());
    idtree.create(e02);
    idtree.insert_child(e02, e0, 3, Some(&notify));
    transforms.insert(e02, Transform::default());
    zdepths.insert(e02, ZDepth::default());
    layouts.insert(e02, Layout{
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
    });

    let e000 = world.create_entity::<Node>();
    let e001 = world.create_entity::<Node>();
    let e002 = world.create_entity::<Node>();
    idtree.create(e000);
    idtree.insert_child(e000, e00, 1, Some(&notify));
    layouts.insert(e000, Layout{
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
    });
    transforms.insert(e000, Transform::default());
    zdepths.insert(e000, ZDepth::default());
    idtree.create(e001);
    idtree.insert_child(e001, e00, 2, Some(&notify));
    transforms.insert(e001, Transform::default());
    zdepths.insert(e001, ZDepth::default());
    layouts.insert(e001, Layout{
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
    });
    idtree.create(e002);
    idtree.insert_child(e002, e00, 3, Some(&notify));
    transforms.insert(e002, Transform::default());
    zdepths.insert(e002, ZDepth::default());
    layouts.insert(e002, Layout{
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
    });

    let e010 = world.create_entity::<Node>();
    let e011 = world.create_entity::<Node>();
    let e012 = world.create_entity::<Node>();
    idtree.create(e010);
    idtree.insert_child(e010, e01, 1, Some(&notify));
    layouts.insert(e010, Layout{
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
    });
    transforms.insert(e010, Transform::default());
    zdepths.insert(e010, ZDepth::default());
    idtree.create(e011);
    idtree.insert_child(e011, e01, 2, Some(&notify));
    transforms.insert(e011, Transform::default());
    zdepths.insert(e011, ZDepth::default());
    layouts.insert(e011, Layout{
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
    });
    idtree.create(e012);
    idtree.insert_child(e012, e01, 3, Some(&notify));
    transforms.insert(e012, Transform::default());
    zdepths.insert(e012, ZDepth::default());
    layouts.insert(e012, Layout{
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
    });
    world.run(&Atom::from("test_world_matrix_sys"));

    unsafe { transforms.get_unchecked_write(e0)}.modify(|transform: &mut Transform|{
        transform.funcs.push(TransformFunc::TranslateX(50.0));
        true
    });

    world.run(&Atom::from("test_transform_sys"));

    debug_println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
        unsafe{world_matrixs.get_unchecked(e0)},
        unsafe{world_matrixs.get_unchecked(e00)},
        unsafe{world_matrixs.get_unchecked(e01)},
        unsafe{world_matrixs.get_unchecked(e02)},
        unsafe{world_matrixs.get_unchecked(e000)},
        unsafe{world_matrixs.get_unchecked(e001)},
        unsafe{world_matrixs.get_unchecked(e002)},
        unsafe{world_matrixs.get_unchecked(e010)},
        unsafe{world_matrixs.get_unchecked(e011)},
        unsafe{world_matrixs.get_unchecked(e012)},
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

    world.add_dispatcher( Atom::from("test_world_matrix_sys"), dispatch);
    world
}