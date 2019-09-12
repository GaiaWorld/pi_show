/**
 * 样式标记
 * StyleMarkSys系统会在Node实体创建时， 自动为Node创建一个StyleMark组件， 该组件用于标记了各种样式脏、是否为本地样式
 * StyleMarkSys系统会监听本地样式的修改，以标记样式的脏， 并覆盖class设置的样式属性（覆盖的方式为：修改该属性的本地样式标记为1）
 * StyleMarkSys系统会监听ClassName的修改， 遍历class中的属性， 如果该属性没有设置本地样式，将覆盖该属性对应的组件，并标记样式脏
 * class中的图片， 是一个url， 在设置class时， 该图片资源可能还未加载， StyleMarkSys会将不存在的图片url放入ImageWaitSheet中， 由外部处理ImageWaitSheet中的等待列表，图片加载完成， 应该将图片放入完成列表中， 并通知ImageWaitSheet修改， 由StyleMarkSys来处理ImageWaitSheet中的完成列表
 * StyleMarkSys系统监听ImageWaitSheet单例的修改， 将完成加载的图片设置在对应的Node组件上， 并标记样式脏
 */
use std::marker::PhantomData;

use ecs::{CreateEvent, ModifyEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, EntityImpl, Runner};
use hal_core::*;

use component::user::*;
use component::calc::*;
use component::calc::Opacity as COpacity;
use component::user::Opacity;
use single::class::*;
use single::*;
use layout::*;
use entity::{Node};
use render::engine::Engine;

//文字样式脏
const TEXT_DIRTY: usize =       StyleType::LetterSpacing as usize | 
                                StyleType::WordSpacing as usize | 
                                StyleType::LineHeight as usize | 
                                StyleType::Indent as usize |
                                StyleType::WhiteSpace as usize | 
                                StyleType::TextAlign as usize | 
                                StyleType::VerticalAlign as usize |
                                StyleType::TextShadow as usize |
                                StyleType::Color as usize | 
                                StyleType::Stroke as usize;

//字体脏
const FONT_DIRTY: usize =       StyleType::FontStyle as usize | 
                                StyleType::FontFamily as usize | 
                                StyleType::FontSize as usize | 
                                StyleType::FontWeight as usize;

// 节点属性脏（不包含text， image， background等渲染属性）
const NODE_DIRTY: usize =       StyleType::Filter as usize | 
                                StyleType::Opacity as usize | 
                                StyleType::BorderRadius as usize; 
// 节点属性脏（不包含text， image， background等渲染属性）
const NODE_DIRTY1: usize =      StyleType1::Visibility as usize | 
                                StyleType1::Enable as usize | 
                                StyleType1::ZIndex as usize |
                                StyleType1::Transform as usize | 
                                StyleType1::Display as usize;

const TEXT_STYLE_DIRTY: usize = TEXT_DIRTY | FONT_DIRTY | StyleType::TextShadow as usize;    

// 节点属性脏（不包含text， image， background等渲染属性）
const IMAGE_DIRTY: usize =      StyleType::Image as usize | 
                                StyleType::ImageClip as usize | 
                                StyleType::ObjectFit as usize;
// 节点属性脏（不包含text， image， background等渲染属性）
const BORDER_IMAGE_DIRTY: usize =       StyleType::BorderImage as usize | 
                                        StyleType::BorderImageClip as usize | 
                                        StyleType::BorderImageSlice as usize |
                                        StyleType::BorderImageRepeat as usize;
// 布局脏
const LAYOUT_DIRTY: usize = StyleType1::Width as usize |
                            StyleType1::Height as usize | 
                            StyleType1::Margin as usize | 
                            StyleType1::Padding as usize |
                            StyleType1::Border as usize |
                            StyleType1::Position as usize |
                            StyleType1::MinWidth as usize |
                            StyleType1::MinHeight as usize |
                            StyleType1::MaxHeight as usize |
                            StyleType1::MaxWidth as usize |
                            StyleType1::FlexBasis as usize |
                            StyleType1::FlexShrink as usize |
                            StyleType1::FlexGrow as usize |
                            StyleType1::PositionType as usize |
                            StyleType1::FlexWrap as usize |
                            StyleType1::FlexDirection as usize |
                            StyleType1::AlignContent as usize |
                            StyleType1::AlignItems as usize |
                            StyleType1::AlignSelf as usize |
                            StyleType1::JustifyContent as usize;

pub struct StyleMarkSys<L, C>{
    text_style: TextStyle,
    default_text: TextStyle,
    show: Show,
    mark: PhantomData<(L, C)>,
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> StyleMarkSys<L, C> {
    pub fn new() -> Self {
        Self{
            text_style: TextStyle::default(),
            default_text: TextStyle::default(),
            show: Show::default(),
            mark: PhantomData,
        }
    }
}

#[inline]
fn set_local_dirty(dirty_list: &mut DirtyList, id: usize, ty: usize, style_marks: &mut MultiCaseImpl<Node, StyleMark> ) {
    let style_mark = unsafe { style_marks.get_unchecked_mut(id) };
    if style_mark.dirty == 0 {
        dirty_list.0.push(id);
    }
    style_mark.dirty |= ty;
    style_mark.local_style |= ty;
}

#[inline]
fn set_local_dirty1(dirty_list: &mut DirtyList, id: usize, ty: usize, style_marks: &mut MultiCaseImpl<Node, StyleMark> ) {
    let style_mark = unsafe { style_marks.get_unchecked_mut(id) };
    if style_mark.dirty == 0 {
        dirty_list.0.push(id);
    }
    style_mark.dirty |= ty;
    style_mark.local_style1 |= ty;
}

#[inline]
fn set_dirty(dirty_list: &mut DirtyList, id: usize, ty: usize, style_mark: &mut StyleMark) {
    if style_mark.dirty == 0 {
        dirty_list.0.push(id);
    }
    style_mark.dirty |= ty;
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> Runner<'a> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn run(&mut self, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        for id in dirty_list.0.iter() {
            match style_marks.get_mut(*id) {
                Some(style_mark) => style_mark.dirty = 0,
                None => (),
            }
        }
        dirty_list.0.clear();
    }
}

