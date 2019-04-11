use std::rc::Rc;
use std::cell::RefCell;

use cg::{Vector3 as CgVector3};
use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent, ModifyFieldEvent};
use wcs::world::System;
use vecmap::VecMap;

use component::math::{Color as MathColor, Vector2, Aabb3, Matrix4 as MathMatrix4};
use component::color::Color;
use world_doc::component::node::{Node};
use world_doc::component::style::generic::{ Decorate, BoxShadow };
use world_doc::component::style::transform::Transform;
use world_doc::WorldDocMgr;
use world_2d::component::image::Image;
use world_2d::component::sdf::{ Sdf };
use util::dirty_mark::DirtyMark;
// use util::math::decompose;

//背景边框系统
pub struct BBSys(Rc<RefCell<BBSysImpl>>);

impl BBSys {
    pub fn init(component_mgr: &mut WorldDocMgr) -> Rc<BBSys> {
        let r = Rc::new(BBSys(Rc::new(RefCell::new(BBSysImpl::new()))));
        //监听backgroud_image的修改事件， 修改image2d上对应的值
        component_mgr.node.decorate.backgroud_image.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Decorate, ModifyFieldEvent, WorldDocMgr>>)));
        //监听background_color修改事件， 修改sdf2d上对应的值
        component_mgr.node.decorate.background_color._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Color, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node.decorate.background_color._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Color, CreateEvent, WorldDocMgr>>)));
        component_mgr.node.decorate.background_color._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Color, DeleteEvent, WorldDocMgr>>)));
        //监听border_color修改事件， 修改sdf2d上对应的值
        component_mgr.node.decorate.border_color._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<MathColor, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node.decorate.border_color._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<MathColor, CreateEvent, WorldDocMgr>>)));
        component_mgr.node.decorate.border_color._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<MathColor, DeleteEvent, WorldDocMgr>>)));
        //监听box_shadow修改事件， 修改sdf2d上对应的值
        component_mgr.node.decorate.box_shadow._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<BoxShadow, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node.decorate.box_shadow._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<BoxShadow, CreateEvent, WorldDocMgr>>)));
        component_mgr.node.decorate.box_shadow._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<BoxShadow, DeleteEvent, WorldDocMgr>>)));
    
        // 监听node的改变， 修改sdf2d， image2d组件
        component_mgr.node.z_depth.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node.by_overflow.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node.layout.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node.real_opacity.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));

        //监听boundbox的变化
        component_mgr.node.bound_box._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Aabb3, ModifyFieldEvent, WorldDocMgr>>)));

        //监听world_matrix的变化
        component_mgr.node.world_matrix._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<MathMatrix4, ModifyFieldEvent, WorldDocMgr>>)));
        r
    }
}

//监听backgroud_image的修改事件， 修改image2d上对应的值
impl ComponentHandler<Decorate, ModifyFieldEvent, WorldDocMgr> for BBSys {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr) {
        let ModifyFieldEvent { id, parent, field: _ } = event;
        let mut borrow_mut = self.0.borrow_mut();
        match borrow_mut.image_image2d_map.get(*id) {
            Some(image_id) => { //如果image2d已经存在
                let src = component_mgr.node.decorate._group.get(*id).backgroud_image.clone();
                match src { 
                    0 => (),//component_mgr.world_2d.component_mgr.remove_image_mut(*image_id), //并且 src==0, 应该删除image2d
                    _ => component_mgr.world_2d.component_mgr.get_image_mut(*image_id).set_src(src), //并且 src>0, 应该修改image2d
                }
                return;
            },
            None => (),
        }
        //如果image2d不存在
        let src = component_mgr.node.decorate._group.get(*id).backgroud_image;
        match src {
            0 => (), //并且src == 0， 不需要进行任何操作
            _ => { // 并且src>0， 应该创建image2d
                let image = create_image2d(src, *parent);
                let image2d_ref = component_mgr.world_2d.component_mgr.add_image(image);
                borrow_mut.image_image2d_map.insert(*id, image2d_ref.id);
            },
            
        }
    }
}

