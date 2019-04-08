// use std::rc::Rc;
// use std::cell::RefCell;
use std::mem::transmute;

use stdweb::web::TypedArray;

use wcs::component::{Builder};

use world_doc::component::node::{ NodeWriteRef};
use world_doc::component::style::generic::{DecorateBuilder, DecorateWriteRef, BoxShadow, BoxShadowBuilder, BoxShadowWriteRef};
use component::color::{Color};
// use world_doc::component::style::generic::{ClipPath, Clip, Opacity, OpacityWriteRef};
use component::math::{Color as CgColor};
use cg::color::{Color as CgColor1};

use bind::{Pointer};
use bind::data::*;

// #[no_mangle] pub fn get_display(_own: u32) -> u32{ 
//     1
// }

// #[no_mangle] pub fn get_layout(_own: u32) -> u32{
//     1
// }

// #[no_mangle] pub fn get_clip_path(_own: u32) -> u32{
//     1
// }

// #[no_mangle] pub fn get_text_style(_own: u32) -> u32{ 
//     1
// }

// #[no_mangle] pub fn get_transform(_own: u32) -> u32{
//     1
// }

// #[no_mangle] pub fn get_background_color(_own: u32) -> u32{
//     1
// }

// #[no_mangle] pub fn get_background_color(_own: u32) -> u32{
//     1
// }


// 设置边框颜色， 类型为rgba
#[no_mangle]
pub fn set_border_color(own: u32, r: f32, g: f32, b: f32, a: f32){
    js!{console.log("set_border_color");}
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let decorate_id = world.component_mgr.node._group.get(node.id).decorate;
    if decorate_id == 0 {
        let decorate = DecorateBuilder::new()
        .border_color(CgColor(CgColor1::new(r, g, b, a)))
        .build(&mut world.component_mgr.node.decorate);
        let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_decorate(decorate);
    }else {
        let mut rect_ref = DecorateWriteRef::new(decorate_id, world.component_mgr.node.decorate.to_usize(), &mut world.component_mgr);
        rect_ref.set_border_color(CgColor(CgColor1::new(r, g, b, a)));
        return;
    }
}

// 设置边框圆角
#[no_mangle]
pub fn set_border_radius(own: u32, value: f32){
    js!{console.log("set_border_radius");}
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let decorate_id = world.component_mgr.node._group.get(node.id).decorate;
    if decorate_id == 0 {
        let decorate = DecorateBuilder::new()
        .border_radius(value)
        .build(&mut world.component_mgr.node.decorate);
        let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_decorate(decorate);
    }else {
        let mut rect_ref = DecorateWriteRef::new(decorate_id, world.component_mgr.node.decorate.to_usize(), &mut world.component_mgr);
        rect_ref.set_border_radius(value);
        return;
    }
}

// 设置阴影颜色
#[no_mangle]
pub fn set_box_shadow_color(own: u32, r: f32, g: f32, b: f32, a: f32){
    js!{console.log("set_box_shadow_color");}
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let decorate_id = world.component_mgr.node._group.get(node.id).decorate;
    if decorate_id == 0 {
        let decorate = DecorateBuilder::new()
        .box_shadow(BoxShadowBuilder::new().color(CgColor(CgColor1::new(r, g, b, a)))
            .build(&mut world.component_mgr.node.decorate.box_shadow))
        .build(&mut world.component_mgr.node.decorate);
        let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_decorate(decorate);
    }else {
        let box_shadow_id = world.component_mgr.node.decorate._group.get(decorate_id).box_shadow;
        if box_shadow_id == 0 {
            let box_shadow = BoxShadowBuilder::new().color(CgColor(CgColor1::new(r, g, b, a)))
            .build(&mut world.component_mgr.node.decorate.box_shadow);
            let mut decorate_ref = DecorateWriteRef::new(decorate_id, world.component_mgr.node.decorate.to_usize(), &mut world.component_mgr);
            decorate_ref.set_box_shadow(box_shadow);
        } else {
            let mut box_shadow_ref = BoxShadowWriteRef::new(decorate_id, world.component_mgr.node.decorate.box_shadow.to_usize(), &mut world.component_mgr);
            box_shadow_ref.set_color(CgColor(CgColor1::new(r, g, b, a)));
        }
    }
}


