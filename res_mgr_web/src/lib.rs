/// 曝露ResMgr接口， 使ResMgr能够被js调用
extern crate res;
extern crate share;
extern crate wasm_bindgen;

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

/// 资源管理器
#[wasm_bindgen]
pub struct ResMgr {
	inner: ResMgrRaw,
}

/// 资源
#[allow(dead_code)]
#[wasm_bindgen]
pub struct ResRef {
	inner: Share<JsRes>,
}

#[wasm_bindgen]
impl ResRef {
	pub fn link(&self) -> wasm_bindgen::JsValue {
		self.inner.0.clone()
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
		Self{inner: r}
	}
	// 10 * 1024 * 1024,
	// 		50 * 1024 * 1024,
	// 		5 * 60000,
	/// 创建一个资源， 如果资源已经存在，程序会崩溃
	pub fn register_res(&mut self, ty: usize, min_capacity: usize, max_capacity: usize, time_out: usize) {
		self.inner.register::<JsRes>(min_capacity, max_capacity, time_out, ty, "".to_string());
	}

	/// 创建一个资源， 如果资源已经存在，程序会崩溃
	pub fn create_res(&mut self, ty: usize, key: usize, res: Res, cost: usize) -> ResRef {
		ResRef{inner: self.inner.create(key, ty,JsRes(res), cost, )}
	}

	/// 获取资源
	pub fn get_res(&self, ty: usize, key: usize) -> Option<ResRef> {
		match self.inner.get(&key, ty) {
			Some(r) => Some(ResRef{inner: r}),
			None => None
		}
	}

	/// 整理方法， 将无人使用的资源放入到LruCache， 清理过时的资源
	/// 就是LruMgr有总内存上限， 按权重分给其下的LRU。 如果有LRU有空闲， 则会减少其max_size, 按权重提高那些满的LRU的max_size
	pub fn collect(&mut self, now: usize) {
		self.inner.collect(now);
	}
}

struct JsRes (Res);

impl std::ops::Drop for JsRes {
	fn drop(&mut self) {
        self.0.destroy();
    }
}
impl ResRaw for JsRes {
    type Key = usize;
}

pub fn main() {
	alert(&format!("Hello!"));
}