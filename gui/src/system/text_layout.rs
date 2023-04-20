use std::mem::transmute;
// 文字布局及布局系统
// 文本节点的布局算法： 文本节点本身所对应的yoga节点总是一个0大小的节点。文本节点的父节点才是进行文本布局的节点，称为P节点。P节点如果没有设置布局，则默认用flex布局模拟文档流布局。会将文本拆成每个字（英文为单词）的yoga节点加入P节点上。这样可以支持图文混排。P节点如果有flex布局，则遵循该布局。
// 字节点，根据字符是否为单字决定是需要字符容器还是单字。
// 文字根据样式，会处理：缩进，是否合并空白符，是否自动换行，是否允许换行符。来设置相应的flex布局。 换行符采用高度为0, 宽度100%的yoga节点来模拟。
use std::{marker::PhantomData, result::Result};
use ordered_float::OrderedFloat;

use ecs::{Event, StdCell};
use ecs::{
	component::MultiCaseImpl,
	entity::EntityImpl,
    monitor::{ModifyEvent},
    single::SingleCaseImpl,
	system::{MultiCaseListener, Runner},
};

use share::Share;

use flex_layout::*;
use hal_core::*;
use flex_layout::{Dimension, INodeStateType};
use crate::component::{calc::*, user::*, calc::LayoutR};
use crate::entity::Node;
use crate::font::font_sheet::{get_line_height, get_size, split, FontSheet, SplitResult, TexFont};
use crate::single::*;
use crate::render::engine::ShareEngine;

const MARK_LAYOUT: usize = StyleType::LetterSpacing as usize
    | StyleType::WordSpacing as usize
    | StyleType::LineHeight as usize
    | StyleType::Indent as usize
    | StyleType::WhiteSpace as usize
    | StyleType::TextAlign as usize
    | StyleType::VerticalAlign as usize
    | StyleType::Stroke as usize
    | StyleType::FontStyle as usize
    | StyleType::FontFamily as usize
    | StyleType::FontSize as usize
    | StyleType::FontWeight as usize;

const MARK: usize = MARK_LAYOUT | StyleType::Text as usize;

type Read<'a> = (
    &'a MultiCaseImpl<Node, TextContent>,
    &'a MultiCaseImpl<Node, ClassName>,
    &'a MultiCaseImpl<Node, WorldMatrix>,
    &'a MultiCaseImpl<Node, StyleMark>,
    &'a SingleCaseImpl<DirtyList>,
);
type Write<'a> = (
    &'a mut MultiCaseImpl<Node, NodeState>, // TODO
	&'a mut MultiCaseImpl<Node, LayoutR>,
	&'a mut MultiCaseImpl<Node, RectLayoutStyle>,
	&'a mut MultiCaseImpl<Node, OtherLayoutStyle>,
	&'a mut MultiCaseImpl<Node, TextStyle>,
	&'a mut SingleCaseImpl<Share<StdCell<FontSheet>>>,
	&'a mut SingleCaseImpl<IdTree>,
	&'a mut EntityImpl<Node>,
);

pub struct LayoutImpl {
    read: usize,
    write: usize,
}

pub struct TextGlphySys<C: HalContext + 'static>(pub PhantomData<C>);