// 监听TextStyle属性的改变
impl<'a, L: FlexNode + 'static, C: HalContext + 'static> EntityListener<'a, Node, CreateEvent> for StyleMarkSys<L, C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut MultiCaseImpl<Node, ClassName>);

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, class_names) = write;
        style_marks.insert(event.id, StyleMark::default());
        class_names.insert_no_notify(event.id, ClassName::default());
    }
}

// 监听TextStyle属性的改变
impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, TextStyle, ModifyEvent> for StyleMarkSys<L, C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        
        let r = match event.field {
            "letter_spacing" => StyleType::LetterSpacing,
            "word_spacing" => StyleType::WordSpacing,
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
            _ => return
        };
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, r as usize, style_marks);
    }
}

// 监听TextContente属性的改变
impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, TextContent, CreateEvent> for StyleMarkSys<L, C> {
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, TextStyle>,
        &'a mut MultiCaseImpl<Node, StyleMark>, 
        &'a mut SingleCaseImpl<DirtyList>
    );

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (text_styles, style_marks, dirty_list) = write;
        text_styles.insert_no_notify(event.id, self.default_text.clone());
        let style_mark = unsafe { style_marks.get_unchecked_mut(event.id) };
        set_dirty(dirty_list, event.id, TEXT_STYLE_DIRTY | StyleType::Text as usize, style_mark);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, TextContent, ModifyEvent> for StyleMarkSys<L, C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = unsafe { style_marks.get_unchecked_mut(event.id) };
        set_dirty(dirty_list, event.id, StyleType::Text as usize, style_mark);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, Image, ModifyEvent> for StyleMarkSys<L, C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::Image as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, ImageClip, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::ImageClip as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, ObjectFit, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::ObjectFit as usize, style_marks);
    }
}


impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImageClip, ModifyEvent> for StyleMarkSys<L, C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderImageClip as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImageSlice, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderImageSlice as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImage, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderImage as usize, style_marks);
    }
}


impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImageRepeat, ModifyEvent> for StyleMarkSys<L, C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderImageRepeat as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderColor, CreateEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderColor as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderColor, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderColor as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BackgroundColor, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BackgroundColor as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BoxShadow, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BoxShadow as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, Image, CreateEvent> for StyleMarkSys<L, C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::Image as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, ImageClip, CreateEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::ImageClip as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, ObjectFit, CreateEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::ObjectFit as usize, style_marks);
    }
}


impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImageClip, CreateEvent> for StyleMarkSys<L, C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderImageClip as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImageSlice, CreateEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderImageSlice as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImage, CreateEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderImage as usize, style_marks);
    }
}


impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImageRepeat, CreateEvent> for StyleMarkSys<L, C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderImageRepeat as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BackgroundColor, CreateEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BackgroundColor as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BoxShadow, CreateEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BoxShadow as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::Opacity as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::ByOverflow as usize, style_marks);
    }
}

// visibility修改， 设置ByOverflow脏（clipsys 使用， dirty上没有位置容纳Visibility脏了， 因此设置在ByOverflow上）
impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, Visibility, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::ByOverflow as usize, style_marks);
    }
}

// RenderObjs 创建， 设置ByOverflow脏
impl<'a, L: FlexNode + 'static, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, CreateEvent> for StyleMarkSys<L, C> {
    type ReadData = &'a SingleCaseImpl<RenderObjs>;
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &CreateEvent, render_objs: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        let id = unsafe { render_objs.get_unchecked(event.id) }.context;
        let style_mark = unsafe { style_marks.get_unchecked_mut(id) };
        set_dirty(dirty_list, id, StyleType::ByOverflow as usize, style_mark);
    }
}

// impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, Transform, ModifyEvent> for StyleMarkSys<L, C>{
//     type ReadData = ();
//     type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
//     fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
//         let (style_marks, dirty_list) = write;
//         set_local_dirty1(dirty_list, event.id, StyleType1::Transform as usize, style_marks);
//     }
// }

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderRadius, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderRadius as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, Filter, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::Filter as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, COpacity, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        let style_mark = unsafe { style_marks.get_unchecked_mut(event.id) };
        set_dirty(dirty_list, event.id, StyleType::Opacity as usize, style_mark);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, Layout, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        let style_mark = unsafe { style_marks.get_unchecked_mut(event.id) };
        set_dirty(dirty_list, event.id, StyleType::Layout as usize, style_mark);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        let style_mark = unsafe { style_marks.get_unchecked_mut(event.id) };
        set_dirty(dirty_list, event.id, StyleType::Matrix as usize, style_mark);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, ZDepth, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        let style_mark = unsafe { style_marks.get_unchecked_mut(event.id) };
        set_dirty(dirty_list, event.id, StyleType::Matrix as usize, style_mark);
    }
}

