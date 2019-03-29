//裁剪矩形系统
// 容器设置了overflow的，就会产生一个裁剪矩形及对应的编号（编号都是2的次方），其下的所有的物件的by_overflow将会被设置为受到该id的影响
// 因为很少来回变动，所以直接根据变化进行设置，不采用dirty


use std::rc::Rc;
use std::ops::Deref;

use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent, ModifyFieldEvent, SingleModifyEvent};
use cg::{Vector4};

use world::{GuiComponentMgr, Overflow};
use component::node::{Node, RectSize};
use component::math::{Matrix4, Vector3, Point2};

pub struct OverflowSys();

impl OverflowSys {
  pub fn init(mgr: &mut GuiComponentMgr) -> Rc<OverflowSys> {
    let rc = Rc::new(OverflowSys());
    mgr.node.overflow.register_handler(Rc::downgrade(&(rc.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, GuiComponentMgr>>)));
    mgr.node._group.register_create_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<Node, CreateEvent, GuiComponentMgr>>),
    ));
    mgr.node._group.register_delete_handler(Rc::downgrade(
      &(rc.clone() as Rc<ComponentHandler<Node, DeleteEvent, GuiComponentMgr>>),
    ));
    mgr.node.world_matrix._group.register_modify_field_handler(Rc::downgrade(&(rc.clone() as Rc<ComponentHandler<Matrix4, ModifyFieldEvent, GuiComponentMgr>>)));
    rc
  }
}

