//zindex系统
// zindex的min max, 设计分配如下： 如果父容器为 0 100.
//  子节点为1个的话：1 100. 为2个的话： 1 51, 51 100. 为3个的话： 1 34, 34 67, 67 100.


use std::{
  f32,
  cmp::{Ordering},
};

use map::{vecmap::VecMap};
use heap::simple_heap::SimpleHeap;
use dirty::LayerDirty;

use ecs::{
  system::{Runner, MultiCaseListener, SingleCaseListener, EntityListener},
  monitor::{CreateEvent, DeleteEvent, ModifyEvent, NotifyImpl},
  component::MultiCaseImpl,
  single::SingleCaseImpl,
};

use idtree::Node as IdNode;
use single::{DirtyList, IdTree};
use entity::{Node};
use component::{
  user::{ZIndex as ZI},
  calc::{ZDepth, ZDepthWrite, NodeState},
};
use util::vecmap_default::VecMapWithDefault;
use map::Map;
use Z_MAX;
use ROOT;

impl<'a> EntityListener<'a, Node, CreateEvent> for ZIndexImpl {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, ZI>, &'a mut MultiCaseImpl<Node, ZDepth>);

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
		// 为root节点设置最大范围值
		if event.id == ROOT {
			let zi = &mut self.map[event.id];
			zi.pre_min_z = -Z_MAX;
			zi.pre_max_z = Z_MAX;
			zi.min_z = -Z_MAX;
			zi.max_z = Z_MAX;
		} else {
			self.map.insert(event.id, ZIndex::default());
		}
		//   self.map.insert(event.id, zi);
		//   write.0.insert(event.id, ZI::default());
		//   write.1.insert(event.id, ZDepth::default());
    }
}

// 监听节点树的删除，重置empty_min_z
impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for ZIndexImpl {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();

    fn listen(&mut self, event: &DeleteEvent, idtree: Self::ReadData, _write: Self::WriteData) {
		let id = event.id;
		let node = &idtree[id];
		let parent = node.parent();
		if parent > 0 {
			let tail = idtree[parent].children().tail;
			if tail == id {
				let prev = node.prev();
				if prev > 0 {
					// let max_z = ;
					self.map[parent].empty_min_z = self.map[prev].pre_max_z;
				} else {
					self.map[parent].empty_min_z = self.map[parent].pre_max_z + 1.0;
				}
			}
		}
    }
}

impl<'a> MultiCaseListener<'a, Node, ZI, ModifyEvent> for ZIndexImpl {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, ZI>);
    type WriteData = ();

    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData) {
		self.modifyz(event.id, read);
    }
}