type ReadData<'a> = (
    &'a MultiCaseImpl<Node, ClassName>,
    &'a SingleCaseImpl<ClassSheet>
);
type WriteData<'a, L, C> = (
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
    &'a mut MultiCaseImpl<Node, StyleMark>,
    &'a mut MultiCaseImpl<Node, L>,
    &'a mut SingleCaseImpl<Engine<C>>,
    &'a mut SingleCaseImpl<ImageWaitSheet>,
    &'a mut SingleCaseImpl<DirtyList>,
);

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, ClassName, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ReadData<'a>;
    type WriteData = WriteData<'a, L, C>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, mut write: Self::WriteData){
        let (class_names, class_sheet) = read;
        let class_name = unsafe { class_names.get_unchecked(event.id) };

        let old = unsafe { Box::from_raw(event.index as *mut ClassName) };
        let mark = unsafe { write.18.get_unchecked_mut(event.id) };
        let (old_style, old_style1) = (mark.class_style, mark.class_style1);
        mark.class_style = 0;
        mark.class_style1 = 0;

        if class_name.one > 0 {
            if old.one != class_name.one {
                set_attr(event.id, class_name.one, &mut self.text_style, &mut self.show, read, &mut write);
                if class_name.two > 0 {
                    set_attr(event.id, class_name.two, &mut self.text_style, &mut self.show, read, &mut write);
                }
                for i in 0..class_name.other.len() {
                    set_attr(event.id, class_name.other[i], &mut self.text_style, &mut self.show, read, &mut write);
                }
            } else {
                set_mark(class_sheet, class_name.one, mark);
                if class_name.two > 0 {
                    if old.two != class_name.two {
                        set_attr(event.id, class_name.two, &mut self.text_style, &mut self.show, read, &mut write);
                        for i in 0..class_name.other.len() {
                            set_attr(event.id, class_name.other[i], &mut self.text_style, &mut self.show, read, &mut write);
                        }
                    } else {
                        set_mark(class_sheet, class_name.two, mark);
                        let mut index = 0;
                        // 跳过class id相同的项
                        for i in 0..class_name.other.len() {
                            match old.other.get(i) {
                                Some(r) => if *r == class_name.other[i]{
                                    set_mark(class_sheet, class_name.other[i], mark);
                                    index += 1;
                                }else {
                                    break;
                                },
                                None => break,
                            } 
                        }
                        // 设置class属性
                        for i in index..class_name.other.len() {
                            set_attr(event.id, class_name.other[i], &mut self.text_style, &mut self.show, read, &mut write);
                        }
                    }
                }
            }
            
        }

        
        // 重置旧的class中设置的属性
        if old_style > 0 || old_style1 > 0 {
            reset_attr(event.id, read, &mut write, old_style, old_style1, &self.default_text);
        }
    }
}

// 监听图片等待列表的改变， 将已加载完成的图片设置到对应的组件上
impl<'a, L: FlexNode + 'static, C: HalContext + 'static> SingleCaseListener<'a, DefaultTable, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData =  &'a SingleCaseImpl<DefaultTable>;
    type WriteData = ();
    fn listen(&mut self, _: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData){
        self.default_text = read.get::<TextStyle>().unwrap().clone();
    }
}

// 监听图片等待列表的改变， 将已加载完成的图片设置到对应的组件上
impl<'a, L: FlexNode + 'static, C: HalContext + 'static> SingleCaseListener<'a, ImageWaitSheet, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ClassName>,
        &'a SingleCaseImpl<ClassSheet>
    );
    type WriteData = (
        &'a mut EntityImpl<Node>,
        &'a mut MultiCaseImpl<Node, Image>,
        &'a mut MultiCaseImpl<Node, BorderImage>,
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<ImageWaitSheet>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, _: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        // let (class_names, class_sheet) = read;
        let (
            entitys,
            images,
            border_images,
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
                // 判断等待类型， 设置对应的组件
                match image_wait.ty {
                    ImageType::ImageLocal => {
                        images.insert(image_wait.id, Image{src: wait.1.clone(), url: wait.0.clone()});
                        set_local_dirty(dirty_list, image_wait.id, StyleType::Image as usize, style_marks);
                    },
                    ImageType::ImageClass => {
                        let style_mark = unsafe { style_marks.get_unchecked_mut(image_wait.id) };
                        if style_mark.local_style & StyleType::Image as usize != 0 { // 本地样式存在Image， 跳过
                            continue;
                        }
                        // 判断该图片是否应该插入到组件中， TODO   
                        images.insert_no_notify(image_wait.id, Image{src: wait.1.clone(), url: wait.0.clone()});
                        set_dirty(dirty_list, image_wait.id, StyleType::Image as usize, style_mark);
                    },
                    ImageType::BorderImageLocal => {
                        border_images.insert(image_wait.id, BorderImage{src: wait.1.clone(), url: wait.0.clone()});
                        set_local_dirty(dirty_list, image_wait.id, StyleType::BorderImage as usize, style_marks);
                    },
                    ImageType::BorderImageClass => {
                        let style_mark = unsafe { style_marks.get_unchecked_mut(image_wait.id) };
                        if style_mark.local_style & StyleType::BorderImage as usize != 0 { // 本地样式存在BorderImage， 跳过
                            continue;
                        }
                        // 判断该图片是否应该插入到组件中， TODO                 
                        border_images.insert(image_wait.id, BorderImage{src: wait.1.clone(), url: wait.0.clone()});
                        set_dirty(dirty_list, image_wait.id, StyleType::BorderImage as usize, style_mark);
                    },
                }
            }
        }
        image_wait_sheet.finish.clear(); // 清空
    }
}

