// 文字布局及布局系统
// 文本节点的布局算法： 文本节点本身所对应的yoga节点总是一个0大小的节点。文本节点的父节点才是进行文本布局的节点，称为P节点。P节点如果没有设置布局，则默认用flex布局模拟文档流布局。会将文本拆成每个字（英文为单词）的yoga节点加入P节点上。这样可以支持图文混排。P节点如果有flex布局，则遵循该布局。
// 字节点，根据字符是否为单字决定是需要字符容器还是单字。
// 文字根据样式，会处理：缩进，是否合并空白符，是否自动换行，是否允许换行符。来设置相应的flex布局。 换行符采用高度为0, 宽度100%的yoga节点来模拟。

use std::{
    os::raw::{c_void},
    mem::replace,
    marker::PhantomData,
};

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
use single::{TextStyleClassMap};
use layout::{YGDirection, YGFlexDirection, FlexNode, YGAlign, YGJustify, YGWrap, YGUnit};
use font::font_sheet::{ get_line_height, SplitResult, split, FontSheet};

type Read<'a, C, L> = (
    &'a SingleCaseImpl<FontSheet<C>>,
    &'a SingleCaseImpl<TextStyleClassMap>,
    &'a MultiCaseImpl<Node, L>,
    &'a MultiCaseImpl<Node, Text>,
    &'a MultiCaseImpl<Node, Font>,
    &'a MultiCaseImpl<Node, TextStyle>,
    &'a MultiCaseImpl<Node, TextShadow>,
    &'a MultiCaseImpl<Node, TextStyleClass>);
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
    fn set_dirty(&mut self, id: usize, dirty: usize, write: &'a mut MultiCaseImpl<Node, CharBlock<L>>) {
        match write.get_mut(id) {
        Some(node) => {
            if node.dirty == 0 {
                self.dirty.push(id);
                node.dirty = dirty;
            }else if node.dirty & dirty == 0  {
                node.dirty |= dirty;
            }
        },
        _ => {
            write.insert(id, CharBlock{
                clazz: TextStyleClazz::default(),
                font_size: 0.0,
                line_height: 0.0,
                chars: Vec::new(),
                lines: Vec::new(),
                last_line: (0, 0.0),
                size: Vector2::default(),
                wrap_size: Vector2::default(),
                pos: Point2::default(),
                line_count: 1,
                fix_width: true,
                dirty: dirty,
                local_style: false,
                style_class: 0,
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
        unsafe{ read.2.get_unchecked(ROOT)}.calculate_layout_by_callback(w, h, YGDirection::YGDirectionLTR, callback::<C, L>, self as *const LayoutImpl<C, L> as *const c_void);
  }
}

// 监听text属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, Text, CreateEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, DirtyType::Text as usize, write)
    }
}
// 监听text属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, Text, ModifyEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, DirtyType::Text as usize, write)
    }
}
// 监听TextStyleClass属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, TextStyleClass, CreateEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, DirtyType::StyleClass as usize, write)
    }
}
// 监听TextStyleClass属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, TextStyleClass, ModifyEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, DirtyType::StyleClass as usize, write)
    }
}
// 监听TextStyle属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, TextStyle, CreateEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, DirtyType::LocalStyle as usize, write)
    }
}
// 监听TextStyle属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, TextStyle, ModifyEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, DirtyType::LocalStyle as usize, write)
    }
}
// 监听Font属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, Font, CreateEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, DirtyType::LocalStyle as usize, write)
    }
}
// 监听Font属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, Font, ModifyEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, DirtyType::LocalStyle as usize, write)
    }
}
// 监听TextStyle属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, TextShadow, CreateEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, DirtyType::LocalStyle as usize, write)
    }
}
// 监听TextStyle属性的改变
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, TextShadow, ModifyEvent> for LayoutImpl< C, L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, DirtyType::LocalStyle as usize, write)
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
            // 延后布局的计算， 根据是否居中靠右或填充，或者换行，进行文字重布局
            let node = node.get_parent();
            match node.get_style_width_unit() {
                YGUnit::YGUnitPoint => return,
                _ => ()
            }
            calc_wrap_align(cb, &node.get_layout());
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

}
// 计算节点的L的布局参数， 返回是否保留在脏列表中
fn calc<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait>(id: usize, read: &Read<C, L>, write: &mut Write<L>) -> bool {;
    let cb = unsafe{ write.0.get_unchecked_mut(id)};
    let yoga = unsafe { read.2.get_unchecked(id).clone() };
    let parent_yoga = yoga.get_parent();
    if parent_yoga.is_null() {
        #[cfg(feature = "warning")]
        println!("parent_yoga.is_null");
        return true
    }
    if cb.dirty & DirtyType::StyleClass as usize != 0 {
        cb.style_class = read.7.get(id).unwrap().0;
        match (read.1).0.get(&cb.style_class) {
            Some(c) => cb.clazz = c.clone(),
            _ => (),
        };
    }
    if cb.dirty & DirtyType::LocalStyle as usize != 0 || cb.local_style {
        cb.local_style = true;
        // TODO 改成局部拷贝
        match read.4.get(id) {
            Some(r) => cb.clazz.font = r.clone(),
            _ => ()
        };
        match read.5.get(id) {
            Some(r) => cb.clazz.style = r.clone(),
            _ => ()
        };
        match read.6.get(id) {
            Some(r) => cb.clazz.shadow = r.clone(),
            _ => ()
        };
    }
    // 获得字体高度
    cb.font_size = read.0.get_size(&cb.clazz.font.family, &cb.clazz.font.size);
    if cb.font_size == 0.0 {
        #[cfg(feature = "warning")]
        println!("font_size==0.0");
        return true
    }
    cb.line_height = (get_line_height(cb.font_size, &cb.clazz.style.line_height) * 100.0).round()/100.0;
    cb.pos.y = (cb.line_height - cb.font_size)/2.0;
    match parent_yoga.get_style_justify() {
        YGJustify::YGJustifyCenter => cb.clazz.style.text_align = TextAlign::Center,
        YGJustify::YGJustifyFlexEnd => cb.clazz.style.text_align = TextAlign::Right,
        YGJustify::YGJustifySpaceBetween => cb.clazz.style.text_align = TextAlign::Justify,
        _ => (),
    }
    cb.dirty = 0;
    let count = parent_yoga.get_child_count() as usize;
    let text = match read.3.get(id) {
        Some(t) => t.0.as_ref(),
        _ => "",
    };
    // 如果父节点只有1个子节点，则认为是Text节点. 如果没有设置宽度，则立即进行不换行的文字布局计算，并设置自身的大小为文字大小
    if count == 1 {
        let old = yoga.get_layout();
        calc_text(cb, text, read.0);
        match parent_yoga.get_style_width_unit() {
            YGUnit::YGUnitUndefined => (),
            YGUnit::YGUnitAuto => (),
            YGUnit::YGUnitPoint => {
                //calc_wrap_align(cb, &parent_yoga.get_layout())
            },
            _ => ()
        }
        if old.width != cb.wrap_size.x || old.height != cb.wrap_size.y {
            yoga.set_width(cb.wrap_size.x);
            yoga.set_height(cb.wrap_size.y);
        }else{
            unsafe { write.0.get_unchecked_write(id).modify(|_|{
                return true;
            }) };
        }
        return false
    }
    if cb.clazz.style.white_space.allow_wrap() {
        parent_yoga.set_flex_wrap(YGWrap::YGWrapWrap);
    }else {
        parent_yoga.set_flex_wrap(YGWrap::YGWrapNoWrap);
    }
    // 如果有缩进变化, 则设置本span节点, 宽度为缩进值
    yoga.set_width(cb.clazz.style.indent);
    
    // 计算节点的yoga节点在父节点的yoga节点的位置
    let mut yg_index: usize = 0;
    while yg_index < count && parent_yoga.get_child(yg_index as u32) != yoga {
        yg_index+=1;
    }
    yg_index += 1; // yg_index此时为span节点的位置， 应该+1， 保持span的yoga节点排在整个文字块的第一个位置
    let mut index = 0;
    let mut word = L::new_null();
    let mut word_index = 0;
    // 根据每个字符, 创建对应的yoga节点, 加入父容器或字容器中
    for cr in split(text, true, cb.clazz.style.white_space.preserve_spaces()) {
        match cr {
            SplitResult::Newline =>{
                update_char(id, cb, '\n', 0.0, read.0, &mut index, &parent_yoga, &mut yg_index);
                update_char(id, cb, '\t', cb.clazz.style.indent, read.0, &mut index, &parent_yoga, &mut yg_index);
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
            cb.chars[i].node.free(); // 调用remove_child方法是， node会被释放
        }
        unsafe{cb.chars.set_len(index)};
    }
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
        w = font.measure(&cb.clazz.font.family, cb.font_size, c);
        node.set_width(w + cb.clazz.style.letter_spacing);
        node.set_height(cb.line_height);
        match cb.clazz.style.vertical_align {
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

fn calc_text<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait>(cb: &mut CharBlock<L>, text: &'a str, font: &FontSheet<C>) {
    let mut calc = Calc{
        index: 0,
        pos: Point2::default(),
        max_w: 0.0,
        word: 0,
    };
    // 根据每个字符, 创建对应的yoga节点, 加入父容器或字容器中
    for cr in split(text, true, cb.clazz.style.white_space.preserve_spaces()) {
        match cr {
            SplitResult::Newline =>{
                cb.last_line.1 = calc.pos.x;
                cb.lines.push(cb.last_line.clone());
                cb.last_line = (0, 0.0);
                cb.line_count += 1;
                update_char1(cb, '\n', 0.0, font, &mut calc);
                // 行首缩进
                calc.pos.x += cb.clazz.style.indent;
            },
            SplitResult::Whitespace =>{
                // 设置成宽度为默认字宽, 高度0
                update_char1(cb, ' ', cb.font_size, font, &mut calc);
                calc.pos.x += cb.clazz.style.word_spacing;
                cb.last_line.0 += 1;
            },
            SplitResult::Word(c) => {
                update_char1(cb, c, 0.0, font, &mut calc);
                calc.pos.x += cb.clazz.style.word_spacing;
                cb.last_line.0 += 1;
            },
            SplitResult::WordStart(c) => {
                calc.word = calc.index;
                update_char1(cb, 0 as char, 0.0, font, &mut calc);
                update_char1(cb, c, 0.0, font, &mut calc);
            },
            SplitResult::WordNext(c) =>{
                calc.pos.x += cb.clazz.style.letter_spacing;
                update_char1(cb, c, 0.0, font, &mut calc);
            },
            SplitResult::WordEnd =>{
                let node = unsafe {cb.chars.get_unchecked_mut(calc.word)};
                node.width = calc.pos.x - node.pos.x;
                node.pos.y = (calc.index - calc.word) as f32;
                calc.word = 0;
                calc.pos.x += cb.clazz.style.word_spacing;
                cb.last_line.0 += 1;
            },
        }
    }
    //清除多余的CharNode
    if calc.index < cb.chars.len() {
        for i in calc.index..cb.chars.len() {
            cb.chars[i].node.free(); // 调用remove_child方法是， node会被释放
        }
        unsafe{cb.chars.set_len(calc.index)};
    }
    cb.size.x = calc.max_w;
    cb.size.y = calc.pos.y + cb.line_height;
    cb.wrap_size = cb.size;
}
// 更新字符，如果字符不同，则清空后重新插入
fn update_char1<C: Context + ShareTrait, L: FlexNode + ShareTrait>(cb: &mut CharBlock<L>, c: char, w: f32, font: &FontSheet<C>,  calc: &mut Calc) {
    if  calc.index < cb.chars.len() {
        let cn = &cb.chars[calc.index];
        if cn.ch == c {
            calc.index += 1;
            return
        }
        // 字符不同，将当前的，和后面的节点都释放掉
        for j in calc.index..cb.chars.len() {
            cb.chars[j].node.free()
        }
        unsafe {cb.chars.set_len(calc.index)};
    }

    cb.chars.push(CharNode {
        ch: c,
        width: w,
        pos: calc.pos,
        node: L::new_null(),
    });
    set_node1(cb, c, w, font, calc);
    calc.index += 1;
}
// 设置节点的宽高
fn set_node1<C: Context + ShareTrait, L: FlexNode + ShareTrait>(cb: &mut CharBlock<L>, c: char, mut w: f32, font: &FontSheet<C>,  calc: &mut Calc) {
    if c > ' ' {
        w = font.measure(&cb.clazz.font.family, cb.font_size, c);
        if w != cb.font_size && cb.fix_width {
            cb.fix_width = false
        }
        calc.pos.x += w;
        if calc.max_w < calc.pos.x {
            calc.max_w = calc.pos.x
        }
    }else if c == '\n' {
        calc.pos.x = 0.0;
        calc.pos.y += cb.line_height;
    }else if c == char::from(0) {

    }else if c == ' ' {
        calc.pos.x += w;
        if calc.max_w < calc.pos.x {
            calc.max_w = calc.pos.x
        }
    }
}
/// 计算换行和对齐， 如果是单行或多行左对齐，可以直接改cb.pos
fn calc_wrap_align<L: FlexNode + ShareTrait>(cb: &mut CharBlock<L>, layout: &Layout) {
    let x = layout.border_left + layout.padding_left;
    let w = layout.width - x - layout.border_right - layout.padding_right;
    if cb.clazz.style.white_space.allow_wrap() || cb.size.x < w {
        return;
    }
    let y = layout.border_top + layout.padding_top;
    let h = layout.height - cb.pos.y - layout.border_bottom - layout.padding_bottom;
    if cb.line_count == 1 {
        match cb.clazz.style.text_align {
            TextAlign::Center => cb.pos.x += (w - cb.size.x) / 2.0,
            TextAlign::Right => cb.pos.x += w - cb.size.x,
            TextAlign::Justify if cb.size.x > w => justify(cb, w, cb.size.x),
            _ => (),
        };
        cb.pos.y += (cb.line_height - cb.font_size)/2.0;
        if h > 0.0 {
            match cb.clazz.style.vertical_align {
                VerticalAlign::Middle => cb.pos.y += (h - cb.size.y) / 2.0,
                VerticalAlign::Bottom => cb.pos.y += h - cb.size.y,
                _ => (),
            }
        }
    }

}

// if calc.pos.x > 0.0 && limit < w + calc.pos.x {
//             // 需要换行
//             calc.pos.y += cb.line_height;
//             calc.pos.x = 0.0;
//             cb.line_count += 1;
//             if calc.word > 0 {
//                 // 整个单词进行换行
//                 for i in calc.word .. calc.index {
//                     cb.chars[i].pos = calc.pos;
//                     calc.pos.x += cb.chars[i].width + cb.letter_spacing;
//                 }
//             }
//         } 
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