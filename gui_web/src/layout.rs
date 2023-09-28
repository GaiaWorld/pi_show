// use wasm_bindgen::prelude::*;
// /// 将设置布局属性的接口导出到js
// use std::mem::transmute;

// use ecs::LendMut;

// use gui::component::calc::{StyleType2};
// use gui::component::user::{OtherLayoutStyleWrite};
// use flex_layout::style::*;
// use flex_layout::Rect;
// use crate::world::GuiWorld;

// #[macro_use()]
// macro_rules! func_enum {
//     ($func:ident, $ty:ident) => {
//         #[allow(unused_attributes)]
//         #[wasm_bindgen]
//         pub fn $func(world: u32, node_id: u32, value: u8) {
//             let value = unsafe { transmute(value) };
//             let node_id = node_id as usize;
//             let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
// 			let world = &mut world.gui;
//             world.style_mark.lend_mut()[node_id].local_style2 |=
//                 StyleType2::$ty as usize;
//             unsafe { world.other_layout_style.lend_mut().get_unchecked_write(node_id) }.$func(value);
//         }
//     };
// }
// #[macro_use()]
// macro_rules! func_value_dimension {
// 	($func:ident, $feild1:ident, $feild2:ident, $dime:ident, $notify_feild1:expr, $ty:ident, $style_tyle:ident) => {
//         #[allow(unused_attributes)]
//         #[wasm_bindgen]
//         pub fn $func(world: u32, node_id: u32, value: f32) {
//             let node_id = node_id as usize;
//             let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
//             let world = &mut world.gui;
//             world.style_mark.lend_mut()[node_id].local_style2 |=
// 				StyleType2::$ty as usize;
// 			let layout_styles = world.$style_tyle.lend_mut();
// 			layout_styles[node_id].$feild1.$feild2 = Dimension::$dime(value);
// 			layout_styles.get_notify_ref().modify_event(node_id, $notify_feild1, 0);
//         }
// 	};

// 	($func:ident, $feild1:ident, $dime:ident, $notify_feild0:expr, $notify_feild1:expr, $notify_feild2:expr, $notify_feild3:expr, $notify_feild4:expr,$ty:ident, $style_tyle:ident) => {
//         #[allow(unused_attributes)]
//         #[wasm_bindgen]
//         pub fn $func(world: u32, node_id: u32, edge: u8, value: f32) {
//             let node_id = node_id as usize;
//             let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
//             let world = &mut world.gui;
// 			let layout_styles = world.$style_tyle.lend_mut();
			 
// 			$crate::paste::item! {
// 				match unsafe {transmute(edge)} {
// 					Edge::All => {
// 						layout_styles[node_id].$feild1 = Rect{
// 							start: Dimension::$dime(value),
// 							end: Dimension::$dime(value),
// 							top: Dimension::$dime(value),
// 							bottom: Dimension::$dime(value),
// 						};
// 						world.style_mark.lend_mut()[node_id].local_style2 |= StyleType2::[<$ty Top>] as usize;
// 						layout_styles.get_notify_ref().modify_event(node_id, $notify_feild1, 0);
// 					},
// 					Edge::Top => {
// 						layout_styles[node_id].$feild1.top = Dimension::$dime(value);
// 						world.style_mark.lend_mut()[node_id].local_style2 |= StyleType2::[<$ty Top>] as usize;
// 						layout_styles.get_notify_ref().modify_event(node_id, $notify_feild1, 0);
// 					}
// 					Edge::Right => {
// 						layout_styles[node_id].$feild1.end = Dimension::$dime(value);
// 						world.style_mark.lend_mut()[node_id].local_style2 |= StyleType2::[<$ty Left>] as usize;
// 						layout_styles.get_notify_ref().modify_event(node_id, $notify_feild2, 0);
// 					}
// 					Edge::Bottom => {
// 						layout_styles[node_id].$feild1.bottom = Dimension::$dime(value);
// 						world.style_mark.lend_mut()[node_id].local_style2 |= StyleType2::[<$ty Bottom>] as usize;
// 						layout_styles.get_notify_ref().modify_event(node_id, $notify_feild3, 0);
// 					},
// 					Edge::Left => {
// 						layout_styles[node_id].$feild1.start = Dimension::$dime(value);
// 						world.style_mark.lend_mut()[node_id].local_style2 |= StyleType2::[<$ty Left>] as usize;
// 						layout_styles.get_notify_ref().modify_event(node_id, $notify_feild4, 0);
// 					}
// 					_ => return
// 				};
// 			}
			
//         }
// 	};

