//裁剪矩形系统
// 容器设置了overflow的，就会产生一个裁剪矩形及对应的编号（编号都是2的次方），其下的所有的物件的by_overflow将会被设置为受到该id的影响
// 因为很少来回变动，所以直接根据变化进行设置，不采用dirty

use ecs::{
    component::MultiCaseImpl,
    idtree::IdTree,
    monitor::NotifyImpl,
    monitor::{CreateEvent, DeleteEvent, ModifyEvent},
    single::SingleCaseImpl,
    system::{EntityListener, MultiCaseListener, Runner},
    SingleCaseListener,
};
use hal_core::*;
use share::Share;

use component::{calc::*, user::*};
use dirty::LayerDirty;
use entity::Node;
use single::{Clip, DefaultTable, Oct, OverflowClip, ViewMatrix};

type Read<'a> = (
    &'a SingleCaseImpl<IdTree>,
    &'a MultiCaseImpl<Node, Overflow>,
    &'a MultiCaseImpl<Node, WorldMatrix>,
    &'a MultiCaseImpl<Node, Transform>,
    &'a MultiCaseImpl<Node, Layout>,
    &'a SingleCaseImpl<DefaultTable>,
    &'a SingleCaseImpl<Oct>,
    &'a MultiCaseImpl<Node, TransformWillChangeMatrix>,
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
    overflow_dirty: LayerDirty,
}

impl<'a> Runner<'a> for OverflowImpl {
    type ReadData = Read<'a>;
    type WriteData = Write<'a>;
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        // let time = std::time::Instant::now();
        let (overflow_clip, by_overflows, cullings, style_marks) = write;
        for (id, layer) in self.overflow_dirty.iter() {
            let dirty1 = match style_marks.get(*id) {
                Some(r) => r.dirty1,
                None => continue,
            };
            if read.0[*id].layer != layer {
                continue;
            }
            if dirty1 & StyleType1::Overflow as usize != 0 {
                let by = by_overflows[*id].0;
                let aabb = unsafe { &*(overflow_clip as *const SingleCaseImpl<OverflowClip>) }
                    .clip_map
                    .get(&by);
                let parent_will_change_matrix = get_will_change_matrix(*id, &read.0, read.7);
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

impl<'a> EntityListener<'a, Node, CreateEvent> for OverflowImpl {
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, Overflow>,
        &'a mut MultiCaseImpl<Node, ByOverflow>,
        &'a mut MultiCaseImpl<Node, Culling>,
    );

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        write.1.insert(event.id, ByOverflow::default());
        write.0.insert(event.id, Overflow::default());
        write.2.insert(event.id, Culling(false));
    }
}

impl<'a> MultiCaseListener<'a, Node, Overflow, DeleteEvent> for OverflowImpl {
    type ReadData = &'a MultiCaseImpl<Node, Overflow>;
    type WriteData = &'a mut SingleCaseImpl<OverflowClip>;

    fn listen(
        &mut self,
        event: &DeleteEvent,
        overflows: Self::ReadData,
        overflow_clip: Self::WriteData,
    ) {
        let overflow = overflows[event.id].0;
        if overflow {
            let notify = overflow_clip.get_notify();
            remove_index(&mut *overflow_clip, event.id, &notify);
        }
    }
}

//监听overflow属性的改变
impl<'a> MultiCaseListener<'a, Node, Overflow, ModifyEvent> for OverflowImpl {
    type ReadData = Read<'a>;
    type WriteData = Write<'a>;

    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, mut write: Self::WriteData) {
        let node = &read.0[event.id];
        if node.layer == 0 {
            return;
        }
        let overflow = match read.1.get(event.id) {
            Some(r) => **r,
            _ => false,
        };
        let mut by = *write.1[event.id] ;
        let notify = write.0.get_notify();
        let index = if overflow {
            let mut i = create_clip(event.id, write.0);
            if i == 0 {
                return;
            }
            self.mark_dirty(
                event.id,
                StyleType1::Overflow as usize,
                node.layer,
                &mut write.3,
            );
            i = 1 << (i - 1);
            by = add_index(by, i);
            i
        } else {
            // 删除根上的overflow的裁剪矩形
            let mut i = remove_index(&mut *write.0, event.id, &notify);
            if i == 0 {
                return;
            }
            i = 1 << (i - 1);
            by = del_index(by, i);
            i
        };
        if by & index != 0 {
            adjust(&read.0, write.1, node.children.head, index, add_index);
        } else {
            adjust(&read.0, write.1, node.children.head, index, del_index);
        }
    }
}

