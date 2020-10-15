/// 将设置布局属性的接口导出到js
use std::mem::transmute;

use ecs::LendMut;

use gui::component::calc::StyleType1;
use gui::component::user::{OtherLayoutStyleWrite};
use flex_layout::style::*;
use flex_layout::Rect;
use GuiWorld;

#[macro_use()]
macro_rules! func_enum {
    ($func:ident, $ty:ident) => {
        #[allow(unused_attributes)]
        #[no_mangle]
		#[js_export]
        pub fn $func(world: u32, node_id: u32, value: u8) {
            let value = unsafe { transmute(value) };
            let node_id = node_id as usize;
            let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
			let world = &mut world.gui;
            world.style_mark.lend_mut()[node_id].local_style1 |=
                StyleType1::$ty as usize;
            unsafe { world.other_layout_style.lend_mut().get_unchecked_write(node_id) }.$func(value);
        }
    };
}
#[macro_use()]
macro_rules! func_value_dimension {
	($func:ident, $feild1:ident, $feild2:ident, $dime:ident, $notify_feild1:expr, $ty:ident, $style_tyle:ident) => {
        #[allow(unused_attributes)]
        #[no_mangle]
		#[js_export]
        pub fn $func(world: u32, node_id: u32, value: f32) {
            let node_id = node_id as usize;
            let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
            let world = &mut world.gui;
            world.style_mark.lend_mut()[node_id].local_style1 |=
				StyleType1::$ty as usize;
			let layout_styles = world.$style_tyle.lend_mut();
			layout_styles[node_id].$feild1.$feild2 = Dimension::$dime(value);
			layout_styles.get_notify_ref().modify_event(node_id, $notify_feild1, 0);
        }
	};

	($func:ident, $feild1:ident, $dime:ident, $notify_feild1:expr, $ty:ident, $style_tyle:ident) => {
        #[allow(unused_attributes)]
        #[no_mangle]
		#[js_export]
        pub fn $func(world: u32, node_id: u32, edge: u8, value: f32) {
            let node_id = node_id as usize;
            let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
            let world = &mut world.gui;
            world.style_mark.lend_mut()[node_id].local_style1 |=
				StyleType1::$ty as usize;
			let layout_styles = world.$style_tyle.lend_mut();

			match unsafe {transmute(edge)} {
				Edge::All => layout_styles[node_id].$feild1 = Rect{
					start: Dimension::$dime(value),
					end: Dimension::$dime(value),
					top: Dimension::$dime(value),
					bottom: Dimension::$dime(value),
				},
				Edge::Left => layout_styles[node_id].$feild1.start = Dimension::$dime(value),
				Edge::Top => layout_styles[node_id].$feild1.top = Dimension::$dime(value),
				Edge::Right => layout_styles[node_id].$feild1.end = Dimension::$dime(value),
				Edge::Bottom => layout_styles[node_id].$feild1.bottom = Dimension::$dime(value),
				_ => return
			};
			layout_styles.get_notify_ref().modify_event(node_id, $notify_feild1, 0);
        }
	};
}

#[macro_use()]
macro_rules! func_value_dimension_simple {
	($func:ident, $feild1:ident, $feild2:ident, $dime:ident, $notify_feild1:expr, $ty:ident, $style_tyle:ident) => {
        #[allow(unused_attributes)]
        #[no_mangle]
		#[js_export]
        pub fn $func(world: u32, node_id: u32) {
            let node_id = node_id as usize;
            let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
            let world = &mut world.gui;
            world.style_mark.lend_mut()[node_id].local_style1 |=
				StyleType1::$ty as usize;
			let layout_styles = world.$style_tyle.lend_mut();
			layout_styles[node_id].$feild1.$feild2 = Dimension::$dime;
			layout_styles.get_notify_ref().modify_event(node_id, $notify_feild1, 0);
        }
	};
}

