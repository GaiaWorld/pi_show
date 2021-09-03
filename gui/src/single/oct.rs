use bevy_ecs::event::EventWriter;

use bevy_ecs::prelude::Entity;
/// 八叉树单例封装
use spatialtree::QuadTree;

// use ecs::monitor::NotifyImpl;

use crate::component::user::{Aabb2, Point2, Vector2};
use crate::util::event::{EntityEvent, ImMessenger};
use crate::component::calc::StyleIndex;

#[derive(Deref, DerefMut)]
pub struct Oct(QuadTree<f32, usize>);

impl Oct {
    pub fn new() -> Self {
		let max = Vector2::new(100f32, 100f32);
    	let min = max / 100f32;
        Oct(QuadTree::new(
            Aabb2::new(
                Point2::new(-1024f32, -1024f32),
                Point2::new(3072f32, 3072f32),
            ),
			max,
            min,
            0,
            0,
			16, //????
        ))
	}

	pub fn with_capacity(_capacity: usize) -> Self {
		let max = Vector2::new(100f32, 100f32);
    	let min = max / 100f32;
        
		let r = Oct(QuadTree::new(
            Aabb2::new(
                Point2::new(-1024f32, -1024f32),
                Point2::new(3072f32, 3072f32),
            ),
            max,
            min,
            0,
			0,
			16
        ));
		r
	}

    pub fn mem_size(&self) -> usize {
        self.0.mem_size()
    }
    // 添加一个aabb及其绑定
    pub fn add(&mut self, entity: Entity, aabb: Aabb2, bind: usize, notify: Option<&mut ImMessenger<EntityEvent<Self>>>) {
        self.0.add(entity.id() as usize, aabb, bind);
        match notify {
            Some(n) => n.send(EntityEvent::new_create(entity)),
            _ => (),
        }
	}
	
	// 添加一个aabb及其绑定
    pub fn get(&self, id: usize) -> Option<(&Aabb2, &usize)> {
        self.0.get(id)
    }

    // 更新指定id的aabb
    pub fn update(&mut self, entity: Entity, aabb: Aabb2, mut notify: Option<&mut ImMessenger<EntityEvent<Self>>>) -> bool {
		// 更新前发出修改事件（特殊处理，因为需要修改前的包围盒用于计算最终显示界面的最大修改包围盒）
		match &mut notify {
			Some(n) => (*n).send(EntityEvent::new_modify(entity, StyleIndex::Oct)),
			_ => (),
		};
        let r = self.0.update(entity.id() as usize, aabb);
        if r {
            match notify {
                Some(n) => n.send(EntityEvent::new_modify(entity, StyleIndex::Oct)),
                _ => (),
            }
        }
        r
    }
    // // 移动指定id的aabb，性能比update要略好
    // pub fn shift(&mut self, id: usize, distance: Vector2, notify: Option<&mut EventWriter<EntityEvent<Self>>>) -> bool {
    //     let r = self.0.shift(id, distance);
    //     if r {
    //         match notify {
    //             Some(n) => n.send(EntityEvent::Modify(id, "", 0)),
    //             _ => (),
    //         }
    //     }
    //     r
    // }

    // 移除指定id的aabb及其绑定
    pub fn remove(&mut self, entity: Entity, notify: Option<&mut EventWriter<EntityEvent<Self>>>) -> Option<(Aabb2, usize)> {
		let id = entity.id() as usize;
		if self.0.get(id).is_some() {
			match notify {
				Some(n) => n.send(EntityEvent::new_delete(entity)),
				_ => (),
			}
			let r = self.0.remove(id);
			return r;
		}
        None
    }
}
