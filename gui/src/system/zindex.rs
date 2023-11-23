//! zindex系统
//! zindex的[min max), 采用Range, 左闭右开区间。
//! 预计每个节点自身占据Z_SELF个数，Z_SELF一般等于3.
//! 每节点考虑会有两边空间隔及自身， Z_SPLIT=3， Count个节点最多产生Count*Z_SPLIT个段。
//! 判断节点的zrange是否足够，在于全部的子节点数量(Count+1)*Z_SELF 不小于zrange。
//! 分配的间隔： S = (zrange-(Count+1)*Z_SELF)/(Count*Z_SPLIT), 为整数。 每个空间隔和节点都会加上这个S.
//! 分配节点的range为: 自身空间(S+Z_SELF) + 子节点及间隔空间(Count*(S*Z_SPLIT+Z_SELF))
//! 设计分配如下： 如果父容器为 0 6.
//! 子节点为1个的话，间隔为0： Empty(3,3), Node(3,6), Empty(6,6).
//! 设计分配如下： 如果父容器为 0 9.
//! 子节点为2个的话，间隔为0： Empty(3,3), Node(3,6), Empty(6,6), Node(6,9), Empty(9,9).
//! 设计分配如下： 如果父容器为 0 9.
//! 递归子节点为2个的话，间隔为0： Empty(3,3), Node(3,9), Empty(9,9).
//!                               Empty(6,6), Node(6,9), Empty(9,9).
//! 设计分配如下： 如果父容器为 0 9.
//! 子节点为1个的话，间隔为1： Empty(3,4), Node(4,8), Empty(8,9).
//! 设计分配如下： 如果父容器为 0 15.
//! 子节点为2个的话，间隔为1： Empty(3,4), Node(4,8), Empty(8,9), Node(9,13), Empty(13,15).
//! 递归子节点为2个的话，间隔为1： Empty(3,4), Node(4,14), Empty(14,15).
//!                               Empty(7,8), Node(8,12), Empty(12,13).
//!
//! 判断节点脏时，首先收集当前节点排序环境下的子节点，排序，然后：
//! 一类是节点重算zrange，重置全部的子节点的zrange。
//! 另一类是父节点下的子节点局部比较：顺序找到没有脏标志的节点，将其前面的节点重算zrange，继续选择没有脏标志的节点。 需要保证，前后节点区间的zrange能装的下所在的递归子节点，如果装不下，则扩大区间。
//!
//! # 注意
//! 本系统能够计算ZRange的前提是，ZRange组件必须存在于节点上，本系统不会新增ZRange组件

use std::ops::Range;

use crate::{util::{Dirty as LayerDirty, vecmap_default::VecMapWithDefault, DirtyMark}, component::calc::ZRange};

use ecs::{Event, component::MultiCaseImpl, monitor::{CreateEvent, ModifyEvent}, single::SingleCaseImpl, system::{Runner, MultiCaseListener, SingleCaseListener, EntityListener}};

use map::Map;
use crate::Z_MAX;
use crate::single::IdTree;
use crate::entity::Node;
use crate::component::{
	user::ZIndex as ZI,
	calc::NodeState,
};
use ecs::component::Component;

type Entity = usize;

/// 如果节点设置zindex为auto，则自身zindex为-1
const Z_AUTO: isize = -1;
/// 节点zindex的最大区间
// const Z_MAX: usize = 16;//usize::MAX;
/// 每个节点自身占用的zindex区间大小
const Z_SELF: usize = 1;
/// 子节点将区间劈分成3段，自身在中间段
const Z_SPLIT: usize = 3;

pub struct ZIndexImpl {
	dirty: LayerDirty,
	map: VecMapWithDefault<ZI>,
	cache: Vec<ZSort>,
}

impl ZIndexImpl {
	pub fn new() -> ZIndexImpl {
		ZIndexImpl {
		dirty: LayerDirty::default(),
		map: VecMapWithDefault::default(),
		cache: Vec::new(),
		}
	}

	pub fn with_capacity(capacity: usize) -> ZIndexImpl {
		ZIndexImpl {
			dirty: LayerDirty::default(),
			map: VecMapWithDefault::with_capacity(capacity),
			cache: Vec::new(),
		}
	}
}