impl<'a> MultiCaseListener<'a, Node, ZI, CreateEvent> for ZIndexImpl {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, ZI>);
    type WriteData = ();

    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, _write: Self::WriteData) {
      self.modifyz(event.id, read);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for ZIndexImpl {
    type ReadData = (&'a SingleCaseImpl<IdTree>,&'a MultiCaseImpl<Node, ZI>, &'a MultiCaseImpl<Node, NodeState>);
    type WriteData = ();

    fn listen(&mut self, event: &CreateEvent, (idtree, zindexs, node_states): Self::ReadData, _write: Self::WriteData) {
		let id = event.id;
		let node = &idtree[id];

		// if let Some(r) = self.map.get_mut(&id) {
		// 	r.pre_min_z = Z_MAX;
		// 	r.pre_max_z = Z_MAX;
		// }

		let zi = &mut self.map[id];
		if zi.old != AUTO {
			// 设置自己成强制脏
			// debug_println!("Recursive==================={}", id);
			zi.dirty = DirtyType::Recursive;
			self.dirty.mark(id, node.layer());
		} else {
			// 设置自己所有非AUTO的子节点为强制脏
			recursive_dirty(&mut self.map, node_states, &mut self.dirty, idtree, node.children().head);
		}

		if zindexs[id].0 == 0 {
			self.set_parent_dirty_width_empty(node, idtree);
		} else {
			self.set_parent_dirty(node.parent(), idtree);
		}
    }
}

impl<'a> Runner<'a> for ZIndexImpl {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, NodeState>);
    type WriteData = &'a mut MultiCaseImpl<Node, ZDepth>;

    fn setup(&mut self, _read: Self::ReadData, _write: Self::WriteData) {
    }
    fn run(&mut self, (idtree, node_states): Self::ReadData, write: Self::WriteData) {
		// if (read.1).0.len() > 0 {
		// 	set_print(true);
		// } else {
		// 	set_print(false);
		// }
		self.calc(idtree, write, node_states)
    }
    fn dispose(&mut self, _read: Self::ReadData, _write: Self::WriteData) {

    }
}


const AUTO: isize = -1;
#[derive(Debug, Clone, PartialEq, EnumDefault)]
pub enum DirtyType {
  None, // 不脏，跳过
  Normal, // 常规脏，计算自己
  Recursive, // 递归脏，计算所有的子节点
  Empty, // 空余z空间中增加节点
}

#[derive(Debug, Clone)]
pub struct ZIndex {
  pub dirty: DirtyType, // 子节点设zindex时，将不是auto的父节点设脏
  pub old: isize, // 旧值
  pub pre_min_z: f32, // 预设置的节点的最小z值
  pub pre_max_z: f32, // 预设置的节点的最大z值
  pub min_z: f32, // 节点的最小z值，也是节点自身的z值
  pub max_z: f32, // 节点的最大z值，z-index == -1, 则和min_z一样。
  pub empty_min_z: f32, // 剩余空间得最小z值
  pub max_child_count: usize, // 曾经添加过的最大子节点数量
}

impl Default for ZIndex {
	fn default() -> Self {
		Self {
			dirty: DirtyType::None, // 子节点设zindex时，将不是auto的父节点设脏
			old: 0, // 旧值
			pre_min_z: Z_MAX, // 预设置的节点的最小z值
			pre_max_z: Z_MAX, // 预设置的节点的最大z值
			min_z: Z_MAX, // 节点的最小z值，也是节点自身的z值
			max_z: Z_MAX, // 节点的最大z值，z-index == -1, 则和min_z一样。
			empty_min_z: Z_MAX, // 剩余空间得最小z值
			max_child_count: 0, // 曾经添加过的最大子节点数量
		}
	}
}

pub struct ZIndexImpl {
  dirty: LayerDirty,
  map: VecMapWithDefault<ZIndex>,
  cache: Cache,
}

impl ZIndexImpl {
  pub fn new() -> ZIndexImpl {
    ZIndexImpl {
      dirty: LayerDirty::default(),
      map: VecMapWithDefault::default(),
      cache: Cache::new(),
    }
  }

  pub fn with_capacity(capacity: usize) -> ZIndexImpl {
    ZIndexImpl {
      dirty: LayerDirty::default(),
      map: VecMapWithDefault::with_capacity(capacity),
      cache: Cache::new(),
    }
  }

  fn modifyz(&mut self, id: usize, read: (&SingleCaseImpl<IdTree>, &MultiCaseImpl<Node, ZI>)) {
	let z = *read.1[id];
	let zi = &mut self.map[id];
	let old = zi.old;
	zi.old = z;
	debug_println!("zindex modify， id： {}， z: {}", id, z);
	let node = &read.0[id];
	if node.layer() == 0 {
	return;
	}
	if old == AUTO {
	if zi.dirty == DirtyType::None {
		// 如果zindex由auto变成有值，则产生新的堆叠上下文，则自身需要设脏。
		zi.dirty = DirtyType::Normal;
		self.dirty.mark(id, node.layer());
	}
	} else if z == AUTO {
	// 为了防止adjust的auto跳出，提前设置为false
	zi.dirty = DirtyType::None;
	}
	self.set_parent_dirty(node.parent(), &read.0);
}

  // 设置节点对应堆叠上下文的节点脏
  fn set_parent_dirty(&mut self, mut id: usize, idtree: &IdTree) {
    while id > 0 {
      let zi = &mut self.map[id];
      let node = &idtree[id];
      // 如果为z为auto，则向上找zindex不为auto的节点，zindex不为auto的节点有堆叠上下文
      if zi.old != AUTO {
        if zi.dirty == DirtyType::None {
		//   zi.dirty = DirtyType::Normal; // bug, 等待修复， TODO
		  zi.dirty = DirtyType::Recursive;
          self.dirty.mark(id, node.layer());
          debug_println!("zindex- set_parent_dirty: {:?} {:?} {:?} {:?} {:?} {:?}", id, zi, node.parent(), node.layer(), node.count(), node.children().head);
        }
        if (node.count() as f32) < (zi.pre_max_z - zi.pre_min_z) {
          return;
        }
        // 如果z范围超过自身全部子节点及其下子节点数量，则继续向上设置脏，等calc_z调整以获得足够的z范围
      }
      id = node.parent();
    }
  }

//   // 设置节点对应堆叠上下文的节点脏
//   fn set_parent_dirty_width_empty<'a>(&mut self, mut node: &'a IdNode<usize>, idtree: &'a IdTree) {
// 	let parent = node.parent();
//     if parent > 0 {
// 		let prev = node.prev();
// 		let mut pre_max_z = 0.0;
// 		if prev > 0 {
// 			pre_max_z = self.map[prev].pre_max_z;
// 		}
// 		let zi = &mut self.map[parent];
		
// 		node = &idtree[parent];
//       // 如果为z为auto，则向上找zindex不为auto的节点，zindex不为auto的节点有堆叠上下文
//       if zi.old != AUTO {
//         if zi.dirty == DirtyType::None {
// 			// 当前节点前面的节点存在，并且其最大的z大于等于父节点空余z空间的最小值，则当前节点应该添加到空余z空间中
// 			debug_println!("add,zi: {:?}, parent: {}", zi, parent);
// 			if prev == 0 || pre_max_z >= zi.empty_min_z  {
// 				zi.dirty = DirtyType::Empty;
// 			} else {
// 				zi.dirty = DirtyType::Normal;
// 			}
// 			self.dirty.mark(parent, node.layer());
// 			//debug_println!("zindex- set_parent_dirty: {:?} {:?} {:?} {:?} {:?} {:?}", id, zi, node.parent, node.layer, node.count, node.children.head);
// 		} else if zi.dirty == DirtyType::Empty && !(prev == 0 || pre_max_z >= zi.empty_min_z) {
// 			zi.dirty = DirtyType::Normal;
// 		}
// 		// 如果z范围超过自身全部子节点及其下子节点数量，则继续向上设置脏，等calc_z调整以获得足够的z范围
//         if (node.count() as f32) < zi.pre_max_z - zi.pre_min_z {
//           return;
// 		}
// 		return self.set_parent_dirty(node.parent(), idtree);
// 	  }
//     }
//   }

  // 设置节点对应堆叠上下文的节点脏
  fn set_parent_dirty_width_empty<'a>(&mut self, mut node: &'a IdNode<usize>, idtree: &'a IdTree) {
	let parent = node.parent();
    if parent > 0 {
		let zi = &mut self.map[parent];
		
		node = &idtree[parent];
      // 如果为z为auto，则向上找zindex不为auto的节点，zindex不为auto的节点有堆叠上下文
      if zi.old != AUTO {
        if zi.dirty == DirtyType::None {
			// 当前节点前面的节点存在，并且其最大的z大于等于父节点空余z空间的最小值，则当前节点应该添加到空余z空间中
			debug_println!("add,zi: {:?}, parent: {}", zi, parent);
			zi.dirty = DirtyType::Normal;
			self.dirty.mark(parent, node.layer());
			//debug_println!("zindex- set_parent_dirty: {:?} {:?} {:?} {:?} {:?} {:?}", id, zi, node.parent, node.layer, node.count, node.children.head);
		} else /*if zi.dirty == DirtyType::Empty && !(prev == 0 || pre_max_z >= zi.empty_min_z) */{
			zi.dirty = DirtyType::Normal;
		}
		// 如果z范围超过自身全部子节点及其下子节点数量，则继续向上设置脏，等calc_z调整以获得足够的z范围
        if (node.count() as f32) < zi.pre_max_z - zi.pre_min_z {
          return;
		}
		return self.set_parent_dirty(node.parent(), idtree);
	  }
    }
  }

  // 整理方法
  fn calc(&mut self, idtree: &IdTree, zdepth: &mut MultiCaseImpl<Node, ZDepth>, node_states: &MultiCaseImpl<Node, NodeState>) {
    for (id, layer) in self.dirty.iter() {
      let (min_z, max_z, old_empty_min_z, dirty) = {
        let zi = &mut self.map[*id];
        debug_println!("calc, id: {:?} zi: {:?}", id, zi);
        if zi.dirty == DirtyType::None {
          continue;
		}
		
        let dirty = zi.dirty.clone();
        zi.dirty = DirtyType::None;
        zi.min_z = zi.pre_min_z;
        zi.max_z = zi.pre_max_z;
        (zi.min_z, zi.max_z, zi.empty_min_z, dirty)
      };
		let node = match idtree.get(*id) {
			Some(r) => if r.layer() == layer {r} else {continue},
			None => continue,
		};
      // 设置 z_depth, 其他系统会监听该值
      unsafe{zdepth.get_unchecked_write(*id)}.set_0(min_z);
      //debug_println!("zindex- calc: {:?} {:?} {:?} {:?}", id, min_z, max_z, normal);
      if node.count() == 0 {
		self.map[*id].empty_min_z = min_z + 1.;
        continue;
	  }
	  
	  let max_z_z = match dirty {
		  DirtyType::Normal => {
			self.cache.sort(zdepth, node_states, &self.map, idtree, node.children().head, 0);
			debug_println!("Normal calc, id:{:?}, min_z:{:?}, max_z: {:?}", id, min_z, max_z);
			self.cache.calc(&mut self.map, idtree, zdepth, min_z, max_z, node.count(), node_states)
		  },
		  DirtyType::Recursive => {
			self.cache.sort(zdepth, node_states, &self.map, idtree, node.children().head, 0);
			debug_println!("recursive_calc, id:{:?}, min_z:{:?}, max_z: {:?}", id, min_z, max_z);
			self.cache.recursive_calc(node_states, &mut self.map, idtree, zdepth, min_z, max_z, node.count())
		  },
		  DirtyType::Empty => { // 
			if self.cache.node_heap.len() == 0 {
				let (count, arr) = self.cache.sort_width_empty(zdepth, node_states, &self.map, idtree, node.children().head, 0, old_empty_min_z);
				let empty_z = max_z - old_empty_min_z;
				debug_println!("calc_empty, id:{} count: {}, empty_z:{}, min_z:{}, max_z: {}", id, count, empty_z, min_z, max_z);
				if (count as f32) <= empty_z && empty_z <= max_z - min_z - 1.{ // 如果是空
					self.cache.calc_empty(&mut self.map, idtree, zdepth, old_empty_min_z, max_z, count, arr, node_states)
				} else {
					debug_println!("Empty1 calc, id:{:?}, min_z:{:?}, max_z: {:?}", id, min_z, max_z);
					self.cache.calc(&mut self.map, idtree, zdepth, min_z, max_z, node.count(), node_states)
				}
			} else {
				self.cache.sort(zdepth, node_states, &self.map, idtree, node.children().head, 0);
				debug_println!("Empty2 calc, id:{:?}, min_z:{:?}, max_z: {:?}", id, min_z, max_z);
				self.cache.calc(&mut self.map, idtree, zdepth, min_z, max_z, node.count(), node_states)
			}
		  },
		  DirtyType::None => continue,
	  };
	  self.map[*id].empty_min_z = max_z_z;
    }
    if self.dirty.count() > 0 {
      // 详细打印
      for (_id, n) in idtree.recursive_iter(1) {
        let mut v = String::new();
        for _ in 1..n.layer() {
          v.push('-')
        }
      }
    }
    self.dirty.clear();
  }

}

