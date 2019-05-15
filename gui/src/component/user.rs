/// 用户操作的组件


use std::{
  f32,
  cmp::{Ordering},
  marker::PhantomData,
  default::Default,
  mem::transmute,
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
pub struct ZIndex(pub isize);

//不透明度
#[derive(Deref, DerefMut, Component, Debug)]
pub struct Opacity(f32);

impl Default for Opacity {
  fn default() -> Opacity{
    Opacity(1.0)
  }
}

enum ShowType{
  Display = 1, // 0表示 Flex
  Visibility = 2, // 0表示no Visible
  Enable = 4, // 0表示no Enable
}

#[derive(Clone, Copy, Debug)]
pub enum Display{
  Flex,
  None,
}

#[derive(Deref, DerefMut, Component, Debug)]
pub struct Show(usize);

impl Show {
  #[inline]
  pub fn get_display(&self) -> Display {
    unsafe { transmute((self.0 & 1) as u8) }
  }

  #[inline]
  pub fn set_display(&mut self, display: Display){
    match display {
      Display::Flex => self.0 &= !(ShowType::Display as usize),
      Display::None => self.0 |= ShowType::Display as usize,
    }
  }

  #[inline]
  pub fn get_visibility(&self) -> bool{
    (self.0 & 2) == 2
  }

  #[inline]
  pub fn set_visibility(&mut self, visibility: bool){
    if visibility {
      self.0 |= ShowType::Visibility as usize;
    }else{
      self.0 &= !(ShowType::Visibility as usize);
    }
  }

  #[inline]
  pub fn get_enable(&self) -> bool{
    (self.0 & 4) == 4
  }

  #[inline]
  pub fn set_enable(&mut self, enable: bool){
    if enable {
      self.0 |= ShowType::Enable as usize;
    }else{
      self.0 &= !(ShowType::Enable as usize);
    }
  }
}

impl Default for Show {
  fn default() -> Show {
    Show(6)
  }
}