impl<'a, C: HalContext + 'static> Runner<'a> for TextGlphySys<C> {
    type ReadData = (
		&'a MultiCaseImpl<Node, TextContent>,
		&'a MultiCaseImpl<Node, ClassName>,
		&'a MultiCaseImpl<Node, WorldMatrix>,
		&'a MultiCaseImpl<Node, StyleMark>,
		&'a SingleCaseImpl<DirtyList>,
		&'a SingleCaseImpl<ShareEngine<C>>,
		&'a SingleCaseImpl<RenderBegin>
	);
    type WriteData = Write<'a>;

    fn run(&mut self, read: Self::ReadData, mut write: Self::WriteData) {
		let mut flag = true;
		let mut count = 0;
		while flag {
			flag = false;
			count += 1;
			if count > 2 { // 迭代了两次以上，则可能进入了死循环，报错
				log::debug!("TextGlphySys dead loop, the current texture size cannot cache existing text");
				panic!("TextGlphySys dead loop");
			}

			for id in (read.4).0.iter() {
				let r = match read.3.get(*id) {
					Some(r) => r,
					None => continue,
				};
	
				if (r.dirty & MARK == 0) || read.0.get(*id).is_none(){
					continue;
				}
	
				match set_gylph(*id, &(read.0, read.1, read.2, read.3, read.4), &mut write) {
					Result::Err(_message) => {	
						log::info!("textTexture flow, reset textTexture, ${:?}", _message);
						// panic!("err:{:?}", message);
						let mut font_sheet = write.5.borrow_mut();
						font_sheet.clear_gylph();

						// // // 纹理清空为玫红 （不清理，已经处理了采样溢出边界问题，因此可以不用清理，使用sdf时，用的是alpha通道纹理，也没法通过绘制的方式清理纹理）
						// let (width, height) = (font_sheet.font_tex.texture.width, font_sheet.font_tex.texture.height);
						// // let mut vec = Vec::with_capacity(width * height * 4);
						// // for _a in 0..width * height {
						// // 	vec.push(0);
						// // 	vec.push(0);
						// // 	vec.push(1);
						// // 	vec.push(1);
						// // }
						// // read.5.gl.texture_update(&font_sheet.font_tex.texture.bind, 0, &TextureData::U8(0, 0, width as u32, height as u32, vec.as_slice()));

						// let target = read.5.gl.rt_create(
						// 	width as u32,
						// 	height as u32,
						// )
						// .unwrap();
						// // let texture = read.5.gl.texture_create_2d(
						// // 	0, 
						// // 	width as u32,
						// // 	height as u32,
						// // 		PixelFormat::RGBA,
						// // 		DataFormat::UnsignedByte,
						// // 	false, 
						// // 	None
						// // ).unwrap();
						// read.5.gl.rt_set_color(&target, Some(&font_sheet.font_tex.texture.bind));
						
						// read.5.gl.render_begin(Some(&target), &RenderBeginDesc{
						// 	viewport: (0,0,width as i32,height as i32),
						// 	scissor: (0,0,width as i32,height as i32),
						// 	clear_color: Some((OrderedFloat(1.0), OrderedFloat(0.0), OrderedFloat(1.0), OrderedFloat(1.0))),
						// 	clear_depth: read.6.0.clear_depth.clone(),
						// 	clear_stencil: read.6.0.clear_stencil.clone(),
						// }, true);

						// read.5.gl.render_end();
						
						// 对界面上的文字全部重新计算字形
						let idtree = &write.6;
						let root = &idtree[1];
						let notify = read.0.get_notify_ref();
						for (id, _node) in idtree.recursive_iter(root.children().head) {
							if read.0.get(id).is_some() { // 文字节点，发送修改事件
								notify.modify_event(id, "", 0)
							}
						}
						flag = true; // 重新迭代
						break;
					},
					_ => ()
				};
			}
		}
    }
}

impl<'a> LayoutImpl {
    pub fn new() -> Self {
        LayoutImpl {
            read: 0,
            write: 0,
        }
    }
}

// node.data.state.vnode_true()

// impl<'a> MultiCaseListener<'a, Node, TextContent, CreateEvent> for LayoutImpl {
// 	type ReadData = ();
//     type WriteData = &'a mut MultiCaseImpl<Node, TextStyle>;
//     fn listen(&mut self, event: &CreateEvent, _: Self::ReadData, text_styles: Self::WriteData) {
// 		// node_states[event.id].0.set_vnode(true);
// 		// 如果不存在TextStyle， 默认插入
// 		// if let None = text_styles.get(event.id) {
// 		// 	text_styles.insert_no_notify(event.id, TextStyle::default());
// 		// }
// 	}
// }

impl<'a> Runner<'a> for LayoutImpl {
    type ReadData = Read<'a>;
    type WriteData = Write<'a>;
	
