use std::mem::transmute;

use stdweb::unstable::TryInto;

use wcs::component::{Builder};
use wcs::world::{World};
use atom::Atom;

use world_doc::WorldDocMgr;
use world_doc::component::style::element::{ElementId, TextWriteRef};
use world_doc::component::style::text::{TextStyleWriteRef, Shadow, TextStyle, TextStyleBuilder, ShadowBuilder, OutLine};
use world_doc::component::style::font::{Font, FontWriteRef};
use text_layout::layout::{LineHeight};
use font::font_sheet::FontSize;
use component::math::Color as MathColor;

pub use layout::yoga::{YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType};

#[no_mangle]
pub fn set_letter_spacing(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let style_id = world.component_mgr.node.element.text._group.get(text_id).text_style;
    if style_id == 0 {
        let mut style = TextStyle::default();
        style.letter_spacing = value;
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_text_style(style);
    } else {
        TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr).set_letter_spacing(value);
    }
    debug_println!("set_letter_spacing"); 
}

#[no_mangle]
pub fn set_line_height_normal(world: u32, node_id: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let style_id = world.component_mgr.node.element.text._group.get(text_id).text_style;
    if style_id == 0 {
        let style = TextStyle::default();
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_text_style(style);
    } else {
        TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr).set_line_height(LineHeight::Normal);
    }
    debug_println!("set_line_height_normal"); 
}

#[no_mangle]
pub fn set_line_height(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let style_id = world.component_mgr.node.element.text._group.get(text_id).text_style;
    if style_id == 0 {
        let mut style = TextStyle::default();
        style.line_height = LineHeight::Length(value);
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_text_style(style);
    } else {
        TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr).set_line_height(LineHeight::Length(value));
    }
    debug_println!("set_line_height"); 
}

#[no_mangle]
pub fn set_line_height_percent(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let style_id = world.component_mgr.node.element.text._group.get(text_id).text_style;
    if style_id == 0 {
        let mut style = TextStyle::default();
        style.line_height = LineHeight::Percent(value);
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_text_style(style);
    } else {
        TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr).set_line_height(LineHeight::Percent(value));
    }
    debug_println!("set_line_height_percent"); 
}

#[no_mangle]
pub fn set_text_indent(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let style_id = world.component_mgr.node.element.text._group.get(text_id).text_style;
    if style_id == 0 {
        let mut style = TextStyle::default();
        style.text_indent = value;
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_text_style(style);
    } else {
        TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr).set_text_indent(value);
    }
    debug_println!("set_text_indent"); 
}

#[no_mangle]
pub fn set_out_line(world: u32, node_id: u32, thickness: f32, blur: f32, r: f32, g: f32, b: f32, a: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let style_id = world.component_mgr.node.element.text._group.get(text_id).text_style;
    let out_line = OutLine {
        thickness, blur,
        color: MathColor::new(r, g, b, a),
    };
    if style_id == 0 {
        let mut style = TextStyle::default();
        style.out_line = out_line;
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_text_style(style);
    } else {
        TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr).set_out_line(out_line);
    }
    debug_println!("set_white_space"); 
}

#[no_mangle]
pub fn set_white_space(world: u32, node_id: u32, value: u8){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let style_id = world.component_mgr.node.element.text._group.get(text_id).text_style;
    if style_id == 0 {
        let mut style = TextStyle::default();
        style.white_space = unsafe {transmute(value)};
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_text_style(style);
    } else {
        TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr).set_white_space(unsafe {transmute(value)});
    }
    debug_println!("set_white_space"); 
}

#[no_mangle]
pub fn set_text_shadow_h(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let style_id = world.component_mgr.node.element.text._group.get(text_id).text_style;
    if style_id == 0 {
        let style = TextStyleBuilder::new()
        .shadow(
            ShadowBuilder::new()
            .h(value)
            .build(&mut world.component_mgr.node.element.text.text_style.shadow))
        .build(&mut world.component_mgr.node.element.text.text_style);
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_text_style(style);
    } else {
        {
            let mut style_ref = TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr);
            let mut shadow_ref = style_ref.get_shadow_mut();
            if shadow_ref.id > 0 {
                shadow_ref.set_h(value);
                return;
            }
        }
        let shadow = ShadowBuilder::new()
            .h(value)
            .build(&mut world.component_mgr.node.element.text.text_style.shadow);
        let mut style_ref = TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr);
        style_ref.set_shadow(shadow);
    }   
    debug_println!("set_h_text_shadow"); 
}

#[no_mangle]
pub fn set_text_shadow_v(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let style_id = world.component_mgr.node.element.text._group.get(text_id).text_style;
    if style_id == 0 {
        let style = TextStyleBuilder::new()
        .shadow(
            ShadowBuilder::new()
            .v(value)
            .build(&mut world.component_mgr.node.element.text.text_style.shadow))
        .build(&mut world.component_mgr.node.element.text.text_style);
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_text_style(style);
    } else {
        {   
            let mut style_ref = TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr);
            let mut shadow_ref = style_ref.get_shadow_mut();
            if shadow_ref.id > 0 {
                shadow_ref.set_v(value);
                return;
            }
        }
        let shadow = ShadowBuilder::new()
            .v(value)
            .build(&mut world.component_mgr.node.element.text.text_style.shadow);
        let mut style_ref = TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr);
        style_ref.set_shadow(shadow);
    }
    debug_println!("set_text_shadow_v"); 
}

