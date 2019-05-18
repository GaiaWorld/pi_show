//八叉树系统

use collision::Bound;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
use ecs::idtree::{ IdTree};

use component::calc::{WorldMatrix};
use component::user::{Transform};
use component::{Aabb3, Vector4, Point3, Matrix4, Point2};
use single::oct::Oct;
use layout::Layout;
use entity::{Node};

#[derive(Default)]
pub struct OctSys;

impl<'a> Runner<'a> for OctSys{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<Oct>;
    fn run(&mut self, _read: Self::ReadData, _write: Self::WriteData){
        // write.collect(); TODO
    }
}

impl OctSys {
    fn modify_oct(
        id: usize,
        idtree: &SingleCaseImpl<IdTree>,
        world_matrixs: &MultiCaseImpl<Node, WorldMatrix>,
        layouts: &mut MultiCaseImpl<Node, Layout>,
        transforms: &mut MultiCaseImpl<Node, Transform>,
        octree:  &mut SingleCaseImpl<Oct>,
    ){
        if idtree.get(id).is_none() {
            return;
        };

        let world_matrix = unsafe {world_matrixs.get_unchecked(id)};
        let layout = unsafe {layouts.get_unchecked(id)};
        let transform = unsafe {transforms.get_unchecked(id)};

        let origin = transform.origin.to_value(layout.width, layout.height);
        let aabb = cal_bound_box((layout.width, layout.height), world_matrix, &origin);
        
        let notify = octree.get_notify();
        octree.update(id, aabb, Some(notify));
    }
}

impl<'a> EntityListener<'a, Node, CreateEvent> for OctSys{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<Oct>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        let notify = write.get_notify();
        write.add(event.id, Aabb3::empty(), 0, Some(notify));
    }
}

impl<'a> EntityListener<'a, Node, DeleteEvent> for OctSys{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<Oct>;
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, write: Self::WriteData){
        let notify = write.get_notify();
        write.remove(event.id, Some(notify));
    }
}

impl<'a> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for OctSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (&'a mut MultiCaseImpl<Node, WorldMatrix>, &'a mut MultiCaseImpl<Node, Layout>, &'a mut MultiCaseImpl<Node, Transform>, &'a mut SingleCaseImpl<Oct>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        OctSys::modify_oct(event.id, read, write.0, write.1, write.2, write.3);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for OctSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (&'a mut MultiCaseImpl<Node, WorldMatrix>, &'a mut MultiCaseImpl<Node, Layout>, &'a mut MultiCaseImpl<Node, Transform>, &'a mut SingleCaseImpl<Oct>);
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        OctSys::modify_oct(event.id, read, write.0, write.1, write.2, write.3);
    }
}

impl_system!{
    OctSys,
    true,
    {
        EntityListener<Node, CreateEvent>
        EntityListener<Node, DeleteEvent>
        MultiCaseListener<Node, WorldMatrix, ModifyEvent>
        SingleCaseListener<IdTree, CreateEvent>
    }
}

fn cal_bound_box(size: (f32, f32), matrix: &Matrix4, origin: &Point2) -> Aabb3{
    let start = (- origin.x, - origin.y);
    let left_top = matrix * Vector4::new(start.0, start.1, 0.0, 1.0);
    let right_top = matrix * Vector4::new(start.0 + size.0, start.1, 0.0, 1.0);
    let left_bottom = matrix * Vector4::new(start.0, start.1 + size.1, 0.0, 1.0);
    let right_bottom = matrix * Vector4::new(start.0 + size.0, start.1 + size.1, 0.0, 1.0);

    let min = Point3::new(
        left_top.x.min(right_top.x).min(left_bottom.x).min(right_bottom.x),
        left_top.y.min(right_top.y).min(left_bottom.y).min(right_bottom.y),
        0.0,
    );

    let max = Point3::new(
        left_top.x.max(right_top.x).max(left_bottom.x).max(right_bottom.x),
        left_top.y.max(right_top.y).max(left_bottom.y).max(right_bottom.y),
        1.0,
    );
    
    Aabb3::new(min, max)
}


#[cfg(test)]
use ecs::{World, BorrowMut, SeqDispatcher, Dispatcher};
#[cfg(test)]
use atom::Atom;
#[cfg(test)]
use component::user::{TransformWrite, TransformFunc};
#[cfg(test)]
use component::calc::{ZDepth};
#[cfg(test)]
use system::world_matrix::{WorldMatrixSys, CellWorldMatrixSys};