//监听overflow属性的改变
impl ComponentHandler<Node, ModifyFieldEvent, GuiComponentMgr> for OverflowSys {
  fn handle(&self, event: &ModifyFieldEvent, mgr: &mut GuiComponentMgr) {
    let ModifyFieldEvent{id, parent: _, field: _} = event;
    let (index, by, child) = {
      let node = mgr.node._group.get_mut(*id);
      if node.overflow {
        // 添加根上的overflow的裁剪矩形
        let i = set_index(&mut mgr.overflow, 0, *id);
        if i == 0 {
          return;
        }
        (i, node.by_overflow | i, node.get_childs_mut().get_first())
      }else{
        // 删除根上的overflow的裁剪矩形
        let i = set_index(&mut mgr.overflow, *id, 0);
        if i == 0 {
          return;
        }
        (i, node.by_overflow & !i, node.get_childs_mut().get_first())
      }
    };
    mgr.get_node_mut(*id).set_by_overflow(by);
    if by & index != 0 {
      adjust(mgr, child, index, add_index);
    }else{
      adjust(mgr, child, index, del_index);
    }
    mgr.overflow.get_handlers().notify(SingleModifyEvent{field:""}, mgr);
  }
}
//监听了Matrix组件的修改
impl ComponentHandler<Matrix4, ModifyFieldEvent, GuiComponentMgr> for OverflowSys{
    fn handle(&self, event: &ModifyFieldEvent, mgr: &mut GuiComponentMgr){
        let ModifyFieldEvent{id, parent, field: _} = event;
        let node = mgr.node._group.get(*parent);
        if node.overflow {
          let i = get_index(&mgr.overflow, *parent);
          if i > 0 {
            let world_matrix = mgr.node.world_matrix._group.get(*id);
            let pos = mgr.node.position._group.get(node.position);
            let size = mgr.node.extent._group.get(node.extent);
            mgr.overflow.1[i] = calc_point(pos, size, world_matrix);
            mgr.overflow.get_handlers().notify(SingleModifyEvent{field:""}, mgr);
          }
        }
    }
}
//监听Node的创建， 设置脏标志
impl ComponentHandler<Node, CreateEvent, GuiComponentMgr> for OverflowSys {
  fn handle(&self, event: &CreateEvent, mgr: &mut GuiComponentMgr) {
    let CreateEvent{id, parent} = event;
    let by = {
      // 检查该节点的父容器是否有by_overflow
      let b = mgr.node._group.get_mut(*parent).by_overflow;
      // 检查该节点是否有overflow, 如果有,则其自身的by_overflow受overflow影响
      let node = mgr.node._group.get_mut(*id);
      if node.overflow { // 其裁剪矩形需要等Matrix被设置时设置
        let i = set_index(&mut mgr.overflow, 0, *id);
        node.by_overflow | i | b
      }else if b > 0 {
        node.by_overflow | b
      }else{
        return
      }
    };
    mgr.get_node_mut(*id).set_by_overflow(by);
    mgr.overflow.get_handlers().notify(SingleModifyEvent{field:""}, mgr);
  }
}
//监听Node的删除创建， 删除脏标志
impl ComponentHandler<Node, DeleteEvent, GuiComponentMgr> for OverflowSys {
  fn handle(&self, event: &DeleteEvent, mgr: &mut GuiComponentMgr) {
    let DeleteEvent{id, parent: _} = event;
    // 检查该节点是否有overflow
    if mgr.node._group.get_mut(*id).overflow {
      // 删除根上的overflow的裁剪矩形
      if set_index(&mut mgr.overflow, *id, 0) > 0 {
        mgr.overflow.get_handlers().notify(SingleModifyEvent{field:""}, mgr);
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
      return i;
    }
  }
  0
}
// 寻找指定当前值cur的偏移量, 设置成指定的值. 返回偏移量, 0表示没找到
#[inline]
fn set_index(overflow: &mut Overflow, cur: usize, value: usize) -> usize {
  let i = get_index(overflow, cur);
  if i > 0 {
    overflow.0[i] = value;
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
fn adjust(mgr: &mut GuiComponentMgr, mut child: usize, index: usize, ops: fn(a:usize, b:usize)->usize) {
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
fn calc_point(position: &Vector3, size: &RectSize, matrix: &Matrix4) -> [Point2;4] {
  let p = position.deref();
  let m = matrix.deref();
  let left_top = m * Vector4::new(p.x, p.y, 0.0, 1.0);
  let right_top = m * Vector4::new(p.x + size.width, p.y, 0.0, 1.0);
  let left_bottom = m * Vector4::new(p.x, p.y + size.height, 0.0, 1.0);
  let right_bottom = m * Vector4::new(p.x + size.width, p.y + size.height, 0.0, 1.0);
  let lt = Point2(cg::Point2{x: left_top.x, y: left_top.y});
  let rt = Point2(cg::Point2{x: right_top.x, y: right_top.y});
  let lb = Point2(cg::Point2{x: left_bottom.x, y: left_bottom.y});
  let rb = Point2(cg::Point2{x: right_bottom.x, y: right_bottom.y});
  [lt, rt, lb, rb]
}

#[cfg(test)]
#[cfg(not(feature = "web"))]
mod test {
    use wcs::world::{World};

    use component::node::{Node, InsertType};
    use world::GuiComponentMgr;
    use system::node_count::NodeCountSys;
    use system::overflow::OverflowSys;

    #[test]
    fn test(){
        let mut world: World<GuiComponentMgr, ()> = World::new(GuiComponentMgr::new());
        let _node_count = NodeCountSys::init(&mut world.component_mgr);
        let _overflow_sys = OverflowSys::init(&mut world.component_mgr);
        // let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![NodeCountSys::init(&mut world.component_mgr), OverflowSys::init(&mut world.component_mgr)];
        // world.set_systems(systems);
        test_world_overflow(&mut world);
    }

    fn test_world_overflow(world: &mut World<GuiComponentMgr, ()>){
        let (root, node1, node2, node3, node4, node5) = {
            let component_mgr = &mut world.component_mgr;
            {
                
                let (root, node1, node2, node3, node4, node5) = {
                    let root = Node::default();
                    let mut root_ref = component_mgr.add_node(root);
                    (
                        root_ref.id,
                        (root_ref.insert_child(Node::default(), InsertType::Back)).id,
                        (root_ref.insert_child(Node::default(), InsertType::Back)).id,
                        (root_ref.insert_child(Node::default(), InsertType::Back)).id,
                        (root_ref.insert_child(Node::default(), InsertType::Back)).id,
                        (root_ref.insert_child(Node::default(), InsertType::Back)).id,
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

        println!("modify run-----------------------------------------");
        world.run(());
        print_node(&world.component_mgr, root);
        print_node(&world.component_mgr, node1);
        print_node(&world.component_mgr, node2);
        print_node(&world.component_mgr, node3);
        print_node(&world.component_mgr, node4);
        print_node(&world.component_mgr, node5);
    }

    fn print_node(mgr: &GuiComponentMgr, id: usize) {
        let node = mgr.node._group.get(id);

        println!("nodeid: {}, ov:{:?}, byov: {}", id, node.overflow, node.by_overflow);
    }
}

