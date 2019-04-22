//zindex系统
// zindex的min max, 设计分配如下： 如果父容器为 0 100.
//  子节点为1个的话：1 100. 为2个的话： 1 51, 51 100. 为3个的话： 1 34, 34 67, 67 100.


use std::cell::RefCell;
use std::rc::Rc;
use std::f32;
use std::cmp::{Ordering};

use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent, ModifyFieldEvent};
use wcs::world::System;
use heap::simple_heap::SimpleHeap;
use vecmap::VecMap;

use world_doc::{Z_MAX, WorldDocMgr};
use world_doc::component::node::{Node};

pub struct ZIndexSys(RefCell<ZIndexImpl>);

impl ZIndexSys {
  pub fn init(mgr: &mut WorldDocMgr) -> Rc<ZIndexSys> {
    let rc = Rc::new(ZIndexSys(RefCell::new(ZIndexImpl::new())));
    mgr.node.zindex.register_handler(Rc::downgrade(&(rc.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
    mgr.node._group.register_create_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<Node, CreateEvent, WorldDocMgr>>),
    ));
    mgr.node._group.register_delete_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<Node, DeleteEvent, WorldDocMgr>>),
    ));
    rc
  }
}
//监听zindex属性的改变
impl ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr> for ZIndexSys {
  fn handle(&self, event: &ModifyFieldEvent, mgr: &mut WorldDocMgr) {
    let ModifyFieldEvent{id, parent, field: _} = event;
    let mut zimpl = self.0.borrow_mut();
    let zi = unsafe {zimpl.links.get_unchecked_mut(*id)};
    let node = mgr.node._group.get_mut(*id);
    let z = node.zindex;
    if z == zi.old {
      return;
    }
    let old = zi.old;
    zi.old = z;
    if old == AUTO {
      if !zi.dirty {
        // 如果zindex由auto变成有值，则产生新的堆叠上下文，则自身需要设脏。
        zi.dirty = true;
        marked_dirty(&mut zimpl.dirty, *id, node.layer);
      }
    }else if z == AUTO {
      // 为了防止adjust的auto跳出，提前设置为false
      zi.dirty = false;
    }
    zimpl.set_dirty(*parent, mgr);
  }
}
//监听Node的创建， 设置脏标志
impl ComponentHandler<Node, CreateEvent, WorldDocMgr> for ZIndexSys {
  fn handle(&self, event: &CreateEvent, mgr: &mut WorldDocMgr) {
    let CreateEvent{id, parent} = event;
    let mut zi = ZIndex::default();
    zi.old = mgr.node._group.get(*id).zindex;
    let mut zimpl = self.0.borrow_mut();
    zimpl.links.insert(*id, zi);
    zimpl.set_dirty(*parent, mgr);
  }
}
//监听Node的删除创建， 删除脏标志
impl ComponentHandler<Node, DeleteEvent, WorldDocMgr> for ZIndexSys {
  fn handle(&self, event: &DeleteEvent, mgr: &mut WorldDocMgr) {
    let DeleteEvent{id, parent: _} = event;
    self.0.borrow_mut().delete_dirty(*id, mgr);
  }
}

impl System<(), WorldDocMgr> for ZIndexSys {
  fn run(&self, _e: &(), mgr: &mut WorldDocMgr) {
    self.0.borrow_mut().calc(mgr);
    
    // let mut arr = Vec::new();
    // for (id, node) in mgr.node._group.iter() {
    //   arr.push((id, node.z_depth));
    // }
    // println!("arr----z_depth-------------------------{:?}", arr);
  }
}

const AUTO: isize = -1;

#[derive(Debug, Clone, Copy, Default)]
pub struct ZIndex {
  pub dirty: bool, // 子节点设zindex时，将不是auto的父节点设脏
  pub old: isize, // 旧值
  pub pre_min_z: f32, // 预设置的节点的最小z值 // 下面4个值需要单独独立出来吗？ TODO
  pub pre_max_z: f32, // 预设置的节点的最大z值
  pub min_z: f32, // 节点的最小z值，也是节点自身的z值
  pub max_z: f32, // 节点的最大z值，z-index == -1, 则和min_z一样。
}

struct ZIndexImpl {
  dirty: (Vec<Vec<usize>>, usize, usize), // 脏节点, 及脏节点数量，及脏节点的起始层
  links: VecMap<ZIndex>,
  cache: Cache,
}

