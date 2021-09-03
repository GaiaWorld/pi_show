use wasm_bindgen::prelude::*;
/// 将设置布局属性的接口导出到js
use std::mem::transmute;
use std::ops::DerefMut;
// use ecs::LendMut;
use bevy_ecs::prelude::{World};

use gui::component::calc::{StyleIndex};
use gui::component::user::*;
use gui::util::event::{EventType};
use gui::single::to_entity;
use gui::world::App;
use flex_layout::style::*;
use flex_layout::Rect;
use crate::world::GuiWorld;

#[macro_use()]
macro_rules! func_enum {
    ($func:ident, $ty:ident) => {
		$crate::paste::item! {
			#[allow(unused_attributes)]
			#[wasm_bindgen]
			pub fn [<set_ $func>](world: u32, node_id: u32, node_version: u32, value: u8) {
				let node_id = to_entity(node_id as usize, node_version);
				// let node_id = node_id as usize as *mut Entity;
				let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
				let world = &mut world.gui;
				world.com_context.get_mut::<OtherLayoutStyle>(world, node_id).unwrap().$func = unsafe { transmute(value) };
				
				// 通知
				world.com_context.send_modify_event::<OtherLayoutStyle>(node_id, StyleIndex::$ty, world);
			}
		}
    };
}
#[macro_use()]
macro_rules! func_value_dimension {
	($func:ident, $feild1:ident, $feild2:ident, $dime:ident, $notify_feild1:expr, $ty:ident, $style_tyle:ident) => {
		$crate::paste::item! {
			#[allow(unused_attributes)]
			#[wasm_bindgen]
			pub fn [<set_ $func>](world: u32, node_id: u32, node_version: u32, value: f32) {
				let node_id = to_entity(node_id as usize, node_version);
				let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
				let world = &mut world.gui;
				world.get_mut::<$style_tyle>(node_id).unwrap().$feild1.$feild2 = Dimension::$dime(value);

				// 通知
				world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::$ty, world);
			}
		}
	};

	($func:ident, $feild1:ident, $dime:ident, $notify_feild0:expr, $notify_feild1:expr, $notify_feild2:expr, $notify_feild3:expr, $notify_feild4:expr,$ty:ident, $style_tyle:ident) => {
		$crate::paste::item! {
			#[allow(unused_attributes)]
			#[wasm_bindgen]
			pub fn [<set_ $func>](world: u32, node_id: u32, node_version: u32, edge: u8, value: f32) {
				let node_id = to_entity(node_id as usize, node_version);
				let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
				let world = &mut world.gui;
				let mut layout_style = world.get_mut::<$style_tyle>(node_id).unwrap();
				
				match unsafe {transmute(edge)} {
					Edge::All => {
						layout_style.$feild1 = Rect{
							start: Dimension::$dime(value),
							end: Dimension::$dime(value),
							top: Dimension::$dime(value),
							bottom: Dimension::$dime(value),
						};
						
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Top>], world);
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Right>], world);
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Bottom>], world);
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Left>], world);
					},
					Edge::Top => {
						layout_style.$feild1.top = Dimension::$dime(value);
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Top>], world);
					}
					Edge::Right => {
						layout_style.$feild1.end = Dimension::$dime(value);
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Right>], world);
					}
					Edge::Bottom => {
						layout_style.$feild1.bottom = Dimension::$dime(value);
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Bottom>], world);
					},
					Edge::Left => {
						layout_style.$feild1.start = Dimension::$dime(value);
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Left>], world);
					}
					_ => return
				};
				
			}
		}
	};

	($func:ident, $feild1:ident, $notify_feild1:expr, $ty:ident, $style_tyle:ident) => {
		$crate::paste::item! {
			#[allow(unused_attributes)]
			#[wasm_bindgen]
			pub fn [<set_ $func>](world: u32, node_id: u32, node_version: u32, edge: u8) {
				let node_id = to_entity(node_id as usize, node_version);
				let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
				let world = &mut world.gui;
				// unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) }.local_style1 |=
				// 	StyleType1::$ty as usize;
				// let layout_styles = world.$style_tyle.lend_mut();
				let layout_style = world.get_mut::<$style_tyle>(node_id).unwrap();

				match unsafe {transmute(edge)} {
					Edge::All => {
						layout_style.$feild1 = Rect{
							start: Dimension::Auto,
							end: Dimension::Auto,
							top: Dimension::Auto,
							bottom: Dimension::Auto,
						};
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Top>], world);
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Right>], world);
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Bottom>], world);
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Left>], world);
					},
					Edge::Top => {
						layout_style.$feild1.top = Dimension::Auto;
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Top>], world);
					},
					Edge::Right => {
						layout_style.$feild1.end = Dimension::Auto;
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Right>], world);
					},
					Edge::Bottom => {
						layout_style.$feild1.bottom = Dimension::Auto;
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Bottom>], world);
					},
					Edge::Left => {
						layout_style.$feild1.start = Dimension::Auto;
						world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty Left>], world);
					},
					_ => return
				};
			}
		}
	};
}