// 监听节点创建, 插入ZRange
impl<'a> EntityListener<'a, Node, CreateEvent> for ZIndexImpl {
    type ReadData = &'a MultiCaseImpl<Node, NodeState>;
    type WriteData = &'a mut MultiCaseImpl<Node, ZRange>;

    fn listen(&mut self, event: &Event, node_states: Self::ReadData, z_ranges: Self::WriteData) {
		// log::warn!("node_create==========={:?}, is_rnode{:?}", event.id,  node_states.get(event.id).map(|r| {r.is_rnode()}));
		match node_states.get(event.id) {
			Some(r) if !r.is_rnode() => return,
			None => return,
			_ => ()
		}

		
		z_ranges.insert(event.id, ZRange::default());
    }
}

impl<'a> MultiCaseListener<'a, Node, ZI, (CreateEvent, ModifyEvent)> for ZIndexImpl {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();

    fn listen(&mut self, event: &Event, id_tree: Self::ReadData, _write: Self::WriteData) {
		self.dirty.marked_dirty(event.id, id_tree, 1);
    }
}

// 递归设置每个节点脏
impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for ZIndexImpl {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, NodeState>);
    type WriteData = ();

    fn listen(&mut self, event: &Event, (idtree, node_states): Self::ReadData, _write: Self::WriteData) {
		// log::warn!("IdTree crate==========={:?}, is_rnode{:?}", event.id, node_states.get(event.id).map(|r| {r.is_rnode()}));
		match node_states.get(event.id) {
			Some(r) if !r.is_rnode() => return,
			None => return,
			_ => ()
		}

		self.dirty.marked_dirty(event.id, idtree, 1);
		let head = idtree[event.id].children().head;
		for (child, _) in idtree.recursive_iter(head) {
			match node_states.get(event.id) {
				Some(r) if !r.is_rnode() => return,
				None => return,
				_ => ()
			}
			self.dirty.marked_dirty(child, idtree, 1);
		}
    }
}

impl<'a> Runner<'a> for ZIndexImpl {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, ZI>, &'a MultiCaseImpl<Node, NodeState>);
    type WriteData = &'a mut MultiCaseImpl<Node, ZRange>;

    fn setup(&mut self, _read: Self::ReadData, _write: Self::WriteData) {
    }
    fn run(&mut self, (tree, zindexs, node_states): Self::ReadData, ranges: Self::WriteData) {
		let mut vec: &mut Vec<ZSort> = &mut self.cache;
		for (id, layer) in self.dirty.dirty.iter() {
			match tree.get(*id) {
				Some(node) if node.parent() > 0 => {
					if let Some(r) = self.dirty.dirty_mark_list.get(id) {
						if r.layer != layer {
							continue;
						}
					} else {
						continue;
					}
					match node_states.get(*id) {
						Some(r) if !r.is_rnode() => continue,
						None => continue,
						_ => ()
					}
					let parent = node.parent();
					// 找到能容纳所有子节点的父节点
					// parent节点zindex为AUTO，需要递归向上找到一个不是AUTO的节点, 以该节点作为布局环境，进行z布局
					// 如果parent节点无法容纳三倍子节点， 也需要向上递归，找到能容纳三倍子节点的节点作为布局环境进行z布局
					let (parent1, children_count, zrange, local) = get_parent(zindexs, tree, ranges, parent);
					// log::warn!("calc_zindex======node: {:?}, parent: {:?}, parent1: {:?}, layer: {:?} ", id, node.parent(), parent1, node.layer());
					// 收集父节点排序环境下的子节点
					collect(zindexs, &tree, vec, parent1, 0, node_states);
					// 排序
					vec.sort();
					// println!("---------local:{}, {:?}", local, vec);
					if local {
						// 如果是可以进行局部比较
						local_reset(zindexs, tree, &mut self.dirty.dirty_mark_list, ranges, node_states, &mut vec, children_count, zrange);
					} else {
						// 否则父节点重新设置zrange
						reset(zindexs, tree, &mut self.dirty.dirty_mark_list, ranges, node_states, &mut vec, 0, children_count, zrange);
					}
				}
				_ => {
					// 根节点设置为最大值
					let _ = ranges.get_mut(*id).map(|r| {
						*r = ZRange(Range { start: 0, end: Z_MAX as usize });
					});
				}
			}
		}
		self.dirty.dirty_mark_list.clear();
		self.dirty.dirty.clear();
		vec.clear();
    }
    fn dispose(&mut self, _read: Self::ReadData, _write: Self::WriteData) {

    }
}

