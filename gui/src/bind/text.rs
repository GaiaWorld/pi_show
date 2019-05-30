use std::mem::transmute;

use stdweb::unstable::TryInto;
use stdweb::web::TypedArray;


use atom::Atom;

use ecs::{World, LendMut};

use component::user::*;
use entity::Node;
pub use layout::{YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType};

#[macro_use()]
macro_rules! set_attr {
    ($world:ident, $node_id:ident, $tt:ident, $name:ident, $value:expr) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut World)};
        let attr = world.fetch_multi::<Node, $tt>().unwrap();
        let attr = attr.lend_mut();
        let value = $value;
        $crate::paste::item! {
            match attr.get_write(node_id) {
                Some(mut r) => r.[<set_ $name>](value),
                _ =>{
                    let mut v = $tt::default();
                    v.$name = value;
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
    let color = 0;
    set_attr!(world, node_id, TextStyle, color, Color::RGBA(CgColor::new(r, g, b, a)));
}

// __jsObj: color_and_positions: [r, g, b, a, pos,   r, g, b, a, pos], direction: 0-360度
#[no_mangle]
pub fn set_text_linear_gradient_color(world: u32, node_id: u32, direction: f32){
    let color_and_positions: TypedArray<f32> = js!(return __jsObj;).try_into().unwrap();
    let value = Color::LinearGradient(to_linear_gradient_color(color_and_positions.to_vec(), direction));
    let color = 0;
    set_attr!(world, node_id, TextStyle, color, value);
}


#[no_mangle]
pub fn set_line_height_normal(world: u32, node_id: u32){
    let line_height = 0;
    set_attr!(world, node_id, TextStyle, line_height, LineHeight::Normal);
}

#[no_mangle]
pub fn set_line_height(world: u32, node_id: u32, value: f32){
    let line_height = 0;
    set_attr!(world, node_id, TextStyle, line_height, LineHeight::Length(value));
}

#[no_mangle]
pub fn set_line_height_percent(world: u32, node_id: u32, value: f32){
    let line_height = 0;
    set_attr!(world, node_id, TextStyle, line_height, LineHeight::Percent(value));
}

#[no_mangle]
pub fn set_text_indent(world: u32, node_id: u32, value: f32){
    let indent = 0;
    set_attr!(world, node_id, TextStyle, indent, value);
}

#[no_mangle]
pub fn set_text_stroke(world: u32, node_id: u32, width: f32, r: f32, g: f32, b: f32, a: f32){
    let stroke = 0;
    set_attr!(world, node_id, TextStyle, stroke, Stroke {
        width,
        color: CgColor::new(r, g, b, a),
    });
}

#[no_mangle]
pub fn set_white_space(world: u32, node_id: u32, value: u8){
    let white_space = 0;
    set_attr!(world, node_id, TextStyle, white_space, unsafe {transmute(value)});
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
    set_attr!(world, node_id, TextShadow, color, CgColor::new(r, g, b, a));
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
    let attr = world.fetch_multi::<Node, TextShadow>().unwrap();
    attr.lend_mut().insert(node_id, value);
    debug_println!("set_text_shadow"); 
}

#[no_mangle]
pub fn set_font_style(world: u32, node_id: u32, value: u8){
    let style = 0;
    set_attr!(world, node_id, Font, style, unsafe {transmute(value)});
}

#[no_mangle]
pub fn set_font_weight(world: u32, node_id: u32, value: f32){
    let weight = 0;
    set_attr!(world, node_id, Font, weight, value);
}

#[no_mangle]
pub fn set_font_size_none(world: u32, node_id: u32){
    let size = 0;
    set_attr!(world, node_id, Font, size, FontSize::None);
}

#[no_mangle]
pub fn set_font_size(world: u32, node_id: u32, value: f32){
    let size = 0;
    set_attr!(world, node_id, Font, size, FontSize::Length(value));
}

#[no_mangle]
pub fn set_font_size_percent(world: u32, node_id: u32, value: f32){
    let size = 0;
    set_attr!(world, node_id, Font, size, FontSize::Percent(value));
}

// __jsObj: family name
#[no_mangle]
pub fn set_font_family(world: u32, node_id: u32){
    let value: String = js!(return __jsObj;).try_into().unwrap();
    let family = 0;
    set_attr!(world, node_id, Font, family, Atom::from(value));
    debug_println!("set_font_family"); 
}
