//裁剪矩形系统
// 容器设置了overflow的，就会产生一个裁剪矩形及对应的编号（编号都是2的次方），其下的所有的物件的by_overflow将会被设置为受到该id的影响
// 因为很少来回变动，所以直接根据变化进行设置，不采用dirty
// TODO 可以在没有旋转的情况下，使用包围盒来描述（使用第一个点的x为NaN来标识），提升query和渲染的性能


use std::rc::Rc;
use std::ops::Deref;

use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent, ModifyFieldEvent, SingleModifyEvent};
use wcs::world::System;
use cg::{Vector4};

use world_doc::{WorldDocMgr};
use world_doc::component::node::{Node};
use component::math::{Matrix4, Point2};
use world_2d::{Overflow};
use layout::Layout;

pub struct OverflowSys();

impl OverflowSys {
  pub fn init(mgr: &mut WorldDocMgr) -> Rc<OverflowSys> {
    let rc = Rc::new(OverflowSys());
    mgr.node.overflow.register_handler(Rc::downgrade(&(rc.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
    mgr.node._group.register_create_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<Node, CreateEvent, WorldDocMgr>>),
    ));
    mgr.node._group.register_delete_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<Node, DeleteEvent, WorldDocMgr>>),
    ));
    mgr.node.world_matrix._group.register_modify_field_handler(Rc::downgrade(&(rc.clone() as Rc<ComponentHandler<Matrix4, ModifyFieldEvent, WorldDocMgr>>)));
    rc
  }
}

impl System<(), WorldDocMgr> for OverflowSys{
    fn run(&self, _e: &(), _component_mgr: &mut WorldDocMgr){
    }
}

//监听overflow属性的改变
impl ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr> for OverflowSys {
  fn handle(&self, event: &ModifyFieldEvent, mgr: &mut WorldDocMgr) {
    let ModifyFieldEvent{id, parent: _, field: _} = event;
    let (index, by, child) = {
      let node = mgr.node._group.get_mut(*id);
      if node.overflow {
        // 添加根上的overflow的裁剪矩形
        let i = set_index(&mut mgr.world_2d.component_mgr.overflow, 0, *id);
        if i == 0 {
          return;
        }
        (i, node.by_overflow | i, node.get_childs_mut().get_first())
      }else{
        // 删除根上的overflow的裁剪矩形
        let i = set_index(&mut mgr.world_2d.component_mgr.overflow, *id, 0);
        if i == 0 {
          return;
        }
        (i, node.by_overflow & !i, node.get_childs_mut().get_first())
      }
    };
    if by & index != 0 {
      adjust(mgr, child, index, add_index);
    }else{
      adjust(mgr, child, index, del_index);
    }
    mgr.world_2d.component_mgr.overflow.handlers.clone().notify(SingleModifyEvent{field:""}, &mut mgr.world_2d.component_mgr);
  }
}
//监听了Matrix组件的修改
impl ComponentHandler<Matrix4, ModifyFieldEvent, WorldDocMgr> for OverflowSys{
    fn handle(&self, event: &ModifyFieldEvent, mgr: &mut WorldDocMgr){
        let ModifyFieldEvent{id, parent, field: _} = event;
        let node = mgr.node._group.get(*parent);
        if node.overflow {
          let i = get_index(&mgr.world_2d.component_mgr.overflow, *parent);
          if i > 0 {
            let world_matrix = mgr.node.world_matrix._group.get(*id);
            mgr.world_2d.component_mgr.overflow.1[i-1] = calc_point(&node.layout, world_matrix);
            mgr.world_2d.component_mgr.overflow.handlers.clone().notify(SingleModifyEvent{field:""}, &mut mgr.world_2d.component_mgr);
          }
        }
    }
}
//监听Node的创建
impl ComponentHandler<Node, CreateEvent, WorldDocMgr> for OverflowSys {
  fn handle(&self, event: &CreateEvent, mgr: &mut WorldDocMgr) {
    let CreateEvent{id, parent} = event;
    // 检查该节点是否有overflow, 如果有,则其自身的by_overflow受overflow影响
    let node = mgr.node._group.get_mut(*id);
    if node.overflow { // 其裁剪矩形需要等Matrix被设置时设置
      set_index(&mut mgr.world_2d.component_mgr.overflow, 0, *id);
      mgr.world_2d.component_mgr.overflow.handlers.clone().notify(SingleModifyEvent{field:""}, &mut mgr.world_2d.component_mgr);
    }
    // 根据该节点的父容器是否有by_overflow及overflow, 设置自身的by_overflow
    let pnode = mgr.node._group.get_mut(*parent);
    let b = if pnode.overflow {
      pnode.by_overflow | get_index(&mgr.world_2d.component_mgr.overflow, *parent)
    }else{
      pnode.by_overflow
    };
    if b > 0 {
      mgr.get_node_mut(*id).set_by_overflow(b);
    }
  }
}
//监听Node的删除创建， 删除脏标志
impl ComponentHandler<Node, DeleteEvent, WorldDocMgr> for OverflowSys {
  fn handle(&self, event: &DeleteEvent, mgr: &mut WorldDocMgr) {
    let DeleteEvent{id, parent: _} = event;
    // 检查该节点是否有overflow
    if mgr.node._group.get_mut(*id).overflow {
      // 删除根上的overflow的裁剪矩形
      if set_index(&mut mgr.world_2d.component_mgr.overflow, *id, 0) > 0 {
        mgr.world_2d.component_mgr.overflow.handlers.clone().notify(SingleModifyEvent{field:""}, &mut mgr.world_2d.component_mgr);
      }
    }
  }
}


