//裁剪矩形系统
// 容器设置了overflow的，就会产生一个裁剪矩形及对应的编号（编号都是2的次方），其下的所有的物件的by_overflow将会被设置为受到该id的影响
// 因为很少来回变动，所以直接根据变化进行设置，不采用dirty

use dirty::LayerDirty;
use ecs::{
    component::MultiCaseImpl,
    monitor::NotifyImpl,
    monitor::{CreateEvent, DeleteEvent, ModifyEvent},
    single::SingleCaseImpl,
    system::{EntityListener, MultiCaseListener, Runner},
    Event, SingleCaseListener,
};
use hal_core::*;
use share::Share;

use crate::{component::{calc::LayoutR, calc::*, user::Overflow, user::*}, single::oct::OctKey};
use crate::entity::Node;
use crate::single::IdTree;
use crate::single::{Clip, Oct, OverflowClip, ViewMatrix};

type Read<'a> = (
    &'a SingleCaseImpl<IdTree>,
    &'a MultiCaseImpl<Node, Overflow>,
    &'a MultiCaseImpl<Node, WorldMatrix>,
    &'a MultiCaseImpl<Node, Transform>,
    &'a MultiCaseImpl<Node, LayoutR>,
    &'a SingleCaseImpl<Oct>,
    &'a MultiCaseImpl<Node, TransformWillChangeMatrix>,
    &'a MultiCaseImpl<Node, NodeState>,
    &'a SingleCaseImpl<ViewMatrix>,
);
type Write<'a> = (
    &'a mut SingleCaseImpl<OverflowClip>,
    &'a mut MultiCaseImpl<Node, ByOverflow>,
    &'a mut MultiCaseImpl<Node, Culling>,
    &'a mut MultiCaseImpl<Node, StyleMark>,
);

#[derive(Default)]
pub struct OverflowImpl {
    overflow_dirty: LayerDirty<usize>,
}

impl OverflowImpl {
    fn modify<'a>(&mut self, id: usize, read: Read<'a>, mut write: Write<'a>) {
        let node = &read.0[id];
        if node.layer() == 0 {
            return;
        }
        let overflow = *read.1[id];
        let mut by = *write.1[id];
        let notify = unsafe { &*(write.0.get_notify_ref() as *const NotifyImpl) };
        let index = if overflow {
            let mut i = create_clip(id, write.0);
            if i == 0 {
                return;
            }
            self.mark_dirty(id, StyleType::Overflow as usize, node.layer(), &mut write.3);
            i = 1 << (i - 1);
            by = add_index(by, i);
            i
        } else {
            // 删除根上的overflow的裁剪矩形
            let mut i = remove_index(&mut *write.0, id, &notify);
            if i == 0 {
                return;
            }
            i = 1 << (i - 1);
            by = del_index(by, i);
            i
        };
        if by & index != 0 {
            adjust(&read.0, write.1, &read.7, node.children().head, index, add_index);
        } else {
            adjust(&read.0, write.1, &read.7, node.children().head, index, del_index);
        }
    }
}

impl<'a> Runner<'a> for OverflowImpl {
    type ReadData = Read<'a>;
    type WriteData = Write<'a>;
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        // let time = std::time::Instant::now();
        let (overflow_clip, by_overflows, cullings, style_marks) = write;
        for (id, layer) in self.overflow_dirty.iter() {
            let dirty_other = match style_marks.get(*id) {
                Some(r) => r.dirty_other,
                None => continue,
            };
            if read.0[*id].layer() != layer {
                continue;
            }
            if dirty_other & StyleType::Overflow as usize != 0 {
                let by = by_overflows[*id].0;
                let aabb = unsafe { &*(overflow_clip as *const SingleCaseImpl<OverflowClip>) }.clip_map.get(&by);
                let parent_will_change_matrix = get_will_change_matrix(*id, &read.0, read.6);
                calc_clip(
                    *id,
                    by,
                    parent_will_change_matrix,
                    aabb,
                    read,
                    overflow_clip,
                    by_overflows,
                    cullings,
                    style_marks,
                );
            }
        }
        self.overflow_dirty.clear();
    }
}

// impl<'a> EntityListener<'a, Node, CreateEvent> for OverflowImpl {
//     type ReadData = ();
//     type WriteData = (
//         &'a mut MultiCaseImpl<Node, Overflow>,
//         &'a mut MultiCaseImpl<Node, ByOverflow>,
//         &'a mut MultiCaseImpl<Node, Culling>,
//     );

