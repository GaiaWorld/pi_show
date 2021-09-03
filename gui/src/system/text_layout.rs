// 文字布局及布局系统
// 文本节点的布局算法： 文本节点本身所对应的yoga节点总是一个0大小的节点。文本节点的父节点才是进行文本布局的节点，称为P节点。P节点如果没有设置布局，则默认用flex布局模拟文档流布局。会将文本拆成每个字（英文为单词）的yoga节点加入P节点上。这样可以支持图文混排。P节点如果有flex布局，则遵循该布局。
// 字节点，根据字符是否为单字决定是需要字符容器还是单字。
// 文字根据样式，会处理：缩进，是否合并空白符，是否自动换行，是否允许换行符。来设置相应的flex布局。 换行符采用高度为0, 宽度100%的yoga节点来模拟。
use std::{result::Result};
use bevy_ecs::prelude::{Commands, Entity, EventReader, EventWriter, Query, Res, ResMut, With};
use ordered_float::OrderedFloat;

use share::Share;

use flex_layout::*;
use hal_core::*;
use flex_layout::{Dimension, INodeStateType};
use crate::component::{calc::*, user::*, calc::LayoutR};
use crate::font::font_sheet::{get_line_height, get_size, split, FontSheet, SplitResult, TexFont};
use crate::single::*;
use crate::render::engine::ShareEngine;
use crate::util::cell::StdCell;
use crate::util::event::EntityEvent;

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

// type Read<'a> = (
//     &'a MultiCaseImpl<Node, TextContent>,
//     &'a MultiCaseImpl<Node, ClassName>,
//     &'a MultiCaseImpl<Node, WorldMatrix>,
//     &'a MultiCaseImpl<Node, StyleMark>,
//     &'a SingleCaseImpl<DirtyList>,
// );
// type Write<'a> = (
//     &'a mut MultiCaseImpl<Node, NodeState>, // TODO
// 	&'a mut MultiCaseImpl<Node, LayoutR>,
// 	&'a mut MultiCaseImpl<Node, RectLayoutStyle>,
// 	&'a mut MultiCaseImpl<Node, OtherLayoutStyle>,
// 	&'a mut MultiCaseImpl<Node, TextStyle>,
// 	&'a mut SingleCaseImpl<Share<StdCell<FontSheet>>>,
// 	&'a mut SingleCaseImpl<IdTree>,
// 	&'a mut EntityImpl<Node>,
// );

// id: Entity,
// 	layout_dirty: usize,
// 	mut commands: &mut Commands,
// 	query: &Query<(
// 		&TextContent,
// 		&ClassName,
// 		&WorldMatrix,
// 		&mut NodeState,
// 		&mut LayoutR,
// 		&mut TextStyle), With<TextContent>>,
// 	rect_layout_styles: &mut Query<&mut RectLayoutStyle>,
// 	node_states: &mut Query<&mut NodeState>,
// 	style_mark: &StyleMark,
// 	dirty_list: &Res<DirtyList>,
// 	idtree: &mut ResMut<IdTree>,
// 	defualt_text_style: &Res<TextStyle>,
// 	font_sheet: &mut ResMut<Share<StdCell<FontSheet>>>

/// 生成文字布局对象
pub fn gen_text_layout_obj<'a>(
	mut commands: Commands<'a>,
	mut query: Query<'a, (
		&'a TextContent,
		&'a mut NodeState,
		&'a mut TextStyle), With<TextContent>>,
	mut rect_layout_styles: Query<'a, &'a mut RectLayoutStyle>,
	mut other_layout_styles: Query<'a, &'a mut OtherLayoutStyle>,
	mut node_states: Query<'a, &'a mut NodeState>,
	defualt_text_style: Res<'a, TextStyle>,
	style_mark_query: Query<'a, &'a StyleMark>,
	dirty_list: Res<'a, DirtyList>,
	mut idtree: ResMut<'a, IdTree>,
	mut font_sheet: ResMut<'a, Share<StdCell<FontSheet>>>,
	mut other_layout_styles_writer: EventWriter<'a, EntityEvent<OtherLayoutStyle>>,
) {
	// 暂时拷贝， TODO
	let dirty_list = dirty_list.0.clone();
	// let time = std::time::Instant::now();
	for id in dirty_list.iter() {
		if let Ok(style_mark) = style_mark_query.get(*id) {
			if (style_mark.dirty & MARK == 0) || idtree.get(id.id() as usize).is_none(){
				continue;
			}
			// println!("text dirty===================textContent dirty{:?}, layout_dirty:{}, dirty:{}, id:{}", r.dirty & StyleType::Text as usize, r.dirty & MARK_LAYOUT, r.dirty, id);
			calc(*id, style_mark.dirty & MARK_LAYOUT, &mut commands, &mut query, &mut rect_layout_styles,&mut other_layout_styles, &mut other_layout_styles_writer, &mut node_states, style_mark, &mut idtree, &defualt_text_style, &mut font_sheet);
		};
	}
}

