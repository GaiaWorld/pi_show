use std::mem::transmute;

use ecs::{LendMut};
use bc::*;

use gui::layout::*;
use GuiWorld;


#[macro_use()]
macro_rules! func_enum {
    ($func:ident) => {
        #[allow(unused_attributes)]
        #[no_mangle]
        pub fn $func(world: u32, node_id: u32, value: u32){
            let value = unsafe{transmute(value)};
            let node_id = node_id as usize;
            let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
            unsafe {world.yoga.lend_mut().get_unchecked_write(node_id)}.modify(|s| {
                s.$func(value);
                true
            });
        }
    };
}
#[macro_use()]
macro_rules! func_value {
    ($func:ident) => {
        #[allow(unused_attributes)]
        #[no_mangle]
        pub fn $func(world: u32, node_id: u32, value: f32){
            let node_id = node_id as usize;
            let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
            unsafe {world.yoga.lend_mut().get_unchecked_write(node_id)}.modify(|s| {
                s.$func(value);
                true
            });
        }
    };
}
#[macro_use()]
macro_rules! func_enum_value {
    ($func:ident) => {
        #[allow(unused_attributes)]
        #[no_mangle]
        pub fn $func(world: u32, node_id: u32, edge: u32, value: f32){
            let edge = unsafe{transmute(edge)};
            let node_id = node_id as usize;
            let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
            unsafe {world.yoga.lend_mut().get_unchecked_write(node_id)}.modify(|s| {
                s.$func(edge, value);
                true
            });
        }
    };
}
#[macro_use()]
macro_rules! func_auto {
    ($func:ident) => {
        #[allow(unused_attributes)]
        #[no_mangle]
        pub fn $func(world: u32, node_id: u32){
            let node_id = node_id as usize;
            let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
            unsafe {world.yoga.lend_mut().get_unchecked_write(node_id)}.modify(|s| {
                s.$func();
                true
            });
        }
    };
}

func_enum!(set_align_content);
func_enum!(set_align_items);
func_enum!(set_justify_content);
func_enum!(set_flex_direction);
func_enum!(set_flex_wrap);
func_enum!(set_align_self);
func_enum!(set_margin_auto);
func_enum!(set_position_type);

func_value!(set_flex_grow);
func_value!(set_flex_shrink);
func_value!(set_flex_basis);
func_value!(set_width);
func_value!(set_width_percent);
func_value!(set_height);
func_value!(set_height_percent);
func_value!(set_min_width);
func_value!(set_min_width_percent);
func_value!(set_min_height);
func_value!(set_min_height_percent);
func_value!(set_max_width);
func_value!(set_max_width_percent);
func_value!(set_max_height);
func_value!(set_max_height_percent);

func_auto!(set_flex_basis_auto);
func_auto!(set_width_auto);
func_auto!(set_height_auto);


func_enum_value!(set_padding);
func_enum_value!(set_padding_percent);
func_enum_value!(set_margin);
func_enum_value!(set_margin_percent);
func_enum_value!(set_border);
func_enum_value!(set_position);
func_enum_value!(set_position_percent);