#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct ZSort (isize, usize, usize, usize); // (zindex, index, node_id, children_count)

// 计算z排序时使用的临时数据结构
struct Cache {
  node_heap: SimpleHeap<ZSort>,
  negative_heap: SimpleHeap<ZSort>,
  z_zero: Vec<ZSort>,
  z_auto: Vec<usize>,
  temp: Vec<(usize, f32, f32)>,
}
impl Cache {
  fn new() -> Cache {
    Cache {
      node_heap: SimpleHeap::new(Ordering::Less),
      negative_heap: SimpleHeap::new(Ordering::Less),
      z_zero: Vec::new(),
      z_auto: Vec::new(),
      temp: Vec::new(),
    }
  }

  // 循环计算子节点， 分类排序
  fn sort(&mut self, zdepth: &MultiCaseImpl<Node, ZDepth>, node_states: &MultiCaseImpl<Node, NodeState>, map: &VecMapWithDefault<ZIndex>, idtree: &IdTree, child: usize, mut order: usize) -> usize {
    // zindex为0或-1的不参与排序。 zindex排序。用heap排序，确定每个子节点的z范围。如果子节点的zindex==-1，则需要将其子节点纳入排序。
    for (id, n) in idtree.iter(child) {
		if !node_states[id].0.is_rnode() {
			continue;
		}

		let zi = &map[id];
		let zi = zi.old;
		if zi == 0 {
			self.z_zero.push(ZSort(zi, order, id, n.count()));
		}else if zi == -1 {
			self.z_auto.push(id);
			// 继续递归其子节点
			order = self.sort(zdepth, node_states, map, idtree, n.children().head, order);
		}else if zi > 0 {
			self.node_heap.push(ZSort(zi, order, id, n.count()));
		}else{
			self.negative_heap.push(ZSort(zi-1, order, id, n.count()));
		}
		order+=1;
    }
    order
  }