//监听WorldMatrix组件的修改
impl<'a> MultiCaseListener<'a, Node, TransformWillChangeMatrix, ModifyEvent> for OverflowImpl {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Overflow>,
        &'a SingleCaseImpl<IdTree>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;

    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
        self.matrix_will_change_dirty(event.id, read, write);
    }
}

//监听WorldMatrix组件的修改
impl<'a> MultiCaseListener<'a, Node, TransformWillChangeMatrix, CreateEvent> for OverflowImpl {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Overflow>,
        &'a SingleCaseImpl<IdTree>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;

    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData) {
        self.matrix_will_change_dirty(event.id, read, write);
    }
}

//监听WorldMatrix组件的修改
impl<'a> MultiCaseListener<'a, Node, TransformWillChangeMatrix, DeleteEvent> for OverflowImpl {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Overflow>,
        &'a SingleCaseImpl<IdTree>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;

    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData) {
        self.matrix_will_change_dirty(event.id, read, write);
    }
}

impl<'a> MultiCaseListener<'a, Node, Transform, ModifyEvent> for OverflowImpl {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Overflow>,
        &'a SingleCaseImpl<IdTree>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
        self.matrix_dirty(event.id, read, write);
    }
}

impl<'a> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for OverflowImpl {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Overflow>,
        &'a SingleCaseImpl<IdTree>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
        self.matrix_dirty(event.id, read, write);
    }
}

impl<'a> MultiCaseListener<'a, Node, Transform, CreateEvent> for OverflowImpl {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Overflow>,
        &'a SingleCaseImpl<IdTree>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData) {
        self.matrix_dirty(event.id, read, write);
    }
}

impl<'a> MultiCaseListener<'a, Node, Transform, DeleteEvent> for OverflowImpl {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Overflow>,
        &'a SingleCaseImpl<IdTree>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData) {
        self.matrix_dirty(event.id, read, write);
    }
}

impl<'a> MultiCaseListener<'a, Node, Layout, ModifyEvent> for OverflowImpl {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, Overflow>,
        &'a SingleCaseImpl<IdTree>,
    );
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
        self.matrix_dirty(event.id, read, write);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for OverflowImpl {
    type ReadData = Read<'a>;
    type WriteData = Write<'a>;

    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, mut write: Self::WriteData) {
        let node = &read.0[event.id];
        // 获得父节点的ByOverflow
        let mut by = *write.1[node.parent];
        let overflow = match read.1.get(node.parent) {
            Some(r) => **r,
            _ => false,
        };
        if overflow {
            let i = get_index(write.0, node.parent);
            if i > 0 {
                by = add_index(by, 1 << (i - 1));
            }
        }
        self.set_overflow(event.id, by, &read, &mut write);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for OverflowImpl {
    type ReadData = (
        &'a SingleCaseImpl<IdTree>,
        &'a MultiCaseImpl<Node, Overflow>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<OverflowClip>,
        &'a mut MultiCaseImpl<Node, ByOverflow>,
    );

    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData) {
        let node = &read.0[event.id];
        let notify = write.0.get_notify();
        if match read.1.get(event.id) {
            Some(r) => **r,
            _ => false,
        } {
            remove_index(&mut *write.0, event.id, &notify);
        }
        write.1.get_write(event.id).unwrap().set_0(0);
        // 递归调用，检查是否有overflow， 撤销设置OverflowClip
        for (id, _n) in read.0.recursive_iter(node.children.head) {
            if match read.1.get(id) {
                Some(r) => **r,
                _ => false,
            } {
                remove_index(&mut *write.0, id, &notify);
            }
            write.1.get_write(id).unwrap().set_0(0);
        }
    }
}