//image create delete TODO

//监听background_color修改事件， 修改sdf2d上对应的值
impl ComponentHandler<Color, ModifyFieldEvent, WorldDocMgr> for BBSys {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr) {
        let ModifyFieldEvent { id: _, parent, field: _ } = event; 
        let sdf_id = *(unsafe { self.0.borrow_mut().color_sdf2d_map.get_unchecked(*parent) });

        let (color_id, node_id) = {
            let decorate = component_mgr.node.decorate._group.get(*parent);
            (decorate.background_color, decorate.parent)
        };
        let color = component_mgr.node.decorate.background_color._group.get(color_id).owner.clone();

        modify_is_opacity(node_id, sdf_id, *parent, component_mgr);

        component_mgr.world_2d.component_mgr.get_sdf_mut(sdf_id).set_color(color);
    }
}

//监听background_color的创建事件， 修改创建或修改对应sdf2d上对应的值
impl ComponentHandler<Color, CreateEvent, WorldDocMgr> for BBSys {
    fn handle(&self, event: &CreateEvent, component_mgr: &mut WorldDocMgr) {
        let CreateEvent { id, parent } = event; 
        let mut borrow_mut = self.0.borrow_mut();
        //background_color创建时，其对应的sdf2d能已经被创建（border_color对应的sdf2d与background_color对应的sdf2d是同一个）
        let node_id =  component_mgr.node.decorate._group.get(*parent).parent;
        let sdf_id = match borrow_mut.color_sdf2d_map.get(*parent) {
            //如果已经存在sdf2d, 直接修改其color值
            Some(sdf_id) => {
                let color = component_mgr.node.decorate.background_color._group.get(*id).owner.clone();
                component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_color(color);
                *sdf_id
            }, 
            None => {
                let sdf = create_box_sdf2d(component_mgr, node_id);
                let sdf_id = component_mgr.world_2d.component_mgr.add_sdf(sdf).id;
                borrow_mut.color_sdf2d_map.insert(*parent, sdf_id);
                sdf_id
            },
        };
        modify_is_opacity(node_id, sdf_id, *parent, component_mgr);
    }
}

//监听background_color的删除事件， 尝试删除对应的sdf2d组件
impl ComponentHandler<Color, DeleteEvent, WorldDocMgr> for BBSys {
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut WorldDocMgr) {
        let DeleteEvent { id: _, parent } = event; 
        let border_color_id = component_mgr.node.decorate._group.get(*parent).border_color;
        if border_color_id == 0 {
            let _sdf_id = unsafe { self.0.borrow_mut().color_sdf2d_map.get_unchecked(*parent) };
            //删除sdf2d组件 TODO
        }
    }
}


//监听border_color的创建事件， 修改创建或修改对应sdf2d上对应的值
impl ComponentHandler<MathColor, CreateEvent, WorldDocMgr> for BBSys {
    fn handle(&self, event: &CreateEvent, component_mgr: &mut WorldDocMgr) {
        let CreateEvent { id, parent } = event; 
        let mut borrow_mut = self.0.borrow_mut();
        let node_id =  component_mgr.node.decorate._group.get(*parent).parent;
        //border_color创建时，其对应的sdf2d可能已经被创建（border_color对应的sdf2d与background_color对应的sdf2d是同一个）
        let sdf_id = match borrow_mut.color_sdf2d_map.get(*parent) {
            //如果已经存在sdf2d, 直接修改其border_color值
            Some(sdf_id) => {
                let border_color = component_mgr.node.decorate.border_color._group.get(*id).owner.clone();
                component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_border_color(border_color);
                *sdf_id
            },
            //如果不存在sdf2d， 创建一个， 插入decorate_id与sdf_id的索引， 以便通过decorate_id插入、删除、和修改sdf
            None => {
                let sdf = create_box_sdf2d(component_mgr, node_id);
                let sdf_id = component_mgr.world_2d.component_mgr.add_sdf(sdf).id;
                borrow_mut.color_sdf2d_map.insert(*parent, sdf_id);
                sdf_id
            },
        };
        modify_is_opacity(node_id, sdf_id, *parent, component_mgr);
    }
}

