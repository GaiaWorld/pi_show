//裁剪矩形系统
// 容器设置了overflow的，就会产生一个裁剪矩形及对应的编号（编号都是2的次方），其下的所有的物件的by_overflow将会被设置为受到该id的影响
// 因为很少来回变动，所以直接根据变化进行设置，不采用dirty

use ecs::{
  system::{MultiCaseListener, SingleCaseListener, EntityListener},
  monitor::{CreateEvent, DeleteEvent, ModifyEvent},
  component::MultiCaseImpl,
  single::SingleCaseImpl,
  idtree::IdTree,
  monitor::NotifyImpl,
};

use entity::{Node};
use component::{
  user::*,
  calc::*,
};
use single::{ OverflowClip, DefaultTable, Clip, Oct };
use system::util::get_or_default;



pub struct OverflowImpl;

impl<'a> EntityListener<'a, Node, CreateEvent> for OverflowImpl {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, Overflow>, &'a mut MultiCaseImpl<Node, ByOverflow>);

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        write.1.insert(event.id, ByOverflow::default());
        write.0.insert(event.id, Overflow::default());
    }
}

type Read<'a> = (
    &'a SingleCaseImpl<IdTree>,
    &'a MultiCaseImpl<Node, Overflow>,
    &'a MultiCaseImpl<Node, WorldMatrix>,
    &'a MultiCaseImpl<Node, Transform>,
    &'a MultiCaseImpl<Node, Layout>,
    &'a SingleCaseImpl<DefaultTable>,
    &'a SingleCaseImpl<Oct>,
);
type Write<'a> = (&'a mut SingleCaseImpl<OverflowClip>, &'a mut MultiCaseImpl<Node, ByOverflow>, &'a mut MultiCaseImpl<Node, Culling>);

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
        let notify = write.0.get_notify();
        let index = if overflow {
            // 添加根上的overflow的裁剪矩形
            // let mut i = set_index(&mut *write.0, 0, event.id);
            // if i == 0 {
            //     return;
            // }
            let mut i = create_clip(event.id, &read, write.0);
            if i == 0 {
                return;
            }
            i = 1<<(i-1);
            let by1 = add_index(by, i);
            add_intersect_clip(by, by1, i, write.0);
            by = by1;
            write.0.get_notify().modify_event(i, "", event.id);
            i
        }else{
            // 删除根上的overflow的裁剪矩形
            let mut i = remove_index(&mut *write.0, event.id, &notify);
            if i == 0 {
                return;
            }
            i = 1<<(i-1);
            by = del_index(by, i);
            i
        };
        if by & index != 0 {
            adjust(&read.0, write.1, read.1, write.0, node.children.head, index, add_index);
        }else{
            adjust(&read.0, write.1, read.1, write.0, node.children.head, index, del_index);
        }
    }
}
//监听WorldMatrix组件的修改
impl<'a> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for OverflowImpl {
    type ReadData = Read<'a>;
    type WriteData = Write<'a>;

    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
        let node = unsafe{ read.0.get_unchecked(event.id)};
        if node.layer == 0 {
            return;
        }

        let overflow = match read.1.get(event.id){Some(r) => **r, _ => false};
        if overflow {
            let i = get_index(write.0, event.id);
            if i > 0 {
                set_clip(event.id, i, &read, write.0);
                let by = **unsafe{ write.1.get_unchecked(event.id)};
                modify_intersect_clip(by, add_index(by, 1<<(i - 1)), i, write.0);
                write.0.get_notify().modify_event(i, "", event.id);
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
        let mut by = **unsafe{ write.1.get_unchecked(node.parent)};
        let overflow = match read.1.get(node.parent){Some(r) => **r, _ => false};
        if overflow {
            let i = get_index(write.0, node.parent);
            if i > 0 {
                by = add_index(by, 1<<(i - 1));
            }
        }
        let mut modify = false;
        set_overflow(event.id, by, &read, &mut write, &mut modify);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for OverflowImpl {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Overflow>);
    type WriteData = &'a mut SingleCaseImpl<OverflowClip>;

    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData) {
        let node = unsafe{ read.0.get_unchecked(event.id)};
        let overflow = match read.1.get(event.id){Some(r) => **r, _ => false};
        let notify = write.get_notify();
        if overflow {
            remove_index(&mut *write, event.id, &notify);
        }
        // 递归调用，检查是否有overflow， 撤销设置OverflowClip
        for (id, _) in read.0.recursive_iter(node.children.head) {
            let overflow = match read.1.get(id){Some(r) => **r, _ => false};
            if overflow{
                remove_index(&mut *write, id, &notify);
            }
        }
    }
}