  // 循环计算子节点， 分类排序
  fn sort_width_empty(&mut self, zdepth: &MultiCaseImpl<Node, ZDepth>, node_states: &MultiCaseImpl<Node, NodeState>, map: &VecMapWithDefault<ZIndex>, idtree: &IdTree, child: usize, mut order: usize, empty_min_z: f32) -> (usize, Vec<ZSort>) {
	  let mut last_count = 0;
	  let mut arr = Vec::new();
    // zindex为0或-1的不参与排序。 zindex排序。用heap排序，确定每个子节点的z范围。如果子节点的zindex==-1，则需要将其子节点纳入排序。
    for (id, n) in idtree.iter(child) {
		if !node_states[id].0.is_rnode() {
			continue;
		}

		let z = &map[id];
		let zi = z.old;
		if zi == 0 {
			self.z_zero.push(ZSort(zi, order, id, n.count()));
		}else if zi == -1 {
			self.z_auto.push(id);
			// 继续递归其子节点
			order = self.sort(zdepth, node_states, map, idtree, n.children().head, order);
		}else if zi > 0 {
			self.node_heap.push(ZSort(zi, order, id, n.count()));
		}else{
			self.negative_heap.push(ZSort(zi-1, order, id, n.count()));
		}

		debug_println!("pre_max_z: {}, empty_min_z: {}, id:{}", z.pre_max_z, empty_min_z, id);
		if z.pre_max_z > empty_min_z {
			last_count += n.count() + 1;
			arr.push(ZSort(zi, order, id, n.count()));
		}
		order += 1;
    }
	(last_count, arr)
  }