//     fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
//         write.1.insert(event.id, ByOverflow::default());
//         write.0.insert(event.id, Overflow::default());
//         write.2.insert(event.id, Culling(false));
//     }
// }

impl<'a> MultiCaseListener<'a, Node, Overflow, DeleteEvent> for OverflowImpl {
    type ReadData = &'a MultiCaseImpl<Node, Overflow>;
    type WriteData = &'a mut SingleCaseImpl<OverflowClip>;

    fn listen(&mut self, event: &Event, overflows: Self::ReadData, overflow_clip: Self::WriteData) {
        let overflow = overflows[event.id].0;
        if overflow {
            let notify = unsafe { &*(overflow_clip.get_notify_ref() as *const NotifyImpl) };
            remove_index(&mut *overflow_clip, event.id, &notify);
        }
    }
}

//监听overflow属性的改变
impl<'a> MultiCaseListener<'a, Node, Overflow, (CreateEvent, ModifyEvent)> for OverflowImpl {
    type ReadData = Read<'a>;
    type WriteData = Write<'a>;

    fn listen(&mut self, event: &Event, read: Self::ReadData, mut write: Self::WriteData) { self.modify(event.id, read, write); }
}

impl<'a> SingleCaseListener<'a, Oct, (CreateEvent, ModifyEvent)> for OverflowImpl {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Overflow>,
        &'a SingleCaseImpl<IdTree>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &Event, read: Self::ReadData, write: Self::WriteData) { self.matrix_dirty(event.id, read, write); }
}

//监听WorldMatrix组件的修改
impl<'a> MultiCaseListener<'a, Node, TransformWillChangeMatrix, (CreateEvent, ModifyEvent, DeleteEvent)> for OverflowImpl {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Overflow>,
        &'a SingleCaseImpl<IdTree>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;

    fn listen(&mut self, event: &Event, read: Self::ReadData, write: Self::WriteData) { self.matrix_will_change_dirty(event.id, read, write); }
}

impl<'a> MultiCaseListener<'a, Node, Transform, (CreateEvent, ModifyEvent, DeleteEvent)> for OverflowImpl {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Overflow>,
        &'a SingleCaseImpl<IdTree>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &Event, read: Self::ReadData, write: Self::WriteData) { self.matrix_dirty(event.id, read, write); }
}

impl<'a> MultiCaseListener<'a, Node, WorldMatrix, (CreateEvent, ModifyEvent)> for OverflowImpl {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Overflow>,
        &'a SingleCaseImpl<IdTree>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &Event, read: Self::ReadData, write: Self::WriteData) { self.matrix_dirty(event.id, read, write); }
}

impl<'a> MultiCaseListener<'a, Node, LayoutR, ModifyEvent> for OverflowImpl {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Overflow>,
        &'a SingleCaseImpl<IdTree>,
        &'a MultiCaseImpl<Node, NodeState>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &Event, read: Self::ReadData, write: Self::WriteData) {
        // // 虚拟节点不需要计算overflowis_rnode
        // if let None = read.1.get(event.id) {
        // 	return;
        // }
        let isrnode = read.3[event.id].0.is_rnode();
        if isrnode {
            self.matrix_dirty(event.id, (read.0, read.1, read.2), write);
        }
    }
}

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for OverflowImpl {
    type ReadData = Read<'a>;
    type WriteData = Write<'a>;

    fn listen(&mut self, event: &Event, read: Self::ReadData, mut write: Self::WriteData) {
        let node = &read.0[event.id];
        // 获得父节点的ByOverflow
        let mut by = *write.1[node.parent()];
        let overflow = *read.1[node.parent()];
        if overflow {
            let i = get_index(write.0, node.parent());
            if i > 0 {
                by = add_index(by, 1 << (i - 1));
            }
        }
        self.set_overflow(event.id, by, &read, &mut write);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for OverflowImpl {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Overflow>);
    type WriteData = (&'a mut SingleCaseImpl<OverflowClip>, &'a mut MultiCaseImpl<Node, ByOverflow>);

    fn listen(&mut self, event: &Event, read: Self::ReadData, write: Self::WriteData) {
        let node = &read.0[event.id];
        let notify = unsafe { &*(write.0.get_notify_ref() as *const NotifyImpl) };
        if match read.1.get(event.id) {
            Some(r) => **r,
            _ => false,
        } {
            remove_index(&mut *write.0, event.id, &notify);
        }
        unsafe { write.1.get_unchecked_write(event.id) }.set_0(0);
        // 递归调用，检查是否有overflow， 撤销设置OverflowClip
        for (id, _n) in read.0.recursive_iter(node.children().head) {
            if *read.1[id] {
                remove_index(&mut *write.0, id, &notify);
            }
            match write.1.get_write(id) {
                Some(mut r) => r.set_0(0),
                None => continue,
            };
        }
    }
}

impl<'a> EntityListener<'a, Node, ModifyEvent> for OverflowImpl {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, Overflow>);
    type WriteData = (&'a mut SingleCaseImpl<OverflowClip>, &'a mut MultiCaseImpl<Node, ByOverflow>);

    fn listen(&mut self, event: &Event, read: Self::ReadData, write: Self::WriteData) {
        let overflow = *read.1[event.id];
        if overflow {
            let notify = unsafe { &*(write.0.get_notify_ref() as *const NotifyImpl) };
            remove_index(&mut *write.0, event.id, &notify);
        }
    }
}

