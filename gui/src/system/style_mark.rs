/**
 * 监听transform和layout组件， 利用transform和layout递归计算节点的世界矩阵（worldmatrix组件）
 */
use std::marker::PhantomData;

use ecs::{CreateEvent, ModifyEvent, MultiCaseListener, EntityListener, SingleCaseImpl, SingleCaseListener, MultiCaseImpl};
use ecs::idtree::{ IdTree};
use hal_core::*;

use component::user::*;
use component::calc::*;
use component::calc::Opacity as COpacity;
use component::user::Opacity;
use single::class::ClassSheet;
use entity::{Node};
use render::engine::Engine;

#[derive(Default)]
pub struct StyleMarkSys<C>(PhantomData<C>);

#[inline]
fn set_dirty(id: usize, ty: usize, style_marks: &mut MultiCaseImpl<Node, StyleMark> ) {
    unsafe { style_marks.get_unchecked_mut(id) }.dirty |= ty;
}

// 监听TextStyle属性的改变
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, TextStyle, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, style_marks: Self::WriteData) {
        let r = match event.field {
            "letter_spacing" => StyleType::LetterSpacing,
            "word_spacing" => StyleType::WordSpacing,
            "line_height" => StyleType::LineHeight,
            "indent" => StyleType::Indent,
            "color" => StyleType::Color,
            "stroke" => StyleType::Stroke,
            "text_align" => StyleType::TextAlign,
            "vertical_align" => StyleType::VerticalAlign,
            _ => return
        };
        set_dirty(event.id, r as usize, style_marks);
    }
}

// 监听Font属性的改变
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Font, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, style_marks: Self::WriteData) {
        let r = match event.field {
            "style" => StyleType::FontStyle,
            "weight" => StyleType::FontWeight,
            "size" => StyleType::FontSize,
            "family" => StyleType::FontFamily,
            _ => return
        };
        set_dirty(event.id, r as usize, style_marks);
    }
}

// 监听TextShadow属性的改变
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, TextShadow, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;

    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, style_marks: Self::WriteData) {
        set_dirty(event.id, StyleType::TextShadow as usize, style_marks);
    }
}

