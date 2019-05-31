// 文字布局及布局系统
// 文本节点的布局算法： 文本节点本身所对应的yoga节点总是一个0大小的节点。文本节点的父节点才是进行文本布局的节点，称为P节点。P节点如果没有设置布局，则默认用flex布局模拟文档流布局。会将文本拆成每个字（英文为单词）的yoga节点加入P节点上。这样可以支持图文混排。P节点如果有flex布局，则遵循该布局。
// 字节点，根据字符是否为单字决定是需要字符容器还是单字。
// 文字根据样式，会处理：缩进，是否合并空白符，是否自动换行，是否允许换行符。来设置相应的flex布局。 换行符采用高度为0, 宽度100%的yoga节点来模拟。

use std::{
  os::raw::{c_void},
  mem::replace,
  marker::PhantomData,
};

use atom::Atom;
use lib_util::VecIndex;
use hal_core::Context;
use ecs::{
  system::{Runner, MultiCaseListener},
  monitor::{CreateEvent, DeleteEvent, ModifyEvent},
  component::MultiCaseImpl,
  single::SingleCaseImpl,
};

use ROOT;
use entity::{Node};
use component::{
  user::*,
  calc::*,
};
use layout::{YGDirection, YGFlexDirection, YgNode,  YGAlign};
use font::font_sheet::{ get_line_height, SplitResult, split, FontSheet};



type Read<'a, C> = (&'a SingleCaseImpl<FontSheet<C>>, &'a MultiCaseImpl<Node, YgNode>, &'a MultiCaseImpl<Node, Text>, &'a MultiCaseImpl<Node, TextStyle>, &'a MultiCaseImpl<Node, Font>);
type Write<'a> = (&'a mut MultiCaseImpl<Node, CharBlock>, &'a mut MultiCaseImpl<Node, Layout>);

pub struct LayoutImpl<C: Context + 'static + Send + Sync> {
  dirty: Vec<usize>, 
  temp: Vec<usize>,
  write: usize,
  mark: PhantomData<C>,
}

impl<'a, C: Context + 'static + Send + Sync> LayoutImpl< C> {
  pub fn new() -> Self{
    LayoutImpl {
      dirty: Vec::new(), 
      temp: Vec::new(),
      write: 0,
      mark: PhantomData,
    }
  }
  fn set_dirty(&mut self, id: usize, write: &'a mut MultiCaseImpl<Node, CharBlock>) {
    match write.get_mut(id) {
      Some(node) => {
         if !node.dirty {
          node.dirty = true;
          self.dirty.push(id)
        }
      },
      _ => {
        write.insert(id, CharBlock{
          family: Atom::from(""),
          font_size: 0.0,
          line_height: 0.0,
          letter_spacing: 0.0,
          vertical_align: VerticalAlign::Middle,
          indent: 0.0,
          chars: Vec::new(),
          dirty: true,
          layout_dirty: false,
        });
        self.dirty.push(id);
      }
    }
  }
}
impl<'a, C: Context + 'static + Send + Sync> Runner<'a> for LayoutImpl< C> {
  type ReadData = Read<'a, C>;
  type WriteData = Write<'a>;

  fn setup(&mut self, _read: Self::ReadData, _write: Self::WriteData) {
  }
  fn run(&mut self, read: Self::ReadData, mut write: Self::WriteData) {
    debug_println!("LayoutImpl run, dirty len: {}", self.dirty.len());
    for id in self.dirty.iter() {
      if calc(*id, &read, &mut write) {
        self.temp.push(*id)
      }
    }
    self.dirty.clear();
    if self.temp.len() > 0 {
      let vec = replace(&mut self.temp, Vec::new());
      let temp = replace(&mut self.dirty, vec);
      replace(&mut self.temp, temp);
    }
    let (w, h) = {
      let layout = unsafe{ write.1.get_unchecked(ROOT)};
      (layout.width, layout.height)
    };
    self.write= &mut write as *mut Write<'a> as usize;
    //计算布局，如果布局更改， 调用回调来设置layout属性，及字符的位置
    unsafe{ read.1.get_unchecked(ROOT)}.calculate_layout_by_callback(w, h, YGDirection::YGDirectionLTR, callback::<C>, self as *const LayoutImpl<C> as *const c_void);
  }
  fn dispose(&mut self, _read: Self::ReadData, _write: Self::WriteData) {
  }
}