//监听border_color的删除事件， 尝试删除对应的sdf2d组件
impl ComponentHandler<MathColor, DeleteEvent, WorldDocMgr> for BBSys {
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut WorldDocMgr) {
        let DeleteEvent { id: _, parent } = event; 
        let border_color_id = component_mgr.node.decorate._group.get(*parent).border_color;
        if border_color_id == 0 {
            let _sdf_id = unsafe { self.0.borrow_mut().color_sdf2d_map.get_unchecked(*parent) };
            //删除sdf2d组件 TODO
        }
    }
}

//监听border_color的删除事件， 尝试删除对应的sdf2d组件
impl ComponentHandler<MathColor, ModifyFieldEvent, WorldDocMgr> for BBSys {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr) {
        let ModifyFieldEvent { id: _, parent, field: _ } = event; 
        let sdf_id = *(unsafe { self.0.borrow_mut().color_sdf2d_map.get_unchecked(*parent) });

        let (border_color_id, node_id) = {
            let decorate = component_mgr.node.decorate._group.get(*parent);
            (decorate.border_color, decorate.parent)
        };
        let color = component_mgr.node.decorate.border_color._group.get(border_color_id).owner.clone();

        modify_is_opacity(node_id, sdf_id, *parent, component_mgr);

        component_mgr.world_2d.component_mgr.get_sdf_mut(sdf_id).set_border_color(color);
    }
}

impl ComponentHandler<BoxShadow, CreateEvent, WorldDocMgr> for BBSys {
    fn handle(&self, event: &CreateEvent, component_mgr: &mut WorldDocMgr) {
        let CreateEvent { id: _, parent } = event; 
        //创建一个， 插入decorate_id与sdf_id的索引， 以便通过decorate_id插入、删除、和修改sdf
        let node_id =  component_mgr.node.decorate._group.get(*parent).parent;
        let sdf = create_shadow_sdf2d(component_mgr, node_id);
        let sdf_id = component_mgr.world_2d.component_mgr.add_sdf(sdf).id;
        let mut borrow_mut = self.0.borrow_mut();
        borrow_mut.shadow_sdf2d_map.insert(*parent, sdf_id);
        borrow_mut.shadow_matrix_dirty.dirty_mark_list.insert(*parent, false);
        borrow_mut.shadow_matrix_dirty.marked_dirty(*parent);
    }
}


impl ComponentHandler<BoxShadow, DeleteEvent, WorldDocMgr> for BBSys {
    fn handle(&self, event: &DeleteEvent, _component_mgr: &mut WorldDocMgr) {
        let DeleteEvent { id: _, parent } = event; 
        let mut borrow_mut = self.0.borrow_mut();
        let _sdf_id = unsafe { borrow_mut.shadow_sdf2d_map.get_unchecked(*parent).clone() };
        borrow_mut.shadow_matrix_dirty.delete_dirty(*parent);
        //删除sdf2d TODO
    }
}

impl ComponentHandler<BoxShadow, ModifyFieldEvent, WorldDocMgr> for BBSys {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr) {
        let ModifyFieldEvent { id, parent, field } = event; 
        let shadow = component_mgr.node.decorate.box_shadow._group.get(*id);
        let sdf_id = unsafe { self.0.borrow_mut().shadow_sdf2d_map.get_unchecked(*parent).clone() };
        if *field == "blur" {
            // component_mgr.world_2d.component_mgr.get_sdf_mut(sdf_id).set_blur(shadow.blur);
        }else if *field == "h" || *field == "v"{
            self.0.borrow_mut().shadow_matrix_dirty.marked_dirty(*parent);
        } else if *field == "color" {
            component_mgr.world_2d.component_mgr.get_sdf_mut(sdf_id).set_color(Color::RGBA(shadow.color.clone()));
        }
        //删除sdf2d TODO
    }
}