// 设置阴影h
#[no_mangle]
pub fn set_box_shadow_h(own: u32, h: f32){
    js!{console.log("set_box_shadow_h");}
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let decorate_id = world.component_mgr.node._group.get(node.id).decorate;
    if decorate_id == 0 {
        let decorate = DecorateBuilder::new()
        .box_shadow(BoxShadowBuilder::new().h(h)
            .build(&mut world.component_mgr.node.decorate.box_shadow))
        .build(&mut world.component_mgr.node.decorate);
        let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_decorate(decorate);
    }else {
        let box_shadow_id = world.component_mgr.node.decorate._group.get(decorate_id).box_shadow;
        if box_shadow_id == 0 {
            let box_shadow = BoxShadowBuilder::new().h(h)
            .build(&mut world.component_mgr.node.decorate.box_shadow);
            let mut decorate_ref = DecorateWriteRef::new(decorate_id, world.component_mgr.node.decorate.to_usize(), &mut world.component_mgr);
            decorate_ref.set_box_shadow(box_shadow);
        } else {
            let mut box_shadow_ref = BoxShadowWriteRef::new(decorate_id, world.component_mgr.node.decorate.box_shadow.to_usize(), &mut world.component_mgr);
            box_shadow_ref.set_h(h);
        }
    }
}

// 设置阴影v
#[no_mangle]
pub fn set_box_shadow_v(own: u32, v: f32){
    js!{console.log("set_box_shadow_v");}
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let decorate_id = world.component_mgr.node._group.get(node.id).decorate;
    if decorate_id == 0 {
        let decorate = DecorateBuilder::new()
        .box_shadow(BoxShadowBuilder::new().v(v)
            .build(&mut world.component_mgr.node.decorate.box_shadow))
        .build(&mut world.component_mgr.node.decorate);
        let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_decorate(decorate);
    }else {
        let box_shadow_id = world.component_mgr.node.decorate._group.get(decorate_id).box_shadow;
        if box_shadow_id == 0 {
            let box_shadow = BoxShadowBuilder::new().v(v)
            .build(&mut world.component_mgr.node.decorate.box_shadow);
            let mut decorate_ref = DecorateWriteRef::new(decorate_id, world.component_mgr.node.decorate.to_usize(), &mut world.component_mgr);
            decorate_ref.set_box_shadow(box_shadow);
        } else {
            let mut box_shadow_ref = BoxShadowWriteRef::new(decorate_id, world.component_mgr.node.decorate.box_shadow.to_usize(), &mut world.component_mgr);
            box_shadow_ref.set_v(v);
        }
    }
}


// 设置背景颜色， 类型为rgba
#[no_mangle]
pub fn set_backgroud_rgba_color(own: u32, r: f32, g: f32, b: f32, a: f32){
    js!{console.log("set_background_color");}
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let decorate_id = world.component_mgr.node._group.get(node.id).decorate;
    if decorate_id == 0 {
        let decorate = DecorateBuilder::new()
        .background_color(Color::RGBA(CgColor(CgColor1::new(r, g, b, a))))
        .build(&mut world.component_mgr.node.decorate);
        let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_decorate(decorate);
    }else {
        let mut rect_ref = DecorateWriteRef::new(decorate_id, world.component_mgr.node.decorate.to_usize(), &mut world.component_mgr);
        rect_ref.set_background_color(Color::RGBA(CgColor(CgColor1::new(r, g, b, a))));
        return;
    }
}

