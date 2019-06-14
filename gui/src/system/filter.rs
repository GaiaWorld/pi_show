/**
 * 监听transform和layout组件， 利用transform和layout递归计算节点的世界矩阵（worldmatrix组件）
 */

use ecs::{CreateEvent, ModifyEvent, MultiCaseListener, EntityListener, SingleCaseImpl, SingleCaseListener, MultiCaseImpl};
use ecs::idtree::{ IdTree};

use component::user::{ Filter };
use component::calc::{ HSV };
use entity::{Node};

#[derive(Default)]
pub struct FilterSys;


impl<'a> EntityListener<'a, Node, CreateEvent> for FilterSys{
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, HSV>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, hsvs: Self::WriteData){
        hsvs.insert(event.id, HSV::default());
    }
}

impl<'a> MultiCaseListener<'a, Node, Filter, CreateEvent> for FilterSys{
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Filter>);
    type WriteData = &'a mut MultiCaseImpl<Node, HSV>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, hsvs: Self::WriteData){
        let (idtree, filters) = read;
        cal_hsv(event.id, idtree, filters, hsvs);
    }
}

impl<'a> MultiCaseListener<'a, Node, Filter, ModifyEvent> for FilterSys{
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Filter>);
    type WriteData = &'a mut MultiCaseImpl<Node, HSV>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, hsvs: Self::WriteData){
        let (idtree, filters) = read;
        cal_hsv(event.id, idtree, filters, hsvs);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for FilterSys{
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Filter>);
    type WriteData = &'a mut MultiCaseImpl<Node, HSV>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, hsvs: Self::WriteData){
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
){  
    let parent_id = match idtree.get(id) {
        Some(node) => node.parent,
        None => return,
    };
    let hsv = match hsvs.get(parent_id){
        Some(hsv) => hsv.clone(),
        None => HSV::default(),
    };;

    recursive_cal_hsv(id, idtree, &hsv, filters, hsvs)
}

#[inline]
fn recursive_cal_hsv(
    id: usize,
    idtree: &SingleCaseImpl<IdTree>,
    parent_hsv: &HSV,
    filters: &MultiCaseImpl<Node, Filter>,
    hsvs: &mut MultiCaseImpl<Node, HSV>,
){
    let hsv = match filters.get(id){
        Some(filter) => {
            let hsv = HSV {
                h: cal_h_from_hue(filter.hue_rotate + parent_hsv.h),
                s: cal_s_from_grayscale(filter.gray_scale - parent_hsv.s),
                v: cal_v_from_brightness(filter.bright_ness) * parent_hsv.v,
            };
            println!("filter: {:?}, hsv:{:?}, parent_hsv: {:?}", filter, hsv, parent_hsv);
            hsvs.insert(id, hsv.clone());   
            hsv
        },
        None => {hsvs.insert(id, parent_hsv.clone()); parent_hsv.clone()},
    };

    let first = unsafe { idtree.get_unchecked(id).children.head };
    for child_id in idtree.iter(first) {
        recursive_cal_hsv(child_id.0, idtree, &hsv, filters, hsvs);
    }
}

// 计算hue， hue的值在0~360度范围内
fn cal_h_from_hue(mut hue_rotate: f32) -> f32{
    if hue_rotate > 0.0 {
        loop {
            if hue_rotate <= 360.0 {
                return hue_rotate;
            }
            hue_rotate -= 360.0;
        } 
    }else {
        loop {
            if hue_rotate >= 0.0 {
                return hue_rotate;
            }
            hue_rotate += 360.0;
        } 
    }
}

// 计算grayscale， hue的值在0~1度范围内， 大于1.0， 取1.0的值，小于0.0 取0.0
fn cal_s_from_grayscale(grayscale: f32) -> f32{
    if grayscale > 1.0 {
        -1.0
    }else if grayscale < 0.0{
        0.0
    } else {
        -grayscale
    }
}