// 监听text属性的改变
impl<'a, C: Context + 'static + Send + Sync> MultiCaseListener<'a, Node, Text, CreateEvent> for LayoutImpl< C> {
  type ReadData = ();
  type WriteData = &'a mut MultiCaseImpl<Node, CharBlock>;

  fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
    println!("create Text------------------------------");
    self.set_dirty(event.id, write)
  }
}
// 监听text属性的改变
impl<'a, C: Context + 'static + Send + Sync> MultiCaseListener<'a, Node, Text, ModifyEvent> for LayoutImpl< C> {
  type ReadData = ();
  type WriteData = &'a mut MultiCaseImpl<Node, CharBlock>;

  fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
    self.set_dirty(event.id, write)
  }
}
// 监听TextStyle属性的改变
impl<'a, C: Context + 'static + Send + Sync> MultiCaseListener<'a, Node, TextStyle, CreateEvent> for LayoutImpl< C> {
  type ReadData = ();
  type WriteData = &'a mut MultiCaseImpl<Node, CharBlock>;

  fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
    self.set_dirty(event.id, write)
  }
}
// 监听TextStyle属性的改变
impl<'a, C: Context + 'static + Send + Sync> MultiCaseListener<'a, Node, TextStyle, ModifyEvent> for LayoutImpl< C> {
  type ReadData = ();
  type WriteData = &'a mut MultiCaseImpl<Node, CharBlock>;

  fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
    self.set_dirty(event.id, write)
  }
}
// 监听Font属性的改变
impl<'a, C: Context + 'static + Send + Sync> MultiCaseListener<'a, Node, Font, CreateEvent> for LayoutImpl< C> {
  type ReadData = ();
  type WriteData = &'a mut MultiCaseImpl<Node, CharBlock>;

  fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
    self.set_dirty(event.id, write)
  }
}
// 监听Font属性的改变
impl<'a, C: Context + 'static + Send + Sync> MultiCaseListener<'a, Node, Font, ModifyEvent> for LayoutImpl< C> {
  type ReadData = ();
  type WriteData = &'a mut MultiCaseImpl<Node, CharBlock>;

  fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
    self.set_dirty(event.id, write)
  }
}

// 监听CharBlock的删除
impl<'a, C: Context + 'static + Send + Sync> MultiCaseListener<'a, Node, CharBlock, DeleteEvent> for LayoutImpl< C> {
  type ReadData = ();
  type WriteData = &'a mut MultiCaseImpl<Node, CharBlock>;

  fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, write: Self::WriteData) {
    // 删除脏列表中的CharBlock
    self.dirty.swap_delete(&event.id);
    let cb = unsafe{ write.get_unchecked(event.id)};
    // 删除所有yoga节点
    for cn in cb.chars.iter() {
        cn.node.free();
    }
  }
}

//================================ 内部静态方法
//回调函数
extern "C" fn callback<C: Context + 'static + Send + Sync>(node: YgNode, callback_args: *const c_void) {
  //更新布局
  let b = node.get_bind() as usize;
  let c = node.get_context() as usize;
  let layout_impl = unsafe{ &mut *(callback_args as usize as *mut LayoutImpl<C>) };
  let write = unsafe{ &mut *(layout_impl.write as *mut Write) };
  debug_println!("callback------------------------------{} {}", c, b);
  if b == 0 {
    // 节点布局更新
    write.1.insert(c, node.get_layout());
  }else {
    update(node, c, b - 1, write);
  }
}