// 监听Image的改变
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Image, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, style_marks: Self::WriteData) {
        set_dirty(event.id, StyleType::Image as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> EntityListener<'a, ImageClip, CreateEvent> for StyleMarkSys<C>{
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, style_marks: Self::WriteData){
        set_dirty(event.id, StyleType::ImageClip as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ObjectFit, CreateEvent> for StyleMarkSys<C>{
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, style_marks: Self::WriteData){
        set_dirty(event.id, StyleType::ObjectFit as usize, style_marks);
    }
}

// 监听Image的改变
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImageClip, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, style_marks: Self::WriteData) {
        set_dirty(event.id, StyleType::BorderImageClip as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> EntityListener<'a, BorderImageSlice, CreateEvent> for StyleMarkSys<C>{
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, style_marks: Self::WriteData){
        set_dirty(event.id, StyleType::BorderImageSlice as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImage, CreateEvent> for StyleMarkSys<C>{
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, style_marks: Self::WriteData){
        set_dirty(event.id, StyleType::BorderImage as usize, style_marks);
    }
}

// 监听Image的改变
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImageRepeat, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, style_marks: Self::WriteData) {
        set_dirty(event.id, StyleType::BorderImageRepeat as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> EntityListener<'a, BorderColor, CreateEvent> for StyleMarkSys<C>{
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, style_marks: Self::WriteData){
        set_dirty(event.id, StyleType::BorderColor as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BackgroundColor, CreateEvent> for StyleMarkSys<C>{
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, style_marks: Self::WriteData){
        set_dirty(event.id, StyleType::BackgroundColor as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BoxShadow, CreateEvent> for StyleMarkSys<C>{
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, style_marks: Self::WriteData){
        set_dirty(event.id, StyleType::BoxShadow as usize, style_marks);
   
   
    }
}

impl<'a, C: HalContext + 'static> EntityListener<'a, WorldMatrix, CreateEvent> for StyleMarkSys<C>{
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, style_marks: Self::WriteData){
        set_dirty(event.id, StyleType::Matrix as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Opacity, CreateEvent> for StyleMarkSys<C>{
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, style_marks: Self::WriteData){
        set_dirty(event.id, StyleType::Opacity as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Layout, CreateEvent> for StyleMarkSys<C>{
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, style_marks: Self::WriteData){
        set_dirty(event.id, StyleType::Layout as usize, style_marks);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ClassName, ModifyEvent> for StyleMarkSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ClassName>,
        &'a SingleCaseImpl<ClassSheet>
    );
    type WriteData = (
        &'a mut MultiCaseImpl<Node, TextStyle>,
        &'a mut MultiCaseImpl<Node, Font>,
        &'a mut MultiCaseImpl<Node, TextShadow>,
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
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (class_names, class_sheet) = read;
        let (
            text_styles,
            fonts,
            text_shadows,
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
            world_matrixs,
            opacitys,
            transforms,
            border_radiuses,
            style_marks,
            engine,
            ) = write;
        
        let class_name = unsafe { class_names.get_unchecked(event.id) }.0;
        let class = match class_sheet.class.get(class_name) {
            Some(class) => class,
            None => return,
        };

        let style_mark = unsafe { style_marks.get_unchecked(event.id) };

        if class.text > 0 {
            let mut text_style = unsafe { text_styles.get_unchecked_mut(event.id) };
            let mut font = unsafe { fonts.get_unchecked_mut(event.id) };
            let mut text_shadow = unsafe { text_shadows.get_unchecked_mut(event.id) };
            let c = unsafe { class_sheet.text.get_unchecked(class.text) };
            if style_mark.local_style & StyleType::FontStyle as usize == 0 {
                font.style = c.font.style
            }
            if style_mark.local_style & StyleType::FontWeight as usize == 0 {
                font.weight = c.font.weight
            }
            if style_mark.local_style & StyleType::FontSize as usize == 0 {
                font.size = c.font.size
            }
            if style_mark.local_style & StyleType::FontFamily as usize == 0 {
                font.family = c.font.family.clone()
            }
            if style_mark.local_style & StyleType::LetterSpacing as usize == 0 {
                text_style.letter_spacing = c.style.letter_spacing
            }
            if style_mark.local_style & StyleType::WordSpacing as usize == 0 {
                text_style.word_spacing = c.style.word_spacing
            }
            if style_mark.local_style & StyleType::LineHeight as usize == 0 {
                text_style.line_height = c.style.line_height
            }
            if style_mark.local_style & StyleType::Indent as usize == 0 {
                text_style.indent = c.style.indent
            }
            if style_mark.local_style & StyleType::WhiteSpace as usize == 0 {
                text_style.white_space = c.style.white_space
            }
            if style_mark.local_style & StyleType::Color as usize == 0 {
                text_style.color = c.style.color.clone()
            }
            if style_mark.local_style & StyleType::Stroke as usize == 0 {
                text_style.stroke = c.style.stroke.clone()
            }
            if style_mark.local_style & StyleType::TextAlign as usize == 0 {
                text_style.text_align = c.style.text_align
            }
            if style_mark.local_style & StyleType::VerticalAlign as usize == 0 {
                text_style.vertical_align = c.style.vertical_align
            }
            if style_mark.local_style & StyleType::TextShadow as usize == 0 {
                *text_shadow = c.shadow.clone()
            }
        }

        if class.image > 0 {
            let mut image = unsafe { images.get_unchecked_mut(event.id) };
            let mut image_clip = unsafe { image_clips.get_unchecked_mut(event.id) };
            let mut obj_fit = unsafe { obj_fits.get_unchecked_mut(event.id) };
            let c = unsafe { class_sheet.image.get_unchecked(class.image) };
            if style_mark.local_style & StyleType::Image as usize == 0 {
                // 异步加载图片
                // *image = c.style.text_align
            }
            if style_mark.local_style & StyleType::ImageClip as usize == 0 {
                match &
                c.image_clip {
                    Some(clip) => *image_clip = clip.clone(),
                    None => ()
                }
            }
            if style_mark.local_style & StyleType::ObjectFit as usize == 0 {
                obj_fit.0 = c.obj_fit.clone()
            }
        }

        if class.border_image > 0 {
            let mut border_image = unsafe { border_images.get_unchecked_mut(event.id) };
            let mut border_image_clip = unsafe { border_image_clips.get_unchecked_mut(event.id) };
            let mut border_image_slice = unsafe { border_image_slices.get_unchecked_mut(event.id) };
            let mut border_image_repeat = unsafe { border_image_repeats.get_unchecked_mut(event.id) };
            let c = unsafe { class_sheet.border_image.get_unchecked(class.image) };
            if style_mark.local_style & StyleType::BorderImage as usize == 0 {
                // 异步加载图片
                // *image = c.style.text_align
            }
            if style_mark.local_style & StyleType::BorderImageClip as usize == 0 {
                *border_image_clip = c.border_image_clip.clone()
            }
            if style_mark.local_style & StyleType::BorderImageSlice as usize == 0 {
                *border_image_slice = c.border_image_slice.clone()
            }
            if style_mark.local_style & StyleType::BorderImageClip as usize == 0 {
                *border_image_repeat = c.border_image_repeat.clone()
            }
        }

        if class.border_color > 0 {
            let mut border_color = unsafe { border_colors.get_unchecked_mut(event.id) };
            let c = unsafe { class_sheet.border_color.get_unchecked(class.image) };
            if style_mark.local_style & StyleType::BorderColor as usize == 0 {
                *border_color = c.clone();
            }
        }

        if class.background_color > 0 {
            let mut background_color = unsafe { background_colors.get_unchecked_mut(event.id) };
            let c = unsafe { class_sheet.background_color.get_unchecked(class.image) };
            if style_mark.local_style & StyleType::BackgroundColor as usize == 0 {
                *background_color = c.clone();
            }
        }

        if class.box_shadow > 0 {
            let mut box_shadow = unsafe { box_shadows.get_unchecked_mut(event.id) };
            let c = unsafe { class_sheet.box_shadow.get_unchecked(class.image) };
            if style_mark.local_style & StyleType::BoxShadow as usize == 0 {
                *box_shadow = c.clone();
            }
        }

        if class.class_style_mark & StyleType::Opacity as usize != 0 && style_mark.local_style & StyleType::Opacity as usize == 0{
            border_radiuses.insert(event.id, class.border_radius.clone());
        }

        if class.class_style_mark & StyleType::Transform as usize != 0 && style_mark.local_style & StyleType::Transform as usize == 0{
            transforms.insert(event.id, class.transform.clone());
        }
        // Transform, Opacity， Layout设置， TODO

    }
}

// impl_system!{
//     StyleMarkSys,
//     false,
//     {
//         EntityListener<Node, CreateEvent>
//         MultiCaseListener<Node, Text, ModifyEvent>
//         MultiCaseListener<Node, Font, ModifyEvent>
//         MultiCaseListener<Node, TextShadow, ModifyEvent>

//         MultiCaseListener<Node, Image, ModifyEvent>
//         MultiCaseListener<Node, ImageClip, ModifyEvent>
//         MultiCaseListener<Node, ObjectFit, ModifyEvent>
//         MultiCaseListener<Node, BorderImage, ModifyEvent>
//         MultiCaseListener<Node, BorderImageClip, ModifyEvent>
//         MultiCaseListener<Node, BorderImageSlice, ModifyEvent>
//         MultiCaseListener<Node, BorderImageRepeat, ModifyEvent>
//         MultiCaseListener<Node, BorderColor, ModifyEvent>
//         MultiCaseListener<Node, BackgroundColor, ModifyEvent>
//         MultiCaseListener<Node, BoxShadow, ModifyEvent>   

//         MultiCaseListener<Node, WorldMatrix, ModifyEvent>
//         MultiCaseListener<Node, Opacity, ModifyEvent>
//         MultiCaseListener<Node, Layout, ModifyEvent>   
//     }
// }
