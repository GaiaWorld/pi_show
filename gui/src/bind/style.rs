// use std::rc::Rc;
// use std::cell::RefCell;
use std::mem::transmute;

use stdweb::web::TypedArray;

use wcs::component::{Builder};

use component::node::{ NodeWriteRef};
use component::style::element::{ElementId, RectWriteRef, Element};
use component::style::generic::{Overflow, OverflowWriteRef, ClipPathWriteRef};
use component::style::element::{ RectBuilder as RectElementBuilder};
use component::style::color::{Color};
use component::style::generic::{ClipPath, Clip, Opacity, OpacityWriteRef};
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

// #[no_mangle] pub fn get_backgroud_color(_own: u32) -> u32{
//     1
// }

// 设置背景颜色， 类型为rgba
#[no_mangle]
pub fn set_backgroud_rgba_color(own: u32, r: f32, g: f32, b: f32, a: f32){
    js!{console.log("set_backgroud_color");}
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let element_id = world.component_mgr.node._group.get(node.id).element.clone();
    match element_id {
        ElementId::Rect(elem_id) => {
            if elem_id > 0 {
                let mut rect_ref = RectWriteRef::new(elem_id, world.component_mgr.node.element.rect.to_usize(), &mut world.component_mgr);
                rect_ref.set_color(Color::RGBA(CgColor(CgColor1::new(r, g, b, a))));
                return;
            }
        },
        ElementId::None => (),
        _ => return,
    }
    let rect = RectElementBuilder::new()
    .color(Color::RGBA(CgColor(CgColor1::new(r, g, b, a))))
    .build(&mut world.component_mgr.node.element.rect);
    let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    node_ref.set_element(Element::Rect(rect));
}

// 设置一个线性渐变的背景颜色
#[no_mangle]
pub fn set_backgroud_radial_gradient_color(own: u32, color_and_positions: TypedArray<f32>, center_x: f32, center_y: f32, shape: u8, size: u8 ){
    js!{console.log("set_backgroud_radial_gradient_color");} 
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let element_id = world.component_mgr.node._group.get(node.id).element.clone();
    match element_id {
        ElementId::Rect(elem_id) => {
            if elem_id > 0 {
                let mut rect_ref = RectWriteRef::new(elem_id, world.component_mgr.node.element.rect.to_usize(), &mut world.component_mgr);
                rect_ref.set_color(Color::RadialGradient(to_radial_gradient_color(color_and_positions, center_x, center_y, shape, size)));
                return;
            }
        },
        ElementId::None => (),
        _ => return,
    }
    let rect = RectElementBuilder::new()
    .color(Color::RadialGradient(to_radial_gradient_color(color_and_positions, center_x, center_y, shape, size)))
    .build(&mut world.component_mgr.node.element.rect);
    let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    node_ref.set_element(Element::Rect(rect));
}

// 设置一个径向渐变的背景颜色
#[no_mangle]
pub fn set_backgroud_linear_gradient_color(own: u32, color_and_positions: TypedArray<f32>, direction: f32){
    js!{console.log("set_backgroud_linear_gradient_color");} 
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let element_id = world.component_mgr.node._group.get(node.id).element.clone();
    match element_id {
        ElementId::Rect(elem_id) => {
            if elem_id > 0 {
                let mut rect_ref = RectWriteRef::new(elem_id, world.component_mgr.node.element.rect.to_usize(), &mut world.component_mgr);
                rect_ref.set_color(Color::LinearGradient(to_linear_gradient_color(color_and_positions, direction)));
                return;
            }
        },
        ElementId::None => (),
        _ => return,
    }
    let rect = RectElementBuilder::new()
    .color(Color::LinearGradient(to_linear_gradient_color(color_and_positions, direction)))
    .build(&mut world.component_mgr.node.element.rect);
    let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    node_ref.set_element(Element::Rect(rect));
}

