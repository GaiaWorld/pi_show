// 文字布局及布局系统
// 文本节点的布局算法： 文本节点本身所对应的yoga节点总是一个0大小的节点。文本节点的父节点才是进行文本布局的节点，称为P节点。P节点如果没有设置布局，则默认用flex布局模拟文档流布局。会将文本拆成每个字（英文为单词）的yoga节点加入P节点上。这样可以支持图文混排。P节点如果有flex布局，则遵循该布局。
// 字节点，根据字符是否为单字决定是需要字符容器还是单字。
// 文字根据样式，会处理：缩进，是否合并空白符，是否自动换行，是否允许换行符。来设置相应的flex布局。 换行符采用高度为0, 宽度100%的yoga节点来模拟。
use cgmath::InnerSpace;

use ecs::{
	component::MultiCaseImpl,
	entity::EntityImpl,
    monitor::{CreateEvent},
    single::SingleCaseImpl,
    system::{MultiCaseListener, Runner},
};

use flex_layout::*;
use flex_layout::{Dimension, INodeStateType};
use component::{calc::*, user::*, calc::LayoutR};
use entity::Node;
use font::font_sheet::{get_line_height, get_size, split, FontSheet, SplitResult};
use single::class::*;
use single::*;

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
    &'a SingleCaseImpl<ClassSheet>,
    &'a MultiCaseImpl<Node, TextContent>,
    &'a MultiCaseImpl<Node, ClassName>,
    &'a MultiCaseImpl<Node, WorldMatrix>,
    &'a MultiCaseImpl<Node, StyleMark>,
    &'a SingleCaseImpl<DirtyList>,
);
type Write<'a> = (
    &'a mut MultiCaseImpl<Node, CharNode>,
	&'a mut MultiCaseImpl<Node, LayoutR>,
	&'a mut MultiCaseImpl<Node, RectLayoutStyle>,
	&'a mut MultiCaseImpl<Node, OtherLayoutStyle>,
	&'a mut MultiCaseImpl<Node, TextStyle>,
	&'a mut SingleCaseImpl<FontSheet>,
	&'a mut SingleCaseImpl<IdTree>,
	&'a mut EntityImpl<Node>,
	&'a mut MultiCaseImpl<Node, NodeState>,
);

pub struct LayoutImpl {
    read: usize,
    write: usize,
}

pub struct TextGlphySys;

impl<'a> Runner<'a> for TextGlphySys {
    type ReadData = Read<'a>;
    type WriteData = Write<'a>;

