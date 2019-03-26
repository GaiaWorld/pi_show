//zindex系统
// zindex的min max, 设计分配如下： 如果父容器为 0 100.
//  子节点为1个的话：1 100. 为2个的话： 1 51, 51 100. 为3个的话： 1 34, 34 67, 67 100.


use std::cell::RefCell;
use std::rc::Rc;
use std::cmp::{Ordering};

use wcs::component::{ComponentHandler, Builder, Event, notify};
use wcs::world::System;
use heap::simple_heap::SimpleHeap;
use world::GuiComponentMgr;

use component::node::{NodeBuilder, ZIndex, ZIndexWriteRef};

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
        let node = mgr.node._group.get_mut(*parent);
        self.0.borrow_mut().set_dirty(node.parent, mgr);
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
        if old == AUTO {
          if !node.z_dirty {
            // 如果zindex由auto变成有值，则产生新的堆叠上下文，则自身需要设脏。
            node.z_dirty = true;
            zimpl.marked_dirty(*parent, node.layer);
          }
        }else if z == AUTO {
          // 为了防止adjust的auto跳出，提前设置为false
          node.z_dirty = false;
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
    self.0.borrow_mut().calc(mgr);
  }
}

const AUTO: isize = -1;
const MAX: f32 = 8388608.0;

#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct ZSort (isize, usize, usize, usize); // (zindex, index, node_id, children_count)

struct ZIndexImpl {
  dirty: (Vec<Vec<usize>>, usize, usize), // 脏节点, 及脏节点数量，及脏节点的起始层
  cache: Cache,
}

impl ZIndexImpl {
  fn new() -> ZIndexImpl {
    ZIndexImpl {
      dirty: (Vec::new(), 0, usize::max_value()),
      cache: Cache::new(),
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
        let zi = mgr.node.zindex._group.get(node.zindex);
        if (node.count as f32) < zi.pre_max_z - zi.pre_min_z {
          return;
        }
        // 如果z范围超过自身全部子节点及其下子节点数量，则继续向上设置脏，等calc_z调整以获得足够的z范围
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
  }
  // 整理方法
  fn calc(&mut self, mgr: &mut GuiComponentMgr) {
    let mut count = self.dirty.1;
    if count == 0 {
      return;
    }
    let cache = &mut self.cache;
    for i in self.dirty.2..self.dirty.0.len() {
      let vec = unsafe { self.dirty.0.get_unchecked_mut(i) };
      let c = vec.len();
      if c == 0 {
        continue;
      }
      for j in 0..c {
        let node_id = unsafe { vec.get_unchecked(j) };
        let (min_z, max_z, count, child) = {
          let node = mgr.node._group.get_mut(*node_id);
          if !node.z_dirty {
            continue;
          }
          node.z_dirty = false;
          let zi = mgr.node.zindex._group.get_mut(node.zindex);
          zi.min_z = zi.pre_min_z;
          zi.max_z = zi.pre_max_z;
          (zi.min_z, zi.max_z, node.count, node.get_childs_mut().get_first())
        };
        if count == 0 {
          continue;
        }
        cache.sort(mgr, child, 0);
        cache.calc(mgr, min_z, max_z, count);
      }
      if count <= c {
        break;
      }
      count -= c;
      vec.clear();
    }
    self.dirty.1 = 0;
    self.dirty.2 = usize::max_value();
  }

}

// 计算z排序时使用的临时数据结构
struct Cache {
  node_heap: SimpleHeap<ZSort>,
  negative_heap: SimpleHeap<ZSort>,
  z_zero: Vec<ZSort>,
  z_auto: Vec<usize>,
}
impl Cache {
  fn new() -> Cache {
    Cache {
      node_heap: SimpleHeap::new(Ordering::Less),
      negative_heap: SimpleHeap::new(Ordering::Less),
      z_zero: Vec::new(),
      z_auto: Vec::new(),
    }
  }