//================================ 内部静态方法
// 设置裁剪区域，需要 world_matrix layout
fn set_clip(id: usize, i: usize, read: &Read, clip: &mut SingleCaseImpl<OverflowClip>) {
    let world_matrix = unsafe{ read.2.get_unchecked(id)};
    let layout = unsafe{ read.4.get_unchecked(id)};
    let origin = get_or_default(id, read.3, read.5).origin.to_value(layout.width, layout.height);
    let c = unsafe { clip.clip.get_unchecked_mut(i) };
    *c = Clip{
        view: calc_point(layout, world_matrix, &origin),
        has_rotate: world_matrix.1,
        old_has_rotate: c.has_rotate,
        node_id: id,
    };
}

//================================ 内部静态方法
// 创建裁剪区域，需要 world_matrix layout
fn create_clip(id: usize, read: &Read, clip: &mut SingleCaseImpl<OverflowClip>) -> usize {
    if clip.clip.len() >= 32 {
        println!("clip overflow!!!!!!!");
        return 0;
    }
    let world_matrix = unsafe{ read.2.get_unchecked(id)};
    let layout = unsafe{ read.4.get_unchecked(id)};
    let origin = get_or_default(id, read.3, read.5).origin.to_value(layout.width, layout.height);
    let i = clip.clip.insert(Clip{
        view: calc_point(layout, world_matrix, &origin),
        has_rotate: world_matrix.1,
        old_has_rotate: world_matrix.1,
        node_id: id,
    });
    clip.id_map.insert(id, i);
    i
}

