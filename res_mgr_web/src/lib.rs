/// 曝露ResMgr接口， 使ResMgr能够被js调用
extern crate res;
extern crate share;
extern crate wasm_bindgen;

use std::cell::RefCell;
use wasm_bindgen::prelude::*;

use res::{ResMgr as ResMgrRaw, Res as ResRaw};

use share::Share;

#[wasm_bindgen(typescript_custom_section)]
const Res: &'static str = r#"
interface Res {
    destroy():void
}
"#;

#[wasm_bindgen]
extern "C" {
	fn alert(s: &str);
	#[wasm_bindgen(typescript_type = "Res")]
	pub type Res;
	#[wasm_bindgen(method)]
    fn destroy(this: &Res);
}

#[wasm_bindgen(module = "/js/res_utils.js")]
extern "C" {
	fn destroy_res(res: &Res);
}


/// 资源管理器
#[wasm_bindgen]
pub struct ResMgr {
	inner: Share<RefCell<ResMgrRaw>>,
}

impl ResMgr {
	pub fn get_inner(&self) -> &Share<RefCell<ResMgrRaw>>{
		&self.inner
	}

	pub fn get_inner_mut(&mut self) -> &mut Share<RefCell<ResMgrRaw>>{
		&mut self.inner
	}
}

/// 资源
#[allow(dead_code)]
#[wasm_bindgen]
pub struct ResRef {
	inner: Share<JsRes>,
	// inner: Share<dyn ResRaw<Key = usize>>,
}

#[wasm_bindgen]
impl ResRef {
	pub fn link(&self) -> wasm_bindgen::JsValue {
		// match self.inner.clone().downcast::<JsRes>() {
		// 	Ok(r) => r.0,
		// 	Err(_) => wasm_bindgen::JsValue::from(None),
		// }
		self.inner.0.clone()
	}
}

impl ResRef {
	pub fn new(inner: Share<JsRes>) -> Self {
		ResRef{inner}
	}
}

#[wasm_bindgen]
impl ResMgr {
	/// 创建资源管理器的实例
	#[wasm_bindgen(constructor)]
	pub fn new(total_capacity: usize) -> Self {
		let r = if total_capacity > 0 {
			ResMgrRaw::with_capacity(total_capacity)
		} else {
			ResMgrRaw::default()
		};
		Self{inner: Share::new(RefCell::new(r))}
	}
	// // 10 * 1024 * 1024,
	// // 		50 * 1024 * 1024,
	// // 		5 * 60000,
	// /// 创建一个资源， 如果资源已经存在，则会修改资源的配置
	// pub fn register_res(&mut self, ty: usize, min_capacity: usize, max_capacity: usize, time_out: usize) {
	// 	self.inner.register::<JsRes>(min_capacity, max_capacity, time_out, ty, "".to_string());
	// }

	// /// 创建一个资源， 如果资源已经存在，旧的资源将被覆盖
	// /// 如果创建的资源类型未注册，将崩溃
	// pub fn create_res(&mut self, ty: usize, key: usize, res: Res, cost: usize) -> ResRef {
	// 	ResRef{inner: self.inner.create(key, ty,JsRes(res), cost, )}
	// }

	// /// 获取资源
	// pub fn get_res(&self, ty: usize, key: usize) -> Option<ResRef> {
	// 	match self.inner.get(&key, ty) {
	// 		Some(r) => Some(ResRef{inner: r}),
	// 		None => None
	// 	}
	// }

	/// 整理方法， 将无人使用的资源放入到LruCache， 清理过时的资源
	/// 就是LruMgr有总内存上限， 按权重分给其下的LRU。 如果有LRU有空闲， 则会减少其max_size, 按权重提高那些满的LRU的max_size
	pub fn collect(&mut self, now: usize) {
		self.inner.borrow_mut().collect(now);
	}
}

// trait ResPack {
// 	// 10 * 1024 * 1024,
// 	// 		50 * 1024 * 1024,
// 	// 		5 * 60000,
// 	/// 创建一个资源， 如果资源已经存在，则会修改资源的配置
// 	fn register_res(&mut self, group: usize, min_capacity: usize, max_capacity: usize, time_out: usize);

// 	/// 创建一个资源， 如果资源已经存在，旧的资源将被覆盖
// 	/// 如果创建的资源类型未注册，将崩溃
// 	fn create_res(&mut self, group: usize, key: usize, res: Res, cost: usize) -> Self;

// 	/// 获取资源
// 	fn get_res(&self, ty: usize, key: usize) -> Option<Self>;
// }
/// 资源包装
pub trait ResPack {
	fn register_to_resmgr(mgr: &mut ResMgr,ty: usize, min_capacity: usize, max_capacity: usize, time_out: usize);
	/// 创建一个资源， 如果资源已经存在，旧的资源将被覆盖
	/// 如果创建的资源类型未注册，将崩溃
	fn create_res(self, mgr: &mut ResMgr,ty: usize, key: usize, cost: usize) -> ResRef;

	/// 获取资源
	fn get_res(mgr: &ResMgr,ty: usize, key: usize) -> Option<ResRef>;
}

#[wasm_bindgen]
/// 资源包装
pub struct JsRes (Res);

unsafe impl Sync for JsRes {}
unsafe impl Send for JsRes {}

#[wasm_bindgen]
impl JsRes {
	pub fn new(res: Res) -> Self {
		JsRes(res)
	}
	// 10 * 1024 * 1024,
	// 		50 * 1024 * 1024,
	// 		5 * 60000,
	/// 创建一个资源， 如果资源已经存在，则会修改资源的配置
	pub fn register_to_resmgr(mgr: &mut ResMgr, ty: usize, min_capacity: usize, max_capacity: usize, time_out: usize) {
		mgr.inner.borrow_mut().register::<JsRes>(min_capacity, max_capacity, time_out, ty, "".to_string());
	}

	/// 创建一个资源， 如果资源已经存在，旧的资源将被覆盖
	/// 如果创建的资源类型未注册，将崩溃
	pub fn create_res(self, mgr: &mut ResMgr, ty: usize, key: usize, cost: usize) -> ResRef {
		ResRef{inner: mgr.inner.borrow_mut().create::<JsRes>(key, ty,self, cost, )}
	}

	/// 获取资源
	pub fn get_res(mgr: &ResMgr, ty: usize, key: usize) -> Option<ResRef> {
		// return None;
		match mgr.inner.borrow().get::<JsRes>(&key, ty) {
			Some(r) => Some(ResRef{inner: r}),
			None => None
		}
	}
}

#[wasm_bindgen]
pub struct TypeId (pub usize);

// impl ResPack for JsRes {
// 	fn inner(self) -> dyn ResRaw<Key=usize> {
//         self
//     }
// }

impl std::ops::Drop for JsRes {
	fn drop(&mut self) {
		destroy_res(&self.0);
        // self.0.destroy();
    }
}

impl ResRaw for JsRes {
    type Key = usize;
}
