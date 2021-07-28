/**
 * 样式标记
* StyleMarkSys系统会在Node实体创建时， 自动为Node创建一个StyleMark组件， 该组件用于标记了各种样式脏、是否为本地样式
* StyleMarkSys系统会监听本地样式的修改，以标记样式的脏， 并覆盖class设置的样式属性（覆盖的方式为：修改该属性的本地样式标记为1）
* StyleMarkSys系统会监听ClassName的修改， 遍历class中的属性， 如果该属性没有设置本地样式，将覆盖该属性对应的组件，并标记样式脏
* class中的图片， 是一个url， 在设置class时， 该图片资源可能还未加载， StyleMarkSys会将不存在的图片url放入ImageWaitSheet中， 由外部处理ImageWaitSheet中的等待列表，图片加载完成， 应该将图片放入完成列表中， 并通知ImageWaitSheet修改， 由StyleMarkSys来处理ImageWaitSheet中的完成列表
* StyleMarkSys系统监听ImageWaitSheet单例的修改， 将完成加载的图片设置在对应的Node组件上， 并标记样式脏
*/
use std::marker::PhantomData;
// use std::mem::transmute;
use std::rc::Rc;

use ecs::{
    CreateEvent, DeleteEvent, EntityImpl, EntityListener, ModifyEvent,
    MultiCaseImpl, MultiCaseListener, Runner, SingleCaseImpl, SingleCaseListener, StdCell
};
use hal_core::*;
use flex_layout::*;
use share::Share;

use crate::component::calc::{Opacity as COpacity, LayoutR};
use crate::component::calc::*;
use crate::component::user::{Opacity, Overflow};
use crate::component::user::*;
use crate::entity::Node;
use crate::render::engine::{Engine, ShareEngine};
use crate::render::res::TextureRes;
use crate::single::class::*;
use crate::single::*;
use crate::single::IdTree;

//文字样式脏
const TEXT_DIRTY: usize = StyleType::LetterSpacing as usize
    | StyleType::WordSpacing as usize
    | StyleType::LineHeight as usize
    | StyleType::Indent as usize
    | StyleType::WhiteSpace as usize
    | StyleType::TextAlign as usize
    | StyleType::VerticalAlign as usize
    | StyleType::TextShadow as usize
    | StyleType::Color as usize
    | StyleType::Stroke as usize;

//字体脏
const FONT_DIRTY: usize = StyleType::FontStyle as usize
    | StyleType::FontFamily as usize
    | StyleType::FontSize as usize
    | StyleType::FontWeight as usize;

// 节点属性脏（不包含text， image， background等渲染属性）
const NODE_DIRTY: usize =
    StyleType::Filter as usize | StyleType::Opacity as usize | StyleType::BorderRadius as usize;
// 节点属性脏（不包含text， image， background等渲染属性）
const NODE_DIRTY1: usize = StyleType1::Visibility as usize
    | StyleType1::Enable as usize
    | StyleType1::ZIndex as usize
    | StyleType1::Transform as usize
    | StyleType1::Display as usize;

const TEXT_STYLE_DIRTY: usize = TEXT_DIRTY | FONT_DIRTY | StyleType::TextShadow as usize;

// 节点属性脏（不包含text， image， background等渲染属性）
const IMAGE_DIRTY: usize =
    StyleType::Image as usize | StyleType::ImageClip as usize | StyleType::ObjectFit as usize;
// 节点属性脏（不包含text， image， background等渲染属性）
const BORDER_IMAGE_DIRTY: usize = StyleType::BorderImage as usize
    | StyleType::BorderImageClip as usize
    | StyleType::BorderImageSlice as usize
	| StyleType::BorderImageRepeat as usize;

// 布局脏
const LAYOUT_OTHER_DIRTY: usize = StyleType2::Width as usize
    | StyleType2::Height as usize
    | LAYOUT_MARGIN_MARK
	| LAYOUT_PADDING_MARK
	| LAYOUT_BORDER_MARK
	| LAYOUT_POSITION_MARK
    | StyleType2::MinWidth as usize
    | StyleType2::MinHeight as usize
    | StyleType2::MaxHeight as usize
    | StyleType2::MaxWidth as usize
    | StyleType2::FlexShrink as usize
    | StyleType2::FlexGrow as usize
    | StyleType2::PositionType as usize
    | StyleType2::FlexWrap as usize
    | StyleType2::FlexDirection as usize
    | StyleType2::AlignContent as usize
    | StyleType2::AlignItems as usize
    | StyleType2::AlignSelf as usize
    | StyleType2::JustifyContent as usize;

pub struct StyleMarkSys<C> {
    text_style: TextStyle,
    show: Show,
    mark: PhantomData<(C)>,
}

impl<'a, C: HalContext + 'static> StyleMarkSys<C> {
    pub fn new() -> Self {
        Self {
            text_style: TextStyle::default(),
            show: Show::default(),
            mark: PhantomData,
        }
    }
}

#[inline]
fn set_local_dirty(
    dirty_list: &mut DirtyList,
    id: usize,
    ty: usize,
    style_marks: &mut MultiCaseImpl<Node, StyleMark>,
) {
	let style_mark = &mut style_marks[id];
	set_dirty(dirty_list, id, ty, style_mark);
	style_mark.local_style |= ty;
}

#[inline]
fn set_local_dirty1(
    dirty_list: &mut DirtyList,
    id: usize,
    ty: usize,
    style_marks: &mut MultiCaseImpl<Node, StyleMark>,
) {
    let style_mark = &mut style_marks[id];
    set_dirty1(dirty_list, id, ty, style_mark);
    style_mark.local_style1 |= ty;
}

#[inline]
fn set_local_dirty2(
    dirty_list: &mut DirtyList,
    id: usize,
    ty: usize,
    style_marks: &mut MultiCaseImpl<Node, StyleMark>,
) {
    let style_mark = &mut style_marks[id];
    set_dirty2(dirty_list, id, ty, style_mark);
    style_mark.local_style2 |= ty;
}

#[inline]
fn set_dirty(dirty_list: &mut DirtyList, id: usize, ty: usize, style_mark: &mut StyleMark) {
    if style_mark.dirty == 0 && style_mark.dirty1 == 0 && style_mark.dirty2 == 0{
        dirty_list.0.push(id);
	}
    style_mark.dirty |= ty;
}

#[inline]
fn set_dirty1(dirty_list: &mut DirtyList, id: usize, ty: usize, style_mark: &mut StyleMark) {
    if style_mark.dirty == 0 && style_mark.dirty1 == 0 && style_mark.dirty2 == 0 {
        dirty_list.0.push(id);
    }
    style_mark.dirty1 |= ty;
}

#[inline]
fn set_dirty2(dirty_list: &mut DirtyList, id: usize, ty: usize, style_mark: &mut StyleMark) {
    if style_mark.dirty == 0 && style_mark.dirty1 == 0 && style_mark.dirty2 == 0{
        dirty_list.0.push(id);
	}
    style_mark.dirty2 |= ty;
}

impl<'a, C: HalContext + 'static> Runner<'a> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn run(&mut self, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        for id in dirty_list.0.iter() {
            match style_marks.get_mut(*id) {
                Some(style_mark) => {
					style_mark.dirty = 0;
					style_mark.dirty1 = 0;
					style_mark.dirty2 = 0;
					style_mark.dirty_other = 0;
				},
                None => (),
            }
        }
        dirty_list.0.clear();
    }
}

// 监听TextStyle属性的改变
impl<'a, C: HalContext + 'static> EntityListener<'a, Node, CreateEvent>
    for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
		&'a mut MultiCaseImpl<Node, ClassName>,
		&'a mut SingleCaseImpl<DirtyList>,
    );

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, (style_marks, class_names, dirty_list): Self::WriteData) {
        style_marks.insert(event.id, StyleMark::default());
		class_names.insert_no_notify(event.id, ClassName::default());
		set_local_dirty1(dirty_list, event.id, StyleType1::Create as usize, style_marks);
    }
}

// 监听节点销毁事件，添加到脏列表
impl<'a, C: HalContext + 'static> EntityListener<'a, Node, ModifyEvent>
    for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, (style_marks, dirty_list): Self::WriteData) {
		if let Some(r) = style_marks.get_mut(event.id) {
			set_dirty1(dirty_list, event.id, StyleType1::Delete as usize, r);
		}
    }
}


// 监听TextStyle属性的改变
impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, RectLayoutStyle, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
		
        let r = match event.field {
			"margin" => LAYOUT_MARGIN_MARK,
			"margin-top" => StyleType2::MarginTop as usize,
			"margin-right" => StyleType2::MarginRight as usize,
			"margin-bottom" => StyleType2::MarginBottom as usize,
			"margin-left" => StyleType2::MarginLeft as usize,
			"width" => StyleType2::Width as usize,
			"height" => StyleType2::Height as usize,
			// "aspect_ratio" => StyleType1::As,
            _ => return,
		};
		let (style_marks, dirty_list) = write;
		set_local_dirty2(dirty_list, event.id, r as usize, style_marks);
    }
}

// 监听TextStyle属性的改变
impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, OtherLayoutStyle, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
		
        let r = match event.field {
			"position_type" => StyleType2::PositionType as usize,
			"direction" => StyleType2::FlexDirection as usize,

			"flex_direction" => StyleType2::FlexDirection as usize,
			"flex_wrap" => StyleType2::FlexWrap as usize,
			"justify_content" => StyleType2::JustifyContent as usize,
			"align_items" => StyleType2::AlignItems as usize,
			"align_content" => StyleType2::AlignContent as usize,

			// "order" => StyleType1::FlexDirection,
			"flex_grow" => StyleType2::FlexGrow as usize,
			"flex_shrink" => StyleType2::FlexShrink as usize,
			"align_self" => StyleType2::AlignSelf as usize,

			"position" => LAYOUT_PADDING_MARK,
			"top" => StyleType2::PositionTop as usize,
			"right" => StyleType2::PositionRight as usize,
			"bottom" => StyleType2::PositionBottom as usize,
			"left" => StyleType2::PositionLeft as usize,
			"padding" => LAYOUT_PADDING_MARK,
			"padding-top" => StyleType2::PaddingTop as usize,
			"padding-right" => StyleType2::PaddingRight as usize,
			"padding-bottom" => StyleType2::PaddingBottom as usize,
			"padding-left" => StyleType2::PaddingLeft as usize,
			"border" => LAYOUT_BORDER_MARK,
			"border-top" => StyleType2::BorderTop as usize,
			"border-right" => StyleType2::BorderRight as usize,
			"border-bottom" => StyleType2::BorderBottom as usize,
			"border-left" => StyleType2::BorderLeft as usize,
			"min_width" => StyleType2::MinWidth as usize,
			"min_height" => StyleType2::MinHeight as usize,
			"max_width" => StyleType2::MaxWidth as usize,
			"max_height" => StyleType2::MaxHeight as usize,
			// "aspect_ratio" => StyleType1::As,
            _ => 0,
		};
		let (style_marks, dirty_list) = write;
		if r > 0 {
			set_local_dirty2(dirty_list, event.id, r as usize, style_marks);
			return;
		}
		
		let r = match event.field {
			"display" => StyleType1::Display as usize,
			"overflow" =>  StyleType1::Overflow as usize,
			"flex_basis" => StyleType1::FlexBasis as usize,
			_ => return,
		};
		set_local_dirty1(dirty_list, event.id, r as usize, style_marks);
    }
}