// sys.es6.js?UYrzHvK7tB8BRAmJ3JrJZ6:4903 range_set========[(21, ZRange(5557..60992), 14632, 0, ZRange(5557..5558), 0, 0, 16), (20, ZRange(5558..60992), 14632, 0, ZRange(5558..5559), 0, 0, 16), (19, ZRange(5559..60992), 14632, 0, ZRange(5559..5560), 0, 0, 16), (18, ZRange(5560..60992), 14632, 0, ZRange(5560..5561), 0, 0, 16), (17, ZRange(5561..60992), 14632, 0, ZRange(5561..5639), 77, 0, 16), (16, ZRange(5639..60992), 14632, 0, ZRange(5639..5640), 0, 0, 16), (15, ZRange(5640..60992), 14632, 0, ZRange(5640..5641), 0, 0, 16), (14, ZRange(5641..60992), 14632, 0, ZRange(5641..20181), 14539, 0, 16), (13, ZRange(20181..60992), 14632, 0, ZRange(20181..20182), 0, 0, 16), (12, ZRange(20182..60992), 14632, 0, ZRange(20182..20183), 0, 0, 16), (11, ZRange(20183..60992), 14632, 0, ZRange(20183..20184), 0, 0, 16), (10, ZRange(20184..60992), 14632, 0, ZRange(20184..20185), 0, 0, 16), (9, ZRange(20185..60992), 14632, 0, ZRange(20185..20186), 0, 0, 16), (8, ZRange(20186..60992), 14632, 0, ZRange(20186..20187), 0, 0, 16), (7, ZRange(20187..60992), 14632, 0, ZRange(20187..20188), 0, 0, 16), (6, ZRange(20188..60992), 14632, 0, ZRange(20188..20189), 0, 0, 16)]


/// 获得能装下全部子节点的父节点
fn get_parent(query: &MultiCaseImpl<Node, ZI>, tree: &IdTree, ranges: &MultiCaseImpl<Node, ZRange>, mut node: Entity) -> (Entity, usize, ZRange, bool) {
    let mut local = true;
    // println!("node:{:?}, ", &node);
    loop {
        if let Some(z) = query.get(node) {
            if z.0 == Z_AUTO {
                // 如果该节点设置为Z_AUTO，则没有自己的排序环境，继续向父节点寻找
                node = tree[node].parent();
                // 有可能父不存在， 则直接将该节点当做非auto的节点处理
                if node > 0 {
                    continue;
                }
            }
        }

        let children_count = tree[node].count();
        let range = match ranges.get(node) {
            Some(r) => r.clone(),
            _ => ZRange::default(),
        };

        // log::warn!("get_parent======node: {:?}, parent: {:?}, children_count: {:?}, layer: {:?}, z_index: {:?}, z_range: {:?} ", node, tree[node].parent(), children_count, tree[node].layer(), query.get(node), range);
		if range.end - range.start >= children_count + 1 {
        // if range.end - range.start >= (children_count + 1) * Z_SELF {
            return (node, children_count, range, local);
        }
        // println!("node range:{:?}, children_count:{}", range, children_count);
        // 节点的范围应该包含自身和递归子节点的z范围

        node = tree[node].parent();
        local = false // 因为父节点上没有脏标记，所以无法使用局部脏算法，只能全部排序
    }
}