impl OverflowImpl {
    #[inline]
    fn mark_dirty(&mut self, id: usize, dirty_type: usize, layer: usize, style_marks: &mut MultiCaseImpl<Node, StyleMark>) {
        let style_mark = match style_marks.get_mut(id) {
            Some(r) => r,
            None => return,
        };
        if style_mark.dirty_other & dirty_type == 0 {
            style_mark.dirty_other |= dirty_type;
            self.overflow_dirty.mark(id, layer);
        }
    }

    fn matrix_dirty(
        &mut self,
        id: usize,
        read: (&MultiCaseImpl<Node, ByOverflow>, &MultiCaseImpl<Node, Overflow>, &SingleCaseImpl<IdTree>),
        write: &mut MultiCaseImpl<Node, StyleMark>,
    ) {
        let node = match read.2.get(id) {
            Some(r) => r,
            None => return,
        };
        if node.layer() == 0
            || (read.0[id].0 == 0
                && !match read.1.get(id) {
                    Some(r) => **r,
                    _ => false,
                })
        {
            return;
        }
        self.mark_dirty(id, StyleType::Overflow as usize, node.layer(), write);
    }

    fn matrix_will_change_dirty(
        &mut self,
        id: usize,
        read: (&MultiCaseImpl<Node, ByOverflow>, &MultiCaseImpl<Node, Overflow>, &SingleCaseImpl<IdTree>),
        write: &mut MultiCaseImpl<Node, StyleMark>,
    ) {
        let node = match read.2.get(id) {
            Some(r) => r,
            None => return,
        };
        if node.layer() == 0 {
            return;
        }
        self.mark_dirty(id, StyleType::Overflow as usize, node.layer(), write);
    }

    // 递归调用，检查是否有overflow， 设置OverflowClip， 设置所有子元素的by_overflow
    fn set_overflow(&mut self, id: usize, mut by: usize, read: &Read, write: &mut Write) {
        if !read.7[id].0.is_rnode() {
            return;
        }
        let overflow = *read.1[id];
        if by > 0 {
            unsafe { write.1.get_unchecked_write(id) }.set_0(by);
        }
        if overflow {
            // 添加根上的overflow的裁剪矩形
            // let i = set_index(&mut *write.0, 0, id);
            let i = create_clip(id, write.0);
            self.mark_dirty(id, StyleType::Overflow as usize, read.0[id].layer(), &mut write.3);
            if i > 0 {
                by = add_index(by, 1 << (i - 1));
            }
        }

        let node = &read.0[id];
        for (id, _n) in read.0.iter(node.children().head) {
            self.set_overflow(id, by, read, write);
        }
    }
}

fn get_will_change_matrix<'a, 'b>(
    mut id: usize,
    idtree: &'a IdTree,
    transform_will_change_matrixs: &'b MultiCaseImpl<Node, TransformWillChangeMatrix>,
) -> Option<&'b TransformWillChangeMatrix> {
    loop {
        if let Some(r) = transform_will_change_matrixs.get(id) {
            return Some(r);
        }
        let node = &idtree[id];
        if node.parent() == 0 {
            return None;
        }
        id = node.parent();
    }
}


