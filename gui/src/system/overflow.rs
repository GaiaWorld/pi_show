//裁剪矩形系统
// 容器设置了overflow的，就会产生一个裁剪矩形及对应的编号（编号都是2的次方），其下的所有的物件的by_overflow将会被设置为受到该id的影响
// 因为很少来回变动，所以直接根据变化进行设置，不采用dirty
// TODO 可以在没有旋转的情况下，使用包围盒来描述（使用第一个点的x为NaN来标识），提升query和渲染的性能。以后支持clip_path
// TODO engine对象封装GLContext,ResMgr, style要整理一下,现在有点乱, 移除text_layout
// TODO CowList, MapAPI统一， VecMap性能优化, remove() -> T 改成 Option<T>
// TODO EntityImpl 改为 Entity, Entity 改为 EntityT
// TODO 支持全局设置em对应的px


use ecs::{
  system::{MultiCaseListener, SingleCaseListener, EntityListener},
  monitor::{CreateEvent, DeleteEvent, ModifyEvent},
  component::MultiCaseImpl,
  single::SingleCaseImpl,
  idtree::IdTree,
};

use entity::{Node};
use component::{
  user::*,
  calc::*,
};
use single::OverflowClip;



pub struct OverflowImpl;

impl<'a> EntityListener<'a, Node, CreateEvent> for OverflowImpl {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, Overflow>, &'a mut MultiCaseImpl<Node, ByOverflow>);

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
      write.1.insert(event.id, ByOverflow::default());
    }
}

type Read<'a> = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Overflow>, &'a MultiCaseImpl<Node, WorldMatrix>, &'a MultiCaseImpl<Node, Transform>, &'a MultiCaseImpl<Node, Layout>);
type Write<'a> = (&'a mut SingleCaseImpl<OverflowClip>, &'a mut MultiCaseImpl<Node, ByOverflow>);

//监听overflow属性的改变
impl<'a> MultiCaseListener<'a, Node, Overflow, ModifyEvent> for OverflowImpl {
  type ReadData = Read<'a>;
  type WriteData = Write<'a>;

  fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
    let node = unsafe{ read.0.get_unchecked(event.id)};
    if node.layer == 0 {
      return
    }
    let overflow = match read.1.get(event.id){Some(r) => **r, _ => false};
    let mut by = **unsafe{ write.1.get_unchecked(event.id)};
    let index = if overflow {
      // 添加根上的overflow的裁剪矩形
      let i = set_index(&mut *write.0, 0, event.id);
      if i == 0 {
        return;
      }
      set_clip(event.id, i, &read, write.0);
      by |= i;
      i
    }else{
      // 删除根上的overflow的裁剪矩形
      let i = set_index(&mut *write.0, event.id, 0);
      if i == 0 {
        return;
      }
      by &=!i;
      i
    };
    if by & index != 0 {
      adjust(&read.0, write.1, node.children.head, index, add_index);
    }else{
      adjust(&read.0, write.1, node.children.head, index, del_index);
    }
    write.0.get_notify().modify_event(0, "", 0)
  }
}
//监听WorldMatrix组件的修改
impl<'a> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for OverflowImpl {
  type ReadData = Read<'a>;
  type WriteData = &'a mut SingleCaseImpl<OverflowClip>;

  fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
    let node = unsafe{ read.0.get_unchecked(event.id)};
    if node.layer == 0 {
      return
    }
    let overflow = match read.1.get(event.id){Some(r) => **r, _ => false};
    if overflow {
      let i = get_index(&mut *write, event.id);
      if i > 0 {
        set_clip(event.id, i, &read, write)
      }
    }
  }
}

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for OverflowImpl {
  type ReadData = Read<'a>;
  type WriteData = Write<'a>;

  fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, mut write: Self::WriteData) {
    let node = unsafe{ read.0.get_unchecked(event.id)};
    // 获得父节点的ByOverflow
    let by = **unsafe{ write.1.get_unchecked(node.parent)};
    let mut modify = false;
    set_overflow(event.id, by, &read, &mut write, &mut modify);
    if modify {
      write.0.get_notify().modify_event(0, "", 0)
    }
  }
}

impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for OverflowImpl {
  type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Overflow>);
  type WriteData = &'a mut SingleCaseImpl<OverflowClip>;

  fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData) {
    let node = unsafe{ read.0.get_unchecked(event.id)};
    let overflow = match read.1.get(event.id){Some(r) => **r, _ => false};
    let mut b = false;
    if overflow {
      set_index(&mut *write, event.id, 0);
      b = true;
    }
    // 递归调用，检查是否有overflow， 撤销设置OverflowClip
    for (id, _) in read.0.recursive_iter(node.children.head) {
      let overflow = match read.1.get(id){Some(r) => **r, _ => false};
      if overflow {
        set_index(&mut *write, id, 0);
        b = true;
      }
    }
    if b {
      write.get_notify().modify_event(0, "", 0)
    }
  }
}

//================================ 内部静态方法
// 设置裁剪区域，需要 world_matrix layout
fn set_clip(id: usize, i: usize, read: &Read, clip: &mut SingleCaseImpl<OverflowClip>) {
  let world_matrix = unsafe{ read.2.get_unchecked(id)};
  let layout = unsafe{ read.4.get_unchecked(id)};
  let origin = match read.3.get(id) {
    Some(transform) => {
      transform.origin.to_value(layout.width, layout.height)
    },
    _ => Point2::default()
  };
  clip.clip[i-1] = calc_point(layout, world_matrix, &origin);
}
// 递归调用，检查是否有overflow， 设置OverflowClip， 设置所有子元素的by_overflow
fn set_overflow(id: usize, mut by: usize, read: &Read, write: &mut Write, modify: &mut bool) {
  if by > 0 {
    unsafe {write.1.get_unchecked_write(by)};
  }
  let overflow = match read.1.get(id){Some(r) => **r, _ => false};
  if overflow {
    // 添加根上的overflow的裁剪矩形
    let i = set_index(&mut *write.0, 0, id);
    if i > 0 {
      set_clip(id, i, read, write.0);
      by |= i;
      *modify = true;
    }
  }
  let node = unsafe{ read.0.get_unchecked(id)};
  for (id, _n) in read.0.iter(node.children.head) {
    set_overflow(id, by, read, write, modify)
  }
}

// 寻找指定当前值cur的偏移量
#[inline]
fn get_index(overflow: &OverflowClip, cur: usize) -> usize {
  for i in 0..overflow.id_vec.len() {
    if cur == overflow.id_vec[i] {
      return i + 1;
    }
  }
  0
}
// 寻找指定当前值cur的偏移量, 设置成指定的值. 返回偏移量, 0表示没找到
#[inline]
fn set_index(overflow: &mut OverflowClip, cur: usize, value: usize) -> usize {
  let i = get_index(overflow, cur);
  if i > 0 {
    overflow.id_vec[i-1] = value;
  }
  i
}
#[inline]
fn add_index(by: usize, index: usize) ->usize {
  by | index
}
#[inline]
fn del_index(by: usize, index: usize) ->usize {
  by & !index
}
// 整理方法。设置或取消所有子节点的by_overflow上的index。
#[inline]
fn adjust(idtree: &SingleCaseImpl<IdTree>, by_overflow: &mut MultiCaseImpl<Node, ByOverflow>, child: usize, index: usize, ops: fn(a:usize, b:usize)->usize) {
  for (id, _n) in idtree.recursive_iter(child) {
    let by = **unsafe {by_overflow.get_unchecked(id)};
    unsafe {by_overflow.get_unchecked_write(ops(by, index))};
  }
}
// 计算指定矩形的4个点
fn calc_point(layout: &Layout, m: &Matrix4, origin: &Point2) -> [Point2;4]{
    let width = layout.width - layout.padding_left - layout.padding_right - layout.border_left - layout.border_right;
    let height = layout.height - layout.padding_top - layout.padding_bottom - layout.border_top - layout.border_bottom;
    let start = (layout.border_left + layout.padding_left - origin.x, layout.border_top + layout.padding_top - origin.y);
    let left_top = m * Vector4::new(start.0,  start.1, 0.0, 1.0);
    let right_top = m * Vector4::new(start.0 + width, start.1, 0.0, 1.0);
    let left_bottom = m * Vector4::new(start.0, start.1 + height, 0.0, 1.0);
    let right_bottom = m * Vector4::new(start.1 + width, start.1 + height, 0.0, 1.0);

    let lt = Point2{x: left_top.x, y: left_top.y};
    let rt = Point2{x: right_top.x, y: right_top.y};
    let lb = Point2{x: left_bottom.x, y: left_bottom.y};
    let rb = Point2{x: right_bottom.x, y: right_bottom.y};
    [lt, rt, lb, rb]
}

