// 显卡资源管理器
use share::Share;
use std::hash::Hash;
use std::any::{ TypeId, Any };
use std::ops::{ Deref };
use std::cell::RefCell;

use fnv::FnvHashMap;
use { set_timeout, now_time, cancel_timeout };

// 定时的时间
static mut DEFAULT_TIMEOUT: usize = 1000;

// 最小释放的时间
static mut MIN_RELEASE_TIMEOUT: usize = 500;

// 回收方法的定时器的引用
static mut TIMER_REF: usize = 0;

static mut TIMER_TIME: u64 = std::u64::MAX;

lazy_static! {
    //common attribute
    pub static ref RELEASE_ARRAY: ReleaseArray = ReleaseArray(RefCell::new(Vec::new()));
}

pub struct ReleaseArray(RefCell<Vec<(Share<dyn Release>, u64)>>);

unsafe impl Send for ReleaseArray{}
unsafe impl Sync for ReleaseArray{}
//资源接口
pub trait ResTrait: Release {
    type Key: Hash + Eq + Clone + 'static;
	// 获得资源的唯一名称
	fn name(&self) -> &Self::Key;
	// 判断是否存活
	//fn is_alive(&self) -> bool;
	// 创建资源, 如果异步，可以返回Result<Promise>
	//fn create(&mut self) -> bool;
}

pub trait Release: 'static {}

pub struct Res<T: ResTrait>{
    timeout: u32,
    pub value: Share<T>,
}

impl<T: ResTrait> Res<T> {
    pub fn new(timeout: u32, value: Share<T>) -> Res<T>{
        Res {
            timeout,
            value
        }
    }
}

impl<T: ResTrait> Clone for Res<T> {
    fn clone(&self) -> Self {
        Self{
            timeout: self.timeout,
            value: self.value.clone(),
        }
    }
}

impl<T: ResTrait> Deref for Res<T> {
    type Target = T;
    fn deref(&self) -> &T{
        &self.value
    }
}

fn timeout_release(timeout: usize){   
    println!("timeout: {}", timeout);
    unsafe { TIMER_REF = set_timeout(timeout, Box::new(|| {
        let mut list = RELEASE_ARRAY.0.borrow_mut();
        let now = now_time();
        let mut len = list.len();
        let mut i = 0;
        TIMER_TIME = std::u64::MAX;
        loop {
            if i < len {
                let timeout = list[i].1;
                if timeout <= now {
                    if timeout < TIMER_TIME {
                        TIMER_TIME = timeout;
                    }
                    list.swap_remove(i); 
                    len -= 1;
                } else {
                    i += 1;
                }
            } else {
                break;
            }
        }
        if len > 0 {
            timeout_release((TIMER_TIME - now) as usize);
        }
    }))};
}

impl<T: ResTrait> Drop for Res<T> {
    fn drop(&mut self) {
        if Share::strong_count(&self.value) == 1 {
            let r = self.value.clone();
            let now = now_time();
            let release_point = now + (self.timeout as u64);
            RELEASE_ARRAY.0.borrow_mut().push((r, release_point));
            unsafe { if release_point < TIMER_TIME {
                TIMER_TIME = release_point;
                if TIMER_REF != 0 {
                    cancel_timeout(TIMER_REF);
                }
                timeout_release(self.timeout as usize);
            }}
        }
    }
}

unsafe impl<T: ResTrait> Sync for Res<T> {}
unsafe impl<T: ResTrait> Send for Res<T> {}

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

    pub fn get<T: ResTrait>(&self, key: &<T as ResTrait>::Key) -> Option<Res<T>>{
        match self.tables.get(&TypeId::of::<T>()) {
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
        self.tables.entry(TypeId::of::<T>()).or_insert(Share::new(ResMap::<T>::new())).clone().downcast::<ResMap<T>>().unwrap().create(value, self.timeout)
    }
}

//资源表
pub struct ResMap<T: ResTrait> (FnvHashMap<<T as ResTrait>::Key, (Share<T>, u32)>);

impl<T:ResTrait> ResMap<T> {
    pub fn new() -> ResMap<T>{
        ResMap(FnvHashMap::with_capacity_and_hasher(0, Default::default()))
    }
	// 获得指定键的资源
	pub fn get(&self, name: &<T as ResTrait>::Key) -> Option<Res<T>> {
        if let Some(v) = self.0.get(name) {
            return Some(Res{
                timeout: v.1,
                value: v.0.clone(),
            });
        }
        None
    }
	// 创建资源
	pub fn create(&self, res: T, timeout: u32) -> Res<T> {
        let name = res.name().clone();
        let r = Share::new(res);
        unsafe{&mut *(self as *const Self as usize as *mut Self)}.0.insert(name, (r.clone(), 0));
        Res{
            timeout,
            value: r,
        }
        // match self.0.entry(res.name()) {
        //     Entry::Occupied(mut e) => {
        //         let v = e.get_mut();
        //         match v.upgrade() {
        //             Some(r) => r,
        //             None =>{
        //                 res.create();
        //                 let r = Share::new(res);
        //                 swap(&mut Share::downgrade(&r), v);
        //                 r
        //             }
        //         }
        //     },
        //     Entry::Vacant(e) => {
        //         res.create();
        //         let r = Share::new(res);
        //         e.insert(Share::downgrade(&r));
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