/**
 * 监听transform和layout组件， 利用transform和layout递归计算节点的世界矩阵（worldmatrix组件）
 */

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
use ecs::idtree::{ IdTree, Node as IdTreeNode};
use dirty::LayerDirty;

use component::user::{ Filter };
use component::calc::{ HSV, HSVWrite };
use map::vecmap::{VecMap};

use component::user::*;
use entity::{Node};

#[derive(Default)]
pub struct FilterSys;


impl<'a> EntityListener<'a, Node, CreateEvent> for FilterSys{
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, HSV>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, hsvs: Self::WriteData){
        hsvs.insert(event.id, HSV::default());
    }
}

// impl<'a> MultiCaseListener<'a, Node, Filter, CreateEvent> for FilterSys{
//     type ReadData = (&'a SingleCaseImpl<IdTree>);
//     type WriteData = ();
//     fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, _write: Self::WriteData){
//         self.marked_dirty(event.id, read);
//     }
// }

// impl<'a> MultiCaseListener<'a, Node, Layout, ModifyEvent> for FilterSys{
//     type ReadData = &'a SingleCaseImpl<IdTree>;
//     type WriteData = ();
//     fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData){
//         self.marked_dirty(event.id, read);
//     }
// }

// impl<'a> MultiCaseListener<'a, Node, ZDepth, ModifyEvent> for FilterSys{
//     type ReadData = &'a SingleCaseImpl<IdTree>;
//     type WriteData = ();
//     fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData){
//         self.marked_dirty(event.id, read);
//     }
// }

#[inline]
fn cal_hsv(
    id: usize,
    idtree: &SingleCaseImpl<IdTree>,
    filters: &MultiCaseImpl<Node, Filter>,
    hsvs: &mut MultiCaseImpl<Node, HSV>,
){
    let node = unsafe { idtree.get_unchecked(id)};
    let hsv = match hsvs.get(node.parent){
        Some(hsv) => hsv.clone(),
        None => HSV::default(),
    };

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
                s: cal_s_from_grayscale(filter.gray_scale + parent_hsv.s),
                v: cal_v_from_brightness(filter.bright_ness + parent_hsv.v),
            };
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

