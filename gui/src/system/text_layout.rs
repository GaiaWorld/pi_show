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
use ecs::{
    system::{Runner, MultiCaseListener},
    monitor::{CreateEvent, DeleteEvent, ModifyEvent},
    component::MultiCaseImpl,
    single::SingleCaseImpl,
    
};

use ROOT;
use entity::*;
use single::*;
use component::{
    user::*,
    calc::*,
};

use layout::{YGDirection, YGFlexDirection, FlexNode, YGAlign, YGJustify, YGWrap, YGUnit};
use font::font_sheet::{ get_line_height, SplitResult, split, FontSheet};

type Read<'a, L> = (
    &'a SingleCaseImpl<FontSheet>,
    &'a MultiCaseImpl<Node, L>,
    &'a MultiCaseImpl<Node, Text>,
    &'a MultiCaseImpl<Node, Font>,
    &'a MultiCaseImpl<Node, TextStyle>,
    &'a MultiCaseImpl<Node, TextShadow>,
    &'a MultiCaseImpl<Node, ClassName>,
    &'a SingleCaseImpl<ClassSheet>,
);
type Write<'a, L> = (&'a mut MultiCaseImpl<Node, CharBlock<L>>, &'a mut MultiCaseImpl<Node, Layout>);

pub struct LayoutImpl<L: FlexNode> {
    dirty: Vec<usize>, 
    temp: Vec<usize>,
    read: usize,
    write: usize,
    mark: PhantomData<(L)>,
}

impl<'a, L: FlexNode> LayoutImpl< L> {
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
                clazz: TextClass::default(),
                font_size: 16.0,
                line_height: 20.0,
                chars: Vec::new(),
                lines: Vec::new(),
                last_line: (0, 0, 0.0),
                size: Vector2::default(),
                wrap_size: Vector2::default(),
                pos: Point2::default(),
                line_count: 1,
                fix_width: true,
                dirty: dirty,
                local_style: 0,
                style_class: 0,
                modify: 0,
                });
                self.dirty.push(id);
            }
        }
    }
}
impl<'a, L: FlexNode> Runner<'a> for LayoutImpl< L> {
  type ReadData = Read<'a, L>;
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
        self.read= &read as *const Read<'a, L> as usize;
        self.write= &mut write as *mut Write<'a, L> as usize;
        //计算布局，如果布局更改， 调用回调来设置layout属性，及字符的位置
        unsafe{ read.1.get_unchecked(ROOT)}.calculate_layout_by_callback(w, h, YGDirection::YGDirectionLTR, callback::<L>, self as *const LayoutImpl<L> as *const c_void);
  }
}

// 监听text属性的改变
impl<'a, L: FlexNode> MultiCaseListener<'a, Node, Text, CreateEvent> for LayoutImpl< L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, DirtyType::Text as usize, write)
    }
}
// 监听text属性的改变
impl<'a, L: FlexNode> MultiCaseListener<'a, Node, Text, ModifyEvent> for LayoutImpl< L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, DirtyType::Text as usize, write)
    }
}
// 监听TextStyleClass属性的改变
impl<'a, L: FlexNode> MultiCaseListener<'a, Node, ClassName, CreateEvent> for LayoutImpl< L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        self.set_dirty(event.id, DirtyType::StyleClass as usize, write)
    }
}
// // 监听TextStyleClass属性的改变
// impl<'a, L: FlexNode> MultiCaseListener<'a, Node, TextStyleClass, ModifyEvent> for LayoutImpl< L> {
//     type ReadData = ();
//     type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

//     fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
//         self.set_dirty(event.id, DirtyType::StyleClass as usize, write)
//     }
// }

