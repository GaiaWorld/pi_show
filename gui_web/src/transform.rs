/// 将设置几何变换属性的接口导出到js

use wasm_bindgen::prelude::*;
use gui::component::calc::*;

// use ecs::LendMut;

use gui::component::user::*;
use gui::single::to_entity;
use crate::world::GuiWorld;

#[macro_use()]
macro_rules! push_func {
    ($world:ident, $node_id:ident, $version:ident, $value:expr) => {
        let node_id = to_entity($node_id, $version);
        let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
        let world = &mut world.gui;
        let willchange = world.get_mut::<TransformWillChange>(node_id);
        match willchange {
            Some(mut r) => {
				r.0.funcs.push($value);
				world.com_context.send_modify_event::<TransformWillChange>(node_id, StyleIndex::TransformWillChange, world);
			},
            None => {
                let transform = world.get_mut::<Transform>(node_id);
                match transform {
                    Some(mut r) => {
						r.funcs.push($value);
					},
                    None => {
                        let mut transform = Transform::default();
                        transform.funcs.push($value);
						world.entity_mut(to_entity($node_id, $version)).insert(transform);
                    }
                }
				world.com_context.send_modify_event::<Transform>(node_id, StyleIndex::Transform, world);
            }
        }
    };
}

#[macro_use()]
macro_rules! push_tanslate {
    ($world:ident, $node_id:ident, $version:ident, $modify: ident) => {
        let node_id = to_entity($node_id, $version);
        let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
        let world = &mut world.gui;
        let willchange = world.get_mut::<TransformWillChange>(node_id);
        match willchange {
            Some(mut r) => {
				$modify(&mut r.0);
				world.com_context.send_modify_event::<TransformWillChange>(node_id, StyleIndex::TransformWillChange, world);
			},
            None => {
                let transform = world.get_mut::<Transform>(node_id);
                match transform {
                    Some(mut r) => {
						$modify(&mut r);
					},
                    None => {
                        let mut transform = Transform::default();
                        $modify(&mut transform);
						world.entity_mut(to_entity($node_id, $version)).insert(transform);
                    }
                }
				world.com_context.send_modify_event::<Transform>(node_id, StyleIndex::Transform, world);
            }
        }
    };
}

/// 清空所有变换
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn clear_transform(world: u32, node_id: usize, version: u32) {
    // println!("clear_transform============={}", node_id);
    let node_id = node_id as usize;
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let attr = world.get_mut::<Transform>(to_entity(node_id, version));
    match attr {
        Some(mut r) => {
			r.funcs.clear();
            // r.modify(|transform: &mut Transform| {
            //     if transform.funcs.len() > 0 {
            //         transform.funcs.clear();
            //         true
            //     } else {
            //         false
            //     }
            // });
        }
        None => (),
    }
}

/// 移动变化
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn transform_translate(world: u32, node_id: usize, version: u32, x: f32, y: f32) {
    let transform_translate_m = |transform: &mut Transform| {
        if let Some(r) = transform.funcs.last_mut() {
            if let TransformFunc::Translate(x1, y1) = r {
                *x1 += x;
                *y1 += y;
                return;
            }
        }
        transform.funcs.push(TransformFunc::Translate(x, y));
    };
    push_tanslate!(world, node_id, version, transform_translate_m);
}

/// 移动变化
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn transform_translate_x(world: u32, node_id: usize, version: u32, value: f32) {
    let transform_translate_m = |transform: &mut Transform| {
        if let Some(r) = transform.funcs.last_mut() {
            match r {
                TransformFunc::Translate(x1, _) => {
                    *x1 += value;
                    return;
                }
                _ => (),
            }
        }
        transform.funcs.push(TransformFunc::Translate(value, 0.0));
    };
    push_tanslate!(world, node_id, version, transform_translate_m);
}

/// 移动变化
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn transform_translate_y(world: u32, node_id: usize, version: u32, value: f32) {
    let transform_translate_m = |transform: &mut Transform| {
        if let Some(r) = transform.funcs.last_mut() {
            match r {
                TransformFunc::Translate(_, y1) => {
                    *y1 += value;
                    return;
                }
                _ => (),
            }
        }
        // let len = transform.funcs.len();
        // println!("last--------------------------id: {}, {:?}, len:{}", node_id, transform.funcs.last_mut(), len);
        transform.funcs.push(TransformFunc::Translate(0.0, value));
        // let len = transform.funcs.len();
        // println!("last--------------------------id1: {}, {:?}: len{}", node_id, transform.funcs.last_mut(), len);
    };
    push_tanslate!(world, node_id, version, transform_translate_m);
}