// 	($func:ident, $feild1:ident, $notify_feild1:expr, $ty:ident, $style_tyle:ident) => {
//         #[allow(unused_attributes)]
//         #[wasm_bindgen]
//         pub fn $func(world: u32, node_id: u32, edge: u8) {
//             let node_id = node_id as usize;
//             let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
//             let world = &mut world.gui;
//             unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) }.local_style1 |=
// 				StyleType1::$ty as usize;
// 			let layout_styles = world.$style_tyle.lend_mut();

// 			match unsafe {transmute(edge)} {
// 				Edge::All => layout_styles[node_id].$feild1 = Rect{
// 					start: Dimension::Auto,
// 					end: Dimension::Auto,
// 					top: Dimension::Auto,
// 					bottom: Dimension::Auto,
// 				},
// 				Edge::Left => layout_styles[node_id].$feild1.start = Dimension::Auto,
// 				Edge::Top => layout_styles[node_id].$feild1.top = Dimension::Auto,
// 				Edge::Right => layout_styles[node_id].$feild1.end = Dimension::Auto,
// 				Edge::Bottom => layout_styles[node_id].$feild1.bottom = Dimension::Auto,
// 				_ => return
// 			};
// 			layout_styles.get_notify().modify_event(node_id, $notify_feild1, 0);
//         }
// 	};
// }

// #[macro_use()]
// macro_rules! func_value_dimension_simple {
// 	($func:ident, $feild1:ident, $feild2:ident, $dime:ident, $notify_feild1:expr, $ty:ident, $style_tyle:ident) => {
//         #[allow(unused_attributes)]
//         #[wasm_bindgen]
//         pub fn $func(world: u32, node_id: u32) {
//             let node_id = node_id as usize;
//             let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
//             let world = &mut world.gui;
//             world.style_mark.lend_mut()[node_id].local_style2 |=
// 				StyleType2::$ty as usize;
// 			let layout_styles = world.$style_tyle.lend_mut();
// 			layout_styles[node_id].$feild1.$feild2 = Dimension::$dime;
// 			layout_styles.get_notify_ref().modify_event(node_id, $notify_feild1, 0);
//         }
// 	};
// }


// #[macro_use()]
// macro_rules! func_value {
	
// 	($func:ident, $ty:ident) => {
//         #[allow(unused_attributes)]
//         #[wasm_bindgen]
//         pub fn $func(world: u32, node_id: u32, value: f32) {
//             let node_id = node_id as usize;
//             let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
//             let world = &mut world.gui;
//             world.style_mark.lend_mut()[node_id].local_style2 |=
//                 StyleType2::$ty as usize;
//             unsafe { world.other_layout_style.lend_mut().get_unchecked_write(node_id) }.$func(value);
//         }
//     };
//     ($func:ident, $ty:ident, $feild:expr) => {
//         #[allow(unused_attributes)]
//         #[wasm_bindgen]
//         pub fn $func(world: u32, node_id: u32, value: f32) {
//             let node_id = node_id as usize;
//             let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
//             let world = &mut world.gui;
//             world.style_mark.lend_mut()[node_id].local_style2 |=
//                 StyleType2::$ty as usize;
//             unsafe { world.other_layout_style.lend_mut().get_unchecked_write(node_id) }.$func(value);
//         }
//     };
// }


// func_enum!(set_align_content, AlignContent);
// func_enum!(set_align_items, AlignItems);
// func_enum!(set_justify_content, JustifyContent);
// func_enum!(set_flex_direction, FlexDirection);
// func_enum!(set_flex_wrap, FlexWrap);
// func_enum!(set_align_self, AlignSelf);
// func_enum!(set_position_type, PositionType);


// func_value!(set_flex_grow, FlexGrow);
// func_value!(set_flex_shrink, FlexShrink);
// // func_value!(set_flex_basis, FlexBasis);

// func_value_dimension!(set_width,              size,     width, Points, "width",  Width, rect_layout_style);
// func_value_dimension!(set_height,             size,     height, Points, "height",  Height, rect_layout_style);
// func_value_dimension!(set_min_width,          min_size, width, Points, "min_width", MinWidth, other_layout_style);
// func_value_dimension!(set_min_height,         min_size, height, Points, "min_height", MinHeight, other_layout_style);
// func_value_dimension!(set_max_width,          max_size, width, Points, "max_width", MaxWidth, other_layout_style);
// func_value_dimension!(set_max_height,         max_size, height, Points, "max_height", MaxHeight, other_layout_style);

// func_value_dimension!(set_width_percent,      size,     width, Percent, "width",  Width, rect_layout_style);
// func_value_dimension!(set_height_percent,     size,     height, Percent, "height",  Height, rect_layout_style);
// func_value_dimension!(set_min_width_percent,  min_size, width, Percent, "min_width", MinWidth, other_layout_style);
// func_value_dimension!(set_min_height_percent, min_size, height, Percent, "min_height", MinHeight, other_layout_style);
// func_value_dimension!(set_max_width_percent,  max_size, height, Percent, "max_width", MaxWidth, other_layout_style);
// func_value_dimension!(set_max_height_percent, max_size, height, Percent, "max_height", MaxHeight, other_layout_style);