    fn run(&mut self, read: Self::ReadData, mut write: Self::WriteData) {
		
		// 暂时拷贝， TODO
		let dirty_list = (read.4).0.clone();
		// let time = std::time::Instant::now();
        for id in dirty_list.iter() {
            let r = match read.3.get(*id) {
                Some(r) => r,
                None => continue,
			};

			
            if (r.dirty & MARK == 0) || read.0.get(*id).is_none(){
                continue;
			}
			// println!("text dirty===================textContent dirty{:?}, layout_dirty:{}, dirty:{}, id:{}", r.dirty & StyleType::Text as usize, r.dirty & MARK_LAYOUT, r.dirty, id);
            calc(*id, &read, &mut write, r.dirty & MARK_LAYOUT);
		}
	}
}

#[derive(Default)]
pub struct TextLayoutUpdateSys(Vec<usize>);

impl<'a> Runner<'a> for TextLayoutUpdateSys {
    type ReadData =  ();
    type WriteData = (
		&'a mut MultiCaseImpl<Node, NodeState>,
		&'a mut MultiCaseImpl<Node, LayoutR>,
		&'a mut SingleCaseImpl<IdTree>);

    fn run(&mut self, _read: Self::ReadData, write: Self::WriteData) {
        for id in self.0.iter() {
			update_layout(*id, write.0, write.1,write.2);
		}
		self.0.clear();
    }
}

impl<'a> MultiCaseListener<'a, Node, LayoutR, ModifyEvent> for TextLayoutUpdateSys {
    type ReadData =  &'a MultiCaseImpl<Node, TextContent>;
    type WriteData = (
		&'a mut MultiCaseImpl<Node, NodeState>,
		&'a mut MultiCaseImpl<Node, LayoutR>,
		&'a mut SingleCaseImpl<IdTree>);
    fn listen(&mut self, event: &Event, text_contents: Self::ReadData, write: Self::WriteData) {
		let id = event.id;
		// 如果是虚拟节点，并且是文字节点，需要将字符的布局结果拷贝到INode中
		if write.0[id].0.is_vnode() && text_contents.get(id).is_some() {
			self.0.push(id);
		}
	}
}


fn update_layout(
	id: usize, 
	node_states: &mut MultiCaseImpl<Node, NodeState>,
	layout_rs: &mut MultiCaseImpl<Node, LayoutR>,
	idtree: &SingleCaseImpl<IdTree>,
) {
	let n = match idtree.get(id) {
		Some(r) => r,
		None => return,
	};
	let node_state = unsafe{&mut *((&node_states[id]) as *const NodeState as *mut NodeState)};
	let chars = &mut node_state.0.text;

	// 可能已经不再是文字节点
	if chars.len() == 0 {
		return
	}
	let mut rect = Rect{
		start: std::f32::MAX,
		end: 0.0,
		top: std::f32::MAX,
		bottom: 0.0,
	};
	for (id, _node) in idtree.recursive_iter(n.children().head) {
		let char = &mut chars[node_states[id].char_index];
		let l = &layout_rs[id].rect;
		char.pos = (l.start, l.top);

		if l.start < rect.start {
			rect.start = l.start;
		}
		if l.top < rect.top {
			rect.top = l.top;
		}

		if l.end > rect.end {
			rect.end = l.end;
		}
		if l.bottom > rect.bottom {
			rect.bottom = l.bottom;
		}
	}
	
	// 设置文字虚拟节点的矩形
	if rect.start <= rect.end {
		layout_rs[id].rect = rect;
	}
}

// 设置字形的id
fn set_gylph<'a>(
	id: usize, 
	(_text_contents, _class_names, world_matrixs, _style_marks, _dirty_list): &Read, 
	(node_states, _layout_rs, _rect_layout_styles, _other_layout_styles, text_styles, font_sheet, _idtree, _nodes): &mut Write) -> Result<(), String> {
	let scale = Vector4::from(world_matrixs[id].fixed_columns(1));
	let scale = scale.dot(&scale).sqrt();
	if scale < 0.000001 {
		return Ok(());
	}

	let text_style = &text_styles[id];
	let font_sheet = &mut font_sheet.borrow_mut();
	
    let (tex_font, font_size) = match font_sheet.get_font_info(&text_style.font.family) {
        Some(r) => (r.0.clone(), get_size(r.1, &text_style.font.size) as f32),
        None => {
			log::debug!("font is not exist, face_name: {:?}, id: {:?}",
			text_style.font.family,
			id);
			return Ok(());
        }
	};

	let weight = text_style.font.weight;
	let sw = text_style.text.stroke.width;

	node_states[id].0.scale = scale;
	let chars = &mut node_states[id].0.text;
	let mut char_id;
	// clear_gylph
    for char_node in chars.iter_mut(){
        if char_node.ch > ' ' {
            char_id = font_sheet.calc_gylph(
                &tex_font,
                font_size as usize,
                sw as usize,
                weight,
                scale,
                char_node.base_width,
                char_node.ch,
            );
			// 异常，无法计算字形
			if char_id == 0 {
				return Result::Err(String::from(format!("异常，无法计算字形,char:{:?}, family:{:?}, id:{:?}", char_node.ch, text_style.font.family, id) ));
			}
			char_node.ch_id_or_count = char_id;
        }
    }
	return Ok(())
}

struct Calc<'a> {
	id: usize,
	text_content: &'a MultiCaseImpl<Node, TextContent>,
	style_marks: &'a MultiCaseImpl<Node, StyleMark>,
	layout_rs: &'a mut MultiCaseImpl<Node, LayoutR>,
	rect_layout_styles: &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
	other_layout_styles: &'a mut MultiCaseImpl<Node, OtherLayoutStyle>,
	nodes: &'a mut EntityImpl<Node>,
	idtree: &'a mut SingleCaseImpl<IdTree>,
	font_sheet: &'a mut FontSheet,