/// 节点排序对象， 依次比较zindex, 自身所在位置
#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct ZSort {
    z_index: isize,
    index: usize,
    node: usize,
    children_count: usize,
}
impl ZSort {
    fn new(z_index: isize, index: usize, node: Entity, children_count: usize) -> Self {
        ZSort {
            z_index,
            index,
            node,
            children_count,
        }
    }
}
/// 收集父节点排序环境下的子节点
fn collect(query: &MultiCaseImpl<Node, ZI>, tree: &IdTree, vec: &mut Vec<ZSort>, parent: Entity, mut index: usize, node_states: &MultiCaseImpl<Node, NodeState>) -> usize {
    if let Some(node) = tree.get(parent) {
		let head = node.children().head;
        for (child_id, _) in tree.iter(head) {
			match node_states.get(child_id) {
				Some(r) if !r.is_rnode() => continue,
				None => continue,
				_ => ()
			}
            // 获得该节点的zindex
            let z = if let Some(z) = query.get(child_id) {
                if z.0 == Z_AUTO {
                    // 如果该节点设置为Z_AUTO，则没有自己的排序环境，继续遍历其子节点
                    vec.push(ZSort::new(z.0, index, child_id, 0));
                    index = collect(query, tree, vec, child_id, index + 1, node_states);
                    continue;
                }
                z.0
            } else {
                0
            };
            // 获得该节点的递归子节点数
            let c = if let Some(node) = tree.get(child_id) { node.count() } else { 0 };
            vec.push(ZSort::new(z, index, child_id, c));
            index += 1;
        }
    }
    index
}

#[inline]
fn get_or_default<T: Clone + Default + Component>(query: &MultiCaseImpl<Node, T>, id: Entity) -> T {
    match query.get(id) {
        Some(r) => r.clone(),
        _ => T::default(),
    }
}