// 监听TextStyle属性的改变
impl<'a, L: FlexNode> MultiCaseListener<'a, Node, TextStyle, ModifyEvent> for LayoutImpl< L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let r = match event.field {
            "letter_spacing" => DirtyType::LetterSpacing,
            "word_spacing" => DirtyType::WordSpacing,
            "line_height" => DirtyType::LineHeight,
            "indent" => DirtyType::Indent,
            "color" => DirtyType::Color,
            "stroke" => DirtyType::Stroke,
            "text_align" => DirtyType::TextAlign,
            "vertical_align" => DirtyType::VerticalAlign,
            _ => return
        };
        self.set_dirty(event.id, r as usize, write)
    }
}

// 监听Font属性的改变
impl<'a, L: FlexNode> MultiCaseListener<'a, Node, Font, ModifyEvent> for LayoutImpl< L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let r = match event.field {
            "style" => DirtyType::FontStyle,
            "weight" => DirtyType::FontWeight,
            "size" => DirtyType::FontSize,
            "family" => DirtyType::FontFamily,
            _ => return
        };
        self.set_dirty(event.id, r as usize, write)
    }
}

// 监听TextShadow属性的改变
impl<'a, L: FlexNode> MultiCaseListener<'a, Node, TextShadow, ModifyEvent> for LayoutImpl< L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let r = match event.field {
            "h" => DirtyType::ShadowHV as usize,
            "v" => DirtyType::ShadowHV as usize,
            "color" => DirtyType::ShadowColor as usize,
            "blur" => DirtyType::ShadowBlur as usize,
            _ => DirtyType::ShadowHV as usize + DirtyType::ShadowColor as usize + DirtyType::ShadowBlur as usize,
        };
        self.set_dirty(event.id, r, write)
    }
}
// 监听CharBlock的删除
impl<'a, L: FlexNode> MultiCaseListener<'a, Node, CharBlock<L>, DeleteEvent> for LayoutImpl< L> {
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
extern "C" fn callback<L: FlexNode>(node: L, callback_args: *const c_void) {
    //更新布局
    let b = node.get_bind() as usize;
    let id = node.get_context() as usize;
    let layout_impl = unsafe{ &mut *(callback_args as usize as *mut LayoutImpl<L>) };
    let write = unsafe{ &mut *(layout_impl.write as *mut Write<L>) };
    if b == 0 {  
        //如果是span节点， 不更新布局， 因为渲染对象使用了span的世界矩阵， 如果span布局不更新， 那么其世界矩阵与父节点的世界矩阵相等
        if let Some(cb) = write.0.get_mut(id) {
            // 只有百分比大小的需要延后布局的计算， 根据是否居中靠右或填充，或者换行，进行文字重布局
            let node = node.get_parent();
            if node.get_child_count() == 1 {
                match node.get_style_width_unit() {
                    YGUnit::YGUnitPercent | YGUnit::YGUnitPoint => {
                        calc_wrap_align(cb, &node.get_layout());
                    },
                    _ => ()
                }
            }
            unsafe { write.0.get_unchecked_write(id).modify(|_|{
                return true;
            }) };
            return;
        }
        let layout = node.get_layout();
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
fn update<'a, L: FlexNode>(mut node: L, id: usize, char_index: usize, write: &mut Write<L>) {
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
}
// 计算节点的L的布局参数， 返回是否保留在脏列表中
fn calc<'a, L: FlexNode>(id: usize, read: &Read<L>, write: &mut Write<L>) -> bool {;
    let cb = unsafe{ write.0.get_unchecked_mut(id)};
    let yoga = unsafe { read.1.get_unchecked(id).clone() };
    let parent_yoga = yoga.get_parent();
    if parent_yoga.is_null() {
        #[cfg(feature = "warning")]
        println!("parent_yoga.is_null");
        return true
    }
    cb.style_class = unsafe { read.6.get_unchecked(id) }.0;
    if cb.dirty & DirtyType::StyleClass as usize != 0 {
        let text_class = if cb.style_class > 0 {
            match read.7.class.get(cb.style_class) {
                Some(class) => class.text,
                None => 0,
            }
        } else {
            0
        };
        if text_class > 0 {
            match read.7.text.get(cb.style_class) {
               Some(c) if cb.local_style == 0 => {
                    cb.clazz = c.clone();
                },
                Some(c) => {
                    if cb.local_style & DirtyType::FontStyle as usize == 0 {
                        cb.clazz.font.style = c.font.style
                    }
                    if cb.local_style & DirtyType::FontWeight as usize == 0 {
                        cb.clazz.font.weight = c.font.weight
                    }
                    if cb.local_style & DirtyType::FontSize as usize == 0 {
                        cb.clazz.font.size = c.font.size
                    }
                    if cb.local_style & DirtyType::FontFamily as usize == 0 {
                        cb.clazz.font.family = c.font.family.clone()
                    }
                    if cb.local_style & DirtyType::LetterSpacing as usize == 0 {
                        cb.clazz.style.letter_spacing = c.style.letter_spacing
                    }
                    if cb.local_style & DirtyType::WordSpacing as usize == 0 {
                        cb.clazz.style.word_spacing = c.style.word_spacing
                    }
                    if cb.local_style & DirtyType::LineHeight as usize == 0 {
                        cb.clazz.style.line_height = c.style.line_height
                    }
                    if cb.local_style & DirtyType::Indent as usize == 0 {
                        cb.clazz.style.indent = c.style.indent
                    }
                    if cb.local_style & DirtyType::WhiteSpace as usize == 0 {
                        cb.clazz.style.white_space = c.style.white_space
                    }
                    if cb.local_style & DirtyType::Color as usize == 0 {
                        cb.clazz.style.color = c.style.color.clone()
                    }
                    if cb.local_style & DirtyType::Stroke as usize == 0 {
                        cb.clazz.style.stroke = c.style.stroke.clone()
                    }
                    if cb.local_style & DirtyType::TextAlign as usize == 0 {
                        cb.clazz.style.text_align = c.style.text_align
                    }
                    if cb.local_style & DirtyType::VerticalAlign as usize == 0 {
                        cb.clazz.style.vertical_align = c.style.vertical_align
                    }
                    if cb.local_style & DirtyType::ShadowColor as usize == 0 {
                        cb.clazz.shadow.color = c.shadow.color
                    }
                    if cb.local_style & DirtyType::ShadowHV as usize == 0 {
                        cb.clazz.shadow.h = c.shadow.h;
                        cb.clazz.shadow.v = c.shadow.v;
                    }
                    if cb.local_style & DirtyType::ShadowBlur as usize == 0 {
                        cb.clazz.shadow.blur = c.shadow.blur
                    }
                },
                _ => (),
            };
        }
    }
    if cb.dirty > DirtyType::StyleClass as usize {
        cb.local_style |= cb.dirty & (!(DirtyType::Text as usize + DirtyType::StyleClass as usize));
        if cb.dirty & (DirtyType::FontStyle as usize + DirtyType::FontSize as usize + DirtyType::FontFamily as usize) != 0 {
            match read.3.get(id) {
                Some(r) => {
                    if cb.dirty & DirtyType::FontStyle as usize != 0 {
                        cb.clazz.font.style = r.style
                    }
                    if cb.dirty & DirtyType::FontWeight as usize != 0 {
                        cb.clazz.font.weight = r.weight
                    }
                    if cb.dirty & DirtyType::FontSize as usize != 0 {
                        cb.clazz.font.size = r.size
                    }
                    if cb.dirty & DirtyType::FontFamily as usize != 0 {
                        cb.clazz.font.family = r.family.clone()
                    }
                },
                _ => ()
            };
        }
        if cb.dirty & (
            DirtyType::LetterSpacing as usize +
            DirtyType::WordSpacing as usize +
            DirtyType::LineHeight as usize +
            DirtyType::Indent as usize +
            DirtyType::WhiteSpace as usize +
            DirtyType::Color as usize +
            DirtyType::Stroke as usize +
            DirtyType::TextAlign as usize +
            DirtyType::VerticalAlign as usize) != 0 {
            match read.4.get(id) {
                Some(r) => {
                    if cb.dirty & DirtyType::LetterSpacing as usize != 0 {
                    cb.clazz.style.letter_spacing = r.letter_spacing
                    }
                    if cb.dirty & DirtyType::WordSpacing as usize != 0 {
                        cb.clazz.style.word_spacing = r.word_spacing
                    }
                    if cb.dirty & DirtyType::LineHeight as usize != 0 {
                        cb.clazz.style.line_height = r.line_height
                    }
                    if cb.dirty & DirtyType::Indent as usize != 0 {
                        cb.clazz.style.indent = r.indent
                    }
                    if cb.dirty & DirtyType::WhiteSpace as usize != 0 {
                        cb.clazz.style.white_space = r.white_space
                    }
                    if cb.dirty & DirtyType::Color as usize != 0 {
                        cb.clazz.style.color = r.color.clone()
                    }
                    if cb.dirty & DirtyType::Stroke as usize != 0 {
                        cb.clazz.style.stroke = r.stroke.clone()
                    }
                    if cb.dirty & DirtyType::TextAlign as usize != 0 {
                        cb.clazz.style.text_align = r.text_align
                    }
                    if cb.dirty & DirtyType::VerticalAlign as usize != 0 {
                        cb.clazz.style.vertical_align = r.vertical_align
                    }
                },
                _ => ()
            };
        }
        if cb.dirty & (DirtyType::ShadowColor as usize + DirtyType::ShadowHV as usize + DirtyType::ShadowBlur as usize) != 0 {
            match read.5.get(id) {
                Some(r) => {
                    if cb.dirty & DirtyType::ShadowColor as usize != 0 {
                        cb.clazz.shadow.color = r.color
                    }
                    if cb.dirty & DirtyType::ShadowHV as usize != 0 {
                        cb.clazz.shadow.h = r.h;
                        cb.clazz.shadow.v = r.v;
                    }
                    if cb.dirty & DirtyType::ShadowBlur as usize != 0 {
                        cb.clazz.shadow.blur = r.blur
                    }
                },
                _ => ()
            };
        }
    }
    // 获得字体高度
    cb.font_size = read.0.get_size(&cb.clazz.font.family, &cb.clazz.font.size);
    if cb.font_size == 0.0 {
        #[cfg(feature = "warning")]
        println!("font_size==0.0");
        return true
    }
    cb.line_height = (get_line_height(cb.font_size, &cb.clazz.style.line_height) * 100.0).round()/100.0;
    if cb.line_height < cb.font_size {
        cb.line_height = cb.font_size;
    }
    cb.pos.y = (cb.line_height - cb.font_size)/2.0;
    match parent_yoga.get_style_justify() {
        YGJustify::YGJustifyCenter => cb.clazz.style.text_align = TextAlign::Center,
        YGJustify::YGJustifyFlexEnd => cb.clazz.style.text_align = TextAlign::Right,
        YGJustify::YGJustifySpaceBetween => cb.clazz.style.text_align = TextAlign::Justify,
        _ => (),
    };

    match parent_yoga.get_style_align_content() {
        YGAlign::YGAlignCenter => cb.clazz.style.vertical_align = VerticalAlign::Middle,
        YGAlign::YGAlignFlexEnd => cb.clazz.style.vertical_align = VerticalAlign::Bottom,
        _ => (),
    };

    match parent_yoga.get_style_align_items() {
        YGAlign::YGAlignCenter => cb.clazz.style.vertical_align = VerticalAlign::Middle,
        YGAlign::YGAlignFlexEnd => cb.clazz.style.vertical_align = VerticalAlign::Bottom,
        _ => (),
    };
    // TODO 如果没有文字变动， 则可以直接返回或计算布局
    cb.modify = cb.dirty;
    cb.dirty = 0;
    let count = parent_yoga.get_child_count() as usize;
    let text = match read.2.get(id) {
        Some(t) => t.0.as_ref(),
        _ => "",
    };
    // 如果父节点只有1个子节点，则认为是Text节点. 如果没有设置宽度，则立即进行不换行的文字布局计算，并设置自身的大小为文字大小
    if count == 1 {
        // let old = yoga.get_layout();
        let old_size = cb.wrap_size;
        calc_text(cb, text, read.0);
        if old_size.x != cb.wrap_size.x || old_size.y != cb.wrap_size.y {
            yoga.set_width(cb.wrap_size.x);
            yoga.set_height(cb.wrap_size.y);
        }else{
            match parent_yoga.get_style_width_unit() {
                YGUnit::YGUnitPercent | YGUnit::YGUnitPoint => {
                    calc_wrap_align(cb, &parent_yoga.get_layout());
                },
                _ => ()
            }
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
fn update_char<L: FlexNode>(id: usize, cb: &mut CharBlock<L>, c: char, w: f32, font: &FontSheet, index: &mut usize, parent: &L, yg_index: &mut usize) -> L {
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
fn set_node<L: FlexNode>(cb: &CharBlock<L>, c: char, mut w: f32, font: &FontSheet, node: L) -> L {
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

fn calc_text<'a, L: FlexNode>(cb: &mut CharBlock<L>, text: &'a str, font: &FontSheet) {
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
                cb.last_line.2 = calc.pos.x;
                cb.last_line.0 = cb.chars.len();
                cb.lines.push(cb.last_line.clone());
                cb.last_line = (0, 0, 0.0);
                cb.line_count += 1;
                update_char1(cb, '\n', 0.0, font, &mut calc);
                // 行首缩进
                calc.pos.x += cb.clazz.style.indent;
            },
            SplitResult::Whitespace =>{
                // 设置成宽度为默认字宽, 高度0
                update_char1(cb, ' ', cb.font_size, font, &mut calc);
                calc.pos.x += cb.clazz.style.letter_spacing;
                cb.last_line.1 += 1;
            },
            SplitResult::Word(c) => {
                update_char1(cb, c, 0.0, font, &mut calc);
                calc.pos.x += cb.clazz.style.letter_spacing;
                cb.last_line.1 += 1;
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
                cb.last_line.1 += 1;
            },
        }
    }
    cb.last_line.2 = calc.pos.x;
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
fn update_char1<L: FlexNode>(cb: &mut CharBlock<L>, c: char, w: f32, font: &FontSheet,  calc: &mut Calc) {
    if calc.index < cb.chars.len() {
        // let line_height = cb.line_height;
        let cn = &cb.chars[calc.index];
        // if cn.ch == c {
        //     println!("cn--------------------------{:?}", cn);
        //     set_node2(cn, line_height, cn.width, font, calc);   
        //     calc.index += 1;
        //     return
        // }
        if cn.ch != c {
            // 字符不同，将当前的，和后面的节点都释放掉
            for j in calc.index..cb.chars.len() {
                cb.chars[j].node.free()
            }
            unsafe {cb.chars.set_len(calc.index)};
        }
    }
    let p = calc.pos;
    let w = set_node1(cb, c, w, font, calc);
    cb.chars.push(CharNode {
        ch: c,
        width: w,
        pos: p,
        node: L::new_null(),
    });
    calc.index += 1;
}
// 设置节点的宽高
fn set_node1<L: FlexNode>(cb: &mut CharBlock<L>, c: char, mut w: f32, font: &FontSheet,  calc: &mut Calc) -> f32 {
    if c as u32 == 0 {
    } else if c > ' ' {
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
    }else if c == ' ' {
        calc.pos.x += w;
        if calc.max_w < calc.pos.x {
            calc.max_w = calc.pos.x
        }
    }
    w
}

/// 计算换行和对齐， 如果是单行或多行左对齐，可以直接改cb.pos
fn calc_wrap_align<L: FlexNode>(cb: &mut CharBlock<L>, layout: &Layout) {
    let x = layout.border_left + layout.padding_left;
    let w = layout.width - x - layout.border_right - layout.padding_right;
    let y = layout.border_top + layout.padding_top;
    let h = layout.height - y - layout.border_bottom - layout.padding_bottom;
    if cb.clazz.style.white_space.allow_wrap() && cb.size.x > w {
        // 换行计算
        let mut y_fix = 0.0;
        for i in 0..cb.lines.len() + 1 {
            y_fix = wrap_line(cb, i, w, y_fix)
        }
        cb.wrap_size.y += y_fix;
        cb.wrap_size.x = w;
    }
    cb.pos.x = x;
    cb.pos.y = y + (cb.line_height - cb.font_size)/2.0;
    if h > 0.0 {// 因为高度没有独立的变化，所有可以统一放在cb.pos.y
        match cb.clazz.style.vertical_align {
            VerticalAlign::Middle => cb.pos.y += (h - cb.wrap_size.y) / 2.0,
            VerticalAlign::Bottom => cb.pos.y += h - cb.wrap_size.y,
            _ => (),
        }
    }
    if cb.wrap_size.y > cb.size.y {
        // 如果换行则返回
        return;
    }
    if cb.line_count == 1 { // 单行优化
        match cb.clazz.style.text_align {
            TextAlign::Center => cb.pos.x += (w - cb.size.x) / 2.0,
            TextAlign::Right => cb.pos.x += w - cb.size.x,
            TextAlign::Justify => justify_line(cb, line_info(cb, 0), w, 0.0, 0.0),
            _ => (),
        };
        return
    }
    // 多行的3种布局的处理
    match cb.clazz.style.text_align {
        TextAlign::Center => {
            for i in 0..cb.lines.len() + 1 {
                align_line(cb, line_info(cb, i), w, 0.0, 0.0, get_center_fix)
            }
        },
        TextAlign::Right => {
            for i in 0..cb.lines.len() + 1 {
                align_line(cb, line_info(cb, i), w, 0.0, 0.0, get_right_fix)
            }
        },
        TextAlign::Justify =>{
            for i in 0..cb.lines.len() + 1 {
                justify_line(cb, line_info(cb, i), w, 0.0, 0.0)
            }
        },
        _ => (),
    };
}
fn wrap_line<L: FlexNode>(cb: &mut CharBlock<L>, line: usize, limit_width: f32, mut y_fix: f32) -> f32 {
    let (end, mut start, mut word_count, line_width) = line_info(cb, line);
    let mut x_fix = 0.0;
    let mut w = 0.0;
    while line_width + x_fix > limit_width && start < end {
        while start < end {//换行计算
            let n = unsafe {cb.chars.get_unchecked_mut(start)};
            w = n.pos.x;
            if n.pos.x + x_fix + n.width > limit_width && x_fix != -w {
                break
            }

            n.pos.x += x_fix;
            n.pos.y += y_fix;
            start += 1;
            if n.ch >= ' ' {
                word_count -= 1;
            }else if n.ch == '\n' {
            }else if n.ch == char::from(0) {
                if x_fix < 0.0 || y_fix > 0.0 {
                    let end = start + n.pos.y as usize;
                    while start < end {
                        let n = unsafe {cb.chars.get_unchecked_mut(start)};
                        n.pos.x += x_fix;
                        n.pos.y += y_fix;
                        start += 1;
                    }
                }else {
                    start += n.pos.y as usize;
                }
                word_count -= 1;
            }
        }
        y_fix += cb.line_height;
        x_fix = -w;
    }
    // 剩余的宽度， 需要计算行对齐
    match cb.clazz.style.text_align {
        TextAlign::Center => {
            align_line(cb, (end, start, word_count, line_width + x_fix), limit_width, x_fix, y_fix, get_center_fix)
        },
        TextAlign::Right => {
            align_line(cb, (end, start, word_count, line_width + x_fix), limit_width, x_fix, y_fix, get_right_fix)
        },
        TextAlign::Justify =>{
            justify_line(cb, (end, start, word_count, line_width + x_fix), limit_width, x_fix, y_fix)
        },
        _ if x_fix < 0.0 || y_fix > 0.0  => { // 如果x或者y需要修正
            while start < end {
                let n = unsafe {cb.chars.get_unchecked_mut(start)};
                n.pos.x += x_fix;
                n.pos.y += y_fix;
                start+=1;
            }
        },
        _ => (),
    };
    
    // while w > limit_width {
    //     // 计算折行
    // }
    // match cb.clazz.style.text_align {
    //     TextAlign::Center => cb.pos.x += (w - cb.size.x) / 2.0,
    //     TextAlign::Right => cb.pos.x += w - cb.size.x,
    //     // TextAlign::Justify if cb.size.x > w => justify(cb, w, cb.size.x),
    //     _ => (),
    // };
    0.0
}
fn align_line<L: FlexNode>(cb: &mut CharBlock<L>, (end, mut start, _, line_width): (usize, usize, usize, f32), limit_width: f32, x_fix: f32, y_fix: f32, get_x_fix: fn(f32, f32) -> f32) {
    let fix = get_x_fix(limit_width, line_width) + x_fix;
    if y_fix > 0.0 {
        while start < end {
            let n = unsafe {cb.chars.get_unchecked_mut(start)};
            n.pos.x += fix;
            n.pos.y += y_fix;
            start+=1;
        }
    }else{
        while start < end {
            unsafe {cb.chars.get_unchecked_mut(start)}.pos.x += fix;
            start+=1;
        }
    }
}
fn justify_line<L: FlexNode>(cb: &mut CharBlock<L>, (end, mut start, word_count, line_width): (usize, usize, usize, f32), limit_width: f32, x_fix: f32, y_fix: f32) {
    if word_count == 1 {
        unsafe {cb.chars.get_unchecked_mut(start)}.pos.x += (limit_width - line_width)/2.0;
        return;
    }
    let fix = (limit_width - line_width)/(word_count - 1) as f32;
    let i = start;
    if y_fix > 0.0 {
        while start < end {
            let n = unsafe {cb.chars.get_unchecked_mut(start)};
            // n 是容器 TODO
            n.pos.x += (start - i) as f32 * fix + x_fix;
            n.pos.y += y_fix;
            start+=1;
        }
    }else{
        while start < end {
            unsafe {cb.chars.get_unchecked_mut(start)}.pos.x += (start - i) as f32 * fix;
            start+=1;
        }
    }
}
fn line_info<L: FlexNode>(cb: &CharBlock<L>, line: usize) -> (usize, usize, usize, f32) {
    if line >= cb.lines.len() {
        (cb.chars.len(), cb.last_line.0, cb.last_line.1, cb.last_line.2)
    }else if line + 1 >= cb.lines.len(){
        let r = unsafe {cb.lines.get_unchecked(line)};
        (cb.last_line.0, r.0, r.1, r.2)
    }else{
        let r = unsafe {cb.lines.get_unchecked(line)};
        (unsafe {cb.lines.get_unchecked(line + 1)}.0, r.0, r.1, r.2)
    }
}
#[inline]
fn get_center_fix(limit_width: f32, line_width: f32) -> f32 {
    (limit_width - line_width) / 2.0
}
#[inline]
fn get_right_fix(limit_width: f32, line_width: f32) -> f32 {
    limit_width - line_width
}

impl_system!{
    LayoutImpl<L> where [L: FlexNode],
    true,
    {
        MultiCaseListener<Node, Text, CreateEvent>
        MultiCaseListener<Node, Text, ModifyEvent>
        MultiCaseListener<Node, Font, ModifyEvent>
        MultiCaseListener<Node, TextStyle, ModifyEvent>
        MultiCaseListener<Node, TextShadow, ModifyEvent>
        MultiCaseListener<Node, CharBlock<L>, DeleteEvent>
    }
}