#[inline]
fn reset_attr<L: FlexNode, C: HalContext>(
    id: usize,
    read: ReadData,
    write: &mut WriteData<L, C>,
    old_style: usize,
    old_style1: usize,
    defualt_text: &TextStyle,
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
        style_marks,
        yogas,
        _engine,
        _image_wait_sheet,
        dirty_list,
        ) = write;
    
    let style_mark = unsafe { style_marks.get_unchecked_mut(id) };
    // old_style中为1， class_style和local_style不为1的属性, 应该删除
    let old_style = !(!old_style | (old_style & (style_mark.class_style | style_mark.local_style) ));
    let old_style1 = !(!old_style1 | (old_style1 & (style_mark.class_style1 | style_mark.local_style1)));
    if old_style != 0 {
        if style_mark.local_style & TEXT_STYLE_DIRTY == 0 {
            if let Some(text_style) = text_styles.get_mut(id) {
                if style_mark.local_style & TEXT_DIRTY == 0 {
                    if old_style & StyleType::LetterSpacing as usize == 0 {
                        text_style.text.letter_spacing = defualt_text.text.letter_spacing;
                        set_dirty(dirty_list, id, StyleType::LetterSpacing as usize, style_mark);
                    }
                    if old_style & StyleType::WordSpacing as usize == 0 {
                        text_style.text.word_spacing = defualt_text.text.word_spacing;
                        set_dirty(dirty_list, id, StyleType::WordSpacing as usize, style_mark);
                    }
                    if old_style & StyleType::LineHeight as usize == 0 {
                        text_style.text.line_height = defualt_text.text.line_height;
                        set_dirty(dirty_list, id, StyleType::LineHeight as usize, style_mark);
                    }
                    if old_style & StyleType::Indent as usize == 0 {
                        text_style.text.indent = defualt_text.text.indent;
                        set_dirty(dirty_list, id, StyleType::Indent as usize, style_mark);
                    }
                    if old_style & StyleType::WhiteSpace as usize == 0 {
                        text_style.text.white_space = defualt_text.text.white_space;
                        set_dirty(dirty_list, id, StyleType::WhiteSpace as usize, style_mark);
                    }

                    if old_style & StyleType::Color as usize == 0 {
                        text_style.text.color = defualt_text.text.color.clone();
                        set_dirty(dirty_list, id, StyleType::Color as usize, style_mark);
                    }

                    if old_style & StyleType::Stroke as usize == 0 {
                        text_style.text.stroke = defualt_text.text.stroke.clone();
                        set_dirty(dirty_list, id, StyleType::Stroke as usize, style_mark);
                    }

                    if old_style & StyleType::TextAlign as usize == 0 {
                        text_style.text.text_align = defualt_text.text.text_align;
                        set_dirty(dirty_list, id, StyleType::TextAlign as usize, style_mark);
                    }

                    if old_style & StyleType::VerticalAlign as usize == 0 {
                        text_style.text.vertical_align = defualt_text.text.vertical_align;
                        set_dirty(dirty_list, id, StyleType::VerticalAlign as usize, style_mark);
                    }
                }

                if old_style & StyleType::TextShadow as usize == 0 {
                    text_style.shadow = defualt_text.shadow.clone();
                    set_dirty(dirty_list, id, StyleType::TextShadow as usize, style_mark);
                }

                if old_style & FONT_DIRTY == 0 {
                    if old_style & StyleType::FontStyle as usize == 0 {
                        text_style.font.style = defualt_text.font.style;
                        set_dirty(dirty_list, id, StyleType::FontStyle as usize, style_mark);
                    }
                    if old_style & StyleType::FontWeight as usize == 0 {
                        text_style.font.weight = defualt_text.font.weight;
                        set_dirty(dirty_list, id, StyleType::FontWeight as usize, style_mark);
                    }
                    if old_style & StyleType::FontSize as usize == 0 {
                        text_style.font.size = defualt_text.font.size;
                        set_dirty(dirty_list, id, StyleType::FontSize as usize, style_mark);
                    }
                    if old_style & StyleType::FontFamily as usize == 0 {
                        text_style.font.family = defualt_text.font.family.clone();
                        set_dirty(dirty_list, id, StyleType::FontFamily as usize, style_mark);
                    }
                }
            }
        }       

        if old_style & IMAGE_DIRTY != 0 {
            if old_style & StyleType::Image as usize == 0 {
                images.delete(id);
                set_dirty(dirty_list, id, StyleType::Image as usize, style_mark);
            }
            if old_style & StyleType::ImageClip as usize == 0 {
                image_clips.delete(id);
                set_dirty(dirty_list, id, StyleType::ImageClip as usize, style_mark);
            }
            if old_style & StyleType::ObjectFit as usize == 0 {
                obj_fits.delete(id);
                set_dirty(dirty_list, id, StyleType::ObjectFit as usize, style_mark);
            }
        }

        if old_style & BORDER_IMAGE_DIRTY != 0 {
            if old_style & StyleType::BorderImage as usize == 0 {
                border_images.delete(id);
                set_dirty(dirty_list, id, StyleType::BorderImage as usize, style_mark);
            }
            if old_style & StyleType::BorderImageClip as usize == 0 {
                border_image_clips.delete(id);
                set_dirty(dirty_list, id, StyleType::BorderImageClip as usize, style_mark);
            }
            if old_style & StyleType::BorderImageSlice as usize == 0 {
                border_image_slices.delete(id);
                set_dirty(dirty_list, id, StyleType::BorderImageSlice as usize, style_mark);
            }
            if old_style & StyleType::BorderImageRepeat as usize == 0 {
                border_image_repeats.delete(id);
                set_dirty(dirty_list, id, StyleType::BorderImageRepeat as usize, style_mark);
            }
        }

        if old_style & StyleType::BorderColor as usize == 0 {
            border_colors.delete(id);
            set_dirty(dirty_list, id, StyleType::BorderColor as usize, style_mark);
        }

        if old_style & StyleType::BackgroundColor as usize == 0 {
            background_colors.delete(id);
            set_dirty(dirty_list, id, StyleType::BackgroundColor as usize, style_mark);
        }

        if old_style & StyleType::BoxShadow as usize == 0 {
            box_shadows.delete(id);
            set_dirty(dirty_list, id, StyleType::BoxShadow as usize, style_mark);
        }

        if old_style & NODE_DIRTY != 0 {
            if old_style & StyleType::Opacity as usize == 0 {
                opacitys.delete(id);
                set_dirty(dirty_list, id, StyleType::Opacity as usize, style_mark);
            }

            if old_style & StyleType::BorderRadius as usize == 0 {
                border_radiuss.delete(id);
                set_dirty(dirty_list, id, StyleType::BorderRadius as usize, style_mark);
            }

            if old_style & StyleType::Filter as usize == 0 {
                filters.delete(id);
                set_dirty(dirty_list, id, StyleType::Filter as usize, style_mark);
            }
        }

    }

    if old_style1 != 0 {
        if old_style & NODE_DIRTY1 != 0 {
            if old_style & StyleType1::Enable as usize == 0 || old_style & StyleType1::Display as usize == 0 || old_style & StyleType1::Visibility as usize == 0 {
                if let Some(show) = shows.get_mut(id) {
                    if old_style & StyleType1::Enable as usize == 0 {
                        show.set_enable(EnableType::Auto);
                    }
                    if old_style & StyleType1::Display as usize == 0 {
                        show.set_display(Display::Flex);
                    }
                    if old_style & StyleType1::Visibility as usize == 0 {
                        show.set_visibility(true);
                    }
                }
                shows.get_notify_ref().modify_event(id, "", 0);
                
                if old_style & StyleType1::ZIndex as usize == 0 {
                    zindexs.insert(id, ZIndex(0));
                }

                if old_style & StyleType1::Transform as usize == 0{
                    transforms.delete(id);
                }
            }
        }

        if old_style1 & LAYOUT_DIRTY != 0 {
            let yoga = unsafe {yogas.get_unchecked(id)};
            reset_layout_attr(yoga, old_style1);
        }
    } 
}