#[macro_use()]
macro_rules! func_value {
	
	($func:ident, $ty:ident) => {
        #[allow(unused_attributes)]
        #[no_mangle]
#[js_export]
        pub fn $func(world: u32, node_id: u32, value: f32) {
            let node_id = node_id as usize;
            let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
            let world = &mut world.gui;
            world.style_mark.lend_mut()[node_id].local_style1 |=
                StyleType1::$ty as usize;
            unsafe { world.other_layout_style.lend_mut().get_unchecked_write(node_id) }.$func(value);
        }
    };
    ($func:ident, $ty:ident, $feild:expr) => {
        #[allow(unused_attributes)]
        #[no_mangle]
#[js_export]
        pub fn $func(world: u32, node_id: u32, value: f32) {
            let node_id = node_id as usize;
            let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
            let world = &mut world.gui;
            world.style_mark.lend_mut()[node_id].local_style1 |=
                StyleType1::$ty as usize;
            unsafe { world.other_layout_style.lend_mut().get_unchecked_write(node_id) }.$func(value);
        }
    };
}


func_enum!(set_align_content, AlignContent);
func_enum!(set_align_items, AlignItems);
func_enum!(set_justify_content, JustifyContent);
func_enum!(set_flex_direction, FlexDirection);
func_enum!(set_flex_wrap, FlexWrap);
func_enum!(set_align_self, AlignSelf);
func_enum!(set_position_type, PositionType);


func_value!(set_flex_grow, FlexGrow);
func_value!(set_flex_shrink, FlexShrink);
// func_value!(set_flex_basis, FlexBasis);

func_value_dimension!(set_width,              size,     width, Points, "width",  Width, rect_layout_style);
func_value_dimension!(set_height,             size,     height, Points, "height",  Height, rect_layout_style);
func_value_dimension!(set_min_width,          min_size, width, Points, "min_width", MinWidth, other_layout_style);
func_value_dimension!(set_min_height,         min_size, height, Points, "min_height", MinHeight, other_layout_style);
func_value_dimension!(set_max_width,          max_size, height, Points, "max_width", MaxWidth, other_layout_style);
func_value_dimension!(set_max_height,         max_size, height, Points, "max_height", MaxHeight, other_layout_style);

func_value_dimension!(set_width_percent,      size,     width, Percent, "width",  Width, rect_layout_style);
func_value_dimension!(set_height_percent,     size,     height, Percent, "height",  Height, rect_layout_style);
func_value_dimension!(set_min_width_percent,  min_size, width, Percent, "min_width", MinWidth, other_layout_style);
func_value_dimension!(set_min_height_percent, min_size, height, Percent, "min_height", MinHeight, other_layout_style);
func_value_dimension!(set_max_width_percent,  max_size, height, Percent, "max_width", MaxWidth, other_layout_style);
func_value_dimension!(set_max_height_percent, max_size, height, Percent, "max_height", MaxHeight, other_layout_style);


func_value_dimension_simple!(set_width_auto, size, width, Auto, "width", Width, rect_layout_style);
func_value_dimension_simple!(set_height_auto,  size, height, Auto, "height", Height, rect_layout_style);
// func_value_dimension_simple!(set_margin_auto, margin, margin, Auto, "margin", Margin);
// func_auto!(set_flex_basis_auto, FlexBasis::Auto);

func_value_dimension!(set_padding,          padding, Points, "padding", Padding, other_layout_style);
func_value_dimension!(set_margin,           margin, Points, "margin", Margin, rect_layout_style);
func_value_dimension!(set_border,           border, Points, "border", Border, other_layout_style);
func_value_dimension!(set_position,         position, Points, "position", Position, other_layout_style);

func_value_dimension!(set_padding_percent,  padding, Percent, "padding", Padding, other_layout_style);
func_value_dimension!(set_margin_percent,   margin, Percent, "margin", Margin, rect_layout_style);
func_value_dimension!(set_position_percent, position, Percent, "position", Position, other_layout_style);

// #[no_mangle]
// #[js_export]
// pub fn init_height(world: u32, node_id: u32, height: u32) {
//     let node_id = node_id as usize;
//     let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
//     let yogas = world.gui.yoga.lend_mut();
//     let yoga = unsafe { yogas.get_unchecked_mut(node_id) };
//     match yoga.get_height().unit {
//         yoga::YGUnit::YGUnitAuto => yoga.set_height(height as f32),
//         _ => (),
//     }
// }

pub enum Edge {
    Left = 0,
    Top = 1,
    Right = 2,
    Bottom = 3,
    Start = 4,
    End = 5,
    Horizontal = 6,
    Vertical = 7,
    All = 8,
}
