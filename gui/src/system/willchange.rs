// 快速变换系统
// 容器设置了willchange的，就会产生一个Transform及对应的编号（编号都是2的次方），其下的所有的物件的by_willchange将会被设置为受到该id的影响
// 一般在进行变换前设置在相应的组件上， 变换完后应该取消


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
use single::WillChangeTransform;
use super::overflow::*;



pub struct WillChangeImpl;

impl<'a> EntityListener<'a, Node, CreateEvent> for WillChangeImpl {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, WillChange>, &'a mut MultiCaseImpl<Node, ByWillChange>);

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
      write.1.insert(event.id, ByWillChange::default());
    }
}

type Read<'a> = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, WillChange>);
type Write<'a> = (&'a mut SingleCaseImpl<WillChangeTransform>, &'a mut MultiCaseImpl<Node, ByWillChange>);

//监听willchange属性的改变
impl<'a> MultiCaseListener<'a, Node, WillChange, ModifyEvent> for WillChangeImpl {
  type ReadData = Read<'a>;
  type WriteData = Write<'a>;

  fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
    let node = unsafe{ read.0.get_unchecked(event.id)};
    if node.layer == 0 {
      return
    }
    let willchange = match read.1.get(event.id){Some(r) => **r, _ => false};
    let mut by = **unsafe{ write.1.get_unchecked(event.id)};
    let index = if willchange {
      // 添加根上的willchange的Transform
      let i = set_index(&mut write.0.id_arr, 0, event.id);
      if i == 0 {
        return;
      }
      // TODO set_clip(event.id, i, &read, write.0);
      by |= i;
      i
    }else{
      // 删除根上的willchange的Transform
      let i = set_index(&mut write.0.id_arr, event.id, 0);
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

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for WillChangeImpl {
  type ReadData = Read<'a>;
  type WriteData = Write<'a>;

  fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, mut write: Self::WriteData) {
    let node = unsafe{ read.0.get_unchecked(event.id)};
    // 获得父节点的ByWillChange
    let by = **unsafe{ write.1.get_unchecked(node.parent)};
    let mut modify = false;
    set_willchange(event.id, by, &read, &mut write, &mut modify);
    if modify {
      write.0.get_notify().modify_event(0, "", 0)
    }
  }
}

impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for WillChangeImpl {
  type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, WillChange>);
  type WriteData = &'a mut SingleCaseImpl<WillChangeTransform>;

  fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData) {
    let node = unsafe{ read.0.get_unchecked(event.id)};
    let willchange = match read.1.get(event.id){Some(r) => **r, _ => false};
    let mut b = false;
    if willchange {
      set_index(&mut write.id_arr, event.id, 0);
      b = true;
    }
    // 递归调用，检查是否有willchange， 撤销设置WillChangeTransform
    for (id, _) in read.0.recursive_iter(node.children.head) {
      let willchange = match read.1.get(id){Some(r) => **r, _ => false};
      if willchange {
        set_index(&mut write.id_arr, id, 0);
        b = true;
      }
    }
    if b {
      write.get_notify().modify_event(0, "", 0)
    }
  }
}
//================================ 内部静态方法

// 递归调用，检查是否有willchange， 设置WillChangeTransform， 设置所有子元素的by_willchange
fn set_willchange(id: usize, mut by: usize, read: &Read, write: &mut Write, modify: &mut bool) {
  if by > 0 {
    unsafe {write.1.get_unchecked_write(by)};
  }
  let willchange = match read.1.get(id){Some(r) => **r, _ => false};
  if willchange {
    // 添加根上的willchange的Transform
    let i = set_index(&mut write.0.id_arr, 0, id);
    if i > 0 {
      // TODO set_clip(id, i, read, write.0);
      by |= i;
      *modify = true;
    }
  }
  let node = unsafe{ read.0.get_unchecked(id)};
  for (id, _n) in read.0.iter(node.children.head) {
    set_willchange(id, by, read, write, modify)
  }
}

// 整理方法。设置或取消所有子节点的by_willchange上的index。
#[inline]
fn adjust(idtree: &SingleCaseImpl<IdTree>, by_willchange: &mut MultiCaseImpl<Node, ByWillChange>, child: usize, index: usize, ops: fn(a:usize, b:usize)->usize) {
  for (id, _n) in idtree.recursive_iter(child) {
    let by = **unsafe {by_willchange.get_unchecked(id)};
    unsafe {by_willchange.get_unchecked_write(ops(by, index))};
  }
}