#[macro_use()]
macro_rules! func_value_dimension_simple {
	($func:ident, $feild1:ident, $feild2:ident, $dime:ident, $notify_feild1:expr, $ty:ident, $style_tyle:ident) => {
		$crate::paste::item! {
			#[allow(unused_attributes)]
			#[wasm_bindgen]
			pub fn [<set_ $func>](world: u32, node_id: u32, node_version: u32) {
				let node_id = to_entity(node_id as usize, node_version);
				let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
				let world = &mut world.gui;
				let mut layout_style = world.get_mut::<$style_tyle>(node_id).unwrap();
				layout_style.$feild1.$feild2 = Dimension::$dime;

				world.com_context.send_modify_event::<$style_tyle>(node_id, StyleIndex::[<$ty>], world);
			}
		}
	};
}


#[macro_use()]
macro_rules! func_value {
	
	($func:ident, $ty:ident) => {
		$crate::paste::item! {
			#[allow(unused_attributes)]
			#[wasm_bindgen]
			pub fn [<set_ $func>](world: u32, node_id: u32, node_version: u32, value: f32) {
				let node_id = to_entity(node_id as usize, node_version);
				let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
				let world = &mut world.gui;
				world.com_context.get_mut::<OtherLayoutStyle>(world, node_id).unwrap().$func = value;
				
				world.com_context.send_modify_event::<OtherLayoutStyle>(node_id, StyleIndex::[<$ty>], world);
			}
		}
    };
    ($func:ident, $ty:ident, $feild:expr) => {
		$crate::paste::item! {
			#[allow(unused_attributes)]
			#[wasm_bindgen]
			pub fn [<set_ $func>](world: u32, node_id: u32, node_version: u32, value: f32) {
				let node_id = to_entity(node_id as usize, node_version);
				let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
				let world = &mut world.gui;
				world.com_context.get_mut::<OtherLayoutStyle>(world, node_id).unwrap().$func = value;

				world.com_context.send_modify_event::<OtherLayoutStyle>(node_id, StyleIndex::[<$ty>], world);
			}
		}
    };
}


func_enum!(align_content, AlignContent);
func_enum!(align_items, AlignItems);
func_enum!(justify_content, JustifyContent);
func_enum!(flex_direction, FlexDirection);
func_enum!(flex_wrap, FlexWrap);
func_enum!(align_self, AlignSelf);
func_enum!(position_type, PositionType);


func_value!(flex_grow, FlexGrow);
func_value!(flex_shrink, FlexShrink);
// func_value!(flex_basis, FlexBasis);

func_value_dimension!(width,              size,     width, Points, "width",  Width, RectLayoutStyle);
func_value_dimension!(height,             size,     height, Points, "height",  Height, RectLayoutStyle);
func_value_dimension!(min_width,          min_size, width, Points, "min_width", MinWidth, OtherLayoutStyle);
func_value_dimension!(min_height,         min_size, height, Points, "min_height", MinHeight, OtherLayoutStyle);
func_value_dimension!(max_width,          max_size, width, Points, "max_width", MaxWidth, OtherLayoutStyle);
func_value_dimension!(max_height,         max_size, height, Points, "max_height", MaxHeight, OtherLayoutStyle);

