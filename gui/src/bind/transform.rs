use std::mem::transmute;


use ecs::{World, LendMut};

use component::user::*;
use entity::Node;


#[macro_use()]
macro_rules! push_func {
    ($world:ident, $node_id:ident, $value:expr) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut World)};
        let attr = world.fetch_multi::<Node, Transform>().unwrap();
        let attr = attr.lend_mut();
        match attr.get_write(node_id) {
            Some(mut r) => r.modify(|transform: &mut Transform| {
                transform.funcs.push($value);
                true
            }),
            None => {
                let mut transform = Transform::default();
                transform.funcs.push($value);
                attr.insert(node_id, transform);
            }
        }
    };
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn clear_transform(world: u32, node_id: u32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World)};
    let attr = world.fetch_multi::<Node, Transform>().unwrap();
    unsafe {attr.lend_mut().get_unchecked_write(node_id)}.modify(|transform: &mut Transform| {
        if transform.funcs.len() > 0 {
            transform.funcs.clear();
            true
        }else {
            false
        }
        
    });
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn transform_translate(world: u32, node_id: u32, x: f32, y: f32) {
    push_func!(world, node_id, TransformFunc::Translate(x, y));
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn transform_translate_x(world: u32, node_id: u32, value: f32) {
    push_func!(world, node_id, TransformFunc::TranslateX(value));
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn transform_translate_y(world: u32, node_id: u32, value: f32) {
    push_func!(world, node_id, TransformFunc::TranslateY(value));
}


#[allow(unused_attributes)]
#[no_mangle]
pub fn transform_translate_percent(world: u32, node_id: u32, x: f32, y: f32) {
    push_func!(world, node_id, TransformFunc::TranslatePercent(x/100.0, y/100.0));
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn transform_translate_x_percent(world: u32, node_id: u32, value: f32) {
    push_func!(world, node_id, TransformFunc::TranslateXPercent(value/100.0));

}

#[allow(unused_attributes)]
#[no_mangle]
pub fn transform_translate_y_percent(world: u32, node_id: u32, value: f32) {
    push_func!(world, node_id, TransformFunc::TranslateYPercent(value/100.0));
}


#[allow(unused_attributes)]
#[no_mangle]
pub fn transform_scale(world: u32, node_id: u32, x: f32, y: f32) {
    push_func!(world, node_id, TransformFunc::Scale(x, y));
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn transform_scale_x(world: u32, node_id: u32, value: f32) {
    push_func!(world, node_id, TransformFunc::ScaleX(value));
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn transform_scale_y(world: u32, node_id: u32, value: f32) {
    push_func!(world, node_id, TransformFunc::ScaleY(value));
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn transform_rotate(world: u32, node_id: u32, value: f32) {
    push_func!(world, node_id, TransformFunc::RotateZ(value));
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn transform_none(world: u32, node_id: u32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World)};
    let attr = world.fetch_multi::<Node, Transform>().unwrap();
    unsafe {attr.lend_mut().get_unchecked_write(node_id)}.modify(|transform: &mut Transform| {
        if transform.funcs.len() > 0 {
            transform.funcs.clear();
            true
        }else{
            false
        }
    });
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn transform_origin(world: u32, node_id: u32, x_ty: u8, x: f32, y_ty: u8, y: f32) {
    let x_ty = unsafe{transmute(x_ty)};
    let y_ty = unsafe {transmute(y_ty)};
    let x_value = match x_ty {
        LengthUnitType::Pixel => LengthUnit::Pixel(x),
        LengthUnitType::Percent => LengthUnit::Percent(x/100.0),
    };
    let y_value = match y_ty {
        LengthUnitType::Pixel => LengthUnit::Pixel(y),
        LengthUnitType::Percent => LengthUnit::Percent(y/100.0),
    };
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World)};
    let attr = world.fetch_multi::<Node, Transform>().unwrap();
    unsafe {attr.lend_mut().get_unchecked_write(node_id)}.modify(|transform: &mut Transform| {
        transform.origin = TransformOrigin::XY(x_value, y_value);
        true
    });
}
