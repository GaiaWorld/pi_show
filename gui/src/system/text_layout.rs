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
    Share as ShareTrait,
};

use ROOT;
use entity::{Node};
use component::{
    user::*,
    calc::*,
};
use layout::{YGDirection, YGFlexDirection, FlexNode, YGAlign, YGJustify, YGWrap, YGUnit};
use font::font_sheet::{ get_line_height, SplitResult, split, FontSheet};

type Read<'a, C, L> = (&'a SingleCaseImpl<FontSheet<C>>, &'a MultiCaseImpl<Node, L>, &'a MultiCaseImpl<Node, Text>, &'a MultiCaseImpl<Node, TextStyle>, &'a MultiCaseImpl<Node, Font>);
type Write<'a, L> = (&'a mut MultiCaseImpl<Node, CharBlock<L>>, &'a mut MultiCaseImpl<Node, Layout>);

pub struct LayoutImpl<C: Context + ShareTrait, L: FlexNode + ShareTrait> {
    dirty: Vec<usize>, 
    temp: Vec<usize>,
    read: usize,
    write: usize,
    mark: PhantomData<(C, L)>,
}

impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> LayoutImpl< C, L> {
    pub fn new() -> Self{
        LayoutImpl {
            dirty: Vec::new(), 
            temp: Vec::new(),
            read: 0,
            write: 0,
            mark: PhantomData,
        }
    }
    fn set_dirty(&mut self, id: usize, write: &'a mut MultiCaseImpl<Node, CharBlock<L>>) {
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
                text_align: TextAlign::Center,
                vertical_align: VerticalAlign::Middle,
                indent: 0.0,
                preserve_spaces: false,
                chars: Vec::new(),
                pos: Point2::default(),
                line_count: 1,
                fix_width: true,
                dirty: true,
                });
                self.dirty.push(id);
            }
        }
    }
}
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> Runner<'a> for LayoutImpl< C, L> {
  type ReadData = Read<'a, C, L>;
  type WriteData = Write<'a, L>;

  fn run(&mut self, read: Self::ReadData, mut write: Self::WriteData) {;
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
        self.read= &read as *const Read<'a, C, L> as usize;
        self.write= &mut write as *mut Write<'a, L> as usize;
        //计算布局，如果布局更改， 调用回调来设置layout属性，及字符的位置
        unsafe{ read.1.get_unchecked(ROOT)}.calculate_layout_by_callback(w, h, YGDirection::YGDirectionLTR, callback::<C, L>, self as *const LayoutImpl<C, L> as *const c_void);
  }
}

// 监听text属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, Text, CreateEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, write)
    }
}
// 监听text属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, Text, ModifyEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, write)
    }
}
// 监听TextStyle属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, TextStyle, CreateEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, write)
    }
}
// 监听TextStyle属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, TextStyle, ModifyEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, write)
    }
}
// 监听Font属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, Font, CreateEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, write)
    }
}
// 监听Font属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, Font, ModifyEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, write)
    }
}

// 监听CharBlock的删除
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, CharBlock<L>, DeleteEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

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
extern "C" fn callback<C: Context + ShareTrait, L: FlexNode + ShareTrait>(node: L, callback_args: *const c_void) {
    //更新布局
    let b = node.get_bind() as usize;
    let id = node.get_context() as usize;
    let layout_impl = unsafe{ &mut *(callback_args as usize as *mut LayoutImpl<C, L>) };
    let write = unsafe{ &mut *(layout_impl.write as *mut Write<L>) };
    if b == 0 {   
        //如果是span节点， 不更新布局， 因为渲染对象使用了span的世界矩阵， 如果span布局不更新， 那么其世界矩阵与父节点的世界矩阵相等
        let layout = node.get_layout();
        if let Some(cb) = write.0.get_mut(id) {
            if cb.dirty {
                // 延后布局的计算
                cb.dirty = false;
                let read = unsafe{ &*(layout_impl.read as *const Read<C, L>) };
                let text = match read.2.get(id) {
                    Some(t) => t.0.as_ref(),
                    _ => "",
                };
                let node = node.get_parent();
                let layout = node.get_layout();
                cb.pos.x = layout.border_left + layout.padding_left;
                cb.pos.y = layout.border_top + layout.padding_top + (cb.line_height - cb.font_size)/2.0;
                let w = layout.width - cb.pos.x - layout.border_right - layout.padding_right;
                let h = layout.height - cb.pos.y - layout.border_bottom - layout.padding_bottom;
                let size = calc_text(cb, text, w, read.0);
                match cb.text_align {
                    TextAlign::Center => cb.pos.x += (w - size.x) / 2.0,
                    TextAlign::Right => cb.pos.x += w - size.x,
                    TextAlign::Justify if size.x > w => justify(cb, w, size.x),
                    _ => (),
                };
                match cb.vertical_align {
                    VerticalAlign::Middle => cb.pos.y = (h - size.y) / 2.0,
                    VerticalAlign::Bottom => cb.pos.y = h - size.y,
                    _ => (),
                };
            }
            return;
        }
        if &layout == unsafe {write.1.get_unchecked(id)} {
            return;
        }
        // 节点布局更新
        write.1.insert(id, layout);
    }else if id > 0 {
        update(node, id, b - 1, write);
    }
}