	text: &'a str,
	style_mark: &'a StyleMark,
	tex_font: (Vec<usize>, usize),
	font_size: f32,
	font_height: f32,
	line_height: f32,
	sw: f32,
	char_margin: f32,
	word_margin: f32,
	text_style: &'a TextStyle,
	parent: usize,
}

impl<'a> Calc<'a> {
	// 将文字样式用flex布局属性替换, 可以考虑不支持文字的布局属性？
	fn fit_text_style(&mut self) {
		let (local_style, class_style, local_style2, class_style2, text, other_layout_styles, id) = (self.style_mark.local_style, self.style_mark.class_style, self.style_mark.local_style2, self.style_mark.class_style2, &self.text_style.text, &mut self.other_layout_styles, self.id);
		// 兼容目前使用父节点的对齐属性来对齐文本， 如果项目将其修改正确， 应该去掉该段TODO
		if local_style & StyleType::TextAlign as usize > 0 || class_style & StyleType::TextAlign as usize > 0 {
			other_layout_styles[id].justify_content = match text.text_align {
				TextAlign::Center => JustifyContent::Center,
				TextAlign::Right => JustifyContent::FlexEnd,
				TextAlign::Left => JustifyContent::FlexStart,
				TextAlign::Justify => JustifyContent::SpaceBetween,
			};
		}
		
		if local_style & StyleType::VerticalAlign as usize > 0 || class_style & StyleType::VerticalAlign as usize > 0 {
			let r= match text.vertical_align {
				VerticalAlign::Middle => AlignItems::Center,
				VerticalAlign::Bottom => AlignItems::FlexEnd,
				VerticalAlign::Top => AlignItems::FlexStart
			};
			other_layout_styles[id].align_items = r;
			let r= match text.vertical_align {
				VerticalAlign::Middle => AlignContent::Center,
				VerticalAlign::Bottom => AlignContent::FlexEnd,
				VerticalAlign::Top => AlignContent::FlexStart
			};
			other_layout_styles[id].align_content = r;
		
		} else if local_style2 & StyleType2::AlignContent as usize == 0 && class_style2 & StyleType2::AlignContent as usize == 0 {
			// 文字的容器默认align_content为FlexStart
			other_layout_styles[id].align_content = AlignContent::FlexStart;
		}
	
		if local_style & StyleType::WhiteSpace as usize > 0 || class_style & StyleType::WhiteSpace as usize > 0 {
			other_layout_styles[id].flex_wrap = if text.white_space.allow_wrap() {
				FlexWrap::Wrap
			} else {
				FlexWrap::NoWrap
			}
		} else if local_style2 & StyleType2::FlexWrap as usize == 0 && class_style2 & StyleType2::FlexWrap as usize == 0{
			// 文字的容器默认flex_wrap为FlexWrap::Wrap
			other_layout_styles[id].flex_wrap = FlexWrap::Wrap;
		}

		// 通知样式脏，才能使布局系统布局文字字符
		other_layout_styles.get_notify().modify_event(id, "justify_content", 0);
	}