#[test]
fn test(){
    let world = new_world();

    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = BorrowMut::borrow_mut(&idtree);
    let oct = world.fetch_single::<Oct>().unwrap();
    let oct = BorrowMut::borrow_mut(&oct);
    let notify = idtree.get_notify();
    let transforms = world.fetch_multi::<Node, Transform>().unwrap();
    let transforms = BorrowMut::borrow_mut(&transforms);
    let layouts = world.fetch_multi::<Node, Layout>().unwrap();
    let layouts = BorrowMut::borrow_mut(&layouts);
    let world_matrixs = world.fetch_multi::<Node, WorldMatrix>().unwrap();
    let world_matrixs = BorrowMut::borrow_mut(&world_matrixs);
    let zdepths = world.fetch_multi::<Node, ZDepth>().unwrap();
    let zdepths = BorrowMut::borrow_mut(&zdepths);

    let e0 = world.create_entity::<Node>();
    
    idtree.create(e0);
    transforms.insert(e0, Transform::default());
    zdepths.insert(e0, ZDepth::default());
    layouts.insert(e0, Layout{
        left: 0.0,
        top: 0.0,
        width: 900.0,
        height: 900.0,
        border: 0.0,
        padding_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
    });
    idtree.insert_child(e0, 0, 0, Some(&notify)); //根
    
    let e00 = world.create_entity::<Node>();
    let e01 = world.create_entity::<Node>();
    let e02 = world.create_entity::<Node>();
    idtree.create(e00);
    transforms.insert(e00, Transform::default());
    zdepths.insert(e00, ZDepth::default());
    layouts.insert(e00, Layout{
        left: 0.0,
        top: 0.0,
        width: 300.0,
        height: 900.0,
        border: 0.0,
        padding_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
    });
    idtree.insert_child(e00, e0, 1, Some(&notify));

    idtree.create(e01);
    layouts.insert(e01, Layout{
        left: 300.0,
        top: 0.0,
        width: 300.0,
        height: 900.0,
        border: 0.0,
        padding_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
    });
    transforms.insert(e01, Transform::default());
    zdepths.insert(e01, ZDepth::default());
    idtree.insert_child(e01, e0, 2, Some(&notify));

    idtree.create(e02);
    transforms.insert(e02, Transform::default());
    zdepths.insert(e02, ZDepth::default());
    layouts.insert(e02, Layout{
        left: 600.0,
        top: 0.0,
        width: 300.0,
        height: 900.0,
        border: 0.0,
        padding_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
    });
    idtree.insert_child(e02, e0, 3, Some(&notify));

    let e000 = world.create_entity::<Node>();
    let e001 = world.create_entity::<Node>();
    let e002 = world.create_entity::<Node>();
    idtree.create(e000);
    layouts.insert(e000, Layout{
        left: 0.0,
        top: 0.0,
        width: 100.0,
        height: 900.0,
        border: 0.0,
        padding_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
    });
    transforms.insert(e000, Transform::default());
    zdepths.insert(e000, ZDepth::default());
    idtree.insert_child(e000, e00, 1, Some(&notify));

    idtree.create(e001);
    transforms.insert(e001, Transform::default());
    zdepths.insert(e001, ZDepth::default());
    layouts.insert(e001, Layout{
        left: 100.0,
        top: 0.0,
        width: 100.0,
        height: 900.0,
        border: 0.0,
        padding_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
    });
    idtree.insert_child(e001, e00, 2, Some(&notify));

    idtree.create(e002);
    transforms.insert(e002, Transform::default());
    zdepths.insert(e002, ZDepth::default());
    layouts.insert(e002, Layout{
        left: 200.0,
        top: 0.0,
        width: 100.0,
        height: 900.0,
        border: 0.0,
        padding_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
    });
    idtree.insert_child(e002, e00, 3, Some(&notify));

    let e010 = world.create_entity::<Node>();
    let e011 = world.create_entity::<Node>();
    let e012 = world.create_entity::<Node>();
    idtree.create(e010);
    layouts.insert(e010, Layout{
        left: 0.0,
        top: 0.0,
        width: 100.0,
        height: 900.0,
        border: 0.0,
        padding_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
    });
    transforms.insert(e010, Transform::default());
    zdepths.insert(e010, ZDepth::default());
    idtree.insert_child(e010, e01, 1, Some(&notify));

    idtree.create(e011);
    transforms.insert(e011, Transform::default());
    zdepths.insert(e011, ZDepth::default());
    layouts.insert(e011, Layout{
        left: 100.0,
        top: 0.0,
        width: 100.0,
        height: 900.0,
        border: 0.0,
        padding_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
    });
    idtree.insert_child(e011, e01, 2, Some(&notify));

    idtree.create(e012);
    transforms.insert(e012, Transform::default());
    zdepths.insert(e012, ZDepth::default());
    layouts.insert(e012, Layout{
        left: 200.0,
        top: 0.0,
        width: 100.0,
        height: 900.0,
        border: 0.0,
        padding_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
    });
    idtree.insert_child(e012, e01, 3, Some(&notify));
    
    unsafe { transforms.get_unchecked_write(e0)}.modify(|transform: &mut Transform|{
        transform.funcs.push(TransformFunc::TranslateX(50.0));
        true
    });
    world.run(&Atom::from("test_oct_sys"));
    println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
        unsafe{oct.get_unchecked(e0)},
        unsafe{oct.get_unchecked(e00)},
        unsafe{oct.get_unchecked(e01)},
        unsafe{oct.get_unchecked(e02)},
        unsafe{oct.get_unchecked(e000)},
        unsafe{oct.get_unchecked(e001)},
        unsafe{oct.get_unchecked(e002)},
        unsafe{oct.get_unchecked(e010)},
        unsafe{oct.get_unchecked(e011)},
        unsafe{oct.get_unchecked(e012)},
    );
}

#[cfg(test)]
fn new_world() -> World {
    let mut world = World::default();

    world.register_entity::<Node>();
    world.register_multi::<Node, Layout>();
    world.register_multi::<Node, Transform>();
    world.register_multi::<Node, ZDepth>();
    world.register_multi::<Node, WorldMatrix>();
    world.register_single::<IdTree>(IdTree::default());
    world.register_single::<Oct>(Oct::new());
     
    let system = CellOctSys::new(OctSys::default());
    world.register_system(Atom::from("oct_system"), system);
    let system = CellWorldMatrixSys::new(WorldMatrixSys::default());
    world.register_system(Atom::from("world_matrix_system"), system);

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("oct_system, world_matrix_system".to_string(), &world);

    world.add_dispatcher( Atom::from("test_oct_sys"), dispatch);
    world
}