/// 脏状态
#[derive(Debug)]
struct Dirty {
    children_count: usize, // 子节点总数量
    // begin、count、start分别描述了同一个节点的三个不同属性，
    begin: usize, // 在子元素数组中的索引，
    count: usize, // 顺序扫描， 统计了上次扫描到的不脏的节点到当前节点范围内的所有子节点
    start: usize, // z值起始范围
}
/// 父节点下的子节点局部比较
fn local_reset(
    query: &MultiCaseImpl<Node, ZI>,
    tree: &IdTree,
    mark: &mut VecMapWithDefault<DirtyMark>,
    ranges: &mut MultiCaseImpl<Node, ZRange>,
	node_states: &MultiCaseImpl<Node, NodeState>,
    vec: &mut Vec<ZSort>,
    children_count: usize,
    mut zrange: ZRange,
) {
    fn empty(_mark: &mut VecMapWithDefault<DirtyMark>, _node: &Entity) {}
    zrange.start += Z_SELF;
    // 脏状态
    let mut dirty = Dirty {
        children_count,
        count: 0,
        begin: 0,
        start: zrange.start,
    };
    let len = vec.len();
    for i in 0..len {
        let id = vec[i].node;
        // 获得当前节点及子节点的数量
        let cur_count = vec[i].children_count + 1;
        // 寻找修改的节点
        // 清理脏标志，这样层脏迭代器就不会弹出这个节点
        // println!("mark clear11, {}", vec[i].node.local().offset());
        if mark.remove(&id).is_some() {
            // println!("mark clear, {}", vec[i].node.local().offset());
            // 如果节点脏了， 统计到dirty的数量中
            dirty.count += cur_count;
            continue;
        }
        // 直到找到了一个未被修改的节点，下面成这个未被修改的节点为当前节点

        // 获得当前节点的zrange
        let cur_range = get_or_default(ranges, id);
        // log::warn!("local_reset====id: {:?}, zrange: {:?}, dirty: {:?}, cur_range: {:?}, cur_count: {:?}, down: {:?}, i: {:?}, len: {:?}", id, zrange, dirty, cur_range, cur_count, tree[id].children(), i, len);
        // 判断当前节点右边（不包含当前节点）能否放下，如果不行，则继续（右边z范围不能容纳右边的所有子节点， 将当前节点卷入脏统计，并继续）
        if zrange.end - cur_range.end < (dirty.children_count - dirty.count - cur_count) * Z_SELF {
            dirty.count += cur_count;
            continue;
        }
        // 右边已经能容纳右边的节点，而当前统计的脏节点数量为0， 由于当前节点不脏， 则不需要处理当前节点
        if dirty.count == 0 {
            dirty.begin = i + 1;
            dirty.start = cur_range.end;
            continue;
        }
        // 右边已经能容纳右边的节点，如果左边（包含当前节点）也不能能容纳左边（包含当前节点）的节点 则将当前节点卷入脏统计，并继续
        if cur_range.end - dirty.start < (dirty.count + cur_count) * Z_SELF {
            dirty.count += cur_count;
            continue;
        }

        //
        let (r, start, end) = if cur_range.start - dirty.start < dirty.count * Z_SELF {
            // 不含当前节点的情况下， 左侧需要调整容量无法容纳左侧需要调整节点，则将当前节点自身纳入脏统计
            dirty.count += cur_count;
            (ZRange(dirty.start..cur_range.end), dirty.begin, i + 1) // 含自身
        } else {
            (ZRange(dirty.start..cur_range.start), dirty.begin, i)
        };
        // // 前面有被修改节点，则获取脏段
        // let r = dirty_range(ranges, vec, zrange.start, range.start, &mut dirty);
        // dirty.start = range.end;
        // log::warn!("local_reset====start: {:?}, end: {:?}, zrange: {:?}, dirty: {:?}", &vec[start].node, &vec[end - 1].node, zrange, dirty);
        // 重置脏段
        range_set(query, tree, mark, ranges, node_states, vec, start, end, dirty.count, r, empty);
        // 将总子节点数量减去已经处理的数量
        dirty.children_count -= dirty.count;
        dirty.count = 0;
        dirty.begin = i + 1;
        dirty.start = cur_range.end;
    }
    // println!("dirty.count, {}", dirty.count);
    if dirty.count > 0 {
        // log::warn!("local_reset1====start: {:?}, end: {:?}, zrange: {:?}, dirty: {:?}", &vec[dirty.begin as usize].node, &vec[len - 1].node, zrange, dirty);
        // 前面有被修改节点，则获取脏段
        // let r = dirty_range(ranges, vec, zrange.start, zrange.end, &mut dirty);
        range_set(
            query,
            tree,
            mark,
            ranges,
			node_states,
            vec,
            dirty.begin as usize,
            len,
            dirty.count,
            ZRange(dirty.start..zrange.end),
            empty,
        );
    }
    // 清空
    vec.clear();
}
// /// 获取脏段，如果左边都可以放下，则返回true，否则返回false
// fn dirty_range(ranges: &Query<&mut ZRange>, vec: &Vec<ZSort>, parent_start: usize, dirty_end: usize, dirty: &mut Dirty) -> ZRange {
//     // println!("dirty_range, parent_start:{}, dirty_end:{}, dirty:{:?}", parent_start, dirty_end, dirty);
//     // 然后判断左边能否放下， 放不下， 则尝试向左移动，再次尝试能否放下
//     loop {
// 		// log::warn!("dirty======{:?}, {:?}, {:?}", dirty, dirty_end, Z_SELF);
//         // 判断左节点端及其子节点，都能被装下
//         if dirty_end - dirty.start >= dirty.count * Z_SELF {
//             return ZRange(Range {
//                 start: dirty.start,
//                 end: dirty_end,
//             });
//         }
//         if dirty.begin < 0 {
//             dirty.start = parent_start;
//         } else {
//             dirty.start = get_or_default(ranges, *vec[dirty.begin as usize].node).end;
// 			dirty.count += vec[dirty.begin as usize].children_count + 1;
// 			dirty.begin -= 1;
// 		}
//     }
// }
/// 设置子节点数组中一段节点的ZRange，并递归设置子节点的ZRange
fn range_set(
    query: &MultiCaseImpl<Node, ZI>,
    tree: &IdTree,
    mark: &mut VecMapWithDefault<DirtyMark>,
    ranges: &mut MultiCaseImpl<Node, ZRange>,
	node_states: &MultiCaseImpl<Node, NodeState>,
    vec: &mut Vec<ZSort>,
    begin: usize,
    end: usize,
    children_count: usize,
    mut zrange: ZRange,
    func: fn(&mut VecMapWithDefault<DirtyMark>, &Entity),
) {
    // println!("range set: zrange:{:?}, begin: {}, end: {}, count: {}", zrange, begin, end, children_count);
    // 获得间隔s
    let s = (zrange.end - zrange.start - children_count * Z_SELF) / (children_count * Z_SPLIT);
    zrange.start += s;
	let mut arr = Vec::new();
    for i in begin..end {
        let count = vec[i].children_count;
        let node = vec[i].node;
        func(mark, &node); // 移除脏标记
        // 分配节点的range为: 自身空间(S+Z_SELF) + 子节点及间隔空间(Count*(S*Z_SPLIT+Z_SELF))
        let r = ZRange(Range {
            start: zrange.start,
            end: zrange.start + s + Z_SELF + count * (s * Z_SPLIT + Z_SELF),
        });
		arr.push((node, zrange.clone(), children_count, s, r.clone(), count));
        zrange.start = r.end + s;
        set(query, tree, mark, ranges, node_states, vec, node, count, r);
    }
	//  log::warn!("range_set========{:?}, {:?}, {:?}", begin, end, &arr);
}
/// 父节点下的子节点全部重置zrange
fn reset(
    query: &MultiCaseImpl<Node, ZI>,
    tree: &IdTree,
    mark: &mut VecMapWithDefault<DirtyMark>,
    ranges: &mut MultiCaseImpl<Node, ZRange>,
	node_states: &MultiCaseImpl<Node, NodeState>,
    vec: &mut Vec<ZSort>,
    index: usize,
    children_count: usize,
    mut zrange: ZRange,
) {
    zrange.start += Z_SELF;
    let len = vec.len();
    fn mark_remove(mark: &mut VecMapWithDefault<DirtyMark>, node: &Entity) {
        // 清理脏标志，这样层脏迭代器就不会弹出这个节点
        mark.remove(&node);
    }
    // log::warn!("reset========list: {:?}", &vec[index..len]);
    range_set(query, tree, mark, ranges, node_states, vec, index, len, children_count, zrange, mark_remove);
    // 清空
    vec.truncate(index);
}