pub fn update_text_glphy<C: HalContext + 'static>(
	mut query: Query<(
		&WorldMatrix,
		&mut NodeState,
		&mut TextStyle)>,
	style_mark_query: Query<&StyleMark>,
	text_query: Query<Option<&TextContent>>,
	dirty_list: Res<DirtyList>,
	idtree: Res<IdTree>,
	mut font_sheet: ResMut<Share<StdCell<FontSheet>>>,
	engine: Res<ShareEngine<C>>,
	render_begin: Res<RenderBegin>,
	mut text_content_event_writer: EventWriter<EntityEvent<TextContent>>
) {
	let mut flag = true;
	let mut count = 0;
	while flag {
		flag = false;
		count += 1;
		if count > 2 { // 迭代了两次以上，则可能进入了死循环，报错
			log::info!("TextGlphySys 死循环, 当前纹理尺寸无法缓存现有的文字");
			panic!("TextGlphySys 死循环");
		}
		for id in dirty_list.0.iter() {
			if let Ok(style_mark) = style_mark_query.get(*id) {
				if (style_mark.dirty & MARK == 0) || idtree.get(id.id() as usize).is_none(){
					continue;
				}

				match set_gylph(*id, &mut query, &mut font_sheet) {
					Result::Err(_message) => {	
						log::info!("textTexture flow, reset textTexture");
						// panic!("err:{:?}", message);
						let mut font_sheet = font_sheet.borrow_mut();
						font_sheet.clear_gylph();
	
						// // 纹理清空为蓝色
						let (width, height) = (font_sheet.font_tex.texture.width, font_sheet.font_tex.texture.height);
						// let mut vec = Vec::with_capacity(width * height * 4);
						// for _a in 0..width * height {
						// 	vec.push(0);
						// 	vec.push(0);
						// 	vec.push(1);
						// 	vec.push(1);
						// }
						// read.5.gl.texture_update(&font_sheet.font_tex.texture.bind, 0, &TextureData::U8(0, 0, width as u32, height as u32, vec.as_slice()));
	
						let target = engine.gl.rt_create(
							Some(&font_sheet.font_tex.texture.bind),
							width as u32,
							height as u32,
							PixelFormat::RGBA,
							DataFormat::UnsignedByte,
							false,
						)
						.unwrap();
						
						engine.gl.render_begin(Some(&target), &RenderBeginDesc{
							viewport: (0,0,width as i32,height as i32),
							scissor: (0,0,width as i32,height as i32),
							clear_color: Some((OrderedFloat(0.0), OrderedFloat(0.0), OrderedFloat(1.0), OrderedFloat(1.0))),
							clear_depth: render_begin.0.clear_depth.clone(),
							clear_stencil: render_begin.0.clear_stencil.clone(),
						});
	
						engine.gl.render_end();
						
						// 对界面上的文字全部重新计算字形
						let root = &idtree[1];
						for (id, node) in idtree.recursive_iter(root.children().head) {
							let entity = to_entity(id, node.data);
							if text_query.get(entity).unwrap().is_some() { // 文字节点，发送修改事件
								text_content_event_writer.send(EntityEvent::new_modify(entity, StyleIndex::Text));
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

/// 布局修改
pub fn text_layout_update(
	mut query: Query<(
		&mut NodeState,
		&mut LayoutR)>,
	mut node_states: Query<&mut NodeState>,
	idtree: Res<IdTree>,
	mut layout_reader: EventReader<EntityEvent<LayoutR>>,
) {
	for e in layout_reader.iter() {
		let r = node_states.get_mut(e.id);
		if let Err(_) = r {
			continue;
		}
		let mut node_state = r.unwrap();

		let n = &idtree[e.id.id()as usize];
		let chars = &mut node_state.text;
		let mut rect = Rect{
			start: std::f32::MAX,
			end: 0.0,
			top: std::f32::MAX,
			bottom: 0.0,
		};
		for (id, node) in idtree.recursive_iter(n.children().head) {
			let entity = to_entity(id, node.data);
			if let Ok((node_state, layout_r)) = query.get_mut(entity) {
				let char = &mut chars[node_state.char_index];
				let l = &layout_r.rect;
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
		}

		if let Ok((mut _node_state, mut layout_r)) = query.get_mut(e.id) {
			// 设置文字虚拟节点的矩形
			if rect.start <= rect.end {
				layout_r.rect = rect;
			}
		}
	}
}

// 设置字形的id
fn set_gylph<'a>(
	id: Entity, 
	query: &mut Query<(
		&WorldMatrix,
		&mut NodeState,
		&mut TextStyle)>,
	font_sheet: &mut ResMut<Share<StdCell<FontSheet>>>) -> Result<(), String> {
	
	if let Ok((
		world_matrix,
		mut node_state,
		text_style)) = query.get_mut(id) {
		let scale = Vector4::from(world_matrix.fixed_columns(1));
		let scale = scale.dot(&scale).sqrt();
		let font_sheet = &mut font_sheet.borrow_mut();
		
		let (tex_font, font_size) = match font_sheet.get_font_info(&text_style.font.family) {
			Some(r) => (r.0.clone(), get_size(r.1, &text_style.font.size) as f32),
			None => {
				log::info!("font is not exist, face_name: {:?}, id: {:?}",
				text_style.font.family,
				id);
				return Ok(());
			}
		};

		let weight = text_style.font.weight;
		let sw = text_style.text.stroke.width;

		node_state.scale = scale;
		let chars = &mut node_state.text;
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
	}

	return Ok(())
}

struct Calc<'a, 'b> {
	id: Entity,
	commands: &'b mut Commands<'a>,
	idtree: &'b mut ResMut<'a , IdTree>,
	font_sheet: &'b mut FontSheet,
	font_weight: usize,
	node_states: &'b mut Query<'a, &'a mut NodeState>,
	rect_layout_styles: &'b mut Query<'a, &'a mut RectLayoutStyle>,
	other_layout_styles: &'b mut Query<'a, &'a mut OtherLayoutStyle>,
	other_layout_styles_writer: &'b mut EventWriter<'a, EntityEvent<OtherLayoutStyle>>,

	text: &'b str,
	style_mark: &'b StyleMark,
	tex_font: (TexFont, usize),
	font_size: f32,
	font_height: f32,
	line_height: f32,
	sw: f32,
	char_margin: f32,
	word_margin: f32,
	// text_style: &'b TextStyle,
	// node_state: &'b NodeState,
	parent: usize,
}

impl<'a, 'c> Calc<'a, 'c> {
	// 将文字样式用flex布局属性替换, 可以考虑不支持文字的布局属性？
	fn fit_text_style(&mut self, text_style: &TextStyle) {
		let (local_style, class_style, local_style2, class_style2, text, other_layout_styles, id) = (self.style_mark.local_style, self.style_mark.class_style, self.style_mark.local_style2, self.style_mark.class_style2, &text_style.text, &mut self.other_layout_styles, self.id);
		let mut other_layout_style = other_layout_styles.get_mut(id).unwrap();
		// 兼容目前使用父节点的对齐属性来对齐文本， 如果项目将其修改正确， 应该去掉该段TODO
		if local_style & StyleType::TextAlign as usize > 0 || class_style & StyleType::TextAlign as usize > 0 {
			other_layout_style.justify_content = match text.text_align {
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
			other_layout_style.align_items = r;
			let r= match text.vertical_align {
				VerticalAlign::Middle => AlignContent::Center,
				VerticalAlign::Bottom => AlignContent::FlexEnd,
				VerticalAlign::Top => AlignContent::FlexStart
			};
			other_layout_style.align_content = r;
		
		} else if local_style2 & StyleType2::AlignContent as usize == 0 && class_style2 & StyleType2::AlignContent as usize == 0 {
			// 文字的容器默认align_content为FlexStart
			other_layout_style.align_content = AlignContent::FlexStart;
		}
	
		if local_style & StyleType::WhiteSpace as usize > 0 || class_style & StyleType::WhiteSpace as usize > 0 {
			other_layout_style.flex_wrap = if text.white_space.allow_wrap() {
				FlexWrap::Wrap
			} else {
				FlexWrap::NoWrap
			}
		} else if local_style2 & StyleType2::FlexWrap as usize == 0 && class_style2 & StyleType2::FlexWrap as usize == 0{
			// 文字的容器默认flex_wrap为FlexWrap::Wrap
			other_layout_style.flex_wrap = FlexWrap::Wrap;
		}

		// 通知样式脏，才能使布局系统布局文字字符
		self.other_layout_styles_writer.send(EntityEvent::new_modify(id, StyleIndex::JustifyContent));
		// other_layout_styles.get_notify().modify_event(id, "justify_content", 0);
	}

	// 简单布局， 将文字劈分，单词节点的内部字符使用绝对布局，其余节点使用相对布局
	// 与图文混排的布局方式不同，该布局不需要为每个字符节点创建实体
	fn cacl_simple(&mut self, node_state: &mut NodeState, text_style: &TextStyle) {
		let id = self.id;
		node_state.set_vnode(false);
		
		let chars = &mut node_state.text;
		let (mut word_index, mut p_x, mut word_margin_start, mut char_index) = (0, 0.0, 0.0, 0);

		if text_style.text.indent > 0.0 {
			self.create_or_get_indice(chars, text_style.text.indent, char_index);
			char_index += 1;
		}

		// 根据每个字符, 创建charNode
		for cr in split(self.text, true, text_style.text.white_space.preserve_spaces()) {
			// println!("cacl_simple, cr: {:?}, char_index:{}, word_index: {}, word_margin_start: {}, p_x:{}", cr, char_index, word_index, word_margin_start, p_x);
			// 如果是单词的结束字符，释放掉当前节点后面的所有兄弟节点， 并将当前节点索引重置为当前节点的父节点的下一个兄弟节点
			match cr {
				SplitResult::Word(c) => {
					let cn = self.create_or_get(c, chars, char_index, p_x);
					cn.margin_start = word_margin_start;
					char_index += 1;
					word_margin_start = self.char_margin;
				}
				SplitResult::WordNext(c) => {
					let cn = self.create_or_get(c, chars, char_index, p_x);
					
					p_x += cn.size.0 + self.char_margin; // 下一个字符的位置
					char_index += 1;
					chars[word_index].ch_id_or_count += 1;
				}
				// 存在WordStart， 表示开始一个多字符单词
				SplitResult::WordStart(c) => {
					self.create_or_get_container(chars, char_index, word_margin_start);
					word_index = char_index;
					p_x = 0.0;
					word_margin_start = self.char_margin;
					char_index += 1;

					let cn = self.create_or_get(c, chars, char_index, p_x);
					p_x += cn.size.0 + self.char_margin; // 下一个字符的位置
					chars[word_index].ch_id_or_count += 1;
					char_index += 1;
				}
				SplitResult::WordEnd => {
					chars[word_index].size = (p_x - self.char_margin, self.line_height);
				},
				SplitResult::Whitespace => {
					let cn = self.create_or_get(' ', chars, char_index, p_x);
					cn.margin_start = word_margin_start;
					char_index += 1;
					word_margin_start = self.char_margin;
					// word_margin_start += self.font_size/3.0 + self.word_margin;
				}
				SplitResult::Newline => {
					self.create_or_get_breakline(chars, char_index);
					char_index += 1;
				}
			}
		}

		let cur_child = self.idtree[id.id() as usize].children().head;
		if cur_child < usize::max_value() {
			let n = &self.idtree[cur_child];
			free_childs(to_entity(cur_child, n.data), self.idtree, &mut self.commands);
		}

		while char_index < chars.len() {
			chars.pop();
		}
	}

	// 图文混排布局，由于每个字符需要与文字节点的其它兄弟节点在同一层进行布局，因此，每个字符将被当成一个实体进行布局
	fn calc_mixed(&mut self, node_state: &mut NodeState, text_style: &TextStyle) {
		let id = self.id;
		node_state.set_vnode(true);
		node_state.set_line_start_margin_zero(true);

		let parent = id;
		let mut child_id = self.idtree[id.id() as usize].children().head;
		let mut child_node = &self.idtree[child_id];
		let mut cur_child = to_entity(child_id, child_node.data);
		let (mut word_index, mut p_x, mut word_margin_start, mut word_id, mut char_index) = (0, 0.0 ,0.0, Entity::new(std::u32::MAX), 0);
		
		let chars = &mut node_state.text;
		if text_style.text.indent > 0.0 {
			let cn = self.create_or_get_indice(chars, text_style.text.indent, char_index);
			cur_child = self.create_entity(cur_child, parent, &cn.clone(), word_margin_start, char_index);
			char_index += 1;
			child_id = self.idtree[cur_child.id() as usize].next();
			child_node = &self.idtree[child_id];
			cur_child = to_entity(child_id, child_node.data);
		}

		// 根据每个字符, 创建charNode
		for cr in split(self.text, true, text_style.text.white_space.preserve_spaces()) {
			// 如果是单词的结束字符，释放掉当前节点后面的所有兄弟节点， 并将当前节点索引重置为当前节点的父节点的下一个兄弟节点
			match cr {
				SplitResult::Word(c) => {
					let cn = self.create_or_get(c, chars, char_index, 0.0);
					cn.margin_start = word_margin_start;
					word_margin_start = self.char_margin;
					cur_child = self.create_entity(cur_child, parent, &cn.clone(), word_margin_start, char_index);
					char_index += 1;
				}
				SplitResult::WordNext(c) => {
					let cn = self.create_or_get(c, chars, char_index, p_x);
					p_x += cn.size.0 + self.char_margin; // 下一个字符的位置
					chars[word_index].ch_id_or_count += 1;
					char_index += 1;
					continue;
				}
				// 存在WordStart， 表示开始一个多字符单词
				SplitResult::WordStart(c) => {
					// 容器节点
					let cn = self.create_or_get_container(chars, char_index, word_margin_start);
					cur_child = self.create_entity(cur_child, parent, &cn.clone(), word_margin_start, char_index);

					word_id = cur_child;
					word_index = char_index;
					p_x = 0.0;
					word_margin_start = self.char_margin;
					char_index += 1;

					let cn = self.create_or_get(c, chars, char_index, 0.0);
					p_x += cn.size.0 + self.char_margin; // 下一个字符的位置
					chars[word_index].ch_id_or_count += 1;
					char_index += 1;
				}
				SplitResult::WordEnd => {
					let mut rect_layout_style = self.rect_layout_styles.get_mut(word_id).unwrap();
					rect_layout_style.size = Size{
						width: Dimension::Points(p_x - self.char_margin),
						height: Dimension::Points(self.line_height),
					};
					chars[word_index].size = (p_x - self.char_margin, self.line_height);
					continue;
				},
				SplitResult::Whitespace => {
					let cn = self.create_or_get(' ', chars, char_index, 0.0);
					cn.margin_start = word_margin_start;
					word_margin_start = self.char_margin;
					cur_child = self.create_entity(cur_child, parent, &cn.clone(), word_margin_start, char_index,);
					char_index += 1;

					// 如果用magine-start来表示空格，会导致行首的空格无效
					// word_margin_start += self.font_size/3.0 + self.word_margin;
					// continue;
				}
				SplitResult::Newline => {
					let cn = self.create_or_get_breakline(chars, char_index);
					cur_child = self.create_entity(cur_child, parent, &cn.clone(), 0.0, char_index);
					char_index += 1;
				}
			};
			child_id = self.idtree[cur_child.id() as usize].next();
			child_node = &self.idtree[child_id];
			cur_child = to_entity(child_id, child_node.data);
		}

		if cur_child.id() != std::u32::MAX {
			free_childs(cur_child, self.idtree, self.commands);
		}

		while char_index < chars.len() {
			chars.pop();
		}
	}

	fn create_entity(&mut self, mut entity: Entity, parent: Entity, cn: &CharNode, margin: f32, index: usize) -> Entity {
		if let Ok(mut style) = self.rect_layout_styles.get_mut(entity) {
			style.margin.start = Dimension::Points(margin);
			style.size = Size{
				width: Dimension::Points(cn.size.0),
				height: Dimension::Points(cn.size.1)
			};
			self.node_states.get_mut(entity).unwrap().char_index = index;
		} else{
			entity = self.commands.spawn_bundle((RectLayoutStyle {
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
			}, 
			LayoutR::default(),
			NodeState::new(INodeStateType::SelfDirty, index))).id();
			// id = self.nodes.create_but_no_notify();
			// self.rect_layout_styles.insert_no_notify(id, RectLayoutStyle {
			// 	margin: Rect{
			// 		start: Dimension::Points(margin),
			// 		end: Dimension::Points(0.0),
			// 		top: Dimension::Points(0.0),
			// 		bottom: Dimension::Points(0.0),
			// 	},
			// 	size: Size{
			// 		width: Dimension::Points(cn.size.0),
			// 		height: Dimension::Points(cn.size.1)
			// 	},
			// });
			// self.layout_rs.insert_no_notify(id, LayoutR::default());
			// node_states.insert_no_notify(id, NodeState(INode::new(INodeStateType::SelfDirty, index)));
			
			self.idtree.create(entity.id() as usize, entity.generation());
			self.idtree.insert_child(entity.id() as usize, parent.id() as usize, std::usize::MAX);
		}
		entity
	}

	fn create_char_node(&mut self, ch: char, p_x: f32) -> CharNode {
		let r = self.font_sheet.measure(
			&self.tex_font.0,
			self.font_size as usize,
			self.sw as usize,
			self.font_weight,
			ch,
		);

		CharNode {
			ch,
			size: (r.0, self.line_height),
			margin_start: self.char_margin,
			pos: (p_x, 0.0),
			base_width: r.1,
			ch_id_or_count: 0,
		}
	}

	fn create_or_get<'b>(&mut self, ch: char, chars: &'b mut Vec<CharNode>, index: usize, p_x: f32) -> &'b mut CharNode {
		if index >= chars.len() {
			chars.push(self.create_char_node(ch, p_x));
		} else {
			let cn = &chars[index];
			if cn.ch != ch {
				chars[index] = self.create_char_node(ch, p_x);
			}
		}
		let cn = &mut chars[index];
		cn.pos.0 = p_x;
		cn
	}

	fn create_or_get_container<'b>(&mut self, chars: &'b mut Vec<CharNode>, index: usize, word_margin_start: f32) -> &'b mut CharNode {
		let r = CharNode {
			ch: char::from(0),
			size: (0.0, self.line_height),
			margin_start: word_margin_start,
			pos: (0.0, 0.0),
			base_width: self.font_size,
			ch_id_or_count: 1,
		};
		if index >= chars.len() {
			chars.push(r);
		} else {
			chars[index] = r;
		}
		&mut chars[index]
	}

	fn create_or_get_breakline<'b>(&mut self, chars: &'b mut Vec<CharNode>, index: usize) -> &'b mut CharNode {
		if index >= chars.len() {
			chars.push(CharNode {
				ch: '\n',
				size: (0.0, self.line_height),
				margin_start: 0.0,
				pos: (0.0, 0.0),
				base_width: 0.0,
				ch_id_or_count: 0,
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
				};
			}
		}
		&mut chars[index]
	}

	fn create_or_get_indice<'b>(&mut self, chars: &'b mut Vec<CharNode>, indice: f32, index: usize) -> &'b mut CharNode {
		if index >= chars.len() {
			chars.push(CharNode {
				ch: ' ',
				size: (indice, self.line_height),
				margin_start: 0.0,
				pos: (0.0, 0.0),
				base_width: 0.0,
				ch_id_or_count: 0,
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
				};
			} else {
				chars[index].size.0 = indice;
			}
		}
		&mut chars[index]
	}
}

fn calc<'a, 'b>(
	id: Entity,
	layout_dirty: usize,
	commands: &'a mut Commands<'b>,
	query: &'a mut Query<'b, (
		&'b TextContent,
		&'b mut NodeState,
		&'b mut TextStyle), With<TextContent>>,
	rect_layout_styles: &'a mut Query<'b, &'b mut RectLayoutStyle>,
	other_layout_styles: &'a mut Query<'b, &'b mut OtherLayoutStyle>,
	other_layout_styles_writer: &'a mut EventWriter<'b, EntityEvent<OtherLayoutStyle>>,
	node_states: &'a mut Query<'b, &'b mut NodeState>,
	style_mark: &'a StyleMark,
	idtree: &'a mut ResMut<'b, IdTree>,
	defualt_text_style: &'a Res<'b, TextStyle>,
	font_sheet: &'a mut ResMut<'b, Share<StdCell<FontSheet>>>) {
		
	let (text_content, 
		mut node_state, 
		mut text_style) = query.get_mut(id).unwrap();
	let mut font_sheet = font_sheet.borrow_mut();
	let defaultFamily = defualt_text_style.font.family; // 0不存在text_style， 必然取到默认值

	let tex_font = match font_sheet.get_font_info(&text_style.font.family) {
		Some(r) => (r.0.clone(), r.1),
		None => {
			log::info!("font is not exist, face_name: {:?}, id: {:?}, will use default font: {:?}",
			text_style.font.family,
			id,
			defaultFamily);

			text_style.font.family = defaultFamily;
			match font_sheet.get_font_info(&text_style.font.family) {
				Some(r) => (r.0.clone(), r.1),
				None => {
					log::info!("默认字体 is not exist, face_name: {:?}, id: {:?}",
					text_style.font.family,
					id);
					return
				}
			}
		}
	};
	let font_size = get_size(tex_font.1, &text_style.font.size) as f32;
	let font_height = tex_font
		.0
		.get_font_height(font_size as usize, text_style.text.stroke.width);
	let sw = text_style.text.stroke.width;
	let parent = idtree[id.id() as usize].parent();
	let mut calc = Calc {
		text: text_content.0.as_ref(),
		style_mark: style_mark,
		tex_font,
		font_size,
		font_height,
		line_height: get_line_height(font_height as usize, &text_style.text.line_height),
		sw: text_style.text.stroke.width,
		char_margin: text_style.text.letter_spacing - sw,
		word_margin: text_style.text.word_spacing - sw,
		font_weight: text_style.font.weight,
		// text_style: &text_style,
		parent: parent,
		// query: query,
		rect_layout_styles: rect_layout_styles,
		node_states: node_states,

		commands: commands,
		other_layout_styles: other_layout_styles,
		other_layout_styles_writer: other_layout_styles_writer,

		id,
		// text_content,
		// style_marks,
		// layout_rs,
		// rect_layout_styles,
		// other_layout_styles,
		// nodes,
		font_sheet: &mut font_sheet,
		idtree,
	};


	calc.fit_text_style(&text_style);
	if layout_dirty > 0 {
		// 如果布局属性修改，清除CharNode
		node_state.text.clear();
	}
	
	let size = &calc.rect_layout_styles.get_mut(id).unwrap().size;
	let position_type = calc.other_layout_styles.get_mut(id).unwrap().position_type;
	// 如果父节点没有其它子节点，或者，自身定义了宽度或高度，则可使用简单布局
	if parent < usize::max_value() && calc.idtree[parent].children().len == 1 {
		// if size.width == Dimension::Undefined {
		// 	size.width = Dimension::Percent(1.0);
		// }
		// if size.height == Dimension::Undefined {
		// 	size.height = Dimension::Percent(1.0);
		// }
		calc.cacl_simple(&mut node_state, &text_style);
	} else if size.width != Dimension::Undefined || size.height != Dimension::Undefined || position_type == PositionType::Absolute {
		calc.cacl_simple(&mut node_state, &text_style);
	}else {
		calc.calc_mixed(&mut node_state, &text_style);
	}
}

fn free_childs(mut start: Entity, idtree: &mut ResMut<IdTree>, commands: &mut Commands) {
	while start.id() != std::u32::MAX && start.id() < usize::max_value() {
		// println!("free_childs text================={}", start);
		// let notify = idtree.get_notify();
		// idtree.remove_with_notify(start, &notify);
		idtree.destroy(start.id() as usize);
		commands.entity(start).despawn();
		let n_id = idtree[start.id() as usize].next();
		let n = &idtree[n_id];
		start = to_entity(n_id, n.data);
	}
}
