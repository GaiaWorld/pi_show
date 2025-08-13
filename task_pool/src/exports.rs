use std::mem::transmute;
use std::num::NonZeroU32;

use pi_slot_wheel::TimerKey;
use slotmap::{KeyData, SlotMap};
use pi_weight_task::{Deque as Deque1, DequeKey, TaskPool as TaskPool1, WeightType as WeightType1};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use slotmap::Key;


/// 队列权重类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub enum WeightType {
    /// 标准权重
    Normal = 0,
    /// 单位权重，总权重为队列长度*单位权重
    Unit = 1,
}


#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct TaskPool {
	pool: TaskPool1<DequeKey, (), 128, 60, 2>,
	slot_map: SlotMap<DequeKey, TaskType>,
}

enum TaskType {
	Other,
	CancelTimer(TimerKey),
	// Timer,
	// Deque(DequeKey/** 队列id */, DequeKey),
	// Async,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl TaskPool {
	/// 创建定时器
	pub fn new() -> Self {
		Self {
			pool: TaskPool1::<DequeKey, (), 128, 60, 2>::default(),
			slot_map: Default::default()
		}
	}

	/// 创建串行任务队, 并加入任务池， 返回队列id
	pub fn create_deque(&mut self, weight_type: WeightType, weight: u32) -> f64 {
		let weight_type = to_deque_weight(weight_type, weight);
        let deque = Deque1::new(weight_type, ());
		let key = self.pool.push_deque(deque);
		to_f64(key)
	}

	/// 修复队列状态
	pub fn repair_deque_state(&mut self, key: f64) {
		let key = to_key(key);
		if let Some(deque) = self.pool.get_deque(key) {
			let state = deque.state(); // 取到队列状态
			self.pool.repair_deque_state(key, state);
		}		
	}

	/// 重设置队列的权重
	pub fn reset_deque_weight(&mut self, key: f64, weight_type: WeightType, weight: u32) {
		let key = to_key(key);
		let weight_type = to_deque_weight(weight_type, weight);
		self.pool.reset_deque_weight(key, weight_type);

	}

	/// 队列长度
	pub fn deque_len(&self, key: f64) -> Option<u32> {
		self.pool.get_deque(to_key(key)).map(|r| { r.deque.len() as u32 })
	}

	/// 释放队列的锁，成功释放，则返回true， 否则返回false
    pub fn deque_unlock(&mut self, key: f64) -> bool {
		self.pool.deque_unlock(to_key(key))
	}

	/// 删除一个任务队列，如果删除成功，返回true， 否则返回false
    pub fn remove_deque(&mut self, key: f64) -> bool {
		self.pool.remove_deque(to_key(key))
	}

	/// 插入一个指定任务权重的并行任务
    pub fn push_deque_task(&mut self, id: f64) -> Option<f64> {
		let id = to_key(id);
		if let Some(deque) = self.pool.get_deque(id) { // 这里不使用可变， 是为了让队列状态中的旧长度不更新
			let ptr_usize = deque as *const Deque1<DequeKey, ()> as usize;
			fn ptr(ptr: usize) -> *mut Deque1<DequeKey, ()> {
				ptr as *mut Deque1<DequeKey, ()>
			}
			let deque = unsafe {&mut *ptr(ptr_usize)};
			
			let key = self.slot_map.insert(TaskType::Other);
			deque.deque.push_back(key);
			return Some(to_f64(key))
		}
		None
	}

	/// 更新队列旧的长度
	pub fn update_deque_old_len(&mut self, id: f64) {
		let id = to_key(id);
		if let Some(_deque) = self.pool.get_deque_mut(id) { // 仅仅为了更新旧的长度
			// return Some(TaskState(deque.state()))
		}
		// None
	}

	/// 插入一个指定任务权重的并行任务
    pub fn push_async(&mut self, weight: u32) -> f64 {
		let key = self.slot_map.insert(TaskType::Other);
		self.pool.push_async(key, weight);
		to_f64(key)
	}

	/// push一个可取消的定时任务
	pub fn push_cancel_timer(&mut self, mut timeout: f64) -> f64 {
		if timeout < 0.0 {
			timeout = 0.0;
		}
		let key: DequeKey = self.slot_map.insert(TaskType::Other);
		let timer_key = self.pool.get_cancel_timer_mut().push_time(timeout as u64, key);
		self.slot_map[key] = TaskType::CancelTimer(timer_key);
		to_f64(key)
	}

	/// 取到可取消定时器的滚动次数
	pub fn roll_count(&mut self) -> f64 {
		self.pool.get_cancel_timer_mut().roll_count() as f64

	}

	/// 删除一个定时任务
	pub fn delete_cancel_timer(&mut self, key: f64) {
		let key = to_key(key);
		if let Some(TaskType::CancelTimer(timer_key)) = self.slot_map.remove(key) {
			self.pool.get_cancel_timer_mut().cancel(timer_key);
		}
	}

	/// push一个不可取消的定时任务
	pub fn push_timer(&mut self, mut timeout: f64) -> f64 {
		if timeout < 0.0 {
			timeout = 0.0;
		}
		let key: DequeKey = self.slot_map.insert(TaskType::Other);
		self.pool.get_timer_mut().push_time(timeout as u64, key);
		to_f64(key)
	}

	/// 弹出一个任务
	pub fn pop(&mut self, now: u32) -> Option<u32> {
		let task = self.pool.pop(now as u64);
		match task.0 {
			Some(r) => {
				self.slot_map.remove(r);
				Some(to_index(r))
			},
			_ => None
		}
	}

	/// 弹出一个任务（忽略定时器的任务）
	pub fn pop_ignore_timer(&mut self) -> Option<u32> {
		let task = self.pool.pop_ignore_timer();
		match task.0 {
			Some(r) => {
				self.slot_map.remove(r);
				Some(to_index(r))
			},
			_ => None
		}
	}
}

#[inline]
fn to_f64(r: DequeKey) -> f64 {
	let data = r.data();
	unsafe { transmute(data.as_ffi()) }
}

#[inline]
fn to_index(r: DequeKey) -> u32 {
	let data = r.data().as_ffi();
	(data << 32 >> 32) as u32
}

#[inline]
fn to_key(r: f64) -> DequeKey {
	DequeKey::from(KeyData::from_ffi(unsafe { transmute::<_, u64>(r) }))
}

#[inline]
fn to_deque_weight(weight_type: WeightType, mut weight: u32 ) -> WeightType1 {
	if weight == 0 {
		weight = 1;
	}
	let weight = unsafe { NonZeroU32::new_unchecked(weight) };
	match weight_type {
		WeightType::Normal => WeightType1::Normal( weight),
		WeightType::Unit => WeightType1::Normal(weight),
	}
}


