/// 八叉树单例封装
use pi_spatial::quad_helper::QuadTree;
use pi_slotmap::{Key, KeyData};

use ecs::monitor::NotifyImpl;

use crate::component::user::{Aabb2, Point2, Vector2};

// use crate::Z_MAX;

#[derive(Deref, DerefMut, Clone, Copy, Hash, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct OctKey(pub usize);

impl Default for OctKey {
    fn default() -> Self {
        Self(std::usize::MAX)
    }
}

unsafe impl Key for OctKey {
    fn data(&self) -> KeyData {
        KeyData::from_ffi((1 << 32) | self.0 as u64)
    }
}

impl From<KeyData> for OctKey {
    fn from(value: KeyData) -> Self {
		Self((value.as_ffi() & 0xffff_ffff) as usize)
    }
}

#[derive(Deref, DerefMut)]
pub struct Oct(QuadTree<OctKey, usize>);



impl Oct {
    pub fn new() -> Self {
		let max = Vector2::new(1024f32, 1024f32);
    	let min = Vector2::new(16f32, 16f32);
        Oct(QuadTree::new(
            Aabb2::new(
                Point2::new(-1024f32, -1024f32),
                Point2::new(4096f32, 4096f32),
            ),
			max,
            min,
            0,
            0,
			16, //????
        ))
	}

	pub fn with_capacity(capacity: usize) -> Self {
		let max = Vector2::new(100f32, 100f32);
    	let min = max / 100f32;
        Oct(QuadTree::new(
            Aabb2::new(
                Point2::new(-1024f32, -1024f32),
                Point2::new(4096f32, 4096f32),
            ),
            max,
            min,
            0,
			0,
			16
        ))
	}

    // pub fn mem_size(&self) -> usize {
    //     self.0.mem_size()
    // }
    // 添加一个aabb及其绑定
    pub fn add(&mut self, id: usize, aabb: Aabb2, bind: usize, notify: Option<&NotifyImpl>) {
        self.0.add(OctKey(id), aabb, bind);
        match notify {
            Some(n) => n.create_event(id),
            _ => (),
        }
	}
	
	// 添加一个aabb及其绑定
    pub fn get(&self, id: usize) -> Option<&(Aabb2, usize)> {
        self.0.get(OctKey(id))
    }

    // 更新指定id的aabb
    pub fn update(&mut self, id: usize, aabb: Aabb2, notify: Option<&NotifyImpl>) -> bool {
		// 更新前发出修改事件（特殊处理，因为需要修改前的包围盒用于计算最终显示界面的最大修改包围盒）
		match notify {
			Some(n) => n.modify_event(id, "", 0), // 0代表更新前
			_ => (),
		};
        let r = self.0.update(OctKey(id), aabb);
        if r {
            match notify {
                Some(n) => n.modify_event(id, "", 1), // 1代表更新后
                _ => (),
            }
        }
        r
    }
    // 移动指定id的aabb，性能比update要略好
    pub fn shift(&mut self, id: usize, distance: Vector2, notify: Option<&NotifyImpl>) -> bool {
        let r = self.0.shift(OctKey(id), distance);
        if r {
            match notify {
                Some(n) => n.modify_event(id, "", 0),
                _ => (),
            }
        }
        r
    }

    // 移除指定id的aabb及其绑定
    pub fn remove(&mut self, id: usize, notify: Option<&NotifyImpl>) -> Option<(Aabb2, usize)> {
		if self.0.get(OctKey(id)).is_some() {
			match notify {
				Some(n) => n.delete_event(id),
				_ => (),
			}
			let r = self.0.remove(OctKey(id));
			return r;
		}
        None
    }
}
