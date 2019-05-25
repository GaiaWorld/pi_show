use std::mem::transmute;

use stdweb::web::TypedArray;
use stdweb::unstable::TryInto;

use ecs::{World, LendMut};

use component::user::*;
use entity::Node;

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
#[macro_use()]
macro_rules! insert_value {
    ($world:ident, $node_id:ident, $tt:ident, $value:expr) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut World)};
        let attr = world.fetch_multi::<Node, $tt>().unwrap();
        attr.lend_mut().insert(node_id, $tt($value));
    };
}
#[macro_use()]
macro_rules! insert_attr {
    ($world:ident, $node_id:ident, $tt:ident, $value:expr) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut World)};
        let attr = world.fetch_multi::<Node, $tt>().unwrap();
        attr.lend_mut().insert(node_id, $value);
    };
}
#[macro_use()]
macro_rules! set_show {
    ($world:ident, $node_id:ident, $name:ident, $value:expr) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut World)};
        let attr = world.fetch_multi::<Node, Show>().unwrap();
        unsafe {attr.lend_mut().get_unchecked_write(node_id)}.modify(|s| {
            let old = s.clone();
            s.$name($value);
            old == *s
        });
    };
}

#[no_mangle]
pub fn set_background_rgba_color(world: u32, node: u32, r: f32, g: f32, b: f32, a: f32){
    let background = 0;
    set_attr!(world, node, BoxColor, background, Color::RGBA(CgColor::new(r, g, b, a)));
}

// // 设置一个径向渐变的背景颜色
// #[no_mangle]
// pub fn set_background_radial_gradient_color(world: u32, node: u32, center_x: f32, center_y: f32, shape: u8, size: u8 ){
//     let color_and_positions: TypedArray<f32> = js!(return __jsObj;).try_into().unwrap();
//     let value = Color::RadialGradient(to_radial_gradient_color(color_and_positions, center_x, center_y, shape, size));
//     insert_value!(world, node, BackgroundColor, value);
// }

// 设置一个线性渐变的背景颜色
#[no_mangle]
pub fn set_background_linear_gradient_color(world: u32, node: u32, direction: f32){
    let color_and_positions: TypedArray<f32> = js!(return __jsObj;).try_into().unwrap();
    let value = Color::LinearGradient(to_linear_gradient_color(color_and_positions.to_vec(), direction));
    insert_value!(world, node, BackgroundColor, value);
}

// 设置边框颜色， 类型为rgba
#[no_mangle]
pub fn set_border_color(world: u32, node: u32, r: f32, g: f32, b: f32, a: f32){
    insert_value!(world, node, BorderColor, CgColor::new(r, g, b, a));
}

// 设置边框圆角
#[no_mangle]
pub fn set_border_radius(world: u32, node: u32, x: f32, y: f32){ 
    insert_attr!(world, node, BorderRadius, BorderRadius{x: LengthUnit::Pixel(x), y: LengthUnit::Pixel(y)});
}

// 设置边框圆角
#[no_mangle]
pub fn set_border_radius_percent(world: u32, node: u32, x: f32, y: f32){
    insert_attr!(world, node, BorderRadius, BorderRadius{x: LengthUnit::Percent(x), y: LengthUnit::Percent(y)});
}

// 设置阴影颜色
#[no_mangle]
pub fn set_box_shadow_color(world: u32, node: u32, r: f32, g: f32, b: f32, a: f32){
    let color = 0;
    set_attr!(world, node, BoxShadow, color, CgColor::new(r, g, b, a));
}

#[no_mangle]
pub fn set_box_shadow_blur(world: u32, node: u32, value: f32){
    let blur = 0;
    set_attr!(world, node, BoxShadow, blur, value);
}

// 设置阴影h
#[no_mangle]
pub fn set_box_shadow_h(world: u32, node: u32, value: f32){
    let h = 0;
    set_attr!(world, node, BoxShadow, h, value);
}

// 设置阴影v
#[no_mangle]
pub fn set_box_shadow_v(world: u32, node: u32, value: f32){
    let v = 0;
    set_attr!(world, node, BoxShadow, v, value);
}
//设置object_fit
#[no_mangle]
pub fn set_object_fit(world: u32, node: u32, value: u8){
    insert_value!(world, node, ObjectFit, unsafe{ transmute(value)});
}
// 设置图像裁剪
#[no_mangle]
pub fn set_image_clip(world: u32, node: u32, u1: f32, v1: f32, u2: f32, v2: f32){
    insert_value!(world, node, ImageClip, Aabb2{min: Point2::new(u1, v1), max: Point2::new(u2, v2)});
}
// 设置图像裁剪
#[no_mangle]
pub fn set_border_image_clip(world: u32, node: u32, u1: f32, v1: f32, u2: f32, v2: f32){
    insert_value!(world, node, BorderImageClip, Aabb2{min: Point2::new(u1, v1), max: Point2::new(u2, v2)});
}
//设置border_image_slice
#[no_mangle]
pub fn set_border_image_slice(world: u32, node: u32, top: f32, right: f32, bottom: f32, left: f32, fill: bool){
    insert_attr!(world, node, BorderImageSlice, BorderImageSlice{top, right, bottom, left, fill});
}
//设置border_image_slice
#[no_mangle]
pub fn set_border_image_repeat(world: u32, node: u32, vertical: u8, horizontal: u8){
    insert_attr!(world, node, BorderImageRepeat, BorderImageRepeat(unsafe{ transmute(vertical)}, unsafe{ transmute(horizontal)}));
}

//设置overflow
#[no_mangle]
pub fn set_overflow(world: u32, node: u32, value: bool){
    insert_value!(world, node, Overflow, value);
}

//设置不透明度
#[no_mangle]
pub fn set_opacity(world: u32, node: u32, value: f32) {
    insert_value!(world, node, Opacity, value);
}

//设置display
#[no_mangle]
pub fn set_display(world: u32, node: u32, value: u8){
    let value = unsafe{ transmute(value)};
    set_show!(world, node, set_display, value);
}

//设置visibility, true: visible, false: hidden,	默认true
#[no_mangle]
pub fn set_visibility(world: u32, node: u32, value: bool) {
    set_show!(world, node, set_visibility, value);
}

//设置visibility, true: visible, false: hidden,	默认true
#[no_mangle]
pub fn set_enable(world: u32, node: u32, value: bool) {
    set_show!(world, node, set_enable, value);
}

#[no_mangle]
pub fn set_zindex(world: u32, node: u32, value: i32) {
    let value = value as isize;
    insert_value!(world, node, ZIndex, value);
}