// 计算明亮度， 值为0-2， 小于0是取0， 如果值是0，会全黑。值是1， 无变化， 大于1会更亮
fn cal_v_from_brightness(brightness: f32) -> f32{
    if brightness < 0.0 {
        0.0
    }else if brightness > 2.0 {
        2.0
    }else {
        brightness
    }
}

impl_system!{
    FilterSys,
    false,
    {
        EntityListener<Node, CreateEvent>
        MultiCaseListener<Node, Filter, CreateEvent>
        MultiCaseListener<Node, Filter, ModifyEvent>
        SingleCaseListener<IdTree, CreateEvent>
    }
}


#[cfg(test)]
use ecs::{World, LendMut, SeqDispatcher, Dispatcher};
#[cfg(test)]
use atom::Atom;

#[test]
fn test(){
    let world = new_world();

    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = LendMut::lend_mut(&idtree);
    let notify = idtree.get_notify();
    let filters = world.fetch_multi::<Node, Filter>().unwrap();
    let filters = LendMut::lend_mut(&filters);
    let hsvs = world.fetch_multi::<Node, HSV>().unwrap();
    let hsvs = LendMut::lend_mut(&hsvs);

    let e0 = world.create_entity::<Node>();
    
    idtree.create(e0);
    idtree.insert_child(e0, 0, 0, Some(&notify)); //根
    let filter = Filter{
        hue_rotate: 380.0,
        bright_ness: 0.5,
        gray_scale: 0.3,
    };
    filters.insert(e0, filter.clone());

    world.run(&Atom::from("test_filter_sys"));
    
    let e00 = world.create_entity::<Node>();
    let e01 = world.create_entity::<Node>();
    let e02 = world.create_entity::<Node>();
    idtree.create(e00);
    idtree.insert_child(e00, e0, 1, Some(&notify));
    filters.insert(e00, filter.clone());
    idtree.create(e01);
    idtree.insert_child(e01, e0, 2, Some(&notify));
    filters.insert(e01, filter.clone());
    idtree.create(e02);
    idtree.insert_child(e02, e0, 3, Some(&notify));
    filters.insert(e02, filter.clone());

    let e000 = world.create_entity::<Node>();
    let e001 = world.create_entity::<Node>();
    let e002 = world.create_entity::<Node>();
    idtree.create(e000);
    idtree.insert_child(e000, e00, 1, Some(&notify));
    idtree.create(e001);
    idtree.insert_child(e001, e00, 2, Some(&notify));
    idtree.create(e002);
    idtree.insert_child(e002, e00, 3, Some(&notify));

    let e010 = world.create_entity::<Node>();
    let e011 = world.create_entity::<Node>();
    let e012 = world.create_entity::<Node>();
    idtree.create(e010);
    idtree.insert_child(e010, e01, 1, Some(&notify));
    filters.insert(e010, Filter::default());
    idtree.create(e011);
    idtree.insert_child(e011, e01, 2, Some(&notify));
    filters.insert(e011, Filter::default());
    idtree.create(e012);
    idtree.insert_child(e012, e01, 3, Some(&notify));
    filters.insert(e012, Filter::default());
    world.run(&Atom::from("test_opacity_sys"));

    // unsafe { filters.get_unchecked_write(e0)}.set_0(0.5);
    // unsafe { filters.get_unchecked_write(e00)}.set_0(0.5);

    // world.run(&Atom::from("test_opacity_sys"));

    debug_println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
        unsafe{hsvs.get_unchecked(e0)},
        unsafe{hsvs.get_unchecked(e00)},
        unsafe{hsvs.get_unchecked(e01)},
        unsafe{hsvs.get_unchecked(e02)},
        unsafe{hsvs.get_unchecked(e000)},
        unsafe{hsvs.get_unchecked(e001)},
        unsafe{hsvs.get_unchecked(e002)},
        unsafe{hsvs.get_unchecked(e010)},
        unsafe{hsvs.get_unchecked(e011)},
        unsafe{hsvs.get_unchecked(e012)},
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

    world.add_dispatcher( Atom::from("test_filter_sys"), dispatch);
    world
}

