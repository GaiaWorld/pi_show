use std::rc::Rc;
use std::cell::RefCell;

use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent, ModifyFieldEvent};
use wcs::world::System;
use map::vecmap::{VecMap};

use component::math::{Vector2, Matrix4 as MathMatrix4};
use world_doc::component::node::{Node};
use world_doc::component::style::element::{ Image, ElementId};
use world_doc::WorldDocMgr;
use world_2d::component::image::{ Image as Image2d };
use render::res::TextureRes;

//背景边框系统
pub struct ImageSys(Rc<RefCell<ImageSysImpl>>);

impl System<(), WorldDocMgr> for ImageSys{
    fn run(&self, _e: &(), _component_mgr: &mut WorldDocMgr){
    }
}

impl ImageSys {
    pub fn init(component_mgr: &mut WorldDocMgr) -> Rc<ImageSys> {
        let r = Rc::new(ImageSys(Rc::new(RefCell::new(ImageSysImpl::new()))));
        //监听image的创建， 修改， 删除 事件
        component_mgr.node.element.image._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Image, CreateEvent, WorldDocMgr>>)));
        component_mgr.node.element.image._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Image, DeleteEvent, WorldDocMgr>>)));
        component_mgr.node.element.image._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Image, ModifyFieldEvent, WorldDocMgr>>)));
        
        // 监听node中z_depth， by_overflow， real_opacity， layout, real_visibility的改变， 修改Image渲染对象上对应的值
        component_mgr.node.z_depth.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node.by_overflow.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node.real_opacity.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node.layout.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        component_mgr.node.real_visibility.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        
        // 监听worldmatrix的改变， 修改Image渲染对象上对应的值
        component_mgr.node.world_matrix._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<MathMatrix4, ModifyFieldEvent, WorldDocMgr>>)));
        r
    }
}

//监听image的创建事件， 创建对应Image渲染对象
impl ComponentHandler<Image, CreateEvent, WorldDocMgr> for ImageSys {
    fn handle(&self, event: &CreateEvent, component_mgr: &mut WorldDocMgr) {
        let CreateEvent { id, parent} = event;
        let mut borrow_mut = self.0.borrow_mut();
        let src = component_mgr.node.element.image._group.get(*id).src;
        let texture = usize_to_textrue(src);
        let mut image = Image2d::new(texture);

        let node = component_mgr.node._group.get(*parent);
        let layout = &node.layout;
        let matrix =  component_mgr.node.world_matrix._group.get(node.world_matrix).owner.clone();
        image.alpha = node.real_opacity;
        image.is_opaque = true;
        image.by_overflow = node.by_overflow;
        image.z_depth = node.z_depth;
        image.extend = Vector2::new(layout.width/2.0 - layout.border, layout.height/2.0 - layout.border);
        image.world_matrix = matrix;
        image.visibility = node.real_visibility;

        let mut image2d_ref = component_mgr.world_2d.component_mgr.add_image(image);
        image2d_ref.set_parent(*id);
        borrow_mut.image_image2d_map.insert(*id, image2d_ref.id);
    }
}

//监听image的修改， 删除对应Image渲染对象
impl ComponentHandler<Image, DeleteEvent, WorldDocMgr> for ImageSys {
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut WorldDocMgr) {
        let DeleteEvent { id, parent: _} = event;
        let mut borrow_mut = self.0.borrow_mut();
        let image2d_id = unsafe { borrow_mut.image_image2d_map.remove_unchecked(*id) };
        component_mgr.world_2d.component_mgr.del_image(image2d_id);
    }
}

//监听image的修改， 删除对应Image渲染对象
impl ComponentHandler<Image, ModifyFieldEvent, WorldDocMgr> for ImageSys {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr) {
        let ModifyFieldEvent { id, parent: _, field: _} = event;
        let borrow = self.0.borrow();
        let image2d_id = *unsafe { borrow.image_image2d_map.get_unchecked(*id)};
        let src = component_mgr.node.element.image._group.get(*id).src;
        let texture = usize_to_textrue(src);

        component_mgr.world_2d.component_mgr.get_image_mut(image2d_id).set_src(texture);
    }
}

//node的修改事件
impl ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr> for ImageSys {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr) {
        let ModifyFieldEvent { id, parent: _, field} = event;

        let node = component_mgr.node._group.get(*id);
        let image_id = match node.element {
            ElementId::Image(image_id) => {
                if image_id == 0 {
                    return
                }
                image_id
            },
            _ => return,
        };

        let borrow = self.0.borrow();
        let image_2d_id = unsafe { borrow.image_image2d_map.get_unchecked(image_id)};
        if *field == "z_depth" {
            component_mgr.world_2d.component_mgr.get_image_mut(*image_2d_id).set_z_depth(node.z_depth);
        }else if *field == "by_overflow" {
            component_mgr.world_2d.component_mgr.get_image_mut(*image_2d_id).set_by_overflow(node.by_overflow);
        }else if *field == "real_opacity" {
            component_mgr.world_2d.component_mgr.get_image_mut(*image_2d_id).set_alpha(node.real_opacity);
        } else if *field == "layout" {
            let layout = &node.layout;
            component_mgr.world_2d.component_mgr.get_image_mut(*image_2d_id).set_extend(Vector2::new(layout.width/2.0 - layout.border, layout.height/2.0 - layout.border));
        } else if *field == "real_visibility" {
            component_mgr.world_2d.component_mgr.get_image_mut(*image_2d_id).set_visibility(node.real_visibility);
        }
    }
}

//监听世界矩阵的改变， 修改image_2d中的世界矩阵
impl ComponentHandler<MathMatrix4, ModifyFieldEvent, WorldDocMgr> for ImageSys {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut WorldDocMgr) {
        let ModifyFieldEvent { id: _, parent, field: _ } = event; 
        let node = component_mgr.node._group.get(*parent);
        let image_id = match node.element {
            ElementId::Image(image_id) => {
                if image_id == 0 {
                    return
                }
                image_id
            },
            _ => return ,
        };

        let borrow = self.0.borrow_mut();
        let image_2d_id = unsafe { borrow.image_image2d_map.get_unchecked(image_id)};

        let world_matrix = cal_matrix(*parent, component_mgr);
        component_mgr.world_2d.component_mgr.get_image_mut(*image_2d_id).set_world_matrix(MathMatrix4(world_matrix));
    }
}

fn cal_matrix(node_id: usize, mgr: &mut WorldDocMgr) -> cg::Matrix4<f32>{
    let node = mgr.node._group.get(node_id);
    let world_matrix = &mgr.node.world_matrix._group.get(node.world_matrix).owner;
    if node.transform != 0 {
        let transform = mgr.node.transform._group.get(node.transform);
        let origin = transform.origin.to_value(node.layout.width, node.layout.height);
        if origin.x != 0.0 || origin.y != 0.0 {
            return world_matrix.0 * cg::Matrix4::from_translation(cg::Vector3::new(-origin.x, -origin.y, 0.0));
        }
    }
    world_matrix.0.clone()
}


pub struct ImageSysImpl {
    image_image2d_map: VecMap<usize>, // id: image_id, value: image2d_id
}

impl ImageSysImpl {
    fn new() -> ImageSysImpl {
        ImageSysImpl{
            image_image2d_map: VecMap::new(),
        }
    }
}

fn usize_to_textrue(src: usize) -> Rc<TextureRes> {
    unsafe{& *(src as *const Rc<TextureRes>)} .clone()
}