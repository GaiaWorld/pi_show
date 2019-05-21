use std::mem::transmute;

// use stdweb::unstable::TryInto;
// use stdweb::web::TypedArray;


use atom::Atom;

use ecs::World;
use ecs::idtree::IdTree;

use component::user::*;
use component::*;
use font::font_sheet::{ get_line_height, LineHeight, FontSheet, FontSize};
use Node;
pub use layout::{YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType};

#[macro_use()]
macro_rules! set_attr {
    ($world:ident, $node_id:ident, $tt:ident, $name:ident, $value:ident) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut World)};
        get_text(world, node_id);
        let attr = world.fetch_multi::<Node, $tt>().unwrap();
        let mut attr = attr.borrow_mut();
        $crate::paste::item! {
            match attr.get_write(node_id) {
                Some(mut r) => r.[<set_ $name>]($value),
                _ =>{
                    let mut v = $tt::default();
                    v.$name = $value;
                    attr.insert(node_id, v);
                }
            }
        }
        debug_println!("set_{}", $name);
    };
}

#[no_mangle]
pub fn set_letter_spacing(world: u32, node_id: u32, value: f32){
    let letter_spacing = 0;
    set_attr!(world, node_id, TextStyle, letter_spacing, value);
}

#[no_mangle]
pub fn set_text_rgba_color(world: u32, node_id: u32, r: f32, g: f32, b: f32, a: f32){
    let value = Color::RGBA(CgColor::new(r, g, b, a));
    let color = 0;
    set_attr!(world, node_id, TextStyle, color, value);
}

#[no_mangle]
pub fn set_text_linear_gradient_color(world: u32, node_id: u32, direction: f32){
    // debug_println!("set_text_linear_gradient_color");
    //  let color_and_positions: TypedArray<f32> = js!(return __jsObj;).try_into().unwrap();
    // let node_id = node_id as usize;
    // let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    // let text_id = get_text_id(node_id, world);
    // let style_id = world.component_mgr.node.element.text._group.get(text_id).text_style;
    // if style_id == 0 {
    //     let mut style = TextStyle::default();
    //     style.color = Color::LinearGradient(to_linear_gradient_color(color_and_positions, direction));
    //     TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_text_style(style);
    // } else {
    //     TextStyleWriteRef::new(style_id, world.component_mgr.node.element.text.text_style.to_usize(), &mut world.component_mgr).set_color(Color::LinearGradient(to_linear_gradient_color(color_and_positions, direction)));
    // }
}


#[no_mangle]
pub fn set_line_height_normal(world: u32, node_id: u32){
    let value = LineHeight::Normal;
    let line_height = 0;
    set_attr!(world, node_id, TextStyle, line_height, value);
}

#[no_mangle]
pub fn set_line_height(world: u32, node_id: u32, value: f32){
    let value = LineHeight::Length(value);
    let line_height = 0;
    set_attr!(world, node_id, TextStyle, line_height, value);
}

#[no_mangle]
pub fn set_line_height_percent(world: u32, node_id: u32, value: f32){
    let value = LineHeight::Percent(value);
    let line_height = 0;
    set_attr!(world, node_id, TextStyle, line_height, value);
}

#[no_mangle]
pub fn set_text_indent(world: u32, node_id: u32, value: f32){
    let indent = 0;
    set_attr!(world, node_id, TextStyle, indent, value);
}

#[no_mangle]
pub fn set_text_stroke(world: u32, node_id: u32, width: f32, r: f32, g: f32, b: f32, a: f32){
    let value = Stroke {
        width,
        color: CgColor::new(r, g, b, a),
    };
    let stroke = 0;
    set_attr!(world, node_id, TextStyle, stroke, value);
}

#[no_mangle]
pub fn set_white_space(world: u32, node_id: u32, value: u8){
    let value = unsafe {transmute(value)};
    let white_space = 0;
    set_attr!(world, node_id, TextStyle, white_space, value);
}

#[no_mangle]
pub fn set_text_shadow_h(world: u32, node_id: u32, value: f32){
    let h = 0;
    set_attr!(world, node_id, TextShadow, h, value);
}

#[no_mangle]
pub fn set_text_shadow_v(world: u32, node_id: u32, value: f32){
    let v = 0;
    set_attr!(world, node_id, TextShadow, v, value);
 
}

#[no_mangle]
pub fn set_text_shadow_blur(world: u32, node_id: u32, value: f32){
    let blur = 0;
    set_attr!(world, node_id, TextShadow, blur, value);
}

#[no_mangle]
pub fn set_text_shadow_color(world: u32, node_id: u32, r: f32, g: f32, b: f32, a: f32){
    let value = CgColor::new(r, g, b, a);
    let color = 0;
    set_attr!(world, node_id, TextShadow, color, value);
}

#[no_mangle]
pub fn set_text_shadow(world: u32, node_id: u32, h: f32, v: f32, r: f32, g: f32, b: f32, a: f32, blur: f32){
    let value = TextShadow {
        h: h,
        v: v,
        blur: blur,
        color: CgColor::new(r, g, b, a),
    };
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World)};
    get_text(world, node_id);
    let attr = world.fetch_multi::<Node, TextShadow>().unwrap();
    attr.borrow_mut().insert(node_id, value);
    debug_println!("set_text_shadow"); 
}

#[no_mangle]
pub fn set_font_style(world: u32, node_id: u32, value: u8){
    let value = unsafe {transmute(value)};
    let style = 0;
    set_attr!(world, node_id, Font, style, value);
}

#[no_mangle]
pub fn set_font_weight(world: u32, node_id: u32, value: f32){
    let weight = 0;
    set_attr!(world, node_id, Font, weight, value);
}

#[no_mangle]
pub fn set_font_size_none(world: u32, node_id: u32){
    let value = FontSize::None;
    let size = 0;
    set_attr!(world, node_id, Font, size, value);
}

#[no_mangle]
pub fn set_font_size(world: u32, node_id: u32, value: f32){
    let value = FontSize::Length(value);
    let size = 0;
    set_attr!(world, node_id, Font, size, value);
}

#[no_mangle]
pub fn set_font_size_percent(world: u32, node_id: u32, value: f32){
    let value = FontSize::Percent(value);
    let size = 0;
    set_attr!(world, node_id, Font, size, value);
}

// #[no_mangle]
// pub fn set_font_family(world: u32, node_id: u32){
//     let value: String = js!(return __jsObj;).try_into().unwrap();
//     let node_id = node_id as usize;
//     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
//     let text_id = get_text_id(node_id, world);
//     let font_id = world.component_mgr.node.element.text._group.get(text_id).font;
//     if font_id == 0 {
//         let mut font = Font::default();
//         font.family = Atom::from(value);
//         TextWriteRef::new(text_id, world.component_mgr.node.element.text.to_usize(), &mut world.component_mgr).set_font(font);
//     }else{
//         FontWriteRef::new(font_id, world.component_mgr.node.element.text.font.to_usize(), &mut world.component_mgr).set_family(Atom::from(value));
//     }
//     debug_println!("set_font_family"); 
// }

fn get_text(world: &World, node_id: usize) -> &mut Text {
    let text = world.fetch_multi::<Node, Text>().unwrap();
    let mut text = text.borrow_mut();
    match text.get_mut(node_id) {
        Some(r) => unsafe{&mut *(r as *mut Text)},
        _ => panic!("it's not a text"),
    }
}