  // 计算真正的z
  fn calc(&mut self, map: &mut VecMapWithDefault<ZIndex>, idtree: &IdTree, zdepth: &mut MultiCaseImpl<Node, ZDepth>, mut min_z: f32, mut max_z: f32, count: usize, node_states: &MultiCaseImpl<Node, NodeState>) -> f32 {
    min_z += 1.; // 第一个子节点的z，要在父节点z上加1
    let auto_len = self.z_auto.len();
    debug_println!("calc count--------------------------count: {}, auto_len: {}", count, auto_len);
    // 计算大致的劈分间距
    let split = if count > auto_len {
      (max_z - min_z - auto_len as f32) / (count - auto_len) as f32
    }else{
		1.
    };
    debug_println!("negative_heap: len: {:?}, value: {:?}", self.negative_heap.len(), self.negative_heap);
    while let Some(ZSort(_, _, n_id, c)) = self.negative_heap.pop() {
      max_z = min_z + split + split * c as f32;
      adjust(node_states, map, idtree, zdepth, n_id, &idtree[n_id], min_z, max_z, f32::NAN, 0.);
      min_z = max_z;
    }
    debug_println!("z_auto: len: {:?}, value: {:?}", self.z_auto.len(), self.z_auto);
    for n_id in &self.z_auto {
      adjust(node_states, map, idtree, zdepth, *n_id, &idtree[*n_id], min_z, min_z, f32::NAN, 0.);
      min_z += 1.;
    }
    self.z_auto.clear();
    debug_println!("z_zero: len: {:?}, value: {:?}", self.z_zero.len(), self.z_zero);
    for &ZSort(_, _, n_id, c) in &self.z_zero {
      max_z = min_z + split + split * c as f32;
      adjust(node_states,  map, idtree, zdepth, n_id, &idtree[n_id], min_z, max_z, f32::NAN, 0.);
      min_z = max_z;
    }
    self.z_zero.clear();
    debug_println!("z_node_heapzero: len: {:?}, value: {:?}", self.node_heap.len(), self.node_heap);
    while let Some(ZSort(_, _, n_id, c)) = self.node_heap.pop() {
      max_z = min_z + split + split * c as f32;
      adjust(node_states, map, idtree, zdepth, n_id, &idtree[n_id], min_z, max_z, f32::NAN, 0.);
      min_z = max_z;
	}
	max_z
  }