	// 简单布局， 将文字劈分，单词节点的内部字符使用绝对布局，其余节点使用相对布局
	// 与图文混排的布局方式不同，该布局不需要为每个字符节点创建实体
	fn cacl_simple(&mut self, node_states: &mut MultiCaseImpl<Node, NodeState>) {
		let (id, text_style) = (self.id, self.text_style);
		node_states[id].0.set_vnode(false);
		
		let chars = &mut node_states[id].0.text;
		let (mut word_index, mut p_x, mut word_margin_start, mut char_index) = (0, 0.0, 0.0, 0);

		if text_style.text.indent > 0.0 {
			self.create_or_get_indice(chars, text_style.text.indent, char_index, -1);
			char_index += 1;
		}

		// 根据每个字符, 创建charNode
		for cr in split(self.text, true, text_style.text.white_space.preserve_spaces()) {
			// println!("cacl_simple, cr: {:?}, char_index:{}, word_index: {}, word_margin_start: {}, p_x:{}", cr, char_index, word_index, word_margin_start, p_x);
			// 如果是单词的结束字符，释放掉当前节点后面的所有兄弟节点， 并将当前节点索引重置为当前节点的父节点的下一个兄弟节点
			match cr {
				SplitResult::Word(char_i,c) => {
					let cn = self.create_or_get(c, chars, char_index, p_x, char_i);
					cn.margin_start = word_margin_start;
					char_index += 1;
					word_margin_start = self.char_margin;
				}
				SplitResult::WordNext(char_i,c) => {
					let cn = self.create_or_get(c, chars, char_index, p_x, char_i);
					cn.context_id = word_index as isize;
					p_x += cn.size.0 + self.char_margin; // 下一个字符的位置
					char_index += 1;
					chars[word_index].ch_id_or_count += 1;
				}
				// 存在WordStart， 表示开始一个多字符单词
				SplitResult::WordStart(char_i,c) => {
					self.create_or_get_container(chars, char_index, word_margin_start, -1);
					word_index = char_index;
					p_x = 0.0;
					word_margin_start = self.char_margin;
					char_index += 1;

					let cn = self.create_or_get(c, chars, char_index, p_x, char_i);
					cn.context_id = word_index as isize;
					p_x += cn.size.0 + self.char_margin; // 下一个字符的位置
					chars[word_index].ch_id_or_count += 1;
					char_index += 1;
				}
				SplitResult::WordEnd(_) => {
					chars[word_index].size = (p_x - self.char_margin, self.line_height);
				},
				SplitResult::Whitespace(char_i) => {
					let cn = self.create_or_get(' ', chars, char_index, p_x, char_i);
					cn.margin_start = word_margin_start;
					char_index += 1;
					word_margin_start = self.char_margin;
					word_margin_start += self.font_size/3.0 + self.word_margin;
					cn.size.0 = self.font_size/3.0;
					// log::info!("space====={:?}, {:?}", self.font_size, self.word_margin);
				}
				SplitResult::Newline(char_i) => {
					self.create_or_get_breakline(chars, char_index, char_i);
					char_index += 1;
				}
			}
		}

		let cur_child = self.idtree[id].children().head;
		if cur_child > 0 {
			free_childs(cur_child, self.idtree, self.nodes);
		}

		while char_index < chars.len() {
			chars.pop();
		}
	}