// 监听TextStyle属性的改变
impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, TextStyle, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let r = match event.field {
            "letter_spacing" => StyleType::LetterSpacing,
			"word_spacing" => StyleType::WordSpacing,
			"white_space" => StyleType::WhiteSpace,
            "line_height" => StyleType::LineHeight,
            "text_indent" => StyleType::Indent,
            "color" => StyleType::Color,
            "stroke" => StyleType::Stroke,
            "text_align" => StyleType::TextAlign,
            "vertical_align" => StyleType::VerticalAlign,
            "text_shadow" => StyleType::TextShadow,
            "font_style" => StyleType::FontStyle,
            "font_weight" => StyleType::FontWeight,
            "font_size" => StyleType::FontSize,
            "font_family" => StyleType::FontFamily,
            _ => return,
		};
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, r as usize, style_marks);
    }
}

// 监听TextContente属性的改变
impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, TextContent, CreateEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        // &'a mut MultiCaseImpl<Node, TextStyle>,
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (/*text_styles, */style_marks, dirty_list) = write;
		// text_styles.insert_no_notify(event.id, self.default_text.clone());
        let style_mark = &mut style_marks[event.id];
        set_dirty(
            dirty_list,
            event.id,
            TEXT_STYLE_DIRTY | StyleType::Text as usize,
            style_mark,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, TextContent, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = &mut style_marks[event.id];
        set_dirty(dirty_list, event.id, StyleType::Text as usize, style_mark);
    }
}

// impl<'a, C: HalContext + 'static>
//     MultiCaseListener<'a, Node, MaskImage, CreateEvent> for StyleMarkSys<C>
// {
//     type ReadData = &'a SingleCaseImpl<IdTree>;
//     type WriteData = ImageWrite<'a, C>;
//     fn listen(&mut self, event: &CreateEvent, idtree: Self::ReadData, write: Self::WriteData) {
//         set_image_local(event.id, idtree, write);
//     }
// }

// impl<'a, C: HalContext + 'static>
//     MultiCaseListener<'a, Node, MaskImage, ModifyEvent> for StyleMarkSys<C>
// {
//     type ReadData = &'a SingleCaseImpl<IdTree>;
//     type WriteData = MaskImageWrite<'a, C>;
//     fn listen(&mut self, event: &ModifyEvent, idtree: Self::ReadData, write: Self::WriteData) {
//         set_mask_image_local(event.id, idtree, write);
//     }
// }

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, Image, CreateEvent> for StyleMarkSys<C>
{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ImageWrite<'a, C>;
    fn listen(&mut self, event: &CreateEvent, idtree: Self::ReadData, write: Self::WriteData) {
        set_image_local(event.id, idtree, write);
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, Image, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ImageWrite<'a, C>;
    fn listen(&mut self, event: &ModifyEvent, idtree: Self::ReadData, write: Self::WriteData) {
        set_image_local(event.id, idtree, write);
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, ImageClip, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
		&'a mut MultiCaseImpl<Node, RectLayoutStyle>,
        &'a mut MultiCaseImpl<Node, ImageClip>,
        &'a mut MultiCaseImpl<Node, Image>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list, layout_styles, image_clips, images) = write;
		let id = event.id;
		set_local_dirty(dirty_list, id, StyleType::ImageClip as usize, style_marks);
        if let Some(image) = images.get(id) {
            if let Some(src) = &image.src {
                set_image_size(
                    src,
                    &mut layout_styles[id],
                    image_clips.get(id),
                    &mut style_marks[id],
                );
            }
        }
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, ObjectFit, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::ObjectFit as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BorderImageClip, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let id = event.id;
        set_local_dirty(
            dirty_list,
            id,
            StyleType::BorderImageClip as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BorderImageSlice, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::BorderImageSlice as usize,
            style_marks,
        );
    }
}

// impl<'a, C: HalContext + 'static>
//     MultiCaseListener<'a, Node, MaskImageClip, CreateEvent> for StyleMarkSys<C>
// {
//     type ReadData = ();
//     type WriteData = (
//         &'a mut MultiCaseImpl<Node, StyleMark>,
//         &'a mut SingleCaseImpl<DirtyList>,
//     );
//     fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
//         let (style_marks, dirty_list) = write;
//         let id = event.id;
//         set_local_dirty1(
//             dirty_list,
//             id,
//             StyleType1::MaskImageClip as usize,
//             style_marks,
//         );
//     }
// }

// impl<'a, C: HalContext + 'static>
//     MultiCaseListener<'a, Node, MaskImageClip, ModifyEvent> for StyleMarkSys<C>
// {
//     type ReadData = ();
//     type WriteData = (
//         &'a mut MultiCaseImpl<Node, StyleMark>,
//         &'a mut SingleCaseImpl<DirtyList>,
//     );
//     fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
//         let (style_marks, dirty_list) = write;
//         set_local_dirty1(
//             dirty_list,
//             event.id,
//             StyleType1::MaskImageClip as usize,
//             style_marks,
//         );
//     }
// }

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BorderImage, CreateEvent> for StyleMarkSys<C>
{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = BorderImageWrite<'a, C>;
    fn listen(&mut self, event: &CreateEvent, idtree: Self::ReadData, write: Self::WriteData) {
        set_border_image_local(event.id, idtree, write);
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BorderImage, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = BorderImageWrite<'a, C>;
    fn listen(&mut self, event: &ModifyEvent, idtree: Self::ReadData, write: Self::WriteData) {
        set_border_image_local(event.id, idtree, write);
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BorderImageRepeat, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::BorderImageRepeat as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BorderColor, CreateEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::BorderColor as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BorderColor, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::BorderColor as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BackgroundColor, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::BackgroundColor as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BoxShadow, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
		let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::BoxShadow as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, ImageClip, CreateEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::ImageClip as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, ObjectFit, CreateEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::ObjectFit as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BorderImageClip, CreateEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::BorderImageClip as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BorderImageSlice, CreateEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::BorderImageSlice as usize,
            style_marks,
        );
    }
}

type BorderImageWrite<'a, C> = (
    &'a mut MultiCaseImpl<Node, StyleMark>,
    &'a mut SingleCaseImpl<DirtyList>,
    &'a mut SingleCaseImpl<ShareEngine<C>>,
    &'a mut SingleCaseImpl<ImageWaitSheet>,
    &'a mut MultiCaseImpl<Node, BorderImage>,
    &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
    &'a mut MultiCaseImpl<Node, BorderImageClip>,
);

type ImageWrite<'a, C> = (
    &'a mut MultiCaseImpl<Node, StyleMark>,
    &'a mut SingleCaseImpl<DirtyList>,
    &'a mut SingleCaseImpl<ShareEngine<C>>,
    &'a mut SingleCaseImpl<ImageWaitSheet>,
    &'a mut MultiCaseImpl<Node, Image>,
    &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
    &'a mut MultiCaseImpl<Node, ImageClip>,
);

type MaskImageWrite<'a, C> = (
    &'a mut MultiCaseImpl<Node, StyleMark>,
    &'a mut SingleCaseImpl<DirtyList>,
    &'a mut SingleCaseImpl<ShareEngine<C>>,
    &'a mut SingleCaseImpl<ImageWaitSheet>,
    &'a mut MultiCaseImpl<Node, MaskImage>,
    &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
    &'a mut MultiCaseImpl<Node, MaskImageClip>,
);

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BorderImageRepeat, CreateEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::BorderImageRepeat as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BackgroundColor, CreateEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::BackgroundColor as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BoxShadow, CreateEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
		let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::BoxShadow as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, Opacity, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
		let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::Opacity as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::ByOverflow as usize,
            style_marks,
        );
    }
}

// // visibility修改， 设置ByOverflow脏（clipsys 使用， dirty上没有位置容纳Visibility脏了， 因此设置在ByOverflow上）
// impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Visibility, ModifyEvent> for StyleMarkSys<C>{
// 	type ReadData = ();
// 	type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
// 	fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
// 		let (style_marks, dirty_list) = write;
// 		set_local_dirty(dirty_list, event.id, StyleType::ByOverflow as usize, style_marks);
// 	}
// }

