// 显卡资源管理器
use std::rc::{Rc, Weak};
use fnv::FnvHashMap;

use atom::{Atom};


//资源接口
pub trait Res {
	// 获得资源的唯一名称
	fn name(&self) -> &Atom;
	// 判断是否存活
	//fn is_alive(&self) -> bool;
	// 创建资源, 如果异步，可以返回Result<Promise>
	//fn create(&mut self) -> bool;
	// 释放
	fn release(&self);
}

//资源表
pub struct ResMap<T> (FnvHashMap<Atom, Weak<T>>);

impl<T:Res> ResMap<T> {
    pub fn new() -> ResMap<T>{
        ResMap(FnvHashMap::with_capacity_and_hasher(0, Default::default()))
    }
	// 获得指定键的资源
	pub fn get(&self, name: &Atom) -> Option<Rc<T>> {
        if let Some(v) = self.0.get(name) {
            if let Some(r) = v.upgrade() {
                return Some(r)
            }
        }
        None
    }
	// 创建资源
	pub fn create(&mut self, res: T) -> Rc<T> {
        let name = res.name().clone();
        let r = Rc::new(res);
        self.0.insert(name, Rc::downgrade(&r));
        r
        // match self.0.entry(res.name()) {
        //     Entry::Occupied(mut e) => {
        //         let v = e.get_mut();
        //         match v.upgrade() {
        //             Some(r) => r,
        //             None =>{
        //                 res.create();
        //                 let r = Rc::new(res);
        //                 swap(&mut Rc::downgrade(&r), v);
        //                 r
        //             }
        //         }
        //     },
        //     Entry::Vacant(e) => {
        //         res.create();
        //         let r = Rc::new(res);
        //         e.insert(Rc::downgrade(&r));
        //         r
        //     }
        // }
    }
	// 定期整理，去除已经释放的资源的弱引用
	pub fn collate(&mut self) {
    }

}

pub struct ResMgr {
    pub img: ResMap<ImgRes>,
}

impl ResMgr {
    pub fn new() -> ResMgr{
        ResMgr{
            img: ResMap::new(),
        }
    }
}


pub struct ImgRes {
    pub name: Atom,
    pub width: usize,
    pub height: usize,
    pub opacity: usize,
    pub compress: usize,
    pub handler: usize,
}

impl ImgRes {
    // 创建资源
	pub fn new(key: Atom, width: usize,
    height: usize,
    opacity: usize,
    compress: usize) -> Self{
        ImgRes {
            name: key,
            width: width,
            height: height,
            opacity: opacity,
            compress: compress,
            handler: 0,
        }
    }
}
impl Res for ImgRes {
	// 创建资源
	fn name(&self) -> &Atom{
        &self.name
    }
	// 释放
	fn release(&self){

    }

}