fn cal_h_from_hue(hue_rotate: f32) -> f32{
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

fn cal_s_from_grayscale(grayscale: f32) -> f32{
    if grayscale > 1.0 {
        1.0
    }else if grayscale < 0.0{
        0.0
    } else {
        grayscale
    }
}

fn cal_v_from_brightness(brightness: f32) -> f32{
    if brightness < 0.0 {
        0.0
    }else {
        brightness
    }
}

// impl<'a> EntityListener<'a, Node, CreateEvent> for FilterSys{
//     type ReadData = ();
//     type WriteData = (&'a mut MultiCaseImpl<Node, Transform>, &'a mut MultiCaseImpl<Node, WorldMatrix>);
//     fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
//         write.0.insert(event.id, Transform::default());
//         write.1.insert(event.id, WorldMatrix::default());
//         self.dirty_mark_list.insert(event.id, false);
//     }
// }

// impl<'a> EntityListener<'a, Node, DeleteEvent> for FilterSys{
//     type ReadData = ();
//     type WriteData = (&'a mut MultiCaseImpl<Node, Transform>, &'a mut MultiCaseImpl<Node, Layout>);
//     fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, _write: Self::WriteData){
//         unsafe { self.dirty_mark_list.remove_unchecked(event.id) };
//     }
// }

// impl<'a> MultiCaseListener<'a, Node, Transform, ModifyEvent> for FilterSys{
//     type ReadData = &'a SingleCaseImpl<IdTree>;
//     type WriteData = ();
//     fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData){
//         self.marked_dirty(event.id, read);
//     }
// }

// impl<'a> MultiCaseListener<'a, Node, Layout, ModifyEvent> for FilterSys{
//     type ReadData = &'a SingleCaseImpl<IdTree>;
//     type WriteData = ();
//     fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData){
//         self.marked_dirty(event.id, read);
//     }
// }

// impl<'a> MultiCaseListener<'a, Node, ZDepth, ModifyEvent> for FilterSys{
//     type ReadData = &'a SingleCaseImpl<IdTree>;
//     type WriteData = ();
//     fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData){
//         self.marked_dirty(event.id, read);
//     }
// }

// impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for FilterSys{
//     type ReadData = &'a SingleCaseImpl<IdTree>;
//     type WriteData = ();
//     fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, _write: Self::WriteData){
//         self.marked_dirty(event.id, read);
//     }
// }

// impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for FilterSys{
//     type ReadData = &'a SingleCaseImpl<IdTree>;
//     type WriteData = ();
//     fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, _write: Self::WriteData){
//         let node = unsafe { read.get_unchecked(event.id) };
//         self.recursive_delete_dirty(event.id, &node, read);
//     }
// }


impl_system!{
    FilterSys,
    true,
    {
        EntityListener<Node, CreateEvent>
        // EntityListener<Node, DeleteEvent>
        // MultiCaseListener<Node, Transform, ModifyEvent>
        // MultiCaseListener<Node, Layout, ModifyEvent>
        // MultiCaseListener<Node, ZDepth, ModifyEvent>
        // SingleCaseListener<IdTree, CreateEvent>
        // SingleCaseListener<IdTree, DeleteEvent>
    }
}


// #[cfg(test)]
// use ecs::{World, LendMut, SeqDispatcher, Dispatcher};
// #[cfg(test)]
// use atom::Atom;
// #[cfg(test)]
// use component::user::{TransformWrite, TransformFunc};

// #[test]
// fn test(){
//     let world = new_world();

//     let idtree = world.fetch_single::<IdTree>().unwrap();
//     let idtree = LendMut::lend_mut(&idtree);
//     let notify = idtree.get_notify();
//     let transforms = world.fetch_multi::<Node, Transform>().unwrap();
//     let transforms = LendMut::lend_mut(&transforms);
//     let layouts = world.fetch_multi::<Node, Layout>().unwrap();
//     let layouts = LendMut::lend_mut(&layouts);
//     let world_matrixs = world.fetch_multi::<Node, WorldMatrix>().unwrap();
//     let world_matrixs = LendMut::lend_mut(&world_matrixs);
//     let zdepths = world.fetch_multi::<Node, ZDepth>().unwrap();
//     let zdepths = LendMut::lend_mut(&zdepths);

//     let e0 = world.create_entity::<Node>();
    
//     idtree.create(e0);
//     idtree.insert_child(e0, 0, 0, Some(&notify)); //根
//     transforms.insert(e0, Transform::default());
//     zdepths.insert(e0, ZDepth::default());
//     layouts.insert(e0, Layout{
//         left: 0.0,
//         top: 0.0,
//         width: 900.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });

//     world.run(&Atom::from("test_transform_sys"));
    
//     let e00 = world.create_entity::<Node>();
//     let e01 = world.create_entity::<Node>();
//     let e02 = world.create_entity::<Node>();
//     idtree.create(e00);
//     idtree.insert_child(e00, e0, 1, Some(&notify));
//     transforms.insert(e00, Transform::default());
//     zdepths.insert(e00, ZDepth::default());
//     layouts.insert(e00, Layout{
//         left: 0.0,
//         top: 0.0,
//         width: 300.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     idtree.create(e01);
//     idtree.insert_child(e01, e0, 2, Some(&notify));
//     layouts.insert(e01, Layout{
//         left: 300.0,
//         top: 0.0,
//         width: 300.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     transforms.insert(e01, Transform::default());
//     zdepths.insert(e01, ZDepth::default());
//     idtree.create(e02);
//     idtree.insert_child(e02, e0, 3, Some(&notify));
//     transforms.insert(e02, Transform::default());
//     zdepths.insert(e02, ZDepth::default());
//     layouts.insert(e02, Layout{
//         left: 600.0,
//         top: 0.0,
//         width: 300.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });

//     let e000 = world.create_entity::<Node>();
//     let e001 = world.create_entity::<Node>();
//     let e002 = world.create_entity::<Node>();
//     idtree.create(e000);
//     idtree.insert_child(e000, e00, 1, Some(&notify));
//     layouts.insert(e000, Layout{
//         left: 0.0,
//         top: 0.0,
//         width: 100.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     transforms.insert(e000, Transform::default());
//     zdepths.insert(e000, ZDepth::default());
//     idtree.create(e001);
//     idtree.insert_child(e001, e00, 2, Some(&notify));
//     transforms.insert(e001, Transform::default());
//     zdepths.insert(e001, ZDepth::default());
//     layouts.insert(e001, Layout{
//         left: 100.0,
//         top: 0.0,
//         width: 100.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     idtree.create(e002);
//     idtree.insert_child(e002, e00, 3, Some(&notify));
//     transforms.insert(e002, Transform::default());
//     zdepths.insert(e002, ZDepth::default());
//     layouts.insert(e002, Layout{
//         left: 200.0,
//         top: 0.0,
//         width: 100.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });

//     let e010 = world.create_entity::<Node>();
//     let e011 = world.create_entity::<Node>();
//     let e012 = world.create_entity::<Node>();
//     idtree.create(e010);
//     idtree.insert_child(e010, e01, 1, Some(&notify));
//     layouts.insert(e010, Layout{
//         left: 0.0,
//         top: 0.0,
//         width: 100.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     transforms.insert(e010, Transform::default());
//     zdepths.insert(e010, ZDepth::default());
//     idtree.create(e011);
//     idtree.insert_child(e011, e01, 2, Some(&notify));
//     transforms.insert(e011, Transform::default());
//     zdepths.insert(e011, ZDepth::default());
//     layouts.insert(e011, Layout{
//         left: 100.0,
//         top: 0.0,
//         width: 100.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     idtree.create(e012);
//     idtree.insert_child(e012, e01, 3, Some(&notify));
//     transforms.insert(e012, Transform::default());
//     zdepths.insert(e012, ZDepth::default());
//     layouts.insert(e012, Layout{
//         left: 200.0,
//         top: 0.0,
//         width: 100.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     world.run(&Atom::from("test_world_matrix_sys"));

//     unsafe { transforms.get_unchecked_write(e0)}.modify(|transform: &mut Transform|{
//         transform.funcs.push(TransformFunc::TranslateX(50.0));
//         true
//     });

//     world.run(&Atom::from("test_transform_sys"));

//     debug_println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
//         unsafe{world_matrixs.get_unchecked(e0)},
//         unsafe{world_matrixs.get_unchecked(e00)},
//         unsafe{world_matrixs.get_unchecked(e01)},
//         unsafe{world_matrixs.get_unchecked(e02)},
//         unsafe{world_matrixs.get_unchecked(e000)},
//         unsafe{world_matrixs.get_unchecked(e001)},
//         unsafe{world_matrixs.get_unchecked(e002)},
//         unsafe{world_matrixs.get_unchecked(e010)},
//         unsafe{world_matrixs.get_unchecked(e011)},
//         unsafe{world_matrixs.get_unchecked(e012)},
//     );
// }

// #[cfg(test)]
// fn new_world() -> World {
//     let mut world = World::default();

//     world.register_entity::<Node>();
//     world.register_multi::<Node, Transform>();
//     world.register_multi::<Node, Layout>();
//     world.register_multi::<Node, WorldMatrix>();
//     world.register_multi::<Node, ZDepth>();
//     world.register_single::<IdTree>(IdTree::default());
     
//     let system = CellFilterSys::new(FilterSys::default());
//     world.register_system(Atom::from("system"), system);

//     let mut dispatch = SeqDispatcher::default();
//     dispatch.build("system".to_string(), &world);

//     world.add_dispatcher( Atom::from("test_world_matrix_sys"), dispatch);
//     world
// }