  // 循环计算子节点， 分类排序
  fn sort(&mut self, mgr: &mut GuiComponentMgr, mut child: usize, mut order: usize) -> usize {
    // zindex为0或-1的不参与排序。 zindex排序。用heap排序，确定每个子节点的z范围。如果子节点的zindex==-1，则需要将其子节点纳入排序。
    loop {
        if child == 0 {
            return order;
        }
        let node_id = {
            let v = unsafe{ mgr.node_container.get_unchecked(child) };
            child = v.next;
            v.elem.clone()
        };
    //println!("-----------sort, {} {}", node_id, order);
        let (zi, count, child_id) = {
          let n = mgr.node._group.get_mut(node_id);
          (n.z_index, n.count, n.get_childs_mut().get_first())
        };
        if zi == 0 {
            self.z_zero.push(ZSort(zi, order, node_id, count));
        }else if zi == -1 {
          self.z_auto.push(node_id);
          // 继续递归其子节点
          order = self.sort(mgr, child_id, order);
        }else if zi > 0 {
          self.node_heap.push(ZSort(zi, order, node_id, count));
        }else{
          self.negative_heap.push(ZSort(zi-1, order, node_id, count));
        }
        order+=1;
    }
  }
  // 计算真正的z
  fn calc(&mut self, mgr: &mut GuiComponentMgr, mut min_z: f32, mut max_z: f32, count: usize) {
    let auto_len = self.z_auto.len();
    // 计算大致的劈分间距
    let split = if count > auto_len {
      (max_z - min_z - 1. - auto_len as f32) / (count - auto_len) as f32
    }else{
      1.
    };
    min_z += 1.; // 第一个子节点的z，要在父节点z上加1
    while let Some(ZSort(_, _, n_id, c)) = self.negative_heap.pop() {
      max_z = min_z + split + split * c as f32;
      adjust(mgr, n_id, min_z, max_z, 0., 0.);
      min_z = max_z;
    }
    for n_id in &self.z_auto {
      adjust(mgr, *n_id, min_z, min_z, 0., 0.);
      min_z += 1.;
    }
    self.z_auto.clear();
    for zs in &self.z_zero {
      max_z = min_z + split + split * zs.3 as f32;
      adjust(mgr, zs.2, min_z, max_z, 0., 0.);
      min_z = max_z;
    }
    self.z_zero.clear();
    while let Some(ZSort(_, _, n_id, c)) = self.node_heap.pop() {
      max_z = min_z + split + split * c as f32;
      adjust(mgr, n_id, min_z, max_z, 0., 0.);
      min_z = max_z;
    }
  }
}
//================================ 内部静态方法
// 整理方法。z范围变小或相交，则重新扫描修改一次。两种情况。
// 1. 有min_z max_z，修改该节点，计算rate，递归调用。
// 2. 有min_z rate parent_min， 根据rate和新旧min, 计算新的min_z max_z。 要分辨是否为auto节点
fn adjust(mgr: &mut GuiComponentMgr, node_id: usize, min_z: f32, max_z: f32, rate: f32, parent_min: f32) {
    //println!("-----------adjust, node_id {}, min_z {}, max_z {}, rate {}, parent_min {}", node_id, min_z, max_z, rate, parent_min);
  let (mut child, min, r, old_min) = {
    let node = mgr.node._group.get_mut(node_id);
    let zi_id = node.zindex;
    let zi = mgr.node.zindex._group.get_mut(zi_id);
    let (min, max) = if rate > 0. {
      (((zi.pre_min_z - parent_min) * rate) + min_z + 1., ((zi.pre_max_z - parent_min) * rate) + min_z + 1.)
    }else{
      (min_z, max_z)
    };
    zi.pre_min_z = min;
    zi.pre_max_z = max;
    if node.z_dirty {
      // 如果节点脏，则跳过，后面会进行处理
      return;
    }
    if max >= zi.max_z && min <= zi.min_z {
      // 如果子节点的z范围变大，则可以不继续处理该子节点
      return;
    }
    let old_min_z = zi.min_z + 1.; // 以后用于算子节点的z，所以提前加1
    let old_max_z = zi.max_z;
    // 更新当前值
    zi.min_z = min;
    zi.max_z = max;
    let child = node.get_childs_mut().get_first();
    notify(Event::ModifyField{id: zi_id, parent: node_id, field: "min_z"}, &mgr.node.zindex._group.get_handlers().borrow(), mgr);
    //ZIndexWriteRef::new(zi); TODO 改成node_ref.set_z(min)
    // 判断是否为auto
    if min != max {
      (child, min, (max_z - min_z - 1.) as f32 / (old_max_z - old_min_z), old_min_z)
    }else if rate > 0.{
      // 如果是auto，则重用min_z, rate, parent_min
      (child, min_z, rate, parent_min)
    }else{
      return
    }
  };
  //递归计算子节点的z
  loop {
      if child == 0 {
          return;
      }
      let node_id = {
          let v = unsafe{ mgr.node_container.get_unchecked(child) };
          child = v.next;
          v.elem.clone()
      };
      adjust(mgr, node_id, min, 0., r, old_min);
  }
}

#[cfg(test)]
use wcs::world::{World};
#[cfg(test)]
use component::node::{Node, InsertType};
#[cfg(test)]
use super::node_count::{NodeCount};

#[test]
fn test(){
    let mut world: World<GuiComponentMgr, ()> = World::new(GuiComponentMgr::default());
    let nc = NodeCount::init(&mut world.component_mgr);
    let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![ZIndexS::init(&mut world.component_mgr)];
    world.set_systems(systems);
    test_world_z(&mut world);
}

#[cfg(test)]
fn test_world_z(world: &mut World<GuiComponentMgr, ()>){
    let (root, node1, node2, node3, node4, node5) = {
        let component_mgr = &mut world.component_mgr;
        {
            
            let (root, node1, node2, node3, node4, node5) = {
                let root = Node::default(); // 创建根节点
                let root_id = component_mgr.add_node(root).id;// NodeWriteRef{id, component_mgr write 'a Ref}
                // let mut zi_ref = root_ref.get_zindex_mut(); // ZIndexWriteRef{id = 0, component_mgr write 'a Ref}
                // 根节点必须手工设置zindex及其范围
                let mut zi = ZIndex::default();
                zi.zindex = 0;
                zi.pre_min_z = -MAX;
                zi.pre_max_z = MAX;
                zi.min_z = -MAX;
                zi.pre_max_z = MAX;
                let zi_id = component_mgr.node.zindex._group.insert(zi, root_id);
                let n = component_mgr.node._group.get_mut(root_id);// ComponentNode{parent:usize, owner: 'a &mut Node}
                n.zindex = zi_id; // 避免引发监听
                let node1 = NodeBuilder::new().build(&mut component_mgr.node);
                let node2 = NodeBuilder::new().build(&mut component_mgr.node);
                let node3 = NodeBuilder::new().build(&mut component_mgr.node);
                let node4 = NodeBuilder::new().build(&mut component_mgr.node);
                let node5 = NodeBuilder::new().build(&mut component_mgr.node);
                // let mut root_ref = component_mgr.get_node_mut(root_id);
                let n1_id = component_mgr.get_node_mut(root_id).insert_child(node1, InsertType::Back).id;
                let n2_id = component_mgr.get_node_mut(root_id).insert_child(node2, InsertType::Back).id;
                let n3_id = component_mgr.get_node_mut(n1_id).insert_child(node3, InsertType::Back).id;
                let n4_id = component_mgr.get_node_mut(n1_id).insert_child(node4, InsertType::Back).id;
                let n5_id = component_mgr.get_node_mut(n2_id).insert_child(node5, InsertType::Back).id;
                (
                    root_id,
                    n1_id,
                    n2_id,
                    n3_id,
                    n4_id,
                    n5_id,
                )
           };
           component_mgr.get_node_mut(node1).get_zindex_mut().set_zindex(-1);
           component_mgr.get_node_mut(node3).get_zindex_mut().set_zindex(2);
            print_node(component_mgr, node1);
            print_node(component_mgr, node2);
            print_node(component_mgr, node3);
            print_node(component_mgr, node4);
            print_node(component_mgr, node5);
            (root, node1, node2, node3, node4, node5)
        }
    };

    println!("modify run-----------------------------------------");
    world.run(());
    print_node(&world.component_mgr, root);
    print_node(&world.component_mgr, node1);
    print_node(&world.component_mgr, node2);
    print_node(&world.component_mgr, node3);
    print_node(&world.component_mgr, node4);
    print_node(&world.component_mgr, node5);
    let n = NodeBuilder::new().build(&mut world.component_mgr.node);
    let node6 = world.component_mgr.get_node_mut(root).insert_child(n, InsertType::Back).id;
    println!("modify2 run-----------------------------------------");
    world.run(());
    print_node(&world.component_mgr, root);
    print_node(&world.component_mgr, node1);
    print_node(&world.component_mgr, node2);
    print_node(&world.component_mgr, node3);
    print_node(&world.component_mgr, node4);
    print_node(&world.component_mgr, node5);
    print_node(&world.component_mgr, node6);
}

#[cfg(test)]
fn print_node(mgr: &GuiComponentMgr, id: usize) {
    let node = mgr.node._group.get(id);
    let mut z = &ZIndex::default();
    if node.zindex > 0 {
      z = mgr.node.zindex._group.get(node.zindex)
    };

    println!("nodeid: {}, z:{:?}, z_index: {:?}, z_dirty: {}, count: {}", id, z, node.z_index, node.z_dirty, node.count);
}