impl<'a> EntityListener<'a, Node, ModifyEvent> for OverflowImpl {
    type ReadData = (
        &'a SingleCaseImpl<IdTree>,
        &'a MultiCaseImpl<Node, Overflow>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<OverflowClip>,
        &'a mut MultiCaseImpl<Node, ByOverflow>,
    );

    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
        let overflow = match read.1.get(event.id) {
            Some(r) => **r,
            _ => false,
        };
        if overflow {
            let notify = write.0.get_notify();
            remove_index(&mut *write.0, event.id, &notify);
        }
    }
}

impl OverflowImpl {
    #[inline]
    fn mark_dirty(
        &mut self,
        id: usize,
        dirty_type: usize,
        layer: usize,
        style_marks: &mut MultiCaseImpl<Node, StyleMark>,
    ) {
        let style_mark = &mut style_marks[id];
        if style_mark.dirty1 & dirty_type == 0 {
            style_mark.dirty1 |= dirty_type;
            self.overflow_dirty.mark(id, layer);
        }
    }

    fn matrix_dirty(
        &mut self,
        id: usize,
        read: (
            &MultiCaseImpl<Node, ByOverflow>,
            &MultiCaseImpl<Node, Overflow>,
            &SingleCaseImpl<IdTree>,
        ),
        write: &mut MultiCaseImpl<Node, StyleMark>,
    ) {
        let node = match read.2.get(id) {
            Some(r) => r,
            None => return,
        };
        if node.layer == 0
            || (read.0[id].0 == 0
                && !match read.1.get(id) {
                    Some(r) => **r,
                    _ => false,
                })
        {
            return;
        }
        self.mark_dirty(id, StyleType1::Overflow as usize, node.layer, write);
    }

    fn matrix_will_change_dirty(
        &mut self,
        id: usize,
        read: (
            &MultiCaseImpl<Node, ByOverflow>,
            &MultiCaseImpl<Node, Overflow>,
            &SingleCaseImpl<IdTree>,
        ),
        write: &mut MultiCaseImpl<Node, StyleMark>,
    ) {
        let node = match read.2.get(id) {
            Some(r) => r,
            None => return,
        };
        if node.layer == 0 {
            return;
        }
        self.mark_dirty(id, StyleType1::Overflow as usize, node.layer, write);
    }

