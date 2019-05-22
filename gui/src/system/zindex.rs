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
  monitor::{CreateEvent, DeleteEvent, ModifyEvent},
  component::MultiCaseImpl,
  single::SingleCaseImpl,
  idtree::{IdTree, Node as IdNode},
};

use entity::{Node};
use component::{
  user::{ZIndex as ZI},
  calc::{ZDepth, ZDepthWrite},
};
use Z_MAX;
use Root;

impl<'a> EntityListener<'a, Node, CreateEvent> for ZIndexImpl {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, ZI>, &'a mut MultiCaseImpl<Node, ZDepth>);

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
      self.map.insert(event.id, ZIndex::default());
      write.0.insert(event.id, ZI::default());
      write.1.insert(event.id, ZDepth::default());
    }
}

impl<'a> EntityListener<'a, Node, DeleteEvent> for ZIndexImpl {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();

    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, _write: Self::WriteData) {
      match read.get(event.id) {
        Some(r) => self.delete_dirty(event.id, r.layer),
        _ => ()
      }
    }
}
impl<'a> MultiCaseListener<'a, Node, ZI, ModifyEvent> for ZIndexImpl {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, ZI>);
    type WriteData = ();

    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData) {
      let z = **unsafe{ read.1.get_unchecked(event.id)};
      let zi = unsafe {self.map.get_unchecked_mut(event.id)};
      let old = zi.old;
      zi.old = z;
      let node = unsafe{ read.0.get_unchecked(event.id)};
      if node.layer == 0 {
        return;
      }
      if old == AUTO {
        if zi.dirty == DirtyType::None {
          // 如果zindex由auto变成有值，则产生新的堆叠上下文，则自身需要设脏。
          zi.dirty = DirtyType::Normal;
          self.dirty.mark(event.id, node.layer);
        }
      }else if z == AUTO {
        // 为了防止adjust的auto跳出，提前设置为false
        zi.dirty = DirtyType::None;
      }
      self.set_parent_dirty(node.parent, &read.0);
    }
}
impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for ZIndexImpl {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();

    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, _write: Self::WriteData) {
      let node = unsafe{ read.get_unchecked(event.id)};
      let zi = unsafe {self.map.get_unchecked_mut(event.id)};
      if zi.old != AUTO {
        // 设置自己成强制脏
        zi.dirty = DirtyType::Recursive;
        self.dirty.mark(event.id, node.layer);
      }else {
        // 设置自己所有非AUTO的子节点为强制脏
        recursive_dirty(&mut self.map, &mut self.dirty, read, node.children.head);
      }
      self.set_parent_dirty(node.parent, read);
    }
}

impl<'a> Runner<'a> for ZIndexImpl {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = &'a mut MultiCaseImpl<Node, ZDepth>;

    fn setup(&mut self, _read: Self::ReadData, _write: Self::WriteData) {
    }
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
      self.calc(read, write)
    }
    fn dispose(&mut self, _read: Self::ReadData, _write: Self::WriteData) {

    }
}


const AUTO: isize = -1;
#[derive(Debug, Clone, PartialEq, EnumDefault)]
pub enum DirtyType {
  None,
  Normal, // 
  Recursive, // 递归脏，计算所有的子节点
}

#[derive(Debug, Clone, Default)]
pub struct ZIndex {
  pub dirty: DirtyType, // 子节点设zindex时，将不是auto的父节点设脏
  pub old: isize, // 旧值
  pub pre_min_z: f32, // 预设置的节点的最小z值
  pub pre_max_z: f32, // 预设置的节点的最大z值
  pub min_z: f32, // 节点的最小z值，也是节点自身的z值
  pub max_z: f32, // 节点的最大z值，z-index == -1, 则和min_z一样。
}

pub struct ZIndexImpl {
  dirty: LayerDirty,
  map: VecMap<ZIndex>,
  cache: Cache,
}

impl ZIndexImpl {
  pub fn new() -> ZIndexImpl {
    let mut map = VecMap::new();
    // 为root节点设置最大范围值
    let mut zi = ZIndex::default();
    zi.pre_min_z = -Z_MAX;
    zi.pre_max_z = Z_MAX;
    zi.min_z = -Z_MAX;
    zi.pre_max_z = Z_MAX;
    map.insert(Root, zi);
    ZIndexImpl {
      dirty: LayerDirty::default(),
      map: map,
      cache: Cache::new(),
    }
  }