fn reset_layout_attr<L: FlexNode>(yoga: &L, old_style1: usize){
    if old_style1 & StyleType1::Width as usize != 0 {
        yoga.set_width(std::f32::NAN);
    }
    if old_style1 & StyleType1::Height as usize != 0 {
        yoga.set_height(std::f32::NAN);
    }
    if old_style1 & StyleType1::Margin as usize != 0 {
        yoga.set_margin(YGEdge::YGEdgeAll, std::f32::NAN);
    }
    if old_style1 & StyleType1::Padding as usize != 0 {
        yoga.set_padding(YGEdge::YGEdgeAll, std::f32::NAN);
    }
    if old_style1 & StyleType1::Border as usize != 0 {
        yoga.set_border(YGEdge::YGEdgeAll, std::f32::NAN);
    }
    if old_style1 & StyleType1::Position as usize != 0 {
        yoga.set_position(YGEdge::YGEdgeAll, std::f32::NAN);
    }
    if old_style1 & StyleType1::MinWidth as usize != 0 {
        yoga.set_min_width(std::f32::NAN);
    }
    if old_style1 & StyleType1::MinHeight as usize != 0 {
        yoga.set_min_height(std::f32::NAN);
    }
    if old_style1 & StyleType1::MaxWidth as usize != 0 {
        yoga.set_max_width(std::f32::NAN);
    }
    if old_style1 & StyleType1::MaxHeight as usize != 0 {
        yoga.set_max_height(std::f32::NAN);
    }
    if old_style1 & StyleType1::FlexBasis as usize != 0 {
        yoga.set_flex_basis(std::f32::NAN);
    }
    if old_style1 & StyleType1::FlexShrink as usize != 0 {
        yoga.set_flex_shrink(std::f32::NAN);
    }
    if old_style1 & StyleType1::FlexGrow as usize != 0 {
        yoga.set_flex_grow(std::f32::NAN);
    }
    if old_style1 & StyleType1::PositionType as usize != 0 {
        yoga.set_position_type(YGPositionType::YGPositionTypeAbsolute);
    }
    if old_style1 & StyleType1::FlexWrap as usize != 0 {
        yoga.set_flex_wrap(YGWrap::YGWrapWrap);
    }
    if old_style1 & StyleType1::FlexDirection as usize != 0 {
        yoga.set_flex_direction(YGFlexDirection::YGFlexDirectionRow);
    }
    if old_style1 & StyleType1::AlignContent as usize != 0 {
        yoga.set_align_content(YGAlign::YGAlignFlexStart);
    }
    if old_style1 & StyleType1::AlignItems as usize != 0 {
        yoga.set_align_items(YGAlign::YGAlignFlexStart);
    }
    if old_style1 & StyleType1::AlignSelf as usize != 0 {
        yoga.set_align_self(YGAlign::YGAlignFlexStart);
    }
    if old_style1 & StyleType1::JustifyContent as usize != 0 {
        yoga.set_justify_content(YGJustify::YGJustifyFlexStart);
    }
}

fn set_attr<L: FlexNode, C: HalContext>(
    id: usize,
    class_name: usize,
    text_style: &mut TextStyle,
    show: &mut Show,
    read: ReadData,
    write: &mut WriteData<L, C>,
){
    if class_name == 0{
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
        _shows,
        style_marks,
        yogas,
        engine,
        image_wait_sheet,
        dirty_list,
        ) = write;
    let style_mark = unsafe { style_marks.get_unchecked_mut(id) };
    // 设置布局属性， 没有记录每个个属性是否在本地样式表中存在， TODO
    let yoga = unsafe {yogas.get_unchecked(id)};

    let class = match class_sheet.class_map.get(&class_name) {
        Some(class) => class,
        None => return,
    };
    
    let mut text_style = text_style;
    if let Some(r) = text_styles.get_mut(id) {
        text_style = r;
    }

    set_attr1(id, dirty_list, &class.attrs1, style_mark, text_style, show, yoga, obj_fits);
    set_attr2(id, dirty_list, &class.attrs2, style_mark, text_style, yoga, zindexs, opacitys, border_image_repeats, images, border_images, image_wait_sheet, engine);
    set_attr3(id, dirty_list, &class.attrs3, style_mark, text_style, border_image_slices, border_image_clips, image_clips, box_shadows, background_colors, border_colors, border_radiuss, filters, transforms);
    style_mark.class_style |= class.class_style_mark;
    style_mark.class_style1 |= class.class_style_mark1;
}

#[inline]
fn set_mark(class_sheet: &ClassSheet, name: usize, mark: &mut StyleMark){
    match class_sheet.class_map.get(&name) {
        Some(class) => {
            mark.class_style |= class.class_style_mark;
            mark.class_style1 |= class.class_style_mark1;
        },
        None =>(),
    };
}

