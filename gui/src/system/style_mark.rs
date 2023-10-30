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

use ecs::{
    CreateEvent, DeleteEvent, EntityImpl, EntityListener, Event, ModifyEvent, MultiCaseImpl, MultiCaseListener, Runner, SingleCaseImpl,
    SingleCaseListener, StdCell,
};
use flex_layout::*;
use hal_core::*;
use hash::XHashSet;
use share::Share;

use crate::component::calc::*;
use crate::component::calc::{LayoutR, Opacity as COpacity};
use crate::component::user::*;
use crate::component::user::{Opacity, Overflow};
use crate::entity::Node;
use crate::render::engine::{Engine, ShareEngine};
use crate::render::res::TextureRes;
use crate::single::class::*;
use crate::single::IdTree;
use crate::single::*;

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
const FONT_DIRTY: usize =
    StyleType::FontStyle as usize | StyleType::FontFamily as usize | StyleType::FontSize as usize | StyleType::FontWeight as usize;

// 节点属性脏（不包含text， image， background等渲染属性）
const NODE_DIRTY: usize = StyleType::Filter as usize | StyleType::Opacity as usize | StyleType::BorderRadius as usize;
// 节点属性脏（不包含text， image， background等渲染属性）
const NODE_DIRTY1: usize = StyleType1::Visibility as usize
    | StyleType1::Enable as usize
    | StyleType1::ZIndex as usize
    | StyleType1::Transform as usize
    | StyleType1::Display as usize;

const TEXT_STYLE_DIRTY: usize = TEXT_DIRTY | FONT_DIRTY | StyleType::TextShadow as usize;

// 节点属性脏（不包含text， image， background等渲染属性）
const IMAGE_DIRTY: usize = StyleType::Image as usize | StyleType::ImageClip as usize | StyleType::ObjectFit as usize;

const MASK_IMAGE_DIRTY: usize = StyleType1::MaskImage as usize | StyleType1::MaskImageClip as usize;
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

pub struct ClassSetting<C> {
    dirtys: XHashSet<usize>,
    mark: PhantomData<C>,
}
impl<C> ClassSetting<C> {
    pub fn new() -> Self {
        Self {
            dirtys: XHashSet::default(),
            mark: PhantomData,
        }
    }
}

// 将class的设置延迟
impl<'a, C: HalContext + 'static> Runner<'a> for ClassSetting<C> {
    type ReadData = ReadData<'a>;
    type WriteData = WriteData<'a, C>;
    fn run(&mut self, read: Self::ReadData, mut write: Self::WriteData) {
        if self.dirtys.len() > 0 {
            for id in self.dirtys.iter() {
                let id = *id;
                if let Some(class_name) = read.0.get(id) {
                    let mark = &mut write.19[id];
                    let (old_style, old_style1, old_style2) = (mark.class_style, mark.class_style1, mark.class_style2);
                    mark.class_style = 0;
                    mark.class_style1 = 0;
                    mark.class_style2 = 0;

                    if class_name.one > 0 {
                        set_attr(id, class_name.one, read, &mut write);
                        if class_name.two > 0 {
                            set_attr(id, class_name.two, read, &mut write);

                            if class_name.other.len() > 0 {
                                for i in 0..class_name.other.len() {
                                    set_attr(id, class_name.other[i], read, &mut write);
                                }
                            }
                        }
                    }

                    // 重置旧的class中设置的属性
                    if old_style > 0 || old_style1 > 0 || old_style2 > 0 {
                        reset_attr(id, read, &mut write, old_style, old_style1, old_style2);
                    }
                }
            }
            self.dirtys.clear();
        }
    }
}


pub struct StyleMarkSys<C> {
    show: Show,
    mark: PhantomData<C>,
}

impl<'a, C: HalContext + 'static> StyleMarkSys<C> {
    pub fn new() -> Self {
        Self {
            show: Show::default(),
            mark: PhantomData,
        }
    }
}

#[inline]
fn set_local_dirty(dirty_list: &mut DirtyList, id: usize, ty: usize, style_marks: &mut MultiCaseImpl<Node, StyleMark>) {
    let style_mark = &mut style_marks[id];
    set_dirty(dirty_list, id, ty, style_mark);
    style_mark.local_style |= ty;
}

#[inline]
fn set_local_dirty1(dirty_list: &mut DirtyList, id: usize, ty: usize, style_marks: &mut MultiCaseImpl<Node, StyleMark>) {
    let style_mark = match style_marks.get_mut(id) {
        Some(r) => r,
        None => return,
    };
    set_dirty1(dirty_list, id, ty, style_mark);
    style_mark.local_style1 |= ty;
}

#[inline]
fn set_local_dirty2(dirty_list: &mut DirtyList, id: usize, ty: usize, style_marks: &mut MultiCaseImpl<Node, StyleMark>) {
    let style_mark = &mut style_marks[id];
    set_dirty2(dirty_list, id, ty, style_mark);
    style_mark.local_style2 |= ty;
}

#[inline]
pub fn set_dirty(dirty_list: &mut DirtyList, id: usize, ty: usize, style_mark: &mut StyleMark) {
    if style_mark.dirty == 0 && style_mark.dirty1 == 0 && style_mark.dirty2 == 0 {
        dirty_list.0.push(id);
    }
    style_mark.dirty |= ty;
}

#[inline]
pub fn set_dirty1(dirty_list: &mut DirtyList, id: usize, ty: usize, style_mark: &mut StyleMark) {
    if style_mark.dirty == 0 && style_mark.dirty1 == 0 && style_mark.dirty2 == 0 {
        dirty_list.0.push(id);
    }
    style_mark.dirty1 |= ty;
}

#[inline]
pub fn set_dirty2(dirty_list: &mut DirtyList, id: usize, ty: usize, style_mark: &mut StyleMark) {
    if style_mark.dirty == 0 && style_mark.dirty1 == 0 && style_mark.dirty2 == 0 {
        dirty_list.0.push(id);
    }
    style_mark.dirty2 |= ty;
}

impl<'a, C: HalContext + 'static> Runner<'a> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn run(&mut self, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        for id in dirty_list.0.iter() {
            match style_marks.get_mut(*id) {
                Some(style_mark) => {
                    style_mark.dirty = 0;
                    style_mark.dirty1 = 0;
                    style_mark.dirty2 = 0;
                    style_mark.dirty_other = 0;
                }
                None => (),
            }
        }
        dirty_list.0.clear();
    }
}

// 监听TextStyle属性的改变
impl<'a, C: HalContext + 'static> EntityListener<'a, Node, CreateEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut MultiCaseImpl<Node, ClassName>);

    fn listen(&mut self, event: &Event, _read: Self::ReadData, (style_marks, class_names): Self::WriteData) {
        style_marks.insert(event.id, StyleMark::default());
        class_names.insert_no_notify(event.id, ClassName::default());
    }
}

