use std::mem::transmute;
use std::str::FromStr;

use component::user::*;
use single::class::*;
pub use layout::{YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType};

pub fn parse_class_from_string(value: &str) -> Result<(Vec<Attribute>, Vec<LayoutAttr>), String> {
    let mut show_attr = Vec::new();
    let mut layout_attr = Vec::new();
    for p in value.split(";") {
        match p.find(":") {
            Some(index) => {
                let p = p.split_at(index);
                let key = p.0.trim();
                let value = p.1[1..p.1.len()].trim();
                match match_key(key, value, &mut show_attr, &mut layout_attr) {
                    Err(r) => return Err(format!("{}, key: {}, value: {}", r, key, value)),
                    _ => (),
                };
            },
            None => if p.trim() != "" {
                return Err(format!("class parse err: {:?}", p))
            },
        }
        
    }
    Ok((show_attr, layout_attr))
}

fn match_key(key: &str, value: &str, show_attr: &mut Vec<Attribute>, layout_attr: &mut Vec<LayoutAttr>) -> Result<(), String> {
    match key {
        "background-color" => show_attr.push(Attribute::BGColor( BackgroundColor(Color::RGBA(parse_color_string(value)?)))),
        "background" => {
            if value.starts_with("linear-gradient") {
                show_attr.push(Attribute::BGColor( BackgroundColor(parse_linear_gradient_color_string(value)?)));
            } else {
                println!("background err: {}", value);
                return Ok(());
            }
        },
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
    };
    Ok(())
}

fn parse_linear_gradient_color_string(value: &str) -> Result<Color, String> {
    let value = &value[15..].trim();
    let value = value[1..value.len() - 1].trim();
    let mut iter= value.split(",");
    let first = iter.nth(0);
    let mut color = LinearGradientColor::default();
    let mut list = Vec::new();
    let mut pre_percent = 0.0;
    match first {
        Some(first) => {
            let first = first.trim();
            if first.ends_with("deg") {
                color.direction = parse_f32(&first[0..first.len() - 3])?;
            } else {
                parser_color_stop(first, &mut list, &mut color.list, &mut pre_percent)?;
            }
        },
        None => return Ok(Color::LinearGradient(color)),
    };

    for value in iter {
        let value = value.trim();
        parser_color_stop(value, &mut list, &mut color.list, &mut pre_percent)?;
    }

    parser_color_stop_last(1.0, &mut list, &mut color.list, &mut pre_percent, None)?;
    
    println!("color: {:?}, ", color);
    Ok(Color::LinearGradient(color))
}

fn parser_color_stop(value: &str, list: &mut Vec<CgColor>, color_stop: &mut Vec<ColorAndPosition>, pre_percent: &mut f32) -> Result<(), String>{
    if value.ends_with("%") {
        if let Some(index) = value.find(" ") {
            let r = value.split_at(index);
            let pos = r.1.trim();
            let v = match f32::from_str(&pos[0..pos.len() - 1]) {
                Ok(r) => r,
                Err(e) => return Err(e.to_string()),
            };
            let v = v/100.0;
            return parser_color_stop_last(v, list, color_stop, pre_percent, Some(parse_color_string(r.0.trim())?))
        }
    }
    list.push(parse_color_string(value.trim())?);
    Ok(())
}

fn parser_color_stop_last(v: f32, list: &mut Vec<CgColor>, color_stop: &mut Vec<ColorAndPosition>, pre_percent: &mut f32, last_color: Option<CgColor>) -> Result<(), String>{
    println!("v: {}, len: {}", v, list.len());
    if list.len() > 0 {
        let pos = (v - *pre_percent) / list.len() as f32;
        if color_stop.len() != 0 {
            for i in 0..list.len() {
                color_stop.push(ColorAndPosition{position: *pre_percent + pos * (i + 1) as f32, rgba: list[i]});
            }
        } else {
            for i in 0..list.len() {
                color_stop.push(ColorAndPosition{position: *pre_percent + pos * i as f32, rgba: list[i]});
            }
        }
        
        list.clear();
    }
    *pre_percent = v;
    if let Some(last_color) = last_color {
        color_stop.push(ColorAndPosition{position: v, rgba: last_color});
    }   
    Ok(())
}