pub fn set_attr1<L: FlexNode>(
    id: usize,
    dirty_list: &mut DirtyList,
    layout_attrs: &Vec<Attribute1>,
    style_mark: &mut StyleMark,
    text_style: &mut TextStyle,
    show: &mut Show,
    yoga: &L,
    obj_fits: &mut MultiCaseImpl<Node, ObjectFit>,
){
    for layout_attr in layout_attrs.iter() {
        match layout_attr {
            Attribute1::AlignContent(r) => if StyleType1::AlignContent as usize & style_mark.local_style1 == 0 {yoga.set_align_content(*r)},
            Attribute1::AlignItems(r) => if StyleType1::AlignItems as usize & style_mark.local_style1 == 0 {yoga.set_align_items(*r)},
            Attribute1::AlignSelf(r) => if StyleType1::AlignSelf as usize & style_mark.local_style1 == 0 {yoga.set_align_self(*r)},
            Attribute1::JustifyContent(r) => if StyleType1::JustifyContent as usize & style_mark.local_style1 == 0 {yoga.set_justify_content(*r)},
            Attribute1::FlexDirection(r) => if StyleType1::FlexDirection as usize & style_mark.local_style1 == 0 {yoga.set_flex_direction(*r)},
            Attribute1::FlexWrap(r) => if StyleType1::FlexWrap as usize & style_mark.local_style1 == 0 {yoga.set_flex_wrap(*r)},
            Attribute1::PositionType(r) => if StyleType1::PositionType as usize & style_mark.local_style1 == 0 {yoga.set_position_type(*r)},

            Attribute1::ObjectFit(r) => if style_mark.local_style == 0 & StyleType::ObjectFit as usize {
                obj_fits.insert_no_notify(id, r.clone());
                set_dirty(dirty_list, id, StyleType::ObjectFit as usize, style_mark);
            },
            Attribute1::TextAlign(r) => if style_mark.local_style & StyleType::TextAlign as usize == 0 {
                text_style.text.text_align = *r;
                set_dirty(dirty_list, id, StyleType::TextAlign as usize, style_mark);
            },
            Attribute1::VerticalAlign(r) => if style_mark.local_style & StyleType::VerticalAlign as usize == 0 {
                text_style.text.vertical_align = *r;
                set_dirty(dirty_list, id, StyleType::VerticalAlign as usize, style_mark);
            },
            Attribute1::WhiteSpace(r) => if style_mark.local_style & StyleType::WhiteSpace as usize == 0 {
                text_style.text.white_space = *r;
                set_dirty(dirty_list, id, StyleType::WhiteSpace as usize, style_mark);
            },
            Attribute1::FontStyle(r) => if style_mark.local_style & StyleType::FontStyle as usize == 0 {
                text_style.font.style = *r;
                set_dirty(dirty_list, id, StyleType::WhiteSpace as usize, style_mark);
            },
            Attribute1::Enable(r) => if style_mark.local_style1 & StyleType1::Enable as usize == 0 {
                show.set_enable(*r);
            },
            Attribute1::Display(r) => if style_mark.local_style1 & StyleType1::Display as usize == 0{
                show.set_display(*r);
            },
            Attribute1::Visibility(r) => if style_mark.local_style1 & StyleType1::Visibility as usize == 0{
                show.set_visibility(*r);
            },
        }
    }
}