// RenderObjs 创建， 设置ByOverflow脏
impl<'a, C: HalContext + 'static>
    SingleCaseListener<'a, RenderObjs, CreateEvent> for StyleMarkSys<C>
{
    type ReadData = &'a SingleCaseImpl<RenderObjs>;
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &CreateEvent, render_objs: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let id = render_objs[event.id].context;
        let style_mark = &mut style_marks[id];
        set_dirty(dirty_list, id, StyleType::ByOverflow as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BorderRadius, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::BorderRadius as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, Filter, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(
            dirty_list,
            event.id,
            StyleType::Filter as usize,
            style_marks,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, COpacity, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
		let style_mark = &mut style_marks[event.id];
        set_dirty(
            dirty_list,
            event.id,
            StyleType::Opacity as usize,
            style_mark,
        );
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, LayoutR, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
		let (style_marks, dirty_list) = write;
		// 虚拟节点不存在StyleMark组件
        let style_mark = match style_marks.get_mut(event.id) {
			Some(r) => r,
			None => return,
		};
        set_dirty(dirty_list, event.id, StyleType::Layout as usize, style_mark);
    }
}

// 包围盒修改，
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, Oct, ModifyEvent>
    for StyleMarkSys<C>
{
	type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
	// type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
	// type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
	fn listen(
        &mut self,
        event: &ModifyEvent,
        read: Self::ReadData,
        write: Self::WriteData,
    ) {
		let (style_marks, dirty_list) = write;
		let style_mark = &mut style_marks[event.id];
        set_dirty(dirty_list, event.id, StyleType::Oct as usize, style_mark);
    }
}

// 包围盒修改，
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, Oct, CreateEvent>
    for StyleMarkSys<C>
{
	type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
	// type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
	// type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
	fn listen(
        &mut self,
        event: &CreateEvent,
        read: Self::ReadData,
        write: Self::WriteData,
    ) {
		let (style_marks, dirty_list) = write;
		let style_mark = &mut style_marks[event.id];
        set_dirty(dirty_list, event.id, StyleType::Oct as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
		let style_mark = &mut style_marks[event.id];
        set_dirty(dirty_list, event.id, StyleType::Matrix as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, ZDepth, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = &mut style_marks[event.id];
        set_dirty(dirty_list, event.id, StyleType::Matrix as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, IdTree, CreateEvent>
    for StyleMarkSys<C>
{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ImageTextureWrite<'a, C>;
    fn listen(&mut self, event: &CreateEvent, idtree: Self::ReadData, mut write: Self::WriteData) {
        load_image(event.id, &mut write);

        let node = &idtree[event.id];
        for (id, _n) in idtree.recursive_iter(node.children().head) {
            load_image(id, &mut write);
        }
    }
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, IdTree, DeleteEvent>
    for StyleMarkSys<C>
{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ImageTextureWrite<'a, C>;
    fn listen(&mut self, event: &DeleteEvent, idtree: Self::ReadData, mut write: Self::WriteData) {
        release_image(event.id, &mut write);

        let node = &idtree[event.id];
        for (id, _n) in idtree.recursive_iter(node.children().head) {
            release_image(id, &mut write);
        }
    }
}

type ReadData<'a> = (
    &'a MultiCaseImpl<Node, ClassName>,
    &'a SingleCaseImpl<Share<StdCell<ClassSheet>>>,
);
type WriteData<'a, C> = (
    &'a mut MultiCaseImpl<Node, TextStyle>,
    &'a mut MultiCaseImpl<Node, Image>,
    &'a mut MultiCaseImpl<Node, ImageClip>,
    &'a mut MultiCaseImpl<Node, ObjectFit>,
    &'a mut MultiCaseImpl<Node, BorderImage>,
    &'a mut MultiCaseImpl<Node, BorderImageClip>,
    &'a mut MultiCaseImpl<Node, BorderImageSlice>,
    &'a mut MultiCaseImpl<Node, BorderImageRepeat>,
    &'a mut MultiCaseImpl<Node, BorderColor>,
    &'a mut MultiCaseImpl<Node, BackgroundColor>,
    &'a mut MultiCaseImpl<Node, BoxShadow>,
    &'a mut MultiCaseImpl<Node, WorldMatrix>,
    &'a mut MultiCaseImpl<Node, Opacity>,
    &'a mut MultiCaseImpl<Node, Transform>,
    &'a mut MultiCaseImpl<Node, BorderRadius>,
    &'a mut MultiCaseImpl<Node, Filter>,
    &'a mut MultiCaseImpl<Node, ZIndex>,
    &'a mut MultiCaseImpl<Node, Show>,
    &'a mut MultiCaseImpl<Node, Overflow>,
    &'a mut MultiCaseImpl<Node, StyleMark>,
	&'a mut MultiCaseImpl<Node, RectLayoutStyle>,
	&'a mut MultiCaseImpl<Node, OtherLayoutStyle>,
    &'a mut SingleCaseImpl<IdTree>,
    &'a mut SingleCaseImpl<ShareEngine<C>>,
    &'a mut SingleCaseImpl<ImageWaitSheet>,
    &'a mut SingleCaseImpl<DirtyList>,
);

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, ClassName, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = ReadData<'a>;
    type WriteData = WriteData<'a, C>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, mut write: Self::WriteData) {
		let (class_names, class_sheet) = read;
		let class_name = &class_names[event.id];
		let class_sheet = &class_sheet.borrow();

		//event.index是接的className的指针
        let oldr = unsafe { &* Box::from_raw(event.index as *mut Option<ClassName>) };
		let mark = &mut write.19[event.id];
        let (old_style, old_style1, old_style2) = (mark.class_style, mark.class_style1, mark.class_style2);
        mark.class_style = 0;
		mark.class_style1 = 0;
		mark.class_style2 = 0;

		// TODO
		let oldt;
		let old;
		match oldr {
			Some(r) => old = r,
			None => {
				oldt = ClassName::default();
				old = &oldt;
			},
		}
        if class_name.one > 0 {
            if old.one != class_name.one {
                set_attr(
                    event.id,
                    class_name.one,
                    &mut self.text_style,
                    read,
                    &mut write,
                );
                if class_name.two > 0 {
                    set_attr(
                        event.id,
                        class_name.two,
                        &mut self.text_style,
                        read,
                        &mut write,
                    );
                }
                for i in 0..class_name.other.len() {
                    set_attr(
                        event.id,
                        class_name.other[i],
                        &mut self.text_style,
                        read,
                        &mut write,
                    );
                }
            } else {
                set_mark(class_sheet, class_name.one, mark);
                if class_name.two > 0 {
                    if old.two != class_name.two {
                        set_attr(
                            event.id,
                            class_name.two,
                            &mut self.text_style,
                            read,
                            &mut write,
                        );
                        for i in 0..class_name.other.len() {
                            set_attr(
                                event.id,
                                class_name.other[i],
                                &mut self.text_style,
                                read,
                                &mut write,
                            );
                        }
                    } else {
                        set_mark(class_sheet, class_name.two, mark);
                        let mut index = 0;
                        // 跳过class id相同的项
                        for i in 0..class_name.other.len() {
                            match old.other.get(i) {
                                Some(r) => {
                                    if *r == class_name.other[i] {
                                        set_mark(class_sheet, class_name.other[i], mark);
                                        index += 1;
                                    } else {
                                        break;
                                    }
                                }
                                None => break,
                            }
                        }
                        // 设置class属性
                        for i in index..class_name.other.len() {
                            set_attr(
                                event.id,
                                class_name.other[i],
                                &mut self.text_style,
                                read,
                                &mut write,
                            );
                        }
                    }
                }
            }
		}

        // 重置旧的class中设置的属性
        if old_style > 0 || old_style1 > 0 || old_style2 > 0 {
            reset_attr(
                event.id,
                read,
                &mut write,
                old_style,
				old_style1,
				old_style2,
            );
		}
    }
}


// 监听图片等待列表的改变， 将已加载完成的图片设置到对应的组件上
impl<'a, C: HalContext + 'static>
    SingleCaseListener<'a, ImageWaitSheet, ModifyEvent> for StyleMarkSys<C>
{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (
        &'a mut EntityImpl<Node>,
        &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
        &'a mut MultiCaseImpl<Node, Image>,
        &'a mut MultiCaseImpl<Node, ImageClip>,
        &'a mut MultiCaseImpl<Node, BorderImage>,
		&'a mut MultiCaseImpl<Node, MaskImage>,
        // &'a mut MultiCaseImpl<Node, BorderImageClip>,
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<ImageWaitSheet>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, _event: &ModifyEvent, idtree: Self::ReadData, write: Self::WriteData) {
        let (
            entitys,
            layout_nodes,
            images,
            image_clips,
            border_images,
            // border_image_clips,
			mask_images,
            style_marks,
            image_wait_sheet,
            dirty_list,
        ) = write;

        for wait in image_wait_sheet.finish.iter() {
            for image_wait in wait.2.iter() {
                // 图片加载完成后， 节点可能已经删除， 因此跳过
                if !entitys.is_exist(image_wait.id) {
                    continue;
                }
                // 节点不可见， 跳过
                if idtree.get(image_wait.id).is_none() {
                    continue;
                }
                // 判断等待类型， 设置对应的组件
                match image_wait.ty {
                    ImageType::ImageLocal => {
                        if let Some(image) = images.get_mut(image_wait.id) {
                            if image.url == wait.0 {
								image.src = Some(wait.1.clone());

                                // if Some image_clips.get(image_wait.id)
                                set_local_dirty(
                                    dirty_list,
                                    image_wait.id,
                                    StyleType::Image as usize,
                                    style_marks,
								);
								
								set_image_size(
                                    &wait.1,
                                    &mut layout_nodes[image_wait.id],
                                    image_clips.get(image_wait.id),
                                    &mut style_marks[image_wait.id],
                                );
                            }
                        }
                    }
                    ImageType::ImageClass => {
                        let style_mark = &mut style_marks[image_wait.id];
                        if style_mark.local_style & StyleType::Image as usize != 0 {
                            // 本地样式存在Image， 跳过
                            continue;
                        }
                        if let Some(image) = images.get_mut(image_wait.id) {
                            if image.url == wait.0 {
								image.src = Some(wait.1.clone());
								set_dirty(
                                    dirty_list,
                                    image_wait.id,
                                    StyleType::Image as usize,
                                    style_mark,
                                );
                                set_image_size(
                                    &wait.1,
                                    &mut layout_nodes[image_wait.id],
                                    image_clips.get(image_wait.id),
                                    style_mark,
                                );
                            }
                        }
                    }
                    ImageType::BorderImageLocal => {
                        if let Some(image) = border_images.get_mut(image_wait.id) {
                            if image.0.url == wait.0 {
                                image.0.src = Some(wait.1.clone());
                                set_local_dirty(
                                    dirty_list,
                                    image_wait.id,
                                    StyleType::BorderImage as usize,
                                    style_marks,
                                );
                            }
                        }
                    }
                    ImageType::BorderImageClass => {
                        let style_mark = &mut style_marks[image_wait.id];
                        if style_mark.local_style & StyleType::BorderImage as usize != 0 {
                            // 本地样式存在BorderImage， 跳过
                            continue;
                        }
                        if let Some(image) = border_images.get_mut(image_wait.id) {
                            if image.0.url == wait.0 {
                                image.0.src = Some(wait.1.clone());
                                set_dirty(
                                    dirty_list,
                                    image_wait.id,
                                    StyleType::BorderImage as usize,
                                    style_mark,
                                );
                            }
                        }
                    }
					ImageType::MaskImageLocal => {
                        if let Some(image) = mask_images.get_mut(image_wait.id) {
                            if image.url == wait.0 {
								image.src = Some(wait.1.clone());
								set_local_dirty1(
                                    dirty_list,
                                    image_wait.id,
                                    StyleType1::MaskImage as usize,
                                    style_marks,
								);

                            }
                        }
                    }
                    ImageType::MaskImageClass => {
                        let style_mark = &mut style_marks[image_wait.id];
                        if style_mark.local_style1 & StyleType1::MaskImage as usize != 0 {
                            // 本地样式存在MaskImage， 跳过
                            continue;
                        }
                        if let Some(image) = mask_images.get_mut(image_wait.id) {
                            if image.url == wait.0 {
                                image.src = Some(wait.1.clone());
                                set_dirty1(
                                    dirty_list,
                                    image_wait.id,
                                    StyleType1::MaskImage as usize,
                                    style_mark,
                                );
                            }
                        }
                    }
                }
            }
        }
		image_wait_sheet.finish.clear(); // 清空
    }
}

fn set_image_size(
    src: &Rc<TextureRes>,
	layout_style: &mut RectLayoutStyle,
    image_clip: Option<&ImageClip>,
    style_mark: &mut StyleMark,
) {
	let img_clip;
	let image_clip = match image_clip {
		Some(r) => r,
		None => {
			img_clip = Aabb2::new(Point2::new(0.0, 0.0), Point2::new(1.0, 1.0));
			&img_clip
		},
	};

	if style_mark.local_style2 & (StyleType2::Width as usize) == 0
		&& style_mark.class_style2 & (StyleType2::Width as usize) == 0
	{
		
		layout_style.size.width = Dimension::Points(src.width as f32 * (image_clip.maxs.x - image_clip.mins.x));
		// set_image_size必然是在样式脏的情况下调用，只需要标记脏类型，无需再次添加到脏列表
		style_mark.dirty2 |= StyleType2::Width as usize;
	}

	if style_mark.local_style2 & (StyleType2::Height as usize) == 0
		&& style_mark.class_style2 & (StyleType2::Height as usize) == 0
	{
		layout_style.size.height = Dimension::Points(src.height as f32 * (image_clip.maxs.y - image_clip.mins.y));
		// set_image_size必然是在样式脏的情况下调用，只需要标记脏类型，无需再次添加到脏列表
		style_mark.dirty2 |= StyleType2::Height as usize;
	}
}


#[inline]
fn reset_attr<C: HalContext>(
    id: usize,
    read: ReadData,
    write: &mut WriteData<C>,
    old_style: usize,
	old_style1: usize,
	old_style2: usize,
) {
    let (_class_names, _class_sheet) = read;
    let (
        text_styles,
        images,
        image_clips,
        obj_fits,
        border_images,
        border_image_clips,
        border_image_slices,
        border_image_repeats,
        border_colors,
        background_colors,
        box_shadows,
        _world_matrixs,
        opacitys,
        transforms,
        border_radiuss,
        filters,
        zindexs,
        shows,
        _overflows,
        style_marks,
		rect_layout_styles,
		other_layout_styles,
        _idtree,
        _engine,
        _image_wait_sheet,
        dirty_list,
    ) = write;

	let rect_layout_style =  &mut rect_layout_styles[id];
	let other_layout_style =  &mut other_layout_styles[id];
    let style_mark = &mut style_marks[id];
    // old_style中为1， class_style和local_style不为1的属性, 应该删除
    let old_style = !(!old_style | (old_style & (style_mark.class_style | style_mark.local_style)));
    let old_style1 =
		!(!old_style1 | (old_style1 & (style_mark.class_style1 | style_mark.local_style1)));
	let old_style2 =
		!(!old_style2 | (old_style2 & (style_mark.class_style2 | style_mark.local_style2)));
    if old_style != 0 {
        if old_style & TEXT_STYLE_DIRTY != 0 {
			let defualt_text = unsafe { &*(&text_styles[0] as *const TextStyle as usize as *const TextStyle) };
            if let Some(text_style) = text_styles.get_mut(id) {
                if old_style & TEXT_DIRTY != 0 {
                    if old_style & StyleType::LetterSpacing as usize != 0 {
                        text_style.text.letter_spacing = defualt_text.text.letter_spacing;
                        set_dirty(
                            dirty_list,
                            id,
                            StyleType::LetterSpacing as usize,
                            style_mark,
                        );
                    }
                    if old_style & StyleType::WordSpacing as usize != 0 {
                        text_style.text.word_spacing = defualt_text.text.word_spacing;
                        set_dirty(dirty_list, id, StyleType::WordSpacing as usize, style_mark);
                    }
                    if old_style & StyleType::LineHeight as usize != 0 {
                        text_style.text.line_height = defualt_text.text.line_height;
                        set_dirty(dirty_list, id, StyleType::LineHeight as usize, style_mark);
                    }
                    if old_style & StyleType::Indent as usize != 0 {
                        text_style.text.indent = defualt_text.text.indent;
                        set_dirty(dirty_list, id, StyleType::Indent as usize, style_mark);
                    }
                    if old_style & StyleType::WhiteSpace as usize != 0 {
                        text_style.text.white_space = defualt_text.text.white_space;
                        set_dirty(dirty_list, id, StyleType::WhiteSpace as usize, style_mark);
                    }

                    if old_style & StyleType::Color as usize != 0 {
                        text_style.text.color = defualt_text.text.color.clone();
                        set_dirty(dirty_list, id, StyleType::Color as usize, style_mark);
                    }

                    if old_style & StyleType::Stroke as usize != 0 {
                        text_style.text.stroke = defualt_text.text.stroke.clone();
                        set_dirty(dirty_list, id, StyleType::Stroke as usize, style_mark);
                    }

                    if old_style & StyleType::TextAlign as usize != 0 {
                        text_style.text.text_align = defualt_text.text.text_align;
                        set_dirty(dirty_list, id, StyleType::TextAlign as usize, style_mark);
                    }

                    if old_style & StyleType::VerticalAlign as usize != 0 {
                        text_style.text.vertical_align = defualt_text.text.vertical_align;
                        set_dirty(
                            dirty_list,
                            id,
                            StyleType::VerticalAlign as usize,
                            style_mark,
                        );
                    }
                }

                if old_style & StyleType::TextShadow as usize != 0 {
                    text_style.shadow = defualt_text.shadow.clone();
                    set_dirty(dirty_list, id, StyleType::TextShadow as usize, style_mark);
                }

                if old_style & FONT_DIRTY == 0 {
                    if old_style & StyleType::FontStyle as usize != 0 {
                        text_style.font.style = defualt_text.font.style;
                        set_dirty(dirty_list, id, StyleType::FontStyle as usize, style_mark);
                    }
                    if old_style & StyleType::FontWeight as usize != 0 {
                        text_style.font.weight = defualt_text.font.weight;
                        set_dirty(dirty_list, id, StyleType::FontWeight as usize, style_mark);
                    }
                    if old_style & StyleType::FontSize as usize != 0 {
						text_style.font.size = defualt_text.font.size;
                        set_dirty(dirty_list, id, StyleType::FontSize as usize, style_mark);
                    }
                    if old_style & StyleType::FontFamily as usize != 0 {
                        text_style.font.family = defualt_text.font.family.clone();
                        set_dirty(dirty_list, id, StyleType::FontFamily as usize, style_mark);
                    }
                }
            }
        }

        if old_style & IMAGE_DIRTY != 0 {
            if old_style & StyleType::Image as usize != 0 {
                images.delete(id);
                set_dirty(dirty_list, id, StyleType::Image as usize, style_mark);
            }
            if old_style & StyleType::ImageClip as usize != 0 {
                image_clips.delete(id);
                set_dirty(dirty_list, id, StyleType::ImageClip as usize, style_mark);
            }
            if old_style & StyleType::ObjectFit as usize != 0 {
                obj_fits.delete(id);
                set_dirty(dirty_list, id, StyleType::ObjectFit as usize, style_mark);
            }
        }

        if old_style & BORDER_IMAGE_DIRTY != 0 {
            if old_style & StyleType::BorderImage as usize != 0 {
                border_images.delete(id);
                set_dirty(dirty_list, id, StyleType::BorderImage as usize, style_mark);
            }
            if old_style & StyleType::BorderImageClip as usize != 0 {
                border_image_clips.delete(id);
                set_dirty(
                    dirty_list,
                    id,
                    StyleType::BorderImageClip as usize,
                    style_mark,
                );
            }
            if old_style & StyleType::BorderImageSlice as usize != 0 {
                border_image_slices.delete(id);
                set_dirty(
                    dirty_list,
                    id,
                    StyleType::BorderImageSlice as usize,
                    style_mark,
                );
            }
            if old_style & StyleType::BorderImageRepeat as usize != 0 {
                border_image_repeats.delete(id);
                set_dirty(
                    dirty_list,
                    id,
                    StyleType::BorderImageRepeat as usize,
                    style_mark,
                );
            }
        }

        if old_style & StyleType::BorderColor as usize != 0 {
            border_colors.delete(id);
            set_dirty(dirty_list, id, StyleType::BorderColor as usize, style_mark);
        }

        if old_style & StyleType::BackgroundColor as usize != 0 {
            background_colors.delete(id);
            set_dirty(
                dirty_list,
                id,
                StyleType::BackgroundColor as usize,
                style_mark,
            );
        }

        if old_style & StyleType::BoxShadow as usize != 0 {
            box_shadows.delete(id);
            set_dirty(dirty_list, id, StyleType::BoxShadow as usize, style_mark);
        }

        if old_style & NODE_DIRTY != 0 {
            if old_style & StyleType::Opacity as usize != 0 {
                opacitys.delete(id);
                set_dirty(dirty_list, id, StyleType::Opacity as usize, style_mark);
            }

            if old_style & StyleType::BorderRadius as usize != 0 {
                border_radiuss.delete(id);
                set_dirty(dirty_list, id, StyleType::BorderRadius as usize, style_mark);
            }

            if old_style & StyleType::Filter as usize != 0 {
                filters.delete(id);
                set_dirty(dirty_list, id, StyleType::Filter as usize, style_mark);
            }
        }
    }

    if old_style1 != 0 {
        if old_style1 & NODE_DIRTY1 != 0 {
            if old_style1 & StyleType1::Enable as usize != 0
                || old_style1 & StyleType1::Display as usize != 0
                || old_style1 & StyleType1::Visibility as usize != 0
            {
                if let Some(show) = shows.get_mut(id) {
                    if old_style1 & StyleType1::Enable as usize != 0 {
                        show.set_enable(EnableType::Auto);
                    }
                    if old_style1 & StyleType1::Display as usize != 0 {
						other_layout_style.display = Display::Flex;
                        show.set_display(Display::Flex);
                    }
                    if old_style1 & StyleType1::Visibility as usize != 0 {
                        show.set_visibility(true);
                    }
                }
                shows.get_notify_ref().modify_event(id, "", 0);

                if old_style1 & StyleType1::ZIndex as usize != 0 {
                    zindexs.insert(id, ZIndex(0));
                }

                if old_style1 & StyleType1::Transform as usize != 0 {
                    transforms.delete(id);
                }
            }
		}

		if old_style1 & StyleType1::FlexBasis as usize != 0 {
			other_layout_style.flex_basis = Dimension::Undefined;
		}
	}
	
	if old_style2 != 0 {
		if old_style2 & LAYOUT_RECT_MARK != 0 {
			if old_style2 & StyleType2::Width as usize != 0 {
				rect_layout_style.size.width = Dimension::Undefined;
			}
			if old_style2 & StyleType2::Height as usize != 0 {
				rect_layout_style.size.height = Dimension::Undefined;
			}
			if old_style2 & StyleType2::MarginTop as usize != 0 {
				rect_layout_style.margin.top = Dimension::Undefined;
			}
			if old_style2 & StyleType2::MarginRight as usize != 0 {
				rect_layout_style.margin.end = Dimension::Undefined;
			}
			if old_style2 & StyleType2::MarginBottom as usize != 0 {
				rect_layout_style.margin.bottom = Dimension::Undefined;
			}
			if old_style2 & StyleType2::MarginLeft as usize != 0 {
				rect_layout_style.margin.start = Dimension::Undefined;
			}
			// *rect_layout_style = RectLayoutStyle::default();
            // reset_layout_attr(layout_style, old_style2);
		}
		
		if old_style2 & LAYOUT_OTHER_DIRTY != 0 {
			if old_style2 & StyleType2::PaddingTop as usize != 0 {
				other_layout_style.padding.top = Dimension::Undefined;
			}
			if old_style2 & StyleType2::PaddingRight as usize != 0 {
				other_layout_style.padding.end = Dimension::Undefined;
			}
			if old_style2 & StyleType2::PaddingBottom as usize != 0 {
				other_layout_style.padding.bottom = Dimension::Undefined;
			}
			if old_style2 & StyleType2::PaddingLeft as usize != 0 {
				other_layout_style.padding.start = Dimension::Undefined;
			}

			if old_style2 & StyleType2::BorderTop as usize != 0 {
				other_layout_style.border.top = Dimension::Undefined;
			}
			if old_style2 & StyleType2::BorderRight as usize != 0 {
				other_layout_style.border.end = Dimension::Undefined;
			}
			if old_style2 & StyleType2::BorderBottom as usize != 0 {
				other_layout_style.border.bottom = Dimension::Undefined;
			}
			if old_style2 & StyleType2::BorderLeft as usize != 0 {
				other_layout_style.border.start = Dimension::Undefined;
			}

			if old_style2 & StyleType2::PositionTop as usize != 0 {
				other_layout_style.position.top = Dimension::Undefined;
			}
			if old_style2 & StyleType2::PositionRight as usize != 0 {
				other_layout_style.position.end = Dimension::Undefined;
			}
			if old_style2 & StyleType2::PositionBottom as usize != 0 {
				other_layout_style.position.bottom = Dimension::Undefined;
			}
			if old_style2 & StyleType2::PositionLeft as usize != 0 {
				other_layout_style.position.start = Dimension::Undefined;
			}
			if old_style2 & StyleType2::MinWidth as usize != 0{
				other_layout_style.min_size.width = Dimension::Undefined;
			}
			if old_style2 & StyleType2::MinHeight as usize != 0{
				other_layout_style.min_size.height = Dimension::Undefined;
			}
			if old_style2 & StyleType2::MaxWidth as usize != 0{
				other_layout_style.max_size.width = Dimension::Undefined;
			}
			if old_style2 & StyleType2::MaxHeight as usize != 0{
				other_layout_style.max_size.height = Dimension::Undefined;
			}

			if old_style2 & StyleType2::FlexShrink as usize != 0 {
				other_layout_style.flex_shrink = 0.0;
			}
			if old_style2 & StyleType2::FlexGrow as usize != 0 {
				other_layout_style.flex_grow = 0.0;
			}
			if old_style2 & StyleType2::PositionType as usize != 0 {
				other_layout_style.position_type = PositionType::Absolute;
			}
			if old_style2 & StyleType2::FlexWrap as usize != 0 {
				other_layout_style.flex_wrap = FlexWrap::NoWrap;
			}
			if old_style2 & StyleType2::FlexDirection as usize != 0 {
				other_layout_style.flex_direction = FlexDirection::Row;
			}
			if old_style2 & StyleType2::AlignContent as usize != 0 {
				other_layout_style.align_content = AlignContent::FlexStart;
			}
			if old_style2 & StyleType2::AlignItems as usize != 0 {
				other_layout_style.align_items = AlignItems::FlexStart;
			}
			if old_style2 & StyleType2::AlignSelf as usize != 0 {
				other_layout_style.align_self = AlignSelf::FlexStart;
			}
			if old_style2 & StyleType2::JustifyContent as usize != 0 {
				other_layout_style.justify_content = JustifyContent::FlexStart;
			}


			// *other_layout_style = OtherLayoutStyle::default();
            // reset_layout_attr(layout_style, old_style2);
        }
	}
}

// fn reset_layout_attr(layout_style: &LayoutStyle, old_style1: usize) {
//     if old_style1 & StyleType2::Width as usize != 0 {
// 		layout_style.size.width = Dimension::undefined;
//     }
//     if old_style1 & StyleType2::Height as usize != 0 {
// 		layout_style.size.height = Dimension::undefined;
//     }
//     if old_style1 & StyleType1::Margin as usize != 0 {
// 		layout_style.margin.start = Dimension::undefined;
// 		layout_style.margin.end = Dimension::undefined;
// 		layout_style.margin.top = Dimension::undefined;
// 		layout_style.margin.bottom = Dimension::undefined;
//     }
//     if old_style1 & StyleType1::Padding as usize != 0 {
//         layout_style.padding.start = Dimension::undefined;
// 		layout_style.size.padding.end = Dimension::undefined;
// 		layout_style.size.padding.top = Dimension::undefined;
// 		layout_style.size.padding.bottom = Dimension::undefined;
//     }
//     if old_style1 & StyleType1::Border as usize != 0 {
//         layout_style.size.border.start = Dimension::undefined;
// 		layout_style.size.border.end = Dimension::undefined;
// 		layout_style.size.border.top = Dimension::undefined;
// 		layout_style.size.border.bottom = Dimension::undefined;
//     }
//     if old_style1 & StyleType1::Position as usize != 0 {
// 		layout_style.position = Rect{start:Dimension::undefined, end: Dimension::undefined, top: Dimension::undefined, top: Dimension::bottom};
//     }
//     if old_style1 & StyleType1::MinWidth as usize != 0 || old_style1 & StyleType1::MinHeight as usize != 0 {
// 		layout_style.min_size = Size{width: Dimension::undefined, height: Dimension::undefined};
//     }

//     if old_style1 & StyleType1::MaxWidth as usize != 0 {
//         layout_style.set_max_width(std::f32::NAN);
//     }
//     if old_style1 & StyleType1::MaxHeight as usize != 0 {
//         layout_style.set_max_height(std::f32::NAN);
//     }
//     if old_style1 & StyleType1::FlexBasis as usize != 0 {
//         layout_style.set_flex_basis(std::f32::NAN);
//     }
//     if old_style1 & StyleType1::FlexShrink as usize != 0 {
//         layout_style.set_flex_shrink(std::f32::NAN);
//     }
//     if old_style1 & StyleType1::FlexGrow as usize != 0 {
//         layout_style.set_flex_grow(std::f32::NAN);
//     }
//     if old_style1 & StyleType1::PositionType as usize != 0 {
//         layout_style.set_position_type(YGPositionType::YGPositionTypeAbsolute);
//     }
//     if old_style1 & StyleType1::FlexWrap as usize != 0 {
//         layout_style.set_flex_wrap(YGWrap::YGWrapWrap);
//     }
//     if old_style1 & StyleType1::FlexDirection as usize != 0 {
//         layout_style.set_flex_direction(YGFlexDirection::YGFlexDirectionRow);
//     }
//     if old_style1 & StyleType1::AlignContent as usize != 0 {
//         layout_style.set_align_content(YGAlign::YGAlignFlexStart);
//     }
//     if old_style1 & StyleType1::AlignItems as usize != 0 {
//         layout_style.set_align_items(YGAlign::YGAlignFlexStart);
//     }
//     if old_style1 & StyleType1::AlignSelf as usize != 0 {
//         layout_style.set_align_self(YGAlign::YGAlignFlexStart);
//     }
//     if old_style1 & StyleType1::JustifyContent as usize != 0 {
//         layout_style.set_justify_content(YGJustify::YGJustifyFlexStart);
//     }
// }

fn set_attr<C: HalContext>(
    id: usize,
    class_name: usize,
    text_style: &mut TextStyle,
    read: ReadData,
    write: &mut WriteData<C>,
) {
    if class_name == 0 {
        return;
    }
    let (_class_names, class_sheet) = read;
    let (
        text_styles,
        images,
        image_clips,
        obj_fits,
        border_images,
        border_image_clips,
        border_image_slices,
        border_image_repeats,
        border_colors,
        background_colors,
        box_shadows,
        _world_matrixs,
        opacitys,
        transforms,
        border_radiuss,
        filters,
        zindexs,
        shows,
        overflows,
        style_marks,
		rect_layout_styles,
		other_layout_styles,
        idtree,
        engine,
        image_wait_sheet,
        dirty_list,
	) = write;
	let class_sheet = &class_sheet.borrow();
    let style_mark = &mut style_marks[id];
    // 设置布局属性， 没有记录每个个属性是否在本地样式表中存在， TODO
	let rect_layout_style = &mut rect_layout_styles[id];
	let other_layout_style = &mut other_layout_styles[id];

    let class = match class_sheet.class_map.get(&class_name) {
        Some(class) => class,
        None => return,
    };

    let text_style = &mut text_styles[id];

	style_mark.class_style |= class.class_style_mark;
	style_mark.class_style1 |= class.class_style_mark1;
	style_mark.class_style2 |= class.class_style_mark2;

    set_attr1(
        id,
        dirty_list,
        &class.attrs1,
        style_mark,
        text_style,
        shows,
        overflows,
        other_layout_style,
        obj_fits,
    );
    set_attr2(
        id,
        dirty_list,
        &class.attrs2,
        style_mark,
        text_style,
		rect_layout_style,
		other_layout_style,
        zindexs,
        opacitys,
        border_image_repeats,
        images,
        border_images,
        image_wait_sheet,
        engine,
        idtree,
        image_clips.get(id),
        // border_image_clips.get(id),
    );
    set_attr3(
        id,
        dirty_list,
        &class.attrs3,
        style_mark,
        text_style,
        border_image_slices,
        border_image_clips,
        // border_images,
        image_clips,
        images,
        box_shadows,
        background_colors,
        border_colors,
        border_radiuss,
        filters,
        transforms,
        rect_layout_styles,
    );
}

#[inline]
fn set_mark(class_sheet: &ClassSheet, name: usize, mark: &mut StyleMark) {
    match class_sheet.class_map.get(&name) {
        Some(class) => {
            mark.class_style |= class.class_style_mark;
			mark.class_style1 |= class.class_style_mark1;
			mark.class_style2 |= class.class_style_mark2;
        }
        None => (),
    };
}

pub fn set_attr1(
    id: usize,
    dirty_list: &mut DirtyList,
    layout_attrs: &Vec<Attribute1>,
    style_mark: &mut StyleMark,
    text_style: &mut TextStyle,
    shows: &mut MultiCaseImpl<Node, Show>,
    overflows: &mut MultiCaseImpl<Node, Overflow>,
    other_style: &mut OtherLayoutStyle,
    obj_fits: &mut MultiCaseImpl<Node, ObjectFit>,
) {
    for layout_attr in layout_attrs.iter() {
        match layout_attr {
            Attribute1::AlignContent(r) => {
                if StyleType2::AlignContent as usize & style_mark.local_style2 == 0 {
					other_style.align_content = *r;
                }
            }
            Attribute1::AlignItems(r) => {
                if StyleType2::AlignItems as usize & style_mark.local_style2 == 0 {
					other_style.align_items = *r;
                }
            }
            Attribute1::AlignSelf(r) => {
                if StyleType2::AlignSelf as usize & style_mark.local_style2 == 0 {
					other_style.align_self = *r;
                }
            }
            Attribute1::JustifyContent(r) => {
                if StyleType2::JustifyContent as usize & style_mark.local_style2 == 0 {
					other_style.justify_content = *r;
                }
            }
            Attribute1::FlexDirection(r) => {
                if StyleType2::FlexDirection as usize & style_mark.local_style2 == 0 {
					other_style.flex_direction = *r;
                }
            }
            Attribute1::FlexWrap(r) => {
                if StyleType2::FlexWrap as usize & style_mark.local_style2 == 0 {
					other_style.flex_wrap = *r;
                }
            }
            Attribute1::PositionType(r) => {
                if StyleType2::PositionType as usize & style_mark.local_style2 == 0 {
					other_style.position_type = *r;
                }
            }

            Attribute1::ObjectFit(r) => {
                if style_mark.local_style == 0 & StyleType::ObjectFit as usize {
                    obj_fits.insert_no_notify(id, r.clone());
                    set_dirty(dirty_list, id, StyleType::ObjectFit as usize, style_mark);
                }
            }
            Attribute1::TextAlign(r) => {
                if style_mark.local_style & StyleType::TextAlign as usize == 0 {
                    text_style.text.text_align = *r;
                    set_dirty(dirty_list, id, StyleType::TextAlign as usize, style_mark);
                }
            }
            Attribute1::VerticalAlign(r) => {
                if style_mark.local_style & StyleType::VerticalAlign as usize == 0 {
                    text_style.text.vertical_align = *r;
                    set_dirty(
                        dirty_list,
                        id,
                        StyleType::VerticalAlign as usize,
                        style_mark,
                    );
                }
            }
            Attribute1::WhiteSpace(r) => {
                if style_mark.local_style & StyleType::WhiteSpace as usize == 0 {
                    text_style.text.white_space = *r;
                    set_dirty(dirty_list, id, StyleType::WhiteSpace as usize, style_mark);
                }
            }
            Attribute1::FontStyle(r) => {
                if style_mark.local_style & StyleType::FontStyle as usize == 0 {
                    text_style.font.style = *r;
                    set_dirty(dirty_list, id, StyleType::WhiteSpace as usize, style_mark);
                }
            }
            Attribute1::Enable(r) => {
                if style_mark.local_style1 & StyleType1::Enable as usize == 0 {
                    unsafe{shows.get_unchecked_write(id)}.modify(|show: &mut Show| {
                        show.set_enable(*r);
                        true
                    });
                }
            }
            Attribute1::Display(r) => {
                if style_mark.local_style1 & StyleType1::Display as usize == 0 {
					other_style.display = *r;
                    // layout_style.set_display(unsafe { transmute(*r) });
                    unsafe{shows.get_unchecked_write(id)}.modify(|show: &mut Show| {
                        show.set_display(*r);
                        true
                    });
                }
            }
            Attribute1::Visibility(r) => {
                if style_mark.local_style1 & StyleType1::Visibility as usize == 0 {
                    unsafe{shows.get_unchecked_write(id)}.modify(|show: &mut Show| {
                        show.set_visibility(*r);
                        true
                    });
                }
            }
            Attribute1::Overflow(r) => {
                if style_mark.local_style1 & StyleType1::Overflow as usize == 0 {
                    unsafe{overflows.get_unchecked_write(id)}.modify(
                        |overflow: &mut Overflow| {
                            overflow.0 = *r;
                            true
                        },
                    );
                }
            }
        }
    }
}

pub fn set_attr2<C: HalContext>(
    id: usize,
    dirty_list: &mut DirtyList,
    layout_attrs: &Vec<Attribute2>,
    style_mark: &mut StyleMark,
    text_style: &mut TextStyle,
	rect_layout_style: &mut RectLayoutStyle,
	other_layout_style: &mut OtherLayoutStyle,
    zindexs: &mut MultiCaseImpl<Node, ZIndex>,
    opacitys: &mut MultiCaseImpl<Node, Opacity>,
    border_image_repeats: &mut MultiCaseImpl<Node, BorderImageRepeat>,
    images: &mut MultiCaseImpl<Node, Image>,
    border_images: &mut MultiCaseImpl<Node, BorderImage>,
    image_wait_sheet: &mut SingleCaseImpl<ImageWaitSheet>,
    engine: &mut Engine<C>,
    idtree: &SingleCaseImpl<IdTree>,
    image_clip: Option<&ImageClip>,
    // border_image_clip: Option<&BorderImageClip>,
) {
    for layout_attr in layout_attrs.iter() {
        match layout_attr {
            Attribute2::LetterSpacing(r) => {
                if style_mark.local_style & StyleType::LetterSpacing as usize == 0 && text_style.text.letter_spacing != *r{
                    text_style.text.letter_spacing = *r;
                    set_dirty(
                        dirty_list,
                        id,
                        StyleType::LetterSpacing as usize,
                        style_mark,
                    );
                }
            }
            Attribute2::LineHeight(r) => {
                if style_mark.local_style & StyleType::LineHeight as usize == 0 {
                    text_style.text.line_height = *r;
                    set_dirty(dirty_list, id, StyleType::LineHeight as usize, style_mark);
                }
            }
            Attribute2::TextIndent(r) => {
                if style_mark.local_style & StyleType::Indent as usize == 0 && text_style.text.indent != *r {
                    text_style.text.indent = *r;
                    set_dirty(dirty_list, id, StyleType::Indent as usize, style_mark);
                }
            }
            Attribute2::WordSpacing(r) => {
                if style_mark.local_style & StyleType::WordSpacing as usize == 0 {
                    text_style.text.word_spacing = *r;
                    set_dirty(dirty_list, id, StyleType::WordSpacing as usize, style_mark);
                }
            }
            Attribute2::FontWeight(r) => {
                if style_mark.local_style & StyleType::FontWeight as usize == 0 {
                    text_style.font.weight = *r as usize;
                    set_dirty(dirty_list, id, StyleType::FontWeight as usize, style_mark);
                }
            }
            Attribute2::FontSize(r) => {
                if style_mark.local_style & StyleType::FontSize as usize == 0 {
					text_style.font.size = *r;
                    set_dirty(dirty_list, id, StyleType::FontSize as usize, style_mark);
                }
            }
            Attribute2::FontFamily(r) => {
                if style_mark.local_style & StyleType::FontFamily as usize == 0 {
                    text_style.font.family = *r;
                    set_dirty(dirty_list, id, StyleType::FontFamily as usize, style_mark);
                }
            }
            Attribute2::ZIndex(r) => {
                if style_mark.local_style1 & StyleType1::ZIndex as usize == 0 {
                    zindexs.insert(id, ZIndex(*r));
                }
            }
            Attribute2::Opacity(r) => {
                if style_mark.local_style & StyleType::Opacity as usize == 0 {
                    opacitys.insert(id, r.clone());
                    // set_dirty(dirty_list, event.id, StyleType::Opacity as usize, style_mark); 不需要设脏，opacity还需要通过级联计算得到最终值， 监听到该值的变化才会设脏
                }
            }
            Attribute2::BorderImageRepeat(r) => {
                if style_mark.local_style & StyleType::BorderImageRepeat as usize == 0 {
                    border_image_repeats.insert_no_notify(id, r.clone());
                    set_dirty(
                        dirty_list,
                        id,
                        StyleType::BorderImageRepeat as usize,
                        style_mark,
                    );
                }
            }

            Attribute2::ImageUrl(r) => {
                if style_mark.local_style & StyleType::Image as usize == 0 {
                    let mut image = Image {
                        src: None,
                        url: r.clone(),
                        width: None,
                        height: None,
                    };
                    if let Some(_) = idtree.get(id) {
                        if set_image(
                            id,
                            StyleType::Image,
                            engine,
                            image_wait_sheet,
                            dirty_list,
                            &mut image,
                            style_mark,
                            ImageType::ImageClass,
                        ) {
                            set_image_size(
                                image.src.as_ref().unwrap(),
                                rect_layout_style,
                                image_clip,
                                style_mark,
                            );
                        }
                    }
                    images.insert_no_notify(id, image);
                }
            }
            Attribute2::BorderImageUrl(r) => {
                if style_mark.local_style & StyleType::BorderImage as usize == 0 {
                    let mut image = BorderImage(Image {
                        src: None,
                        url: r.clone(),
                        width: None,
                        height: None,
                    });
                    if let Some(_) = idtree.get(id) {
                        // if
                        set_image(
                            id,
                            StyleType::BorderImage,
                            engine,
                            image_wait_sheet,
                            dirty_list,
                            &mut image.0,
                            style_mark,
                            ImageType::BorderImageClass,
                        );
                        //  {
                        //     set_border_image_size(
                        //         image.0.src.as_ref().unwrap(),
                        //         layout_style,
                        //         border_image_clip,
                        //         style_mark,
                        //     );
                        // }
                    }
                    border_images.insert_no_notify(id, image);
                }
            }

            Attribute2::Width(r) => {
                if StyleType2::Width as usize & style_mark.local_style2 == 0 {
					rect_layout_style.size.width = r.clone();
					set_dirty2(dirty_list, id, StyleType2::Width as usize, style_mark);
                }
            }
            Attribute2::Height(r) => {
                if StyleType2::Height as usize & style_mark.local_style2 == 0 {
					rect_layout_style.size.height = r.clone();
					set_dirty2(dirty_list, id, StyleType2::Height as usize, style_mark);
                }
            }
            Attribute2::MarginLeft(r) => {
                if StyleType2::MarginLeft as usize & style_mark.local_style2 == 0 {
					rect_layout_style.margin.start = r.clone();
					set_dirty2(dirty_list, id, StyleType2::MarginLeft as usize, style_mark);
                }
            }
            Attribute2::MarginTop(r) => {
                if StyleType2::MarginTop as usize & style_mark.local_style2 == 0 {
					rect_layout_style.margin.top = r.clone();
					set_dirty2(dirty_list, id, StyleType2::MarginTop as usize, style_mark);
                }
            }
            Attribute2::MarginBottom(r) => {
                if StyleType2::MarginBottom as usize & style_mark.local_style2 == 0 {
					rect_layout_style.margin.bottom = r.clone();
					set_dirty2(dirty_list, id, StyleType2::MarginBottom as usize, style_mark);
                }
            }
            Attribute2::MarginRight(r) => {
                if StyleType2::MarginRight as usize & style_mark.local_style2 == 0 {
					rect_layout_style.margin.end = r.clone();
					set_dirty2(dirty_list, id, StyleType2::MarginRight as usize, style_mark);
                }
            }
            Attribute2::Margin(r) => {
				if StyleType2::MarginLeft as usize & style_mark.local_style2 == 0 {
					rect_layout_style.margin.start = r.clone();
					set_dirty2(dirty_list, id, StyleType2::MarginLeft as usize, style_mark);
				}
				if StyleType2::MarginTop as usize & style_mark.local_style2 == 0 {
					rect_layout_style.margin.top = r.clone();
					set_dirty2(dirty_list, id, StyleType2::MarginTop as usize, style_mark);
				}
				if StyleType2::MarginBottom as usize & style_mark.local_style2 == 0 {
					rect_layout_style.margin.bottom = r.clone();
					set_dirty2(dirty_list, id, StyleType2::MarginBottom as usize, style_mark);
				}
				if StyleType2::MarginRight as usize & style_mark.local_style2 == 0 {
					rect_layout_style.margin.end = r.clone();
					set_dirty2(dirty_list, id, StyleType2::MarginRight as usize, style_mark);
                }
            }
            Attribute2::PaddingLeft(r) => {
                if StyleType2::PaddingLeft as usize & style_mark.local_style2 == 0 {
					other_layout_style.padding.start = r.clone();
					set_dirty2(dirty_list, id, StyleType2::PaddingLeft as usize, style_mark);
                }
            }
            Attribute2::PaddingTop(r) => {
                if StyleType2::PaddingTop as usize & style_mark.local_style2 == 0 {
					other_layout_style.padding.top = r.clone();
					set_dirty2(dirty_list, id, StyleType2::PaddingTop as usize, style_mark);
                }
            }
            Attribute2::PaddingBottom(r) => {
                if StyleType2::PaddingBottom as usize & style_mark.local_style2 == 0 {
					other_layout_style.padding.bottom = r.clone();
					set_dirty2(dirty_list, id, StyleType2::PaddingBottom as usize, style_mark);
                }
            }
            Attribute2::PaddingRight(r) => {
                if StyleType2::PaddingRight as usize & style_mark.local_style2 == 0 {
					other_layout_style.padding.end = r.clone();
					set_dirty2(dirty_list, id, StyleType2::PaddingRight as usize, style_mark);
                }
            }
            Attribute2::Padding(r) => {
				if StyleType2::PaddingLeft as usize & style_mark.local_style2 == 0 {
					other_layout_style.padding.start = r.clone();
					set_dirty2(dirty_list, id, StyleType2::PaddingLeft as usize, style_mark);
				}
				if StyleType2::PaddingTop as usize & style_mark.local_style2 == 0 {
					other_layout_style.padding.top = r.clone();
					set_dirty2(dirty_list, id, StyleType2::PaddingTop as usize, style_mark);
				}
				if StyleType2::PaddingBottom as usize & style_mark.local_style2 == 0 {
					other_layout_style.padding.bottom = r.clone();
					set_dirty2(dirty_list, id, StyleType2::PaddingBottom as usize, style_mark);
				}
				if StyleType2::PaddingRight as usize & style_mark.local_style2 == 0 {
					other_layout_style.padding.end = r.clone();
					set_dirty2(dirty_list, id, StyleType2::PaddingRight as usize, style_mark);
                }
            }
            Attribute2::BorderLeft(r) => {
                if StyleType2::BorderLeft as usize & style_mark.local_style2 == 0 {
					other_layout_style.border.start = r.clone();
					set_dirty2(dirty_list, id, StyleType2::BorderLeft as usize, style_mark);
                }
            }
            Attribute2::BorderTop(r) => {
                if StyleType2::BorderTop as usize & style_mark.local_style2 == 0 {
					other_layout_style.border.top = r.clone();
					set_dirty2(dirty_list, id, StyleType2::BorderTop as usize, style_mark);
                }
            }
            Attribute2::BorderBottom(r) => {
                if StyleType2::BorderBottom as usize & style_mark.local_style2 == 0 {
					other_layout_style.border.bottom = r.clone();
					set_dirty2(dirty_list, id, StyleType2::BorderBottom as usize, style_mark);
                }
            }
            Attribute2::BorderRight(r) => {
                if StyleType2::BorderRight as usize & style_mark.local_style2 == 0 {
					other_layout_style.border.end = r.clone();
					set_dirty2(dirty_list, id, StyleType2::BorderRight as usize, style_mark);
                }
            }
            Attribute2::Border(r) => {
				if StyleType2::BorderLeft as usize & style_mark.local_style2 == 0 {
					other_layout_style.border.start = r.clone();
					set_dirty2(dirty_list, id, StyleType2::BorderLeft as usize, style_mark);
				}
				if StyleType2::BorderTop as usize & style_mark.local_style2 == 0 {
					other_layout_style.border.top = r.clone();
					set_dirty2(dirty_list, id, StyleType2::BorderTop as usize, style_mark);
				}
				if StyleType2::BorderBottom as usize & style_mark.local_style2 == 0 {
					other_layout_style.border.bottom = r.clone();
					set_dirty2(dirty_list, id, StyleType2::BorderBottom as usize, style_mark);
				}
				if StyleType2::BorderBottom as usize & style_mark.local_style2 == 0 {
					other_layout_style.border.bottom = r.clone();
					set_dirty2(dirty_list, id, StyleType2::BorderBottom as usize, style_mark);
                }
            }
            Attribute2::PositionLeft(r) => {
                if StyleType2::PositionLeft as usize & style_mark.local_style2 == 0 {
					other_layout_style.position.start = r.clone();
					set_dirty2(dirty_list, id, StyleType2::PositionLeft as usize, style_mark);
                }
            }
            Attribute2::PositionTop(r) => {
                if StyleType2::PositionTop as usize & style_mark.local_style2 == 0 {
					other_layout_style.position.top = r.clone();
					set_dirty2(dirty_list, id, StyleType2::PositionTop as usize, style_mark);
                }
            }
            Attribute2::PositionRight(r) => {
                if StyleType2::PositionRight as usize & style_mark.local_style2 == 0 {
					other_layout_style.position.end = r.clone();
					set_dirty2(dirty_list, id, StyleType2::PositionRight as usize, style_mark);
                }
            }
            Attribute2::PositionBottom(r) => {
                if StyleType2::PositionBottom as usize & style_mark.local_style2 == 0 {
					other_layout_style.position.bottom = r.clone();
					set_dirty2(dirty_list, id, StyleType2::PositionBottom as usize, style_mark);
                }
            }
            Attribute2::MinWidth(r) => {
                if StyleType2::MinWidth as usize & style_mark.local_style2 == 0 {
					other_layout_style.min_size.width = r.clone();
					set_dirty2(dirty_list, id, StyleType2::MinWidth as usize, style_mark);
                }
            }
            Attribute2::MinHeight(r) => {
                if StyleType2::MinHeight as usize & style_mark.local_style2 == 0 {
					other_layout_style.min_size.height = r.clone();
					set_dirty2(dirty_list, id, StyleType2::MinHeight as usize, style_mark);
                }
            }
            Attribute2::MaxHeight(r) => {
                if StyleType2::MaxHeight as usize & style_mark.local_style2 == 0 {
					other_layout_style.max_size.height = r.clone();
					set_dirty2(dirty_list, id, StyleType2::MaxHeight as usize, style_mark);
                }
            }
            Attribute2::MaxWidth(r) => {
                if StyleType2::MaxWidth as usize & style_mark.local_style2 == 0 {
					other_layout_style.max_size.width = r.clone();
					set_dirty2(dirty_list, id, StyleType2::MaxWidth as usize, style_mark);
                }
            }
            Attribute2::FlexBasis(r) => {
                if StyleType1::FlexBasis as usize & style_mark.local_style1 == 0 {
					other_layout_style.flex_basis = r.clone();
					set_dirty1(dirty_list, id, StyleType1::FlexBasis as usize, style_mark);
                }
            }
            Attribute2::FlexShrink(r) => {
                if StyleType2::FlexShrink as usize & style_mark.local_style2 == 0 {
					other_layout_style.flex_shrink = *r;
					set_dirty2(dirty_list, id, StyleType2::FlexShrink as usize, style_mark);
                }
            }
            Attribute2::FlexGrow(r) => {
                if StyleType2::FlexGrow as usize & style_mark.local_style2 == 0 {
					other_layout_style.flex_grow = *r;
					set_dirty2(dirty_list, id, StyleType2::FlexGrow as usize, style_mark);
                }
            }
        }
    }
}

pub fn set_attr3(
    id: usize,
    dirty_list: &mut DirtyList,
    attrs: &Vec<Attribute3>,
    style_mark: &mut StyleMark,
    text_style: &mut TextStyle,
    border_image_slices: &mut MultiCaseImpl<Node, BorderImageSlice>,
    border_image_clips: &mut MultiCaseImpl<Node, BorderImageClip>,
    // border_images: &mut MultiCaseImpl<Node, BorderImage>,
    image_clips: &mut MultiCaseImpl<Node, ImageClip>,
    images: &mut MultiCaseImpl<Node, Image>,
    box_shadows: &mut MultiCaseImpl<Node, BoxShadow>,
    background_colors: &mut MultiCaseImpl<Node, BackgroundColor>,
    border_colors: &mut MultiCaseImpl<Node, BorderColor>,
    border_radiuss: &mut MultiCaseImpl<Node, BorderRadius>,
    filters: &mut MultiCaseImpl<Node, Filter>,
    transforms: &mut MultiCaseImpl<Node, Transform>,
    rect_layout_styles: &mut MultiCaseImpl<Node, RectLayoutStyle>,
) {
    for attr in attrs.iter() {
        match attr {
            Attribute3::BGColor(r) => {
                if style_mark.local_style & StyleType::BackgroundColor as usize == 0 {
                    background_colors.insert_no_notify(id, r.clone());
                    set_dirty(
                        dirty_list,
                        id,
                        StyleType::BackgroundColor as usize,
                        style_mark,
                    );
                }
            }
            Attribute3::BorderColor(r) => {
                if style_mark.local_style & StyleType::BorderColor as usize == 0 {
                    border_colors.insert_no_notify(id, r.clone());
                    set_dirty(dirty_list, id, StyleType::BorderColor as usize, style_mark);
                }
            }
            Attribute3::BoxShadow(r) => {
                if style_mark.local_style & StyleType::BoxShadow as usize == 0 {
                    box_shadows.insert_no_notify(id, r.clone());
                    set_dirty(dirty_list, id, StyleType::BoxShadow as usize, style_mark);
                }
            }

            Attribute3::ImageClip(r) => {
                if style_mark.local_style & StyleType::ImageClip as usize == 0 {
					set_dirty(dirty_list, id, StyleType::ImageClip as usize, style_mark);
                    if let Some(image) = images.get(id) {
                        if let Some(src) = &image.src {
                            set_image_size(
                                src,
                                &mut rect_layout_styles[id],
                                Some(r),
                                style_mark,
                            );
                        }
                    }
                    image_clips.insert_no_notify(id, r.clone());
                    
				}
            }

            Attribute3::BorderImageClip(r) => {
                if style_mark.local_style & StyleType::BorderImageClip as usize == 0 {
                    border_image_clips.insert_no_notify(id, r.clone());
                    set_dirty(
                        dirty_list,
                        id,
                        StyleType::BorderImageClip as usize,
                        style_mark,
                    );
                }
            }
            Attribute3::BorderImageSlice(r) => {
                if style_mark.local_style & StyleType::BorderImageSlice as usize == 0 {
                    border_image_slices.insert_no_notify(id, r.clone());
                    set_dirty(
                        dirty_list,
                        id,
                        StyleType::BorderImageSlice as usize,
                        style_mark,
                    );
                }
            }

            Attribute3::Color(r) => {
                if style_mark.local_style & StyleType::Color as usize == 0 {
                    text_style.text.color = r.clone();
                    set_dirty(dirty_list, id, StyleType::Color as usize, style_mark);
                }
            }
            Attribute3::TextShadow(r) => {
                if style_mark.local_style & StyleType::TextShadow as usize == 0 {
                    text_style.shadow = r.clone();
                    set_dirty(dirty_list, id, StyleType::TextShadow as usize, style_mark);
                }
            }
            Attribute3::TextStroke(r) => {
                if style_mark.local_style & StyleType::Stroke as usize == 0 {
                    text_style.text.stroke = r.clone();
                    set_dirty(dirty_list, id, StyleType::Stroke as usize, style_mark);
                }
            }

            Attribute3::BorderRadius(r) => {
                if style_mark.local_style & StyleType::BorderRadius as usize == 0 {
                    border_radiuss.insert_no_notify(id, r.clone());
                    set_dirty(dirty_list, id, StyleType::BorderRadius as usize, style_mark);
                }
            }
            Attribute3::TransformFunc(r) => {
                if style_mark.local_style1 & StyleType1::Transform as usize == 0 {
                    match transforms.get_mut(id) {
                        Some(t) => t.funcs = r.clone(),
                        None => {
                            transforms.insert(
                                id,
                                Transform {
                                    funcs: r.clone(),
                                    origin: TransformOrigin::Center,
                                },
                            );
                        }
                    };
                }
            }
            Attribute3::TransformOrigin(r) => {
                if style_mark.local_style & StyleType::Color as usize == 0 {
                    match transforms.get_mut(id) {
                        Some(t) => t.origin = r.clone(),
                        None => {
                            transforms.insert(
                                id,
                                Transform {
                                    funcs: Vec::default(),
                                    origin: r.clone(),
                                },
                            );
                        }
                    };
                }
            }
            Attribute3::Filter(r) => {
                if style_mark.local_style & StyleType::Filter as usize == 0 {
                    filters.insert(id, r.clone());
                    set_dirty(dirty_list, id, StyleType::Filter as usize, style_mark);
                }
            }
        }
    }
}

fn set_image<C: HalContext>(
    id: usize,
    ty: StyleType,
    engine: &mut Engine<C>,
    image_wait_sheet: &mut ImageWaitSheet,
    dirty_list: &mut DirtyList,
    image: &mut Image,
    style_mark: &mut StyleMark,
    wait_ty: ImageType,
) -> bool {
    if image.src.is_none() {
        match engine.texture_res_map.get(&image.url) {
            Some(r) => {
                image.src = Some(r);
                set_dirty(dirty_list, id, ty as usize, style_mark);
                return true;
            }
            None => {
                image_wait_sheet.add(
                    image.url,
                    ImageWait {
                        id: id,
                        ty: wait_ty,
                    },
                );
                return false;
            }
        }
    } else {
        set_dirty(dirty_list, id, ty as usize, style_mark);
        return true;
    }
}

type ImageTextureWrite<'a, C> = (
    &'a mut MultiCaseImpl<Node, Image>,
    &'a mut MultiCaseImpl<Node, BorderImage>,
    &'a mut MultiCaseImpl<Node, StyleMark>,
    &'a mut SingleCaseImpl<DirtyList>,
    &'a mut SingleCaseImpl<ShareEngine<C>>,
    &'a mut SingleCaseImpl<ImageWaitSheet>,
    &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
    &'a mut MultiCaseImpl<Node, ImageClip>,
    &'a mut MultiCaseImpl<Node, BorderImageClip>,
);
// 节点被添加到树上， 加载图片
fn load_image<'a, C: HalContext>(id: usize, write: &mut ImageTextureWrite<'a, C>) {
    if let Some(image) = write.0.get_mut(id) {
        let style_mark = &mut write.2[id];
        if style_mark.local_style & StyleType::Image as usize != 0 {
            if set_image(
                id,
                StyleType::Image,
                &mut *write.4,
                &mut *write.5,
                &mut *write.3,
                image,
                style_mark,
                ImageType::ImageLocal,
            ) {
                set_image_size(
                    image.src.as_ref().unwrap(),
                    &mut write.6[id],
                    write.7.get(id),
                    style_mark,
                );
            }
        } else {
            if set_image(
                id,
                StyleType::Image,
                &mut *write.4,
                &mut *write.5,
                &mut *write.3,
                image,
                style_mark,
                ImageType::ImageClass,
            ) {
                set_image_size(
                    image.src.as_ref().unwrap(),
                    &mut write.6[id],
                    write.7.get(id),
                    style_mark,
                );
            }
        }
    }
    if let Some(image) = write.1.get_mut(id) {
        let style_mark = &mut write.2[id];
        if style_mark.local_style & StyleType::BorderImage as usize != 0 {
            // if
            set_image(
                id,
                StyleType::BorderImage,
                &mut *write.4,
                &mut *write.5,
                &mut *write.3,
                &mut image.0,
                style_mark,
                ImageType::BorderImageLocal,
            );
        } else {
            // if
            set_image(
                id,
                StyleType::BorderImage,
                &mut *write.4,
                &mut *write.5,
                &mut *write.3,
                &mut image.0,
                style_mark,
                ImageType::BorderImageClass,
            );
        }
    }
}

// 从树上删除节点， 删除节点对图片资源的引用
fn release_image<'a, C: HalContext>(
    id: usize,
    write: &mut ImageTextureWrite<'a, C>,
) {
    if let Some(image) = write.0.get_mut(id) {
        let style_mark = &mut write.2[id];
        if image.src.is_some() {
            image.src = None;
            set_dirty(&mut *write.3, id, StyleType::Image as usize, style_mark);
        }
    }
    if let Some(image) = write.1.get_mut(id) {
        let style_mark = &mut write.2[id];
        if image.0.src.is_some() {
            image.0.src = None;
            set_dirty(
                &mut *write.3,
                id,
                StyleType::BorderImage as usize,
                style_mark,
            );
        }
    }
}