fn calc_clip<'a>(
    id: usize,
    mut by: usize,
    mut transform_will_change_matrix: Option<&'a TransformWillChangeMatrix>,
    mut by_clip_aabb: Option<&'a (Aabb2, Share<dyn UniformBuffer>)>,
    read: Read<'a>,
    overflow_clip: &'a mut SingleCaseImpl<OverflowClip>,
    by_overflows: &'a mut MultiCaseImpl<Node, ByOverflow>,
    cullings: &'a mut MultiCaseImpl<Node, Culling>,
    style_marks: &'a mut MultiCaseImpl<Node, StyleMark>,
) {
    if by > 0 {
        // 裁剪剔除
        if let Some(item) = by_clip_aabb {
            match transform_will_change_matrix {
                Some(m) => {
                    // // 如果没有旋转
                    // if !(m.0).1 {
                    //     unsafe{cullings.get_unchecked_write(id)}.set_0(!is_intersect(
                    //         &item.0,
                    //         &unsafe { read.6.get_unchecked(id) }.0,
                    //     ))
                    // }
                    if !(m.0).1 {
						// if !is_intersect(&item.0, &matrix_mul_aabb(&m.0, &unsafe { read.5.get_unchecked(id) }.0)) {
						// 	log::warn!("cull will change=======id:{:?}, by:{}, clip:{:?}, aabb:{:?}", id, by, &item.0, &matrix_mul_aabb(&m.0, &unsafe { read.5.get_unchecked(id) }.0));
						// }
						if read.5.get(id).is_none() {
							log::error!("!!!!======read.5 is none, {:?}", id);
							
						}
                        unsafe {
                            cullings
                                .get_unchecked_write(id)
                                .set_0(!is_intersect(&item.0, &matrix_mul_aabb(&m.0, &unsafe { read.5.get_unchecked(OctKey(id)) }.0)))
                        };
                    }
                }
                None => {
					if read.5.get(id).is_none() {
						log::error!("!!!!======read.5 1 is none, {:?}", id);
						
					}
					// if !is_intersect(&item.0, &unsafe { read.5.get_unchecked(id) }.0) {
					// 	log::warn!("cull=======id:{:?}, by:{}, clip:{:?}, aabb:{:?}", id, by, &item.0, &unsafe { read.5.get_unchecked(id) }.0);
					// }
					unsafe { cullings.get_unchecked_write(id) }.set_0(!is_intersect(&item.0, &unsafe { read.5.get_unchecked(OctKey(id)) }.0))
				},
            }
        }
        // 通知by_overflow改变，以修改clipBox
        by_overflows.get_notify_ref().modify_event(id, "", 0);
    }

    if let Some(r) = read.6.get(id) {
        transform_will_change_matrix = Some(r);
    }

    let overflow = *read.1[id];
    if overflow {
        let i = get_index(overflow_clip, id);
        if i > 0 {
            // 计算裁剪平面
            set_clip(id, i, &read, overflow_clip, transform_will_change_matrix);
            let by1 = add_index(by, 1 << (i - 1));
            by_clip_aabb = modify_intersect_clip(
                by,
                by1,
                i,
                unsafe { &mut *(overflow_clip as *mut SingleCaseImpl<OverflowClip>) },
                &(read.8).0,
            );
            by = by1;
            // by_clip_aabb
            overflow_clip.get_notify_ref().modify_event(i, "", id);
        }
    }

    style_marks[id].dirty_other &= !(StyleType::Overflow as usize);

	if read.0.get(id).is_none() {
		log::error!("!!!!======read.0 1 is none, {:?}", id);
		
	}
    let first = read.0[id].children().head;
    for (child_id, _child) in read.0.iter(first) {
		if read.7.get(child_id).is_none() {
			log::error!("!!!!======read.0 1 is none, {:?}", child_id);
			
		}
        if !read.7[child_id].0.is_rnode() {
            continue;
        }
        calc_clip(
            child_id,
            by,
            transform_will_change_matrix,
            by_clip_aabb,
            read,
            overflow_clip,
            by_overflows,
            cullings,
            style_marks,
        );
    }
}