fn parse_color_string(value: &str) -> Result<CgColor, String> {
    macro_rules! rgb {
        ($red: expr, $green: expr, $blue: expr) => {
            CgColor::new($red as f32 / 255.0, $green as f32 / 255.0, $blue as f32 / 255.0, 1.0)
        };
    }
    let color = match value {
        "black" => rgb!(0, 0, 0),
        "silver" => rgb!(192, 192, 192),
        "gray" => rgb!(128, 128, 128),
        "white" => rgb!(255, 255, 255),
        "maroon" => rgb!(128, 0, 0),
        "red" => rgb!(255, 0, 0),
        "purple" => rgb!(128, 0, 128),
        "fuchsia" => rgb!(255, 0, 255),
        "green" => rgb!(0, 128, 0),
        "lime" => rgb!(0, 255, 0),
        "olive" => rgb!(128, 128, 0),
        "yellow" => rgb!(255, 255, 0),
        "navy" => rgb!(0, 0, 128),
        "blue" => rgb!(0, 0, 255),
        "teal" => rgb!(0, 128, 128),
        "aqua" => rgb!(0, 255, 255),

        "aliceblue" => rgb!(240, 248, 255),
        "antiquewhite" => rgb!(250, 235, 215),
        "aquamarine" => rgb!(127, 255, 212),
        "azure" => rgb!(240, 255, 255),
        "beige" => rgb!(245, 245, 220),
        "bisque" => rgb!(255, 228, 196),
        "blanchedalmond" => rgb!(255, 235, 205),
        "blueviolet" => rgb!(138, 43, 226),
        "brown" => rgb!(165, 42, 42),
        "burlywood" => rgb!(222, 184, 135),
        "cadetblue" => rgb!(95, 158, 160),
        "chartreuse" => rgb!(127, 255, 0),
        "chocolate" => rgb!(210, 105, 30),
        "coral" => rgb!(255, 127, 80),
        "cornflowerblue" => rgb!(100, 149, 237),
        "cornsilk" => rgb!(255, 248, 220),
        "crimson" => rgb!(220, 20, 60),
        "cyan" => rgb!(0, 255, 255),
        "darkblue" => rgb!(0, 0, 139),
        "darkcyan" => rgb!(0, 139, 139),
        "darkgoldenrod" => rgb!(184, 134, 11),
        "darkgray" => rgb!(169, 169, 169),
        "darkgreen" => rgb!(0, 100, 0),
        "darkgrey" => rgb!(169, 169, 169),
        "darkkhaki" => rgb!(189, 183, 107),
        "darkmagenta" => rgb!(139, 0, 139),
        "darkolivegreen" => rgb!(85, 107, 47),
        "darkorange" => rgb!(255, 140, 0),
        "darkorchid" => rgb!(153, 50, 204),
        "darkred" => rgb!(139, 0, 0),
        "darksalmon" => rgb!(233, 150, 122),
        "darkseagreen" => rgb!(143, 188, 143),
        "darkslateblue" => rgb!(72, 61, 139),
        "darkslategray" => rgb!(47, 79, 79),
        "darkslategrey" => rgb!(47, 79, 79),
        "darkturquoise" => rgb!(0, 206, 209),
        "darkviolet" => rgb!(148, 0, 211),
        "deeppink" => rgb!(255, 20, 147),
        "deepskyblue" => rgb!(0, 191, 255),
        "dimgray" => rgb!(105, 105, 105),
        "dimgrey" => rgb!(105, 105, 105),
        "dodgerblue" => rgb!(30, 144, 255),
        "firebrick" => rgb!(178, 34, 34),
        "floralwhite" => rgb!(255, 250, 240),
        "forestgreen" => rgb!(34, 139, 34),
        "gainsboro" => rgb!(220, 220, 220),
        "ghostwhite" => rgb!(248, 248, 255),
        "gold" => rgb!(255, 215, 0),
        "goldenrod" => rgb!(218, 165, 32),
        "greenyellow" => rgb!(173, 255, 47),
        "grey" => rgb!(128, 128, 128),
        "honeydew" => rgb!(240, 255, 240),
        "hotpink" => rgb!(255, 105, 180),
        "indianred" => rgb!(205, 92, 92),
        "indigo" => rgb!(75, 0, 130),
        "ivory" => rgb!(255, 255, 240),
        "khaki" => rgb!(240, 230, 140),
        "lavender" => rgb!(230, 230, 250),
        "lavenderblush" => rgb!(255, 240, 245),
        "lawngreen" => rgb!(124, 252, 0),
        "lemonchiffon" => rgb!(255, 250, 205),
        "lightblue" => rgb!(173, 216, 230),
        "lightcoral" => rgb!(240, 128, 128),
        "lightcyan" => rgb!(224, 255, 255),
        "lightgoldenrodyellow" => rgb!(250, 250, 210),
        "lightgray" => rgb!(211, 211, 211),
        "lightgreen" => rgb!(144, 238, 144),
        "lightgrey" => rgb!(211, 211, 211),
        "lightpink" => rgb!(255, 182, 193),
        "lightsalmon" => rgb!(255, 160, 122),
        "lightseagreen" => rgb!(32, 178, 170),
        "lightskyblue" => rgb!(135, 206, 250),
        "lightslategray" => rgb!(119, 136, 153),
        "lightslategrey" => rgb!(119, 136, 153),
        "lightsteelblue" => rgb!(176, 196, 222),
        "lightyellow" => rgb!(255, 255, 224),
        "limegreen" => rgb!(50, 205, 50),
        "linen" => rgb!(250, 240, 230),
        "magenta" => rgb!(255, 0, 255),
        "mediumaquamarine" => rgb!(102, 205, 170),
        "mediumblue" => rgb!(0, 0, 205),
        "mediumorchid" => rgb!(186, 85, 211),
        "mediumpurple" => rgb!(147, 112, 219),
        "mediumseagreen" => rgb!(60, 179, 113),
        "mediumslateblue" => rgb!(123, 104, 238),
        "mediumspringgreen" => rgb!(0, 250, 154),
        "mediumturquoise" => rgb!(72, 209, 204),
        "mediumvioletred" => rgb!(199, 21, 133),
        "midnightblue" => rgb!(25, 25, 112),
        "mintcream" => rgb!(245, 255, 250),
        "mistyrose" => rgb!(255, 228, 225),
        "moccasin" => rgb!(255, 228, 181),
        "navajowhite" => rgb!(255, 222, 173),
        "oldlace" => rgb!(253, 245, 230),
        "olivedrab" => rgb!(107, 142, 35),
        "orange" => rgb!(255, 165, 0),
        "orangered" => rgb!(255, 69, 0),
        "orchid" => rgb!(218, 112, 214),
        "palegoldenrod" => rgb!(238, 232, 170),
        "palegreen" => rgb!(152, 251, 152),
        "paleturquoise" => rgb!(175, 238, 238),
        "palevioletred" => rgb!(219, 112, 147),
        "papayawhip" => rgb!(255, 239, 213),
        "peachpuff" => rgb!(255, 218, 185),
        "peru" => rgb!(205, 133, 63),
        "pink" => rgb!(255, 192, 203),
        "plum" => rgb!(221, 160, 221),
        "powderblue" => rgb!(176, 224, 230),
        "rebeccapurple" => rgb!(102, 51, 153),
        "rosybrown" => rgb!(188, 143, 143),
        "royalblue" => rgb!(65, 105, 225),
        "saddlebrown" => rgb!(139, 69, 19),
        "salmon" => rgb!(250, 128, 114),
        "sandybrown" => rgb!(244, 164, 96),
        "seagreen" => rgb!(46, 139, 87),
        "seashell" => rgb!(255, 245, 238),
        "sienna" => rgb!(160, 82, 45),
        "skyblue" => rgb!(135, 206, 235),
        "slateblue" => rgb!(106, 90, 205),
        "slategray" => rgb!(112, 128, 144),
        "slategrey" => rgb!(112, 128, 144),
        "snow" => rgb!(255, 250, 250),
        "springgreen" => rgb!(0, 255, 127),
        "steelblue" => rgb!(70, 130, 180),
        "tan" => rgb!(210, 180, 140),
        "thistle" => rgb!(216, 191, 216),
        "tomato" => rgb!(255, 99, 71),
        "turquoise" => rgb!(64, 224, 208),
        "violet" => rgb!(238, 130, 238),
        "wheat" => rgb!(245, 222, 179),
        "whitesmoke" => rgb!(245, 245, 245),
        "yellowgreen" => rgb!(154, 205, 50),

        "transparent" => rgba(0, 0, 0, 0),
        _ => if value.starts_with("#") {
            parse_color_hex(&value[1..])?
        } 
        // else if value.starts_with("rgba") {

        // } else if value.starts_with("rgb") {

        // } 
        else {
            return Err(format!("parse color err: '{}'", value))
        },
    };
    Ok(color)
}