// 设置裁剪属性，geometry_box, 可能的值为MarginBox， BorderBox， PaddingBox， ContentBox
#[no_mangle]
pub fn set_clip_path_geometry_box(own: u32, value: u8){
    js!{console.log("set_clip_path")} 
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let clip_id = world.component_mgr.node._group.get(node.id).clip;
    if clip_id == 0 {
        let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        let clip = match unsafe{transmute(value)} {
            ClipPathGeometryBoxType::BorderBox => Clip::BorderBox,
            ClipPathGeometryBoxType::MarginBox => Clip::MarginBox,
            ClipPathGeometryBoxType::PaddingBox => Clip::PaddingBox,
            ClipPathGeometryBoxType::ContentBox => Clip::ContentBox,
        };
        node_ref.set_clip(ClipPath{value: clip} );
    }else {
        let mut clip_ref = ClipPathWriteRef::new(clip_id, world.component_mgr.node.clip.to_usize(), &mut world.component_mgr);
        let clip = match unsafe{transmute(value)} {
            ClipPathGeometryBoxType::BorderBox => Clip::BorderBox,
            ClipPathGeometryBoxType::MarginBox => Clip::MarginBox,
            ClipPathGeometryBoxType::PaddingBox => Clip::PaddingBox,
            ClipPathGeometryBoxType::ContentBox => Clip::ContentBox,
        };
        clip_ref.modify(|clip_path: &mut ClipPath| {
            clip_path.value = clip;
            true
        });
    }
}

// 设置裁剪属性，basic-shape, 目前只支持basic-shape
#[no_mangle]
pub fn set_clip_path_basic_shape(own: u32, ty: u8, data: TypedArray<f32>){
    js!{console.log("set_clip_path")} 
    let node = unsafe {&*(own as *const Pointer)};
    // let ty: ClipPathBasicShapeType = unsafe{transmute(ty)};

    let mut world = node.world.borrow_mut();
    let clip_id = world.component_mgr.node._group.get(node.id).clip;
    if clip_id == 0 {
        //如果不存在clip组件， 创建ClipPath， 并设置node的clip属性
        let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        let clip = match unsafe{transmute(ty)}  {
            ClipPathBasicShapeType::Polygon => Clip::Polygon(to_polygon(data)),
            _ => return,
        };
        node_ref.set_clip(ClipPath{value: clip} );
    }else {
        //如果存在clip组件， 应该直接修改clip属性
        let mut clip_ref = ClipPathWriteRef::new(clip_id, world.component_mgr.node.clip.to_usize(), &mut world.component_mgr);
        let clip = match unsafe{transmute(ty)}  {
            ClipPathBasicShapeType::Polygon => Clip::Polygon(to_polygon(data)),
            _ => return,
        };
        clip_ref.modify(|clip_path: &mut ClipPath| {
            clip_path.value = clip;
            true
        });
    }
}

// 设置overflow属性， 支持visible， hidden
#[no_mangle]
pub fn set_overflow(own: u32, x: u8, y: u8){
    js!{console.log("set_overflow")} 
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let overflow_id = world.component_mgr.node._group.get(node.id).overflow;
    if overflow_id == 0 {
        let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_overflow(Overflow{x: unsafe{transmute(x)}, y: unsafe{transmute(y)}});
    } else {
        let mut overflow_ref = OverflowWriteRef::new(overflow_id, world.component_mgr.node.overflow.to_usize(), &mut world.component_mgr);
        overflow_ref.modify(|overflow: &mut Overflow| {
            overflow.x = unsafe{transmute(x)};
            overflow.y = unsafe{transmute(y)};
            true
        });
    }
}

//设置不透明度
pub fn set_opacity(own: u32, value: f32) {
    js!{console.log("set_opacity")} 
    let node = unsafe {&*(own as *const Pointer)};
    let mut world = node.world.borrow_mut();
    let opacity_id = world.component_mgr.node._group.get(node.id).opacity;
    if opacity_id == 0 {
        let mut node_ref = NodeWriteRef::new(node.id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_opacity(Opacity{value: value});
    } else {
        let mut opacity_ref = OpacityWriteRef::new(opacity_id, world.component_mgr.node.opacity.to_usize(), &mut world.component_mgr);
        opacity_ref.modify(|opacity: &mut Opacity| {
            opacity.value = value;
            true
        });
    }
}