pub fn set_attr2<L: FlexNode, C: HalContext>(
    id: usize,
    dirty_list: &mut DirtyList,
    layout_attrs: &Vec<Attribute2>,
    style_mark: &mut StyleMark,
    text_style: &mut TextStyle,
    yoga: &L,
    zindexs: &mut MultiCaseImpl<Node, ZIndex>,
    opacitys: &mut MultiCaseImpl<Node, Opacity>,
    border_image_repeats: &mut MultiCaseImpl<Node, BorderImageRepeat>, 
    images: &mut MultiCaseImpl<Node, Image>,
    border_images: &mut MultiCaseImpl<Node, BorderImage>,
    image_wait_sheet: &mut SingleCaseImpl<ImageWaitSheet>,
    engine: &mut SingleCaseImpl<Engine<C>>,
){
    for layout_attr in layout_attrs.iter() {
        match layout_attr {
            Attribute2::LetterSpacing(r) => if style_mark.local_style & StyleType::LetterSpacing as usize == 0 {
                text_style.text.letter_spacing = *r;
                set_dirty(dirty_list, id, StyleType::LetterSpacing as usize, style_mark);
            },
            Attribute2::LineHeight(r) => if style_mark.local_style & StyleType::LineHeight as usize == 0 {
                text_style.text.line_height = *r;
                set_dirty(dirty_list, id, StyleType::LineHeight as usize, style_mark);
            },
            Attribute2::TextIndent(r) => if style_mark.local_style & StyleType::Indent as usize == 0 {
                text_style.text.indent = *r;
                set_dirty(dirty_list, id, StyleType::Indent as usize, style_mark);
            },
            Attribute2::WordSpacing(r) => if style_mark.local_style & StyleType::WordSpacing as usize == 0 {
                text_style.text.word_spacing = *r;
                set_dirty(dirty_list, id, StyleType::WordSpacing as usize, style_mark);
            },
            Attribute2::FontWeight(r) => if style_mark.local_style & StyleType::FontWeight as usize == 0 {
                text_style.font.weight = *r as usize;
                set_dirty(dirty_list, id, StyleType::FontWeight as usize, style_mark);
            },
            Attribute2::FontSize(r) => if style_mark.local_style & StyleType::FontSize as usize == 0 {
                text_style.font.size = *r;
                set_dirty(dirty_list, id, StyleType::FontSize as usize, style_mark);
            },
            Attribute2::FontFamily(r) => if style_mark.local_style & StyleType::FontFamily as usize == 0 {
                text_style.font.family = r.clone();
                set_dirty(dirty_list, id, StyleType::FontFamily as usize, style_mark);
            } ,
            Attribute2::ZIndex(r) => if style_mark.local_style1 & StyleType1::ZIndex as usize == 0 {
                zindexs.insert(id, ZIndex(*r));
            },
            Attribute2::Opacity(r) => if style_mark.local_style & StyleType::Opacity as usize == 0 {
                opacitys.insert(id, r.clone());
                // set_dirty(dirty_list, event.id, StyleType::Opacity as usize, style_mark); 不需要设脏，opacity还需要通过级联计算得到最终值， 监听到该值的变化才会设脏
            },
            Attribute2::BorderImageRepeat(r) => if style_mark.local_style & StyleType::BorderImageRepeat as usize == 0 {
                border_image_repeats.insert_no_notify(id, r.clone());
                set_dirty(dirty_list, id, StyleType::BorderImageRepeat as usize, style_mark);
            },

            Attribute2::ImageUrl(r) => if style_mark.local_style & StyleType::Image as usize == 0 {
                match engine.texture_res_map.get(r) {
                    Some(res) => {
                        images.insert_no_notify(id, Image{src: res, url: r.clone()});
                        set_dirty(dirty_list, id, StyleType::Image as usize, style_mark);
                    },
                    None => {
                        // 异步加载图片
                        image_wait_sheet.add(r, ImageWait{id: id, ty: ImageType::ImageClass});
                    },
                }
            },
            Attribute2::BorderImageUrl(r) => if style_mark.local_style & StyleType::BorderImage as usize == 0 {
                match engine.texture_res_map.get(r) {
                    Some(res) => {
                        border_images.insert_no_notify(id, BorderImage{src: res, url: r.clone()});
                        set_dirty(dirty_list, id, StyleType::BorderImage as usize, style_mark);
                    },
                    None => {
                        // 异步加载图片
                        image_wait_sheet.add(r, ImageWait{id: id, ty: ImageType::BorderImageClass})
                    },
                }
            },

            Attribute2::Width(r) => {
				if StyleType1::Width as usize & style_mark.local_style1 == 0 {
					match r {
						ValueUnit::Auto => yoga.set_width_auto(),
						ValueUnit::Undefined => yoga.set_width_auto(),
						ValueUnit::Pixel(r) => yoga.set_width(*r),
						ValueUnit::Percent(r) => yoga.set_width_percent(*r),
					}
				}
			},
            Attribute2::Height(r) => if StyleType1::Height as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Auto => yoga.set_height_auto(),
                    ValueUnit::Undefined => yoga.set_height_auto(),
                    ValueUnit::Pixel(r) => yoga.set_height(*r),
                    ValueUnit::Percent(r) => yoga.set_height_percent(*r),
                }
            },
            Attribute2::MarginLeft(r) => if StyleType1::Margin as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeLeft),
                    ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeLeft),
                    ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeLeft, *r),
                    ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeLeft, *r),
                }
            },
            Attribute2::MarginTop(r) => if StyleType1::Margin as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeTop),
                    ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeTop),
                    ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeTop, *r),
                    ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeTop, *r),
                }
            },
            Attribute2::MarginBottom(r) => if StyleType1::Margin as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeBottom),
                    ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeBottom),
                    ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeBottom, *r),
                    ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeBottom, *r),
                }
            },
            Attribute2::MarginRight(r) => if StyleType1::Margin as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeRight),
                    ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeRight),
                    ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeRight, *r),
                    ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeRight, *r),
                }
            },
            Attribute2::Margin(r) => if StyleType1::Margin as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Auto | ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeAll),
                    ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeAll, *r),
                    ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeAll, *r),
                }
            },
            Attribute2::PaddingLeft(r) => if StyleType1::Padding as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeLeft, *r),
                    ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeLeft, *r),
                    _ => (),
                }
            },
            Attribute2::PaddingTop(r) => if StyleType1::Padding as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeTop, *r),
                    ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeTop, *r),
                    _ => (),
                }
            },
            Attribute2::PaddingBottom(r) => if StyleType1::Padding as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeBottom, *r),
                    ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeBottom, *r),
                    _ => (),
                }
            },
            Attribute2::PaddingRight(r) => if StyleType1::Padding as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeRight, *r),
                    ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeRight, *r),
                    _ => (), 
                }
            },
            Attribute2::Padding(r) => if StyleType1::Padding as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeAll, *r),
                    ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeAll, *r),
                    _ => (),
                }
            },
            Attribute2::BorderLeft(r) => if StyleType1::Border as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeLeft, *r),
                    // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeLeft, *r),
                    _ => (),
                }
            },
            Attribute2::BorderTop(r) => if StyleType1::Border as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeTop, *r),
                    // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeTop, *r),
                    _ => (),
                }
            },
            Attribute2::BorderBottom(r) => if StyleType1::Border as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeBottom, *r),
                    // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeTop, *r),
                    _ => (),
                }
            },
            Attribute2::BorderRight(r) => if StyleType1::Border as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeRight, *r),
                    // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeRight, *r),
                    _ => (),
                }
            },
            Attribute2::Border(r) => if StyleType1::Border as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeAll, *r),
                    _ => (),
                }
            },
            Attribute2::PositionLeft(r) => if StyleType1::Position as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_position(YGEdge::YGEdgeLeft, *r),
                    ValueUnit::Percent(r) => yoga.set_position_percent(YGEdge::YGEdgeLeft, *r),
                    _ => (),
                }
            },
            Attribute2::PositionTop(r) => if StyleType1::Position as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_position(YGEdge::YGEdgeTop, *r),
                    ValueUnit::Percent(r) => yoga.set_position_percent(YGEdge::YGEdgeTop, *r),
                    _ => (),
                }
            },
            Attribute2::PositionRight(r) => if StyleType1::Position as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_position(YGEdge::YGEdgeRight, *r),
                    ValueUnit::Percent(r) => yoga.set_position_percent(YGEdge::YGEdgeRight, *r),
                    _ => (),
                }
            },
            Attribute2::PositionBottom(r) => if StyleType1::Position as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_position(YGEdge::YGEdgeBottom, *r),
                    ValueUnit::Percent(r) => yoga.set_position_percent(YGEdge::YGEdgeBottom, *r),
                    _ => (), 
                }
            },
            Attribute2::MinWidth(r) => if StyleType1::MinWidth as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_min_width(*r),
                    ValueUnit::Percent(r) => yoga.set_min_width_percent(*r),
                    _ => (),
                    
                }
            },
            Attribute2::MinHeight(r) => if StyleType1::MinHeight as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_min_height(*r),
                    ValueUnit::Percent(r) => yoga.set_min_height_percent(*r),
                    _ => (),
                }
            },
            Attribute2::MaxHeight(r) => if StyleType1::MaxHeight as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_max_height(*r),
                    ValueUnit::Percent(r) => yoga.set_max_height_percent(*r),
                    _ => (),
                }
            },
            Attribute2::MaxWidth(r) => if StyleType1::MaxWidth as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Pixel(r) => yoga.set_max_width(*r),
                    ValueUnit::Percent(r) => yoga.set_max_width_percent(*r),
                    _ => (),
                }
            },
            Attribute2::FlexBasis(r) => if StyleType1::FlexBasis as usize & style_mark.local_style1 == 0 {
                match r {
                    ValueUnit::Auto => yoga.set_flex_basis_auto(),
                    ValueUnit::Undefined => yoga.set_flex_basis_auto(),
                    ValueUnit::Pixel(r) => yoga.set_flex_basis(*r),
                    ValueUnit::Percent(r) => yoga.set_flex_basis_percent(*r),
                }
            },
            Attribute2::FlexShrink(r) => if StyleType1::FlexShrink as usize & style_mark.local_style1 == 0 {
                yoga.set_flex_shrink(*r)
            },
            Attribute2::FlexGrow(r) => if StyleType1::FlexGrow as usize & style_mark.local_style1 == 0 {
                yoga.set_flex_grow(*r)
            },
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
    image_clips: &mut MultiCaseImpl<Node, ImageClip>,
    box_shadows: &mut MultiCaseImpl<Node, BoxShadow>, 
    background_colors: &mut MultiCaseImpl<Node, BackgroundColor>,
    border_colors: &mut MultiCaseImpl<Node, BorderColor>,
    border_radiuss: &mut MultiCaseImpl<Node, BorderRadius>,
    filters: &mut MultiCaseImpl<Node, Filter>,
    transforms: &mut MultiCaseImpl<Node, Transform>,
){
    for attr in attrs.iter() {
        match attr {
            Attribute3::BGColor(r) => if style_mark.local_style & StyleType::BackgroundColor as usize == 0 {
                background_colors.insert_no_notify(id, r.clone());
                set_dirty(dirty_list, id, StyleType::BackgroundColor as usize, style_mark);
            },
            Attribute3::BorderColor(r) => if style_mark.local_style & StyleType::BorderColor as usize == 0 {
                border_colors.insert_no_notify(id, r.clone());
                set_dirty(dirty_list, id, StyleType::BorderColor as usize, style_mark);
            },
            Attribute3::BoxShadow(r) => if style_mark.local_style & StyleType::BoxShadow as usize == 0 {
                box_shadows.insert_no_notify(id, r.clone());
                set_dirty(dirty_list, id, StyleType::BoxShadow as usize, style_mark);
            },

            Attribute3::ImageClip(r) => if style_mark.local_style & StyleType::ImageClip as usize == 0 {
                image_clips.insert_no_notify(id, r.clone());
                set_dirty(dirty_list, id, StyleType::ImageClip as usize, style_mark);
            },

            Attribute3::BorderImageClip(r) => if style_mark.local_style & StyleType::BorderImageClip as usize == 0 {
                border_image_clips.insert_no_notify(id, r.clone());
                set_dirty(dirty_list, id, StyleType::BorderImageClip as usize, style_mark);
            },
            Attribute3::BorderImageSlice(r) => if style_mark.local_style & StyleType::BorderImageSlice as usize == 0 {
                border_image_slices.insert_no_notify(id, r.clone());
                set_dirty(dirty_list, id, StyleType::BorderImageSlice as usize, style_mark);
            },

            Attribute3::Color(r) => if style_mark.local_style & StyleType::Color as usize == 0 {
                text_style.text.color = r.clone();
                set_dirty(dirty_list, id, StyleType::Color as usize, style_mark);
            },
            Attribute3::TextShadow(r) => if style_mark.local_style & StyleType::TextShadow as usize == 0 {
                text_style.shadow = r.clone();
                set_dirty(dirty_list, id, StyleType::TextShadow as usize, style_mark);
            },
            Attribute3::TextStroke(r) => if style_mark.local_style & StyleType::Stroke as usize == 0 {
                text_style.text.stroke = r.clone();
                set_dirty(dirty_list, id, StyleType::Stroke as usize, style_mark);
            },

            Attribute3::BorderRadius(r) => if style_mark.local_style & StyleType::BorderRadius as usize == 0 {
                border_radiuss.insert_no_notify(id, r.clone());
                set_dirty(dirty_list, id, StyleType::BorderRadius as usize, style_mark);
            },
            Attribute3::TransformFunc(r) => if style_mark.local_style1 & StyleType1::Transform as usize == 0 {
                match transforms.get_mut(id) {
                    Some(t) => t.funcs = r.clone(),
                    None => {transforms.insert(id, Transform{funcs: r.clone(), origin: TransformOrigin::Center});},
                };
            },
            Attribute3::TransformOrigin(r) => if style_mark.local_style & StyleType::Color as usize == 0 {
                match transforms.get_mut(id) {
                    Some(t) => t.origin = r.clone(),
                    None => {transforms.insert(id, Transform{funcs: Vec::default(), origin: r.clone()});},
                };
            },
            Attribute3::Filter(r) => if style_mark.local_style & StyleType::Filter as usize == 0{
                filters.insert(id, r.clone());
                set_dirty(dirty_list, id, StyleType::Filter as usize, style_mark);
            },
        }
    }
    
}

impl_system!{
    StyleMarkSys<L, C> where [L: FlexNode + 'static, C: HalContext + 'static],
    true,
    {
        EntityListener<Node, CreateEvent>
        MultiCaseListener<Node, TextContent, CreateEvent>
        MultiCaseListener<Node, TextContent, ModifyEvent>
        MultiCaseListener<Node, TextStyle, ModifyEvent>

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
        MultiCaseListener<Node, Layout, ModifyEvent>   
        MultiCaseListener<Node, BorderRadius, ModifyEvent>
        // MultiCaseListener<Node, Transform, ModifyEvent>  
        MultiCaseListener<Node, Filter, ModifyEvent>
        MultiCaseListener<Node, ByOverflow, ModifyEvent>
        MultiCaseListener<Node, Visibility, ModifyEvent>

        MultiCaseListener<Node, ClassName, ModifyEvent> 
        SingleCaseListener<ImageWaitSheet, ModifyEvent>
        SingleCaseListener<RenderObjs, CreateEvent>
        SingleCaseListener<DefaultTable, ModifyEvent>
    }
}
