use stdweb::unstable::TryInto;

use ecs::LendMut;

use gui::component::calc::StyleType;
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
                println!("set color");
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.color = r;
                class.class_style_mark |= StyleType::Color as usize;
            },
            Attribute::LetterSpacing(r) => {
                println!("set LetterSpacing");
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.letter_spacing = r;
                class.class_style_mark |= StyleType::LetterSpacing as usize;
            },
            Attribute::WordSpacing(r) => {
                println!("set WordSpacing");
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.word_spacing = r;
                class.class_style_mark |= StyleType::WordSpacing as usize;
            },
            Attribute::LineHeight(r) => {
                println!("set LineHeight");
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.line_height = r;
                class.class_style_mark |= StyleType::LineHeight as usize;
            },
            Attribute::TextAlign(r) => {
                println!("set TextAlign");
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.text_align = r;
                class.class_style_mark |= StyleType::TextAlign as usize;
            },
            Attribute::TextIndent(r) => {
                // let t = or_insert_text_style(&mut class, class_sheet);
                // t.text.indent = r;
                // class.class_style_mark |= StyleType::Tex as usize;
            },
            Attribute::TextShadow(r) => {
                println!("set TextShadow");
                let t = or_insert_text_style(&mut class, class_sheet);
                t.shadow = r;
                class.class_style_mark |= StyleType::TextShadow as usize;
            },
            Attribute::WhiteSpace(r) => {
                println!("set WhiteSpace");
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.white_space = r;
                class.class_style_mark |= StyleType::WhiteSpace as usize;
            },
            Attribute::TextStroke(r) => {
                println!("set TextStroke");
                let t = or_insert_text_style(&mut class, class_sheet);
                t.text.stroke = r;
                class.class_style_mark |= StyleType::Stroke as usize;
            },
            Attribute::FontWeight(r) => {
                println!("set FontWeight");
                let t = or_insert_text_style(&mut class, class_sheet);
                t.font.weight = r as usize;
                class.class_style_mark |= StyleType::FontWeight as usize;
            },
            Attribute::FontSize(r) => {
                println!("set FontSize");
                let t = or_insert_text_style(&mut class, class_sheet);
                t.font.size = r;
                class.class_style_mark |= StyleType::FontSize as usize;
            },
            Attribute::FontFamily(r) => {
                println!("set FontFamily");
                let t = or_insert_text_style(&mut class, class_sheet);
                t.font.family = r;
                class.class_style_mark |= StyleType::FontFamily as usize;
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
    let t = unsafe { class_sheet.text.get_unchecked_mut(class.text) };
    println!("t ================================{:?}", t);
    t
}