// 设置一个线性渐变的背景颜色
#[no_mangle]
pub fn set_backgroud_radial_gradient_color(own: u32, color_and_positions: TypedArray<f32>, center_x: f32, center_y: f32, shape: u8, size: u8 ){
    js!{console.log("set_backgroud_radial_gradient_color");} 
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let decorate_id = world.component_mgr.node._group.get(node.id).decorate;
    if decorate_id == 0 {
        let decorate = DecorateBuilder::new()
        .background_color(Color::RadialGradient(to_radial_gradient_color(color_and_positions, center_x, center_y, shape, size)))
        .build(&mut world.component_mgr.node.decorate);
        let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_decorate(decorate);
    }else {
        let mut rect_ref = DecorateWriteRef::new(decorate_id, world.component_mgr.node.decorate.to_usize(), &mut world.component_mgr);
        rect_ref.set_background_color(Color::RadialGradient(to_radial_gradient_color(color_and_positions, center_x, center_y, shape, size)));
        return;
    }
}

// 设置一个径向渐变的背景颜色
#[no_mangle]
pub fn set_backgroud_linear_gradient_color(own: u32, color_and_positions: TypedArray<f32>, direction: f32){
    js!{console.log("set_backgroud_linear_gradient_color");} 
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();

    let decorate_id = world.component_mgr.node._group.get(node.id).decorate;
    if decorate_id == 0 {
        let decorate = DecorateBuilder::new()
        .background_color(Color::LinearGradient(to_linear_gradient_color(color_and_positions, direction)))
        .build(&mut world.component_mgr.node.decorate);
        let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_decorate(decorate);
    }else {
        let mut rect_ref = DecorateWriteRef::new(decorate_id, world.component_mgr.node.decorate.to_usize(), &mut world.component_mgr);
        rect_ref.set_background_color(Color::LinearGradient(to_linear_gradient_color(color_and_positions, direction)));
        return;
    }
}

// // 设置裁剪属性，geometry_box, 可能的值为MarginBox， BorderBox， PaddingBox， ContentBox
// #[no_mangle]
// pub fn set_clip_path_geometry_box(own: u32, value: u8){
//     js!{console.log("set_clip_path")} 
//     let node = unsafe {&*(own as *const Pointer)};
//     let mut world = node.world.borrow_mut();
//     let clip_id = world.component_mgr.node._group.get(node.id).clip;
//     if clip_id == 0 {
//         let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
//         let clip = match unsafe{transmute(value)} {
//             ClipPathGeometryBoxType::BorderBox => Clip::BorderBox,
//             ClipPathGeometryBoxType::MarginBox => Clip::MarginBox,
//             ClipPathGeometryBoxType::PaddingBox => Clip::PaddingBox,
//             ClipPathGeometryBoxType::ContentBox => Clip::ContentBox,
//         };
//         node_ref.set_clip(ClipPath{value: clip} );
//     }else {
//         let mut clip_ref = ClipPathWriteRef::new(clip_id, world.component_mgr.node.clip.to_usize(), &mut world.component_mgr);
//         let clip = match unsafe{transmute(value)} {
//             ClipPathGeometryBoxType::BorderBox => Clip::BorderBox,
//             ClipPathGeometryBoxType::MarginBox => Clip::MarginBox,
//             ClipPathGeometryBoxType::PaddingBox => Clip::PaddingBox,
//             ClipPathGeometryBoxType::ContentBox => Clip::ContentBox,
//         };
//         clip_ref.modify(|clip_path: &mut ClipPath| {
//             clip_path.value = clip;
//             true
//         });
//     }
// }

// // 设置裁剪属性，basic-shape, 目前只支持basic-shape
// #[no_mangle]
// pub fn set_clip_path_basic_shape(own: u32, ty: u8, data: TypedArray<f32>){
//     js!{console.log("set_clip_path")} 
//     let node = unsafe {&*(own as *const Pointer)};
//     // let ty: ClipPathBasicShapeType = unsafe{transmute(ty)};