/// 设置指定节点的ZRange，并递归设置子节点的ZRange
fn set(
    query: &MultiCaseImpl<Node, ZI>,
    tree: &IdTree,
    mark: &mut VecMapWithDefault<DirtyMark>,
    ranges: &mut MultiCaseImpl<Node, ZRange>,
	node_states: &MultiCaseImpl<Node, NodeState>,
    vec: &mut Vec<ZSort>,
    node: Entity,
    children_count: usize,
    zrange: ZRange,
) {
    if let Some(r) = ranges.get_mut(node) {
        if *r == zrange {
            return;
        }
        *r = zrange.clone();
		ranges.get_notify_ref().modify_event(node, "", 0);
        // log::warn!("set=========node: {:?}, z: {:?}", node, zrange);
        if children_count > 0 {
            let len = vec.len();
            // 收集该节点的排序环境下的子节点
            collect(&query, &tree, vec, node, 0, node_states);
            // 对新增的子节点进行排序
            let new_len = vec.len();
            // log::warn!("set1========list: {:?}", &vec[len..new_len]);
            vec[len..new_len].sort();
            // 递归设置zrange
            reset(query, tree, mark, ranges, node_states, vec, len, children_count, zrange);
        }
    }
}

impl_system!{
    ZIndexImpl,
    true,
    {
        EntityListener<Node, CreateEvent>
        // EntityListener<Node, DeleteEvent>
		MultiCaseListener<Node, ZI, (CreateEvent, ModifyEvent)>
		SingleCaseListener<IdTree, CreateEvent>
		// SingleCaseListener<IdTree, DeleteEvent>
        // SingleCaseListener<IdTree, DeleteEvent>
    }
}


// #[cfg(test)]
// mod test {
//     use bevy_ecs::app::{App, CoreStage};
//     use bevy_ecs::{
//         prelude::{Entity, EventWriter, World},
//         query::{Changed, QueryState},
//         system::{Local, Res, ResMut, Resource, SystemState},
//     };
//     use pi_bevy_ecs_extend::{
//         prelude::{Down, IdTreeMut, Layer, Up},
//         system_param::layer_dirty::ComponentEvent,
//     };
//     use pi_null::Null;

//     use crate::{
//         components::{
//             calc::{EntityKey, ZRange},
//             user::ZI,
//         },
//         system::node::z_index::calc_zindex,
//     };

//     #[derive(Resource, Deref)]
//     pub struct RootNode(Entity);

//     fn add(v: &mut isize) -> isize {
//         *v = *v + 1;
//         *v
//     }

//     fn init_1(
//         world: &mut World,
//         entity_tree: &mut SystemState<IdTreeMut>,
//         event_writer: &mut SystemState<EventWriter<ComponentEvent<Changed<ZI>>>>,
//         root: &mut SystemState<ResMut<RootNode>>,
//     ) {
//         let root = **root.get_mut(world);
//         entity_tree.get_mut(world).insert_child(root, *EntityKey::null(), 0);