func_value_dimension!(width_percent,      size,     width, Percent, "width",  Width, RectLayoutStyle);
func_value_dimension!(height_percent,     size,     height, Percent, "height",  Height, RectLayoutStyle);
func_value_dimension!(min_width_percent,  min_size, width, Percent, "min_width", MinWidth, OtherLayoutStyle);
func_value_dimension!(min_height_percent, min_size, height, Percent, "min_height", MinHeight, OtherLayoutStyle);
func_value_dimension!(max_width_percent,  max_size, height, Percent, "max_width", MaxWidth, OtherLayoutStyle);
func_value_dimension!(max_height_percent, max_size, height, Percent, "max_height", MaxHeight, OtherLayoutStyle);


func_value_dimension_simple!(width_auto, size, width, Auto, "width", Width, RectLayoutStyle);
func_value_dimension_simple!(height_auto,  size, height, Auto, "height", Height, RectLayoutStyle);
// func_value_dimension!(margin_auto, margin, "margin", Margin, RectLayoutStyle);

// func_auto!(flex_basis_auto, FlexBasis::Auto);

func_value_dimension!(padding,          padding, Points, "padding", "padding-top", "padding-right", "padding-bottom", "padding-left", Padding, OtherLayoutStyle);
func_value_dimension!(margin,           margin, Points, "margin", "margin-top", "margin-right", "margin-bottom", "margin-left", Margin, RectLayoutStyle);
func_value_dimension!(border,           border, Points, "border", "border-top", "border-right", "border-bottom", "border-left", Border, OtherLayoutStyle);
func_value_dimension!(position,         position, Points, "position", "top", "right", "bottom", "left", Position, OtherLayoutStyle);

func_value_dimension!(padding_percent,  padding, Percent, "padding", "padding-top", "padding-right", "padding-bottom", "padding-left", Padding, OtherLayoutStyle);
func_value_dimension!(margin_percent,   margin, Percent, "margin", "margin-top", "margin-right", "margin-bottom", "margin-left", Margin, RectLayoutStyle);
func_value_dimension!(position_percent, position, Percent, "position", "top", "right", "bottom", "left", Position, OtherLayoutStyle);


#[wasm_bindgen]
pub fn set_margin_auto(world: u32, node_id: u32, node_version: u32, edge: u8) {
	let node_id = to_entity(node_id as usize, node_version);
	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
	let world = &mut world.gui;

	let mut layout_style = world.com_context.get_mut::<RectLayoutStyle>(world, node_id).unwrap();
	
	match unsafe {transmute(edge)} {
		Edge::All => {
			layout_style.margin = Rect{
				start: Dimension::Auto,
				end: Dimension::Auto,
				top: Dimension::Auto,
				bottom: Dimension::Auto,
			};

			world.com_context.send_modify_event::<OtherLayoutStyle>(node_id, StyleIndex::MarginTop, world);
			world.com_context.send_modify_event::<OtherLayoutStyle>(node_id, StyleIndex::MarginRight, world);
			world.com_context.send_modify_event::<OtherLayoutStyle>(node_id, StyleIndex::MarginBottom, world);
			world.com_context.send_modify_event::<OtherLayoutStyle>(node_id, StyleIndex::MarginLeft, world);
		},
		Edge::Top => {
			layout_style.margin.top = Dimension::Auto;
			world.com_context.send_modify_event::<OtherLayoutStyle>(node_id, StyleIndex::MarginTop, world);
		}
		Edge::Right => {
			layout_style.margin.end = Dimension::Auto;
			world.com_context.send_modify_event::<OtherLayoutStyle>(node_id, StyleIndex::MarginRight, world);
		}
		Edge::Bottom => {
			layout_style.margin.bottom = Dimension::Auto;
			world.com_context.send_modify_event::<OtherLayoutStyle>(node_id, StyleIndex::MarginBottom, world);
		},
		Edge::Left => {
			layout_style.margin.start = Dimension::Auto;
			world.com_context.send_modify_event::<OtherLayoutStyle>(node_id, StyleIndex::MarginLeft, world);
		}
		_ => return
	};
}

// #[no_mangle]
// #[js_export]
// // pub fn init_height(world: u32, node_id: u32, height: u32) {
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