    fn run(&mut self, read: Self::ReadData, mut write: Self::WriteData) {
        for id in (read.5).0.iter() {
            let r = match read.4.get(*id) {
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

impl<'a> LayoutImpl {
    pub fn new() -> Self {
        LayoutImpl {
            read: 0,
            write: 0,
        }
    }
}

// node.data.state.vnode_true()

impl<'a> MultiCaseListener<'a, Node, TextContent, CreateEvent> for LayoutImpl {
	type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, NodeState>, &'a mut MultiCaseImpl<Node, TextStyle>);
    fn listen(&mut self, event: &CreateEvent, _: Self::ReadData, (node_states, text_styles): Self::WriteData) {
		node_states[event.id].0.set_vnode(true);
		// 如果不存在TextStyle， 默认插入
		if let None = text_styles.get(event.id) {
			text_styles.insert_no_notify(event.id, TextStyle::default());
		}
    }
}
impl<'a> Runner<'a> for LayoutImpl {
    type ReadData = Read<'a>;
    type WriteData = Write<'a>;
	
    fn run(&mut self, read: Self::ReadData, mut write: Self::WriteData) {
		
		// 暂时拷贝， TODO
		let dirty_list = (read.5).0.clone();
		let time = std::time::Instant::now();
        for id in dirty_list.iter() {
            let r = match read.4.get(*id) {
                Some(r) => r,
                None => continue,
			};

			
            if r.dirty & MARK == 0 {
                continue;
			}
			// println!("text dirty===================textContent dirty{:?}, layout_dirty:{}, dirty:{}, id:{}", r.dirty & StyleType::Text as usize, r.dirty & MARK_LAYOUT, r.dirty, id);
            calc(*id, &read, &mut write, r.dirty & MARK_LAYOUT);
		}
		// if dirty_list.len() > 0 {
		// 	println!(
		// 		"text layout=================={:?}, len:{}, dirty_list_len:{}",
		// 		std::time::Instant::now() - time, write.7.len(), dirty_list.len()
		// 	);
		// }
    }
}

// 设置字形的id
fn set_gylph<'a>(
	id: usize, 
	(_class_sheet, _text_contents, _class_names, world_matrixs, _style_marks, _dirty_list): &Read, 
	(char_nodes, _layout_rs, _rect_layout_styles, _other_layout_styles, text_styles, font_sheet, idtree, _nodes, _): &mut Write) {
	let children = idtree[id].children();
    let scale = world_matrixs[id].y.magnitude();
    let text_style = &text_styles[id];
	
    let (tex_font, font_size) = match font_sheet.get_font_info(&text_style.font.family) {
        Some(r) => (r.0.clone(), get_size(r.1, &text_style.font.size) as f32),
        None => {
            println!(
                "font is not exist, face_name: {:?}, id: {:?}",
                text_style.font.family.as_ref(),
                id
            );
            return;
        }
	};

	let weight = text_style.font.weight;
	let sw = text_style.text.stroke.width;

    for n in idtree.recursive_iter(children.head) {
		let char_node = &mut char_nodes[n.0];
        if char_node.ch > ' ' {
            char_node.ch_id_or_count = font_sheet.calc_gylph(
                &tex_font,
                font_size as usize,
                sw as usize,
                weight,
                scale,
                char_node.base_width,
                char_node.ch,
            );
        }
    }
}



// 计算节点的L的布局参数， 返回是否保留在脏列表中
fn calc<'a>(
    id: usize,
    (_class_sheet, text_content, _class_names, _world_matrixs, style_marks, _dirty_list): &Read,
	(char_nodes, layout_rs, rect_layout_styles, other_layout_styles, text_styles, font_sheet, idtree, nodes, node_states):&mut Write,
	layout_dirty: usize,
) -> bool {
	let text_style = &mut text_styles[id];
	let style_mark = &style_marks[id];
	
	let text = match text_content.get(id) {
        Some(t) => t.0.as_ref(),
        _ => "",
    };
    let tex_font = match font_sheet.get_font_info(&text_style.font.family) {
        Some(r) => (r.0.clone(), r.1),
        None => {
            println!(
                "font is not exist, face_name: {:?}, id: {:?}",
                text_style.font.family.as_ref(),
                id
            );
            return true;
        }
	};
	let font_size = get_size(tex_font.1, &text_style.font.size) as f32;
	let font_height = tex_font
        .0
		.get_font_height(font_size as usize, text_style.text.stroke.width);
	
	let line_height = get_line_height(font_height as usize, &text_style.text.line_height);
	if font_size == 0.0 {
        // #[cfg(feature = "warning")]
        println!("font_size==0.0, family: {:?}", text_style.font.family);
        return true;
	}
	
	let parent = idtree[id].parent();

	if layout_dirty > 0 {
		// 兼容目前使用父节点的对齐属性来对齐文本， 如果项目将其修改正确， 应该去掉该段TODO
		if style_mark.local_style & StyleType::TextAlign as usize > 0 || style_mark.class_style & StyleType::TextAlign as usize > 0 {
			other_layout_styles[parent].justify_content = match text_style.text.text_align {
				TextAlign::Center => JustifyContent::Center,
				TextAlign::Right => JustifyContent::FlexEnd,
				TextAlign::Left => JustifyContent::FlexStart,
				TextAlign::Justify => JustifyContent::SpaceBetween,
			};
			// style_notify.modify_event(cur_child, "width", 0);
		}
		
		if style_mark.local_style & StyleType::VerticalAlign as usize > 0 || style_mark.class_style & StyleType::VerticalAlign as usize > 0 {
			let r= match text_style.text.vertical_align {
				VerticalAlign::Middle => AlignItems::Center,
				VerticalAlign::Bottom => AlignItems::FlexEnd,
				VerticalAlign::Top => AlignItems::FlexStart
			};
			other_layout_styles[parent].align_items = r;
			let r= match text_style.text.vertical_align {
				VerticalAlign::Middle => AlignContent::Center,
				VerticalAlign::Bottom => AlignContent::FlexEnd,
				VerticalAlign::Top => AlignContent::FlexStart
			};
			other_layout_styles[parent].align_content = r;
		
		} else if style_mark.local_style1 & StyleType1::AlignContent as usize == 0 && style_mark.class_style1 & StyleType1::AlignContent as usize == 0 {
			// 文字的容器默认align_content为FlexStart
			other_layout_styles[parent].align_content = AlignContent::FlexStart;
		}
	
		if style_mark.local_style & StyleType::WhiteSpace as usize > 0 || style_mark.class_style & StyleType::WhiteSpace as usize > 0 {
			other_layout_styles[parent].flex_wrap = if text_style.text.white_space.allow_wrap() {
				FlexWrap::Wrap
			} else {
				FlexWrap::NoWrap
			}
		} else if style_mark.local_style1 & StyleType1::FlexWrap as usize == 0 && style_mark.class_style1 & StyleType1::FlexWrap as usize == 0{
			// 文字的容器默认flex_wrap为FlexWrap::Wrap
			other_layout_styles[parent].flex_wrap = FlexWrap::Wrap;
		}
	}

    let sw = text_style.text.stroke.width as usize;
	// let letter_spacing = text_style.text.letter_spacing - sw as f32 + 1.0;
	let char_margin = text_style.text.letter_spacing - sw as f32;
	let word_margin = text_style.text.word_spacing - sw as f32;
	// let font_sheet =  &mut write.4;

    // let tex_font = tex_font.0.clone();
    // // 如果有缩进变化, 则设置本span节点, 宽度为缩进值
    // layout_style.set_width(text_style.text.indent);

	let style_notify = rect_layout_styles.get_notify();
	let mut cur_child = idtree[id].children().head;
	let mut parent = id;
	style_notify.modify_event(id, "width", 0);
	let mut i = 0;


	// 保留空白符， 超出不换行， flex布局， 父节点设置了换行，每个字符需要被包含在一个行容器中
	// 这里创建第一行的容器
	if let WhiteSpace::Pre = text_style.text.white_space {
		let (id, mut style, mut char_node) = create_char_node(cur_child, parent, idtree, rect_layout_styles, char_nodes, nodes, node_states, layout_rs);
		parent = id;
		char_node.ch = char::from(0);
		cur_child = idtree[parent].children().head;
		style.size.width = Dimension::Percent(1.0);
	}

    // 根据每个字符, 创建对应的yoga节点, 加入父容器或字容器中
    for cr in split(text, true, text_style.text.white_space.preserve_spaces()) {
		// 如果是单词的结束字符，释放掉当前节点后面的所有兄弟节点， 并将当前节点索引重置为当前节点的父节点的下一个兄弟节点
		if let SplitResult::WordEnd = cr {
			if cur_child > 0 {
				free_childs(cur_child, idtree, nodes);
			}
			let node = &idtree[parent];
			cur_child = node.next();
			parent = node.parent();
			continue;
		} else if let SplitResult::Newline = cr {
			if let WhiteSpace::Pre = text_style.text.white_space {
				// 当text_style.text.white_space为WhiteSpace::Pre时，每遇到一个换行符，设置行节点宽度为100%，
				// 并且将当前节点设置为后续字符节点的父节点，直到遇到下一个换行符
				let p = &idtree[parent];
				parent = p.parent();
				cur_child = idtree[parent].next();

				
				let (id, mut style, mut char_node) = create_char_node(cur_child, parent, idtree, rect_layout_styles, char_nodes, nodes, node_states, layout_rs );
				cur_child = id;
				style.size.width = Dimension::Percent(1.0);
				char_node.ch = char::from(0);
				parent = cur_child;
				cur_child = idtree[cur_child].children().head;
				continue;
			} 
		}

		let (id, mut style, mut char_node) = create_char_node(cur_child, parent, idtree, rect_layout_styles, char_nodes, nodes, node_states, layout_rs);
		cur_child = id;
		
		match cr {
			// 存在WordStart， 表示开始一个多字符单词
			SplitResult::WordStart(_c) => {
				if char_node.ch != char::from(0) || layout_dirty > 0 {
					// 设置字符容器样式
					style.size.width = Dimension::Auto;
					style.size.height = Dimension::Points(line_height);
					style.margin.start = Dimension::Points(word_margin);
					style.line_start_margin = Number::Defined(0.0);
					char_node.ch = char::from(0);

					if i == 0 && text_style.text.indent > 0.0{
						style.line_start_margin = Number::Defined(text_style.text.indent);
					}
				}
				
				// 创建字符节点
				parent = cur_child;
				let cur_word = idtree[parent].children().head;
				let (cur_word, style1, char_node1) = create_char_node(cur_word, parent, idtree, rect_layout_styles, char_nodes, nodes, node_states, layout_rs);
				cur_child = cur_word;
				style = style1;
				char_node = char_node1;
			},
			_ => (),
		}
		match cr {
            SplitResult::Newline => {
				//当text_style.text.white_space为WhiteSpace::Pre, 设置行节点宽度为100%， 使得后面的元素超出本行宽度而换行
				style.size.width = Dimension::Percent(1.0);
				style.size.height = Dimension::Points(0.0);
				char_node.ch = '\n';
            }
            SplitResult::Whitespace => {
				if char_node.ch != ' ' || layout_dirty > 0 {
					style.size.width = Dimension::Points(font_size / 2.0);
					style.size.height = Dimension::Points(line_height);
					char_node.ch = ' ';
				}
            }
            SplitResult::Word(c) | SplitResult::WordStart(c) | SplitResult::WordNext(c) => {
				if char_node.ch != c || layout_dirty > 0 {
					let r = font_sheet.measure(
						&tex_font.0,
						font_size as usize,
						sw,
						text_style.font.weight,
						c,
					);
					
					style.size.width = Dimension::Points(r.0);
					style.size.height = Dimension::Points(line_height);
					style.margin.start = Dimension::Points(char_margin);
					style.line_start_margin = Number::Defined(0.0);

					char_node.ch = c;
					char_node.base_width = r.1;
					char_node.width = r.0;

					if i == 0 && text_style.text.indent > 0.0{
						style.line_start_margin = Number::Defined(text_style.text.indent);
					}
				}
				let head = idtree[cur_child].children().head;
				if head > 0 as usize {
					free_childs(head, idtree, nodes);
				}
				i += 1;
            },
            _ => (),
		}
		cur_child = idtree[cur_child].next();
	}
	if cur_child > 0 {
		free_childs(cur_child, idtree, nodes);
	}
	false
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

fn create_char_node<'a>(mut id: usize, parent: usize, idtree: &mut SingleCaseImpl<IdTree>, styles: &'a mut MultiCaseImpl<Node, RectLayoutStyle>, char_nodes: &'a mut MultiCaseImpl<Node, CharNode>, nodes: &mut EntityImpl<Node>, node_states: &mut MultiCaseImpl<Node, NodeState>, layouts: &mut MultiCaseImpl<Node, LayoutR>) -> (usize, &'a mut RectLayoutStyle, &'a mut CharNode) {
	if id == 0 {
		id = nodes.create_but_no_notify();
		styles.insert_no_notify(id, RectLayoutStyle::default());
		layouts.insert_no_notify(id, LayoutR::default());
		node_states.insert_no_notify(id, NodeState(INode::new(INodeStateType::SelfDirty)));

		idtree.create(id);
		idtree.insert_child(id, parent, std::usize::MAX);
		let char_node = CharNode {
			ch: ' ',              // 字符
			ch_id_or_count: 0, // 字符id或单词的字符数量
			base_width: 0.0,       // font_size 为32 的字符宽度
			width: 0.0,
		};
		char_nodes.insert_no_notify(id, char_node);
	}
	(id, &mut styles[id], &mut char_nodes[id])
}

impl_system! {
    LayoutImpl,
    true,
    {
        MultiCaseListener<Node, TextContent, CreateEvent>
    }
}

impl_system! {
    TextGlphySys,
    true,
    {
    }
}
