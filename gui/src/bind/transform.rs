use wcs::world::{World};

use world_doc::WorldDocMgr;
use world_doc::component::node::{ NodeWriteRef};
use world_doc::component::style::transform::{ TransformWriteRef, TransformFunc};
use world_doc::component::style::transform::Transform;

#[no_mangle]
pub fn transform_translate(world: u32, node_id: u32, x: f32, y: f32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let transform_id = world.component_mgr.node._group.get(node_id).transform;
    if transform_id == 0 {
        let mut transform = Transform::default();
        transform.funcs.push(TransformFunc::Translate(x, y));
        let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_transform(transform);
    }else {
        let mut transform_ref = TransformWriteRef::new(transform_id, world.component_mgr.node.transform.to_usize(), &mut world.component_mgr);
        transform_ref.modify(|transform: &mut Transform| {
            transform.funcs.push(TransformFunc::Translate(x, y));
            true
        });
        return;
    }
}

#[no_mangle]
pub fn transform_translate_x(world: u32, node_id: u32, value: f32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let transform_id = world.component_mgr.node._group.get(node_id).transform;
    if transform_id == 0 {
        let mut transform = Transform::default();
        transform.funcs.push(TransformFunc::TranslateX(value));
        let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_transform(transform);
    }else {
        let mut transform_ref = TransformWriteRef::new(transform_id, world.component_mgr.node.transform.to_usize(), &mut world.component_mgr);
        transform_ref.modify(|transform: &mut Transform| {
            transform.funcs.push(TransformFunc::TranslateX(value));
            true
        });
        return;
    }
}

#[no_mangle]
pub fn transform_translate_y(world: u32, node_id: u32, value: f32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let transform_id = world.component_mgr.node._group.get(node_id).transform;
    if transform_id == 0 {
        let mut transform = Transform::default();
        transform.funcs.push(TransformFunc::TranslateY(value));
        let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_transform(transform);
    }else {
        let mut transform_ref = TransformWriteRef::new(transform_id, world.component_mgr.node.transform.to_usize(), &mut world.component_mgr);
        transform_ref.modify(|transform: &mut Transform| {
            transform.funcs.push(TransformFunc::TranslateY(value));
            true
        });
        return;
    }
}


#[no_mangle]
pub fn transform_translate_percent(world: u32, node_id: u32, x: f32, y: f32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let transform_id = world.component_mgr.node._group.get(node_id).transform;
    if transform_id == 0 {
        let mut transform = Transform::default();
        transform.funcs.push(TransformFunc::TranslatePercent(x, y));
        let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_transform(transform);
    }else {
        let mut transform_ref = TransformWriteRef::new(transform_id, world.component_mgr.node.transform.to_usize(), &mut world.component_mgr);
        transform_ref.modify(|transform: &mut Transform| {
            transform.funcs.push(TransformFunc::TranslatePercent(x, y));
            true
        });
        return;
    }
}

#[no_mangle]
pub fn transform_translate_x_percent(world: u32, node_id: u32, value: f32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let transform_id = world.component_mgr.node._group.get(node_id).transform;
    if transform_id == 0 {
        let mut transform = Transform::default();
        transform.funcs.push(TransformFunc::TranslateXPercent(value));
        let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_transform(transform);
    }else {
        let mut transform_ref = TransformWriteRef::new(transform_id, world.component_mgr.node.transform.to_usize(), &mut world.component_mgr);
        transform_ref.modify(|transform: &mut Transform| {
            transform.funcs.push(TransformFunc::TranslateXPercent(value));
            true
        });
        return;
    }
}

#[no_mangle]
pub fn transform_translate_y_percent(world: u32, node_id: u32, value: f32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let transform_id = world.component_mgr.node._group.get(node_id).transform;
    if transform_id == 0 {
        let mut transform = Transform::default();
        transform.funcs.push(TransformFunc::TranslateYPercent(value));
        let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_transform(transform);
    }else {
        let mut transform_ref = TransformWriteRef::new(transform_id, world.component_mgr.node.transform.to_usize(), &mut world.component_mgr);
        transform_ref.modify(|transform: &mut Transform| {
            transform.funcs.push(TransformFunc::TranslateYPercent(value));
            true
        });
        return;
    }
}


#[no_mangle]
pub fn transform_scale(world: u32, node_id: u32, x: f32, y: f32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let transform_id = world.component_mgr.node._group.get(node_id).transform;
    if transform_id == 0 {
        let mut transform = Transform::default();
        transform.funcs.push(TransformFunc::Scale(x, y));
        let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_transform(transform);
    }else {
        let mut transform_ref = TransformWriteRef::new(transform_id, world.component_mgr.node.transform.to_usize(), &mut world.component_mgr);
        transform_ref.modify(|transform: &mut Transform| {
            transform.funcs.push(TransformFunc::Scale(x, y));
            true
        });
        return;
    }
}

#[no_mangle]
pub fn transform_scale_x(world: u32, node_id: u32, value: f32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let transform_id = world.component_mgr.node._group.get(node_id).transform;
    if transform_id == 0 {
        let mut transform = Transform::default();
        transform.funcs.push(TransformFunc::ScaleX(value));
        let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_transform(transform);
    }else {
        let mut transform_ref = TransformWriteRef::new(transform_id, world.component_mgr.node.transform.to_usize(), &mut world.component_mgr);
        transform_ref.modify(|transform: &mut Transform| {
            transform.funcs.push(TransformFunc::ScaleX(value));
            true
        });
        return;
    }
}

#[no_mangle]
pub fn transform_scale_y(world: u32, node_id: u32, value: f32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let transform_id = world.component_mgr.node._group.get(node_id).transform;
    if transform_id == 0 {
        let mut transform = Transform::default();
        transform.funcs.push(TransformFunc::ScaleY(value));
        let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_transform(transform);
    }else {
        let mut transform_ref = TransformWriteRef::new(transform_id, world.component_mgr.node.transform.to_usize(), &mut world.component_mgr);
        transform_ref.modify(|transform: &mut Transform| {;
            transform.funcs.push(TransformFunc::ScaleY(value));
            true
        });
        return;
    }
}

#[no_mangle]
pub fn transform_rotate(world: u32, node_id: u32, value: f32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let transform_id = world.component_mgr.node._group.get(node_id).transform;
    if transform_id == 0 {
        let mut transform = Transform::default();
        transform.funcs.push(TransformFunc::RotateZ(value));
        let mut node_ref = NodeWriteRef::new(node_id, world.component_mgr.node.to_usize(), &mut world.component_mgr);
        node_ref.set_transform(transform);
    }else {
        let mut transform_ref = TransformWriteRef::new(transform_id, world.component_mgr.node.transform.to_usize(), &mut world.component_mgr);
        transform_ref.modify(|transform: &mut Transform| {;
            transform.funcs.push(TransformFunc::RotateZ(value));
            true
        });
        return;
    }
}

#[no_mangle]
pub fn transform_none(world: u32, node_id: u32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let transform_id = world.component_mgr.node._group.get(node_id).transform;
    if transform_id != 0 {
        let mut transform_ref = TransformWriteRef::new(transform_id, world.component_mgr.node.transform.to_usize(), &mut world.component_mgr);
        transform_ref.modify(|transform: &mut Transform| {;
            transform.funcs.clear();
            true
        });
    }
}