// 文字布局更新
fn update<'a, L: FlexNode + ShareTrait>(mut node: L, id: usize, char_index: usize, write: &mut Write<L>) {
    let layout = node.get_layout();
    let mut pos = Point2{x: layout.left, y: layout.top};
    node = node.get_parent();
    let node_id = node.get_context() as usize;
    if node_id == 0 {
        let layout = node.get_layout();
        pos.x += layout.left;
        pos.y += layout.top;
    }
    let cb = unsafe {write.0.get_unchecked_mut(id)};
    let mut cn = unsafe {cb.chars.get_unchecked_mut(char_index)};
    cn.pos = pos;
    unsafe { write.0.get_unchecked_write(id).modify(|_|{
        return true;
    }) };
            

    // if !cb.layout_dirty {
    //   cb.layout_dirty = true;
    //   unsafe { write.0.get_unchecked_write(id).modify(|_|{
    //     return true;
    //   }) };
    // }
}
// 计算节点的L的布局参数， 返回是否保留在脏列表中
fn calc<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait>(id: usize, read: &Read<C, L>, write: &mut Write<L>) -> bool {;
    let cb = unsafe{ write.0.get_unchecked_mut(id)};
    let yoga = unsafe { read.1.get_unchecked(id).clone() };
    let parent_yoga = yoga.get_parent();
    if parent_yoga.is_null() {
        #[cfg(feature = "warning")]
        println!("parent_yoga.is_null");
        return true
    }

    let font = match read.4.get(id) {
        Some(f) => f.clone(),
        _ => Font::default()
    };
    cb.family = font.family.clone();
    // 获得字体高度
    cb.font_size = read.0.get_size(&font.family, &font.size);
    if cb.font_size == 0.0 {
        #[cfg(feature = "warning")]
        println!("font_size==0.0");
        return true
    }
    let style = match read.3.get(id) {
        Some(f) => f.clone(),
        _ => TextStyle::default()
    };
    cb.line_height = (get_line_height(cb.font_size, &style.line_height) * 100.0).round()/100.0;
    
    cb.letter_spacing = style.letter_spacing;
    // 如果有缩进变化, 则设置本span节点, 宽度为缩进值
    if cb.indent != style.indent {
        yoga.set_width(style.indent);
        cb.indent = style.indent;
    }
    cb.preserve_spaces = style.white_space.preserve_spaces();
    match parent_yoga.get_style_justify() {
        YGJustify::YGJustifyCenter => cb.text_align = TextAlign::Center,
        YGJustify::YGJustifyFlexEnd => cb.text_align = TextAlign::Right,
        YGJustify::YGJustifySpaceBetween => cb.text_align = TextAlign::Justify,
        _ => (),
    }
    // 如果父节点只有1个子节点，则认为是Text节点. 如果没有设置宽度，则立即进行不换行的文字布局计算，并设置自身的大小为文字大小
    if parent_yoga.get_child_count() == 1 {
        match parent_yoga.get_style_width_unit() {
            YGUnit::YGUnitUndefined => (),
            YGUnit::YGUnitAuto => (),
            _ => return false
        }
        let text = match read.2.get(id) {
            Some(t) => t.0.as_ref(),
            _ => "",
        };
        let size = calc_text(cb, text, 65535.0, read.0);
        yoga.set_width(size.x);
        yoga.set_height(size.y);
        cb.pos.y = (cb.line_height - cb.font_size)/2.0;
        cb.dirty = false;
        return false
    }
    let text = match read.2.get(id) {
        Some(t) => t.0.as_ref(),
        _ => "",
    };
    cb.dirty = false;
    if style.white_space.allow_wrap() {
        parent_yoga.set_flex_wrap(YGWrap::YGWrapWrap);
    }else {
        parent_yoga.set_flex_wrap(YGWrap::YGWrapNoWrap);
    }
    // 计算节点的yoga节点在父节点的yoga节点的位置
    let count = parent_yoga.get_child_count() as usize;
    let mut yg_index: usize = 0;
    while yg_index < count && parent_yoga.get_child(yg_index as u32) != yoga {
        yg_index+=1;
    }
    yg_index += 1; // yg_index此时为span节点的位置， 应该+1， 保持span的yoga节点排在整个文字块的第一个位置
    let mut index = 0;
    let mut word = L::new_null();
    let mut word_index = 0;
    // 根据每个字符, 创建对应的yoga节点, 加入父容器或字容器中
    for cr in split(text, true, style.white_space.preserve_spaces()) {
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
                word = update_char(0, cb, char::from(0), 0.0, read.0, &mut index, &parent_yoga, &mut yg_index);
                update_char(id, cb, c, 0.0, read.0, &mut index, &word, &mut word_index);
            },
            SplitResult::WordNext(c) =>{
                update_char(id, cb, c, 0.0, read.0, &mut index, &word, &mut word_index);
            },
            SplitResult::WordEnd =>{
                    word = L::new_null();
                    word_index = 0;
            },
        }
    }
    //清除多余的CharNode
    if index < cb.chars.len() {
        for i in index..cb.chars.len() {
            // cb.chars[i].node.get_parent().remove_child(cb.chars[i].node.clone());
            cb.chars[i].node.free(); // 调用remove_child方法是， node会被释放
        }
        unsafe{cb.chars.set_len(index)};
        
    }
    // unsafe { write.0.get_unchecked_write(id).modify(|_|{
    //     return true;
    // }) };
    // if !cb.layout_dirty {
    //   cb.layout_dirty = true;
    //   unsafe { write.0.get_unchecked_write(id).modify(|_|{
    //     return true;
    //   }) };
    // }
    false
}
// 更新字符，如果字符不同，则清空后重新插入
fn update_char<C: Context + ShareTrait, L: FlexNode + ShareTrait>(id: usize, cb: &mut CharBlock<L>, c: char, w: f32, font: &FontSheet<C>, index: &mut usize, parent: &L, yg_index: &mut usize) -> L {
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
    let node = set_node(cb, c, w, font, L::new());
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
fn set_node<C: Context + ShareTrait, L: FlexNode + ShareTrait>(cb: &CharBlock<L>, c: char, mut w: f32, font: &FontSheet<C>, node: L) -> L {
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
    }else{ // "\t"
        node.set_width(w);
    }
    node
}