fn set_image_local<'a, C: HalContext>(
    id: usize,
    idtree: &SingleCaseImpl<IdTree>,
    write: ImageWrite<'a, C>,
) {
    let style_mark = &mut write.0[id];
    style_mark.local_style |= StyleType::Image as usize;

    if let Some(_) = idtree.get(id) {
        let image = &mut write.4[id];
        if set_image(
            id,
            StyleType::Image,
            &mut *write.2,
            &mut *write.3,
            &mut *write.1,
            image,
            style_mark,
            ImageType::ImageLocal,
        ) {
            set_image_size(
                image.src.as_ref().unwrap(),
                &mut write.5[id],
                write.6.get(id),
                style_mark,
            );
        }
    }
}

fn set_border_image_local<'a, C: HalContext>(
    id: usize,
    idtree: &SingleCaseImpl<IdTree>,
    write: BorderImageWrite<'a, C>,
) {
    let style_mark = &mut write.0[id];
    style_mark.local_style |= StyleType::BorderImage as usize;

    if let Some(_) = idtree.get(id) {
        let image = &mut write.4[id].0;
        // if
        set_image(
            id,
            StyleType::BorderImage,
            &mut *write.2,
            &mut *write.3,
            &mut *write.1,
            image,
            style_mark,
            ImageType::BorderImageLocal,
        );
    }
}