#[no_mangle]
pub fn set_text_shadow_blur(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let style_id = world.component_mgr.node.element.text._group.get(text_id).text_style;
    if style_id == 0 {
        let style = TextStyleBuilder::new()
        .shadow(
            ShadowBuilder::new()
            .blur(value)
            .build(&mut world.component_mgr.node.element.text.text_style.shadow))
        .build(&mut world.component_mgr.node.element.text.text_style);
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_text_style(style);
    } else {  
        {   
            let mut style_ref = TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr);
            let mut shadow_ref = style_ref.get_shadow_mut();
            if shadow_ref.id > 0 {
                shadow_ref.set_blur(value);
                return;
            }
        }
        let shadow = ShadowBuilder::new()
            .blur(value)
            .build(&mut world.component_mgr.node.element.text.text_style.shadow);
        let mut style_ref = TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr);
        style_ref.set_shadow(shadow);
    }
    debug_println!("set_text_shadow_blur"); 
}

#[no_mangle]
pub fn set_text_shadow_color(world: u32, node_id: u32, r: f32, g: f32, b: f32, a: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let style_id = world.component_mgr.node.element.text._group.get(text_id).text_style;
    if style_id == 0 {
        let style = TextStyleBuilder::new()
        .shadow(
            ShadowBuilder::new()
            .color(MathColor::new(r, g, b, a))
            .build(&mut world.component_mgr.node.element.text.text_style.shadow))
        .build(&mut world.component_mgr.node.element.text.text_style);
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_text_style(style);
    } else {
        
        {   
            let mut style_ref = TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr);
            let mut shadow_ref = style_ref.get_shadow_mut();
            if shadow_ref.id > 0 {
                shadow_ref.set_color(MathColor::new(r, g, b, a));
                return;
            }
        }
        let shadow = ShadowBuilder::new()
            .color(MathColor::new(r, g, b, a))
            .build(&mut world.component_mgr.node.element.text.text_style.shadow);
        let mut style_ref = TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr);
        style_ref.set_shadow(shadow);
    }
    debug_println!("set_text_shadow_color"); 
}

#[no_mangle]
pub fn set_text_shadow(world: u32, node_id: u32, h: f32, v: f32, r: f32, g: f32, b: f32, a: f32, blur: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let style_id = world.component_mgr.node.element.text._group.get(text_id).text_style;
    let shadow = Shadow{
        blur, v, h,
        color: MathColor::new(r, g, b, a),
    };
    if style_id == 0 {
        let style = TextStyleBuilder::new()
        .shadow(shadow)
        .build(&mut world.component_mgr.node.element.text.text_style);
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_text_style(style);
    } else {
        let mut style_ref = TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr);
        style_ref.set_shadow(shadow);
    }
    debug_println!("set_text_shadow_color"); 
}

#[no_mangle]
pub fn set_font_style(world: u32, node_id: u32, value: u8){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let font_id = world.component_mgr.node.element.text._group.get(text_id).font;
    if font_id == 0 {
        let mut font = Font::default();
        font.style = unsafe {transmute(value)};
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_font(font);
    }else{
        FontWriteRef::new(font_id, world.component_mgr.node.element.text.font.to_usize(), &mut world.component_mgr).set_style(unsafe {transmute(value)});
    }
    debug_println!("set_font_style"); 
}

#[no_mangle]
pub fn set_font_weight(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let font_id = world.component_mgr.node.element.text._group.get(text_id).font;
    if font_id == 0 {
        let mut font = Font::default();
        font.weight = value;
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_font(font);
    }else{
        FontWriteRef::new(font_id, world.component_mgr.node.element.text.font.to_usize(), &mut world.component_mgr).set_weight(value);
    }
    debug_println!("set_font_weight"); 
}

#[no_mangle]
pub fn set_font_size_none(world: u32, node_id: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let font_id = world.component_mgr.node.element.text._group.get(text_id).font;
    if font_id == 0 {
        let font = Font::default();
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_font(font);
    }else{
        FontWriteRef::new(font_id, world.component_mgr.node.element.text.font.to_usize(), &mut world.component_mgr).set_size(FontSize::None);
    }
    debug_println!("set_font_weight"); 
}

#[no_mangle]
pub fn set_font_size(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let font_id = world.component_mgr.node.element.text._group.get(text_id).font;
    if font_id == 0 {
        let mut font = Font::default();
        font.size = FontSize::Length(value);
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_font(font);
    }else{
        FontWriteRef::new(font_id, world.component_mgr.node.element.text.font.to_usize(), &mut world.component_mgr).set_size(FontSize::Length(value));
    }
    debug_println!("set_font_weight"); 
}

#[no_mangle]
pub fn set_font_size_percent(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let font_id = world.component_mgr.node.element.text._group.get(text_id).font;
    if font_id == 0 {
        let mut font = Font::default();
        font.size = FontSize::Percent(value);
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_font(font);
    }else{
        FontWriteRef::new(font_id, world.component_mgr.node.element.text.font.to_usize(), &mut world.component_mgr).set_size(FontSize::Percent(value));
    }
    debug_println!("set_font_weight"); 
}

#[no_mangle]
pub fn set_font_family(world: u32, node_id: u32){
    let value: String = js!(return __jsObj;).try_into().unwrap();
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let text_id = get_text_id(node_id, world);
    let font_id = world.component_mgr.node.element.text._group.get(text_id).font;
    if font_id == 0 {
        let mut font = Font::default();
        font.family = Atom::from(value);
        TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_font(font);
    }else{
        FontWriteRef::new(font_id, world.component_mgr.node.element.text.font.to_usize(), &mut world.component_mgr).set_family(Atom::from(value));
    }
    debug_println!("set_font_family"); 
}

fn get_text_id(node_id: usize, world: &mut World<WorldDocMgr, ()>) -> usize {
    match world.component_mgr.node._group.get(node_id).element {
        ElementId::Text(id) => id,
        _ => panic!("it's not a text"),
    }
}