//================================ 内部静态方法
// 设置裁剪区域，需要 world_matrix LayoutR
fn set_clip(id: usize, i: usize, read: &Read, clip: &mut SingleCaseImpl<OverflowClip>, transform_will_change: Option<&TransformWillChangeMatrix>) {
	if read.2.get(id).is_none() {
		log::error!("!!!!======read.2 is none, {:?}", id);
		
	}

    let mut world_matrix = &read.2[id];
    let temp;
    if let Some(r) = transform_will_change {
        temp = &r.0 * world_matrix;
        world_matrix = &temp;
    }
	if read.4.get(id).is_none() {
		log::error!("!!!!======read.4 is none, {:?}", id);
		
	}
    let layout = &read.4[id];
    // BUG , overflow与tranwillchange在同一节点上， origin应该从tranwillchange上取
    let origin = match read.3.get(id) {
        Some(r) => r.origin.clone(),
        None => TransformOrigin::Center,
    };
    let origin = origin.to_value(layout.rect.right - layout.rect.left, layout.rect.bottom - layout.rect.top);
	let c = &mut clip.clip[i];
    *c = Clip {
        view: calc_point(layout, world_matrix, &origin),
        has_rotate: world_matrix.1,
        old_has_rotate: c.has_rotate,
        node_id: id,
    };
}

//================================ 内部静态方法
// 创建裁剪区域，需要 world_matrix LayoutR
fn create_clip(id: usize, clip: &mut SingleCaseImpl<OverflowClip>) -> usize {
    if clip.clip.len() >= 32 {
        log::warn!("clip overflow!!!!!!!");
        return 0;
    }
    if let Some(r) = clip.id_map.get(&id) {
        return *r;
    }

    let i = clip.clip.insert(Clip {
        view: [Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)],
        has_rotate: false,
        old_has_rotate: false,
        node_id: id,
    });
    clip.id_map.insert(id, i);
    i
}

// 寻找指定当前值cur的偏移量
#[inline]
fn get_index(overflow: &mut OverflowClip, cur: usize) -> usize {
    match overflow.id_map.get(&cur) {
        Some(r) => *r,
        None => 0,
    }
}

#[inline]
fn remove_index(overflow: &mut OverflowClip, node_id: usize, notify: &NotifyImpl) -> usize {
    if let Some(r) = overflow.id_map.remove(&node_id) {
        notify.modify_event(r, "", node_id);
        overflow.clip.remove(r);
        r
    } else {
        0
    }
}

#[inline]
fn add_index(by: usize, index: usize) -> usize { by | index }
#[inline]
fn del_index(by: usize, index: usize) -> usize { by & !index }
// 整理方法。设置或取消所有子节点的by_overflow上的index。
#[inline]
fn adjust(
    idtree: &SingleCaseImpl<IdTree>,
    by_overflow: &mut MultiCaseImpl<Node, ByOverflow>,
    node_states: &MultiCaseImpl<Node, NodeState>,
    child: usize,
    index: usize,
    ops: fn(a: usize, b: usize) -> usize,
) {
    for (id, _n) in idtree.recursive_iter(child) {
        if !node_states[id].is_rnode() {
            continue;
        }
        let by = *by_overflow[id];
        unsafe { by_overflow.get_unchecked_write(id).set_0(ops(by, index)) };
    }
}
// 计算内容区域矩形的4个点
fn calc_point(layout: &LayoutR, m: &Matrix4, origin: &Point2) -> [Point2; 4] {
    let width = layout.rect.right - layout.rect.left - layout.padding.left - layout.padding.right - layout.border.left - layout.border.right;
    let height = layout.rect.bottom - layout.rect.top - layout.padding.top - layout.padding.bottom - layout.border.top - layout.border.bottom;
    let start = (
        layout.border.left + layout.padding.left - origin.x,
        layout.border.top + layout.padding.top - origin.y,
    );
    let left_top = m * Vector4::new(start.0, start.1, 0.0, 1.0);
    let right_top = m * Vector4::new(start.0 + width, start.1, 0.0, 1.0);
    let left_bottom = m * Vector4::new(start.0, start.1 + height, 0.0, 1.0);
    let right_bottom = m * Vector4::new(start.0 + width, start.1 + height, 0.0, 1.0);

    let lt = Point2::new(left_top.x, left_top.y);
    let rt = Point2::new(right_top.x, right_top.y);
    let lb = Point2::new(left_bottom.x, left_bottom.y);
    let rb = Point2::new(right_bottom.x, right_bottom.y);
    [lt, lb, rb, rt]
}

