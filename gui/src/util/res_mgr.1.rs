// 显卡资源管理器
use share::Share;
use std::hash::Hash;
use std::any::{ TypeId, Any };
use std::ops::{ Deref };
use std::cell::RefCell;
use std::marker::PhantomData;

pub trait Timer {
    fn cancel_timeout(id: usize);
    fn now_time() -> u64;
    fn set_timeout(ms: usize, f: Box<dyn FnOnce()>) -> usize;
}



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
    type Key: Hash + Eq + Clone + Send + 'static + Sync;
	// 获得资源的唯一名称
	fn name(&self) -> &Self::Key;
	// 判断是否存活
	//fn is_alive(&self) -> bool;
	// 创建资源, 如果异步，可以返回Result<Promise>
	//fn create(&mut self) -> bool;
}

pub trait Release: Send + 'static + Sync {}

pub struct Res<R: ResTrait, T: Timer>{
    timeout: u32,
    pub value: Share<R>,
    marker: PhantomData<T>
}

impl<R: ResTrait, T: Timer> Res<R, T> {
    pub fn new(timeout: u32, value: Share<R>) -> Res<R, T>{
        Self {
            timeout,
            value,
            marker: PhantomData
        }
    }
}

impl<R: ResTrait, T: Timer> Clone for Res<R, T> {
    fn clone(&self) -> Self {
        Self{
            timeout: self.timeout,
            value: self.value.clone(),
            marker: PhantomData,
        }
    }
}

impl<R: ResTrait, T: Timer> Deref for Res<R, T> {
    type Target = R;
    fn deref(&self) -> &R{
        &self.value
    }
}

fn timeout_release<T: Timer>(timeout: usize){   
    unsafe { TIMER_REF = T::set_timeout(timeout, Box::new(|| {
        let mut list = RELEASE_ARRAY.0.borrow_mut();
        let now = T::now_time();
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

impl<R: ResTrait, T: Timer> Drop for Res<R, T> {
    fn drop(&mut self) {
        if Share::strong_count(&self.value) == 1 {
            let r = self.value.clone();
            let now = T::now_time();
            let release_point = now + (self.timeout as u64);
            RELEASE_ARRAY.0.borrow_mut().push((r, release_point));
            unsafe { if release_point < TIMER_TIME {
                TIMER_TIME = release_point;
                if TIMER_REF != 0 {
                    T::cancel_timeout(TIMER_REF);
                }
                timeout_release(self.timeout as usize);
            }}
        }
    }
}

pub struct ResMgr<T: Timer>{
    tables: FxHashMap32<TypeId, Share<dyn Any>>,
    pub timeout: u32,
}

impl<T: Timer> ResMgr<T> {
    pub fn new(timeout: u32) -> Self{
        ResMgr{
            timeout,
            tables: FxHashMap32::default(),
        }
    }

    pub fn get<R: ResTrait>(&self, key: &<R as ResTrait>::Key) -> Option<Res<R, T>>{
        match self.tables.get(&TypeId::of::<R>()) {
            Some(map) => {
                match map.clone().downcast::<ResMap<R, T>>() {
                    Ok(r) => r.get(key),
                    Err(_) => None
                }
            },
            None => None,
        }
    }

    pub fn create<R: ResTrait>(&mut self, value: R) -> Res<R, T>{
        self.tables.entry(TypeId::of::<R>()).or_insert(Share::new(ResMap::<R, T>::new())).clone().downcast::<ResMap<R, T>>().unwrap().create(value, self.timeout)
    }
}

//资源表
pub struct ResMap<R: ResTrait, T: Timer> (FxHashMap32<<R as ResTrait>::Key, (Share<R>, u32)>);

impl<R: ResTrait, T: Timer> ResMap<R, T> {
    pub fn new() -> ResMap<R, T>{
        ResMap(FxHashMap32::default())
    }
	// 获得指定键的资源
	pub fn get(&self, name: &<R as ResTrait>::Key) -> Option<Res<R, T>> {
        if let Some(v) = self.0.get(name) {
            return Some(Res{
                timeout: v.1,
                value: v.0.clone(),
                marker: PhantomData,
            });
        }
        None
    }
	// 创建资源
	pub fn create(&self, res: R, timeout: u32) -> Res<R, T> {
        let name = res.name().clone();
        let r = Share::new(res);
        unsafe{&mut *(self as *const Self as usize as *mut Self)}.0.insert(name, (r.clone(), 0));
        Res{
            timeout,
            value: r,
            marker: PhantomData,
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