// func_value_dimension_simple!(set_width_auto, size, width, Auto, "width", Width, rect_layout_style);
// func_value_dimension_simple!(set_height_auto,  size, height, Auto, "height", Height, rect_layout_style);
// // func_value_dimension!(set_margin_auto, margin, "margin", Margin, rect_layout_style);

// // func_auto!(set_flex_basis_auto, FlexBasis::Auto);

// func_value_dimension!(set_padding,          padding, Points, "padding", "padding-top", "padding-right", "padding-bottom", "padding-left", Padding, other_layout_style);
// func_value_dimension!(set_margin,           margin, Points, "margin", "margin-top", "margin-right", "margin-bottom", "margin-left", Margin, rect_layout_style);
// func_value_dimension!(set_border,           border, Points, "border", "border-top", "border-right", "border-bottom", "border-left", Border, other_layout_style);
// func_value_dimension!(set_position,         position, Points, "position", "top", "right", "bottom", "left", Position, other_layout_style);

// func_value_dimension!(set_padding_percent,  padding, Percent, "padding", "padding-top", "padding-right", "padding-bottom", "padding-left", Padding, other_layout_style);
// func_value_dimension!(set_margin_percent,   margin, Percent, "margin", "margin-top", "margin-right", "margin-bottom", "margin-left", Margin, rect_layout_style);
// func_value_dimension!(set_position_percent, position, Percent, "position", "top", "right", "bottom", "left", Position, other_layout_style);


// #[wasm_bindgen]
// pub fn set_margin_auto(world: u32, node_id: u32, edge: u8) {
// 	let node_id = node_id as usize;
// 	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
// 	let world = &mut world.gui;
// 	let layout_styles = world.rect_layout_style.lend_mut();
	 
// 	match unsafe {transmute(edge)} {
// 		Edge::All => {
// 			layout_styles[node_id].margin = Rect{
// 				start: Dimension::Auto,
// 				end: Dimension::Auto,
// 				top: Dimension::Auto,
// 				bottom: Dimension::Auto,
// 			};
// 			world.style_mark.lend_mut()[node_id].local_style2 |= StyleType2::MarginTop as usize | StyleType2::MarginRight as usize | StyleType2::MarginLeft as usize | StyleType2::MarginBottom as usize;
// 			layout_styles.get_notify_ref().modify_event(node_id, "margin", 0);
// 		},
// 		Edge::Top => {
// 			layout_styles[node_id].margin.top = Dimension::Auto;
// 			world.style_mark.lend_mut()[node_id].local_style2 |= StyleType2::MarginTop as usize;
// 			layout_styles.get_notify_ref().modify_event(node_id, "margin-top", 0);
// 		}
// 		Edge::Right => {
// 			layout_styles[node_id].margin.end = Dimension::Auto;
// 			world.style_mark.lend_mut()[node_id].local_style2 |= StyleType2::MarginRight as usize;
// 			layout_styles.get_notify_ref().modify_event(node_id, "margin-right", 0);
// 		}
// 		Edge::Bottom => {
// 			layout_styles[node_id].margin.bottom = Dimension::Auto;
// 			world.style_mark.lend_mut()[node_id].local_style2 |= StyleType2::MarginBottom as usize;
// 			layout_styles.get_notify_ref().modify_event(node_id, "margin-bottom", 0);
// 		},
// 		Edge::Left => {
// 			layout_styles[node_id].margin.start = Dimension::Auto;
// 			world.style_mark.lend_mut()[node_id].local_style2 |= StyleType2::MarginLeft as usize;
// 			layout_styles.get_notify_ref().modify_event(node_id, "margin-left", 0);
// 		}
// 		_ => return
// 	};
// }

// // #[no_mangle]
// // #[js_export]
// // // pub fn init_height(world: u32, node_id: u32, height: u32) {
// //     let node_id = node_id as usize;
// //     let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
// //     let yogas = world.gui.yoga.lend_mut();
// //     let yoga = unsafe { yogas.get_unchecked_mut(node_id) };
// //     match yoga.get_height().unit {
// //         yoga::YGUnit::YGUnitAuto => yoga.set_height(height as f32),
// //         _ => (),
// //     }
// // }

// pub enum Edge {
//     Left = 0,
//     Top = 1,
//     Right = 2,
//     Bottom = 3,
//     Start = 4,
//     End = 5,
//     Horizontal = 6,
//     Vertical = 7,
//     All = 8,
// }
