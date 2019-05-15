//裁剪矩形系统
// 容器设置了overflow的，就会产生一个裁剪矩形及对应的编号（编号都是2的次方），其下的所有的物件的by_overflow将会被设置为受到该id的影响
// 因为很少来回变动，所以直接根据变化进行设置，不采用dirty
// TODO 可以在没有旋转的情况下，使用包围盒来描述（使用第一个点的x为NaN来标识），提升query和渲染的性能。以后支持clip_path
// TODO node_count内置, world改成map记录所有的system,id设置run列表, engine对象封装GLContext,ResMgr, style要整理一下,现在有点乱, 移除text_layout
// TODO 以后改成1颗树就可以不要layout上面的转发属性监听器
// TODO 新ecs框架 节点tree 执行器 完美hash处理类型id问题， 监听器自动连接

use std::{
  f32,
  cmp::{Ordering},
};

use ecs::{
  system::{MultiCaseListener, SingleCaseListener, EntityListener},
  monitor::{CreateEvent, DeleteEvent, ModifyEvent},
  component::MultiCaseImpl,
  single::SingleCaseImpl,
  idtree::IdTree,
};

use entity::{Node};
use component::{
  Point2,
  user::{Overflow},
  calc::{ByOverflow, WorldMatrix},
};
use single::OverflowClip;



pub struct OverflowImpl;

impl<'a> EntityListener<'a, Node, CreateEvent> for OverflowImpl {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, Overflow>, &'a mut MultiCaseImpl<Node, ByOverflow>);

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
      write.0.insert(event.id, Overflow::default());
      write.1.insert(event.id, ByOverflow::default());
    }
}

//监听overflow属性的改变
impl<'a> MultiCaseListener<'a, Node, Overflow, ModifyEvent> for OverflowImpl {
  type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Overflow>, &'a MultiCaseImpl<Node, WorldMatrix>); // Transform, Layout
  type WriteData = (&'a mut SingleCaseImpl<OverflowClip>, &'a mut MultiCaseImpl<Node, ByOverflow>);

  fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
    let node = unsafe{ read.0.get_unchecked(event.id)};
    if node.layer == 0 {
      return
    }
    let overflow = **unsafe{ read.1.get_unchecked(event.id)};
    let mut by = **unsafe{ write.1.get_unchecked(event.id)};
    let index = if overflow {
      // 添加根上的overflow的裁剪矩形
      let i = set_index(&mut *write.0, 0, event.id);
      if i == 0 {
        return;
      }
      let world_matrix = unsafe{ read.2.get_unchecked(event.id)};
      // let origin = if node.transform == 0 {
      //     cg::Point2::new(0.0, 0.0)
      // }else {
      //   cg::Point2::new(0.0, 0.0)
      //     //mgr.node.transform._group.get(node.transform).origin.to_value(node.layout.width, node.layout.height)
      // };
      // mgr.world_2d.component_mgr.overflow.1[i-1] = calc_point(&node.layout, world_matrix, &origin);
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
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Overflow>, &'a MultiCaseImpl<Node, WorldMatrix>); // Transform, Layout
    type WriteData = &'a mut SingleCaseImpl<OverflowClip>;

    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
      let node = unsafe{ read.0.get_unchecked(event.id)};
      if node.layer == 0 {
        return
      }
      let overflow = **unsafe{ read.1.get_unchecked(event.id)};
      if overflow {
        let i = get_index(&mut *write, event.id);
        if i > 0 {
          let world_matrix = unsafe{ read.2.get_unchecked(event.id)};
//             let origin = if node.transform == 0 {
//                 cg::Point2::new(0.0, 0.0)
//             }else {
//                 mgr.node.transform._group.get(node.transform).origin.to_value(node.layout.width, node.layout.height)
//             };
//             mgr.world_2d.component_mgr.overflow.1[i-1] = calc_point(&node.layout, world_matrix, &origin);
          write.get_notify().modify_event(0, "", 0)
        }
      }
    }
}

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for OverflowImpl {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Overflow>, &'a MultiCaseImpl<Node, WorldMatrix>); // Transform, Layout
    type WriteData = (&'a mut SingleCaseImpl<OverflowClip>, &'a mut MultiCaseImpl<Node, ByOverflow>);

    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, _write: Self::WriteData) {
      let node = unsafe{ read.0.get_unchecked(event.id)};
      // 获得父节点的ByOverflow
      // 递归调用，检查是否有overflow， 设置OverflowClip， 设置所有子元素的by_overflow
    }
}

impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for OverflowImpl {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Overflow>);
    type WriteData = &'a mut SingleCaseImpl<OverflowClip>;

    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData) {
      let node = unsafe{ read.0.get_unchecked(event.id)};
      let overflow = **unsafe{ read.1.get_unchecked(event.id)};
      let mut b = false;
      if overflow {
        set_index(&mut *write, event.id, 0);
        b = true;
      }
      // 递归调用，检查是否有overflow， 撤销设置OverflowClip
      for (id, n) in read.0.recursive_iter(node.children.head) {
        let overflow = **unsafe{ read.1.get_unchecked(id)};
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
// //监听overflow属性的改变
// impl ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr> for OverflowSys {
//   fn handle(&self, event: &ModifyFieldEvent, mgr: &mut WorldDocMgr) {
//     let ModifyFieldEvent{id, parent: _, field: _} = event;
//     let (index, by, child) = {
//       let node = mgr.node._group.get_mut(*id);
//       if node.overflow {
//         // 添加根上的overflow的裁剪矩形
//         let i = set_index(&mut mgr.world_2d.component_mgr.overflow, 0, *id);
//         if i == 0 {
//           return;
//         }
//         let world_matrix = mgr.node.world_matrix._group.get(node.world_matrix);
//         let origin = if node.transform == 0 {
//             cg::Point2::new(0.0, 0.0)
//         }else {
//             mgr.node.transform._group.get(node.transform).origin.to_value(node.layout.width, node.layout.height)
//         };
//         mgr.world_2d.component_mgr.overflow.1[i-1] = calc_point(&node.layout, world_matrix, &origin);
//         (i, node.by_overflow | i, node.get_childs_mut().get_first())
//       }else{
//         // 删除根上的overflow的裁剪矩形
//         let i = set_index(&mut mgr.world_2d.component_mgr.overflow, *id, 0);
//         if i == 0 {
//           return;
//         }
//         (i, node.by_overflow & !i, node.get_childs_mut().get_first())
//       }
//     };
//     if by & index != 0 {
//       adjust(mgr, child, index, add_index);
//     }else{
//       adjust(mgr, child, index, del_index);
//     }
//     mgr.world_2d.component_mgr.overflow.handlers.clone().notify(SingleModifyEvent{field:""}, &mut mgr.world_2d.component_mgr);
//   }
// }
// //监听了Matrix组件的修改
// impl ComponentHandler<Matrix4, ModifyFieldEvent, WorldDocMgr> for OverflowSys{
//     fn handle(&self, event: &ModifyFieldEvent, mgr: &mut WorldDocMgr){
//         let ModifyFieldEvent{id, parent, field: _} = event;
//         let node = mgr.node._group.get(*parent);
//         if node.overflow {
//           let i = get_index(&mgr.world_2d.component_mgr.overflow, *parent);
//           if i > 0 {
//             let world_matrix = mgr.node.world_matrix._group.get(*id);
//             let origin = if node.transform == 0 {
//                 cg::Point2::new(0.0, 0.0)
//             }else {
//                 mgr.node.transform._group.get(node.transform).origin.to_value(node.layout.width, node.layout.height)
//             };
//             mgr.world_2d.component_mgr.overflow.1[i-1] = calc_point(&node.layout, world_matrix, &origin);
//             mgr.world_2d.component_mgr.overflow.handlers.clone().notify(SingleModifyEvent{field:""}, &mut mgr.world_2d.component_mgr);
//           }
//         }
//     }
// }
// //监听Node的创建
// impl ComponentHandler<Node, CreateEvent, WorldDocMgr> for OverflowSys {
//   fn handle(&self, event: &CreateEvent, mgr: &mut WorldDocMgr) {
//     let CreateEvent{id, parent} = event;
//     // 检查该节点是否有overflow, 如果有,则其自身的by_overflow受overflow影响
//     let node = mgr.node._group.get_mut(*id);
//     if node.overflow { // 其裁剪矩形需要等Matrix被设置时设置
//       set_index(&mut mgr.world_2d.component_mgr.overflow, 0, *id);
//       mgr.world_2d.component_mgr.overflow.handlers.clone().notify(SingleModifyEvent{field:""}, &mut mgr.world_2d.component_mgr);
//     }
//     // 根据该节点的父容器是否有by_overflow及overflow, 设置自身的by_overflow
//     let pnode = mgr.node._group.get_mut(*parent);
//     let b = if pnode.overflow {
//       pnode.by_overflow | get_index(&mgr.world_2d.component_mgr.overflow, *parent)
//     }else{
//       pnode.by_overflow
//     };
//     if b > 0 {
//       mgr.get_node_mut(*id).set_by_overflow(b);
//     }
//   }
// }
// //监听Node的删除创建， 删除脏标志
// impl ComponentHandler<Node, DeleteEvent, WorldDocMgr> for OverflowSys {
//   fn handle(&self, event: &DeleteEvent, mgr: &mut WorldDocMgr) {
//     let DeleteEvent{id, parent: _} = event;
//     // 检查该节点是否有overflow
//     if mgr.node._group.get_mut(*id).overflow {
//       // 删除根上的overflow的裁剪矩形
//       if set_index(&mut mgr.world_2d.component_mgr.overflow, *id, 0) > 0 {
//         mgr.world_2d.component_mgr.overflow.handlers.clone().notify(SingleModifyEvent{field:""}, &mut mgr.world_2d.component_mgr);
//       }
//     }
//   }
// }


//================================ 内部静态方法
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
fn adjust<'a>(idtree: &'a SingleCaseImpl<IdTree>, by_overflow: &'a mut MultiCaseImpl<Node, ByOverflow>, child: usize, index: usize, ops: fn(a:usize, b:usize)->usize) {
  for (id, _n) in idtree.recursive_iter(child) {
    let by = **unsafe {by_overflow.get_unchecked(id)};
    unsafe {by_overflow.get_unchecked_write(ops(by, index))};
  }
}
// 计算指定矩形的4个点
// fn calc_point(layout: &Layout, matrix: &Matrix4, origin: &cg::Point2<f32>) -> [Point2;4]{
//     let m = matrix.deref();
//     let width = layout.width - layout.padding_left - layout.padding_right - layout.border - layout.border;
//     let height = layout.height - layout.padding_top - layout.padding_bottom - layout.border - layout.border;
//     let start = (layout.border + layout.padding_left - origin.x, layout.border + layout.padding_top - origin.y);
//     let left_top = m * Vector4::new(start.0,  start.1, 0.0, 1.0);
//     let right_top = m * Vector4::new(start.0 + width, start.1, 0.0, 1.0);
//     let left_bottom = m * Vector4::new(start.0, start.1 + height, 0.0, 1.0);
//     let right_bottom = m * Vector4::new(start.1 + width, start.1 + height, 0.0, 1.0);

//     let lt = Point2(cg::Point2{x: left_top.x, y: left_top.y});
//     let rt = Point2(cg::Point2{x: right_top.x, y: right_top.y});
//     let lb = Point2(cg::Point2{x: left_bottom.x, y: left_bottom.y});
//     let rb = Point2(cg::Point2{x: right_bottom.x, y: right_bottom.y});
//     [lt, rt, lb, rb]
// }

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
//     let systems: Vec<Rc<System<(), WorldDocMgr>>> = vec![];
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