    // 递归调用，检查是否有overflow， 设置OverflowClip， 设置所有子元素的by_overflow
    fn set_overflow(&mut self, id: usize, mut by: usize, read: &Read, write: &mut Write) {
        if by > 0 {
            write.1.get_write(id).unwrap().set_0(by);
        }
        let overflow = match read.1.get(id) {
            Some(r) => **r,
            _ => false,
        };
        if overflow {
            // 添加根上的overflow的裁剪矩形
            // let i = set_index(&mut *write.0, 0, id);
            let i = create_clip(id, write.0);
            self.mark_dirty(
                id,
				StyleType1::Overflow as usize,
				read.0[id].layer,
                &mut write.3,
            );
            if i > 0 {
                by = add_index(by, 1 << (i - 1));
            }
        }

        let node = &read.0[id];
        for (id, _n) in read.0.iter(node.children.head) {
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
        if node.parent == 0 {
            return None;
        }
        id = node.parent;
    }
}



fn calc_clip<'a>(
    id: usize,
    mut by: usize,
    mut transform_will_change_matrix: Option<&'a TransformWillChangeMatrix>,
    mut by_clip_aabb: Option<&'a (Aabb3, Share<dyn UniformBuffer>)>,
    read: Read<'a>,
    overflow_clip: &'a mut SingleCaseImpl<OverflowClip>,
    by_overflows:&'a mut MultiCaseImpl<Node, ByOverflow>,
    cullings: &'a mut MultiCaseImpl<Node, Culling>,
    style_marks: &'a mut MultiCaseImpl<Node, StyleMark>,
) {
    if by > 0 {
        // 裁剪剔除
        if let Some(item) = by_clip_aabb {
            match transform_will_change_matrix {
                Some(m) => {
                    // 如果没有旋转
                    if !(m.0).1 {
                        cullings.get_write(id).unwrap().set_0(!is_intersect(
                            &item.0,
                            &matrix_mul_aabb(&m.0, &unsafe { read.6.get_unchecked(id) }.0),
                        ))
                    }
                }
                None => cullings.get_write(id).unwrap().set_0(!is_intersect(
                    &item.0,
                    &unsafe { read.6.get_unchecked(id) }.0,
                )),
            }
        }
        // 通知by_overflow改变，以修改clipBox
        by_overflows.get_notify().modify_event(id, "", 0);
    }

    if let Some(r) = read.7.get(id) {
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
            overflow_clip.get_notify().modify_event(i, "", id);
        }
    }

    style_marks[id].dirty1 &= !(StyleType1::Overflow as usize);

    let first = read.0[id].children.head;
    for (child_id, _child) in read.0.iter(first) {
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
// 设置裁剪区域，需要 world_matrix layout
fn set_clip(
    id: usize,
    i: usize,
    read: &Read,
    clip: &mut SingleCaseImpl<OverflowClip>,
    transform_will_change: Option<&TransformWillChangeMatrix>,
) {
    let mut world_matrix = &read.2[id];
    let temp;
    if let Some(r) = transform_will_change {
        temp = &r.0 * world_matrix;
        world_matrix = &temp;
    }
    let layout = &read.4[id];
    // BUG , overflow与tranwillchange在同一节点上， origin应该从tranwillchange上取
    let origin = match read.3.get(id) {
        Some(r) => r.origin.clone(),
        None => TransformOrigin::Center,
    };
    let origin = origin.to_value(layout.width, layout.height);
    let c = &mut clip.clip[i];
    *c = Clip {
        view: calc_point(layout, world_matrix, &origin),
        has_rotate: world_matrix.1,
        old_has_rotate: c.has_rotate,
        node_id: id,
    };
}

//================================ 内部静态方法
// 创建裁剪区域，需要 world_matrix layout
fn create_clip(id: usize, clip: &mut SingleCaseImpl<OverflowClip>) -> usize {
    if clip.clip.len() >= 32 {
        println!("clip overflow!!!!!!!");
        return 0;
    }
    let i = clip.clip.insert(Clip {
        view: [
            Point2::new(0.0, 0.0),
            Point2::new(0.0, 0.0),
            Point2::new(0.0, 0.0),
            Point2::new(0.0, 0.0),
        ],
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
fn add_index(by: usize, index: usize) -> usize {
    by | index
}
#[inline]
fn del_index(by: usize, index: usize) -> usize {
    by & !index
}
// 整理方法。设置或取消所有子节点的by_overflow上的index。
#[inline]
fn adjust(
    idtree: &SingleCaseImpl<IdTree>,
    by_overflow: &mut MultiCaseImpl<Node, ByOverflow>,
    child: usize,
    index: usize,
    ops: fn(a: usize, b: usize) -> usize,
) {
    for (id, _n) in idtree.recursive_iter(child) {
        let by = *by_overflow[id];
        by_overflow.get_write(id).unwrap().set_0(ops(by, index));
    }
}
// 计算内容区域矩形的4个点
fn calc_point(layout: &Layout, m: &Matrix4, origin: &Point2) -> [Point2; 4] {
    let width = layout.width
        - layout.padding_left
        - layout.padding_right
        - layout.border_left
        - layout.border_right;
    let height = layout.height
        - layout.padding_top
        - layout.padding_bottom
        - layout.border_top
        - layout.border_bottom;
    let start = (
        layout.border_left + layout.padding_left - origin.x,
        layout.border_top + layout.padding_top - origin.y,
    );
    let left_top = m * Vector4::new(start.0, start.1, 0.0, 1.0);
    let right_top = m * Vector4::new(start.0 + width, start.1, 0.0, 1.0);
    let left_bottom = m * Vector4::new(start.0, start.1 + height, 0.0, 1.0);
    let right_bottom = m * Vector4::new(start.0 + width, start.1 + height, 0.0, 1.0);

    let lt = Point2 {
        x: left_top.x,
        y: left_top.y,
    };
    let rt = Point2 {
        x: right_top.x,
        y: right_top.y,
    };
    let lb = Point2 {
        x: left_bottom.x,
        y: left_bottom.y,
    };
    let rb = Point2 {
        x: right_bottom.x,
        y: right_bottom.y,
    };
    [lt, lb, rb, rt]
}

// 计算aabb
fn matrix_mul_aabb(m: &WorldMatrix, aabb: &Aabb3) -> Aabb3 {
    let min = m * Vector4::new(aabb.min.x, aabb.min.y, 0.0, 1.0);
    let max = m * Vector4::new(aabb.max.x, aabb.max.y, 0.0, 1.0);
    Aabb3::new(
        Point3::new(min.x, min.y, 1.0),
        Point3::new(max.x, max.y, 1.0),
    )
}

// 将clip求交结果存储在clip_map中
fn add_intersect_clip(
    parent_by: usize,
    by: usize,
    i: usize,
    overflow: &mut OverflowClip,
    view_matrix: &WorldMatrix,
) {
    let r = &overflow.clip[i];
    if r.has_rotate {
        return;
    }
    if let Some(p) = overflow.clip_map.get(&parent_by) {
        let aabb = intersect(
            &Aabb3::new(
                Point3::new(r.view[0].x, r.view[0].y, 0.0),
                Point3::new(r.view[2].x, r.view[2].y, 0.0),
            ),
            &p.0,
        );
        overflow.insert_aabb(by, aabb, view_matrix);
        // overflow.clip_map.insert(by, intersect(&Aabb3::new(Point3::new(r.view[0].x, r.view[0].y, 0.0),  Point3::new(r.view[2].x, r.view[2].y, 0.0)), p));
    }
}

// 将clip求交结果存储在clip_map中
fn modify_intersect_clip<'a, 'b>(
    parent_by: usize,
    by: usize,
    i: usize,
    overflow: &'a mut OverflowClip,
    view_matrix: &'b WorldMatrix,
) -> Option<&'a (Aabb3, Share<dyn UniformBuffer>)> {
    let r = &overflow.clip[i];
    if r.has_rotate {
        overflow.clip_map.remove(&by);
        return None;
    }
    if let Some(p) = overflow.clip_map.get(&parent_by) {
        let aabb = intersect(
            &Aabb3::new(
                Point3::new(r.view[0].x, r.view[0].y, 0.0),
                Point3::new(r.view[2].x, r.view[2].y, 0.0),
            ),
            &p.0,
        );
        Some(overflow.insert_aabb(by, aabb, view_matrix))
    } else {
        None
    }
}

#[inline]
fn remove_intersect_clip(by: usize, overflow: &mut OverflowClip) {
    overflow.clip_map.remove(&by);
}

#[inline]
fn is_intersect(a: &Aabb3, b: &Aabb3) -> bool {
    if a.min.x > b.max.x || a.min.y > b.max.y || b.min.x > a.max.x || b.min.y > a.max.y {
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
    let mut aabb = Aabb3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 0.0));
    if a.min.x >= b.min.x {
        aabb.min.x = a.min.x;
    } else {
        aabb.min.x = b.min.x;
    }

    if a.max.x >= b.max.x {
        aabb.max.x = b.max.x;
    } else {
        aabb.max.x = a.max.x;
    }

    if a.min.y >= b.min.y {
        aabb.min.y = a.min.y;
    } else {
        aabb.min.y = b.min.y;
    }

    if a.max.y >= b.max.y {
        aabb.max.y = b.max.y;
    } else {
        aabb.max.y = a.max.y;
    }
    return aabb;
}

impl_system! {
    OverflowImpl,
    true,
    {
        EntityListener<Node, CreateEvent>
        EntityListener<Node, ModifyEvent>
        MultiCaseListener<Node, Overflow, ModifyEvent>
        MultiCaseListener<Node, Overflow, DeleteEvent>
        MultiCaseListener<Node, WorldMatrix, ModifyEvent>
        MultiCaseListener<Node, Transform, ModifyEvent>
        MultiCaseListener<Node, Transform, CreateEvent>
        MultiCaseListener<Node, Transform, DeleteEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        SingleCaseListener<IdTree, CreateEvent>
        SingleCaseListener<IdTree, DeleteEvent>
        MultiCaseListener<Node, TransformWillChangeMatrix, DeleteEvent>
        MultiCaseListener<Node, TransformWillChangeMatrix, ModifyEvent>
        MultiCaseListener<Node, TransformWillChangeMatrix, CreateEvent>
    }
}
