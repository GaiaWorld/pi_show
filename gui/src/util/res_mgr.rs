// 显卡资源管理器
use std::sync::{Arc};
use std::hash::Hash;
use std::any::{ TypeId, Any };
use std::ops::{ Deref };

use fnv::FnvHashMap;
use set_timeout;

// 定时的时间
static mut DEFAULT_TIMEOUT: usize = 1000;

// 最小释放的时间
static mut MIN_RELEASE_TIMEOUT: usize = 500;

// 回收方法的定时器的引用
static mut TIMER_REF: usize = 0;

lazy_static! {
    //common attribute
    pub static ref RELEASE_ARRAY: Vec<Arc<dyn Release>> = Vec::new();
}

//资源接口
pub trait ResTrait: Release {
    type Key: Hash + Eq + Clone + Send + 'static + Sync;
	// 获得资源的唯一名称
	fn name(&self) -> &Self::Key;
	// 判断是否存活
	//fn is_alive(&self) -> bool;
	// 创建资源, 如果异步，可以返回Result<Promise>
	//fn create(&mut self) -> bool;
}

pub trait Release: Send + 'static + Sync {}

pub struct Res<T: ResTrait>(pub Arc<T>);

impl<T: ResTrait> Clone for Res<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ResTrait> Deref for Res<T> {
    type Target = T;
    fn deref(&self) -> &T{
        &self.0
    }
}

impl<T: ResTrait> Drop for Res<T> {
    fn drop(&mut self) {
        if Arc::strong_count(&self.0) == 1 {
            let r = self.0.clone();
            set_timeout(36000, Box::new(move ||{
                let _r = r;
            }));
        }
    }
}

pub struct ResMgr(FnvHashMap<TypeId, Arc<dyn Any + Send + Sync>>);

impl ResMgr {
    pub fn new() -> Self{
        ResMgr(FnvHashMap::default())
    }

    pub fn get<T: ResTrait>(&self, key: &<T as ResTrait>::Key) -> Option<Res<T>>{
        match self.0.get(&TypeId::of::<T>()) {
            Some(map) => {
                match map.clone().downcast::<ResMap<T>>() {
                    Ok(r) => r.get(key),
                    Err(_) => None
                }
            },
            None => None,
        }
    }

    pub fn create<T: ResTrait>(&mut self, value: T) -> Res<T>{
        self.0.entry(TypeId::of::<T>()).or_insert(Arc::new(ResMap::<T>::new())).clone().downcast::<ResMap<T>>().unwrap().create(value)
    }
}

//资源表
pub struct ResMap<T: ResTrait> (FnvHashMap<<T as ResTrait>::Key, Arc<T>>);

impl<T:ResTrait> ResMap<T> {
    pub fn new() -> ResMap<T>{
        ResMap(FnvHashMap::with_capacity_and_hasher(0, Default::default()))
    }
	// 获得指定键的资源
	pub fn get(&self, name: &<T as ResTrait>::Key) -> Option<Res<T>> {
        if let Some(v) = self.0.get(name) {
            return Some(Res(v.clone()))
        }
        None
    }
	// 创建资源
	pub fn create(&self, res: T) -> Res<T> {
        let name = res.name().clone();
        let r = Arc::new(res);
        unsafe{&mut *(self as *const Self as usize as *mut Self)}.0.insert(name, r.clone());
        Res(r)
        // match self.0.entry(res.name()) {
        //     Entry::Occupied(mut e) => {
        //         let v = e.get_mut();
        //         match v.upgrade() {
        //             Some(r) => r,
        //             None =>{
        //                 res.create();
        //                 let r = Arc::new(res);
        //                 swap(&mut Arc::downgrade(&r), v);
        //                 r
        //             }
        //         }
        //     },
        //     Entry::Vacant(e) => {
        //         res.create();
        //         let r = Arc::new(res);
        //         e.insert(Arc::downgrade(&r));
        //         r
        //     }
        // }
    }
	// 定期整理，去除已经释放的资源的弱引用
	pub fn collate(&mut self) {
    }

}

// pub struct ResMgr {
//     pub img: ResMap<ImgRes>,
// }

// impl ResMgr {
//     pub fn new() -> ResMgr{
//         ResMgr{
//             img: ResMap::new(),
//         }
//     }
// }