fn set_mask_image_local<'a, C: HalContext>(
    id: usize,
    idtree: &SingleCaseImpl<IdTree>,
    (style_marks, dirty_list, engine, wait_sheet, mask_images, layout, mask_image_clips): MaskImageWrite<'a, C>,
) {
    let style_mark = &mut style_marks[id];
    style_mark.local_style1 |= StyleType1::MaskImage as usize;

	

    if let Some(_) = idtree.get(id) {
        let image = &mut mask_images[id];
		if image.src.is_none() {
			match engine.texture_res_map.get(&image.url) {
				Some(r) => {
					image.src = Some(r);
					set_dirty1(dirty_list, id, StyleType1::MaskImage as usize, style_mark);
				}
				None => {
					wait_sheet.add(
						image.url,
						ImageWait {
							id: id,
							ty: ImageType::MaskImageLocal,
						},
					);
				}
			}
		} else {
			set_dirty(dirty_list, id, StyleType1::MaskImage as usize, style_mark);
		}
    }
}

impl_system! {
    StyleMarkSys<C> where [C: HalContext + 'static],
    true,
    {
		EntityListener<Node, CreateEvent>
		EntityListener<Node, ModifyEvent>
        MultiCaseListener<Node, TextContent, CreateEvent>
        MultiCaseListener<Node, TextContent, ModifyEvent>
		MultiCaseListener<Node, TextStyle, ModifyEvent>
		MultiCaseListener<Node, RectLayoutStyle, ModifyEvent>
		MultiCaseListener<Node, OtherLayoutStyle, ModifyEvent>

		// MultiCaseListener<Node, MaskImage, ModifyEvent>
        // MultiCaseListener<Node, MaskImageClip, ModifyEvent>
        MultiCaseListener<Node, Image, ModifyEvent>
        MultiCaseListener<Node, ImageClip, ModifyEvent>
        MultiCaseListener<Node, ObjectFit, ModifyEvent>
        MultiCaseListener<Node, BorderImage, ModifyEvent>
        MultiCaseListener<Node, BorderImageClip, ModifyEvent>
        MultiCaseListener<Node, BorderImageSlice, ModifyEvent>
        MultiCaseListener<Node, BorderImageRepeat, ModifyEvent>
        MultiCaseListener<Node, BorderColor, ModifyEvent>
        MultiCaseListener<Node, BackgroundColor, ModifyEvent>
        MultiCaseListener<Node, BoxShadow, ModifyEvent>

		// MultiCaseListener<Node, MaskImage, CreateEvent>
        // MultiCaseListener<Node, MaskImageClip, CreateEvent>
        MultiCaseListener<Node, Image, CreateEvent>
        MultiCaseListener<Node, ImageClip, CreateEvent>
        MultiCaseListener<Node, ObjectFit, CreateEvent>
        MultiCaseListener<Node, BorderImage, CreateEvent>
        MultiCaseListener<Node, BorderImageClip, CreateEvent>
        MultiCaseListener<Node, BorderImageSlice, CreateEvent>
        MultiCaseListener<Node, BorderImageRepeat, CreateEvent>
        MultiCaseListener<Node, BorderColor, CreateEvent>
        MultiCaseListener<Node, BackgroundColor, CreateEvent>
        MultiCaseListener<Node, BoxShadow, CreateEvent>

        MultiCaseListener<Node, WorldMatrix, ModifyEvent>
        MultiCaseListener<Node, ZDepth, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
        MultiCaseListener<Node, LayoutR, ModifyEvent>
        MultiCaseListener<Node, BorderRadius, ModifyEvent>
        MultiCaseListener<Node, Filter, ModifyEvent>
        MultiCaseListener<Node, ByOverflow, ModifyEvent>
		// MultiCaseListener<Node, Visibility, ModifyEvent>
		SingleCaseListener<Oct, ModifyEvent>
		SingleCaseListener<Oct, CreateEvent>
		// SingleCaseListener<Oct, DeleteEvent>

		MultiCaseListener<Node, COpacity, ModifyEvent>

        MultiCaseListener<Node, ClassName, ModifyEvent>
        SingleCaseListener<ImageWaitSheet, ModifyEvent>
        SingleCaseListener<RenderObjs, CreateEvent>
        SingleCaseListener<IdTree, CreateEvent>
        SingleCaseListener<IdTree, DeleteEvent>
    }
}