impl ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr> for BBSys {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr) {
        let ModifyFieldEvent { id, parent: _, field } = event; 
        let node = component_mgr.node._group.get(*id);
        if node.decorate == 0 {
            return;
        }

        let decorate_id = node.decorate;
        let borrow = self.0.borrow();
        if *field == "z_depth" {
            if let Some(image_id) = borrow.image_image2d_map.get(decorate_id) {
                component_mgr.world_2d.component_mgr.get_image_mut(*image_id).set_z_depth(node.z_depth);
            }
            if let Some(sdf_id) = borrow.shadow_sdf2d_map.get(decorate_id) {
                component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_z_depth(node.z_depth - 0.00001);
            }
            if let Some(sdf_id) = borrow.color_sdf2d_map.get(decorate_id) {
                component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_z_depth(node.z_depth);
            }
        }else if *field == "by_overflow" {
            if let Some(image_id) = borrow.image_image2d_map.get(decorate_id) {
                component_mgr.world_2d.component_mgr.get_image_mut(*image_id).set_by_overflow(node.by_overflow);
            }
            if let Some(sdf_id) = borrow.shadow_sdf2d_map.get(decorate_id) {
                component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_by_overflow(node.by_overflow);
            }
            if let Some(sdf_id) = borrow.color_sdf2d_map.get(decorate_id) {
                component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_by_overflow(node.by_overflow);
            }
        }else if *field == "real_opacity" {
            if let Some(image_id) = borrow.image_image2d_map.get(decorate_id) {
                component_mgr.world_2d.component_mgr.get_image_mut(*image_id).set_alpha(node.real_opacity);
            }
            if let Some(sdf_id) = borrow.shadow_sdf2d_map.get(decorate_id) {
                component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_alpha(node.real_opacity);
            }
            if let Some(sdf_id) = borrow.color_sdf2d_map.get(decorate_id) {
                component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_alpha(node.real_opacity);
                //设置is_opacity
                modify_is_opacity(*id, *sdf_id, decorate_id, component_mgr);
            }
        } else if *field == "layout" {
            let layout = &node.layout;
            if let Some(image_id) = borrow.image_image2d_map.get(decorate_id) {
                component_mgr.world_2d.component_mgr.get_image_mut(*image_id).set_extend(Vector2::new(layout.width/2.0 - layout.border, layout.height/2.0 - layout.border));
            }
            if let Some(sdf_id) = borrow.shadow_sdf2d_map.get(decorate_id) {
                component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_extend(Vector2::new(layout.width/2.0, layout.height/2.0));
            }
            if let Some(sdf_id) = borrow.color_sdf2d_map.get(decorate_id) {
                component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_extend(Vector2::new(layout.width/2.0 - layout.border, layout.height/2.0 - layout.border));
                component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_border_size(layout.border);
            }
        }
    }
}

impl ComponentHandler<Aabb3, ModifyFieldEvent, WorldDocMgr> for BBSys {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr) {
        let ModifyFieldEvent { id, parent, field: _ } = event; 
        let node = component_mgr.node._group.get(*parent);
        if node.decorate == 0 {
            return;
        }

        let decorate_id = node.decorate;
        let borrow = self.0.borrow();
        let bound_box = component_mgr.node.bound_box._group.get(*id).owner.clone();
        // if let Some(image_id) = borrow.image_image2d_map.get(decorate_id) {
        //     component_mgr.world_2d.component_mgr.get_image_mut(*image_id).set_bound_box(bound_box);
        // }
        if let Some(sdf_id) = borrow.shadow_sdf2d_map.get(decorate_id) {
            component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_bound_box(bound_box);
        }
        if let Some(sdf_id) = borrow.color_sdf2d_map.get(decorate_id) {
            component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_bound_box(bound_box);
        }
    }
}

