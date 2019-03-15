//zindex系统
// zindex的min max, 设计分配如下： 如果父容器为 0 100.
//  子节点为1个的话：1 100. 为2个的话： 1 51, 51 100. 为3个的话： 1 34, 34 67, 67 100.


use std::cell::RefCell;
use std::rc::Rc;
use std::cmp::{Ordering};

use wcs::component::{ComponentHandler, Event};
use wcs::world::System;
use heap::simple_heap::SimpleHeap;

use component::component_def::{GuiComponentMgr, Node, ZIndex};

pub struct ZIndexS(RefCell<ZIndexImpl>);

impl ZIndexS {
  pub fn init(mgr: &mut GuiComponentMgr) -> Rc<ZIndexS> {
    let rc = Rc::new(ZIndexS(RefCell::new(ZIndexImpl::new())));
    mgr.node.zindex._group.register_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<ZIndex, GuiComponentMgr>>),
    ));
    rc
  }
}

impl ComponentHandler<ZIndex, GuiComponentMgr> for ZIndexS {
  fn handle(&self, event: &Event, mgr: &mut GuiComponentMgr) {
    match event {
      Event::Create { id: _, parent } => {
        self.0.borrow_mut().set_dirty(*parent, mgr);
      }
      Event::Delete { id: _, parent } => {
        self.0.borrow_mut().delete_dirty(*parent, mgr);
      }
      Event::ModifyField {
        id,
        parent,
        field: _,
      } => {
        let zi = mgr.node.zindex._group.get_mut(*id);
        let z = zi.zindex;
        // 更新z_auto
        let node = mgr.node._group.get_mut(*parent);
        if z == node.z_index {
          return;
        }
        let old = node.z_index;
        node.z_index = z;
        let mut zimpl = self.0.borrow_mut();
        if old == AUTO && !node.z_dirty {
          // 如果zindex由auto变成有值，则产生新的堆叠上下文，则自身需要设脏。 ？？ 好像也不需要
          node.z_dirty = true;
          zimpl.marked_dirty(*parent, node.layer);
        }
        zimpl.set_dirty(node.parent, mgr);
      }
      _ => {
        unreachable!();
      }
    }
  }
}
impl System<(), GuiComponentMgr> for ZIndexS {
  fn run(&self, _e: &(), mgr: &mut GuiComponentMgr) {
    self.0.borrow_mut().cal_z(mgr);
  }
}

const AUTO: isize = -1;

#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct ZSort (isize, usize, usize, usize); // (zindex, index, node_id, children_count)

struct ZIndexImpl {
  dirty: (Vec<Vec<usize>>, usize, usize), // 脏节点, 及脏节点数量，及脏节点的起始层
  // 计算z排序时使用的临时数据结构
  node_heap: SimpleHeap<ZSort>,
  negative_heap: SimpleHeap<ZSort>,
  z_zero: Vec<ZSort>,
  z_auto: Vec<usize>,
}