  fn calc_empty(&mut self, map: &mut VecMapWithDefault<ZIndex>, idtree: &IdTree, zdepth: &mut MultiCaseImpl<Node, ZDepth>, mut min_z: f32, mut max_z: f32, count: usize, arr: Vec<ZSort>, node_states: &MultiCaseImpl<Node, NodeState>) -> f32 {
	min_z += 1.;
    debug_println!("calc_empty--------------------------count: {}, arr:{:?}", count, arr);
	let capacity = max_z - min_z;
	let mut use_capacity = capacity * count as f32/(count as f32 + 8.0); // 
	if use_capacity < count as f32 {
		use_capacity = count as f32;
	}

	// 劈分间距
	let split = use_capacity/count as f32;
	debug_println!("calc_empty, min_z: {}, max_z:{}, use_capacity: {}, split: {}", min_z, max_z, use_capacity, split);
	// debug_println!("z_auto: len: {:?}, value: {:?}", self.z_auto.len(), self.z_auto);
	for &ZSort(_, _, n_id, c) in &arr {
		max_z = min_z + split + split * c as f32;
		adjust(node_states, map, idtree, zdepth, n_id, &idtree[n_id], min_z, max_z, f32::NAN, 0.);
		min_z = max_z;
	}

	self.z_auto.clear();
	self.z_zero.clear();
	self.node_heap.clear();
	self.negative_heap.clear();
	return max_z;
  }
// 计算真正的z
  fn recursive_calc(&mut self, node_states: &MultiCaseImpl<Node, NodeState>, map: &mut VecMapWithDefault<ZIndex>, idtree: &IdTree, zdepth: &mut MultiCaseImpl<Node, ZDepth>, mut min_z: f32, mut max_z: f32, count: usize) -> f32 {
    min_z += 1.; // 第一个子节点的z，要在父节点z上加1
    let auto_len = self.z_auto.len();
    // 计算大致的劈分间距
    let split = if count > auto_len {
      (max_z - min_z - auto_len as f32) / (count - auto_len) as f32
    }else{
		1.
	};
	
    let start = self.temp.len();
    while let Some(ZSort(_, _, n_id, c)) = self.negative_heap.pop() {
      max_z = min_z + split + split * c as f32;
      self.temp.push((n_id, min_z, max_z));
      min_z = max_z;
    }
    for n_id in &self.z_auto {
      self.temp.push((*n_id, min_z, min_z));
      min_z += 1.;
    }
    self.z_auto.clear();
    for &ZSort(_, _, n_id, c) in &self.z_zero {
	  max_z = min_z + split + split * c as f32;
      self.temp.push((n_id, min_z, max_z));
      min_z = max_z;
    }
    self.z_zero.clear();
    while let Some(ZSort(_, _, n_id, c)) = self.node_heap.pop() {
      max_z = min_z + split + split * c as f32;
      self.temp.push((n_id, min_z, max_z));
      min_z = max_z;
	}

    while start < self.temp.len() {
      let (id, min_z, max_z) = self.temp.pop().unwrap();
      let mut zi = &mut map[id];
      zi.dirty = DirtyType::None;
      zi.min_z = min_z;
      zi.pre_min_z = min_z;
      zi.max_z = max_z;
	  zi.pre_max_z = max_z;
      // 设置 z_depth, 其他系统会监听该值
      unsafe {zdepth.get_unchecked_write(id)}.set_0(min_z);
      //debug_println!("zindex- ----recursive_calc: {:?} {:?} {:?}", id, min_z, max_z);
      if min_z == max_z {
        continue
      }
      let node = &idtree[id];
      if node.count() == 0 {
		zi.empty_min_z = min_z + 1.;
        continue;
      }
      self.sort(zdepth, node_states, map, idtree, node.children().head, 0);
      debug_println!("recursive_calc, id:{:?}, min_z:{:?}, max_z: {:?}", id, min_z, max_z);
      map[id].empty_min_z = self.recursive_calc(node_states, map, idtree, zdepth, min_z, max_z, node.count());
	}
	max_z
  }
}
//================================ 内部静态方法
// 设置自己所有非AUTO的子节点为强制脏
fn recursive_dirty(map: &mut VecMapWithDefault<ZIndex>, node_states: &MultiCaseImpl<Node, NodeState>, dirty: &mut LayerDirty, idtree: &IdTree, child: usize) {
  for (id, n) in idtree.iter(child) {
	if !node_states[id].0.is_rnode() {
		continue;
	}
    let zi = &mut map[child];
    if zi.old == -1 {
      recursive_dirty(map, node_states, dirty, idtree, n.children().head);
    }else {
      zi.dirty = DirtyType::Recursive;
      dirty.mark(id, n.layer());
    }
  }
}

// 整理方法。z范围变小或相交，则重新扫描修改一次。两种情况。
// 1. 有min_z max_z，修改该节点，计算rate，递归调用。
// 2. 有min_z rate parent_min， 根据rate和新旧min, 计算新的min_z max_z。 要分辨是否为auto节点
fn adjust(node_states: &MultiCaseImpl<Node, NodeState>, map: &mut VecMapWithDefault<ZIndex>, idtree: &IdTree, zdepth: &mut MultiCaseImpl<Node, ZDepth>, id: usize, node: &IdNode<usize>, min_z: f32, max_z: f32, rate: f32, parent_min: f32) {
	if !node_states[id].0.is_rnode() {
		return;
	}
  let (min, r, old_min) = {
    let zi = &mut map[id];
    debug_println!("---------dirty adjust: {:?} {:?} {:?} {:?} {:?} pre_min_z:{}, pre_max_z:{}", id, min_z, max_z, rate, parent_min, zi.pre_min_z, zi.pre_max_z);
    let (min, max) = if !rate.is_nan(){
      (((zi.pre_min_z - parent_min) * rate) + min_z + 1., ((zi.pre_max_z - parent_min) * rate) + min_z + 1.)
    }else{
      (min_z, max_z)
    };
    zi.pre_min_z = min;
	zi.pre_max_z = max;
	
    // 如果节点脏，则跳过，后面会进行处理
    if zi.dirty != DirtyType::None{
		debug_println!("dirty adjust, id:{:?}, min_z:{:?}, max_z: {:?}", id, min, max);
      return
    }
    if max >= zi.max_z && min <= zi.min_z {
		debug_println!("点的z范围变大 adjust, id:{:?}, min_z:{:?}, max_z: {:?}", id, min, max);
      // 如果子节点的z范围变大，则可以不继续处理该子节点
      return;
    }
    let old_min_z = zi.min_z + 1.; // 以后用于算子节点的z，所以提前加1
    let old_max_z = zi.max_z;
    // 更新当前值
    zi.min_z = min;
	zi.max_z = max;
	zi.empty_min_z = max;
    // 设置 z_depth, 其他系统会监听该值
    unsafe { zdepth.get_unchecked_write(id) }.set_0(min);
    debug_println!("---------adjust: {:?} {:?} {:?}", id, min, max);
    
    // 判断是否为auto
    if min != max {
      debug_println!("xxx---------id: {:?} min_z: {:?} max_z: {:?}, old_min: {:?} old_max:{}, rate:{}", id, min_z, max_z, old_min_z, old_max_z, (max_z - min_z - 1.)/ (old_max_z - old_min_z));
      (min, (max - min - 1.)/ (old_max_z - old_min_z), old_min_z)
    }else if !rate.is_nan() {
      // 如果是auto，则重用min_z, rate, parent_min
      (min_z, rate, parent_min)
    }else{
		zi.empty_min_z = min_z + 1.;
      return
    }
  };
  //递归计算子节点的z
  for (i, n) in idtree.iter(node.children().head) {
    adjust(node_states, map, idtree, zdepth, i, n, min, 0., r, old_min);
  }
}

