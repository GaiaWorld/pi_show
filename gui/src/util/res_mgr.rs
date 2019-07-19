// 显卡资源管理器
use share::Share;
use std::hash::Hash;
use std::any::{ TypeId, Any };

use fnv::FnvHashMap;

pub trait Res{
    type Key: Hash + Eq + Clone;
}

pub struct ResMgr{
    tables: FnvHashMap<TypeId, Share<dyn Any>>,
    pub timeout: u32,
}

impl ResMgr {
    pub fn new(timeout: u32) -> Self{
        ResMgr{
            timeout,
            tables: FnvHashMap::default(),
        }
    }

    pub fn get<T: Res + 'static>(&self, key: &<T as Res>::Key) -> Option<Share<T>>{
        match self.tables.get(&TypeId::of::<T>()) {
            Some(map) => {
                match map.clone().downcast::<ResMap<T>>() {
                    Ok(r) => match r.get(key) {
                        Some(r) => Some(r.clone()),
                        None => None,
                    },
                    Err(_) => None
                }
            },
            None => None,
        }
    }

    pub fn create<T: Res + 'static>(&mut self, name: T::Key, value: T) -> Share<T>{
        self.tables.entry(TypeId::of::<T>()).or_insert(Share::new(ResMap::<T>::new())).clone().downcast::<ResMap<T>>().unwrap().create(name, value, self.timeout)
    }
}

//资源表
pub struct ResMap<T: Res> (FnvHashMap<<T as Res>::Key, (Share<T>, u32)>);

impl<T:Res> ResMap<T> {
    pub fn new() -> ResMap<T>{
        ResMap(FnvHashMap::with_capacity_and_hasher(0, Default::default()))
    }
	// 获得指定键的资源
	pub fn get(&self, name: &<T as Res>::Key) -> Option<&Share<T>> {
        match self.0.get(name) {
            Some(r) => Some(&r.0),
            None => None,
        }
    }
	// 创建资源
	pub fn create(&self, name: T::Key, res: T, timeout: u32) -> Share<T> {
        let r = Share::new(res);
        unsafe{&mut *(self as *const Self as usize as *mut Self)}.0.insert(name, (r.clone(), timeout));
        r
    }
	// 定期整理，去除已经释放的资源的弱引用
	pub fn collate(&mut self) {
    }
}