/// 移动变化
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn transform_translate_percent(world: u32, node_id: usize, version: u32, x: f32, y: f32) {
    let transform_translate_m = |transform: &mut Transform| {
        if let Some(r) = transform.funcs.last_mut() {
            match r {
                TransformFunc::TranslatePercent(x1, y1) => {
                    *x1 += x;
                    *y1 += y;
                    return;
                }
                _ => (),
            }
        }
        transform
            .funcs
            .push(TransformFunc::TranslatePercent(x, y));
    };
    push_tanslate!(world, node_id, version, transform_translate_m);
}

/// 移动变化
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn transform_translate_x_percent(world: u32, node_id: usize, version: u32, value: f32) {
    let transform_translate_m = |transform: &mut Transform| {
        if let Some(r) = transform.funcs.last_mut() {
            match r {
                TransformFunc::TranslatePercent(x1, _) => {
                    *x1 += value;
                    return;
                }
                // TransformFunc::TranslateXPercent(x1) => {
                //     *x1 += value;
                //     return;
                // },
                _ => (),
            }
        }
        transform
            .funcs
            .push(TransformFunc::TranslatePercent(value, 0.0));
    };
    push_tanslate!(world, node_id, version, transform_translate_m);
}

/// 移动变化
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn transform_translate_y_percent(world: u32, node_id: usize, version: u32, value: f32) {
    let transform_translate_m = |transform: &mut Transform| {
        if let Some(r) = transform.funcs.last_mut() {
            match r {
                TransformFunc::TranslatePercent(_, y1) => {
                    *y1 += value;
                    return;
                }
                _ => (),
            }
        }
        transform
            .funcs
            .push(TransformFunc::TranslatePercent(0.0, value));
    };
    push_tanslate!(world, node_id, version, transform_translate_m);
}

/// 缩放变化
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn transform_scale(world: u32, node_id: usize, version: u32, x: f32, y: f32) {
    push_func!(world, node_id, version, TransformFunc::Scale(x, y));
}

/// 缩放变化
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn transform_scale_x(world: u32, node_id: usize, version: u32, value: f32) {
    push_func!(world, node_id, version, TransformFunc::ScaleX(value));
}

/// 缩放变化
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn transform_scale_y(world: u32, node_id: usize, version: u32, value: f32) {
    push_func!(world, node_id, version, TransformFunc::ScaleY(value));
}

/// 旋转变化
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn transform_rotate(world: u32, node_id: usize, version: u32, value: f32) {
    push_func!(world, node_id, version, TransformFunc::RotateZ(value));
}

/// 设置transfrom为none
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn transform_none(world: u32, node_id: usize, version: u32) {
    let node_id = node_id as usize;
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
	if let Some(mut r) = world.get_mut::<Transform>(to_entity(node_id, version)) {
		r.funcs.clear();
	}
    // unsafe { world.transform.lend_mut().get_unchecked_write(node_id) }.modify(
    //     |transform: &mut Transform| {
    //         if transform.funcs.len() > 0 {
    //             transform.funcs.clear();
    //             true
    //         } else {
    //             false
    //         }
    //     },
    // );
}

/// 设置变化原点
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn transform_origin(world: u32, node_id: usize, version: u32, x_ty: u8, x: f32, y_ty: u8, y: f32) {
    // let x_ty = unsafe { transmute(x_ty) };
    // let y_ty = unsafe { transmute(y_ty) };
    // let x_value = match x_ty {
    //     LengthUnitType::Pixel => LengthUnit::Pixel(x),
    //     LengthUnitType::Percent => LengthUnit::Percent(x),
    // };
    // let y_value = match y_ty {
    //     LengthUnitType::Pixel => LengthUnit::Pixel(y),
    //     LengthUnitType::Percent => LengthUnit::Percent(y),
    // };
    // let node_id = node_id as usize;
    // let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    // let world = &mut world.gui;
    // // let attr = world.transform_will_change.lend_mut();
	// let willchange = world.get_mut::<TransformWillChange>(to_entity(node_id, version));

    // match willchange {
    //     Some(mut r) => r.modify(|transform: &mut TransformWillChange| {
    //         transform.0.origin = TransformOrigin::XY(x_value, y_value);
    //         true
    //     }),
    //     None => {
    //         let attr = world.transform.lend_mut();
    //         match attr.get_write(node_id) {
    //             Some(mut r) => r.modify(|transform: &mut Transform| {
    //                 transform.origin = TransformOrigin::XY(x_value, y_value);
    //                 true
    //             }),
    //             None => {
    //                 let mut transform = Transform::default();
    //                 transform.origin = TransformOrigin::XY(x_value, y_value);
    //                 attr.insert(node_id, transform);
    //             }
    //         }
    //     }
    // }
}
