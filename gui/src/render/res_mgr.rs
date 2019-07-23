// 显卡资源管理器
use std::hash::Hash;
use std::any::{ TypeId, Any };
use hal_core::*;

use fnv::FnvHashMap;

use share::Share;

pub trait Res<C: HalContext + 'static + 'static>{
    type Key: Hash + Eq + Clone;

    fn destroy(&self, gl: &C);
}

pub struct ResMgr<C: HalContext + 'static + 'static>{
    tables: FnvHashMap<TypeId, Share<dyn Any>>,
    timeout: u32, // 默认超时时间
    release_array: Vec<Box<dyn Release<C>>>,
    res_array: Vec<Box<dyn Release<C>>>,
    other_res: Vec<Box<dyn Release<C>>>, // 不会被共享， 仅仅为了管理显卡资源的生命周期
}

impl<C: HalContext + 'static + 'static> ResMgr<C> {
    pub fn new(timeout: u32) -> Self{
        ResMgr{
            timeout,
            tables: FnvHashMap::default(),
            release_array: Vec::default(),
            res_array: Vec::default(),
            other_res: Vec::default(),
        }
    }

    pub fn get<T: Res<C> + 'static>(&self, name: &<T as Res<C>>::Key) -> Option<Share<T>>{
        match self.tables.get(&TypeId::of::<T>()) {
            Some(map) => {
                match map.clone().downcast::<ResMap<C, T>>() {
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

    // 添加一个资源， 但不会有任何方法能取到该资源， 仅仅用来管理显卡资源的生命周期
    #[inline]
    pub fn add<T: Res<C> + 'static>(&mut self, value: T ) -> Share<T> {
        let v = Share::new(value);
        self.other_res.push(Box::new(ReleaseRes1(v.clone(), std::marker::PhantomData)));
        v
    }

    #[inline]
    pub fn create<T: Res<C> + 'static>(&mut self, name: T::Key, value: T) -> Share<T>{
        let r = self.tables.entry(TypeId::of::<T>()).or_insert(Share::new(ResMap::<C, T>::new())).clone().downcast::<ResMap<C, T>>().unwrap().create(name.clone(), value);
        self.res_array.push(Box::new(ReleaseRes{
            key: name,
            value: r.clone(),
            release_point: 0,
        }));
        r
    }

    #[inline]
    pub fn remove<T: Res<C> + 'static>(&mut self, name: &<T as Res<C>>::Key) {
        match self.tables.get(&TypeId::of::<T>()) {
            Some(map) => {
                match map.clone().downcast::<ResMap<C, T>>() {
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
                    self.release_array.push(r);
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
    pub fn conllect(&mut self, now: u32, min_release_time: &mut u32, gl: &C) {
        let mut i = 0;
        loop {
            if i < self.release_array.len() {
                match self.release_array[i].release(unsafe{&mut *(self as *const Self as usize as *mut Self)}, now, min_release_time) {
                    ReleaseType::Success => self.release_array.swap_remove(i).destroy(gl), // 释放成功， 从释放列表中移除
                    ReleaseType::Fail => self.res_array.push(self.release_array.swap_remove(i)), // 无法释放， 将资源重新放入资源列表中
                    ReleaseType::None => i += 1, // 未到释放时间， 跳过
                }
            } else {
                break;
            }
        }

        loop {
            if i < self.other_res.len() {
                if self.other_res[i].strong_count() == 1 {
                    self.release_array.swap_remove(i).destroy(gl);
                } else {
                    i += 1;
                }
            } else {
                break;
            }
        }
    }
}

trait Release<C: HalContext + 'static + 'static> {
    fn release(&self, res_mgr: &mut ResMgr<C>, now: u32, min_release_time: &mut u32) -> ReleaseType;
    fn set_release_time(&self, time: u32);
    fn strong_count(&self) -> usize;
    fn destroy(&self, gl: &C);
}

enum ReleaseType {
    Success, // 释放成功
    Fail, // 引用计数大于2， 无法释放
    None, // 未到释放时间
}

struct ReleaseRes1<C: HalContext + 'static, R: Res<C> + 'static>(Share<R>, std::marker::PhantomData<C>);

impl<C: HalContext + 'static, R: Res<C> + 'static> Release<C> for  ReleaseRes1<C, R>{
    fn release(&self, _res_mgr: &mut ResMgr<C>, _now: u32, _min_release_time: &mut u32) -> ReleaseType {
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

    #[inline]
    fn destroy(&self, gl: &C) {
        self.0.destroy(gl);
    }
}

struct ReleaseRes<C: HalContext + 'static, R: Res<C> + 'static>{
    key: R::Key,
    value: Share<R>,
    release_point: u32,
}

impl<C: HalContext + 'static, R: Res<C> + 'static> Release<C> for  ReleaseRes<C, R>{
    fn release(&self, res_mgr: &mut ResMgr<C>, now: u32, min_release_time: &mut u32) -> ReleaseType {
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

    #[inline]
    fn destroy(&self, gl: &C) {
        self.value.destroy(gl);
    }
}

//资源表
pub struct ResMap<C: HalContext + 'static, T: Res<C>>(FnvHashMap<<T as Res<C>>::Key, Share<T>>);

impl<C: HalContext + 'static, T: Res<C>> ResMap<C, T> {
    #[inline]
    pub fn new() -> ResMap<C, T>{
        ResMap(FnvHashMap::with_capacity_and_hasher(0, Default::default()))
    }
	// 获得指定键的资源
    #[inline]
	pub fn get(&self, name: &<T as Res<C>>::Key) -> Option<&Share<T>> {
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
    pub fn remove(&self, name: &<T as Res<C>>::Key) {
        unsafe{&mut *(self as *const Self as usize as *mut Self)}.0.remove(name);
    }

    pub fn conllect(&mut self) {

    }
}