	// 图文混排布局，由于每个字符需要与文字节点的其它兄弟节点在同一层进行布局，因此，每个字符将被当成一个实体进行布局
	fn calc_mixed(&mut self, node_states: &mut MultiCaseImpl<Node, NodeState>) {
		let (id, text_style) = (self.id, self.text_style);
		let node_state = &mut node_states[id];
		node_state.set_vnode(true);
		node_state.set_line_start_margin_zero(true);

		let parent = id;
		let mut cur_child = self.idtree[id].children().head;
		let (mut word_index, mut p_x, mut word_margin_start, mut word_id, mut char_index) = (0, 0.0, 0.0, 0, 0);
		
		if text_style.text.indent > 0.0 {
			let chars = &mut node_states[id].0.text;
			let cn = self.create_or_get_indice(chars, text_style.text.indent, char_index, -1);
			cur_child = self.create_entity(cur_child, parent, &cn.clone(), word_margin_start, char_index, node_states);
			char_index += 1;
			cur_child = self.idtree[cur_child].next();
		}

		// 根据每个字符, 创建charNode
		for cr in split(self.text, true, text_style.text.white_space.preserve_spaces()) {
			// 如果是单词的结束字符，释放掉当前节点后面的所有兄弟节点， 并将当前节点索引重置为当前节点的父节点的下一个兄弟节点
			match cr {
				SplitResult::Word(char_i, c) => {
					let chars = &mut node_states[id].0.text;
					let cn = self.create_or_get(c, chars, char_index, 0.0, char_i);
					cn.margin_start = word_margin_start;
					word_margin_start = self.char_margin;
					cur_child = self.create_entity(cur_child, parent, &cn.clone(), word_margin_start, char_index, node_states);
					char_index += 1;
				}
				SplitResult::WordNext(char_i, c) => {
					let chars = &mut node_states[id].0.text;
					let cn = self.create_or_get(c, chars, char_index, p_x, char_i);
					cn.context_id = word_index as isize;
					p_x += cn.size.0 + self.char_margin; // 下一个字符的位置
					chars[word_index].ch_id_or_count += 1;
					char_index += 1;
					continue;
				}
				// 存在WordStart， 表示开始一个多字符单词
				SplitResult::WordStart(char_i, c) => {
					// 容器节点
					let cn = self.create_or_get_container(&mut node_states[id].0.text, char_index, word_margin_start, -1);
					cur_child = self.create_entity(cur_child, parent, &cn.clone(), word_margin_start, char_index, node_states);

					word_id = cur_child;
					word_index = char_index;
					p_x = 0.0;
					word_margin_start = self.char_margin;
					char_index += 1;

					let chars = &mut node_states[id].0.text;
					let cn = self.create_or_get(c, chars, char_index, 0.0, char_i);
					cn.context_id = word_index as isize;
					p_x += cn.size.0 + self.char_margin; // 下一个字符的位置
					chars[word_index].ch_id_or_count += 1;
					char_index += 1;
				}
				SplitResult::WordEnd(_) => {
					let chars = &mut node_states[id].0.text;
					self.rect_layout_styles[word_id].size = Size{
						width: Dimension::Points(p_x - self.char_margin),
						height: Dimension::Points(self.line_height),
					};
					chars[word_index].size = (p_x - self.char_margin, self.line_height);
					continue;
				},
				SplitResult::Whitespace(char_i) => {
					let chars = &mut node_states[id].0.text;
					let cn = self.create_or_get(' ', chars, char_index, 0.0, char_i);
					cn.margin_start = word_margin_start;
					word_margin_start = self.char_margin;
					cur_child = self.create_entity(cur_child, parent, &cn.clone(), word_margin_start, char_index, node_states);
					char_index += 1;

					// 如果用magine-start来表示空格，会导致行首的空格无效
					word_margin_start += self.font_size/3.0 + self.word_margin;
					// continue;
				}
				SplitResult::Newline(char_i) => {
					let chars = &mut node_states[id].0.text;
					let cn = self.create_or_get_breakline(chars, char_index, char_i);
					cur_child = self.create_entity(cur_child, parent, &cn.clone(), 0.0, char_index, node_states);
					char_index += 1;
				}
			};
			cur_child = self.idtree[cur_child].next();
		}

		if cur_child > 0 {
			free_childs(cur_child, self.idtree, self.nodes);
		}

		let chars = &mut node_states[id].0.text;
		while char_index < chars.len() {
			chars.pop();
		}
	}