impl_system!{
    ZIndexImpl,
    true,
    {
        EntityListener<Node, CreateEvent>
        // EntityListener<Node, DeleteEvent>
		MultiCaseListener<Node, ZI, ModifyEvent>
		MultiCaseListener<Node, ZI, CreateEvent>
		SingleCaseListener<IdTree, CreateEvent>
		SingleCaseListener<IdTree, DeleteEvent>
        // SingleCaseListener<IdTree, DeleteEvent>
    }
}

#[cfg(test)]
use ecs::{World, SeqDispatcher, Dispatcher, Lend, LendMut};
#[cfg(test)]
use std::{usize::MAX as UMAX};

#[cfg(test)]
fn create_world() -> World {
	let mut world = World::default();
	world.register_entity::<Node>();
	world.register_multi::<Node, ZI>();
	world.register_multi::<Node, ZDepth>();

	let mut idtree = IdTree::default();
	idtree.set_statistics_count(true);
	world.register_single::<IdTree>(idtree);
	
	world.register_system(atom::Atom::from("z_index_sys"), CellZIndexImpl::new(ZIndexImpl::new()));

	let mut dispatch = SeqDispatcher::default();
	dispatch.build("z_index_sys".to_string(), &world);

	world.add_dispatcher(atom::Atom::from("z_index_sys"), dispatch);
	
	return world;
}
#[test]
fn test(){
    let mut world= create_world();
    test_world_zz(&mut world);
}

#[cfg(test)]
fn new_node(mgr: &mut World, parent: usize) -> usize {
	let node = mgr.fetch_entity::<Node>().unwrap().lend_mut().create();
	let idtree = mgr.fetch_single::<IdTree>().unwrap();
	let idtree = idtree.lend_mut();
	idtree.create(node);
	let notify = unsafe { &* (idtree.get_notify_ref() as *const NotifyImpl)} ;

	idtree.insert_child_with_notify(node, parent, UMAX, &notify);
	node
}

#[cfg(test)]
fn test_world_zz(mgr: &mut World){
    let body_id = new_node(mgr, 0);
    // mgr.run(&atom::Atom::from("z_index_sys"));

    let root_id = new_node(mgr, body_id);
    let temp_id = new_node(mgr, root_id);
    let root_top_id = new_node(mgr, root_id);
    // mgr.run(&atom::Atom::from("z_index_sys"));

    let node_0 = new_node(mgr, root_top_id);
    let node_0_0 = new_node(mgr, node_0);
    let node_0_1 = new_node(mgr, node_0);
    let node_0_1_0 = new_node(mgr, node_0_1);
    let node_0_1_0_0 = new_node(mgr, node_0_1_0);
 
    mgr.run(&atom::Atom::from("z_index_sys"));
    debug_println!("modify run-----------------------------------------");

    print_node(mgr, body_id, 0);
    print_node(mgr, root_id, 1);
    print_node(mgr, temp_id, 2);
    print_node(mgr, root_top_id, 2);
    print_node(mgr, node_0, 3);
    print_node(mgr, node_0_0, 4);
    print_node(mgr, node_0_1, 4);
    print_node(mgr, node_0_1_0, 5);
    print_node(mgr, node_0_1_0_0, 6);

    let node_1 = new_node(mgr, root_top_id);
    // let node_1_0 = new_node(mgr, node_1);
    // let node_1_1 = new_node(mgr, node_1);
    // let node_1_1_0 = new_node(mgr, node_1_1);
    // let node_1_1_0_0 = new_node(mgr, node_1_1_0);

    mgr.run(&atom::Atom::from("z_index_sys"));
    print_node(&mgr, body_id, 0);
    print_node(&mgr, root_id, 1);
    print_node(&mgr, temp_id, 2);
    print_node(&mgr, root_top_id, 2);
    print_node(&mgr, node_0, 3);
    print_node(&mgr, node_0_0, 4);
    print_node(&mgr, node_0_1, 4);
    print_node(&mgr, node_0_1_0, 5);
    print_node(&mgr, node_0_1_0_0, 6);
    print_node(&mgr, node_1, 3);
    // print_node(&mgr, node_1_0, 4);
    // print_node(&mgr, node_1_1, 4);
    // print_node(&mgr, node_1_1_0, 5);
    // print_node(&mgr, node_1_1_0_0, 6);
}
// #[cfg(not(feature = "web"))]
// #[cfg(test)]
// fn test_world_z(world: &mut World){
//     let (root, node1, node2, node3, node4, node5) = {
//         let component_mgr = &mut mgr;
//         {
            