impl ComponentHandler<MathMatrix4, ModifyFieldEvent, WorldDocMgr> for BBSys {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr) {
        let ModifyFieldEvent { id, parent, field: _ } = event; 
        let node = component_mgr.node._group.get(*parent);
        if node.decorate == 0 {
            return;
        }

        let decorate_id = node.decorate;
        let mut borrow_mut = self.0.borrow_mut();
        if let Some(_sdf_id) = borrow_mut.shadow_sdf2d_map.get(decorate_id) {
            borrow_mut.shadow_matrix_dirty.marked_dirty(decorate_id);
        }
        if let Some(sdf_id) = borrow_mut.color_sdf2d_map.get(decorate_id) {
            let world_matrix = component_mgr.node.world_matrix._group.get(*id);
            component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_world_matrix(world_matrix.owner.clone());
        }
    }
}

impl System<(), WorldDocMgr> for BBSys{
    fn run(&self, _e: &(), component_mgr: &mut WorldDocMgr){
        let mut borrow_mut = self.0.borrow_mut();
        for decorate_id in borrow_mut.shadow_matrix_dirty.dirtys.iter() {
            let (node_id, box_shadow_id) = {
                let decorate = component_mgr.node.decorate._group.get(*decorate_id);
                (decorate.parent, decorate.box_shadow)
            };
            
            let world_matrix = cal_shadow_matrix(node_id, box_shadow_id, component_mgr);
            if let Some(sdf_id) = borrow_mut.shadow_sdf2d_map.get(*decorate_id) {
                component_mgr.world_2d.component_mgr.get_sdf_mut(*sdf_id).set_world_matrix(world_matrix.clone());
            }
        }
        borrow_mut.shadow_matrix_dirty.dirtys.clear();
    }
}

// // 监听
// impl ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr> for BBSys {
//     fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr) {
//         let ModifyFieldEvent { id, parent, field } = event;
//         let node =  component_mgr.node._group.get(*id);

//         if node.background == 0 {
//             return;
//         }

//         let background = component_mgr.node.background._group.get(node.background);

//         if *field == "opacity" {
//             if background.color
//             let sdf_id = *(unsafe { self.0.borrow_mut().color_sdf2d_map.get_unchecked(*id) });
//             let color = component_mgr.node.background._group.get(*id).color.clone();
//             component_mgr.world_2d.component_mgr.get_sdf_mut(sdf_id).set_color(color);
//         }else if *field == "image" {
//             match self.0.borrow_mut().image_image2d_map.get(*id) {
//                 Some(image_id) => {
//                     let url = component_mgr.node.background._group.get(*id).image.clone();
//                     match url {
//                         Some(url) => component_mgr.world_2d.component_mgr.get_image_mut(*image_id).set_url(url),
//                         None => (),//component_mgr.world_2d.component_mgr.remove_image_mut(*image_id),
//                     }
//                 },
//                 None => {
//                     let url = component_mgr.node.background._group.get(*id).image.clone();
//                     match url {
//                         Some(url) => {
//                             let image = create_image(component_mgr, url, *parent);
//                             let image2d_ref = component_mgr.world_2d.component_mgr.add_image(image);
//                             self.0.borrow_mut().image_image2d_map.insert(*id, image2d_ref.id);
//                         },
//                         None => (),
//                     }
//                 },
//             } 
//         }
        
//     }
// }

pub struct BBSysImpl {
    shadow_sdf2d_map: VecMap<usize>, // id: decorate_id, value: sdf_id
    color_sdf2d_map: VecMap<usize>, // id: decorate_id, value: sdf_id
    image_image2d_map: VecMap<usize>,
    shadow_matrix_dirty: DirtyMark,

}

