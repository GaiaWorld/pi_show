/// 资源管理器， 
/// 定期将不被使用的资源移动到预释放列表， 延迟资源的释放
/// 定期扫描预释放列表， 如果超时， 资源未被再利用， 直接从该列表中删除资源， 并从资源表（tables）中删除， 否则将资源转移到res_array中

use std::hash::Hash;
use std::any::{ TypeId, Any };

use fnv::FnvHashMap;

use share::Share;

pub trait Res{
    type Key: Hash + Eq + Clone;
}

pub struct ResMgr{
    tables: FnvHashMap<TypeId, Share<dyn Any>>,
    timeout: u32, // 默认超时时间
    prepare_release_array: Vec<Box<dyn Release>>,
    res_array: Vec<Box<dyn Release>>,
}

impl ResMgr {
    // 创建资源管理器
    pub fn new(timeout: u32) -> Self{
        ResMgr{
            timeout,
            tables: FnvHashMap::default(),
            prepare_release_array: Vec::default(),
            res_array: Vec::default(),
        }
    }

    // 获取资源
    pub fn get<T: Res + 'static>(&self, name: &<T as Res>::Key) -> Option<Share<T>>{
        match self.tables.get(&TypeId::of::<T>()) {
            Some(map) => {
                match map.clone().downcast::<ResMap<T>>() {
                    Ok(r) => match r.get(name) {
                        Some(r) => Some(r.clone()),
                        None => None,
                    },
                    Err(_) => None
                }
            },
            None => None,
        }
    }

    // 创建资源
    #[inline]
    pub fn create<T: Res + 'static>(&mut self, name: T::Key, value: T) -> Share<T>{
        let r = self.tables.entry(TypeId::of::<T>()).or_insert(Share::new(ResMap::<T>::new())).clone().downcast::<ResMap<T>>().unwrap().create(name.clone(), value);
        self.res_array.push(Box::new(ReleaseRes{
            key: name,
            value: r.clone(),
            release_point: 0,
        }));
        r
    }

    // 移除资源
    #[inline]
    pub fn remove<T: Res + 'static>(&mut self, name: &<T as Res>::Key) {
        match self.tables.get(&TypeId::of::<T>()) {
            Some(map) => {
                match map.clone().downcast::<ResMap<T>>() {
                    Ok(r) => r.remove(name),
                    Err(_) => ()
                }
            },
            None => (),
        };
    }

    // 预整理, 将即将释放的资源从资源表中移除， 并添加到释放列表中
    // 该方法应该被外部循环驱动
    pub fn prepare_conllect(&mut self, now: u32, min_release_time: &mut u32) {
        let mut i = 0;
        loop {
            if i < self.res_array.len() {
                if self.res_array[i].strong_count() == 2 {
                    let r = self.res_array.swap_remove(i);
                    let time = now + (self.timeout);
                    r.set_release_time(time);
                    self.prepare_release_array.push(r);
                    if *min_release_time > time {
                        *min_release_time = time;
                    }
                } else {
                    i += 1;
                }
            } else {
                break;
            }
        }
    }

    // 整理，将需要释放的资源从释放列表中移除，且从资源映射表中移除， 不需要释放的资源， 放回资源表中， 未到时间释放的资源继续保留在释放列表中
    // 该方法应该被外部循环驱动
    pub fn conllect(&mut self, now: u32, min_release_time: &mut u32) {
        let mut i = 0;
        loop {
            if i < self.prepare_release_array.len() {
                match self.prepare_release_array[i].release(unsafe{&mut *(self as *const Self as usize as *mut Self)}, now, min_release_time) {
                    ReleaseType::Success => {self.prepare_release_array.swap_remove(i);}, // 释放成功， 从释放列表中移除
                    ReleaseType::Fail => self.res_array.push(self.prepare_release_array.swap_remove(i)), // 无法释放， 将资源重新放入资源列表中
                    ReleaseType::None => i += 1, // 未到释放时间， 跳过
                }
            } else {
                break;
            }
        }
    }
}

trait Release {
    fn release(&self, res_mgr: &mut ResMgr, now: u32, min_release_time: &mut u32) -> ReleaseType;
    fn set_release_time(&self, time: u32);
    fn strong_count(&self) -> usize;
}

enum ReleaseType {
    Success, // 释放成功
    Fail, // 引用计数大于2， 无法释放
    None, // 未到释放时间
}

struct ReleaseRes1<R: Res + 'static>(Share<R>);

impl<R: Res + 'static> Release for  ReleaseRes1<R>{
    fn release(&self, _res_mgr: &mut ResMgr, _now: u32, _min_release_time: &mut u32) -> ReleaseType {
        unimplemented!{}
    }

    #[inline]
    fn set_release_time(&self, _time: u32){
        // unsafe{&mut *(self as *const Self as usize as *mut Self)}.release_point = time;
    }

    #[inline]
    fn strong_count(&self) -> usize {
        Share::strong_count(&self.0)
    }
}

struct ReleaseRes<R: Res + 'static>{
    key: R::Key,
    value: Share<R>,
    release_point: u32,
}

impl<R: Res + 'static> Release for  ReleaseRes<R>{
    fn release(&self, res_mgr: &mut ResMgr, now: u32, min_release_time: &mut u32) -> ReleaseType {
        if now <= self.release_point + 500 {
            if *min_release_time > self.release_point {
                *min_release_time = self.release_point;
            }
            return ReleaseType::None;
        } else if Share::strong_count(&self.value) != 2 {
            return ReleaseType::Fail;
        } else {
            res_mgr.remove::<R>(&self.key);
            return ReleaseType::Success;
        }
    }

    #[inline]
    fn set_release_time(&self, time: u32){
        unsafe{&mut *(self as *const Self as usize as *mut Self)}.release_point = time;
    }

    #[inline]
    fn strong_count(&self) -> usize {
        Share::strong_count(&self.value)
    }
}

//资源表
pub struct ResMap<T: Res>(FnvHashMap<<T as Res>::Key, Share<T>>);

impl<T: Res> ResMap<T> {
    #[inline]
    pub fn new() -> ResMap<T>{
        ResMap(FnvHashMap::with_capacity_and_hasher(0, Default::default()))
    }
	// 获得指定键的资源
    #[inline]
	pub fn get(&self, name: &<T as Res>::Key) -> Option<&Share<T>> {
        match self.0.get(name) {
            Some(r) => Some(&r),
            None => None,
        }
    }
	// 创建资源
    #[inline]
	pub fn create(&self, name: T::Key, res: T) -> Share<T> {
        let r = Share::new(res);
        unsafe{&mut *(self as *const Self as usize as *mut Self)}.0.insert(name, r.clone());
        r
    }

    #[inline]
    pub fn remove(&self, name: &<T as Res>::Key) {
        unsafe{&mut *(self as *const Self as usize as *mut Self)}.0.remove(name);
    }

    pub fn conllect(&mut self) {

    }
}