#[derive(Debug)]
struct Calc {
    index: usize,
    pos: Point2,
    max_w: f32,
    word: usize,
}

fn calc_text<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait>(cb: &mut CharBlock<L>, text: &'a str, limit: f32, font: &FontSheet<C>) -> Vector2 {
    let mut calc = Calc{
        index: 0,
        pos: Point2::default(),
        max_w: 0.0,
        word: 0,
    };
    // 根据每个字符, 创建对应的yoga节点, 加入父容器或字容器中
    for cr in split(text, true, cb.preserve_spaces) {
        match cr {
            SplitResult::Newline =>{
                update_char1(cb, '\n', 0.0, limit, font, &mut calc);
                update_char1(cb, '\t', cb.indent, limit, font, &mut calc);
            },
            SplitResult::Whitespace =>{
                // 设置成宽度为字高, 高度0
                update_char1(cb, ' ', cb.font_size, limit, font, &mut calc);
            },
            SplitResult::Word(c) => {
                update_char1(cb, c, 0.0, limit, font, &mut calc);
            },
            SplitResult::WordStart(c) => {
                calc.word = calc.index;
                update_char1(cb, c, 0.0, limit, font, &mut calc);
            },
            SplitResult::WordNext(c) =>{
                update_char1(cb, c, 0.0, limit, font, &mut calc);
            },
            SplitResult::WordEnd =>{
                calc.word = 0;
            },
        }
    }
    //清除多余的CharNode
    if calc.index < cb.chars.len() {
        unsafe{cb.chars.set_len(calc.index)};
    }
    if calc.pos.x == 0.0 {
        cb.line_count -= 1;
    }
    Vector2::new(calc.max_w, calc.pos.y + cb.line_height)
}
// 更新字符，如果字符不同，则清空后重新插入
fn update_char1<C: Context + ShareTrait, L: FlexNode + ShareTrait>(cb: &mut CharBlock<L>, c: char, w: f32, limit: f32, font: &FontSheet<C>,  calc: &mut Calc) {
    if  calc.index < cb.chars.len() {
        let cn = &cb.chars[calc.index];
        if cn.ch == c {
            calc.index += 1;
            return
        }
        // 字符不同，将当前的，和后面的节点都释放掉
        unsafe {cb.chars.set_len(calc.index)};
    }
    let mut cn = CharNode {
        ch: c,
        width: w,
        pos: Point2::default(),
        node: L::new_null(),
    };
    set_node1(cb, c, w, limit, font, calc, &mut cn);
    cb.chars.push(cn);
    calc.index += 1;
}
// 设置节点的宽高
fn set_node1<C: Context + ShareTrait, L: FlexNode + ShareTrait>(cb: &mut CharBlock<L>, c: char, mut w: f32, limit: f32, font: &FontSheet<C>,  calc: &mut Calc, node: &mut CharNode<L>) {
    if c > ' ' {
        w = font.measure(&cb.family, cb.font_size, c);
        if w != cb.font_size && cb.fix_width {
            cb.fix_width = false
        }
        if calc.pos.x > 0.0 && limit < w + calc.pos.x {
            // 需要换行
            calc.pos.y += cb.line_height;
            calc.pos.x = 0.0;
            cb.line_count += 1;
            if calc.word > 0 {
                // 整个单词进行换行
                for i in calc.word .. calc.index {
                    cb.chars[i].pos = calc.pos;
                    calc.pos.x += cb.chars[i].width + cb.letter_spacing;
                }
            }
        } 
        node.pos=calc.pos;
        calc.pos.x += w;
        if calc.max_w < calc.pos.x {
            calc.max_w = calc.pos.x
        }
        calc.pos.x += cb.letter_spacing;
    }else if c == '\n' {
        calc.pos.x = cb.indent;
        calc.pos.y += cb.line_height;
        cb.line_count += 1;
    }else if c == char::from(0) {
    }else{ // '\t' ' '
        calc.pos.x += w;
        if calc.max_w < calc.pos.x {
            calc.max_w = calc.pos.x
        }
    }
}
// 填充宽度
fn justify<L: FlexNode + ShareTrait>(cb: &mut CharBlock<L>, width: f32, box_w: f32) {
    let len = cb.chars.len();
    if len < 2 {
        return;
    }
    cb.fix_width = false;
    // 简单处理， 以后应该支持精确的按行填充和按列填充, 按行填充还需按单词进行处理 TODO
    let fix = (width - box_w) / (len - 1) as f32;
    for i in 1..len {
        cb.chars[i].pos.x += fix;
    }
}

impl_system!{
    LayoutImpl<C, L> where [C: Context + ShareTrait, L: FlexNode + ShareTrait],
    true,
    {
        MultiCaseListener<Node, Text, CreateEvent>
        MultiCaseListener<Node, Text, ModifyEvent>
        MultiCaseListener<Node, TextStyle, CreateEvent>
        MultiCaseListener<Node, TextStyle, ModifyEvent>
        MultiCaseListener<Node, Font, CreateEvent>
        MultiCaseListener<Node, Font, ModifyEvent>
        MultiCaseListener<Node, CharBlock<L>, DeleteEvent>
    }
}