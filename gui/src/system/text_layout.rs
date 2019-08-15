// 文字布局及布局系统
// 文本节点的布局算法： 文本节点本身所对应的yoga节点总是一个0大小的节点。文本节点的父节点才是进行文本布局的节点，称为P节点。P节点如果没有设置布局，则默认用flex布局模拟文档流布局。会将文本拆成每个字（英文为单词）的yoga节点加入P节点上。这样可以支持图文混排。P节点如果有flex布局，则遵循该布局。
// 字节点，根据字符是否为单字决定是需要字符容器还是单字。
// 文字根据样式，会处理：缩进，是否合并空白符，是否自动换行，是否允许换行符。来设置相应的flex布局。 换行符采用高度为0, 宽度100%的yoga节点来模拟。

use std::{
    os::raw::{c_void},
    marker::PhantomData,
};

use ecs::{
    system::{Runner, MultiCaseListener},
    monitor::{CreateEvent, DeleteEvent},
    component::MultiCaseImpl,
    single::SingleCaseImpl,
};

use ROOT;
use entity::{Node};
use component::{
    user::*,
    calc::*,
};
use single::class::*;
use single::*;
use layout::{YGDirection, YGFlexDirection, FlexNode, YGAlign, /*YGJustify,*/ YGWrap, YGUnit, YGEdge};
use font::font_sheet::{get_size, SplitResult, split, FontSheet, TexFont};

const MARK: usize = StyleType::LetterSpacing as usize | 
                    StyleType::WordSpacing as usize | 
                    StyleType::LineHeight as usize | 
                    StyleType::Indent as usize |
                    StyleType::WhiteSpace as usize | 
                    StyleType::TextAlign as usize | 
                    StyleType::VerticalAlign as usize |
                    StyleType::TextShadow as usize |
                    StyleType::Color as usize | 
                    StyleType::Stroke as usize |
                    StyleType::FontStyle as usize | 
                    StyleType::FontFamily as usize | 
                    StyleType::FontSize as usize | 
                    StyleType::FontWeight as usize |
                    StyleType::Text as usize;

type Read<'a, L> = (
    &'a SingleCaseImpl<ClassSheet>,
    &'a MultiCaseImpl<Node, L>,
    &'a MultiCaseImpl<Node, TextContent>,
    &'a MultiCaseImpl<Node, TextStyle>,
    &'a MultiCaseImpl<Node, ClassName>,
    &'a MultiCaseImpl<Node, WorldMatrix>,
    &'a MultiCaseImpl<Node, StyleMark>,
    &'a SingleCaseImpl<DirtyList>,
);
type Write<'a,L> = (&'a mut MultiCaseImpl<Node, CharBlock<L>>, &'a mut MultiCaseImpl<Node, Layout>, &'a mut SingleCaseImpl<FontSheet>);

pub struct LayoutImpl<L: FlexNode + 'static> {
    read: usize,
    write: usize,
    mark: PhantomData<L>,
}

pub struct TextGlphySys<L: FlexNode + 'static>(PhantomData<L>);

impl<L: FlexNode + 'static> TextGlphySys<L> {
    pub fn new() -> Self {
        Self(PhantomData)
    }   
}

impl<'a, L: FlexNode + 'static> Runner<'a> for TextGlphySys<L> {
    type ReadData = Read<'a, L>;
    type WriteData = Write<'a,L>;

    fn run(&mut self, read: Self::ReadData, mut write: Self::WriteData) {
            
        for id in (read.7).0.iter() {
            let r = match read.6.get(*id) {
                Some(r) => r,
                None => continue,
            };

            if r.dirty & MARK == 0 {
                continue;
            }

            set_gylph(*id, &read, &mut write);
        }
    } 
}


impl<'a, L: FlexNode + 'static> LayoutImpl<L> {
    pub fn new() -> Self{
        LayoutImpl {
            read: 0,
            write: 0,
            mark: PhantomData,
        }
    }
}
impl<'a, L: FlexNode + 'static> Runner<'a> for LayoutImpl<L> {
  type ReadData = Read<'a, L>;
  type WriteData = Write<'a,L>;

  fn run(&mut self, read: Self::ReadData, mut write: Self::WriteData) {;
        for id in (read.7).0.iter() {
            let r = match read.6.get(*id) {
                Some(r) => r,
                None => continue,
            };
            if r.dirty & MARK == 0 {
                continue;
            }

            calc(*id, &read, &mut write);
        }     
        let (w, h) = {
            let layout = unsafe{ write.1.get_unchecked(ROOT)};
            (layout.width, layout.height)
        };
        self.read= &read as *const Read<'a, L> as usize;
        self.write= &mut write as *mut Write<'a,L> as usize;
        //计算布局，如果布局更改， 调用回调来设置layout属性，及字符的位置
        unsafe{ read.1.get_unchecked(ROOT)}.calculate_layout_by_callback(w, h, YGDirection::YGDirectionLTR, callback::<L>, self as *const LayoutImpl<L> as *const c_void);
    }
}

