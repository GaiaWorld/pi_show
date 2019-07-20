use std::mem::transmute;

use stdweb::web::TypedArray;
use stdweb::unstable::TryInto;

use ecs::{LendMut};

use gui::component::user::*;
use GuiWorld;

#[macro_use()]
macro_rules! set_attr {
    ($world:ident, $node_id:ident, $tt:ident, $name:ident, $value:expr, $key:ident) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut GuiWorld)};
		let world = &mut world.gui;
        let attr = world.$key.lend_mut();
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
    ($world:ident, $node_id:ident, $tt:ident, $value:expr, $key:ident) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut GuiWorld)};
		let world = &mut world.gui;
        world.$key.lend_mut().insert(node_id, $tt($value));
    };
}
#[macro_use()]
macro_rules! insert_attr {
    ($world:ident, $node_id:ident, $tt:ident, $value:expr, $key:ident) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut GuiWorld)};
		let world = &mut world.gui;
        world.$key.lend_mut().insert(node_id, $value);
    };
}
#[macro_use()]
macro_rules! set_show {
    ($world:ident, $node_id:ident, $name:ident, $value:expr) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut GuiWorld)};
		let world = &mut world.gui;
        unsafe {world.show.lend_mut().get_unchecked_write(node_id)}.modify(|s| {
            let old = s.clone();
            s.$name($value);
            old != *s
        });
    };
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_background_rgba_color(world: u32, node: u32, r: f32, g: f32, b: f32, a: f32){
    insert_value!(world, node, BackgroundColor, Color::RGBA(CgColor::new(r, g, b, a)), background_color);
}

// // 设置一个径向渐变的背景颜色
// #[allow(unused_attributes)]
#[no_mangle]
// pub fn set_background_radial_gradient_color(world: u32, node: u32, center_x: f32, center_y: f32, shape: u8, size: u8 ){
//     let color_and_positions: TypedArray<f32> = js!(return __jsObj;).try_into().unwrap();
//     let value = Color::RadialGradient(to_radial_gradient_color(color_and_positions, center_x, center_y, shape, size));
//     insert_value!(world, node, BackgroundColor, value);
// }

// 设置一个线性渐变的背景颜色
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_background_linear_gradient_color(world: u32, node: u32, direction: f32){
    let color_and_positions: TypedArray<f32> = js!(return __jsObj;).try_into().unwrap();
    let value = Color::LinearGradient(to_linear_gradient_color(color_and_positions.to_vec(), direction));
    insert_value!(world, node, BackgroundColor, value, background_color);
}

// 设置边框颜色， 类型为rgba
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_border_color(world: u32, node: u32, r: f32, g: f32, b: f32, a: f32){
    insert_value!(world, node, BorderColor, CgColor::new(r, g, b, a), border_color);
}

// 设置边框圆角
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_border_radius(world: u32, node: u32, x: f32, y: f32){ 
    insert_attr!(world, node, BorderRadius, BorderRadius{x: LengthUnit::Pixel(x), y: LengthUnit::Pixel(y)}, border_radius);
}

// 设置边框圆角
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_border_radius_percent(world: u32, node: u32, x: f32, y: f32){
    insert_attr!(world, node, BorderRadius, BorderRadius{x: LengthUnit::Percent(x), y: LengthUnit::Percent(y)}, border_radius);
}

