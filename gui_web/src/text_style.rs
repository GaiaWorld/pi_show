use std::mem::transmute;

use stdweb::unstable::TryInto;
use stdweb::web::{ TypedArray };

use atom::Atom;
use ecs::{LendMut};

use gui::component::user::*;
use gui::single::{TextStyleClassMap};
pub use gui::layout::{YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType};
use GuiWorld;


/**
 * 在指定上下文中创建一个 文本样式表
 */
pub fn create_text_style_class(world: u32, class_id: u32) -> bool {
    let class_id = class_id as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
    let map = world.gui.world.fetch_single::<TextStyleClassMap>().unwrap();
    let map = map.lend_mut();
    match map.0.entry(class_id) {
        std::collections::hash_map::Entry::Occupied(_) => false,
        std::collections::hash_map::Entry::Vacant(e) => {
            e.insert(TextStyleClazz::default());
            true
        }
    }
}


#[macro_use()]
macro_rules! set_attr {
    ($world:ident, $class_id:ident, $tt:ident, $name:ident, $value:expr, $key: ident) => {
        let class_id = $class_id as usize;
        let world = unsafe {&mut *($world as usize as *mut GuiWorld)};
        let map = world.gui.world.fetch_single::<TextStyleClassMap>().unwrap();
        let map = map.lend_mut();
        let value = $value;
        $crate::paste::item! {
            match map.0.get_mut(&class_id) {
                Some(mut r) => r.$key.$name = value,
                _ => ()
            }
        }
        debug_println!("set_{}", $name);
    };
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_letter_spacing(world: u32, class_id: u32, value: f32){
    let letter_spacing = 0;
    set_attr!(world, class_id, TextStyle, letter_spacing, value, style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_text_rgba_color(world: u32, class_id: u32, r: f32, g: f32, b: f32, a: f32){
    let color = 0;
    set_attr!(world, class_id, TextStyle, color, Color::RGBA(CgColor::new(r, g, b, a)), style);
}

// // __jsObj: color_and_positions: [r, g, b, a, pos,   r, g, b, a, pos], direction: 0-360度
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_text_linear_gradient_color(world: u32, class_id: u32, direction: f32){
    let color_and_positions: TypedArray<f32> = js!(return __jsObj;).try_into().unwrap();
    let value = Color::LinearGradient(to_linear_gradient_color(color_and_positions.to_vec(), direction));
    let color = 0;
    set_attr!(world, class_id, TextStyle, color, value, style);
}


#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_line_height_normal(world: u32, class_id: u32){
    let line_height = 0;
    set_attr!(world, class_id, TextStyle, line_height, LineHeight::Normal, style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_line_height(world: u32, class_id: u32, value: f32){
    let line_height = 0;
    set_attr!(world, class_id, TextStyle, line_height, LineHeight::Length(value), style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_line_height_percent(world: u32, class_id: u32, value: f32){
    let line_height = 0;
    set_attr!(world, class_id, TextStyle, line_height, LineHeight::Percent(value), style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_text_indent(world: u32, class_id: u32, value: f32){
    let indent = 0;
    set_attr!(world, class_id, TextStyle, indent, value, style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_text_stroke(world: u32, class_id: u32, width: f32, r: f32, g: f32, b: f32, a: f32){
    let stroke = 0;
    set_attr!(world, class_id, TextStyle, stroke, Stroke {
        width,
        color: CgColor::new(r, g, b, a),
    }, style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_white_space(world: u32, class_id: u32, value: u8){
    let white_space = 0;
    set_attr!(world, class_id, TextStyle, white_space, unsafe {transmute(value)}, style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_text_shadow(world: u32, class_id: u32, h: f32, v: f32, r: f32, g: f32, b: f32, a: f32, blur: f32){
    let value = TextShadow {
        h: h,
        v: v,
        blur: blur,
        color: CgColor::new(r, g, b, a),
    };
    let class_id = class_id as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
    let map = world.gui.world.fetch_single::<TextStyleClassMap>().unwrap();
    let map = map.lend_mut();
    match map.0.get_mut(&class_id) {
        Some(mut r) => r.shadow = value,
        _ => ()
    }
    debug_println!("set_text_shadow"); 
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_font_style(world: u32, class_id: u32, value: u8){
    let style = 0;
    set_attr!(world, class_id, Font, style, unsafe {transmute(value)}, font);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_font_weight(world: u32, class_id: u32, value: f32){
    let weight = 0;
    set_attr!(world, class_id, Font, weight, value, font);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_font_size_none(world: u32, class_id: u32){
    let size = 0;
    set_attr!(world, class_id, Font, size, FontSize::None, font);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_font_size(world: u32, class_id: u32, value: f32){
    let size = 0;
    set_attr!(world, class_id, Font, size, FontSize::Length(value), font);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_font_size_percent(world: u32, class_id: u32, value: f32){
    let size = 0;
    set_attr!(world, class_id, Font, size, FontSize::Percent(value), font);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_style_class_font_family(world: u32, class_id: u32){
    let value: String = js!(return __jsObj;).try_into().unwrap();
    let family = 0;
    set_attr!(world, class_id, Font, family, Atom::from(value), font);
}
