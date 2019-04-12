//对需要渲染的物件按照是否透明进行分类， 并将透明物体依照z值进行排序
//先渲染不透明物体， 再按照次序渲染透明物体

use std::cell::RefCell;
use std::rc::{Rc};
use std::cmp::{Ord, Ordering, Eq, PartialEq};

use wcs::world::{System};

use world_2d::World2dMgr;
use world_2d::system::render_util::sdf;

pub struct Render(RefCell<RenderImpl>);

impl System<(), World2dMgr> for Render{
    fn run(&self, _e: &(), component_mgr: &mut World2dMgr){
        self.0.borrow_mut().render(component_mgr);
    }
}

impl Render {
    pub fn init(_component_mgr: &mut World2dMgr) -> Rc<Render>{
        let r = Rc::new(Render(RefCell::new(RenderImpl::new())));
        r
    }
}

pub struct RenderImpl {
    transparent_objs: Vec<SortObject>,
    opaque_objs: Vec<SortObject>,
}

impl RenderImpl {
    pub fn new() -> RenderImpl{
        RenderImpl{
            transparent_objs: Vec::new(),
            opaque_objs: Vec::new(),
        }
    }

    pub fn render(&mut self, mgr: &mut World2dMgr) {
        // println!("render---------------------------------", );
        self.list_obj(mgr);
        for v in self.opaque_objs.iter() {
            match v.ty {
                RenderType::Sdf => {
                    #[cfg(feature = "log")]
                    println!("sdf opaque_objs render---------------------------------", );
                    sdf::render(mgr, v.id);
                },
                _ => (),
            }
        }

        for v in self.transparent_objs.iter() {
            match v.ty {
                RenderType::Sdf => {
                    #[cfg(feature = "log")]
                    println!("sdf transparent_objs render---------------------------------", );
                    sdf::render(mgr, v.id);
                },
                _ => (),
            }
        }
        self.opaque_objs.clear();
        self.transparent_objs.clear();
    }

    //对不透明物体和透明物体排序
    fn list_obj(&mut self, mgr: &mut World2dMgr){
        // println!("list_obj---------------------------------", );
        for v in mgr.image._group.iter() {
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
            // println!("sdf render---------------------------------", );
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