//             let (root, node1, node2, node3, node4, node5) = {
//                 let root = NodeBuilder::new().build(&mut component_mgr.node); // 创建根节点
//                 debug_println!("root element: {:?}", root.element);
//                 let root_id = 1;// 不通知的方式添加 NodeWriteRef{id, component_mgr write 'a Ref}
//                 let _n = component_mgr.node._group.get_mut(root_id);// ComponentNode{parent:usize, owner: 'a &mut Node}
//                 let node1 = NodeBuilder::new().build(&mut component_mgr.node);
//                 let node2 = NodeBuilder::new().build(&mut component_mgr.node);
//                 let node3 = NodeBuilder::new().build(&mut component_mgr.node);
//                 let node4 = NodeBuilder::new().build(&mut component_mgr.node);
//                 let node5 = NodeBuilder::new().build(&mut component_mgr.node);
//                 // let mut root_ref = component_mgr.get_node_mut(root_id);
//                 let n1_id = component_mgr.get_node_mut(root_id).insert_child(node1, InsertType::Back).id;
//                 let n2_id = component_mgr.get_node_mut(root_id).insert_child(node2, InsertType::Back).id;
//                 let n3_id = component_mgr.get_node_mut(n1_id).insert_child(node3, InsertType::Back).id;
//                 let n4_id = component_mgr.get_node_mut(n1_id).insert_child(node4, InsertType::Back).id;
//                 let n5_id = component_mgr.get_node_mut(n2_id).insert_child(node5, InsertType::Back).id;
//                 (
//                     root_id,
//                     n1_id,
//                     n2_id,
//                     n3_id,
//                     n4_id,
//                     n5_id,
//                 )
//            };
//            component_mgr.get_node_mut(node1).set_zindex(-1);
//            component_mgr.get_node_mut(node3).set_zindex(2);
//             print_node(component_mgr, node1);
//             print_node(component_mgr, node2);
//             print_node(component_mgr, node3);
//             print_node(component_mgr, node4);
//             print_node(component_mgr, node5);
//             (root, node1, node2, node3, node4, node5)
//         }
//     };

//     debug_println!("modify run-----------------------------------------");
//     world.run(());
//     print_node(&mgr, root);
//     print_node(&mgr, node1);
//     print_node(&mgr, node2);
//     print_node(&mgr, node3);
//     print_node(&mgr, node4);
//     print_node(&mgr, node5);
//     let n = NodeBuilder::new().build(&mut mgr.node);
//     let node6 = mgr.get_node_mut(root).insert_child(n, InsertType::Back).id;
//     debug_println!("modify2 run-----------------------------------------");
//     world.run(());
//     print_node(&mgr, root);
//     print_node(&mgr, node1);
//     print_node(&mgr, node2);
//     print_node(&mgr, node3);
//     print_node(&mgr, node4);
//     print_node(&mgr, node5);
//     print_node(&mgr, node6);
// }

#[cfg(test)]
fn print_node(mgr: &World, id: usize, layer: usize) {
	let idtree = mgr.fetch_single::<IdTree>().unwrap();
	let idtree = idtree.lend();
	let z_indexs = mgr.fetch_multi::<Node, ZI>().unwrap();
	let z_indexs = z_indexs.lend();
	let z_depths = mgr.fetch_multi::<Node, ZDepth>().unwrap();
	let z_depths = z_depths.lend();
	let node = &idtree[id];
    let zimpl = mgr.fetch_sys::<CellZIndexImpl>(&atom::Atom::from("z_index_sys")).unwrap();
    let zi = &zimpl.owner.borrow().map[id];

	let mut r = "".to_string();
	for i in 0..layer {
		r = r + "  ";
	}
	debug_println!("{}nodeid: {}, zindex: {:?}, z_depth: {:?}, zz: {:?}, count: {}, parent: {}", r, id, z_indexs[id] , z_depths[id], zi, node.count(), node.parent());
}