// 监听节点销毁事件，添加到脏列表
impl<'a, C: HalContext + 'static> EntityListener<'a, Node, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);

    fn listen(&mut self, event: &Event, _read: Self::ReadData, (style_marks, dirty_list): Self::WriteData) {
        if let Some(r) = style_marks.get_mut(event.id) {
            set_dirty1(dirty_list, event.id, StyleType1::Delete as usize, r);
        }
    }
}


// 监听TextStyle属性的改变
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, RectLayoutStyle, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);

    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        
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
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, OtherLayoutStyle, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = &'a MultiCaseImpl<Node, OtherLayoutStyle>;
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);

    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
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
            "overflow" => StyleType1::Overflow as usize,
            "flex_basis" => StyleType1::FlexBasis as usize,
            _ => return,
        };
		
        set_local_dirty1(dirty_list, event.id, r as usize, style_marks);
    }
}

// 监听TextStyle属性的改变
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, TextStyle, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);

    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
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
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, TextContent, CreateEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (
        // &'a mut MultiCaseImpl<Node, TextStyle>,
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );

    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (/*text_styles, */ style_marks, dirty_list) = write;
        // text_styles.insert_no_notify(event.id, self.default_text.clone());
        let style_mark = &mut style_marks[event.id];
        set_dirty(dirty_list, event.id, TEXT_STYLE_DIRTY | StyleType::Text as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, TextContent, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);

    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        // let style_mark = &mut style_marks[event.id];
        set_local_dirty(dirty_list, event.id, StyleType::Text as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BlendMode, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);

    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty2(dirty_list, event.id, StyleType2::BlendMode as usize, style_marks);
    }
}


impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, MaskTexture, (CreateEvent, ModifyEvent, DeleteEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _: Self::ReadData, (style_marks, dirty_list): Self::WriteData) {
        if let Some(r) = style_marks.get_mut(event.id) {
            set_dirty1(dirty_list, event.id, StyleType1::MaskTexture as usize, r);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ImageTexture, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
        &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
        &'a mut MultiCaseImpl<Node, ImageClip>,
        &'a mut MultiCaseImpl<Node, ImageTexture>,
    );
    fn listen(&mut self, event: &Event, _: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list, layout_styles, image_clips, image_textures) = write;
        let id = event.id;
        set_dirty1(dirty_list, id, StyleType1::ImageTexture as usize, &mut style_marks[id]);

        if let Some(texture) = image_textures.get(id) {
            if let ImageTexture::All(texture, url) = texture {
                set_image_size(texture, &mut layout_styles[id], image_clips.get(id), &mut style_marks[id]);
            }
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ImageTexture, DeleteEvent> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, idtree: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let id = event.id;
        // 当节点不再树上时，设置脏没有意义
        if let Some(r) = idtree.get(id) {
            if r.layer() > 0 {
                if let Some(style_mark) = style_marks.get_mut(id) {
                    set_dirty1(dirty_list, id, StyleType1::ImageTexture as usize, style_mark);
                }
            }
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImageTexture, (CreateEvent, ModifyEvent, DeleteEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _: Self::ReadData, (style_marks, dirty_list): Self::WriteData) {
        if let Some(r) = style_marks.get_mut(event.id) {
            set_dirty1(dirty_list, event.id, StyleType1::BorderImageTexture as usize, r);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ContentBox, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _: Self::ReadData, (style_marks, dirty_list): Self::WriteData) {
        if let Some(r) = style_marks.get_mut(event.id) {
            set_dirty1(dirty_list, event.id, StyleType1::ContentBox as usize, r);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, MaskImage, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = MaskImageWrite<'a, C>;
    fn listen(&mut self, event: &Event, idtree: Self::ReadData, write: Self::WriteData) { set_mask_image_local(event.id, idtree, write); }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, MaskImage, DeleteEvent> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (&'a mut SingleCaseImpl<DirtyList>, &'a mut MultiCaseImpl<Node, StyleMark>);
    fn listen(&mut self, event: &Event, idtree: Self::ReadData, (dirty_list, style_marks): Self::WriteData) {
        if let Some(r) = style_marks.get_mut(event.id) {
            set_dirty1(dirty_list, event.id, StyleType1::ContentBox as usize, r);
        }
    }
}


impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Image, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
        &'a mut SingleCaseImpl<ShareEngine<C>>,
        &'a mut SingleCaseImpl<ImageWaitSheet>,
        &'a mut MultiCaseImpl<Node, Image>,
        &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
        &'a mut MultiCaseImpl<Node, ImageClip>,
        &'a mut MultiCaseImpl<Node, ImageTexture>,
    );
    fn listen(&mut self, event: &Event, idtree: Self::ReadData, write: Self::WriteData) {
        // set_image_local(event.id, idtree, write);
        let id = event.id;
        set_local_dirty(write.1, id, StyleType::Image as usize, write.0);

        if let Some(n) = idtree.get(id) {
            if n.layer() > 0 {
                let image = &mut write.4[id];
                set_image(id, write.2, write.3, image, write.7, ImageType::ImageLocal);
            }
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Image, DeleteEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, ImageTexture>;
    fn listen(&mut self, event: &Event, _: Self::ReadData, image_textutes: Self::WriteData) {
        let id = event.id;
        image_textutes.delete(id); // border_image删除时，删除对应的纹理set_local_dirty
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ImageClip, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
        &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
        &'a mut MultiCaseImpl<Node, ImageClip>,
        &'a mut MultiCaseImpl<Node, ImageTexture>,
    );
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list, layout_styles, image_clips, image_textures) = write;
        let id = event.id;
        set_local_dirty(dirty_list, id, StyleType::ImageClip as usize, style_marks);

        if let Some(texture) = image_textures.get(id) {
            if let ImageTexture::All(texture, _) = texture {
                set_image_size(texture, &mut layout_styles[id], image_clips.get(id), &mut style_marks[id]);
            }
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BackgroundImageOption, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::ObjectFit as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImageClip, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let id = event.id;
        set_local_dirty(dirty_list, id, StyleType::BorderImageClip as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, MaskImageClip, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let id = event.id;
        set_local_dirty1(dirty_list, id, StyleType1::MaskImageClip as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImage, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
        &'a mut SingleCaseImpl<ShareEngine<C>>,
        &'a mut SingleCaseImpl<ImageWaitSheet>,
        &'a mut MultiCaseImpl<Node, BorderImage>,
        &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
        &'a mut MultiCaseImpl<Node, BorderImageClip>,
        &'a mut MultiCaseImpl<Node, BorderImageTexture>,
    );
    fn listen(&mut self, event: &Event, idtree: Self::ReadData, write: Self::WriteData) {
        let id = event.id;
        set_local_dirty(write.1, id, StyleType::BorderImage as usize, write.0);

        if let Some(_) = idtree.get(id) {
            let image = &mut write.4[id];
            set_border_image(id, write.2, write.3, image, write.7, ImageType::BorderImageLocal);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImage, DeleteEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, BorderImageTexture>;
    fn listen(&mut self, event: &Event, _: Self::ReadData, border_image_textutes: Self::WriteData) {
        let id = event.id;
        border_image_textutes.delete(id); // border_image删除时，删除对应的纹理
    }
}


impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImageRepeat, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderImageRepeat as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderColor, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderColor as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImageSlice, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderImageSlice as usize, style_marks);
    }
}

// type BorderImageWrite<'a, C> = (
//     &'a mut MultiCaseImpl<Node, StyleMark>,
//     &'a mut SingleCaseImpl<DirtyList>,
//     &'a mut SingleCaseImpl<ShareEngine<C>>,
//     &'a mut SingleCaseImpl<ImageWaitSheet>,
//     &'a mut MultiCaseImpl<Node, BorderImage>,
//     &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
//     &'a mut MultiCaseImpl<Node, BorderImageClip>,
// );

// type ImageWrite<'a, C> = (
//     &'a mut MultiCaseImpl<Node, StyleMark>,
//     &'a mut SingleCaseImpl<DirtyList>,
//     &'a mut SingleCaseImpl<ShareEngine<C>>,
//     &'a mut SingleCaseImpl<ImageWaitSheet>,
//     &'a mut MultiCaseImpl<Node, Image>,
//     &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
//     &'a mut MultiCaseImpl<Node, ImageClip>,
// 	&'a mut MultiCaseImpl<Node, ImageTexture>,
// );

type MaskImageWrite<'a, C> = (
    &'a mut MultiCaseImpl<Node, StyleMark>,
    &'a mut SingleCaseImpl<DirtyList>,
    &'a mut SingleCaseImpl<ShareEngine<C>>,
    &'a mut SingleCaseImpl<ImageWaitSheet>,
    &'a mut MultiCaseImpl<Node, MaskImage>,
    &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
    &'a mut MultiCaseImpl<Node, MaskImageClip>,
    &'a mut MultiCaseImpl<Node, MaskTexture>,
);

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BackgroundColor, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BackgroundColor as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BoxShadow, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BoxShadow as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::Opacity as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Blur, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty1(dirty_list, event.id, StyleType1::Blur as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ZIndex, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        if event.field == "class" {
            return;
        }
        let (style_marks, dirty_list) = write;
        set_local_dirty1(dirty_list, event.id, StyleType1::ZIndex as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Transform, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        if event.field == "class" {
            return;
        }
        set_local_dirty1(dirty_list, event.id, StyleType1::Transform as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, TransformWillChange, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty1(dirty_list, event.id, StyleType1::TransformWillChange as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Overflow, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty1(dirty_list, event.id, StyleType1::Overflow as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Show, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let ty = match event.field {
            "display" => StyleType1::Display,
            "enable" => StyleType1::Enable,
            "visibility" => StyleType1::Visibility,
            _ => return,
        };

        set_local_dirty1(dirty_list, event.id, ty as usize, style_marks);
    }
}
// MultiCaseListener<Node, ZIndex, (CreateEvent, ModifyEvent)>
// 		MultiCaseListener<Node, Transform, (CreateEvent, ModifyEvent)>
// 		MultiCaseListener<Node, TransformWillChange, (CreateEvent, ModifyEvent)>
// 		MultiCaseListener<Node, Overflow, (CreateEvent, ModifyEvent)>

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::ByOverflow as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ClipPath, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty1(dirty_list, event.id, StyleType1::ClipPath as usize, style_marks);
    }
}

// // visibility修改， 设置ByOverflow脏（clipsys 使用， dirty上没有位置容纳Visibility脏了， 因此设置在ByOverflow上）
// impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Visibility, ModifyEvent> for StyleMarkSys<C>{
// 	type ReadData = ();
// 	type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
// 	fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData){
// 		let (style_marks, dirty_list) = write;
// 		set_local_dirty(dirty_list, event.id, StyleType::ByOverflow as usize, style_marks);
// 	}
// }

// RenderObjs 创建， 设置ByOverflow脏
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, CreateEvent> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<RenderObjs>;
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, render_objs: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let id = render_objs[event.id].context;
        let style_mark = &mut style_marks[id];
        set_dirty(dirty_list, id, StyleType::ByOverflow as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderRadius, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderRadius as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Filter, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        if event.field == "class" {
            return;
        }
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::Filter as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, COpacity, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = &mut style_marks[event.id];
        set_dirty(dirty_list, event.id, StyleType::Opacity as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, LayoutR, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
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
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, Oct, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    // type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
    // type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
    fn listen(&mut self, event: &Event, _: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = &mut style_marks[event.id];
        set_dirty(dirty_list, event.id, StyleType::Oct as usize, style_mark);
    }
}

// 包围盒修改，
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, Oct, CreateEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    // type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
    // type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
    fn listen(&mut self, event: &Event, _: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = &mut style_marks[event.id];
        set_dirty(dirty_list, event.id, StyleType::Oct as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = &mut style_marks[event.id];
        set_dirty(dirty_list, event.id, StyleType::Matrix as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ZRange, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = &mut style_marks[event.id];
        set_dirty(dirty_list, event.id, StyleType::Matrix as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, IdTree, CreateEvent> for StyleMarkSys<C> {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, NodeState>);
    type WriteData = ImageTextureWrite<'a, C>;
    fn listen(&mut self, event: &Event, (idtree, node_states): Self::ReadData, mut write: Self::WriteData) {
        idtree_create(event.id, &idtree, &node_states, &mut write);
    }
}

fn idtree_create<C: HalContext + 'static>(
    id: usize,
    idtree: &IdTree,
    node_states: &MultiCaseImpl<Node, NodeState>,
    write: &mut ImageTextureWrite<'_, C>,
) {
    let node_state = match node_states.get(id) {
        Some(r) => r,
        None => return,
    };

    if !node_state.0.is_rnode() {
        return;
    }

    load_image(id, write);
    set_local_dirty1(&mut write.3, id, StyleType1::Create as usize, &mut write.2);
    let mark = &mut write.2[id];
    let (dirty, dirty1, dirty2) = (
        mark.local_style | mark.class_style,
        mark.local_style1 | mark.class_style1,
        mark.local_style2 | mark.class_style2,
    );
    mark.dirty |= dirty;
    mark.dirty1 |= dirty1;
    mark.dirty2 |= dirty2;

    let head = idtree[id].children().head;
    for (id, _n) in idtree.iter(head) {
        idtree_create(id, idtree, node_states, write);
    }
}


impl<'a, C: HalContext + 'static> SingleCaseListener<'a, IdTree, DeleteEvent> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ImageTextureWrite<'a, C>;
    fn listen(&mut self, event: &Event, idtree: Self::ReadData, mut write: Self::WriteData) {
        release_image(event.id, &mut write);

        let node = &idtree[event.id];
        for (id, _n) in idtree.recursive_iter(node.children().head) {
            release_image(id, &mut write);
        }
    }
}


type ReadData<'a> = (&'a MultiCaseImpl<Node, ClassName>, &'a SingleCaseImpl<Share<StdCell<ClassSheet>>>);
type WriteData<'a, C> = (
    &'a mut MultiCaseImpl<Node, TextStyle>,
    &'a mut MultiCaseImpl<Node, Image>,
    &'a mut MultiCaseImpl<Node, ImageClip>,
    &'a mut MultiCaseImpl<Node, BackgroundImageOption>,
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
    &'a mut MultiCaseImpl<Node, MaskImage>,
    &'a mut MultiCaseImpl<Node, MaskImageClip>,
    &'a mut MultiCaseImpl<Node, MaskTexture>,
    &'a mut MultiCaseImpl<Node, BlendMode>,
    &'a mut MultiCaseImpl<Node, ImageTexture>,
    &'a mut MultiCaseImpl<Node, BorderImageTexture>,
    &'a mut MultiCaseImpl<Node, Blur>,
	&'a mut MultiCaseImpl<Node, ClipPath>,
);

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ClassName, ModifyEvent> for ClassSetting<C> {
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &Event, _: Self::ReadData, _: Self::WriteData) { self.dirtys.insert(event.id); }
}


// 监听图片等待列表的改变， 将已加载完成的图片设置到对应的组件上
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, ImageWaitSheet, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (
        &'a mut EntityImpl<Node>,
        // &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
        &'a mut MultiCaseImpl<Node, Image>,
        &'a mut MultiCaseImpl<Node, ImageTexture>,
        // &'a mut MultiCaseImpl<Node, ImageClip>,
        &'a mut MultiCaseImpl<Node, BorderImage>,
        &'a mut MultiCaseImpl<Node, BorderImageTexture>,
        &'a mut MultiCaseImpl<Node, MaskImage>,
        &'a mut MultiCaseImpl<Node, MaskTexture>,
        // &'a mut MultiCaseImpl<Node, BorderImageClip>,
        // &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<ImageWaitSheet>,
        // &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, _event: &Event, idtree: Self::ReadData, write: Self::WriteData) {
        let (
            entitys,
            // layout_nodes,
            images,
            image_textures,
            // image_clips,
            border_images,
            border_image_textures,
            // border_image_clips,
            mask_images,
            mask_textures,
            // style_marks,
            image_wait_sheet,
            // dirty_list,
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
                    ImageType::ImageLocal | ImageType::ImageClass => {
                        if let Some(image) = images.get_mut(image_wait.id) {
                            if image.url == wait.0 {
                                image_textures.insert(image_wait.id, ImageTexture::All(wait.1.clone(), image.url));

                                // set_image_size(
                                //     &wait.1,
                                //     &mut layout_nodes[image_wait.id],
                                //     image_clips.get(image_wait.id),
                                //     &mut style_marks[image_wait.id],
                                // );

                                // // if Some image_clips.get(image_wait.id)
                                // set_local_dirty(
                                //     dirty_list,
                                //     image_wait.id,
                                //     StyleType::Image as usize,
                                //     style_marks,
                                // );
                            }
                        }
                    }
                    // ImageType::ImageClass => {
                    //     let style_mark = &mut style_marks[image_wait.id];
                    //     if style_mark.local_style & StyleType::Image as usize != 0 {
                    //         // 本地样式存在Image， 跳过
                    //         continue;
                    //     }
                    //     if let Some(image) = images.get_mut(image_wait.id) {
                    //         if image.url == wait.0 {
                    // 			image_textures.insert(image_wait.id, ImageTexture::All(wait.1.clone()));
                    // 			// set_dirty(
                    //             //     dirty_list,
                    //             //     image_wait.id,
                    //             //     StyleType::Image as usize,
                    //             //     style_mark,
                    //             // );
                    //             // set_image_size(
                    //             //     &wait.1,
                    //             //     &mut layout_nodes[image_wait.id],
                    //             //     image_clips.get(image_wait.id),
                    //             //     style_mark,
                    //             // );
                    //         }
                    //     }
                    // }
                    ImageType::BorderImageLocal | ImageType::BorderImageClass => {
                        if let Some(image) = border_images.get_mut(image_wait.id) {
                            if image.url == wait.0 {
                                border_image_textures.insert(image_wait.id, BorderImageTexture(wait.1.clone()));
                            }
                        }
                    }
                    ImageType::MaskImageLocal | ImageType::MaskImageClass => {
                        if let Some(image) = mask_images.get_mut(image_wait.id) {
                            if let MaskImage::Path(url) = image {
                                if *url == wait.0 {
                                    mask_textures.insert(image_wait.id, MaskTexture::All(wait.1.clone()));
                                }
                            }
                        }
                    } // ImageType::MaskImageClass => {
                      //     let style_mark = &mut style_marks[image_wait.id];
                      //     if style_mark.local_style1 & StyleType1::MaskImage as usize != 0 {
                      //         // 本地样式存在MaskImage， 跳过
                      //         continue;
                      //     }
                      //     if let Some(image) = mask_images.get_mut(image_wait.id) {
                      // 		if let MaskImage::Path(url) = image {
                      // 			if *url == wait.0 {
                      // 				mask_textures.insert(image_wait.id, MaskTexture::All(wait.1.clone()));
                      // 			}
                      // 		}

                      //     }
                      // }
                }
            }
        }
        image_wait_sheet.finish.clear(); // 清空
    }
}

fn set_image_size(src: &Share<TextureRes>, layout_style: &mut RectLayoutStyle, image_clip: Option<&ImageClip>, style_mark: &mut StyleMark) {
    let img_clip;
    let image_clip = match image_clip {
        Some(r) => r,
        None => {
            img_clip = Aabb2::new(Point2::new(0.0, 0.0), Point2::new(1.0, 1.0));
            &img_clip
        }
    };

    if style_mark.local_style2 & (StyleType2::Width as usize) == 0 && style_mark.class_style2 & (StyleType2::Width as usize) == 0 {
        layout_style.size.width = Dimension::Points(src.width as f32 * (image_clip.maxs.x - image_clip.mins.x));
        // set_image_size必然是在样式脏的情况下调用，只需要标记脏类型，无需再次添加到脏列表
        style_mark.dirty2 |= StyleType2::Width as usize;
    }

    if style_mark.local_style2 & (StyleType2::Height as usize) == 0 && style_mark.class_style2 & (StyleType2::Height as usize) == 0 {
        layout_style.size.height = Dimension::Points(src.height as f32 * (image_clip.maxs.y - image_clip.mins.y));
        // set_image_size必然是在样式脏的情况下调用，只需要标记脏类型，无需再次添加到脏列表
        style_mark.dirty2 |= StyleType2::Height as usize;
    }
}


#[inline]
fn reset_attr<C: HalContext>(id: usize, read: ReadData, write: &mut WriteData<C>, old_style: usize, old_style1: usize, old_style2: usize) {
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
        mask_images,
        mask_clips,
        _mask_textures,
        blend_modes,
        _image_textures,
        _border_image_textures,
        blur,
		clip_paths,
    ) = write;

    let rect_layout_style = &mut rect_layout_styles[id];
    let other_layout_style = &mut other_layout_styles[id];
    let style_mark = &mut style_marks[id];
    // old_style中为1， class_style和local_style不为1的属性, 应该删除
    let old_style = !(!old_style | (old_style & (style_mark.class_style | style_mark.local_style)));
    let old_style1 = !(!old_style1 | (old_style1 & (style_mark.class_style1 | style_mark.local_style1)));
    let old_style2 = !(!old_style2 | (old_style2 & (style_mark.class_style2 | style_mark.local_style2)));
    if old_style != 0 {
        if old_style & TEXT_STYLE_DIRTY != 0 {
            let defualt_text = unsafe { &*(&text_styles[0] as *const TextStyle as usize as *const TextStyle) };
            if let Some(text_style) = text_styles.get_mut(id) {
                if old_style & TEXT_DIRTY != 0 {
                    if old_style & StyleType::LetterSpacing as usize != 0 {
                        text_style.text.letter_spacing = defualt_text.text.letter_spacing;
                        set_dirty(dirty_list, id, StyleType::LetterSpacing as usize, style_mark);
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
                        set_dirty(dirty_list, id, StyleType::VerticalAlign as usize, style_mark);
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
            }
            if old_style & StyleType::BorderImageClip as usize != 0 {
                border_image_clips.delete(id);
                set_dirty(dirty_list, id, StyleType::BorderImageClip as usize, style_mark);
            }
            if old_style & StyleType::BorderImageSlice as usize != 0 {
                border_image_slices.delete(id);
                set_dirty(dirty_list, id, StyleType::BorderImageSlice as usize, style_mark);
            }
            if old_style & StyleType::BorderImageRepeat as usize != 0 {
                border_image_repeats.delete(id);
                set_dirty(dirty_list, id, StyleType::BorderImageRepeat as usize, style_mark);
            }
        }

        if old_style & StyleType::BorderColor as usize != 0 {
            border_colors.delete(id);
            set_dirty(dirty_list, id, StyleType::BorderColor as usize, style_mark);
        }

        if old_style & StyleType::BackgroundColor as usize != 0 {
            background_colors.delete(id);
            set_dirty(dirty_list, id, StyleType::BackgroundColor as usize, style_mark);
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

        if old_style1 & StyleType1::MaskImage as usize != 0 {
            mask_images.delete(id);
            set_dirty1(dirty_list, id, StyleType1::MaskImage as usize, style_mark);
        }
        if old_style1 & StyleType1::MaskImageClip as usize != 0 {
            mask_clips.delete(id);
            set_dirty1(dirty_list, id, StyleType1::MaskImageClip as usize, style_mark);
        }

		if old_style1 & StyleType1::ClipPath as usize != 0 {
            clip_paths.delete(id);
            set_dirty1(dirty_list, id, StyleType1::ClipPath as usize, style_mark);
        }

        if old_style2 & StyleType2::BlendMode as usize != 0 {
            blend_modes.delete(id);
            set_dirty2(dirty_list, id, StyleType2::BlendMode as usize, style_mark);
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
                    zindexs.insert_no_notify(id, ZIndex(0));
                    // 字段为class， zindex的监听器不会设置zinde为本地样式
                    zindexs.get_notify_ref().modify_event(id, "class", 0);
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
            if old_style2 & StyleType2::MinWidth as usize != 0 {
                other_layout_style.min_size.width = Dimension::Undefined;
            }
            if old_style2 & StyleType2::MinHeight as usize != 0 {
                other_layout_style.min_size.height = Dimension::Undefined;
            }
            if old_style2 & StyleType2::MaxWidth as usize != 0 {
                other_layout_style.max_size.width = Dimension::Undefined;
            }
            if old_style2 & StyleType2::MaxHeight as usize != 0 {
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

fn set_attr<C: HalContext>(id: usize, class_name: usize, read: ReadData, write: &mut WriteData<C>) {
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
        mask_images,
        mask_image_clips,
        mask_textures,
        blend_modes,
        image_textures,
        border_image_textures,
        blurs,
		clip_paths,
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
        blurs,
        border_image_repeats,
        images,
        image_textures,
        border_images,
        image_wait_sheet,
        engine,
        idtree,
        // image_clips.get(id),
        mask_textures,
        mask_images,
        blend_modes,
        border_image_textures,
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
        // images,
        image_textures,
        box_shadows,
        background_colors,
        border_colors,
        border_radiuss,
        filters,
        transforms,
        rect_layout_styles,
        mask_image_clips,
		clip_paths,
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
    obj_fits: &mut MultiCaseImpl<Node, BackgroundImageOption>,
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
                    if let Some(image_option) = obj_fits.get_mut(id) {
                        image_option.object_fit = r.clone();
                    }
                    set_dirty(dirty_list, id, StyleType::ObjectFit as usize, style_mark);
                }
            }

            Attribute1::BackgroundRepeat(r) => {
                if style_mark.local_style == 0 & StyleType::ObjectFit as usize {
                    if let Some(image_option) = obj_fits.get_mut(id) {
                        image_option.repeat = r.clone();
                    }
                    set_dirty1(dirty_list, id, StyleType1::BackgroundRepeat as usize, style_mark);
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
                    set_dirty(dirty_list, id, StyleType::VerticalAlign as usize, style_mark);
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
                    unsafe { shows.get_unchecked_write(id) }.modify(|show: &mut Show| {
                        show.set_enable(*r);
                        true
                    });
                }
            }
            Attribute1::Display(r) => {
                if style_mark.local_style1 & StyleType1::Display as usize == 0 {
                    other_style.display = *r;
                    // layout_style.set_display(unsafe { transmute(*r) });
                    unsafe { shows.get_unchecked_write(id) }.modify(|show: &mut Show| {
                        show.set_display(*r);
                        true
                    });
                }
            }
            Attribute1::Visibility(r) => {
                if style_mark.local_style1 & StyleType1::Visibility as usize == 0 {
                    unsafe { shows.get_unchecked_write(id) }.modify(|show: &mut Show| {
                        show.set_visibility(*r);
                        true
                    });
                }
            }
            Attribute1::Overflow(r) => {
                if style_mark.local_style1 & StyleType1::Overflow as usize == 0 {
                    unsafe { overflows.get_unchecked_write(id) }.modify(|overflow: &mut Overflow| {
                        overflow.0 = *r;
                        true
                    });
                }
            },
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
    blurs: &mut MultiCaseImpl<Node, Blur>,
    border_image_repeats: &mut MultiCaseImpl<Node, BorderImageRepeat>,
    images: &mut MultiCaseImpl<Node, Image>,
    image_textures: &mut MultiCaseImpl<Node, ImageTexture>,
    border_images: &mut MultiCaseImpl<Node, BorderImage>,
    image_wait_sheet: &mut SingleCaseImpl<ImageWaitSheet>,
    engine: &mut Engine<C>,
    idtree: &SingleCaseImpl<IdTree>,
    // image_clip: Option<&ImageClip>,
    mask_textures: &mut MultiCaseImpl<Node, MaskTexture>,
    mask_images: &mut MultiCaseImpl<Node, MaskImage>,
    blend_modes: &mut MultiCaseImpl<Node, BlendMode>,
    // border_image_clip: Option<&BorderImageClip>,
    border_image_textures: &mut MultiCaseImpl<Node, BorderImageTexture>,
) {
    for layout_attr in layout_attrs.iter() {
        match layout_attr {
            Attribute2::LetterSpacing(r) => {
                if style_mark.local_style & StyleType::LetterSpacing as usize == 0 && text_style.text.letter_spacing != *r {
                    text_style.text.letter_spacing = *r;
                    set_dirty(dirty_list, id, StyleType::LetterSpacing as usize, style_mark);
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
                    zindexs.insert_no_notify(id, ZIndex(*r));
                    zindexs.get_notify_ref().modify_event(id, "class", 0);
                }
            }
            Attribute2::Opacity(r) => {
                if style_mark.local_style & StyleType::Opacity as usize == 0 {
                    opacitys.insert_no_notify(id, r.clone());
                    opacitys.get_notify_ref().modify_event(id, "class", 0);
                    // set_dirty(dirty_list, event.id, StyleType::Opacity as usize, style_mark); 不需要设脏，opacity还需要通过级联计算得到最终值， 监听到该值的变化才会设脏
                }
            }
            Attribute2::Blur(r) => {
                if style_mark.local_style1 & StyleType1::Blur as usize == 0 {
                    blurs.insert_no_notify(id, r.clone());
                    blurs.get_notify_ref().modify_event(id, "class", 0);
                    // set_dirty(dirty_list, event.id, StyleType::Opacity as usize, style_mark); 不需要设脏，opacity还需要通过级联计算得到最终值， 监听到该值的变化才会设脏
                }
            }
            Attribute2::BlendMode(r) => {
                if style_mark.local_style2 & StyleType2::BlendMode as usize == 0 {
                    blend_modes.insert_no_notify(id, r.clone());
                }
            }
            Attribute2::BorderImageRepeat(r) => {
                if style_mark.local_style & StyleType::BorderImageRepeat as usize == 0 {
                    border_image_repeats.insert_no_notify(id, r.clone());
                    set_dirty(dirty_list, id, StyleType::BorderImageRepeat as usize, style_mark);
                }
            }

            Attribute2::ImageUrl(r) => {
                if style_mark.local_style & StyleType::Image as usize == 0 {
                    let mut image = Image { url: r.clone() };
                    if let Some(n) = idtree.get(id) {
                        if n.layer() > 0 {
                            set_image(id, engine, image_wait_sheet, &mut image, image_textures, ImageType::ImageClass);
                        }
                    }
                    images.insert_no_notify(id, image);
                }
            }
            Attribute2::MaskImage(r) => {
                if style_mark.local_style1 & StyleType1::MaskImage as usize == 0 {
                    let mut mask_image = r.clone();

                    if let Some(n) = idtree.get(id) {
                        if n.layer() > 0 {
                            set_mask_image(id, engine, image_wait_sheet, &mut mask_image, mask_textures, ImageType::MaskImageClass);
                        }
                    }
                    mask_images.insert_no_notify(id, mask_image);
                }
            }
            Attribute2::BorderImageUrl(r) => {
                if style_mark.local_style & StyleType::BorderImage as usize == 0 {
                    let mut image = BorderImage { url: r.clone() };
                    if let Some(n) = idtree.get(id) {
                        if n.layer() > 0 {
                            set_border_image(
                                id,
                                engine,
                                image_wait_sheet,
                                &mut image,
                                border_image_textures,
                                ImageType::BorderImageClass,
                            );
                        }

                        // if
                        // set_image(
                        //     id,
                        //     StyleType::BorderImage,
                        //     engine,
                        //     image_wait_sheet,
                        //     dirty_list,
                        //     &mut image.0,
                        //     style_mark,
                        //     ImageType::BorderImageClass,
                        // );
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
    // images: &mut MultiCaseImpl<Node, Image>,
    image_textures: &mut MultiCaseImpl<Node, ImageTexture>,
    box_shadows: &mut MultiCaseImpl<Node, BoxShadow>,
    background_colors: &mut MultiCaseImpl<Node, BackgroundColor>,
    border_colors: &mut MultiCaseImpl<Node, BorderColor>,
    border_radiuss: &mut MultiCaseImpl<Node, BorderRadius>,
    filters: &mut MultiCaseImpl<Node, Filter>,
    transforms: &mut MultiCaseImpl<Node, Transform>,
    rect_layout_styles: &mut MultiCaseImpl<Node, RectLayoutStyle>,
    mask_image_clips: &mut MultiCaseImpl<Node, MaskImageClip>,
	clip_path: &mut MultiCaseImpl<Node, ClipPath>,
) {
    for attr in attrs.iter() {
        match attr {
			Attribute3::ClipPath(r) => {
                if style_mark.local_style1 & StyleType1::ClipPath as usize == 0 {
                    clip_path.insert_no_notify(id, r.clone());
                    set_dirty1(dirty_list, id, StyleType1::ClipPath as usize, style_mark);
                }
            }
            Attribute3::BGColor(r) => {
                if style_mark.local_style & StyleType::BackgroundColor as usize == 0 {
                    background_colors.insert_no_notify(id, r.clone());
                    set_dirty(dirty_list, id, StyleType::BackgroundColor as usize, style_mark);
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
                    if let Some(teture) = image_textures.get(id) {
                        if let ImageTexture::All(src, _) = teture {
                            set_image_size(src, &mut rect_layout_styles[id], Some(r), style_mark);
                        }
                    }
                    image_clips.insert_no_notify(id, r.clone());
                }
            }
            Attribute3::MaskImageClip(r) => {
                if style_mark.local_style1 & StyleType1::MaskImageClip as usize == 0 {
                    set_dirty(dirty_list, id, StyleType1::MaskImageClip as usize, style_mark);
                    mask_image_clips.insert_no_notify(id, r.clone());
                }
            }

            Attribute3::BorderImageClip(r) => {
                if style_mark.local_style & StyleType::BorderImageClip as usize == 0 {
                    border_image_clips.insert_no_notify(id, r.clone());
                    set_dirty(dirty_list, id, StyleType::BorderImageClip as usize, style_mark);
                }
            }
            Attribute3::BorderImageSlice(r) => {
                if style_mark.local_style & StyleType::BorderImageSlice as usize == 0 {
                    border_image_slices.insert_no_notify(id, r.clone());
                    set_dirty(dirty_list, id, StyleType::BorderImageSlice as usize, style_mark);
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
                            transforms.insert_no_notify(
                                id,
                                Transform {
                                    funcs: r.clone(),
                                    origin: TransformOrigin::Center,
                                },
                            );
                        }
                    };
                    transforms.get_notify_ref().modify_event(id, "class", 0);
                }
            }
            Attribute3::TransformOrigin(r) => {
                if style_mark.local_style1 & StyleType1::TransformOrigin as usize == 0 {
                    match transforms.get_mut(id) {
                        Some(t) => t.origin = r.clone(),
                        None => {
                            transforms.insert_no_notify(
                                id,
                                Transform {
                                    funcs: Vec::default(),
                                    origin: r.clone(),
                                },
                            );
                        }
                    };
                    transforms.get_notify_ref().modify_event(id, "class", 0);
                }
            }
            Attribute3::Filter(r) => {
                if style_mark.local_style & StyleType::Filter as usize == 0 {
                    filters.insert_no_notify(id, r.clone());
                    filters.get_notify_ref().modify_event(id, "class", 0);
                    set_dirty(dirty_list, id, StyleType::Filter as usize, style_mark);
                }
            }
        }
    }
}

// fn set_image<C: HalContext>(
//     id: usize,
//     ty: StyleType,
//     engine: &mut Engine<C>,
//     image_wait_sheet: &mut ImageWaitSheet,
//     dirty_list: &mut DirtyList,
//     image: &mut Image,
// 	image_textures: &mut MultiCaseImpl<Node, ImageTexture>,
//     style_mark: &mut StyleMark,
//     wait_ty: ImageType,
// ) -> bool {
//     if image.src.is_none() {
//         match engine.texture_res_map.get(&image.url) {
//             Some(r) => {
//                 image.src = Some(r);
//                 set_dirty(dirty_list, id, ty as usize, style_mark);
//                 return true;
//             }
//             None => {
//                 image_wait_sheet.add(
//                     image.url,
//                     ImageWait {
//                         id: id,
//                         ty: wait_ty,
//                     },
//                 );
//                 return false;
//             }
//         }
//     } else {
//         set_dirty(dirty_list, id, ty as usize, style_mark);
//         return true;
//     }ImageTextureWrite
// }

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
    &'a mut MultiCaseImpl<Node, MaskImage>,
    &'a mut MultiCaseImpl<Node, MaskImageClip>,
    &'a mut MultiCaseImpl<Node, MaskTexture>,
    &'a mut MultiCaseImpl<Node, ImageTexture>,
    &'a mut MultiCaseImpl<Node, BorderImageTexture>,
);
// 节点被添加到树上， 加载图片
fn load_image<'a, C: HalContext>(id: usize, write: &mut ImageTextureWrite<'a, C>) {
    if let Some(image) = write.0.get_mut(id) {
        let style_mark = &mut write.2[id];
        let ty = if style_mark.local_style & StyleType::Image as usize != 0 {
            ImageType::ImageLocal
        } else {
            ImageType::ImageClass
        };
        set_image(id, &mut *write.4, &mut *write.5, image, &mut write.12, ty);
        // if style_mark.local_style & StyleType::Image as usize != 0 {
        // 	set_image(
        //         id,
        //         &mut *write.4,
        //         &mut *write.5,
        //         &mut *write.3,
        //         image,
        // 		&mut write.12,
        //         style_mark,
        //         ImageType::ImageLocal,
        //     );
        //     // if  {
        //     //     set_image_size(
        //     //         image.src.as_ref().unwrap(),
        //     //         &mut write.6[id],
        //     //         write.7.get(id),
        //     //         style_mark,
        //     //     );
        //     // }
        // } else {
        //     if set_image(
        //         id,
        //         StyleType::Image,
        //         &mut *write.4,
        //         &mut *write.5,
        //         &mut *write.3,
        //         image,
        //         style_mark,
        //         ImageType::ImageClass,
        //     ) {
        //         set_image_size(
        //             image.src.as_ref().unwrap(),
        //             &mut write.6[id],
        //             write.7.get(id),
        //             style_mark,
        //         );
        //     }
        // }
    }
    if let Some(image) = write.1.get_mut(id) {
        let style_mark = &mut write.2[id];
        let ty = if style_mark.local_style & StyleType::BorderImage as usize != 0 {
            ImageType::BorderImageLocal
        } else {
            ImageType::BorderImageClass
        };
        set_border_image(id, &mut *write.4, &mut *write.5, image, &mut *write.13, ty);
        // if style_mark.local_style & StyleType::BorderImage as usize != 0 {
        //     // if
        //     set_border_image(
        //         id,
        //         &mut *write.4,
        //         &mut *write.5,
        //         &mut *write.3,
        //         &mut image.0,
        // 		&mut *write.3,
        //         style_mark,
        //         ImageType::BorderImageLocal,
        //     );
        // } else {
        //     // if
        //     set_image(
        //         id,
        //         StyleType::BorderImage,
        //         &mut *write.4,
        //         &mut *write.5,
        //         &mut *write.3,
        //         &mut image.0,
        //         style_mark,
        //         ImageType::BorderImageClass,
        //     );
        // }
    }

    if let Some(image) = write.9.get_mut(id) {
        let style_mark = &mut write.2[id];
        let wait_type = if style_mark.local_style & StyleType1::MaskImage as usize != 0 {
            ImageType::MaskImageLocal
        } else {
            ImageType::MaskImageClass
        };
        set_mask_image(id, &mut *write.4, &mut *write.5, image, &mut *write.11, wait_type);
    }
}

// 从树上删除节点， 删除节点对图片资源的引用
fn release_image<'a, C: HalContext>(id: usize, write: &mut ImageTextureWrite<'a, C>) {
    if let Some(_r) = write.11.get(id) {
        write.11.delete(id);
    }
    if let Some(_r) = write.12.get(id) {
        write.12.delete(id);
    }
    if let Some(_r) = write.13.get(id) {
        write.13.delete(id);
    }

    // if let Some(image) = write.12.get_mut(id) {
    //     let style_mark = &mut write.2[id];
    //     if image.src.is_some() {
    //         image.src = None;
    //         set_dirty(&mut *write.3, id, StyleType::Image as usize, style_mark);
    //     }
    // }
    // if let Some(image) = write.1.get_mut(id) {
    //     let style_mark = &mut write.2[id];
    //     if image.0.src.is_some() {
    //         image.0.src = None;
    //         set_dirty(
    //             &mut *write.3,
    //             id,
    //             StyleType::BorderImage as usize,
    //             style_mark,
    //         );
    //     }
    // }
}


fn set_image<C: HalContext + 'static>(
    id: usize,
    engine: &mut Engine<C>,
    image_wait_sheet: &mut ImageWaitSheet,
    image: &mut Image,
    image_textures: &mut MultiCaseImpl<Node, ImageTexture>,
    wait_ty: ImageType,
) {
    match engine.texture_res_map.get(&image.url) {
        Some(texture) => {
            image_textures.insert(id, ImageTexture::All(texture, image.url));
        }
        None => {
			// if image.url == 1196902338 || image.url == 1483981615 {
			// 	log::info!("set image await==============={:?}", image.url);
			// }
            image_wait_sheet.add(image.url, ImageWait { id, ty: wait_ty });
        }
    }
}

fn set_border_image<C: HalContext + 'static>(
    id: usize,
    engine: &mut Engine<C>,
    image_wait_sheet: &mut ImageWaitSheet,
    image: &mut BorderImage,
    image_textures: &mut MultiCaseImpl<Node, BorderImageTexture>,
    wait_ty: ImageType,
) {
    match engine.texture_res_map.get(&image.url) {
        Some(texture) => {
            image_textures.insert(id, BorderImageTexture(texture));
        }
        None => {
            image_wait_sheet.add(image.url, ImageWait { id, ty: wait_ty });
        }
    }
}

// fn set_image_local<'a, C: HalContext>(
//     id: usize,
//     idtree: &SingleCaseImpl<IdTree>,
//     write: ImageWrite<'a, C>,
// ) {
//     let style_mark = &mut write.0[id];
//     style_mark.local_style |= StyleType::Image as usize;
// 	set_dirty(&mut *write.1, id, StyleType::Image as usize, style_mark);

//     if let Some(_) = idtree.get(id) {
//         let image = &mut write.4[id];
//         if set_image(
//             id,
//             StyleType::Image,
//             &mut *write.2,
//             &mut *write.3,
//             &mut *write.1,
//             image,
//             style_mark,
//             ImageType::ImageLocal,
//         ) {
//             set_image_size(
//                 image.src.as_ref().unwrap(),
//                 &mut write.5[id],
//                 write.6.get(id),
//                 style_mark,
//             );
//         }
//     }
// }


// fn set_border_image_local<'a, C: HalContext>(
//     id: usize,
//     idtree: &SingleCaseImpl<IdTree>,
//     write: BorderImageWrite<'a, C>,
// ) {
//     let style_mark = &mut write.0[id];
//     style_mark.local_style |= StyleType::BorderImage as usize;

//     if let Some(_) = idtree.get(id) {
//         let image = &mut write.4[id].0;
//         // if
//         set_image(
//             id,
//             StyleType::BorderImage,
//             &mut *write.2,
//             &mut *write.3,
//             &mut *write.1,
//             image,
//             style_mark,
//             ImageType::BorderImageLocal,
//         );
//     }
// }

fn set_mask_image_local<'a, C: HalContext>(
    id: usize,
    idtree: &SingleCaseImpl<IdTree>,
    (style_marks, _dirty_list, engine, wait_sheet, mask_images, _layout, _mask_image_clips, mask_texture): MaskImageWrite<'a, C>,
) {
    let style_mark = &mut style_marks[id];
    style_mark.local_style1 |= StyleType1::MaskImage as usize;

    if let Some(_) = idtree.get(id) {
        let image = &mut mask_images[id];
        set_mask_image(id, engine, wait_sheet, image, mask_texture, ImageType::MaskImageLocal);
    }
}

fn set_mask_image<C: HalContext>(
    id: usize,
    engine: &mut Engine<C>,
    image_wait_sheet: &mut ImageWaitSheet,
    image: &mut MaskImage,
    texure: &mut MultiCaseImpl<Node, MaskTexture>,
    wait_ty: ImageType,
) {
    if let MaskImage::Path(url) = image {
        if texure.get(id).is_none() {
            match engine.texture_res_map.get(url) {
                Some(r) => {
                    texure.insert(id, MaskTexture::All(r.clone()));
                }
                None => {
                    image_wait_sheet.add(*url, ImageWait { id: id, ty: wait_ty });
                }
            }
        }
    }
}

impl_system! {
    StyleMarkSys<C> where [C: HalContext + 'static],
    true,
    {
        EntityListener<Node, CreateEvent>
        EntityListener<Node, ModifyEvent>
        MultiCaseListener<Node, TextStyle, ModifyEvent>
        MultiCaseListener<Node, RectLayoutStyle, ModifyEvent>
        MultiCaseListener<Node, OtherLayoutStyle, ModifyEvent>

        MultiCaseListener<Node, TextContent, CreateEvent>
        MultiCaseListener<Node, TextContent, ModifyEvent>

        MultiCaseListener<Node, BlendMode, (CreateEvent, ModifyEvent)>

        MultiCaseListener<Node, MaskImage, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, MaskImage, DeleteEvent>
        MultiCaseListener<Node, MaskImageClip, (CreateEvent, ModifyEvent)>

        MultiCaseListener<Node, MaskTexture, (CreateEvent, ModifyEvent, DeleteEvent)>
        MultiCaseListener<Node, ImageTexture, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, ImageTexture, DeleteEvent>
        MultiCaseListener<Node, BorderImageTexture, (CreateEvent, ModifyEvent, DeleteEvent)>

        MultiCaseListener<Node, Image, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, Image, DeleteEvent>
        MultiCaseListener<Node, ImageClip, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, BackgroundImageOption, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, BorderImage, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, BorderImage, DeleteEvent>
        MultiCaseListener<Node, BorderImageClip, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, BorderImageSlice, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, BorderImageRepeat, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, BorderColor, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, BackgroundColor, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, BoxShadow, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, ZIndex, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, Transform, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, TransformWillChange, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, Overflow, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, ContentBox, (CreateEvent, ModifyEvent)>
		MultiCaseListener<Node, ClipPath, (CreateEvent, ModifyEvent)>

        MultiCaseListener<Node, WorldMatrix, ModifyEvent>
        MultiCaseListener<Node, ZRange, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
        MultiCaseListener<Node, Blur, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, LayoutR, ModifyEvent>
        MultiCaseListener<Node, BorderRadius, ModifyEvent>
        MultiCaseListener<Node, Filter, ModifyEvent>
        MultiCaseListener<Node, ByOverflow, ModifyEvent>
        // MultiCaseListener<Node, Visibility, ModifyEvent>
        SingleCaseListener<Oct, ModifyEvent>
        SingleCaseListener<Oct, CreateEvent>
        // SingleCaseListener<Oct, DeleteEvent>

        MultiCaseListener<Node, COpacity, ModifyEvent>

        MultiCaseListener<Node, Show, ModifyEvent>

        SingleCaseListener<ImageWaitSheet, ModifyEvent>
        SingleCaseListener<RenderObjs, CreateEvent>
        SingleCaseListener<IdTree, CreateEvent>
        SingleCaseListener<IdTree, DeleteEvent>
    }
}

// // reset_value_del!(display, other_layout_style, display, "1", Display);
// // reset_value_del!(visibility, , visibility, "1", Visibility);
// // reset_value_del!(enable, "1", Enable);
// reset_value_del!(z_index, "1", ZIndex);
// reset_value_del!(transform, "1", Transform);
// reset_value_del!(transform_will_change, "1", TransformWillChange);
// reset_value_del!(overflow, "1", Overflow);

impl_system! {
    ClassSetting<C> where [C: HalContext + 'static],
    true,
    {
        MultiCaseListener<Node, ClassName, ModifyEvent>
    }
}
