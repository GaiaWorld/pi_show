use std::{cell::RefCell, sync::Arc};

use pi_assets::{allocator::Allocator, asset::{Asset, Handle, Size}, homogeneous::HomogeneousMgr as HomogeneousMgr1, mgr::AssetMgr};
use pi_share::Share;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use js_sys::Function;
use std::mem::transmute;

pub static mut DESTROY_RES: Option<Arc<dyn Fn(f64) + Send + Sync>> = None;
pub struct CanSyncFunction(Function);
unsafe impl Sync for CanSyncFunction {}
unsafe impl Send for CanSyncFunction {}
impl CanSyncFunction {
	fn call(&self, v: &JsValue, v1: &JsValue) {
		self.0.call1(v, v1);
	}
}
#[wasm_bindgen]
pub fn set_destroy_callback(f: Function) {
	let f1 = CanSyncFunction(f);
	unsafe {DESTROY_RES = Some(Arc::new(move |value: f64| {
		f1.call(&JsValue::from_f64(0.0), &value.into());
	}))};
}


#[allow(dead_code)]
#[wasm_bindgen]

/// 资源包装
pub struct ResRef(Handle<JsRes>);

/// 资源管理器
#[wasm_bindgen]

pub struct ResMgr {
	inner: Share<AssetMgr<JsRes>>,
}

#[wasm_bindgen]

impl ResMgr {
	/// 创建一个资源， 如果资源已经存在，旧的资源将被覆盖
	/// 如果创建的资源类型未注册，将崩溃
	
	pub fn create_res(&mut self, key: f64, cost: u32) -> ResRef {
		match self.inner.insert(unsafe { transmute(key) }, JsRes {key: key, cost: cost as usize}) {
			Ok(r) => ResRef(r),
			_ => unreachable!()
		}
	}

	/// 获取资源
	
	pub fn get_res(&mut self, key: f64) -> Option<ResRef> {
		match self.inner.get(&unsafe { transmute(key) }) {
			Some(r) => Some(ResRef(r)),
			None => None
		}
	}
}

/// 同质资源管理器
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct HomogeneousMgr {
	inner: Share<HomogeneousMgr1<JsRes>>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl HomogeneousMgr {
	/// push资产
	pub fn push(&mut self, key: f64, cost: u32) {
		self.inner.push(JsRes {key, cost: 0 as usize/*TODO*/});
	}

	/// 弹出资产
	pub fn pop(&mut self) -> Option<f64> {
		self.inner.pop().map(|r| {r.key})
	}
}

/// 资源管理器
#[wasm_bindgen]

pub struct ResAllocator {
	inner: Share<RefCell<Allocator>>,
}

impl ResAllocator {
	pub fn get_inner(&self) -> &Share<RefCell<Allocator>>{
		&self.inner
	}

	pub fn get_inner_mut(&mut self) -> &mut Share<RefCell<Allocator>>{
		&mut self.inner
	}
}

#[wasm_bindgen]

impl ResAllocator {
	/// 创建资源管理器的实例
	#[wasm_bindgen]
	
	pub fn new(total_capacity: u32) -> Self {
		let r = if total_capacity > 0 {
			Allocator::new(total_capacity as usize)
		} else {
			Allocator::new(16 * 1024 * 1024)
		};
		Self{inner: Share::new(RefCell::new(r))}
	}

	/// 整理方法， 将无人使用的资源放入到LruCache， 清理过时的资源
	/// 就是LruMgr有总内存上限， 按权重分给其下的LRU。 如果有LRU有空闲， 则会减少其max_size, 按权重提高那些满的LRU的max_size
	
	pub fn collect(&mut self) {
		self.inner.borrow_mut().collect(pi_time::now_millisecond());
	}

	// 10 * 1024 * 1024,
	// 		50 * 1024 * 1024,
	// 		5 * 60000,
	/// 注册资产管理器
	pub fn register_to_resmgr(&mut self, ty: u32, min_capacity: u32, weight: u32, time_out: u32) -> ResMgr {
		let mut m = pi_assets::mgr::AssetMgr::<JsRes>::new(pi_assets::asset::GarbageEmpty(),
		false,
		min_capacity as usize,
		time_out as usize,);
		let m1 = Share::get_mut(&mut m).unwrap();
		m1.ty = ty; // 标记类型
		self.inner.borrow_mut().register(m.clone(), min_capacity as usize, weight as usize);
		ResMgr{
			inner: m,
		}
	}

	/// 注册同质资产管理器
	pub fn register_to_homogeneous_resmgr(&mut self, ty: u32, min_capacity: u32, weight: u32, time_out: u32) -> HomogeneousMgr {
		let mut m = HomogeneousMgr1::<JsRes>::new(pi_assets::homogeneous::GarbageEmpty(),
		min_capacity as usize,
		time_out as usize,);
		let m1 = Share::get_mut(&mut m).unwrap();
		m1.ty = ty as usize; // 标记类型
		self.inner.borrow_mut().register(m.clone(), min_capacity as usize, weight as usize);
		HomogeneousMgr{
			inner: m,
		}
	}

	// 资产管理器总大小
	pub fn size(&self) -> f32 {
		let account = self.inner.borrow().account();
		account.cur_total_size
	}

	// 统计详情
	pub fn account_details(&self) -> String {
		let account = self.inner.borrow().account();
		serde_json::to_string(&account).unwrap()
	}
}

/// 资源包装
pub struct JsRes {
	key: f64,
	cost: usize,
}

impl Asset for JsRes {
    type Key = u64;
}

impl Size for JsRes {
	/// 资产的大小
	fn size(&self) -> usize {
		self.cost
	}
}

impl std::ops::Drop for JsRes {
	fn drop(&mut self) {
		unsafe { 
			if let Some(r) = &DESTROY_RES {
				(r)(self.key)
			};
		}
    }
}