// 文字布局更新
fn update<'a>(mut node: YgNode, id: usize, char_index: usize, write: &mut Write) {
  let layout = node.get_layout();
  let mut pos = Point2{x: layout.left, y: layout.top};
  node = node.get_parent();
  let node_id = node.get_context() as usize;
  if node_id != id {
    let layout = node.get_layout();
    pos.x += layout.left;
    pos.y += layout.top;
  }
  let mut cb = unsafe {write.0.get_unchecked_mut(id)};
  let mut cn = unsafe {cb.chars.get_unchecked_mut(char_index)};
  cn.pos = pos;
  if !cb.layout_dirty {
    cb.layout_dirty = true;
    unsafe { write.0.get_unchecked_write(id).modify(|_|{
      return true;
    }) };
  }
}
// 计算节点的YgNode的布局参数， 返回是否保留在脏列表中
fn calc<'a, C: Context + 'static + Send + Sync>(id: usize, read: &Read<C>, write: &mut Write) -> bool {
  debug_println!("calc-----------------------------------");
  let cb = unsafe{ write.0.get_unchecked_mut(id)};
  let yoga = unsafe { read.1.get_unchecked(id).clone() };
  let parent_yoga = yoga.get_parent();
  if parent_yoga.is_null() {
    debug_println!("parent_yoga.is_null");
    return true
  }
  // 计算节点的yoga节点在父节点的yoga节点的位置
  let count = parent_yoga.get_child_count() as usize;
  let mut yg_index: usize = 0;
  while yg_index < count && parent_yoga.get_child(yg_index as u32) != yoga {
      yg_index+=1;
  }
  let font = match read.4.get(id) {
      Some(f) => f.clone(),
      _ => Font::default()
  };
  cb.family = font.family.clone();
  // 获得字体高度
  cb.font_size = read.0.get_size(&font.family, &font.size);
  if cb.font_size == 0.0 {
      debug_println!("font_size==0.0");
      return true
  }
  cb.dirty = false;
  let style = match read.3.get(id) {
      Some(f) => f.clone(),
      _ => TextStyle::default()
  };
  cb.line_height = get_line_height(cb.font_size, &style.line_height);
  let text = match read.2.get(id) {
      Some(t) => t.0.as_ref(),
      _ => "",
  };
  debug_println!("text----------------------------------{:?}", text);
  // 如果有缩进变化, 则设置本span节点, 宽度为缩进值
  if cb.indent != style.indent {
    yoga.set_width(style.indent);
    cb.indent = style.indent;
  }
  let mut index = 0;
  let mut word = YgNode::new_null();
  let mut word_index = 0;
  // 根据每个字符, 创建对应的yoga节点, 加入父容器或字容器中
  for cr in split(text, true, style.white_space.preserve_spaces()) {
    debug_println!("split text----------------------------------");
    match cr {
      SplitResult::Newline =>{
        update_char(id, cb, '\n', 0.0, read.0, &mut index, &parent_yoga, &mut yg_index);
        update_char(id, cb, '\t', cb.indent, read.0, &mut index, &parent_yoga, &mut yg_index);
      },
      SplitResult::Whitespace =>{
        // 设置成宽度为半高, 高度0
        update_char(id, cb, ' ', cb.font_size/2.0, read.0, &mut index, &parent_yoga, &mut yg_index);
      },
      SplitResult::Word(c) => {
        update_char(id, cb, c, 0.0, read.0, &mut index, &parent_yoga, &mut yg_index);
      },
      SplitResult::WordStart(c) => {
        word = update_char(id, cb, char::from(0), 0.0, read.0, &mut index, &parent_yoga, &mut yg_index);
       update_char(id, cb, c, 0.0, read.0, &mut index, &word, &mut word_index);
      },
      SplitResult::WordNext(c) =>{
       update_char(id, cb, c, 0.0, read.0, &mut index, &word, &mut word_index);
      },
      SplitResult::WordEnd =>{
          word = YgNode::new_null();
          word_index = 0;
      },
    }
  }
  //清除多余的CharNode
  if index < cb.chars.len() {
    for i in index..cb.chars.len() {
        cb.chars[i].node.free()
    }
    unsafe{cb.chars.set_len(index)};
  }
  false
}
// 更新字符，如果字符不同，则清空后重新插入
fn update_char<C: Context + 'static + Send + Sync>(id: usize, cb: &mut CharBlock, c: char, w: f32, font: &FontSheet<C>, index: &mut usize, parent: &YgNode, yg_index: &mut usize) -> YgNode {
  let i = *index;
  if i < cb.chars.len() {
    let cn = &cb.chars[i];
    if cn.ch == c {
      *index = i + 1;
      *yg_index += 1;
      return cn.node.clone()
    }
    // 字符不同，将当前的，和后面的节点都释放掉
    for j in i..cb.chars.len() {
       cb.chars[j].node.free()
    }
    unsafe {cb.chars.set_len(i)};
  }
  let node = set_node(cb, c, w, font, YgNode::new());
  let cn = CharNode {
    ch: c,
    width: w,
    pos: Point2::default(),
    node: node.clone(),
  };
  node.set_bind((i + 1) as *mut c_void);
  node.set_context(id as *mut c_void);
  parent.insert_child(node, *yg_index as u32);
  cb.chars.push(cn);
  *index = i + 1;
  *yg_index += 1;
  node
}
// 设置节点的宽高
fn set_node<C: Context + 'static + Send + Sync>(cb: &CharBlock, c: char, mut w: f32, font: &FontSheet<C>, node: YgNode) -> YgNode {
  if c > ' ' {
    w = font.measure(&cb.family, cb.font_size, c);
    node.set_width(w + cb.letter_spacing);
    node.set_height(cb.line_height);
    match cb.vertical_align {
        VerticalAlign::Middle => node.set_align_self(YGAlign::YGAlignCenter),
        VerticalAlign::Top => node.set_align_self(YGAlign::YGAlignFlexStart),
        VerticalAlign::Bottom => node.set_align_self(YGAlign::YGAlignFlexEnd),
    };
  }else if c == '\n' {
    node.set_width_percent(100.0);
  }else if c == char::from(0) {
    node.set_width_auto();
    node.set_height(cb.line_height);
    node.set_flex_direction(YGFlexDirection::YGFlexDirectionRow);
  }else{
    node.set_width(w);
  }
  node
}


impl_system!{
    LayoutImpl<C> where [C: 'static + Context + Send + Sync],
    true,
    {
      MultiCaseListener<Node, Text, CreateEvent>
      MultiCaseListener<Node, Text, ModifyEvent>
      MultiCaseListener<Node, TextStyle, CreateEvent>
      MultiCaseListener<Node, TextStyle, ModifyEvent>
      MultiCaseListener<Node, Font, CreateEvent>
      MultiCaseListener<Node, Font, ModifyEvent>
      MultiCaseListener<Node, CharBlock, DeleteEvent>
    }
}