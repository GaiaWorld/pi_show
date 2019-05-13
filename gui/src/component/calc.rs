/// 中间计算的组件


use std::{
  f32,
  cmp::{Ordering},
  marker::PhantomData,
};

use map::{vecmap::VecMap};
use heap::simple_heap::SimpleHeap;

use ecs::{
  system::{SingleCaseListener, EntityListener},
  monitor::{DeleteEvent},
  single::SingleCaseImpl,
  component::Component,
  idtree::IdTree,
  entity::EntityImpl,
  Share,
};

use entity::Node;

#[derive(Component, Default)]
pub struct ZDepth(pub f32);

//是否可见， 不可见时， 也会占据布局位置
#[derive(Deref, DerefMut, Default, Component)]
pub struct Visibility(bool);

//不透明度
#[derive(Deref, DerefMut, Default, Component)]
pub struct Opacity(f32);