use std::mem::transmute;

use wcs::world::{World};

use world_doc::WorldDocMgr;

#[no_mangle]
pub fn set_align_content(world: u32, node_id: u32, value: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let value = unsafe{transmute(value)};
    node.yoga.set_align_content(value);
    js!{console.log(@{format!("set_align_content, {:?}", value)});}
}

#[no_mangle]
pub fn set_align_items(world: u32, node_id: u32, value: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let value = unsafe{transmute(value)};
    node.yoga.set_align_items(value);
    js!{console.log(@{format!("set_align_items, {:?}", value)} );}
}

#[no_mangle]
pub fn set_justify_content(world: u32, node_id: u32, value: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let value = unsafe{transmute(value)};
    node.yoga.set_justify_content(value);
    js!{console.log(@{format!("set_justify_content, {:?}", value)} );}
}

#[no_mangle]
pub fn set_flex_direction(world: u32, node_id: u32, value: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let value = unsafe{transmute(value)};
    node.yoga.set_flex_direction(value);
    js!{console.log(@{format!("set_flex_direction, {:?}", value)} );}
}

#[no_mangle]
pub fn set_flex_wrap(world: u32, node_id: u32, value: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let value = unsafe{transmute(value)};
    node.yoga.set_flex_wrap(value);
    js!{console.log(@{format!("set_flex_wrap, {:?}", value)} );}
}

#[no_mangle]
pub fn set_flex_grow(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_flex_grow(value);
    js!{console.log(@{format!("set_flex_grow, {:?}", value)} );}
}

#[no_mangle] pub fn set_flex_shrink(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_flex_shrink(value);
    js!{console.log(@{format!("set_flex_shrink, {:?}", value)} );}
}

#[no_mangle] pub fn set_flex_basis(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_flex_basis(value);
    js!{console.log(@{format!("set_flex_basis, {:?}", value)} );} 
}

#[no_mangle] pub fn set_flex_basis_auto(world: u32, node_id: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_flex_basis_auto();
    js!{console.log("set_flex_basis");} 
}



#[no_mangle] pub fn set_align_self(world: u32, node_id: u32, value: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let value = unsafe{transmute(value)};
    node.yoga.set_align_self(value);
    js!{console.log(@{format!("set_align_self, {:?}", value)} );} 
}

#[no_mangle]
pub fn set_padding(world: u32, node_id: u32, edge:u32, value: f32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_padding(edge, value);
    js!{console.log(@{format!("set_padding, edge: {:?}, value{:?}", edge, value)} );}
}

#[no_mangle]
pub fn set_padding_percent(world: u32, node_id: u32, edge:u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_padding_percent(edge, value);
    js!{console.log(@{format!("set_padding_percent, edge: {:?}, value{:?}", edge, value)} );}
}

#[no_mangle]
pub fn set_margin(world: u32, node_id: u32, edge:u32, value: f32) {
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_margin(edge, value);
    js!{console.log(@{format!("set_margin, edge: {:?}, value{:?}", edge, value)} );}
}

#[no_mangle]
pub fn set_margin_percent(world: u32, node_id: u32, edge:u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_margin_percent(edge, value);
    js!{console.log(@{format!("set_margin_percent, edge: {:?}, value{:?}", edge, value)} );}
}

#[no_mangle]
pub fn set_margin_auto(world: u32, node_id: u32, edge:u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_margin_auto(edge);
    js!{console.log(@{format!("set_margin_auto, edge: {:?}", edge)} );}
}

#[no_mangle]
pub fn set_border(world: u32, node_id: u32, edge:u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_border(edge, value);
    js!{console.log(@{format!("set_border, edge: {:?}", edge)} );}
}

#[no_mangle]
pub fn set_position_type(world: u32, node_id: u32, value: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let value = unsafe{transmute(value)};
    node.yoga.set_position_type(value);
    js!{console.log(@{format!("set_position_ty, value{:?}", value)} );}
}

#[no_mangle]
pub fn set_position(world: u32, node_id: u32, edge:u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_position(edge, value);
    js!{console.log(@{format!("set_position, edge:{:?}, value{:?}", edge, value)} );}
}

#[no_mangle]
pub fn set_position_percent(world: u32, node_id: u32, edge:u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    let edge = unsafe{transmute(edge)};
    node.yoga.set_position_percent(edge, value);
    js!{console.log(@{format!("set_position_percent, edge:{:?}, value{:?}", edge, value)} );}
}


#[no_mangle]
pub fn set_width(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_width(value);
    js!{console.log(@{format!("set_width, value{:?}", value)} );}
}

#[no_mangle]
pub fn set_width_percent(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_width_percent(value);
    js!{console.log(@{format!("set_width_percent, value{:?}", value)} );}
}

#[no_mangle]
pub fn set_width_auto(world: u32, node_id: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_width_auto();
    js!{console.log(@{format!("set_width_auto")} );}
}

#[no_mangle]
pub fn set_height(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_height(value);
    js!{console.log(@{format!("set_height, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_height_percent(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_height_percent(value);
    js!{console.log(@{format!("set_height_percent, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_height_auto(world: u32, node_id: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_height_auto();
    js!{console.log(@{format!("set_height_auto")} );}
}

#[no_mangle]
pub fn set_min_width(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_min_width(value);
    js!{console.log(@{format!("set_min_width, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_min_width_percent(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_min_width_percent(value);
    js!{console.log(@{format!("set_min_width_percent, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_min_height(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_min_height(value);
    js!{console.log(@{format!("set_min_height, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_min_height_percent(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_min_height_percent(value);
    js!{console.log(@{format!("set_min_height_percent, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_max_width(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_max_width(value);
    js!{console.log(@{format!("set_max_width, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_max_width_percent(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_max_width_percent(value);
    js!{console.log(@{format!("set_max_width_percent, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_max_height(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_max_height(value);
    js!{console.log(@{format!("set_max_height, value: {:?}", value)} );}
}

#[no_mangle]
pub fn set_max_height_percent(world: u32, node_id: u32, value: f32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)}; 
    let node =  world.component_mgr.node._group.get(node_id);
    node.yoga.set_max_height_percent(value);
    js!{console.log(@{format!("set_max_height_percent, value: {:?}", value)} );}
}