impl_system!{
    OverflowImpl,
    false,
    {
      EntityListener<Node, CreateEvent>
      MultiCaseListener<Node, Overflow, ModifyEvent>
      MultiCaseListener<Node, WorldMatrix, ModifyEvent>
      SingleCaseListener<IdTree, CreateEvent>
      SingleCaseListener<IdTree, DeleteEvent>
    }
}

// #[cfg(test)]
// #[cfg(not(feature = "web"))]
// use wcs::world::{World, System};
// #[cfg(test)]
// #[cfg(not(feature = "web"))]
// use wcs::component::{Builder};
// #[cfg(test)]
// #[cfg(not(feature = "web"))]
// use world_doc::component::node::{NodeBuilder, InsertType};

// #[cfg(not(feature = "web"))]
// #[test]
// fn test(){
//     let mut world: World<WorldDocMgr, ()> = World::new(WorldDocMgr::new());
//     let _zz = OverflowSys::init(&mut world.component_mgr);
//     let systems: Vec<Arc<System<(), WorldDocMgr>>> = vec![];
//     world.set_systems(systems);
//     test_world_overflow(&mut world);
// }

// #[cfg(not(feature = "web"))]
// #[cfg(test)]
// fn test_world_overflow(world: &mut World<WorldDocMgr, ()>){
//     let (root, node1, node2, node3, node4, node5) = {
//         let component_mgr = &mut world.component_mgr;
//         {
            
//             let (root, node1, node2, node3, node4, node5) = {
//                 let root = NodeBuilder::new().build(&mut component_mgr.node); // 创建根节点
//                 println!("root element: {:?}", root.element);
//                 let root_id = component_mgr.node._group.insert(root, 0);// 不通知的方式添加 NodeWriteRef{id, component_mgr write 'a Ref}
//                 let _n = component_mgr.node._group.get_mut(root_id);// ComponentNode{parent:usize, owner: 'a &mut Node}
//                 let node1 = NodeBuilder::new().build(&mut component_mgr.node);
//                 let node2 = NodeBuilder::new().build(&mut component_mgr.node);
//                 let node3 = NodeBuilder::new().build(&mut component_mgr.node);
//                 let node4 = NodeBuilder::new().build(&mut component_mgr.node);
//                 let node5 = NodeBuilder::new().build(&mut component_mgr.node);
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
//             print_node(component_mgr, node1);
//             print_node(component_mgr, node2);
//             print_node(component_mgr, node3);
//             print_node(component_mgr, node4);
//             print_node(component_mgr, node5);
//             (root, node1, node2, node3, node4, node5)
//         }
//     };
//     world.component_mgr.get_node_mut(node1).set_overflow(true);

//     println!("modify run-----------------------------------------");
//     world.run(());
//     println!("ooo:{:?}", world.component_mgr.world_2d.component_mgr.overflow.deref());
//     print_node(&world.component_mgr, root);
//     print_node(&world.component_mgr, node1);
//     print_node(&world.component_mgr, node2);
//     print_node(&world.component_mgr, node3);
//     print_node(&world.component_mgr, node4);
//     print_node(&world.component_mgr, node5);
// }

// #[cfg(not(feature = "web"))]
// #[cfg(test)]
// fn print_node(mgr: &WorldDocMgr, id: usize) {
//     let node = mgr.node._group.get(id);

//     println!("nodeid: {}, ov:{:?}, byov: {}", id, node.overflow, node.by_overflow);
// }