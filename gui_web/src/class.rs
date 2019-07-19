use std::mem::transmute;
use std::str::FromStr;

use stdweb::unstable::TryInto;

use ecs::{LendMut};

use gui::component::user::*;
use gui::single::class::*;
pub use gui::layout::{YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType};
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
    let (mut bg_color, mut bg_color_change) = (BackgroundColor::default(), false);
    let mut class = Class::default();

    for attr in r.0.into_iter() {
        match attr {
            Attribute::BGColor(r) => {
                bg_color = r;
                bg_color_change = true;
            },
            _ => println!("set_class error"),
        }
    }

    if bg_color_change {
        class.background_color = class_sheet.background_color.insert(bg_color);
    }
    let c = class_sheet.class.insert(class);

    class_sheet.class_map.insert(class_id, c);
}

fn parse_class_from_string(value: &str) -> Result<(Vec<Attribute>, Vec<LayoutAttr>), String> {
    let mut show_attr = Vec::new();
    let mut layout_attr = Vec::new();
    for p in value.split(";") {
        match p.find(":") {
            Some(index) => {
                let p = p.split_at(index);
                let key = p.0.trim();
                let value = p.0.trim();
                match key {
                    "background-color" => show_attr.push(Attribute::BGColor( BackgroundColor(Color::RGBA(parse_color_string(value)?)))),
                    "width" => layout_attr.push(LayoutAttr::Width(parse_unity(value)?)),
                    "height" => layout_attr.push(LayoutAttr::Height(parse_unity(value)?)),
                    "margin-left" => layout_attr.push(LayoutAttr::MarginLeft(parse_unity(value)?)),
                    "margin-bottom" => layout_attr.push(LayoutAttr::MarginBottom(parse_unity(value)?)),
                    "margin-right" => layout_attr.push(LayoutAttr::MarginRight(parse_unity(value)?)),
                    "margin-top" => layout_attr.push(LayoutAttr::MarginTop(parse_unity(value)?)),
                    "margin" => layout_attr.push(LayoutAttr::Margin(parse_unity(value)?)),
                    "padding-left" => layout_attr.push(LayoutAttr::PaddingLeft(parse_unity(value)?)),
                    "padding-bottom" => layout_attr.push(LayoutAttr::PaddingBottom(parse_unity(value)?)),
                    "padding-right" => layout_attr.push(LayoutAttr::PaddingRight(parse_unity(value)?)),
                    "padding-top" => layout_attr.push(LayoutAttr::PaddingTop(parse_unity(value)?)),
                    "padding" => layout_attr.push(LayoutAttr::Padding(parse_unity(value)?)),
                    "border-left" => layout_attr.push(LayoutAttr::BorderLeft(parse_unity(value)?)),
                    "border-bottom" => layout_attr.push(LayoutAttr::BorderBottom(parse_unity(value)?)),
                    "border-right" => layout_attr.push(LayoutAttr::BorderRight(parse_unity(value)?)),
                    "border-top" => layout_attr.push(LayoutAttr::BorderTop(parse_unity(value)?)),
                    "border" => layout_attr.push(LayoutAttr::Border(parse_unity(value)?)),
                    "min-width" => layout_attr.push(LayoutAttr::MinWidth(parse_unity(value)?)),
                    "min-height" => layout_attr.push(LayoutAttr::MinHeight(parse_unity(value)?)),
                    "max-width" => layout_attr.push(LayoutAttr::MaxWidth(parse_unity(value)?)),
                    "max-height" => layout_attr.push(LayoutAttr::MaxHeight(parse_unity(value)?)),
                    "flex-basis" => layout_attr.push(LayoutAttr::FlexBasis(parse_unity(value)?)),
                    "flex-shrink" => layout_attr.push(LayoutAttr::FlexShrink(parse_f32(value)?)),
                    "flex-grow" => layout_attr.push(LayoutAttr::FlexGrow(parse_f32(value)?)),

                    "position" => layout_attr.push(LayoutAttr::PositionType(unsafe {transmute(parse_u8(value)?)})),
                    "flex-wrap" => layout_attr.push(LayoutAttr::FlexWrap(unsafe {transmute(parse_u8(value)?)})),
                    "flex-direction" => layout_attr.push(LayoutAttr::FlexDirection(unsafe {transmute(parse_u8(value)?)})),
                    "align-content" => layout_attr.push(LayoutAttr::AlignContent(unsafe {transmute(parse_u8(value)?)})),
                    "align-items" => layout_attr.push(LayoutAttr::AlignItems(unsafe {transmute(parse_u8(value)?)})),
                    "align-self" => layout_attr.push(LayoutAttr::AlignSelf(unsafe {transmute(parse_u8(value)?)})),
                    "justify-content" => layout_attr.push(LayoutAttr::JustifyContent(unsafe {transmute(parse_u8(value)?)})),
                    _ => (),
                }
            },
            None => return Err(format!("class parse err: {:?}", p)),
        }
        
    }
    Ok((show_attr, layout_attr))
}

fn parse_color_string(value: &str) -> Result<CgColor, String> {
    let value = &value[1..];
    match value {
        "red" => Ok(CgColor::new(1.0, 0.0, 0.0, 1.0)),
        "green" => Ok(CgColor::new(0.0, 1.0, 0.0, 1.0)),
        "blue" => Ok(CgColor::new(0.0, 0.0, 1.0, 1.0)),
        _ => Err("parse color err".to_string()),
    }
}

fn parse_unity(value: &str) -> Result<ValueUnit, String> {
    if value.starts_with("%") {
        let v = match f32::from_str(&value[1..]) {
            Ok(r) => r,
            Err(e) => return Err(e.to_string()),
        };
        Ok(ValueUnit::Percent(v/100.0))
    } else if value == "auto" {
        Ok(ValueUnit::Auto)
    } else if value.ends_with("px") {
        let v = match f32::from_str(&value[0..value.len() - 2]) {
            Ok(r) => r,
            Err(e) => return Err(e.to_string()),
        };
        Ok(ValueUnit::Pixel(v))
    }else {
        Err("parse_unity error".to_string())
    }
}

fn parse_u8(value: &str) -> Result<u8, String> {
    match u8::from_str(value) {
        Ok(r) => Ok(r),
        Err(e) => Err(e.to_string()),
    }
}


fn parse_f32(value: &str) -> Result<f32, String> {
    match f32::from_str(value) {
        Ok(r) => Ok(r),
        Err(e) => Err(e.to_string()),
    }
}

pub enum Attribute {
    BGColor(BackgroundColor),
    BorderColor(BorderColor),
    ZIndex(usize),
    Enable(EnableType),
    Display(Display),
    Visibility(bool),
    BorderRadius(BorderRadius),
    Opacity(Opacity),
    Transform(Transform),
    Filter(Filter),
}

