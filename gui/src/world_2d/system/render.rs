//对需要渲染的物件按照是否透明进行分类， 并将透明物体依照z值进行排序
//先渲染不透明物体， 再按照次序渲染透明物体

use std::cell::RefCell;
use std::rc::{Rc};
use std::cmp::{Ord, Ordering, Eq, PartialEq};
use webgl_rendering_context::{WebGLRenderingContext};

use wcs::world::{System};
use wcs::component::{CreateEvent, DeleteEvent, ModifyFieldEvent, ComponentHandler};

use world_2d::World2dMgr;
use world_2d::system::render_util::sdf;
use world_2d::system::render_util::image;
use world_2d::component::image::Image;
use world_2d::component::sdf::Sdf;

pub struct Render(RefCell<RenderImpl>);

impl System<(), World2dMgr> for Render{
    fn run(&self, _e: &(), component_mgr: &mut World2dMgr){
        self.0.borrow_mut().render(component_mgr);
    }
}

impl Render {
    pub fn init(component_mgr: &mut World2dMgr) -> Rc<Render>{
        let r = Rc::new(Render(RefCell::new(RenderImpl::new())));
        component_mgr.sdf._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Sdf, CreateEvent, World2dMgr>>)));
        component_mgr.sdf._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Sdf, DeleteEvent, World2dMgr>>)));
        component_mgr.sdf._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Sdf, ModifyFieldEvent, World2dMgr>>)));
        component_mgr.image._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Image, CreateEvent, World2dMgr>>)));
        component_mgr.image._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Image, DeleteEvent, World2dMgr>>)));
        component_mgr.image._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Image, ModifyFieldEvent, World2dMgr>>)));
        r
    }
}

impl ComponentHandler<Sdf, DeleteEvent, World2dMgr> for Render{
    fn handle(&self, _event: &DeleteEvent, _component_mgr: &mut World2dMgr){
        self.0.borrow_mut().dirty = true;
    }
}

impl ComponentHandler<Sdf, CreateEvent, World2dMgr> for Render{
    fn handle(&self, _event: &CreateEvent, _component_mgr: &mut World2dMgr){
        self.0.borrow_mut().dirty = true;
    }
}

impl ComponentHandler<Sdf, ModifyFieldEvent, World2dMgr> for Render{
    fn handle(&self, _event: &ModifyFieldEvent, _component_mgr: &mut World2dMgr){
        self.0.borrow_mut().dirty = true;
    }
}

impl ComponentHandler<Image, DeleteEvent, World2dMgr> for Render{
    fn handle(&self, _event: &DeleteEvent, _component_mgr: &mut World2dMgr){
        self.0.borrow_mut().dirty = true;
    }
}

impl ComponentHandler<Image, CreateEvent, World2dMgr> for Render{
    fn handle(&self, _event: &CreateEvent, _component_mgr: &mut World2dMgr){
        self.0.borrow_mut().dirty = true;
    }
}

impl ComponentHandler<Image, ModifyFieldEvent, World2dMgr> for Render{
    fn handle(&self, _event: &ModifyFieldEvent, _component_mgr: &mut World2dMgr){
        self.0.borrow_mut().dirty = true;
    }
}

pub struct RenderImpl {
    transparent_objs: Vec<SortObject>,
    opaque_objs: Vec<SortObject>,
    dirty: bool,
}

impl RenderImpl {
    pub fn new() -> RenderImpl{
        RenderImpl{
            transparent_objs: Vec::new(),
            opaque_objs: Vec::new(),
            dirty: false,
        }
    }

    pub fn render(&mut self, mgr: &mut World2dMgr) {
        if self.dirty == false {
            return;
        }
        mgr.engine.gl.clear(WebGLRenderingContext::COLOR_BUFFER_BIT | WebGLRenderingContext::DEPTH_BUFFER_BIT);
        self.list_obj(mgr);
        for v in self.opaque_objs.iter() {
            match v.ty {
                RenderType::Sdf => {
                    sdf::render(mgr, v.id);
                },
                RenderType::Image => {
                    image::render(mgr, v.id);
                },
                _ => (),
            }
        }

        for v in self.transparent_objs.iter() {
            match v.ty {
                RenderType::Sdf => {
                    sdf::render(mgr, v.id);
                },
                RenderType::Image => {
                    image::render(mgr, v.id);
                },
                _ => (),
            }
        }
        self.opaque_objs.clear();
        self.transparent_objs.clear();
    }

    //对不透明物体和透明物体排序
    fn list_obj(&mut self, mgr: &mut World2dMgr){
        for v in mgr.image._group.iter() {
            if v.1.visibility == false {
                continue;
            }
            if v.1.is_opaque {
                self.opaque_objs.push(SortObject {
                    z: v.1.z_depth,
                    id: v.0,
                    ty: RenderType::Image,
                });
            }else {
                self.transparent_objs.push(SortObject {
                    z: v.1.z_depth,
                    id: v.0,
                    ty: RenderType::Image,
                });
            }
        }

        for v in mgr.sdf._group.iter() {
            if v.1.visibility == false {
                continue;
            }
            if v.1.is_opaque {
                self.opaque_objs.push(SortObject {
                    z: v.1.z_depth,
                    id: v.0,
                    ty: RenderType::Sdf,
                });
            }else {
                self.transparent_objs.push(SortObject {
                    z: v.1.z_depth,
                    id: v.0,
                    ty: RenderType::Sdf,
                });
            }
        }
        self.transparent_objs.sort();
    }
}


struct SortObject {
    z: f32,
    id: usize,
    ty: RenderType,
}

#[allow(dead_code)]
enum RenderType {
    Image,
    Word,
    Sdf,
}

impl PartialOrd for SortObject {
	fn partial_cmp(&self, other: &SortObject) -> Option<Ordering> {
		self.z.partial_cmp(&other.z)
	}
}

impl PartialEq for SortObject{
	 fn eq(&self, other: &SortObject) -> bool {
        self.z.eq(&other.z)
    }
}

impl Eq for SortObject{}

impl Ord for SortObject{
	fn cmp(&self, other: &SortObject) -> Ordering {
        let r = self.partial_cmp(&other).unwrap();
        r

    }
}