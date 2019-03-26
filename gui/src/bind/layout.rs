use std::mem::transmute;

use bind::{Pointer};

#[no_mangle]
pub fn set_align_content(own: u32, value: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let value = unsafe{transmute(value)};
    node.yoga.set_align_content(value);
    js!{console.log(@{format!("set_align_content, {:?}", value)});}
}

#[no_mangle]
pub fn set_align_items(own: u32, value: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let value = unsafe{transmute(value)};
    node.yoga.set_align_content(value);
    js!{console.log(@{format!("set_align_items, {:?}", value)} );}
}

#[no_mangle]
pub fn set_justify_content(own: u32, value: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let value = unsafe{transmute(value)};
    node.yoga.set_align_content(value);
    js!{console.log(@{format!("set_justify_content, {:?}", value)} );}
}

#[no_mangle]
pub fn set_flex_direction(own: u32, value: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let value = unsafe{transmute(value)};
    node.yoga.set_align_content(value);
    js!{console.log(@{format!("set_flex_direction, {:?}", value)} );}
}

#[no_mangle]
pub fn set_flex_wrap(own: u32, value: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let value = unsafe{transmute(value)};
    node.yoga.set_align_content(value);
    js!{console.log(@{format!("set_flex_wrap, {:?}", value)} );}
}

#[no_mangle]
pub fn set_flex_grow(own: u32, value: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let value = unsafe{transmute(value)};
    node.yoga.set_align_content(value);
    js!{console.log(@{format!("set_flex_grow, {:?}", value)} );}
}

#[no_mangle] pub fn set_flex_shrink(own: u32, value: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let value = unsafe{transmute(value)};
    node.yoga.set_align_content(value);
    js!{console.log(@{format!("set_flex_shrink, {:?}", value)} );}
}

#[no_mangle] pub fn set_flex_basis(own: u32, value: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let value = unsafe{transmute(value)};
    node.yoga.set_align_content(value);
    js!{console.log(@{format!("set_flex_basis, {:?}", value)} );} 
}

#[no_mangle] pub fn set_align_self(own: u32, value: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let value = unsafe{transmute(value)};
    node.yoga.set_align_content(value);
    js!{console.log(@{format!("set_align_self, {:?}", value)} );} 
}

#[no_mangle]
pub fn set_padding(own: u32, edge:u32, value: f32) {
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_padding(edge, value);
    js!{console.log(@{format!("set_padding, edge: {:?}, value{:?}", edge, value)} );}
}

#[no_mangle]
pub fn set_padding_percent(own: u32, edge:u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_padding_percent(edge, value);
    js!{console.log(@{format!("set_padding_percent, edge: {:?}, value{:?}", edge, value)} );}
}

#[no_mangle]
pub fn set_margin(own: u32, edge:u32, value: f32) {
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_margin(edge, value);
    js!{console.log(@{format!("set_margin, edge: {:?}, value{:?}", edge, value)} );}
}

#[no_mangle]
pub fn set_margin_percent(own: u32, edge:u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_margin_percent(edge, value);
    js!{console.log(@{format!("set_margin_percent, edge: {:?}, value{:?}", edge, value)} );}
}

#[no_mangle]
pub fn set_margin_auto(own: u32, edge:u32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_margin_auto(edge);
    js!{console.log(@{format!("set_margin_auto, edge: {:?}", edge)} );}
}

#[no_mangle]
pub fn set_border(own: u32, edge:u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_border(edge, value);
    js!{console.log(@{format!("set_border, edge: {:?}", edge)} );}
}

#[no_mangle]
pub fn set_position_type(own: u32, value: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let value = unsafe{transmute(value)};
    node.yoga.set_position_type(value);
    js!{console.log(@{format!("set_position_ty, value{:?}", value)} );}
}

#[no_mangle]
pub fn set_position(own: u32, edge:u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_position(edge, value);
    js!{console.log(@{format!("set_position, edge:{:?}, value{:?}", edge, value)} );}
}

#[no_mangle]
pub fn set_position_percent(own: u32, edge:u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_position_percent(edge, value);
    js!{console.log(@{format!("set_position_percent, edge:{:?}, value{:?}", edge, value)} );}
}


#[no_mangle]
pub fn set_width(own: u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_width(value);
    js!{console.log(@{format!("set_width, value{:?}", value)} );}
}

#[no_mangle]
pub fn set_width_percent(own: u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_width_percent(value);
    js!{console.log(@{format!("set_width_percent, value{:?}", value)} );}
}

#[no_mangle]
pub fn set_width_auto(own: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_width_auto();
    js!{console.log(@{format!("set_width_auto")} );}
}

#[no_mangle]
pub fn set_height(own: u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_width_auto();
    js!{console.log(@{format!("set_height, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_height_percent(own: u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_height_percent(value);
    js!{console.log(@{format!("set_height_percent, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_height_auto(own: u32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_height_auto();
    js!{console.log(@{format!("set_height_auto")} );}
}

#[no_mangle]
pub fn set_min_width(own: u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_min_width(value);
    js!{console.log(@{format!("set_min_width, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_min_width_percent(own: u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_min_width_percent(value);
    js!{console.log(@{format!("set_min_width_percent, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_min_height(own: u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_min_height(value);
    js!{console.log(@{format!("set_min_height, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_min_height_percent(own: u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_min_height_percent(value);
    js!{console.log(@{format!("set_min_height_percent, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_max_width(own: u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_max_width(value);
    js!{console.log(@{format!("set_max_width, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_max_width_percent(own: u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_max_width_percent(value);
    js!{console.log(@{format!("set_max_width_percent, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_max_height(own: u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_max_height(value);
    js!{console.log(@{format!("set_max_height, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_max_height_percent(own: u32, value: f32){
    let node = unsafe {&*(own as *const Pointer)};
    let world = node.world.borrow_mut();
    let node =  world.component_mgr.node._group.get(node.id);
    node.yoga.set_max_height_percent(value);
    js!{console.log(@{format!("set_max_height_percent, value: {:?}", value)} );}
}