// 计算aabb
fn matrix_mul_aabb(m: &WorldMatrix, aabb: &Aabb2) -> Aabb2 {
    let min = m * Vector4::new(aabb.mins.x, aabb.mins.y, 0.0, 1.0);
    let max = m * Vector4::new(aabb.maxs.x, aabb.maxs.y, 0.0, 1.0);
    Aabb2::new(Point2::new(min.x, min.y), Point2::new(max.x, max.y))
}

// 将clip求交结果存储在clip_map中
fn add_intersect_clip(parent_by: usize, by: usize, i: usize, overflow: &mut OverflowClip, view_matrix: &WorldMatrix) {
    let r = &overflow.clip[i];
    if r.has_rotate {
        return;
    }
    if let Some(p) = overflow.clip_map.get(&parent_by) {
        let aabb = intersect(
            &Aabb2::new(Point2::new(r.view[0].x, r.view[0].y), Point2::new(r.view[2].x, r.view[2].y)),
            &p.0,
        );
        overflow.insert_aabb(by, aabb, view_matrix);
        // overflow.clip_map.insert(by, intersect(&Aabb2::new(Point2::new(r.view[0].x, r.view[0].y, 0.0),  Point2::new(r.view[2].x, r.view[2].y, 0.0)), p));
    }
}

// 将clip求交结果存储在clip_map中
fn modify_intersect_clip<'a, 'b>(
    parent_by: usize,
    by: usize,
    i: usize,
    overflow: &'a mut OverflowClip,
    view_matrix: &'b WorldMatrix,
) -> Option<&'a (Aabb2, Share<dyn UniformBuffer>)> {
    let r = &overflow.clip[i];
    if r.has_rotate {
        overflow.clip_map.remove(&by);
        return None;
    }
    if let Some(p) = overflow.clip_map.get(&parent_by) {
        let aabb = intersect(
            &Aabb2::new(Point2::new(r.view[0].x, r.view[0].y), Point2::new(r.view[2].x, r.view[2].y)),
            &p.0,
        );
        Some(overflow.insert_aabb(by, aabb, view_matrix))
    } else {
        None
    }
}

#[inline]
fn remove_intersect_clip(by: usize, overflow: &mut OverflowClip) { overflow.clip_map.remove(&by); }

#[inline]
fn is_intersect(a: &Aabb2, b: &Aabb2) -> bool {
    if a.mins.x > b.maxs.x || a.mins.y > b.maxs.y || b.mins.x > a.maxs.x || b.mins.y > a.maxs.y {
        return false;
    } else {
        true
    }
}

// aabb相交
fn intersect(a: &Aabb2, b: &Aabb2) -> Aabb2 {
    if !is_intersect(a, b) {
        return Aabb2::new(Point2::new(0.0, 0.0), Point2::new(0.0, 0.0));
    }
    let mut aabb = Aabb2::new(Point2::new(0.0, 0.0), Point2::new(0.0, 0.0));
    if a.mins.x >= b.mins.x {
        aabb.mins.x = a.mins.x;
    } else {
        aabb.mins.x = b.mins.x;
    }

    if a.maxs.x >= b.maxs.x {
        aabb.maxs.x = b.maxs.x;
    } else {
        aabb.maxs.x = a.maxs.x;
    }

    if a.mins.y >= b.mins.y {
        aabb.mins.y = a.mins.y;
    } else {
        aabb.mins.y = b.mins.y;
    }

    if a.maxs.y >= b.maxs.y {
        aabb.maxs.y = b.maxs.y;
    } else {
        aabb.maxs.y = a.maxs.y;
    }
    return aabb;
}

impl_system! {
    OverflowImpl,
    true,
    {
        // EntityListener<Node, CreateEvent>
        EntityListener<Node, ModifyEvent>
        MultiCaseListener<Node, Overflow, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, Overflow, DeleteEvent>
        SingleCaseListener<Oct, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, WorldMatrix, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, Transform, (CreateEvent, ModifyEvent, DeleteEvent)>
        MultiCaseListener<Node, LayoutR, ModifyEvent>
        SingleCaseListener<IdTree, CreateEvent>
        SingleCaseListener<IdTree, DeleteEvent>
        MultiCaseListener<Node, TransformWillChangeMatrix, (CreateEvent, ModifyEvent, DeleteEvent)>
    }
}
