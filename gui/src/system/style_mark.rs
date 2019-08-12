/**
 * 样式标记
 * StyleMarkSys系统会在Node实体创建时， 自动为Node创建一个StyleMark组件， 该组件用于标记了各种样式脏、是否为本地样式
 * StyleMarkSys系统会监听本地样式的修改，以标记样式的脏， 并覆盖class设置的样式属性（覆盖的方式为：修改该属性的本地样式标记为1）
 * StyleMarkSys系统会监听ClassName的修改， 遍历class中的属性， 如果该属性没有设置本地样式，将覆盖该属性对应的组件，并标记样式脏
 * class中的图片， 是一个url， 在设置class时， 该图片资源可能还未加载， StyleMarkSys会将不存在的图片url放入ImageWaitSheet中， 由外部处理ImageWaitSheet中的等待列表，并图片加载完成， 应该将图片放入完成列表中， 并通知ImageWaitSheet修改， 由StyleMarkSys来处理ImageWaitSheet中的完成列表
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
use render::res::*;

const TEXT_DIRTY: usize =   StyleType::LetterSpacing as usize | 
                            StyleType::WordSpacing as usize | 
                            StyleType::LineHeight as usize | 
                            StyleType::Indent as usize |
                            StyleType::WhiteSpace as usize | 
                            StyleType::TextAlign as usize | 
                            StyleType::VerticalAlign as usize |
                            StyleType::TextShadow as usize |
                            StyleType::Color as usize | 
                            StyleType::Stroke as usize;
const FONT_DIRTY: usize =   StyleType::FontStyle as usize | 
                            StyleType::FontFamily as usize | 
                            StyleType::FontSize as usize | 
                            StyleType::FontWeight as usize; 

const TEXT_STYLE_DIRTY: usize = TEXT_DIRTY | FONT_DIRTY | StyleType::TextShadow as usize;         

pub struct StyleMarkSys<L, C>{
    text_style_mark: usize,
    mark: PhantomData<(L, C)>,
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> StyleMarkSys<L, C> {
    pub fn new() -> Self {
        Self{
            text_style_mark:StyleType::LetterSpacing as usize | 
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
                            StyleType::FontWeight as usize ,
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
fn set_dirty(dirty_list: &mut DirtyList, id: usize, ty: usize, style_mark: &mut StyleMark ) {
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
        class_names.insert(event.id, ClassName::default());
    }
}

// 监听TextStyle属性的改变
impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, TextStyle, ModifyEvent> for StyleMarkSys<L, C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        println!("TextStyle change");
        let r = match event.field {
            "letter_spacing" => StyleType::LetterSpacing,
            "word_spacing" => StyleType::WordSpacing,
            "line_height" => StyleType::LineHeight,
            "text _indent" => StyleType::Indent,
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
        println!("style_mark-------------------{:?}", r);
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, r as usize, style_marks);
    }
}

// 监听TextContente属性的改变
impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, TextContent, CreateEvent> for StyleMarkSys<L, C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);

    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = unsafe { style_marks.get_unchecked_mut(event.id) };
        println!("Text change");
        set_dirty(dirty_list, event.id, StyleType::Text as usize, style_mark);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, TextContent, ModifyEvent> for StyleMarkSys<L, C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = unsafe { style_marks.get_unchecked_mut(event.id) };
        println!("Text change");
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

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderColor, CreateEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::BorderColor as usize, style_marks);
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

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, Transform, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::Transform as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderRadius, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::Transform as usize, style_marks);
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, Filter, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        let (style_marks, dirty_list) = write;
        set_local_dirty(dirty_list, event.id, StyleType::Transform as usize, style_marks);
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

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, ClassName, ModifyEvent> for StyleMarkSys<L, C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ClassName>,
        &'a SingleCaseImpl<ClassSheet>
    );
    type WriteData = (
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
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut MultiCaseImpl<Node, L>,
        &'a mut SingleCaseImpl<Engine<C>>,
        &'a mut SingleCaseImpl<ImageWaitSheet>,
        &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (class_names, class_sheet) = read;
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
            border_radiuses,
            filters,
            style_marks,
            yogas,
            engine,
            image_wait_sheet,
            dirty_list,
            ) = write;
        let class_name = unsafe { class_names.get_unchecked(event.id) }.0;
        let class = match class_sheet.class.get(class_name) {
            Some(class) => class,
            None => return,
        };

        let style_mark = unsafe { style_marks.get_unchecked_mut(event.id) };

        if class.text > 0 {
            let c = unsafe { class_sheet.text.get_unchecked(class.text) }; 
            if class.class_style_mark & self.text_style_mark != 0 {
                if style_mark.local_style & TEXT_STYLE_DIRTY == 0 {
                    // 如果本地样式不存在任何文字相关的属性， 直接设置文字组件与class相同
                    text_styles.insert_no_notify(event.id, c.clone());
                    set_dirty(dirty_list, event.id, class.class_style_mark & TEXT_STYLE_DIRTY, style_mark);
                } else {
                    let text_style = unsafe { text_styles.get_unchecked_mut(event.id) };
                    // text本地样式不存在
                    if style_mark.local_style & TEXT_DIRTY == 0 {
                        text_style.text = c.text.clone();
                        set_dirty(dirty_list, event.id, class.class_style_mark & TEXT_DIRTY, style_mark);
                    } else {
                        if class.class_style_mark & StyleType::LetterSpacing as usize != 0 && style_mark.local_style & StyleType::LetterSpacing as usize == 0 {
                            text_style.text.letter_spacing = c.text.letter_spacing;
                            set_dirty(dirty_list, event.id, StyleType::LetterSpacing as usize, style_mark);
                        }
                        if class.class_style_mark & StyleType::WordSpacing as usize != 0 && style_mark.local_style & StyleType::WordSpacing as usize == 0 {
                            text_style.text.word_spacing = c.text.word_spacing;
                            set_dirty(dirty_list, event.id, StyleType::WordSpacing as usize, style_mark);
                        }
                        if class.class_style_mark & StyleType::LineHeight as usize != 0 && style_mark.local_style & StyleType::LineHeight as usize == 0 {
                            text_style.text.line_height = c.text.line_height;
                            set_dirty(dirty_list, event.id, StyleType::LineHeight as usize, style_mark);
                        }
                        if class.class_style_mark & StyleType::Indent as usize != 0 && style_mark.local_style & StyleType::Indent as usize == 0 {
                            text_style.text.indent = c.text.indent;
                            set_dirty(dirty_list, event.id, StyleType::Indent as usize, style_mark);
                        }
                        if class.class_style_mark & StyleType::WhiteSpace as usize != 0 && style_mark.local_style & StyleType::WhiteSpace as usize == 0 {
                            text_style.text.white_space = c.text.white_space;
                            set_dirty(dirty_list, event.id, StyleType::WhiteSpace as usize, style_mark);
                        }

                        if class.class_style_mark & StyleType::Color as usize != 0 && style_mark.local_style & StyleType::Color as usize == 0 {
                            text_style.text.color = c.text.color.clone();
                            set_dirty(dirty_list, event.id, StyleType::Color as usize, style_mark);
                        }

                        if class.class_style_mark & StyleType::Stroke as usize != 0 && style_mark.local_style & StyleType::Stroke as usize == 0 {
                            text_style.text.stroke = c.text.stroke.clone();
                            set_dirty(dirty_list, event.id, StyleType::Stroke as usize, style_mark);
                        }

                        if class.class_style_mark & StyleType::TextAlign as usize != 0 && style_mark.local_style & StyleType::TextAlign as usize == 0 {
                            text_style.text.text_align = c.text.text_align;
                            set_dirty(dirty_list, event.id, StyleType::TextAlign as usize, style_mark);
                        }

                        if class.class_style_mark & StyleType::VerticalAlign as usize != 0 && style_mark.local_style & StyleType::VerticalAlign as usize == 0 {
                            text_style.text.vertical_align = c.text.vertical_align;
                            set_dirty(dirty_list, event.id, StyleType::VerticalAlign as usize, style_mark);
                        }
                    }

                    if class.class_style_mark & StyleType::TextShadow as usize != 0 && style_mark.local_style & StyleType::TextShadow as usize == 0 {
                        text_style.shadow = c.shadow.clone();
                        set_dirty(dirty_list, event.id, StyleType::TextShadow as usize, style_mark);
                    }

                    if style_mark.local_style & FONT_DIRTY == 0 {
                        text_style.font = c.font.clone();
                        set_dirty(dirty_list, event.id, class.class_style_mark & FONT_DIRTY, style_mark);
                    }else{
                        if class.class_style_mark & StyleType::FontStyle as usize != 0 && style_mark.local_style & StyleType::FontStyle as usize == 0 {
                            text_style.font.style = c.font.style;
                            set_dirty(dirty_list, event.id, StyleType::FontStyle as usize, style_mark);
                        }
                        if class.class_style_mark & StyleType::FontWeight as usize != 0 && style_mark.local_style & StyleType::FontWeight as usize == 0 {
                            text_style.font.weight = c.font.weight;
                            set_dirty(dirty_list, event.id, StyleType::FontWeight as usize, style_mark);
                        }
                        if class.class_style_mark & StyleType::FontSize as usize != 0 && style_mark.local_style & StyleType::FontSize as usize == 0 {
                            text_style.font.size = c.font.size;
                            set_dirty(dirty_list, event.id, StyleType::FontSize as usize, style_mark);
                        }
                        if class.class_style_mark & StyleType::FontFamily as usize != 0 && style_mark.local_style & StyleType::FontFamily as usize == 0 {
                            text_style.font.family = c.font.family.clone();
                            set_dirty(dirty_list, event.id, StyleType::FontFamily as usize, style_mark);
                        }
                    }
                }
            } 
        }

        if class.image > 0 {
            let c = unsafe { class_sheet.image.get_unchecked(class.image) };
            if class.class_style_mark & StyleType::Image as usize != 0 && style_mark.local_style & StyleType::Image as usize == 0 {
                match engine.res_mgr.get::<TextureRes>(&c.image) {
                    Some(r) => {
                        images.insert_no_notify(event.id, Image{src: r, url: c.image.clone()});
                        set_dirty(dirty_list, event.id, StyleType::Image as usize, style_mark);
                    },
                    None => {
                        // 异步加载图片
                        image_wait_sheet.add(&c.image, ImageWait{id: event.id, ty: ImageType::ImageClass})
                    },
                }
            }
            if class.class_style_mark & StyleType::ImageClip as usize != 0 && style_mark.local_style & StyleType::ImageClip as usize == 0 {
                image_clips.insert_no_notify(event.id, c.image_clip.clone());
                set_dirty(dirty_list, event.id, StyleType::ImageClip as usize, style_mark);
            }
            if class.class_style_mark & StyleType::ObjectFit as usize != 0 && style_mark.local_style & StyleType::ObjectFit as usize == 0 {
                obj_fits.insert_no_notify(event.id, ObjectFit(c.obj_fit.clone()));
                set_dirty(dirty_list, event.id, StyleType::ObjectFit as usize, style_mark);
            }
        }

        if class.border_image > 0 {
            let c = unsafe { class_sheet.border_image.get_unchecked(class.border_image) };
            if class.class_style_mark & StyleType::BorderImage as usize != 0 && style_mark.local_style & StyleType::BorderImage as usize == 0 {
                match engine.res_mgr.get::<TextureRes>(&c.border_image) {
                    Some(r) => {
                        border_images.insert_no_notify(event.id, BorderImage{src: r, url: c.border_image.clone()});
                        set_dirty(dirty_list, event.id, StyleType::BorderImage as usize, style_mark);
                    },
                    None => {
                        // 异步加载图片
                        image_wait_sheet.add(&c.border_image, ImageWait{id: event.id, ty: ImageType::BorderImageClass})
                    },
                }
            }
            if class.class_style_mark & StyleType::BorderImageClip as usize != 0 && style_mark.local_style & StyleType::BorderImageClip as usize == 0 {
                border_image_clips.insert_no_notify(event.id, c.border_image_clip.clone());
                set_dirty(dirty_list, event.id, StyleType::BorderImageClip as usize, style_mark);
            }
            if class.class_style_mark & StyleType::BorderImageSlice as usize != 0 && style_mark.local_style & StyleType::BorderImageSlice as usize == 0 {
                border_image_slices.insert_no_notify(event.id, c.border_image_slice.clone());
                set_dirty(dirty_list, event.id, StyleType::BorderImageSlice as usize, style_mark);
            }
            if class.class_style_mark & StyleType::BorderImageClip as usize != 0 && style_mark.local_style & StyleType::BorderImageClip as usize == 0 {
                border_image_repeats.insert_no_notify(event.id, c.border_image_repeat.clone());
                set_dirty(dirty_list, event.id, StyleType::BorderImageClip as usize, style_mark);
            }
        }

        if class.border_color > 0 {
            if class.class_style_mark & StyleType::BorderColor as usize != 0 && style_mark.local_style & StyleType::BorderColor as usize == 0 {
                let c = unsafe { class_sheet.border_color.get_unchecked(class.image) };
                border_colors.insert_no_notify(event.id, c.clone());
                set_dirty(dirty_list, event.id, StyleType::BorderColor as usize, style_mark);
            }
        }

        if class.background_color > 0 {
            if class.class_style_mark & StyleType::BackgroundColor as usize != 0 && style_mark.local_style & StyleType::BackgroundColor as usize == 0 {
                let c = unsafe { class_sheet.background_color.get_unchecked(class.background_color) };
                background_colors.insert_no_notify(event.id, c.clone());
                set_dirty(dirty_list, event.id, StyleType::BackgroundColor as usize, style_mark);
            }
        }

        if class.box_shadow > 0 {
            let c = unsafe { class_sheet.box_shadow.get_unchecked(class.box_shadow) };
            if class.class_style_mark & StyleType::BoxShadow as usize != 0 && style_mark.local_style & StyleType::BoxShadow as usize == 0 {
                box_shadows.insert_no_notify(event.id, c.clone());
                set_dirty(dirty_list, event.id, StyleType::BoxShadow as usize, style_mark);
            }
        }

        if class.class_style_mark & StyleType::Opacity as usize != 0 && style_mark.local_style & StyleType::Opacity as usize == 0{
            opacitys.insert(event.id, Opacity(class.opacity));
            // set_dirty(dirty_list, event.id, StyleType::Opacity as usize, style_mark); 不需要设脏，opacity还需要通过级联计算得到最终值， 监听到该值的变化才会设脏
        }

        if class.class_style_mark & StyleType::Transform as usize != 0 && style_mark.local_style & StyleType::Transform as usize == 0{
            transforms.insert(event.id, class.transform.clone());;
            set_dirty(dirty_list, event.id, StyleType::Transform as usize, style_mark);
        }

        if class.class_style_mark & StyleType::BorderRadius as usize != 0 && style_mark.local_style & StyleType::BorderRadius as usize == 0{
            border_radiuses.insert(event.id, class.border_radius.clone());
            set_dirty(dirty_list, event.id, StyleType::BorderRadius as usize, style_mark);
        }

        if class.class_style_mark & StyleType::Filter as usize != 0 && style_mark.local_style & StyleType::Filter as usize == 0{
            filters.insert(event.id, class.filter.clone());;
            set_dirty(dirty_list, event.id, StyleType::Filter as usize, style_mark);
        }

        // 设置布局属性， 没有记录每个个属性是否在本地样式表中存在， TODO
        let yoga = unsafe {yogas.get_unchecked(event.id)};
        set_layout_style(class, yoga);

        style_mark.class_style = class.class_style_mark;
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
    fn listen(&mut self, _: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (class_names, class_sheet) = read;
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
                        let class_name = unsafe { class_names.get_unchecked(image_wait.id) }.0;
                        let class = match class_sheet.class.get(class_name) {
                            Some(class) => class,
                            None => continue, // 样式不存在， 跳过
                        };

                        if let Some(image_class) = class_sheet.image.get(class.image) {
                            if image_class.image == wait.0 {
                                images.insert_no_notify(image_wait.id, Image{src: wait.1.clone(), url: wait.0.clone()});
                                set_dirty(dirty_list, image_wait.id, StyleType::Image as usize, style_mark);
                            }
                        }
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
                        let class_name = unsafe { class_names.get_unchecked(image_wait.id) }.0;
                        let class = match class_sheet.class.get(class_name) {
                            Some(class) => class,
                            None => continue, // 样式不存在， 跳过
                        };
                        
                        if let Some(border_image_class) = class_sheet.border_image.get(class.border_image) {
                            if border_image_class.border_image == wait.0 {
                                border_images.insert(image_wait.id, BorderImage{src: wait.1.clone(), url: wait.0.clone()});
                                set_dirty(dirty_list, image_wait.id, StyleType::BorderImage as usize, style_mark);
                            }
                        }
                    },
                }
            }
        }
        image_wait_sheet.finish.clear(); // 清空
    }
}

fn set_layout_style<L: FlexNode>(class: &Class, yoga: &L){
    for layout_attr in class.layout.iter() {
        match layout_attr.clone() {
            LayoutAttr::AlignContent(r) => yoga.set_align_content(r),
            LayoutAttr::AlignItems(r) => yoga.set_align_items(r),
            LayoutAttr::AlignSelf(r) => yoga.set_align_self(r),
            LayoutAttr::JustifyContent(r) => yoga.set_justify_content(r),
            LayoutAttr::FlexDirection(r) => yoga.set_flex_direction(r),
            LayoutAttr::FlexWrap(r) => yoga.set_flex_wrap(r),
            LayoutAttr::PositionType(r) => yoga.set_position_type(r),
            LayoutAttr::Width(r) => match r {
                ValueUnit::Auto => yoga.set_width_auto(),
                ValueUnit::Undefined => yoga.set_width_auto(),
                ValueUnit::Pixel(r) => yoga.set_width(r),
                ValueUnit::Percent(r) => yoga.set_width_percent(r),
            },
            LayoutAttr::Height(r) => match r {
                ValueUnit::Auto => yoga.set_height_auto(),
                ValueUnit::Undefined => yoga.set_height_auto(),
                ValueUnit::Pixel(r) => yoga.set_height(r),
                ValueUnit::Percent(r) => yoga.set_height_percent(r),
            },
            LayoutAttr::MarginLeft(r) => match r {
                ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeLeft),
                ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeLeft),
                ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeLeft, r),
                ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeLeft, r),
            },
            LayoutAttr::MarginTop(r) => match r {
                ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeTop),
                ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeTop),
                ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeTop, r),
                ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeTop, r),
            },
            LayoutAttr::MarginBottom(r) => match r {
                ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeTop),
                ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeTop),
                ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeTop, r),
                ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeTop, r),
            },
            LayoutAttr::MarginRight(r) => match r {
                ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeRight),
                ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeRight),
                ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeRight, r),
                ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeRight, r),
            },
            LayoutAttr::Margin(r) => match r {
                ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeAll),
                ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeAll),
                ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeAll, r),
                ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeAll, r),
            },
            LayoutAttr::PaddingLeft(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeLeft, r),
                ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeLeft, r),
                _ => (),
            },
            LayoutAttr::PaddingTop(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeTop, r),
                ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeTop, r),
                _ => (),
            },
            LayoutAttr::PaddingBottom(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeTop, r),
                ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeTop, r),
                _ => (),
            },
            LayoutAttr::PaddingRight(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeRight, r),
                ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeRight, r),
                _ => (),
            },
            LayoutAttr::Padding(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeAll, r),
                ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeAll, r),
                _ => (),
            },
            LayoutAttr::BorderLeft(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeLeft, r),
                // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeLeft, r),
                _ => (),
            },
            LayoutAttr::BorderTop(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeTop, r),
                // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeTop, r),
                _ => (),
            },
            LayoutAttr::BorderBottom(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeTop, r),
                // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeTop, r),
                _ => (),
            },
            LayoutAttr::BorderRight(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeRight, r),
                // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeRight, r),
                _ => (),
            },
            LayoutAttr::Border(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeAll, r),
                // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeAll, r),
                _ => (),
            },
            LayoutAttr::MinWidth(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_min_width(r),
                ValueUnit::Percent(r) => yoga.set_min_width_percent(r),
                _ => (),
            },
            LayoutAttr::MinHeight(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_min_height(r),
                ValueUnit::Percent(r) => yoga.set_min_height_percent(r),
                _ => (),
            },
            LayoutAttr::MaxHeight(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_max_height(r),
                ValueUnit::Percent(r) => yoga.set_max_height_percent(r),
                _ => (),
            },
            LayoutAttr::MaxWidth(r) => match r {
                ValueUnit::Pixel(r) => yoga.set_max_width(r),
                ValueUnit::Percent(r) => yoga.set_max_width_percent(r),
                _ => (),
            },
            LayoutAttr::FlexBasis(r) => match r {
                ValueUnit::Auto => yoga.set_flex_basis_auto(),
                ValueUnit::Undefined => yoga.set_flex_basis_auto(),
                ValueUnit::Pixel(r) => yoga.set_flex_basis(r),
                ValueUnit::Percent(r) => yoga.set_flex_basis_percent(r),
            },
            LayoutAttr::FlexShrink(r) => yoga.set_flex_shrink(r),
            LayoutAttr::FlexGrow(r) => yoga.set_flex_grow(r),
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
        MultiCaseListener<Node, Opacity, ModifyEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>   
        MultiCaseListener<Node, BorderRadius, ModifyEvent>
        MultiCaseListener<Node, Transform, ModifyEvent>  
        MultiCaseListener<Node, Filter, ModifyEvent>

        MultiCaseListener<Node, ClassName, ModifyEvent> 

        SingleCaseListener<ImageWaitSheet, ModifyEvent>
    }
}