	fn create_entity(&mut self, mut id: usize, parent: usize, cn: &CharNode, margin: f32, index: usize, node_states: &mut MultiCaseImpl<Node, NodeState>) -> usize {
		if id == 0 {
			id = self.nodes.create_but_no_notify();
			self.rect_layout_styles.insert_no_notify(id, RectLayoutStyle {
				margin: Rect{
					start: Dimension::Points(margin),
					end: Dimension::Points(0.0),
					top: Dimension::Points(0.0),
					bottom: Dimension::Points(0.0),
				},
				size: Size{
					width: Dimension::Points(cn.size.0),
					height: Dimension::Points(cn.size.1)
				},
			});
			self.layout_rs.insert_no_notify(id, LayoutR::default());
			node_states.insert_no_notify(id, NodeState(INode::new(INodeStateType::SelfDirty, index)));
	
			self.idtree.create(id);
			self.idtree.insert_child(id, parent, std::usize::MAX);
		} else {
			let mut style = &mut self.rect_layout_styles[id];
			style.margin.start = Dimension::Points(margin);
			style.size = Size{
				width: Dimension::Points(cn.size.0),
				height: Dimension::Points(cn.size.1)
			};
			node_states[id].0.char_index = index;
		}
		id
	}

	fn create_char_node(&mut self, ch: char, p_x: f32, char_i: isize) -> CharNode {
		let r = self.font_sheet.measure(
			&self.tex_font.0,
			self.font_size as usize,
			self.sw as usize,
			self.text_style.font.weight,
			ch,
		);

		CharNode {
			ch,
			size: (r.0, self.line_height),
			margin_start: self.char_margin,
			pos: (p_x, 0.0),
			base_width: r.1,
			ch_id_or_count: 0,
			char_i,
			context_id: -1,
		}
	}

	fn create_or_get<'b>(&mut self, ch: char, chars: &'b mut Vec<CharNode>, index: usize, p_x: f32, char_i: isize) -> &'b mut CharNode {
		if index >= chars.len() {
			chars.push(self.create_char_node(ch, p_x, char_i));
		} else {
			let cn = &chars[index];
			if cn.ch != ch {
				chars[index] = self.create_char_node(ch, p_x, char_i);
			}
		}
		let cn = &mut chars[index];
		cn.pos.0 = p_x;
		cn.char_i = char_i;
		cn
	}

	fn create_or_get_container<'b>(&mut self, chars: &'b mut Vec<CharNode>, index: usize, word_margin_start: f32, char_i: isize) -> &'b mut CharNode {
		let r = CharNode {
			ch: char::from(0),
			size: (0.0, self.line_height),
			margin_start: word_margin_start,
			pos: (0.0, 0.0),
			base_width: self.font_size,
			ch_id_or_count: 1,
			char_i,
			context_id: -1,
		};
		if index >= chars.len() {
			chars.push(r);
		} else {
			chars[index] = r;
		}
		&mut chars[index]
	}

	fn create_or_get_breakline<'b>(&mut self, chars: &'b mut Vec<CharNode>, index: usize, char_i: isize) -> &'b mut CharNode {
		if index >= chars.len() {
			chars.push(CharNode {
				ch: '\n',
				size: (0.0, self.line_height),
				margin_start: 0.0,
				pos: (0.0, 0.0),
				base_width: 0.0,
				ch_id_or_count: 0,
				char_i,
				context_id: -1,
			});
		} else {
			let cn = &chars[index];
			if cn.ch != '\n' {
				chars[index] = CharNode {
					ch: '\n',
					size: (0.0, self.line_height),
					margin_start: 0.0,
					pos: (0.0, 0.0),
					base_width: 0.0,
					ch_id_or_count: 0,
					char_i,
					context_id: -1,
				};
			}
		}
		&mut chars[index]
	}

	fn create_or_get_indice<'b>(&mut self, chars: &'b mut Vec<CharNode>, indice: f32, index: usize, char_i: isize) -> &'b mut CharNode {
		if index >= chars.len() {
			chars.push(CharNode {
				ch: ' ',
				size: (indice, self.line_height),
				margin_start: 0.0,
				pos: (0.0, 0.0),
				base_width: 0.0,
				ch_id_or_count: 0,
				char_i,
				context_id: -1,
			});
		} else {
			let cn = &chars[index];
			if cn.ch != ' ' {
				chars[index] = CharNode {
					ch: ' ',
					size: (indice, self.line_height),
					margin_start: 0.0,
					pos: (0.0, 0.0),
					base_width: 0.0,
					ch_id_or_count: 0,
					char_i,
					context_id: -1,
				};
			} else {
				chars[index].size.0 = indice;
			}
		}
		&mut chars[index]
	}
}