impl BBSysImpl {
    fn new() -> BBSysImpl {
        BBSysImpl{
            shadow_sdf2d_map: VecMap::new(),
            color_sdf2d_map: VecMap::new(),
            image_image2d_map: VecMap::new(),
            shadow_matrix_dirty: DirtyMark::new(),
        }
    }
}

fn create_image2d(src: u32, _node_id: usize) -> Image {
    let mut image = Image::default();
    image.src = src;
    image
}

//创建了一个sdf
fn create_box_sdf2d(mgr: &mut WorldDocMgr, node_id: usize) -> Sdf {
    let mut sdf = Sdf::default();
    let node = mgr.node._group.get(node_id);
    sdf.is_opaque = true;
    sdf.alpha = node.real_opacity;
    if sdf.alpha < 1.0 {
        sdf.is_opaque = false;
    }
    sdf.blur = 1.0;
    sdf.z_depth = node.z_depth;
    sdf.by_overflow = node.by_overflow;
    sdf.world_matrix =  mgr.node.world_matrix._group.get(node.world_matrix).owner.clone();

    let bound_box = mgr.node.bound_box._group.get(node.bound_box);
    sdf.center = Vector2::new(0.0, 0.0);

    let layout = &node.layout;
    println!("layout.border------------------{}", layout.border);
    sdf.extend = Vector2::new(layout.width/2.0 - layout.border, layout.height/2.0 - layout.border);

    // if node.world_matrix == 0 {
    //     sdf.rotate = 0.0;
    // }else {
    //     let world_matrix = mgr.node.world_matrix._group.get(node.world_matrix);
    //     sdf.rotate = matrix_to_rotate(world_matrix);
    // }

    sdf.bound_box = bound_box.owner.clone();

    let decorate = mgr.node.decorate._group.get(node.decorate);
    if decorate.background_color > 0 {
        let color = mgr.node.decorate.background_color._group.get(decorate.background_color).owner.clone();
        if sdf.is_opaque == true {
            sdf.is_opaque = color_is_opaque(&color)
        }
        sdf.color = color;
    }

    if decorate.border_color > 0 {
        println!("bordercolor====================");
        let color = mgr.node.decorate.border_color._group.get(decorate.border_color).owner.clone();
        if color.a < 1.0 {
            sdf.is_opaque = false;
        }
        println!("bordercolor===================={:?}", color);
        sdf.border_color = color;
        sdf.border_size = layout.border;
    }
    sdf
}

//创建了一个shadow sdf
fn create_shadow_sdf2d(mgr: &mut WorldDocMgr, node_id: usize) -> Sdf {
    let mut sdf = Sdf::default();
    let node = mgr.node._group.get(node_id);
    sdf.alpha = node.real_opacity;
    sdf.z_depth = node.z_depth - 0.00001;
    sdf.by_overflow = node.by_overflow;

    // let world_matrix = &mgr.node.world_matrix._group.get(node.world_matrix).owner;
    // let offset_matrix = cg::Matrix4::from_translation(Ve);

    sdf.world_matrix =  mgr.node.world_matrix._group.get(node.world_matrix).owner.clone();

    let shadow_id = mgr.node.decorate._group.get(node.decorate).box_shadow;
    let shadow = mgr.node.decorate.box_shadow._group.get(shadow_id);
    sdf.blur = 5.0;
    let bound_box = mgr.node.bound_box._group.get(node.bound_box);
    sdf.center = Vector2::new(bound_box.max.x - bound_box.min.x + shadow.h, bound_box.max.y - bound_box.min.y + shadow.v);

    let layout = &node.layout;
    sdf.extend = Vector2::new(layout.width/2.0, layout.height/2.0);

    
    // if node.world_matrix == 0 {
    //     sdf.rotate = 0.0;
    // }else {
    //     let world_matrix = mgr.node.world_matrix._group.get(node.world_matrix);
    //     sdf.rotate = matrix_to_rotate(world_matrix);
    // }

    sdf.bound_box = bound_box.owner.clone();
    sdf.color = Color::RGBA(shadow.color.clone());
    sdf
}