impl ZIndexImpl {
  fn new() -> ZIndexImpl {
    ZIndexImpl {
      dirty: (Vec::new(), 0, usize::max_value()),
      node_heap: SimpleHeap::new(Ordering::Less),
      negative_heap: SimpleHeap::new(Ordering::Less),
      z_zero: Vec::new(),
      z_auto: Vec::new(),
    }
  }
  // 标记指定节点的脏
  fn marked_dirty(&mut self, node_id: usize, layer: usize) {
    self.dirty.1 += 1;
    if self.dirty.2 > layer {
      self.dirty.2 = layer;
    }
    if self.dirty.0.len() <= layer {
      for _ in self.dirty.0.len()..layer + 1 {
        self.dirty.0.push(Vec::new())
      }
    }
    let vec = unsafe { self.dirty.0.get_unchecked_mut(layer) };
    vec.push(node_id);
  }
  // 设置节点对应堆叠上下文的节点脏
  fn set_dirty(&mut self, mut node_id: usize, mgr: &mut GuiComponentMgr) {
    while node_id > 0 {
      let node = mgr.node._group.get_mut(node_id);
      // 如果为z为auto，则向上找zindex不为auto的节点，zindex不为auto的节点有堆叠上下文
      if node.z_index != AUTO {
        if !node.z_dirty {
          node.z_dirty = true;
          self.marked_dirty(node_id, node.layer);
        }
        return;
      }
      node_id = node.parent;
    }
  }
  fn delete_dirty(&mut self, node_id: usize, mgr: &mut GuiComponentMgr) {
    let node = mgr.node._group.get_mut(node_id);
    if node.z_dirty {
      let vec = unsafe { self.dirty.0.get_unchecked_mut(node.layer) };
      for i in 0..vec.len() {
        if vec[i] == node_id {
          vec.swap_remove(i);
          self.dirty.1 -= 1;
          break;
        }
      }
    }
    // 删除无需设脏，z值可以继续使用
    //self.set_dirty(node.parent, mgr);
  }
  // 整理方法
  fn cal_z(&mut self, mgr: &mut GuiComponentMgr) {
    let mut count = self.dirty.1;
    if count == 0 {
      return;
    }
    for i in self.dirty.2..self.dirty.0.len() {
      let vec = unsafe { self.dirty.0.get_unchecked_mut(i) };
      let c = vec.len();
      if c == 0 {
        continue;
      }
      for j in 0..c {
        let np = unsafe { vec.get_unchecked(j) };
        let node = mgr.node._group.get_mut(*np);
        if !node.z_dirty {
          continue;
        }
        node.z_dirty = false;
        if node.z_index == AUTO || node.count === 0 {
          continue;
        }
        let zi = mgr.node.zindex._group.get_mut(node.zindex);
        if node.count >= zi.max_z - zi.min_z {
          // 如果z范围超过自身全部子节点及其下子节点数量，则向上调整以获得足够的z范围

        }
        // zindex为0或-1的不参与排序。 zindex排序。用heap排序，确定每个子节点的z范围。如果子节点的zindex==-1，则需要将其子节点纳入排序。
        let mut order = 0;
        let mut children_count = 0;
        for r in node.childs.iter(&mut mgr.node_container) {
          let n = mgr.node._group.get_mut(r);
          if n.z_index == 0 {
            self.z_zero.push(ZSort(n.z_index, order, *r, n.count));
          }else if n.z_index == -1 {
            self.z_auto.push(*r);
            // TODO 继续递归其子节点
            continue;
          }else if n.z_index > 0 {
            self.node_heap.push(ZSort(n.z_index, order, *r, n.count));
          }else{
            self.negative_heap.push(ZSort(n.z_index-1, order, *r, n.count));
          }
          children_count += n.count + 1;
          order+=1;
        }
        let split = if children_count > 0 {
          (zi.max_z - zi.min_z - 1 - self.z_auto.len()) / children_count
        }else{
          1
        };
        let start = zi.min_z + 1;
        while self.negative_heap.len() > 0 {

        }
        // zindex为-1（代表auto）的z范围仅min_z有效，max_z为min_z。
        // 如果子节点的z范围变大，则可以不继续处理该子节点。z范围变小或相交，则重新排列一次，因为记录了order，成本也很低。
        self.adjust(mgr, );
        //   &mut self.oct_slab,
        //   &mut self.ab_slab,
        //   &self.adjust,
        //   self.deep,
        //   *oct_id,
        // );
      }
      vec.clear();
      if count <= c {
        break;
      }
      count -= c;
    }
    self.dirty.1 = 0;
    self.dirty.2 = usize::max_value();
  }
  // 整理方法，。z范围变小或相交，则重新扫描一次，因为记录了order，可以根据order和rate重新计算min max，成本也很低。
  fn adjust(&mut self, mgr: &mut GuiComponentMgr, node_id: usize, min_z: usize, max_z: usize) {
    let node = mgr.node._group.get_mut(node_id);
    let (rate, old_min) = {
      let zi = mgr.node.zindex._group.get_mut(node.zindex);
      if !node.z_dirty {
        // 如果节点脏，则设置最大最小z值后跳过
        zi.max_z = max_z;
        zi.min_z = min_z;
        return;
      }
      let old_max_z = zi.max_z;
      let old_min_z = zi.min_z;
      if max_z >= old_max_z && min_z <= old_min_z {
        // 如果子节点的z范围变大，则可以不继续处理该子节点，为了一致性，也不更新最大最小z值
        return;
      }
      zi.max_z = max_z;
      zi.min_z = min_z;
      ((max_z - min_z - 1) as f32 / (old_max_z - old_min_z - 1), old_min_z + 1)
    }
    for r in node.childs.iter(&mut mgr.node_container) {
      let n = mgr.node._group.get_mut(r);
      let zi = mgr.node.zindex._group.get_mut(node.zindex);
      zi.max_z = ((zi.max_z - old_min) * rate) as usize + min_z + 1;
      zi.min_z = ((zi.min_z - old_min) * rate) as usize + min_z + 1;
      if n.z_index == -1 {

        // TODO 继续递归其子节点
        continue;
      }
    }
  }
}