// 设置阴影颜色
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_box_shadow_color(world: u32, node: u32, r: f32, g: f32, b: f32, a: f32){
    let color = 0;
    set_attr!(world, node, BoxShadow, color, CgColor::new(r, g, b, a), box_shadow);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_box_shadow_blur(world: u32, node: u32, value: f32){
    let blur = 0;
    set_attr!(world, node, BoxShadow, blur, value, box_shadow);
}

// 设置阴影h
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_box_shadow_h(world: u32, node: u32, value: f32){
    let h = 0;
    set_attr!(world, node, BoxShadow, h, value, box_shadow);
}

// 设置阴影v
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_box_shadow_v(world: u32, node: u32, value: f32){
    let v = 0;
    set_attr!(world, node, BoxShadow, v, value, box_shadow);
}
//设置object_fit
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_object_fit(world: u32, node: u32, value: u8){
    insert_value!(world, node, ObjectFit, unsafe{ transmute(value)}, object_fit);
}
// 设置图像裁剪
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_image_clip(world: u32, node: u32, u1: f32, v1: f32, u2: f32, v2: f32){
    insert_value!(world, node, ImageClip, Aabb2{min: Point2::new(u1, v1), max: Point2::new(u2, v2)}, image_clip);
}
// 设置图像裁剪
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_border_image_clip(world: u32, node: u32, u1: f32, v1: f32, u2: f32, v2: f32){
    insert_value!(world, node, BorderImageClip, Aabb2{min: Point2::new(u1, v1), max: Point2::new(u2, v2)}, border_image_clip);
}
//设置border_image_slice
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_border_image_slice(world: u32, node: u32, top: f32, right: f32, bottom: f32, left: f32, fill: bool){
    insert_attr!(world, node, BorderImageSlice, BorderImageSlice{top, right, bottom, left, fill}, border_image_slice);
}
//设置border_image_slice
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_border_image_repeat(world: u32, node: u32, vertical: u8, horizontal: u8){
    insert_attr!(world, node, BorderImageRepeat, BorderImageRepeat(unsafe{ transmute(vertical)}, unsafe{ transmute(horizontal)}), border_image_repeat);
}

//设置overflow
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_overflow(world: u32, node: u32, value: bool){
    insert_value!(world, node, Overflow, value, overflow);
}

//设置不透明度
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_opacity(world: u32, node: u32, mut value: f32) {
    if value > 1.0 {
        value = 1.0;
    } else if value < 0.0{
        value = 0.0;
    }
    insert_value!(world, node, Opacity, value, opacity);
}

//设置display
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_display(world: u32, node: u32, value: u8){
    let value = unsafe{ transmute(value)};
    set_show!(world, node, set_display, value);
}

//设置visibility, true: visible, false: hidden,	默认true
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_visibility(world: u32, node: u32, value: bool) {
    set_show!(world, node, set_visibility, value);
}

//enable
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_enable(world: u32, node: u32, value: u32) {
    set_show!(world, node, set_enable, unsafe{transmute(value as u8)});
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_zindex(world: u32, node: u32, value: i32) {
    let value = value as isize;
    insert_value!(world, node, ZIndex, value, z_index);
}

// //将图像转换为灰度图像。值定义转换的比例, 值为100%则完全转为灰度图像，值为0%图像无变化。值在0%到100%之间，则是效果的线性乘子。若未设置，值默认是0；
// #[allow(unused_attributes)]
// #[no_mangle]
// pub fn set_filter_grayscale(world: u32, node: u32, mut value: f32) {
//     if value < 0.0 {
//         value = 0.0;
//     } else if value > 100.0 {
//         value = 1.0;
//     }
//     value = 100.0 - value;
//     let saturate = 0;
//     set_attr!(world, node, Filter, saturate, value/100.0, filter);
// }

// //给图像应用色相旋转。"angle"一值设定图像会被调整的色环角度值。值为0deg，则图像无变化。若值未设置，默认值是0deg。该值虽然没有最大值，超过360deg的值相当于又绕一圈。
// #[allow(unused_attributes)]
// #[no_mangle]
// pub fn set_filter_hue_rotate(world: u32, node: u32, value: f32) {
//     let hue_rotate = 0;
//     set_attr!(world, node, Filter, hue_rotate, value, filter);
// }

// //给图片应用一种线性乘法，使其看起来更亮或更暗。如果值是0%，图像会全黑。值是100%，则图像无变化。其他的值对应线性乘数效果。值超过100%也是可以的，图像会比原来更亮。如果没有设定值，默认是1。
// #[allow(unused_attributes)]
// #[no_mangle]
// pub fn set_filter_bright_ness(world: u32, node: u32, value: f32) {
//     let bright_ness = 0;
//     set_attr!(world, node, Filter, bright_ness, value/100.0, filter);
// }

// #[allow(unused_attributes)]
// #[no_mangle]
// pub fn set_filter_saturate(world: u32, node: u32, mut value: f32) {
//     if value < 0.0 {
//         value = 0.0;
//     }
//     let saturate = 0;
//     set_attr!(world, node, Filter, saturate, value/100.0, filter);
// }

// hsi, 效果与ps一致,  h: -180 ~ 180, s: -100 ~ 100, i: -100 ~ 100
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_filter_hsi(world: u32, node: u32, mut h: f32, mut s: f32, mut i: f32) {
    if h > 180.0 {
        h = 180.0;
    }else if h < -180.0 {
        h = -180.0  
    }
    if s > 100.0 {
        s = 100.0;
    }else if s < -100.0 {
        s = -100.0  
    }
    if i > 100.0 {
        i = 100.0;
    }else if i < -100.0 {
        i = -100.0  
    }
    let value = Filter{hue_rotate: h/360.0, saturate: s/100.0, bright_ness: i/100.0};
    insert_attr!(world, node, Filter, value, filter);
}