impl<'a, L: FlexNode + 'static> MultiCaseListener<'a, Node, TextContent, CreateEvent> for LayoutImpl<L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        write.insert(event.id, CharBlock{
            font_size: 16.0,
            font_height: 16.0,
            stroke_width: 0.0,
            line_height: 20.0,
            chars: Vec::new(),
            lines: Vec::new(),
            last_line: (0, 0, 0.0),
            size: Vector2::default(),
            wrap_size: Vector2::default(),
            pos: Point2::default(),
            line_count: 1,
            fix_width: true,
            style_class: 0,
            is_pixel: false,
        });
    }
}

// 监听CharBlock的删除
impl<'a, L: FlexNode + 'static> MultiCaseListener<'a, Node, CharBlock<L>, DeleteEvent> for LayoutImpl<L> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, CharBlock<L>>;

    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, write: Self::WriteData) {
        let cb = unsafe{ write.get_unchecked(event.id)};
        // 删除所有yoga节点
        for cn in cb.chars.iter() {
            cn.node.get_parent().remove_child(cn.node);
            // cn.node.free();
        }
    }
}

//================================ 内部静态方法
//回调函数
extern "C" fn callback<L: FlexNode + 'static>(node: L, callback_args: *const c_void) {
    //更新布局
    let b = node.get_bind() as usize;
    let id = node.get_context() as usize;
    let layout_impl = unsafe{ &mut *(callback_args as usize as *mut LayoutImpl<L>) };
    let write = unsafe{ &mut *(layout_impl.write as *mut Write<L>) };
    let read = unsafe{ &mut *(layout_impl.read as *mut Read<L>) };
    if b == 0 {  
        //如果是span节点， 不更新布局， 因为渲染对象使用了span的世界矩阵， 如果span布局不更新， 那么其世界矩阵与父节点的世界矩阵相等
        if let Some(cb) = write.0.get_mut(id) {
            let text_style = unsafe { read.3.get_unchecked(id) };
            // 只有百分比大小的需要延后布局的计算， 根据是否居中靠右或填充，或者换行，进行文字重布局
            let node = node.get_parent();
            if node.get_child_count() == 1 {
                match node.get_style_width_unit() {
                    YGUnit::YGUnitPercent | YGUnit::YGUnitPoint => {
                        calc_wrap_align(cb, &text_style, &node.get_layout());
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
fn update<'a, L: FlexNode + 'static>(mut node: L, id: usize, char_index: usize, write: &mut Write<L>) {
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

// 设置字形的id
fn set_gylph<'a, L: FlexNode + 'static>(id: usize, read: &Read<L>, write: &mut Write<L>) {
    let cb = unsafe{ write.0.get_unchecked_mut(id)};
    let scale = unsafe { read.5.get_unchecked(id).y.y };
    let text_style = unsafe{ read.3.get_unchecked(id)};

    let tex_font = match write.2.get_font_info(&text_style.font.family) {
        Some(r) => r.0.clone(),
        None => {
            println!("font is not exist, face_name: {:?}", text_style.font.family.as_ref());
            return;
        },
    };

    let weight = text_style.font.weight;

    for char_node in cb.chars.iter_mut() {
        let ch_id = write.2.calc_gylph(&tex_font, cb.font_size as usize, cb.stroke_width as usize, weight, scale, char_node.base_width as usize, char_node.ch);
        char_node.ch_id = ch_id;
    }
}

// 计算节点的L的布局参数， 返回是否保留在脏列表中
fn calc<'a, L: FlexNode + 'static>(id: usize, read: &Read<L>, write: &mut Write<L>) -> bool {
    let cb = match write.0.get_mut(id) {
        Some(r) => r,
        None => return false, 
    };
    let text_style = unsafe{ read.3.get_unchecked(id)};
    let yoga = unsafe { read.1.get_unchecked(id).clone() };
    let parent_yoga = yoga.get_parent();
    if parent_yoga.is_null() {
        #[cfg(feature = "warning")]
        println!("parent_yoga.is_null");
        return true
    }

    let tex_font = match write.2.get_font_info(&text_style.font.family) {
        Some(r) => r,
        None => {
            println!("font is not exist, face_name: {:?}", text_style.font.family.as_ref());
            return true;
        },
    };
    // 获得字体高度
    cb.font_size = get_size(tex_font.1, &text_style.font.size) as f32;
    cb.font_height = tex_font.0.get_font_height(cb.font_size as usize, text_style.text.stroke.width);
    if cb.font_size == 0.0 {
        #[cfg(feature = "warning")]
        println!("font_size==0.0, family: {:?}", text_style.font.family);
        return true
    }
    cb.line_height = tex_font.0.get_line_height(cb.font_size as usize, &text_style.text.line_height);
    // if cb.line_height < cb.font_size{
    //     cb.line_height = cb.font_size;
    // }
    cb.pos.y = (cb.line_height - cb.font_height)/2.0;
    // match parent_yoga.get_style_justify() {
    //     YGJustify::YGJustifyCenter => text_style.text.text_align = TextAlign::Center,
    //     YGJustify::YGJustifyFlexEnd => text_style.text.text_align = TextAlign::Right,
    //     YGJustify::YGJustifySpaceBetween => text_style.text.text_align = TextAlign::Justify,
    //     _ => (),
    // };

    // match parent_yoga.get_style_align_content() {
    //     YGAlign::YGAlignCenter => text_style.text.vertical_align = VerticalAlign::Middle,
    //     YGAlign::YGAlignFlexEnd => text_style.text.vertical_align = VerticalAlign::Bottom,
    //     _ => (),
    // };

    // match parent_yoga.get_style_align_items() {
    //     YGAlign::YGAlignCenter => text_style.text.vertical_align = VerticalAlign::Middle,
    //     YGAlign::YGAlignFlexEnd => text_style.text.vertical_align = VerticalAlign::Bottom,
    //     _ => (),
    // };
    // TODO 如果没有文字变动， 则可以直接返回或计算布局
    // cb.modify = cb.dirty;
    let count = parent_yoga.get_child_count() as usize;
    let text = match read.2.get(id) {
        Some(t) => t.0.as_ref(),
        _ => "",
    };
    let sw = text_style.text.stroke.width as usize;
    cb.stroke_width = sw as f32;
    // let scale = match read.7.get(id) {
    //     Some(w) => w.y.y,
    //     _ => 1.0
    // };
    let tex_font = tex_font.0.clone();
    cb.is_pixel = tex_font.is_pixel;
    let mut tex_param = TexParam {cb: cb, tex_font: &tex_font, text_style: text_style, word_margin: text_style.text.letter_spacing / 2.0 - text_style.text.stroke.width};
    let tex_param = &mut tex_param;
    // 如果父节点只有1个子节点，则认为是Text节点. 如果没有设置宽度，则立即进行不换行的文字布局计算，并设置自身的大小为文字大小
    if count == 1 {
        // let old = yoga.get_layout();
        let old_size = tex_param.cb.wrap_size;
        calc_text(tex_param, text, sw, write.2);
        if old_size.x != tex_param.cb.wrap_size.x || old_size.y != tex_param.cb.wrap_size.y {
            yoga.set_width(tex_param.cb.wrap_size.x);
            yoga.set_height(tex_param.cb.wrap_size.y);
        }else{
            match parent_yoga.get_style_width_unit() {
                YGUnit::YGUnitPercent | YGUnit::YGUnitPoint => {
                    calc_wrap_align(tex_param.cb, text_style, &parent_yoga.get_layout());
                },
                _ => ()
            }
            unsafe { write.0.get_unchecked_write(id).modify(|_|{
                return true;
            }) };
        }
        return false
    }
    if text_style.text.white_space.allow_wrap() {
        parent_yoga.set_flex_wrap(YGWrap::YGWrapWrap);
    }else {
        parent_yoga.set_flex_wrap(YGWrap::YGWrapNoWrap);
    }
    // 如果有缩进变化, 则设置本span节点, 宽度为缩进值
    yoga.set_width(text_style.text.indent);
    
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
    for cr in split(text, true, text_style.text.white_space.preserve_spaces()) {
        match cr {
            SplitResult::Newline =>{
                update_char(id, tex_param, '\n', 0.0, sw, write.2, &mut index, &parent_yoga, &mut yg_index);
                update_char(id, tex_param, '\t', text_style.text.indent, sw, write.2, &mut index, &parent_yoga, &mut yg_index);
            },
            SplitResult::Whitespace =>{
                let font_size = tex_param.cb.font_size;
                // 设置成宽度为半高, 高度0
                update_char(id, tex_param, ' ', font_size/2.0, sw, write.2, &mut index, &parent_yoga, &mut yg_index);
            },
            SplitResult::Word(c) => {
                update_char(id, tex_param, c, 0.0, sw, write.2, &mut index, &parent_yoga, &mut yg_index);
            },
            SplitResult::WordStart(c) => {
                word = update_char(0, tex_param, char::from(0), 0.0, sw, write.2, &mut index, &parent_yoga, &mut yg_index);
                update_char(id, tex_param, c, 0.0, sw, write.2, &mut index, &word, &mut word_index);
            },
            SplitResult::WordNext(c) =>{
                update_char(id, tex_param, c, 0.0, sw, write.2, &mut index, &word, &mut word_index);
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
            cb.chars[i].node.get_parent().remove_child(cb.chars[i].node);
            // cb.chars[i].node.free(); // 调用remove_child方法是， node会被释放
        }
        unsafe{cb.chars.set_len(index)};
    }

    false
}
// 更新字符，如果字符不同，则清空后重新插入
fn update_char<L: FlexNode + 'static>(id: usize, tex_param: &mut TexParam<L>, c: char, w: f32, sw: usize, font: &mut FontSheet, index: &mut usize, parent: &L, yg_index: &mut usize) -> L {
    let i = *index;
    if i < tex_param.cb.chars.len() {
        let cn = &tex_param.cb.chars[i];
        if cn.ch == c {
        *index = i + 1;
        *yg_index += 1;
        return cn.node.clone()
        }
        // 字符不同，将当前的，和后面的节点都释放掉
        for j in i..tex_param.cb.chars.len() {
            tex_param.cb.chars[j].node.free()
        }
        unsafe {tex_param.cb.chars.set_len(i)};
    }
    let (w, node, base_width) = set_node(tex_param, c, w, sw, font, L::new());
    let cn = CharNode {
        ch: c,
        width: w,
        pos: Point2::default(),
        ch_id: id,
        base_width: base_width,
        node: node.clone(),
    };
    node.set_bind((i + 1) as *mut c_void);
    node.set_context(id as *mut c_void);
    parent.insert_child(node, *yg_index as u32);
    tex_param.cb.chars.push(cn);
    *index = i + 1;
    *yg_index += 1;
    node
}
// 设置节点的宽高
fn set_node<L: FlexNode + 'static>(tex_param: &mut TexParam<L>, c: char, w: f32, sw: usize, font: &mut FontSheet, node: L) -> (f32, L, f32) {
    let TexParam {cb, tex_font, text_style, word_margin} = tex_param;
    if c > ' ' {
        let r = font.measure(tex_font, cb.font_size as usize, sw, text_style.font.weight, c);
        node.set_width(r.0.x);
        node.set_height(cb.line_height);
        node.set_margin(YGEdge::YGEdgeLeft, *word_margin);
        node.set_margin(YGEdge::YGEdgeRight, *word_margin);
        match text_style.text.vertical_align {
            VerticalAlign::Middle => node.set_align_self(YGAlign::YGAlignCenter),
            VerticalAlign::Top => node.set_align_self(YGAlign::YGAlignFlexStart),
            VerticalAlign::Bottom => node.set_align_self(YGAlign::YGAlignFlexEnd),
        };
        return (r.0.x, node, r.1)
    }
    if c == '\n' {
        node.set_width_percent(100.0);
    }else if c == char::from(0) {
        node.set_width_auto();
        node.set_height(cb.line_height);
        node.set_flex_direction(YGFlexDirection::YGFlexDirectionRow);
    }else{ // "\t"
        node.set_width(w);
    }
    (w, node, 0.0)
}

#[derive(Debug)]
struct Calc {
    index: usize,
    pos: Point2,
    max_w: f32,
    word: usize,
}

fn calc_text<'a, L: FlexNode + 'static>(tex_param: &mut TexParam<L>, text: &'a str, sw: usize, font: &mut FontSheet) {
    let mut calc = Calc{
        index: 0,
        pos: Point2::default(),
        max_w: 0.0,
        word: 0,
    };
    // 根据每个字符, 创建对应的yoga节点, 加入父容器或字容器中
    for cr in split(text, true, tex_param.text_style.text.white_space.preserve_spaces()) {
        match cr {
            SplitResult::Newline =>{
                tex_param.cb.last_line.2 = calc.pos.x;
                tex_param.cb.last_line.0 = tex_param.cb.chars.len();
                tex_param.cb.lines.push(tex_param.cb.last_line.clone());
                tex_param.cb.last_line = (0, 0, 0.0);
                tex_param.cb.line_count += 1;
                update_char1(tex_param, '\n', 0.0, sw, font, &mut calc);
                // 行首缩进
                calc.pos.x += tex_param.text_style.text.indent;
            },
            SplitResult::Whitespace =>{
                // 设置成宽度为默认字宽, 高度0
                update_char1(tex_param, ' ', tex_param.cb.font_size, sw, font, &mut calc);
                calc.pos.x += tex_param.text_style.text.letter_spacing - tex_param.cb.stroke_width + 1.0;
                tex_param.cb.last_line.1 += 1;
            },
            SplitResult::Word(c) => {
                update_char1(tex_param, c, 0.0, sw, font, &mut calc);
                calc.pos.x += tex_param.text_style.text.letter_spacing - tex_param.cb.stroke_width + 1.0;
                tex_param.cb.last_line.1 += 1;
            },
            SplitResult::WordStart(c) => {
                calc.word = calc.index;
                update_char1(tex_param, 0 as char, 0.0, sw, font, &mut calc);
                update_char1(tex_param, c, 0.0, sw, font, &mut calc);
            },
            SplitResult::WordNext(c) =>{
                calc.pos.x += tex_param.text_style.text.letter_spacing - tex_param.cb.stroke_width + 1.0;
                update_char1(tex_param, c, 0.0, sw, font, &mut calc);
            },
            SplitResult::WordEnd =>{
                let node = unsafe {tex_param.cb.chars.get_unchecked_mut(calc.word)};
                node.width = calc.pos.x - node.pos.x;
                node.pos.y = (calc.index - calc.word) as f32;
                calc.word = 0;
                calc.pos.x += tex_param.text_style.text.word_spacing;
                tex_param.cb.last_line.1 += 1;
            },
        }
    }
    tex_param.cb.last_line.2 = calc.pos.x;
    //清除多余的CharNode
    if calc.index < tex_param.cb.chars.len() {
        for i in calc.index..tex_param.cb.chars.len() {
            tex_param.cb.chars[i].node.get_parent().remove_child(tex_param.cb.chars[i].node);
            // cb.chars[i].node.free(); // 调用remove_child方法是， node会被释放
        }
        unsafe{tex_param.cb.chars.set_len(calc.index)};
    }
    tex_param.cb.size.x = calc.max_w;
    tex_param.cb.size.y = calc.pos.y + tex_param.cb.line_height;
    tex_param.cb.wrap_size = tex_param.cb.size;
}
// 更新字符，如果字符不同，则清空后重新插入
fn update_char1<L: FlexNode + 'static>(tex_param: &mut TexParam<L>, c: char, w: f32, sw: usize, font: &mut FontSheet,  calc: &mut Calc) {
    if calc.index < tex_param.cb.chars.len() {
        // let line_height = tex_param.cb.line_height;
        let cn = &tex_param.cb.chars[calc.index];
        // if cn.ch == c {
        //     println!("cn--------------------------{:?}", cn);
        //     set_node2(cn, line_height, c, cn.width, font, calc);   
        //     calc.index += 1;
        //     return
        // }
        if cn.ch != c {
            // 字符不同，将当前的，和后面的节点都释放掉
            for j in calc.index..tex_param.cb.chars.len() {
                tex_param.cb.chars[j].node.get_parent().remove_child(tex_param.cb.chars[j].node);
                // tex_param.cb.chars[j].node.free()
            }
            unsafe {tex_param.cb.chars.set_len(calc.index)};
        }
    }
    let p = calc.pos;
    let (w, base_width) = set_node1(tex_param, c, w, sw, font, calc);
    tex_param.cb.chars.push(CharNode {
        ch: c,
        width: w,
        pos: p,
        ch_id: 0,
        base_width: base_width,
        node: L::new_null(),
    });
    calc.index += 1;
}
// 设置节点的宽高
fn set_node1<L: FlexNode + 'static>(tex_param: &mut TexParam<L>, c: char, w: f32, sw: usize, font: &mut FontSheet, calc: &mut Calc) -> (f32, f32) {
    if c > ' ' {
        let r= font.measure(tex_param.tex_font, tex_param.cb.font_size as usize, sw, tex_param.text_style.font.weight, c);
       //  w = font.measure(&text_style.font.family, tex_param.cb.font_size as usize, sw, c).0.x;
        if r.0.x != tex_param.cb.font_size && tex_param.cb.fix_width {
            tex_param.cb.fix_width = false
        }
        calc.pos.x += r.0.x;
        if calc.max_w < calc.pos.x {
            calc.max_w = calc.pos.x
        }
        return (r.0.x, r.1)
    }
    if c == '\n' {
        calc.pos.x = 0.0;
        calc.pos.y += tex_param.cb.line_height;
    }else if c == ' ' {
        calc.pos.x += w;
        if calc.max_w < calc.pos.x {
            calc.max_w = calc.pos.x
        }
    }
    (w, 0.0)
}

/// 计算换行和对齐， 如果是单行或多行左对齐，可以直接改tex_param.cb.pos
fn calc_wrap_align<L: FlexNode + 'static>(cb: &mut CharBlock<L>, text_style: &TextStyle, layout: &Layout) {
    let x = layout.border_left + layout.padding_left;
    let w = layout.width - x - layout.border_right - layout.padding_right;
    let y = layout.border_top + layout.padding_top;
    let h = layout.height - y - layout.border_bottom - layout.padding_bottom;
    if text_style.text.white_space.allow_wrap() && cb.size.x > w {
        // 换行计算
        let mut y_fix = 0.0;
        for i in 0..cb.lines.len() + 1 {
            y_fix = wrap_line(cb, text_style, i, w, y_fix)
        }
        cb.wrap_size.y += y_fix;
        cb.wrap_size.x = w;
    }
    cb.pos.x = x;
    cb.pos.y += y;

    if h > 0.0 {// 因为高度没有独立的变化，所有可以统一放在cb.pos.y
        match text_style.text.vertical_align {
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
        match text_style.text.text_align {
            TextAlign::Center => cb.pos.x += (w - cb.size.x) / 2.0,
            TextAlign::Right => cb.pos.x += w - cb.size.x,
            TextAlign::Justify => justify_line(cb, line_info(cb, 0), w, 0.0, 0.0),
            _ => (),
        };
        return
    }
    // 多行的3种布局的处理
    match text_style.text.text_align {
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
fn wrap_line<L: FlexNode + 'static>(cb: &mut CharBlock<L>, text_style: &TextStyle, line: usize, limit_width: f32, mut y_fix: f32) -> f32 {
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
    match text_style.text.text_align {
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
    // match text_style.text.text_align {
    //     TextAlign::Center => cb.pos.x += (w - cb.size.x) / 2.0,
    //     TextAlign::Right => cb.pos.x += w - cb.size.x,
    //     // TextAlign::Justify if cb.size.x > w => justify(cb, w, cb.size.x),
    //     _ => (),
    // };
    0.0
}
fn align_line<L: FlexNode + 'static>(cb: &mut CharBlock<L>, (end, mut start, _, line_width): (usize, usize, usize, f32), limit_width: f32, x_fix: f32, y_fix: f32, get_x_fix: fn(f32, f32) -> f32) {
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
fn justify_line<L: FlexNode + 'static>(cb: &mut CharBlock<L>, (end, mut start, word_count, line_width): (usize, usize, usize, f32), limit_width: f32, x_fix: f32, y_fix: f32) {
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
fn line_info<L: FlexNode + 'static>(cb: &CharBlock<L>, line: usize) -> (usize, usize, usize, f32) {
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

struct TexParam<'a, L: FlexNode> {
    cb: &'a mut CharBlock<L>,
    text_style: &'a TextStyle,
    tex_font: &'a TexFont,
    word_margin: f32,
}

impl_system!{
    LayoutImpl<L> where [L: FlexNode],
    true,
    {
        MultiCaseListener<Node, TextContent, CreateEvent>
        MultiCaseListener<Node, CharBlock<L>, DeleteEvent>
    }
}

impl_system!{
    TextGlphySys<L> where [L: FlexNode],
    true,
    {
    }
}