  // 设置节点对应堆叠上下文的节点脏
  fn set_parent_dirty(&mut self, mut id: usize, idtree: &IdTree) {
    while id > 0 {
      let zi = unsafe {self.map.get_unchecked_mut(id)};
      let node = unsafe {idtree.get_unchecked(id)};
      // 如果为z为auto，则向上找zindex不为auto的节点，zindex不为auto的节点有堆叠上下文
      if zi.old != AUTO {
        if zi.dirty == DirtyType::None {
          zi.dirty = DirtyType::Normal;
          self.dirty.mark(id, node.layer);
        }
        if (node.count as f32) < zi.pre_max_z - zi.pre_min_z {
          return;
        }
        // 如果z范围超过自身全部子节点及其下子节点数量，则继续向上设置脏，等calc_z调整以获得足够的z范围
      }
      id = node.parent;
    }
  }

  fn delete_dirty(&mut self, id: usize, layer: usize) {
    let zi = unsafe {self.map.remove_unchecked(id)};
    if layer > 0 && zi.dirty != DirtyType::None {
      self.dirty.delete(id, layer)
    }
    // 删除无需设脏，z值可以继续使用
  }
  // 整理方法
  fn calc(&mut self, idtree: &IdTree, zdepth: &mut MultiCaseImpl<Node, ZDepth>) {
    for id in self.dirty.iter() {
        let (min_z, max_z, recursive) = {
          let zi = unsafe {self.map.get_unchecked_mut(*id)};
          if zi.dirty == DirtyType::None {
            continue;
          }
          let b = zi.dirty == DirtyType::Normal;
          zi.dirty = DirtyType::None;
          zi.min_z = zi.pre_min_z;
          zi.max_z = zi.pre_max_z;
          (zi.min_z, zi.max_z, b)
        };
        let node = unsafe {idtree.get_unchecked(*id)};
        if node.layer == 0 {
          continue;
        }
        // 设置 z_depth, 其他系统会监听该值
        unsafe {zdepth.get_unchecked_write(*id)}.set_0(min_z);
        if node.count == 0 {
          continue;
        }
        self.cache.sort(&self.map, idtree, node.children.head, 0);
        if recursive {
          self.cache.calc(&mut self.map, idtree, zdepth, min_z, max_z, node.count);
        }else{
          self.cache.recursive_calc(&mut self.map, idtree, zdepth, min_z, max_z, node.count);
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
  fn sort(&mut self, map: &VecMap<ZIndex>, idtree: &IdTree, child: usize, mut order: usize) -> usize {
    // zindex为0或-1的不参与排序。 zindex排序。用heap排序，确定每个子节点的z范围。如果子节点的zindex==-1，则需要将其子节点纳入排序。
    for (id, n) in idtree.iter(child) {
      let zi = unsafe {map.get_unchecked(id)}.old;
      if zi == 0 {
          self.z_zero.push(ZSort(zi, order, id, n.count));
      }else if zi == -1 {
        self.z_auto.push(id);
        // 继续递归其子节点
        order = self.sort(map, idtree, n.children.head, order);
      }else if zi > 0 {
        self.node_heap.push(ZSort(zi, order, id, n.count));
      }else{
        self.negative_heap.push(ZSort(zi-1, order, id, n.count));
      }
      order+=1;
    }
    order
  }
  // 计算真正的z
  fn calc(&mut self, map: &mut VecMap<ZIndex>, idtree: &IdTree, zdepth: &mut MultiCaseImpl<Node, ZDepth>, mut min_z: f32, mut max_z: f32, count: usize) {
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
      adjust(map, idtree, zdepth, n_id, unsafe {idtree.get_unchecked(n_id)}, min_z, max_z, f32::NAN, 0.);
      min_z = max_z;
    }
    for n_id in &self.z_auto {
      adjust(map, idtree, zdepth, *n_id, unsafe {idtree.get_unchecked(*n_id)}, min_z, min_z, f32::NAN, 0.);
      min_z += 1.;
    }
    self.z_auto.clear();
    for &ZSort(_, _, n_id, c) in &self.z_zero {
      max_z = min_z + split + split * c as f32;
      adjust(map, idtree, zdepth, n_id, unsafe {idtree.get_unchecked(n_id)}, min_z, max_z, f32::NAN, 0.);
      min_z = max_z;
    }
    self.z_zero.clear();
    while let Some(ZSort(_, _, n_id, c)) = self.node_heap.pop() {
      max_z = min_z + split + split * c as f32;
      adjust(map, idtree, zdepth, n_id, unsafe {idtree.get_unchecked(n_id)}, min_z, max_z, f32::NAN, 0.);
      min_z = max_z;
    }
  }
// 计算真正的z
  fn recursive_calc(&mut self, map: &mut VecMap<ZIndex>, idtree: &IdTree, zdepth: &mut MultiCaseImpl<Node, ZDepth>, mut min_z: f32, mut max_z: f32, count: usize) {
    let auto_len = self.z_auto.len();
    // 计算大致的劈分间距
    let split = if count > auto_len {
      (max_z - min_z - 1. - auto_len as f32) / (count - auto_len) as f32
    }else{
      1.
    };
    let start = self.temp.len();
    min_z += 1.; // 第一个子节点的z，要在父节点z上加1
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
      let zi = unsafe{map.get_unchecked_mut(id)};
      zi.dirty = DirtyType::None;
      zi.min_z = min_z;
      zi.pre_min_z = min_z;
      zi.max_z = max_z;
      zi.pre_max_z = max_z;
      // 设置 z_depth, 其他系统会监听该值
      unsafe {zdepth.get_unchecked_write(id)}.set_0(min_z);
      if min_z == max_z {
        continue
      }
      let node = unsafe {idtree.get_unchecked(id)};
      if node.count == 0 {
        continue;
      }
      self.sort(map, idtree, node.children.head, 0);
      self.recursive_calc(map, idtree, zdepth, min_z, max_z, node.count);
    }
  }
}
//================================ 内部静态方法
// 设置自己所有非AUTO的子节点为强制脏
fn recursive_dirty(map: &mut VecMap<ZIndex>, dirty: &mut LayerDirty, idtree: &IdTree, child: usize) {
  for (id, n) in idtree.iter(child) {
    let zi = unsafe {map.get_unchecked_mut(id)};
    if zi.old == -1 {
      recursive_dirty(map, dirty, idtree, n.children.head);
    }else {
      zi.dirty = DirtyType::Recursive;
      dirty.mark(id, n.layer);
    }
  }
}

// 整理方法。z范围变小或相交，则重新扫描修改一次。两种情况。
// 1. 有min_z max_z，修改该节点，计算rate，递归调用。
// 2. 有min_z rate parent_min， 根据rate和新旧min, 计算新的min_z max_z。 要分辨是否为auto节点
fn adjust(map: &mut VecMap<ZIndex>, idtree: &IdTree, zdepth: &mut MultiCaseImpl<Node, ZDepth>, id: usize, node: &IdNode, min_z: f32, max_z: f32, rate: f32, parent_min: f32) {
  let (min, r, old_min) = {
    let zi = unsafe{map.get_unchecked_mut(id)};
    let (min, max) = if !rate.is_nan() {
      (((zi.pre_min_z - parent_min) * rate) + min_z + 1., ((zi.pre_max_z - parent_min) * rate) + min_z + 1.)
    }else{
      (min_z, max_z)
    };
    zi.pre_min_z = min;
    zi.pre_max_z = max;
    // 如果节点脏，则跳过，后面会进行处理
    if zi.dirty != DirtyType::None{
      return
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
    // 设置 z_depth, 其他系统会监听该值
    unsafe {zdepth.get_unchecked_write(id)}.set_0(min);
    // 判断是否为auto
    if min != max {
      (min, (max_z - min_z - 1.)/ (old_max_z - old_min_z), old_min_z)
    }else if !rate.is_nan() {
      // 如果是auto，则重用min_z, rate, parent_min
      (min_z, rate, parent_min)
    }else{
      return
    }
  };
  //递归计算子节点的z
  for (i, n) in idtree.iter(node.children.head) {
    adjust(map, idtree, zdepth, i, n, min, 0., r, old_min);
  }
}

// #[cfg(test)]
// #[cfg(not(feature = "web"))]
// use wcs::world::{World};
// #[cfg(test)]
// #[cfg(not(feature = "web"))]
// use wcs::component::{Builder};
// #[cfg(test)]
// #[cfg(not(feature = "web"))]
// use world_doc::component::node::{NodeBuilder, InsertType};
// #[cfg(test)]
// #[cfg(not(feature = "web"))]
// use super::node_count::{NodeCountSys};

// #[cfg(not(feature = "web"))]
// #[test]
// fn test(){
//     let mut world: World<WorldDocMgr, ()> = World::new(WorldDocMgr::new());
//     let _nc = NodeCountSys::init(&mut world.component_mgr);
//     let zz = ZIndexSys::init(&mut world.component_mgr);
//     let systems: Vec<Arc<System<(), WorldDocMgr>>> = vec![zz.clone()];
//     world.set_systems(systems);
//     test_world_zz(&mut world, zz);
// }

// #[cfg(test)]
// #[cfg(not(feature = "web"))]
// fn new_node(component_mgr: &mut WorldDocMgr, parent_id: usize) -> usize {
//     let node = NodeBuilder::new().build(&mut component_mgr.node);
//     component_mgr.get_node_mut(parent_id).insert_child(node, InsertType::Back).id
// }
// #[cfg(not(feature = "web"))]
// #[cfg(test)]
// fn test_world_zz(world: &mut World<WorldDocMgr, ()>, zz: Arc<ZIndexSys>){
//     let mgr = &mut world.component_mgr;
//     let body_id = new_node(mgr, 1);
//     world.run(());
//     let mgr = &mut world.component_mgr;
//     let root_id = new_node(mgr, body_id);
//     let temp_id = new_node(mgr, root_id);
//     let root_top_id = new_node(mgr, root_id);
//     world.run(());
//     let mgr = &mut world.component_mgr;
//     let node_0 = new_node(mgr, root_top_id);
//     let node_0_0 = new_node(mgr, node_0);
//     let node_0_1 = new_node(mgr, node_0);
//     let node_0_1_0 = new_node(mgr, node_0_1);
//     let node_0_1_0_0 = new_node(mgr, node_0_1_0);
 
//     world.run(());
//     println!("modify run-----------------------------------------");
//     let mgr = &mut world.component_mgr;
//     print_node(mgr, zz.clone(), 1);
//     print_node(mgr, zz.clone(), body_id);
//     print_node(mgr, zz.clone(), root_id);
//     print_node(mgr, zz.clone(), temp_id);
//     print_node(mgr, zz.clone(), root_top_id);
//     print_node(mgr, zz.clone(), node_0);
//     print_node(mgr, zz.clone(), node_0_0);
//     print_node(mgr, zz.clone(), node_0_1);
//     print_node(mgr, zz.clone(), node_0_1_0);
//     print_node(mgr, zz.clone(), node_0_1_0_0);

//     let node_1 = new_node(mgr, root_top_id);
//     let node_1_0 = new_node(mgr, node_1);
//     let node_1_1 = new_node(mgr, node_1);
//     let node_1_1_0 = new_node(mgr, node_1_1);
//     let node_1_1_0_0 = new_node(mgr, node_1_1_0);
//     println!("modify2 run-----------------------------------------");
//     world.run(());
//     print_node(&world.component_mgr, zz.clone(), 1);
//     print_node(&world.component_mgr, zz.clone(), body_id);
//     print_node(&world.component_mgr, zz.clone(), root_id);
//     print_node(&world.component_mgr, zz.clone(), temp_id);
//     print_node(&world.component_mgr, zz.clone(), root_top_id);
//     print_node(&world.component_mgr, zz.clone(), node_0);
//     print_node(&world.component_mgr, zz.clone(), node_0_0);
//     print_node(&world.component_mgr, zz.clone(), node_0_1);
//     print_node(&world.component_mgr, zz.clone(), node_0_1_0);
//     print_node(&world.component_mgr, zz.clone(), node_0_1_0_0);
//     print_node(&world.component_mgr, zz.clone(), node_1);
//     print_node(&world.component_mgr, zz.clone(), node_1_0);
//     print_node(&world.component_mgr, zz.clone(), node_1_1);
//     print_node(&world.component_mgr, zz.clone(), node_1_1_0);
//     print_node(&world.component_mgr, zz.clone(), node_1_1_0_0);
// }
// #[cfg(not(feature = "web"))]
// #[cfg(test)]
// fn test_world_z(world: &mut World<WorldDocMgr, ()>, zz: Arc<ZIndexSys>){
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

// #[cfg(not(feature = "web"))]
// #[cfg(test)]
// fn print_node(mgr: &WorldDocMgr, zz: Arc<ZIndexSys>, id: usize) {
//     let node = mgr.node._group.get(id);
//     let zimpl = zz.0.borrow_mut();
//     let zi = unsafe{zimpl.map.get_unchecked(id)};

//     println!("nodeid: {}, zindex: {:?}, z_depth: {}, zz: {:?}, count: {}, parent: {}", id, node.zindex, node.z_depth, zi, node.count, node.parent);
// }