//================================ 内部静态方法
// 寻找指定当前值cur的偏移量
#[inline]
fn get_index(overflow: &Overflow, cur: usize) -> usize {
  for i in 0..overflow.0.len() {
    if cur == overflow.0[i] {
      return i + 1;
    }
  }
  0
}
// 寻找指定当前值cur的偏移量, 设置成指定的值. 返回偏移量, 0表示没找到
#[inline]
fn set_index(overflow: &mut Overflow, cur: usize, value: usize) -> usize {
  let i = get_index(overflow, cur);
  if i > 0 {
    overflow.0[i-1] = value;
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
fn adjust(mgr: &mut WorldDocMgr, mut child: usize, index: usize, ops: fn(a:usize, b:usize)->usize) {
  //递归计算子节点的z
  loop {
    if child == 0 {
        return;
    }
    let child_child = {
        let v = unsafe{ mgr.node_container.get_unchecked(child) };
        child = v.next;
        let node_id = v.elem.clone();
        let (by, c) = {
          let node = mgr.node._group.get_mut(node_id);
          (node.by_overflow, node.get_childs_mut().get_first())
        };
        mgr.get_node_mut(node_id).set_by_overflow(ops(by, index));
        c
    };
    adjust(mgr, child_child, index, ops);
  }
}
// 计算指定矩形的4个点
fn calc_point(layout: &Layout, matrix: &Matrix4) -> [Point2;4]{
    let half_width = layout.width/2.0;
    let half_height = layout.height/2.0;
    let m = matrix.deref();
    let left_top = m * Vector4::new(-half_width + layout.border + layout.padding_left, -half_height + layout.border + layout.padding_top, 0.0, 1.0);
    let right_top = m * Vector4::new(half_width - layout.border - layout.padding_right, -half_height + layout.border + layout.padding_top , 0.0, 1.0);
    let left_bottom = m * Vector4::new(-half_width + layout.border + layout.padding_left, half_height - layout.border - layout.padding_bottom, 0.0, 1.0);
    let right_bottom = m * Vector4::new(half_width - layout.border - layout.padding_right, half_height - layout.border - layout.padding_bottom, 0.0, 1.0);

    let lt = Point2(cg::Point2{x: left_top.x, y: left_top.y});
    let rt = Point2(cg::Point2{x: right_top.x, y: right_top.y});
    let lb = Point2(cg::Point2{x: left_bottom.x, y: left_bottom.y});
    let rb = Point2(cg::Point2{x: right_bottom.x, y: right_bottom.y});
    [lt, rt, lb, rb]
}

#[cfg(test)]
#[cfg(not(feature = "web"))]
use wcs::world::{World, System};
#[cfg(test)]
#[cfg(not(feature = "web"))]
use wcs::component::{Builder};
#[cfg(test)]
#[cfg(not(feature = "web"))]
use world_doc::component::node::{NodeBuilder, InsertType};

#[cfg(not(feature = "web"))]
#[test]
fn test(){
    let mut world: World<WorldDocMgr, ()> = World::new(WorldDocMgr::new());
    let _zz = OverflowSys::init(&mut world.component_mgr);
    let systems: Vec<Rc<System<(), WorldDocMgr>>> = vec![];
    world.set_systems(systems);
    test_world_overflow(&mut world);
}

#[cfg(not(feature = "web"))]
#[cfg(test)]
fn test_world_overflow(world: &mut World<WorldDocMgr, ()>){
    let (root, node1, node2, node3, node4, node5) = {
        let component_mgr = &mut world.component_mgr;
        {
            
            let (root, node1, node2, node3, node4, node5) = {
                let root = NodeBuilder::new().build(&mut component_mgr.node); // 创建根节点
                println!("root element: {:?}", root.element);
                let root_id = component_mgr.node._group.insert(root, 0);// 不通知的方式添加 NodeWriteRef{id, component_mgr write 'a Ref}
                let _n = component_mgr.node._group.get_mut(root_id);// ComponentNode{parent:usize, owner: 'a &mut Node}
                let node1 = NodeBuilder::new().build(&mut component_mgr.node);
                let node2 = NodeBuilder::new().build(&mut component_mgr.node);
                let node3 = NodeBuilder::new().build(&mut component_mgr.node);
                let node4 = NodeBuilder::new().build(&mut component_mgr.node);
                let node5 = NodeBuilder::new().build(&mut component_mgr.node);
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
            print_node(component_mgr, node1);
            print_node(component_mgr, node2);
            print_node(component_mgr, node3);
            print_node(component_mgr, node4);
            print_node(component_mgr, node5);
            (root, node1, node2, node3, node4, node5)
        }
    };
    world.component_mgr.get_node_mut(node1).set_overflow(true);

    println!("modify run-----------------------------------------");
    world.run(());
    println!("ooo:{:?}", world.component_mgr.world_2d.component_mgr.overflow.deref());
    print_node(&world.component_mgr, root);
    print_node(&world.component_mgr, node1);
    print_node(&world.component_mgr, node2);
    print_node(&world.component_mgr, node3);
    print_node(&world.component_mgr, node4);
    print_node(&world.component_mgr, node5);
}

#[cfg(not(feature = "web"))]
#[cfg(test)]
fn print_node(mgr: &WorldDocMgr, id: usize) {
    let node = mgr.node._group.get(id);

    println!("nodeid: {}, ov:{:?}, byov: {}", id, node.overflow, node.by_overflow);
}