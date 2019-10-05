/// 八叉树单例封装

use octree::Tree;

use ecs::monitor::NotifyImpl;


use component::user::{Aabb3, Point3, Vector3};

use Z_MAX;

#[derive(Deref, DerefMut)]
pub struct Oct(Tree<f32, usize>);

impl Oct {
  pub fn new() -> Self{
    Oct(Tree::new(Aabb3::new(Point3::new(-1024f32,-1024f32,-Z_MAX), Point3::new(3072f32,3072f32,Z_MAX)), 0, 0, 0, 0))
  }
  pub fn mem_size(&self) -> usize {
    self.0.mem_size()
  }
  // 添加一个aabb及其绑定
  pub fn add(&mut self, id: usize, aabb: Aabb3, bind: usize, notify: Option<NotifyImpl>) {
    self.0.add(id, aabb, bind);
    match notify {
      Some(n) => n.create_event(id),
      _ =>()
    }
  }

  // 更新指定id的aabb
  pub fn update(&mut self, id: usize, aabb: Aabb3, notify: Option<NotifyImpl>) -> bool {
    let r = self.0.update(id, aabb);
    if r {
      match notify {
        Some(n) => n.modify_event(id, "", 0),
        _ =>()
      }
    }
    r
  }
  // 移动指定id的aabb，性能比update要略好
  pub fn shift(&mut self, id: usize, distance: Vector3, notify: Option<NotifyImpl>) -> bool {
    let r = self.0.shift(id, distance);
    if r {
      match notify {
        Some(n) => n.modify_event(id, "", 0),
        _ =>()
      }
    }
    r
  }

  // 移除指定id的aabb及其绑定
  pub fn remove(&mut self, id: usize, notify: Option<NotifyImpl>) -> Option<(Aabb3, usize)> {
    let r = self.0.remove(id);
    if r != None {
      match notify {
        Some(n) => n.delete_event(id),
        _ =>()
      }
    }
    r
  }
}