impl ZIndexImpl {
  fn new() -> ZIndexImpl {
    let mut links = VecMap::new();
    // 为root节点设置最大范围值
    let mut zi = ZIndex::default();
    zi.pre_min_z = -Z_MAX;
    zi.pre_max_z = Z_MAX;
    zi.min_z = -Z_MAX;
    zi.pre_max_z = Z_MAX;
    links.insert(1, zi);
    ZIndexImpl {
      dirty: (Vec::new(), 0, usize::max_value()),
      links: links,
      cache: Cache::new(),
    }
  }

  // 设置节点对应堆叠上下文的节点脏
  fn set_dirty(&mut self, mut node_id: usize, mgr: &mut WorldDocMgr) {
    while node_id > 0 {
      let node = mgr.node._group.get_mut(node_id);
      // 如果为z为auto，则向上找zindex不为auto的节点，zindex不为auto的节点有堆叠上下文
      if node.zindex != AUTO {
        let zi = unsafe {self.links.get_unchecked_mut(node_id)};
        if !zi.dirty {
          zi.dirty = true;
          marked_dirty(&mut self.dirty, node_id, node.layer);
        }
        if (node.count as f32) < zi.pre_max_z - zi.pre_min_z {
          return;
        }
        // 如果z范围超过自身全部子节点及其下子节点数量，则继续向上设置脏，等calc_z调整以获得足够的z范围
      }
      node_id = node.parent;
    }
  }
  fn delete_dirty(&mut self, node_id: usize, mgr: &mut WorldDocMgr) {
    let zi = unsafe {self.links.get_unchecked_mut(node_id)};
    if zi.dirty {
      let vec = unsafe { self.dirty.0.get_unchecked_mut(mgr.node._group.get_mut(node_id).layer) };
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
  fn calc(&mut self, mgr: &mut WorldDocMgr) {
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
          let zi = unsafe {self.links.get_unchecked_mut(*node_id)};
          if !zi.dirty {
            continue;
          }
          zi.dirty = false;
          zi.min_z = zi.pre_min_z;
          zi.max_z = zi.pre_max_z;
          (zi.min_z, zi.max_z, node.count, node.get_childs_mut().get_first())
        };
        // 设置node.z_depth, 其他系统会监听该值
        mgr.get_node_mut(*node_id).set_z_depth(min_z);
        if count == 0 {
          continue;
        }
        cache.sort(mgr, child, 0);
        cache.calc(&mut self.links, mgr, min_z, max_z, count);
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

#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct ZSort (isize, usize, usize, usize); // (zindex, index, node_id, children_count)

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
  fn sort(&mut self, mgr: &mut WorldDocMgr, mut child: usize, mut order: usize) -> usize {
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
        let (zi, count, child_id) = {
          let n = mgr.node._group.get_mut(node_id);
          (n.zindex, n.count, n.get_childs_mut().get_first())
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
  fn calc(&mut self, links: &mut VecMap<ZIndex>, mgr: &mut WorldDocMgr, mut min_z: f32, mut max_z: f32, count: usize) {
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
      adjust(links, mgr, n_id, min_z, max_z, f32::NAN, 0.);
      min_z = max_z;
    }
    for n_id in &self.z_auto {
      adjust(links, mgr, *n_id, min_z, min_z, f32::NAN, 0.);
      min_z += 1.;
    }
    self.z_auto.clear();
    for zs in &self.z_zero {
      max_z = min_z + split + split * zs.3 as f32;
      adjust(links, mgr, zs.2, min_z, max_z, f32::NAN, 0.);
      min_z = max_z;
    }
    self.z_zero.clear();
    while let Some(ZSort(_, _, n_id, c)) = self.node_heap.pop() {
      max_z = min_z + split + split * c as f32;
      adjust(links, mgr, n_id, min_z, max_z, f32::NAN, 0.);
      min_z = max_z;
    }
  }
}
//================================ 内部静态方法
// 标记指定节点的脏
fn marked_dirty(dirty: &mut(Vec<Vec<usize>>, usize, usize), node_id: usize, layer: usize) {
  dirty.1 += 1;
  if dirty.2 > layer {
    dirty.2 = layer;
  }
  if dirty.0.len() <= layer {
    for _ in dirty.0.len()..layer + 1 {
      dirty.0.push(Vec::new())
    }
  }
  let vec = unsafe { dirty.0.get_unchecked_mut(layer) };
  vec.push(node_id);
}

// 整理方法。z范围变小或相交，则重新扫描修改一次。两种情况。
// 1. 有min_z max_z，修改该节点，计算rate，递归调用。
// 2. 有min_z rate parent_min， 根据rate和新旧min, 计算新的min_z max_z。 要分辨是否为auto节点
fn adjust(links: &mut VecMap<ZIndex>, mgr: &mut WorldDocMgr, node_id: usize, min_z: f32, max_z: f32, rate: f32, parent_min: f32) {
  let (mut child, min, r, old_min) = {
    let node = mgr.node._group.get_mut(node_id);
    let zi = unsafe{links.get_unchecked_mut(node_id)};
    let (min, max) = if !rate.is_nan() {
      (((zi.pre_min_z - parent_min) * rate) + min_z + 1., ((zi.pre_max_z - parent_min) * rate) + min_z + 1.)
    }else{
      (min_z, max_z)
    };
    // println!("adjust: id:{}, min_z:{}, max_z:{}, rate:{}, parent_min:{}, min:{}, max:{}", node_id, min_z, max_z, rate, parent_min, min, max);
    zi.pre_min_z = min;
    zi.pre_max_z = max;
    if zi.dirty {
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
    // 设置node.z_depth, 其他系统会监听该值
    mgr.get_node_mut(node_id).set_z_depth(min);
    // 判断是否为auto
    if min != max {
      (child, min, (max_z - min_z - 1.)/ (old_max_z - old_min_z), old_min_z)
    }else if !rate.is_nan() {
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
      adjust(links, mgr, node_id, min, 0., r, old_min);
  }
}

#[cfg(test)]
#[cfg(not(feature = "web"))]
use wcs::world::{World};
#[cfg(test)]
#[cfg(not(feature = "web"))]
use wcs::component::{Builder};
#[cfg(test)]
#[cfg(not(feature = "web"))]
use world_doc::component::node::{NodeBuilder, InsertType};
#[cfg(test)]
#[cfg(not(feature = "web"))]
use super::node_count::{NodeCountSys};

#[cfg(not(feature = "web"))]
#[test]
fn test(){
    let mut world: World<WorldDocMgr, ()> = World::new(WorldDocMgr::new());
    let _nc = NodeCountSys::init(&mut world.component_mgr);
    let zz = ZIndexSys::init(&mut world.component_mgr);
    let systems: Vec<Rc<System<(), WorldDocMgr>>> = vec![zz.clone()];
    world.set_systems(systems);
    test_world_zz(&mut world, zz);
}

#[cfg(test)]
#[cfg(not(feature = "web"))]
fn new_node(component_mgr: &mut WorldDocMgr, parent_id: usize) -> usize {
    let node = NodeBuilder::new().build(&mut component_mgr.node);
    component_mgr.get_node_mut(parent_id).insert_child(node, InsertType::Back).id
}
#[cfg(not(feature = "web"))]
#[cfg(test)]
fn test_world_zz(world: &mut World<WorldDocMgr, ()>, zz: Rc<ZIndexSys>){
    let mgr = &mut world.component_mgr;
    let body_id = new_node(mgr, 1);
    world.run(());
    let mgr = &mut world.component_mgr;
    let root_id = new_node(mgr, body_id);
    let temp_id = new_node(mgr, root_id);
    let root_top_id = new_node(mgr, root_id);
    world.run(());
    let mgr = &mut world.component_mgr;
    let node_0 = new_node(mgr, root_top_id);
    let node_0_0 = new_node(mgr, node_0);
    let node_0_1 = new_node(mgr, node_0);
    let node_0_1_0 = new_node(mgr, node_0_1);
    let node_0_1_0_0 = new_node(mgr, node_0_1_0);
 
    world.run(());
    println!("modify run-----------------------------------------");
    let mgr = &mut world.component_mgr;
    print_node(mgr, zz.clone(), 1);
    print_node(mgr, zz.clone(), body_id);
    print_node(mgr, zz.clone(), root_id);
    print_node(mgr, zz.clone(), temp_id);
    print_node(mgr, zz.clone(), root_top_id);
    print_node(mgr, zz.clone(), node_0);
    print_node(mgr, zz.clone(), node_0_0);
    print_node(mgr, zz.clone(), node_0_1);
    print_node(mgr, zz.clone(), node_0_1_0);
    print_node(mgr, zz.clone(), node_0_1_0_0);

    let node_1 = new_node(mgr, root_top_id);
    let node_1_0 = new_node(mgr, node_1);
    let node_1_1 = new_node(mgr, node_1);
    let node_1_1_0 = new_node(mgr, node_1_1);
    let node_1_1_0_0 = new_node(mgr, node_1_1_0);
    println!("modify2 run-----------------------------------------");
    world.run(());
    print_node(&world.component_mgr, zz.clone(), 1);
    print_node(&world.component_mgr, zz.clone(), body_id);
    print_node(&world.component_mgr, zz.clone(), root_id);
    print_node(&world.component_mgr, zz.clone(), temp_id);
    print_node(&world.component_mgr, zz.clone(), root_top_id);
    print_node(&world.component_mgr, zz.clone(), node_0);
    print_node(&world.component_mgr, zz.clone(), node_0_0);
    print_node(&world.component_mgr, zz.clone(), node_0_1);
    print_node(&world.component_mgr, zz.clone(), node_0_1_0);
    print_node(&world.component_mgr, zz.clone(), node_0_1_0_0);
    print_node(&world.component_mgr, zz.clone(), node_1);
    print_node(&world.component_mgr, zz.clone(), node_1_0);
    print_node(&world.component_mgr, zz.clone(), node_1_1);
    print_node(&world.component_mgr, zz.clone(), node_1_1_0);
    print_node(&world.component_mgr, zz.clone(), node_1_1_0_0);
}
#[cfg(not(feature = "web"))]
#[cfg(test)]
// fn test_world_z(world: &mut World<WorldDocMgr, ()>, zz: Rc<ZIndexSys>){
//     let (root, node1, node2, node3, node4, node5) = {
//         let component_mgr = &mut world.component_mgr;
//         {
            
//             let (root, node1, node2, node3, node4, node5) = {
//                 let root = NodeBuilder::new().build(&mut component_mgr.node); // 创建根节点
//                 println!("root element: {:?}", root.element);
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
//             print_node(component_mgr, zz.clone(), node1);
//             print_node(component_mgr, zz.clone(), node2);
//             print_node(component_mgr, zz.clone(), node3);
//             print_node(component_mgr, zz.clone(), node4);
//             print_node(component_mgr, zz.clone(), node5);
//             (root, node1, node2, node3, node4, node5)
//         }
//     };

//     println!("modify run-----------------------------------------");
//     world.run(());
//     print_node(&world.component_mgr, zz.clone(), root);
//     print_node(&world.component_mgr, zz.clone(), node1);
//     print_node(&world.component_mgr, zz.clone(), node2);
//     print_node(&world.component_mgr, zz.clone(), node3);
//     print_node(&world.component_mgr, zz.clone(), node4);
//     print_node(&world.component_mgr, zz.clone(), node5);
//     let n = NodeBuilder::new().build(&mut world.component_mgr.node);
//     let node6 = world.component_mgr.get_node_mut(root).insert_child(n, InsertType::Back).id;
//     println!("modify2 run-----------------------------------------");
//     world.run(());
//     print_node(&world.component_mgr, zz.clone(), root);
//     print_node(&world.component_mgr, zz.clone(), node1);
//     print_node(&world.component_mgr, zz.clone(), node2);
//     print_node(&world.component_mgr, zz.clone(), node3);
//     print_node(&world.component_mgr, zz.clone(), node4);
//     print_node(&world.component_mgr, zz.clone(), node5);
//     print_node(&world.component_mgr, zz.clone(), node6);
// }

#[cfg(not(feature = "web"))]
#[cfg(test)]
fn print_node(mgr: &WorldDocMgr, zz: Rc<ZIndexSys>, id: usize) {
    let node = mgr.node._group.get(id);
    let zimpl = zz.0.borrow_mut();
    let zi = unsafe{zimpl.links.get_unchecked(id)};

    println!("nodeid: {}, zindex: {:?}, z_depth: {}, zz: {:?}, count: {}, parent: {}", id, node.zindex, node.z_depth, zi, node.count, node.parent);
}