fn parse_unity(value: &str) -> Result<ValueUnit, String> {
    if value.ends_with("%") {
        let v = match f32::from_str(value) {
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
        Err(e) => Err(format!("{:?}: {}", e.to_string(), value) ),
    }
}

fn parse_color_hex(value: &str) -> Result<CgColor, String> {
    let value = value.as_bytes();
    match value.len() {
        8 => Ok(rgba(
            from_hex(value[0])? * 16 + from_hex(value[1])?,
            from_hex(value[2])? * 16 + from_hex(value[3])?,
            from_hex(value[4])? * 16 + from_hex(value[5])?,
            from_hex(value[6])? * 16 + from_hex(value[7])?,
        )),
        6 => Ok(rgba(
            from_hex(value[0])? * 16 + from_hex(value[1])?,
            from_hex(value[2])? * 16 + from_hex(value[3])?,
            from_hex(value[4])? * 16 + from_hex(value[5])?,
            255,
        )),
        4 => Ok(rgba(
            from_hex(value[0])? * 17,
            from_hex(value[1])? * 17,
            from_hex(value[2])? * 17,
            from_hex(value[3])? * 17,
        )),
        3 => Ok(rgba(
            from_hex(value[0])? * 17,
            from_hex(value[1])? * 17,
            from_hex(value[2])? * 17,
            255,
        )),
        _ => Err("".to_string()),
    }
}

fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> CgColor {
    CgColor::new(red as f32 / 255.0, green as f32 / 255.0, blue as f32 / 255.0, alpha as f32 / 255.0)
}

fn from_hex(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err("".to_string()),
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

