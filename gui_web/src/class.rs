use stdweb::unstable::TryInto;

use ecs::LendMut;

use gui::component::calc::{StyleType, StyleType1};
use gui::component::user::*;
use gui::single::{ class::*, style_parse::{Attribute, parse_class_from_string} };
pub use gui::layout::{ YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType };
use GuiWorld;


/**
 * 在指定上下文中创建一个 文本样式表
 * __jsObj: class样式的文本描述
 */
#[allow(unused_attributes)]
#[no_mangle]
pub fn create_class(world: u32, class_id: u32) {
    let value: String = js!(return __jsObj;).try_into().unwrap();

    let r = match parse_class_from_string(value.as_str()) {
        Ok(r) => r,
        Err(e) => {
            println!("{:?}", e);
            return;
        },
    };

    set_class(world, class_id, r);
}

fn set_class(world: u32, class_id: u32, r: (Vec<Attribute>, Vec<LayoutAttr>)) {
    let class_id = class_id as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
    let class_sheet = world.gui.class_sheet.lend_mut();
    // let (mut text_style, mut text_style_change) = (TextStyle::default(), false);
    // let (mut bg_color, mut bg_color_change) = (BackgroundColor::default(), false);
    let mut class = Class::default();
    class.layout =  r.1;

    for attr in r.0.into_iter() {
        match attr {
            Attribute::BGColor(r) => {
                class.background_color = class_sheet.background_color.insert(r);
                class.class_style_mark |= StyleType::BackgroundColor as usize;
            },
            Attribute::Color(r) => {
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.color = r;
                class.class_style_mark |= StyleType::Color as usize;
            },
            Attribute::LetterSpacing(r) => {
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.letter_spacing = r;
                class.class_style_mark |= StyleType::LetterSpacing as usize;
            },
            Attribute::WordSpacing(r) => {
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.word_spacing = r;
                class.class_style_mark |= StyleType::WordSpacing as usize;
            },
            Attribute::LineHeight(r) => {
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.line_height = r;
                class.class_style_mark |= StyleType::LineHeight as usize;
            },
            Attribute::TextAlign(r) => {
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.text_align = r;
                class.class_style_mark |= StyleType::TextAlign as usize;
            },
            // Attribute::TextIndent(r) => {
            //     // let t = or_insert_text_style(&mut class, class_sheet);
            //     // t.text.indent = r;
            //     // class.class_style_mark |= StyleType::Tex as usize;
            // },
            Attribute::TextShadow(r) => {
                let t = or_insert_text_style(&mut class, class_sheet);
                t.shadow = r;
                class.class_style_mark |= StyleType::TextShadow as usize;
            },
            Attribute::WhiteSpace(r) => {
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.white_space = r;
                class.class_style_mark |= StyleType::WhiteSpace as usize;
            },
            Attribute::TextStroke(r) => {
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.stroke = r;
                class.class_style_mark |= StyleType::Stroke as usize;
            },
            Attribute::FontWeight(r) => {
                let t = or_insert_text_style(&mut class, class_sheet);
                t.font.weight = r as usize;
                class.class_style_mark |= StyleType::FontWeight as usize;
            },
            Attribute::FontSize(r) => {
                let t = or_insert_text_style(&mut class, class_sheet);
                t.font.size = r;
                class.class_style_mark |= StyleType::FontSize as usize;
            },
            Attribute::FontFamily(r) => {
                let t = or_insert_text_style(&mut class, class_sheet);
                t.font.family = r;
                class.class_style_mark |= StyleType::FontFamily as usize;
            },

            Attribute::BorderImageUrl(r) => {
                let t = or_insert_border_image_style(&mut class, class_sheet);
                t.border_image = r;
                class.class_style_mark |= StyleType::BorderImage as usize;
            },
            Attribute::BorderImageClip(r) => {
                let t = or_insert_border_image_style(&mut class, class_sheet);
                t.border_image_clip = r;
                class.class_style_mark |= StyleType::BorderImageClip as usize;
            },
            Attribute::BorderImageSlice(r) => {
                let t = or_insert_border_image_style(&mut class, class_sheet);
                t.border_image_slice = r;
                class.class_style_mark |= StyleType::BorderImageSlice as usize;
            },
            Attribute::BorderImageRepeat(r) => {
                let t = or_insert_border_image_style(&mut class, class_sheet);
                t.border_image_repeat = r;
                class.class_style_mark |= StyleType::BorderImageRepeat as usize;
            },

            Attribute::ImageUrl(r) => {
                let t = or_insert_image_style(&mut class, class_sheet);
                t.image = r;
                class.class_style_mark |= StyleType::Image as usize;
            },
            Attribute::ImageClip(r) => {
                let t = or_insert_image_style(&mut class, class_sheet);
                t.image_clip = r;
                class.class_style_mark |= StyleType::ImageClip as usize;
            },
            Attribute::ObjectFit(r) => {
                let t = or_insert_image_style(&mut class, class_sheet);
                t.obj_fit = r.0;
                class.class_style_mark |= StyleType::ObjectFit as usize;
            },

            Attribute::BorderColor(r) => {
                class.border_color = class_sheet.border_color.insert(r);
                class.class_style_mark |= StyleType::BorderColor as usize;
            },


            Attribute::TransformFunc(r) => {
                class.transform.funcs = r;
                class.class_style_mark |= StyleType::Transform as usize;
            },
            Attribute::TransformOrigin(r) => {
                class.transform.origin = r;
                class.class_style_mark |= StyleType::Transform as usize;
            },
            Attribute::ZIndex(r) => {
                class.z_index = r;
                class.class_style_mark |= StyleType1::ZIndex as usize;
            },
            Attribute::Visibility(r) => {
                class.visibility = r;
                class.class_style_mark |= StyleType1::Visibility as usize;
            },
            Attribute::Enable(r) => {
                class.enable = r;
                class.class_style_mark |= StyleType1::Enable as usize;
            },
            Attribute::Display(r) => {
                class.display = r;
                class.class_style_mark |= StyleType1::Display as usize;
            },
            Attribute::Filter(r) => {
                class.filter = r;
                class.class_style_mark |= StyleType::Filter as usize;
            },
            Attribute::Opacity(r) => {
                class.opacity = r.0;
                class.class_style_mark |= StyleType::Opacity as usize;
            },  
            Attribute::BorderRadius(r) => {
                class.border_radius = r;
                class.class_style_mark |= StyleType::BorderRadius as usize;
            },
            _ => println!("set_class error"),
        }
    }

    let c = class_sheet.class.insert(class);

    class_sheet.class_map.insert(class_id, c);

    let notify = class_sheet.get_notify();
    notify.create_event(c);
}

fn or_insert_text_style<'a>(class: &'a mut Class, class_sheet: &'a mut ClassSheet) -> &'a mut TextStyle{
    if class.text == 0 {
        let i = class_sheet.text.insert(TextStyle::default());
        class.text = i;

    }
    unsafe { class_sheet.text.get_unchecked_mut(class.text) }
}

fn or_insert_border_image_style<'a>(class: &'a mut Class, class_sheet: &'a mut ClassSheet) -> &'a mut BorderImageClass{
    if class.border_image == 0 {
        let i = class_sheet.border_image.insert(BorderImageClass::default());
        class.border_image = i;

    }
    unsafe { class_sheet.border_image.get_unchecked_mut(class.border_image) }
}

fn or_insert_image_style<'a>(class: &'a mut Class, class_sheet: &'a mut ClassSheet) -> &'a mut ImageClass{
    if class.image == 0 {
        let i = class_sheet.image.insert(ImageClass::default());
        class.image = i;

    }
    unsafe { class_sheet.image.get_unchecked_mut(class.image) }
}