// 递归调用，检查是否有overflow， 设置OverflowClip， 设置所有子元素的by_overflow
fn set_overflow(id: usize, mut by: usize, read: &Read, write: &mut Write, modify: &mut bool) {
    if by > 0 {
        unsafe {write.1.get_unchecked_write(id)}.set_0(by);
    }
    let overflow = match read.1.get(id){Some(r) => **r, _ => false};
    if overflow {
        // 添加根上的overflow的裁剪矩形
        // let i = set_index(&mut *write.0, 0, id);
        let i = create_clip(id, read, write.0);
        if i > 0 {
            let by1 = add_index(by, 1<<(i - 1));
            add_intersect_clip(by, by1, i, write.0);
            by = by1;
            write.0.get_notify().modify_event(i, "", id);
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
fn get_index(overflow: &mut OverflowClip, cur: usize) -> usize {
    match overflow.id_map.get(&cur) {
        Some(r) => *r,
        None => 0,
    }
    
//   for i in 0..overflow.id_vec.len() {
//     if cur == overflow.id_vec[i] {
//       return i + 1;
//     }
//   }
//   0
}

#[inline]
fn remove_index(overflow: &mut OverflowClip, node_id: usize, notify: &NotifyImpl) -> usize{
    if let Some(r) = overflow.id_map.remove(&node_id) {
        notify.modify_event(r, "", node_id);
        overflow.clip.remove(r);
        r
    } else {
        0
    }
}
// // 寻找指定当前值cur的偏移量, 设置成指定的值. 返回偏移量, 0表示没找到
// #[inline]
// fn set_index(overflow: &mut OverflowClip, node_id: usize) -> usize {
//     // let i = get_index(overflow, cur);
//     overflow.id_map.insert(node_id, v: V)
//     if i > 0 {
//         overflow.id_vec[i-1] = value;
//     } else {
//         #[cfg(feature = "warning")]
//         println!("!!!!!!!!!!!!!!!!!!!Overflow reaches the upper limit");
//     }
//     i
// }

// #[inline]
// fn set_index(overflow: &mut OverflowClip, cur: usize, value: usize) -> usize {
//     let i = get_index(overflow, cur);
//     if i > 0 {
//         overflow.id_vec[i-1] = value;
//     } else {
//         #[cfg(feature = "warning")]
//         println!("!!!!!!!!!!!!!!!!!!!Overflow reaches the upper limit");
//     }
//     i
// }

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
fn adjust(idtree: &SingleCaseImpl<IdTree>, by_overflow: &mut MultiCaseImpl<Node, ByOverflow>, overflow: &MultiCaseImpl<Node, Overflow>, overflow_clip: &mut OverflowClip,  child: usize, index: usize, ops: fn(a:usize, b:usize)->usize) {
    for (id, _n) in idtree.recursive_iter(child) {
        let by = **unsafe {by_overflow.get_unchecked(id)};
        unsafe {by_overflow.get_unchecked_write(id)}.set_0(ops(by, index));

        let overflow = **unsafe {overflow.get_unchecked(id)};
        if overflow {
            let i = overflow_clip.id_map.get(&id).unwrap();
            modify_intersect_clip(by, add_index(by, 1<<(*i - 1)), *i, overflow_clip);
        }
    }
}
// 计算内容区域矩形的4个点
fn calc_point(layout: &Layout, m: &Matrix4, origin: &Point2) -> [Point2;4]{
    let width = layout.width - layout.padding_left - layout.padding_right - layout.border_left - layout.border_right;
    let height = layout.height - layout.padding_top - layout.padding_bottom - layout.border_top - layout.border_bottom;
    let start = (layout.border_left + layout.padding_left - origin.x, layout.border_top + layout.padding_top - origin.y);
    let left_top = m * Vector4::new(start.0,             start.1,          0.0, 1.0);
    let right_top = m * Vector4::new(start.0 + width,    start.1,          0.0, 1.0);
    let left_bottom = m * Vector4::new(start.0,          start.1 + height, 0.0, 1.0);
    let right_bottom = m * Vector4::new(start.0 + width, start.1 + height, 0.0, 1.0);

    let lt = Point2{x: left_top.x, y: left_top.y};
    let rt = Point2{x: right_top.x, y: right_top.y};
    let lb = Point2{x: left_bottom.x, y: left_bottom.y};
    let rb = Point2{x: right_bottom.x, y: right_bottom.y};
    [lt, lb, rb, rt]
}

// 将clip求交结果存储在clip_map中
fn add_intersect_clip(parent_by: usize, by: usize, i: usize, overflow: &mut OverflowClip){
    let r = unsafe { overflow.clip.get_unchecked(i) };
    if r.has_rotate {
        return;
    }
    if let Some(p) = overflow.clip_map.get(&parent_by) {
        let aabb = intersect(&Aabb3::new(Point3::new(r.view[0].x, r.view[0].y, 0.0),  Point3::new(r.view[2].x, r.view[2].y, 0.0)), &p.0);
        overflow.insert_aabb(by, aabb);
        // overflow.clip_map.insert(by, intersect(&Aabb3::new(Point3::new(r.view[0].x, r.view[0].y, 0.0),  Point3::new(r.view[2].x, r.view[2].y, 0.0)), p));
    }
}

// 将clip求交结果存储在clip_map中
fn modify_intersect_clip(parent_by: usize, by: usize, i: usize, overflow: &mut OverflowClip){
    let r = unsafe { overflow.clip.get_unchecked(i) };
    if r.has_rotate {
        overflow.clip_map.remove(&by);
        return;
    }
    if let Some(p) = overflow.clip_map.get(&parent_by) {
        let aabb = intersect(&Aabb3::new(Point3::new(r.view[0].x, r.view[0].y, 0.0),  Point3::new(r.view[2].x, r.view[2].y, 0.0)), &p.0);
        overflow.insert_aabb(by, aabb);
    }
}

#[inline]
fn remove_intersect_clip(by: usize, overflow: &mut OverflowClip) {
    overflow.clip_map.remove(&by);
}

#[inline]
fn is_intersect(a: &Aabb3, b: &Aabb3) -> bool {
    if a.min.x >= b.max.x || a.min.y >= b.max.y || b.min.x >= a.max.x || b.min.y >= a.max.y{
        return false;
    } else {
        true
    }
}

// aabb相交
fn intersect(a: &Aabb3, b: &Aabb3) -> Aabb3 {
    if !is_intersect(a, b) {
        return Aabb3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 0.0));
    }
    if a.min.x >= b.min.x {
        if a.max.y < b.max.y { //a被b包含
            return Aabb3::new(a.min, a.max);
        } else if a.min.y >= b.min.y {//a在b的右下
            return Aabb3::new(a.min, b.max);
        } else { // 右上
            return Aabb3::new(Point3::new(a.min.x, b.min.y, 0.0), Point3::new(b.max.x, a.max.y, 0.0));
        }
    } else {
        if b.max.y < a.max.y { //b被a包含
            return Aabb3::new(b.min, b.max);
        }else if b.min.y >= a.min.y { //b在a的右下
            return Aabb3::new(b.min, a.max);
        } else { // 右上
            return Aabb3::new(Point3::new(b.min.x, a.min.y, 0.0), Point3::new(a.max.x, b.max.y, 0.0));
        }
    }
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
//     let systems: Vec<Share<System<(), WorldDocMgr>>> = vec![];
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

//     world.run(());
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

// }