//         let mut i = 0;
//         // 插入2个节点作为子节点,以根节点作为父节点
//         let id = world
//             .spawn((ZI(add(&mut i)), ZRange::default(), Up::default(), Down::default(), Layer::default()))
//             .id();
//         entity_tree.get_mut(world).insert_child(id, root, 0);
//         event_writer.get_mut(world).send(ComponentEvent::new(id));

//         let id = world
//             .spawn((ZI(add(&mut i)), ZRange::default(), Up::default(), Down::default(), Layer::default()))
//             .id();
//         entity_tree.get_mut(world).insert_child(id, root, 0);
//         event_writer.get_mut(world).send(ComponentEvent::new(id));
//     }

//     fn init_2(
//         world: &mut World,
//         entity_tree: &mut SystemState<IdTreeMut>,
//         root: &mut SystemState<Res<RootNode>>,
//         event_writer: &mut SystemState<EventWriter<ComponentEvent<Changed<ZI>>>>,
//         mut local: Local<usize>,
//     ) {
//         *local += 1;
//         if *local != 2 {
//             return;
//         }


//         let root = **root.get_mut(world);
//         let id = world
//             .spawn((ZI(3), ZRange::default(), Up::default(), Down::default(), Layer::default()))
//             .id();
//         // 插入1个节点作为子节点,以根节点作为父节点
//         entity_tree.get_mut(world).insert_child(id, root, 0);
//         event_writer.get_mut(world).send(ComponentEvent::new(id));
//     }

//     fn init_3(
//         world: &mut World,
//         entity_tree: &mut SystemState<IdTreeMut>,
//         root: &mut SystemState<Res<RootNode>>,
//         event_writer: &mut SystemState<EventWriter<ComponentEvent<Changed<ZI>>>>,
//         mut local: Local<usize>,
//     ) {
//         *local += 1;
//         if *local != 3 {
//             return;
//         }

//         let root = **root.get_mut(world);
//         let id = world
//             .spawn((ZI(4), ZRange::default(), Up::default(), Down::default(), Layer::default()))
//             .id();
//         // 插入1个节点作为子节点,以根节点作为父节点
//         entity_tree.get_mut(world).insert_child(id, root, 0);
//         event_writer.get_mut(world).send(ComponentEvent::new(id));
//     }


//     #[test]
//     fn test() {
//         env_logger::Builder::default().filter(None, log::LevelFilter::Warn).init();

//         let mut app = App::default();
//         app.add_event::<ComponentEvent<Changed<ZI>>>();

//         let mut query = app.world.query::<(Entity, Option<&ZI>, &ZRange)>();

//         let root = app.world.spawn((ZRange(0..16), Up::default(), Down::default(), Layer::default())).id();

//         app.insert_resource(RootNode(root))
//             .add_startup_system(init_1) // 插入根节点；插入前两个实体，以根节点作为父节点
//             .add_system_to_stage(CoreStage::PreUpdate, init_2) // 插入第3个实体，以根节点作为父节点
//             .add_system_to_stage(CoreStage::PreUpdate, init_3) // 插入第4个实体，以根节点作为父节点
//             .add_systems(Update, calc_zindex)
//             .update();
//         asset(&mut app.world, &mut query, vec![(0, (0, 16)), (1, (4, 8)), (2, (9, 13))]);
//         println!("------------------------");


//         app.update();
//         asset(&mut app.world, &mut query, vec![(0, (0, 16)), (1, (4, 8)), (2, (9, 13)), (3, (13, 16))]);
//         println!("------------------------");

//         app.update();
//         asset(
//             &mut app.world,
//             &mut query,
//             vec![(0, (0, 16)), (1, (3, 6)), (2, (6, 9)), (3, (9, 12)), (4, (12, 15))],
//         );
//     }

//     fn asset(world: &mut World, query: &mut QueryState<(Entity, Option<&ZI>, &ZRange)>, result: Vec<(usize, (usize, usize))>) {
//         for (e, z, r) in query.iter_mut(world) {
//             let i = &result[e.index() as usize];
//             println!("=========, id:{:?}, z_index:{:?}, result: {:?}, expect: {:?}", e.index(), z, r, i.1);
//             assert_eq!(i.1 .0, r.0.start);
//             assert_eq!(i.1 .1, r.0.end);
//         }
//     }
// }