fn color_is_opaque(color: &Color) -> bool{
    match &color {
        Color::RGB(c) | Color::RGBA(c) => {
            if c.a < 1.0 {
                return false;
            }
            return true;
        },
        Color::LinearGradient(l) => {
            for c in l.list.iter() {
                if c.rgba.a < 1.0 {
                   return false;
                }
            }
            return true;
        },
        Color::RadialGradient(g) => {
            for c in g.list.iter() {
                if c.rgba.a < 1.0 {
                    return false
                }
            }
            return true;
        }
    }
}

fn modify_is_opacity(node_id: usize, sdf_id: usize, decorate_id: usize, component_mgr: &mut WorldDocMgr) {
    let node = component_mgr.node._group.get(node_id);
    if node.real_opacity < 1.0 {
        component_mgr.world_2d.component_mgr.get_sdf_mut(sdf_id).set_is_opaque(false);
    }else if !component_mgr.world_2d.component_mgr.get_sdf_mut(sdf_id).get_is_opaque(){
        let decorate = component_mgr.node.decorate._group.get(decorate_id);

        if decorate.border_color > 0 {
            let color = component_mgr.node.decorate.border_color._group.get(decorate.border_color).owner.clone();
            if color.a < 1.0 {
                return;
            }
        }

        if decorate.background_color > 0 {
            let color = component_mgr.node.decorate.background_color._group.get(decorate.background_color).owner.clone();
            component_mgr.world_2d.component_mgr.get_sdf_mut(sdf_id).set_is_opaque(color_is_opaque(&color));
        }
    }
}

// fn matrix_to_rotate(m: &Matrix4<f32>) -> f32 {
//     let mut q = Quaternion::new(0.0, 0.0, 0.0, 0.0);
//     decompose(m, None, Some(&mut q), None);
//     let e = Euler::from(q);
//     let z: Deg<f32> = Deg::from(e.z);
//     z.0
// }

fn cal_shadow_matrix(node_id: usize, shadow_id: usize, component_mgr: &mut WorldDocMgr) -> MathMatrix4{
    let (mut world_matrix, parent_id) = {
        let (transform_id, l, parent) = {
            let node = component_mgr.node._group.get(node_id);
            (node.transform, &node.layout, node.parent)
        };

        let center = if parent > 0 {
            //parent_layout
            let pl = &component_mgr.node._group.get(parent).layout;
            cg::Vector3::new(
                l.width/2.0 + l.left - pl.width/2.0,
                l.height/2.0 + l.top - pl.height/2.0,
                0.0,
            )
        }else {
            cg::Vector3::new(l.width/2.0, l.height/2.0, 0.0)
        };

        let transform = match transform_id == 0 {
            true => Transform::default().matrix(), // 优化？ 默认的matrix可以从全局取到 TODO
            false => component_mgr.node.transform._group.get(transform_id).matrix(),
        };

        let shadow_offset = {
            let shadow = component_mgr.node.decorate.box_shadow._group.get(shadow_id);
            CgVector3::new(shadow.h, shadow.v, 0.0)
        };
        let offset_matrix = cg::Matrix4::from_translation(shadow_offset);

        let center_matrix = cg::Matrix4::from_translation(center.clone());
        (center_matrix * offset_matrix * transform, parent)
    };

    if parent_id != 0 {
        let parent_world_matrix = {
            let parent_world_matrix_id = component_mgr.node._group.get(parent_id).world_matrix;
            ***component_mgr.node.world_matrix._group.get(parent_world_matrix_id)
        };
        world_matrix = parent_world_matrix * world_matrix;
    }

    MathMatrix4(world_matrix)
}