//     let mut world = node.world.borrow_mut();
//     let clip_id = world.component_mgr.node._group.get(node.id).clip;
//     if clip_id == 0 {
//         //如果不存在clip组件， 创建ClipPath， 并设置node的clip属性
//         let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
//         let clip = match unsafe{transmute(ty)}  {
//             ClipPathBasicShapeType::Polygon => Clip::Polygon(to_polygon(data)),
//             _ => return,
//         };
//         node_ref.set_clip(ClipPath{value: clip} );
//     }else {
//         //如果存在clip组件， 应该直接修改clip属性
//         let mut clip_ref = ClipPathWriteRef::new(clip_id, world.component_mgr.node.clip.to_usize(), &mut world.component_mgr);
//         let clip = match unsafe{transmute(ty)}  {
//             ClipPathBasicShapeType::Polygon => Clip::Polygon(to_polygon(data)),
//             _ => return,
//         };
//         clip_ref.modify(|clip_path: &mut ClipPath| {
//             clip_path.value = clip;
//             true
//         });
//     }
// }

pub fn set_overflow(own: u32, value: bool){
    js!{console.log("set_overflow")} 
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    world.component_mgr.get_node_mut(node.id).set_overflow(value);
}

//设置不透明度
pub fn set_opacity(own: u32, value: f32) {
    js!{console.log("set_opacity")} 
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    world.component_mgr.get_node_mut(node.id).set_opacity(value);
}

// pub fn set_undefined(own: u32, value: u8) {
//     let node = unsafe {&*(own as *const Pointer)};
//     let mut world = node.world.borrow_mut();
//     let ty: UndefinedType = unsafe{ transmute(value) };
//     match ty {
//         BorderColor => {
//             let decorate_ref = world.component_mgr.get_node_mut(node.id).get_decorate_mut();
//             if decorate_ref.id == 0 {
//                 return;
//             }
//             decorate_ref.set_border_color(CgColor(CgColor1::new(1.0, 1.0, 1.0, 0.0)))
//         },
//         BorderRadius => {
//             let decorate_ref = world.component_mgr.get_node_mut(node.id).get_decorate_mut();
//             if decorate_ref.id == 0 {
//                 return;
//             }
//             decorate_ref.set_border_radius(0.0)
//         },

//         BoxShadowColor => {
//             let decorate_ref = world.component_mgr.get_node_mut(node.id).get_decorate_mut();
//             if decorate_ref.id == 0 {
//                 return;
//             }
//             decorate_ref.del_box_shadow();
//         },
//         BoxShadowH => {

//         },
//         BoxShadowV => {

//         },
//         //BoxShadowBlur, 暂不支持

//         BackgroundColor =>  {
//             // let decorate_ref = world.component_mgr.get_node_mut(node.id).get_decorate_mut();
//             // if decorate_ref.id == 0 {
//             //     return;
//             // }
//             // let border_color = decorate_ref.get_border
//         },

//         //
//         Opacity,
//         Overflow,

//         //布局
//         AlignContent,
//         JustifyContent,
//         FlexDirection,
//         FlexWrap,
//         FlexGrow,
//         FlexShrink,
//         FlexBasis,
//         AlignSelf,

//         Left,
//         Top,
//         Right,
//         Bottom,

//         Width,
//         Height,

//         MaxWidth,
//         MaHeight,
//         MinWidth,
//         MinHeight,

//         PaddindLeft,
//         PaddindTop,
//         PaddindRight,
//         PaddindBottom,

//         MarginLeft,
//         MarginTop,
//         MarginRight,
//         MarginBottom,
//     }
// }

pub enum UndefinedType {
    // decorate
    BorderColor,
    BorderRadius,

    BoxShadowColor,
    BoxShadowH,
    BoxShadowV,
    //BoxShadowBlur, 暂不支持

    BackgroundColor,

    //
    Opacity,
    Overflow,

    //布局
    AlignContent,
    JustifyContent,
    FlexDirection,
    FlexWrap,
    FlexGrow,
    FlexShrink,
    FlexBasis,
    AlignSelf,

    Left,
    Top,
    Right,
    Bottom,

    Width,
    Height,

    MaxWidth,
    MaHeight,
    MinWidth,
    MinHeight,

    PaddindLeft,
    PaddindTop,
    PaddindRight,
    PaddindBottom,

    MarginLeft,
    MarginTop,
    MarginRight,
    MarginBottom,


}