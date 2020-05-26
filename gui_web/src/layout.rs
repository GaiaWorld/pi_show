/// 将设置布局属性的接口导出到js
use std::mem::transmute;

use bc::*;
use ecs::LendMut;

use gui::component::calc::StyleType1;
use gui::layout::*;
use yoga;
use GuiWorld;

#[macro_use()]
macro_rules! func_enum {
    ($func:ident, $ty:ident) => {
        #[allow(unused_attributes)]
        #[no_mangle]
        pub fn $func(world: u32, node_id: u32, value: u8) {
            let value = unsafe { transmute(value) };
            let node_id = node_id as usize;
            let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
            let world = &mut world.gui;
            unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) }.local_style1 |=
                StyleType1::$ty as usize;
            unsafe { world.yoga.lend_mut().get_unchecked_write(node_id) }.modify(|s| {
                s.$func(value);
                true
            });
        }
    };
}
#[macro_use()]
macro_rules! func_value {
    ($func:ident, $ty:ident) => {
        #[allow(unused_attributes)]
        #[no_mangle]
        pub fn $func(world: u32, node_id: u32, value: f32) {
            let node_id = node_id as usize;
            let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
            let world = &mut world.gui;
            unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) }.local_style1 |=
                StyleType1::$ty as usize;
            unsafe { world.yoga.lend_mut().get_unchecked_write(node_id) }.modify(|s| {
                s.$func(value);
                true
            });
        }
    };
}
#[macro_use()]
macro_rules! func_enum_value {
    ($func:ident, $ty:ident) => {
        #[allow(unused_attributes)]
        #[no_mangle]
        pub fn $func(world: u32, node_id: u32, edge: u8, value: f32) {
            let edge = unsafe { transmute(edge) };
            let node_id = node_id as usize;
            let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
            let world = &mut world.gui;
            unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) }.local_style1 |=
                StyleType1::$ty as usize;
            unsafe { world.yoga.lend_mut().get_unchecked_write(node_id) }.modify(|s| {
                s.$func(edge, value);
                true
            });
        }
    };
}
#[macro_use()]
macro_rules! func_auto {
    ($func:ident, $ty:ident) => {
        #[allow(unused_attributes)]
        #[no_mangle]
        pub fn $func(world: u32, node_id: u32) {
            let node_id = node_id as usize;
            let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
            let world = &mut world.gui;
            unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) }.local_style1 |=
                StyleType1::$ty as usize;
            unsafe { world.yoga.lend_mut().get_unchecked_write(node_id) }.modify(|s| {
                s.$func();
                true
            });
        }
    };
}

func_enum!(set_align_content, AlignContent);
func_enum!(set_align_items, AlignItems);
func_enum!(set_justify_content, JustifyContent);
func_enum!(set_flex_direction, FlexDirection);
func_enum!(set_flex_wrap, FlexWrap);
func_enum!(set_align_self, AlignSelf);
func_enum!(set_margin_auto, Margin);
func_enum!(set_position_type, PositionType);

func_value!(set_flex_grow, FlexGrow);
func_value!(set_flex_shrink, FlexShrink);
func_value!(set_flex_basis, FlexBasis);
func_value!(set_width, Width);
func_value!(set_width_percent, Width);
func_value!(set_height, Height);
func_value!(set_height_percent, Height);
func_value!(set_min_width, MinWidth);
func_value!(set_min_width_percent, MinWidth);
func_value!(set_min_height, MinHeight);
func_value!(set_min_height_percent, MinHeight);
func_value!(set_max_width, MaxWidth);
func_value!(set_max_width_percent, MaxWidth);
func_value!(set_max_height, MaxHeight);
func_value!(set_max_height_percent, MaxHeight);

func_auto!(set_flex_basis_auto, FlexBasis);
func_auto!(set_width_auto, Width);
func_auto!(set_height_auto, Height);

func_enum_value!(set_padding, Padding);
func_enum_value!(set_padding_percent, Padding);
func_enum_value!(set_margin, Margin);
func_enum_value!(set_margin_percent, Margin);
func_enum_value!(set_border, Border);
func_enum_value!(set_position, Position);
func_enum_value!(set_position_percent, Position);

#[no_mangle]
pub fn init_width(world: u32, node_id: u32, width: u32) {
    let node_id = node_id as usize;
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let yogas = world.gui.yoga.lend_mut();
    let yoga = unsafe { yogas.get_unchecked_mut(node_id) };
    match yoga.get_style_width_unit() {
        YGUnit::YGUnitAuto => yoga.set_width(width as f32),
        _ => (),
    }
}

#[no_mangle]
pub fn init_height(world: u32, node_id: u32, height: u32) {
    let node_id = node_id as usize;
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let yogas = world.gui.yoga.lend_mut();
    let yoga = unsafe { yogas.get_unchecked_mut(node_id) };
    match yoga.get_height().unit {
        yoga::YGUnit::YGUnitAuto => yoga.set_height(height as f32),
        _ => (),
    }
}