fn calc<'a>(
	id: usize,
	(text_content, _class_names, _world_matrixs, style_marks, _dirty_list): &Read,
	(node_states, layout_rs, rect_layout_styles, other_layout_styles, text_styles, font_sheet, idtree, nodes):&mut Write,
	layout_dirty: usize,) {
	let font_sheet = &mut font_sheet.borrow_mut();
	let defaultFamily = text_styles[0].font.family; // 0不存在text_style， 必然取到默认值
	let text_style = &mut text_styles[id];
	let tex_font = match font_sheet.get_font_info(&text_style.font.family) {
		Some(r) => (r.0.clone(), r.1),
		None => {
			log::debug!("font is not exist, face_name: {:?}, id: {:?}, will use default font: {:?}",
			text_style.font.family,
			id,
			defaultFamily);

			text_style.font.family = defaultFamily;
			match font_sheet.get_font_info(&text_style.font.family) {
				Some(r) => (r.0.clone(), r.1),
				None => {
					log::debug!("default font is not exist, face_name: {:?}, id: {:?}",
					text_style.font.family,
					id);
					return
				}
			}
		}
	};
	let font_size = get_size(tex_font.1, &text_style.font.size) as f32;
	let font_height = font_sheet.get_font_height(&tex_font
		.0, font_size as usize, text_style.text.stroke.width);
	let sw = text_style.text.stroke.width;
	let parent = idtree[id].parent();
	let mut calc = Calc {
		text: match text_content.get(id) {
			Some(t) => t.0.as_ref(),
			_ => "",
		},
		style_mark: &style_marks[id],
		tex_font: (tex_font.0.clone(), tex_font.1),
		font_size,
		font_height,
		line_height: get_line_height(font_height as usize, &text_style.text.line_height),
		sw: text_style.text.stroke.width,
		char_margin: text_style.text.letter_spacing - sw,
		word_margin: text_style.text.word_spacing - sw,
		text_style: &mut text_styles[id],
		parent: parent,

		id,
		text_content,
		style_marks,
		layout_rs,
		rect_layout_styles,
		other_layout_styles,
		nodes,
		font_sheet,
		idtree,
	};


	calc.fit_text_style();
	if layout_dirty > 0 {
		// 如果布局属性修改，清除CharNode
		node_states[id].0.text.clear();
	}
	
	let size = &calc.rect_layout_styles[id].size;
	let position_type = calc.other_layout_styles[id].position_type;
	// 如果父节点没有其它子节点，或者，自身定义了宽度或高度，则可使用简单布局
	if parent > 0 && calc.idtree[parent].children().len == 1 {
		// if size.width == Dimension::Undefined {
		// 	size.width = Dimension::Percent(1.0);
		// }
		// if size.height == Dimension::Undefined {
		// 	size.height = Dimension::Percent(1.0);
		// }
		calc.cacl_simple(node_states);
	} else if size.width != Dimension::Undefined || size.height != Dimension::Undefined || position_type == PositionType::Absolute {
		calc.cacl_simple(node_states);
	}else {
		calc.calc_mixed(node_states);
	}
}

fn free_childs(mut start: usize, idtree: &mut SingleCaseImpl<IdTree>, nodes: &mut EntityImpl<Node>) {
	while start > 0 {
		// println!("free_childs text================={}", start);
		let n = idtree[start].next();
		// let notify = idtree.get_notify();
		// idtree.remove_with_notify(start, &notify);
		idtree.destroy(start);
		nodes.delete(start);
		start = n;
	}
}

impl_system! {
    LayoutImpl,
    true,
    {
    }
}

impl_system! {
    TextLayoutUpdateSys,
    true,
    {
		MultiCaseListener<Node, LayoutR, ModifyEvent>
    }
}

impl_system! {
    TextGlphySys<C> where [C: HalContext + 'static],
    true,
    {
    }
}
