// 显卡资源管理器
use std::sync::{Arc, Weak};
use std::hash::Hash;

use fnv::FnvHashMap;

//资源接口
pub trait Res {
    type Key: Hash + Eq + Clone;
	// 获得资源的唯一名称
	fn name(&self) -> &Self::Key;
	// 判断是否存活
	//fn is_alive(&self) -> bool;
	// 创建资源, 如果异步，可以返回Result<Promise>
	//fn create(&mut self) -> bool;
}

//资源表
pub struct ResMap<T: Res> (FnvHashMap<<T as Res>::Key, Weak<T>>);

impl<T:Res> ResMap<T> {
    pub fn new() -> ResMap<T>{
        ResMap(FnvHashMap::with_capacity_and_hasher(0, Default::default()))
    }
	// 获得指定键的资源
	pub fn get(&self, name: &<T as Res>::Key) -> Option<Arc<T>> {
        if let Some(v) = self.0.get(name) {
            if let Some(r) = v.upgrade() {
                return Some(r)
            }
        }
        None
    }
	// 创建资源
	pub fn create(&mut self, res: T) -> Arc<T> {
        let name = res.name().clone();
        let r = Arc::new(res);
        self.0.insert(name, Arc::downgrade(&r));
        r
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