// use ecs::LendMut;
// use flex_layout::Display;
// use wasm_bindgen::prelude::*;

// use gui::component::calc::{StyleType, StyleType1, StyleType2};
// use gui::component::user::*;
// use gui::layout::*;
// use gui::system::style_mark::{set_dirty, set_dirty1, set_dirty2};
// use crate::world::GuiWorld;

// // // 枚举样式的类型
// // #[derive(Debug)]
// // pub enum StyleType1 {
// //     // Width = 1,
// //     // Height = 2,
// //     // Margin = 4,
// //     // Padding = 8,
// //     // Border = 0x10,
// //     // Position = 0x20,
// //     // MinWidth = 0x40,
// //     // MinHeight = 0x80,
// //     // MaxHeight = 0x100,
// //     // MaxWidth = 0x200,
// //     // FlexBasis = 0x400,
// //     // FlexShrink = 0x800,
// //     // FlexGrow = 0x1000,
// //     // PositionType = 0x2000,
// //     // FlexWrap = 0x4000,
// //     // FlexDirection = 0x8000,
// //     // AlignContent = 0x10000,
// //     // AlignItems = 0x20000,
// //     // AlignSelf = 0x40000,
// // 	// JustifyContent = 0x80000,
// // 	Direction = 0x10000,
// // 	AspectRatio = 0x20000,
// // 	Order = 0x40000,
// // 	FlexBasis = 0x80000,

// //     Display = 0x100000,
// //     Visibility = 0x200000,
// //     Enable = 0x400000,
// //     ZIndex = 0x800000,
// //     Transform = 0x1000000,
// //     TransformWillChange = 0x2000000,
// // 	Overflow = 0x4000000,
	
// // 	Create = 0x8000000,
// // 	Delete = 0x10000000,

// // 	MaskImage = 0x20000000,
// // 	MaskImageClip = 0x40000000,
// // 	MaskTexture = std::isize::MIN,
// // }

// // // 枚举样式的类型
// // #[derive(Debug)]
// // pub enum StyleType {
// //     Text = 1,
// //     FontStyle = 2,
// //     FontWeight = 4,
// //     FontSize = 0x8,
// //     FontFamily = 0x10,
// //     LetterSpacing = 0x20,
// //     WordSpacing = 0x40,
// //     LineHeight = 0x80,
// //     Indent = 0x100,
// //     WhiteSpace = 0x200,
// //     TextAlign = 0x400,
// //     VerticalAlign = 0x800,
// //     Color = 0x1000,
// //     Stroke = 0x2000,
// //     TextShadow = 0x4000,

// //     Image = 0x8000,
// //     ImageClip = 0x10000,
// //     ObjectFit = 0x20000,

// //     BorderImage = 0x40000,
// //     BorderImageClip = 0x80000,
// //     BorderImageSlice = 0x100000,
// //     BorderImageRepeat = 0x200000,

// //     BorderColor = 0x400000,

// //     BackgroundColor = 0x800000,

// //     BoxShadow = 0x1000000,

// //     Matrix = 0x2000000,
// //     Opacity = 0x4000000,
// //     Layout = 0x8000000,
// //     BorderRadius = 0x10000000,
// //     ByOverflow = 0x20000000,
// // 	Filter = 0x40000000,
// // 	Oct = std::isize::MIN,
// // }

// // // 布局属性标记
// // pub enum StyleType2 {
// // 	Width = 1,
// //     Height = 2,
	
// // 	MarginTop = 4,
// // 	MarginRight = 8,
// // 	MarginBottom = 0x10,
// // 	MarginLeft = 0x20,

// // 	PaddingTop = 0x40,
// // 	PaddingRight = 0x80,
// // 	PaddingBottom = 0x100,
// // 	PaddingLeft = 0x200,

// // 	BorderTop = 0x400,
// // 	BorderRight = 0x800,
// // 	BorderBottom = 0x1000,
// // 	BorderLeft = 0x2000,

// // 	PositionTop = 0x4000,
// // 	PositionRight = 0x8000,
// // 	PositionBottom = 0x10000,
// // 	PositionLeft = 0x20000,
	
// //     MinWidth = 0x40000,
// //     MinHeight = 0x80000,
// //     MaxHeight = 0x100000,
// // 	MaxWidth = 0x200000,
// // 	JustifyContent = 0x400000,
// //     FlexShrink = 0x800000,
// // 	FlexGrow = 0x1000000,
// // 	PositionType = 0x2000000,
// //     FlexWrap = 0x4000000,
// //     FlexDirection = 0x8000000,
// //     AlignContent = 0x10000000,
// //     AlignItems = 0x20000000,
// //     AlignSelf = 0x40000000,
// // 	BlendMode = std::isize::MIN,
// // }

// #[macro_use()]
// macro_rules! reset_value_del {
// 	(
// 		@INNER,
// 		$key:ident,
// 		$component:ident,
// 		$index:expr,
// 		$ty_value:expr
// 	) => {
// 		$crate::paste::item! {
// 			#[wasm_bindgen]
// 			pub fn [<reset_ $key>](world: u32, node_id: u32){
// 				let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
// 				let world = &mut world.gui;
// 				let node_id = node_id as usize;
// 				let ty = [<StyleType $index>]::$ty_value as usize;

// 				let style_mark = unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) };
// 				if style_mark.[<local_style $index>] & ty == ty {
					
// 					// 将样式还原为默认值(样式本身是单独的一个组件，则直接删除组件)
// 					let c = world.$component.lend_mut();
// 					c.delete(node_id);
// 					c.get_notify_ref().delete_event(node_id);

// 					// 设置脏
// 					[<set_dirty $index>](&mut world.dirty_list.lend_mut(), node_id, ty, style_mark);

// 					// 取消style上，该样式的标记
// 					style_mark.[<local_style $index>] &= std::usize::MAX - ty;

// 					// 如果该样式在class中也设置了，则发出class修改的通知，以便在帧循环时，重新将class上的样式应用到节点上
// 					if style_mark.[<class_style $index>] & ty == ty {
// 						world.class_name.lend_mut().get_notify_ref().modify_event(node_id, "", 0);
// 					}
// 				}
// 			}
// 		}
// 	};
// 	(
// 		@INNER1,
// 		$key:ident,
// 		$component:ident,
// 		$field:ident,
// 		$index:expr,
// 		$ty_value:expr
// 	) => {
// 		$crate::paste::item! {
// 			#[wasm_bindgen]
// 			pub fn [<reset_ $key>](world: u32, node_id: u32){
// 				let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
// 				let world = &mut world.gui;
// 				let node_id = node_id as usize;
// 				let ty = [<StyleType $index>]::$ty_value as usize;

// 				let style_mark = unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) };
// 				if style_mark.[<local_style $index>] & ty == ty {
					
// 					// 将样式还原为默认值
// 					let c = world.$component.lend_mut();
// 					let defualt_v = c[0].$field;// 取到默认组件
// 					unsafe { c.get_unchecked_mut(node_id) }.$field = defualt_v;

// 					// 设置脏
// 					[<set_dirty $index>](&mut world.dirty_list.lend_mut(), node_id, ty, style_mark);

// 					// 取消style上，该样式的标记
// 					style_mark.[<local_style $index>] &= std::usize::MAX - ty;

// 					// 如果该样式在class中也设置了，则发出class修改的通知，以便在帧循环时，重新将class上的样式应用到节点上
// 					if style_mark.[<class_style $index>] & ty == ty {
// 						world.class_name.lend_mut().get_notify_ref().modify_event(node_id, "", 0);
// 					}
// 				}
// 			}
// 		}
// 	};
// 	(
// 		@INNER2,
// 		$key:ident,
// 		$component:ident,
// 		$field:ident,
// 		$field1:ident,
// 		$index:expr,
// 		$ty_value:expr
// 	) => {
// 		$crate::paste::item! {
// 			#[wasm_bindgen]
// 			pub fn [<reset_ $key>](world: u32, node_id: u32){
// 				let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
// 				let world = &mut world.gui;
// 				let node_id = node_id as usize;
// 				let ty = [<StyleType $index>]::$ty_value as usize;

// 				let style_mark = unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) };
// 				if style_mark.[<local_style $index>] & ty == ty {
					
// 					// 将样式还原为默认值
// 					let c = world.$component.lend_mut();
// 					let defualt_v = c[0].$field.$field1;// 取到默认组件
// 					unsafe { c.get_unchecked_mut(node_id) }.$field.$field1 = defualt_v;

// 					// 设置脏
// 					[<set_dirty $index>](&mut world.dirty_list.lend_mut(), node_id, ty, style_mark);

// 					// 取消style上，该样式的标记
// 					style_mark.[<local_style $index>] &= std::usize::MAX - ty;

// 					// 如果该样式在class中也设置了，则发出class修改的通知，以便在帧循环时，重新将class上的样式应用到节点上
// 					if style_mark.[<class_style $index>] & ty == ty {
// 						world.class_name.lend_mut().get_notify_ref().modify_event(node_id, "", 0);
// 					}
// 				}
// 			}
// 		}
// 	};
// 	(
// 		$key:ident,
// 		$component:ident,
// 		$field:ident,
// 		$field1:ident,
// 		$index:expr,
// 		$ty_value:expr
// 	) => {
// 		reset_value_del!(@INNER2, $key, $component, $field, $field1, $index, $ty_value);
// 	};
// 	(
// 		$key:ident,
// 		$component:ident,
// 		$field:ident,
// 		$field1:ident,
// 		$index:expr,
// 		$ty_value:expr,
// 	) => {
// 		reset_value_del!(@INNER2, $key, $component, $field, $field1, $index, $ty_value);
// 	};
// 	(
// 		$key:ident,
// 		$component:ident,
// 		$index:expr,
// 		$ty_value:expr
// 	) => {
// 		reset_value_del!(@INNER, $key, $component, $index, $ty_value);
// 	};
// 	(
// 		$key:ident,
// 		$component:ident,
// 		$index:expr,
// 		$ty_value:expr,
// 	) => {
// 		reset_value_del!(@INNER, $key, $component, $index, $ty_value);
// 	};
// 	(
// 		$key:ident,
// 		$component:ident,
// 		$field:ident,
// 		$index:expr,
// 		$ty_value:expr
// 	) => {
// 		reset_value_del!(@INNER, $key, $component, $index, $ty_value);
// 	};
// 	(
// 		$key:ident,
// 		$component:ident,
// 		$field:ident,
// 		$index:expr,
// 		$ty_value:expr,
// 	) => {
// 		reset_value_del!(@INNER, $key, $component, $index, $ty_value);
// 	};
//     (
// 		$key:ident,
// 		$component:ident,
// 		$index:expr,
// 		$ty_value:expr
// 	) => {
// 		reset_value_del!(@INNER, $key, $component, $component, $index, $ty_value);
// 	};
// 	(
// 		$key:ident,
// 		$component:ident,
// 		$index:expr,
// 		$ty_value:expr,
// 	) => {
// 		reset_value_del!(@INNER, $key, $component, $field, $index, $ty_value);
// 	};
// 	(
// 		$key:ident,
// 		$index:expr,
// 		$ty_value:expr
// 	) => {
// 		reset_value_del!(@INNER, $key, $key, $index, $ty_value);
// 	};
// 	(
// 		$key:ident,
// 		$index:expr,
// 		$ty_value:expr,
// 	) => {
// 		reset_value_del!(@INNER, $key, $key, $index, $ty_value);
// 	}
// }

// reset_value_del!(text_content, text_style, "", Text);
// reset_value_del!(font_style, text_style, "", FontStyle);
// reset_value_del!(font_weight, text_style, "", FontWeight);
// reset_value_del!(font_size, text_style, "", FontSize);
// reset_value_del!(font_family, text_style, "", FontFamily);
// reset_value_del!(letter_spacing, text_style, "", LetterSpacing);
// reset_value_del!(word_spacing, text_style, "", WordSpacing);
// reset_value_del!(line_height, text_style, "", LineHeight);
// reset_value_del!(indent, text_style, "", Indent);
// reset_value_del!(white_space, text_style, "", WhiteSpace);
// reset_value_del!(text_align, text_style, "", TextAlign);
// reset_value_del!(vertical_align, text_style, "", VerticalAlign);
// reset_value_del!(color, text_style, "", Color);
// reset_value_del!(stroke, text_style, "", Stroke);
// reset_value_del!(text_shadow, text_style, "", TextShadow);
// // reset_value_del!(lin, text_style, "", LineHeight);

// reset_value_del!(image, image, "", Image);
// reset_value_del!(image_clip, image_clip, "", ImageClip);
// reset_value_del!(object_fit, object_fit, "", ObjectFit);

// reset_value_del!(border_image, "", BorderImage);
// reset_value_del!(border_image_clip, "", BorderImageClip);
// reset_value_del!(border_image_slice, "", BorderImageSlice);
// reset_value_del!(border_image_repeat, "", BorderImageRepeat);

// reset_value_del!(border_color, "", BorderColor);
// reset_value_del!(border_radius, "", BorderRadius);

// reset_value_del!(background_color, "", BackgroundColor);

// reset_value_del!(box_shadow, "", BoxShadow);

// reset_value_del!(filter, "", Filter);

// reset_value_del!(opacity, "", Opacity);



// // StyleType1
// reset_value_del!(direction, other_layout_style, direction, "1", Direction);
// // reset_value_del!(aspect_ratio, rect_layout_style, AspectRatio, "1", AspectRatio);
// reset_value_del!(order, other_layout_style, order, "1", Order);
// reset_value_del!(flex_basis, other_layout_style, flex_basis, "1", FlexBasis);

// // reset_value_del!(display, other_layout_style, display, "1", Display);
// // reset_value_del!(visibility, , visibility, "1", Visibility);
// // reset_value_del!(enable, "1", Enable);
// reset_value_del!(z_index, "1", ZIndex);
// reset_value_del!(transform, "1", Transform);
// reset_value_del!(transform_will_change, "1", TransformWillChange);
// reset_value_del!(overflow, "1", Overflow);
// reset_value_del!(mask_image, "1", MaskImage);
// reset_value_del!(mask_image_clip, "1", MaskImageClip);

// // StyleType2
// reset_value_del!(width, rect_layout_style, size, width, "2", Width);
// reset_value_del!(height, rect_layout_style, size, height, "2", Height);

// reset_value_del!(margin_top, rect_layout_style, margin, top, "2", MarginTop);
// reset_value_del!(margin_right, rect_layout_style, margin, end, "2", MarginRight);
// reset_value_del!(margin_bottom, rect_layout_style, margin, bottom, "2", MarginBottom);
// reset_value_del!(margin_left, rect_layout_style, margin, start, "2", MarginLeft);

// reset_value_del!(top, other_layout_style, position, top, "2", PositionTop);
// reset_value_del!(right, other_layout_style, position, end, "2", PositionRight);
// reset_value_del!(bottom, other_layout_style, position, bottom, "2", PositionBottom);
// reset_value_del!(left, other_layout_style, position, start, "2", PositionLeft);

// reset_value_del!(padding_top, other_layout_style, padding, top, "2", PaddingTop);
// reset_value_del!(padding_right, other_layout_style, padding, end, "2", PaddingRight);
// reset_value_del!(padding_bottom, other_layout_style, padding, bottom, "2", PaddingBottom);
// reset_value_del!(padding_left, other_layout_style, padding, start, "2", PaddingLeft);

// reset_value_del!(border_top, other_layout_style, border, top, "2", BorderTop);
// reset_value_del!(border_right, other_layout_style, border, end, "2", BorderRight);
// reset_value_del!(border_bottom, other_layout_style, border, bottom, "2", BorderBottom);
// reset_value_del!(border_left, other_layout_style, border, start, "2", BorderLeft);

// reset_value_del!(min_width, other_layout_style, min_size, width, "2", MinWidth);
// reset_value_del!(min_height, other_layout_style, min_size, height, "2", MinHeight);
// reset_value_del!(max_width, other_layout_style, max_size, width, "2", MaxWidth);
// reset_value_del!(max_height, other_layout_style, max_size, height, "2", MaxHeight);

// reset_value_del!(justify_content, other_layout_style, justify_content, "2", JustifyContent);
// reset_value_del!(flex_shrink, other_layout_style, flex_shrink, "2", FlexShrink);
// reset_value_del!(flex_grow, other_layout_style, flex_grow, "2", FlexGrow);
// reset_value_del!(position_type, other_layout_style, position_type, "2", PositionType);
// reset_value_del!(flex_wrap, other_layout_style, flex_wrap, "2", FlexWrap);
// reset_value_del!(flex_direction, other_layout_style, flex_direction, "2", FlexDirection);
// reset_value_del!(align_content, other_layout_style, align_content, "2", AlignContent);
// reset_value_del!(align_items, other_layout_style, align_items, "2", AlignItems);
// reset_value_del!(align_self, other_layout_style, align_self, "2", AlignSelf);

// reset_value_del!(blend_mode, "2", BlendMode);


// /// 设置display
// #[allow(unused_attributes)]
// #[wasm_bindgen]
// pub fn reset_display(world: u32, node: u32) {
// 	let node_id = node as usize;
// 	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
// 	let world = &mut world.gui;
// 	let layouts = world
// 			.other_layout_style
// 			.lend_mut();
// 	layouts[node as usize].display = Display::Flex;

// 	unsafe { world.show.lend_mut().get_unchecked_write(node_id) }.modify(|s| {
// 		let old = s.clone();
// 		s.set_display(Display::Flex);
// 		old != *s
// 	});

// 	let ty = StyleType1::Display as usize;
// 	let style_mark = unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) };
// 	// 设置脏
// 	set_dirty1(&mut world.dirty_list.lend_mut(), node_id, ty, style_mark);

// 	// 取消style上，该样式的标记
// 	style_mark.local_style1 &= std::usize::MAX - ty;

// 	// 如果该样式在class中也设置了，则发出class修改的通知，以便在帧循环时，重新将class上的样式应用到节点上
// 	if style_mark.class_style1 & ty == ty {
// 		world.class_name.lend_mut().get_notify_ref().modify_event(node_id, "", 0);
// 	}
// }

// /// 设置visibility, true: visible, false: hidden,	默认true
// #[allow(unused_attributes)]
// #[wasm_bindgen]
// pub fn reset_visibility(world: u32, node: u32) {
//     let node_id = node as usize;
// 	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
// 	let world = &mut world.gui;

// 	unsafe { world.show.lend_mut().get_unchecked_write(node_id) }.modify(|s| {
// 		let old = s.clone();
// 		s.set_visibility(true);
// 		old != *s
// 	});

// 	let ty = StyleType1::Visibility as usize;
// 	let style_mark = unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) };
// 	// 设置脏
// 	set_dirty1(&mut world.dirty_list.lend_mut(), node_id, ty, style_mark);

// 	// 取消style上，该样式的标记
// 	style_mark.local_style1 &= std::usize::MAX - ty;

// 	// 如果该样式在class中也设置了，则发出class修改的通知，以便在帧循环时，重新将class上的样式应用到节点上
// 	if style_mark.class_style1 & ty == ty {
// 		world.class_name.lend_mut().get_notify_ref().modify_event(node_id, "", 0);
// 	}
// }

// /// 设置enable
// #[allow(unused_attributes)]
// #[wasm_bindgen]
// pub fn reset_enable(world: u32, node: u32) {
//     let node_id = node as usize;
// 	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
// 	let world = &mut world.gui;

// 	unsafe { world.show.lend_mut().get_unchecked_write(node_id) }.modify(|s| {
// 		let old = s.clone();
// 		s.set_enable(EnableType::Auto);
// 		old != *s
// 	});

// 	let ty = StyleType1::Enable as usize;
// 	let style_mark = unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) };
// 	// 设置脏
// 	set_dirty1(&mut world.dirty_list.lend_mut(), node_id, ty, style_mark);

// 	// 取消style上，该样式的标记
// 	style_mark.local_style1 &= std::usize::MAX - ty; 

// 	// 如果该样式在class中也设置了，则发出class修改的通知，以便在帧循环时，重新将class上的样式应用到节点上
// 	if style_mark.class_style1 & ty == ty {
// 		world.class_name.lend_mut().get_notify_ref().modify_event(node_id, "", 0);
// 	}
// }

// // $crate::paste::item! {
// // 	#[allow(unused_attributes)]
// // 	#[wasm_bindgen]
// // 	pub fn [<set_ $func>]

// // #[macro_use()]
// // macro_rules! delete_value {
// //     (
// // 		$world:ident, 
// // 		$node_id:ident, 
// // 		$key:ident, 
// // 		$csy:ident,/*class_style | class_style1*/ 
// // 		$lsy:ident,/*local_style | local_style1*/ 
// // 		$lsyty:expr/*StyleType::XX |*/) => {
// //         let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
// //         let world = &mut world.gui;
// //         let node_id = $node_id as usize;

// //         let ty = $lsyty as usize;
// //         let style_mark = unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) };
// //         if (style_mark.$lsy & ty) == ty {
// //             style_mark.$lsy &= std::usize::MAX - ty;
// //             world.$key.lend_mut().delete(node_id);
// //         }
// //     };
// // }

// // #[macro_use()]
// // macro_rules! reset_value {
// //     (
// // 		$world:ident,
// // 		$node_id:ident,
// // 		$key:ident,
// // 		$value:expr,
// // 		$csy:ident,/*class_style | class_style1*/
// // 		$lsy:ident,/*local_style | local_style1*/
// // 		$lsyty:expr/*StyleType::XX |*/) => {
// //         let node_id = $node_id as usize;
// //         let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
// //         let world = &mut world.gui;

// //         let ty = $lsyty as usize;
// //         let style_mark = unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) };
// //         if (style_mark.$lsy & ty) == ty {
// //             style_mark.$lsy &= std::usize::MAX - ty;
// //             world.$key.lend_mut().insert(node_id, $value);
// //         }
// //     };
// // }

// // #[macro_use()]
// // macro_rules! reset_text_attr {
// //     (
// // 		$world:ident,
// // 		$node_id:ident,
// // 		$key:ident,
// // 		$csy:ident,/*class_style | class_style1*/ 
// // 		$lsy:ident, /*local_style | local_style1*/
// // 		$lsyty:expr, /*StyleType::XX |*/
// // 		$name:ident,
// // 		$name1:ident,
// // 		$name2:expr,
// // 		$value:expr,
// // 	) => {
// //         let node_id = $node_id as usize;
// //         let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
// //         let world = &mut world.gui;
// //         let attr = world.$key.lend_mut();
// //         let v = $value;
// //         let ty = $lsyty as usize;

// //         let style_mark = unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) };
// //         if style_mark.$lsy & ty == ty {
// //             $crate::paste::item! {
// //                 style_mark.$lsy &= std::usize::MAX - ty;
// //                 let r = unsafe { attr.get_unchecked_mut(node_id) };
// //                 r.$name.$name1 = v;
// //                 attr.get_notify_ref().modify_event(node_id, $name2, 0);
// //             }
// //         }
// //     };
// // }

// // /// 设置transform_will_change
// // #[allow(unused_attributes)]
// // #[no_mangle]
// // #[js_export]
// // // pub fn reset_style(world: u32, node_id: u32, ty: u32) {
// //     let defult_text = unsafe { &mut *(world as usize as *mut GuiWorld) }
// //         .default_text_style
// //         .clone();
// //     // let node_id = node_id as usize;

// //     let ty: StyleAttr = unsafe { std::mem::transmute_copy(&ty) };
// //     let undefined = std::f32::NAN;
// //     match ty {
// //         // StyleAttr::Text => (),//reset_text_attr!(world, node_id, text_style, class_style,local_style, StyleType::Text, text,),
// //         StyleAttr::FontStyle => {
// //             let v = defult_text.font.style;
// //             reset_text_attr!(
// //                 world,
// //                 node_id,
// //                 text_style,
// //                 class_style,
// //                 local_style,
// //                 StyleType::FontStyle,
// //                 font,
// //                 style,
// //                 "font_style",
// //                 v,
// //             );
// //         }
// //         StyleAttr::FontWeight => {
// //             let v = defult_text.font.weight;
// //             reset_text_attr!(
// //                 world,
// //                 node_id,
// //                 text_style,
// //                 class_style,
// //                 local_style,
// //                 StyleType::FontWeight,
// //                 font,
// //                 weight,
// //                 "font_weight",
// //                 v,
// //             );
// //         }
// //         StyleAttr::FontSize => {
// //             let v = defult_text.font.size;
// //             reset_text_attr!(
// //                 world,
// //                 node_id,
// //                 text_style,
// //                 class_style,
// //                 local_style,
// //                 StyleType::FontSize,
// //                 font,
// //                 size,
// //                 "font_size",
// //                 v,
// //             );
// //         }
// //         StyleAttr::FontFamily => {
// //             let v = defult_text.font.family;
// //             reset_text_attr!(
// //                 world,
// //                 node_id,
// //                 text_style,
// //                 class_style,
// //                 local_style,
// //                 StyleType::FontFamily,
// //                 font,
// //                 family,
// //                 "font_family",
// //                 v,
// //             );
// //         }
// //         StyleAttr::LetterSpacing => {
// //             let v = defult_text.text.letter_spacing;
// //             reset_text_attr!(
// //                 world,
// //                 node_id,
// //                 text_style,
// //                 class_style,
// //                 local_style,
// //                 StyleType::LetterSpacing,
// //                 text,
// //                 letter_spacing,
// //                 "letter_spacing",
// //                 v,
// //             );
// //         }
// //         StyleAttr::WordSpacing => {
// //             let v = defult_text.text.word_spacing;
// //             reset_text_attr!(
// //                 world,
// //                 node_id,
// //                 text_style,
// //                 class_style,
// //                 local_style,
// //                 StyleType::WordSpacing,
// //                 text,
// //                 word_spacing,
// //                 "word_spacing",
// //                 v,
// //             );
// //         }
// //         StyleAttr::LineHeight => {
// //             let v = defult_text.text.line_height;
// //             reset_text_attr!(
// //                 world,
// //                 node_id,
// //                 text_style,
// //                 class_style,
// //                 local_style,
// //                 StyleType::LineHeight,
// //                 text,
// //                 line_height,
// //                 "line_height",
// //                 v,
// //             );
// //         }
// //         StyleAttr::Indent => {
// //             let v = defult_text.text.indent;
// //             reset_text_attr!(
// //                 world,
// //                 node_id,
// //                 text_style,
// //                 class_style,
// //                 local_style,
// //                 StyleType::Indent,
// //                 text,
// //                 indent,
// //                 "indent",
// //                 v,
// //             );
// //         }
// //         StyleAttr::WhiteSpace => {
// //             let v = defult_text.text.white_space;
// //             reset_text_attr!(
// //                 world,
// //                 node_id,
// //                 text_style,
// //                 class_style,
// //                 local_style,
// //                 StyleType::WhiteSpace,
// //                 text,
// //                 white_space,
// //                 "white_space",
// //                 v,
// //             );
// //         }
// //         StyleAttr::TextAlign => {
// //             let v = defult_text.text.text_align;
// //             reset_text_attr!(
// //                 world,
// //                 node_id,
// //                 text_style,
// //                 class_style,
// //                 local_style,
// //                 StyleType::TextAlign,
// //                 text,
// //                 text_align,
// //                 "text_align",
// //                 v,
// //             );
// //         }
// //         StyleAttr::VerticalAlign => {
// //             let v = defult_text.text.vertical_align;
// //             reset_text_attr!(
// //                 world,
// //                 node_id,
// //                 text_style,
// //                 class_style,
// //                 local_style,
// //                 StyleType::VerticalAlign,
// //                 text,
// //                 vertical_align,
// //                 "vertical_align",
// //                 v,
// //             );
// //         }
// //         StyleAttr::Color => {
// //             let v = defult_text.text.color;
// //             reset_text_attr!(
// //                 world,
// //                 node_id,
// //                 text_style,
// //                 class_style,
// //                 local_style,
// //                 StyleType::Color,
// //                 text,
// //                 color,
// //                 "color",
// //                 v,
// //             );
// //         }
// //         StyleAttr::Stroke => {
// //             let v = defult_text.text.stroke;
// //             reset_text_attr!(
// //                 world,
// //                 node_id,
// //                 text_style,
// //                 class_style,
// //                 local_style,
// //                 StyleType::Stroke,
// //                 text,
// //                 stroke,
// //                 "stroke",
// //                 v,
// //             );
// //         }
// //         // StyleAttr::TextShadow => (),//reset_text_attr!(world, node_id, text_style, class_style,local_style, StyleType::Text, font, font_style, "font_style",defult_text.shadow.stroke),
// //         StyleAttr::Image => {
// //             let v = StyleType::Image;
// //             delete_value!(world, node_id, image, class_style, local_style, v);
// //         }
// //         StyleAttr::ImageClip => {
// //             let v = StyleType::ImageClip;
// //             delete_value!(world, node_id, image_clip, class_style, local_style, v);
// //         }
// //         StyleAttr::ObjectFit => {
// //             let v = StyleType::ObjectFit;
// //             delete_value!(world, node_id, object_fit, class_style, local_style, v);
// //         }

// //         StyleAttr::BorderImage => {
// //             let v = StyleType::BorderImage;
// //             delete_value!(world, node_id, border_image, class_style, local_style, v);
// //         }
// //         StyleAttr::BorderImageClip => {
// //             let v = StyleType::BorderImageClip;
// //             delete_value!(
// //                 world,
// //                 node_id,
// //                 border_image_clip,
// //                 class_style,
// //                 local_style,
// //                 v
// //             );
// //         }
// //         StyleAttr::BorderImageSlice => {
// //             let v = StyleType::BorderImageSlice;
// //             delete_value!(
// //                 world,
// //                 node_id,
// //                 border_image_slice,
// //                 class_style,
// //                 local_style,
// //                 v
// //             );
// //         }
// //         StyleAttr::BorderImageRepeat => {
// //             let v = StyleType::BorderImageRepeat;
// //             delete_value!(
// //                 world,
// //                 node_id,
// //                 border_image_repeat,
// //                 class_style,
// //                 local_style,
// //                 v
// //             );
// //         }

// //         StyleAttr::BorderColor => {
// //             let v = StyleType::BorderColor;
// //             delete_value!(world, node_id, border_color, class_style, local_style, v);
// //         }

// //         StyleAttr::BackgroundColor => {
// //             let v = StyleType::BackgroundColor;
// //             delete_value!(
// //                 world,
// //                 node_id,
// //                 background_color,
// //                 class_style,
// //                 local_style,
// //                 v
// //             );
// //         }

// //         StyleAttr::BoxShadow => {
// //             let v = StyleType::BoxShadow;
// //             delete_value!(world, node_id, box_shadow, class_style, local_style, v);
// //         }
// //         // StyleType::Matrix => 0x2000000,
// //         StyleAttr::Opacity => {
// //             let v = StyleType::Opacity;
// //             reset_value!(
// //                 world,
// //                 node_id,
// //                 opacity,
// //                 Opacity::default(),
// //                 class_style1,
// //                 local_style1,
// //                 v
// //             );
// //         }
// //         // StyleType::Layout => 0x8000000,
// //         StyleAttr::BorderRadius => {
// //             let v = StyleType::BorderRadius;
// //             reset_value!(
// //                 world,
// //                 node_id,
// //                 border_radius,
// //                 BorderRadius {
// //                     x: LengthUnit::Pixel(0.0),
// //                     y: LengthUnit::Pixel(0.0),
// //                 },
// //                 class_style,
// //                 local_style,
// //                 v
// //             );
// //         }
// //         // StyleType::ByOverflow => 0x20000000,
// //         StyleAttr::Filter => {
// //             let v = StyleType::Filter;
// //             reset_value!(
// //                 world,
// //                 node_id,
// //                 filter,
// //                 Filter::default(),
// //                 class_style,
// //                 local_style,
// //                 v
// //             );
// //         }

// //         StyleAttr::Width => reset_layout(world, node_id, StyleType1::Width, |n| {
// //             n.set_width(undefined);
// //         }),
// //         StyleAttr::Height => reset_layout(world, node_id, StyleType1::Height, |n| {
// //             n.set_height(undefined);
// //         }),

// //         StyleAttr::Margin => reset_layout(world, node_id, StyleType1::Margin, |n| {
// //             n.set_margin(YGEdge::YGEdgeAll, undefined);
// //         }),
// //         // StyleAttr::MarginTop => reset_layout(world, node_id, StyleType1::Margin, |n|{n.set_margin(YGEdge::YGEdgeTop, undefined)}),
// //         // StyleAttr::MarginRight => reset_layout(world, node_id, StyleType1::Margin, |n|{n.set_margin(YGEdge::YGEdgeTop, undefined)}),
// //         // StyleAttr::MarginBottom => reset_layout(world, node_id, StyleType1::Margin, |n|{n.set_margin(YGEdge::YGEdgeTop, undefined)}),
// //         // StyleAttr::MarginLeft => reset_layout(world, node_id, StyleType1::Margin, |n|{n.set_margin(YGEdge::YGEdgeTop, undefined)}),
// //         StyleAttr::Padding => reset_layout(world, node_id, StyleType1::Padding, |n| {
// //             n.set_padding(YGEdge::YGEdgeAll, undefined);
// //         }),
// //         // StyleAttr::PaddingTop => reset_layout(world, node_id, StyleType1::Padding, |n|{n.set_padding(YGEdge::YGEdgeTop, undefined)}),
// //         // StyleAttr::PaddingRight => reset_layout(world, node_id, StyleType1::Padding, |n|{n.set_padding(YGEdge::YGEdgeRight, undefined)}),
// //         // StyleAttr::PaddingBottom => reset_layout(world, node_id, StyleType1::Padding, |n|{n.set_padding(YGEdge::YGEdgeBootom, undefined)}),
// //         // StyleAttr::PaddingLeft => reset_layout(world, node_id, StyleType1::Padding, |n|{n.set_padding(YGEdge::YGEdgeLeft, undefined)}),
// //         StyleAttr::Border => reset_layout(world, node_id, StyleType1::Border, |n| {
// //             n.set_border(YGEdge::YGEdgeAll, undefined);
// //         }),
// //         // StyleAttr::BorderTop => reset_layout(world, node_id, StyleType1::Position, |n|{n.set_border(YGEdge::YGEdgeTop, undefined)}),
// //         // StyleAttr::BorderRight => reset_layout(world, node_id, StyleType1::Position, |n|{n.set_border(YGEdge::YGEdgeRight, undefined)}),
// //         // StyleAttr::BorderBottom => reset_layout(world, node_id, StyleType1::Position, |n|{n.set_border(YGEdge::YGEdgeBootom, undefined)}),
// //         // StyleAttr::BorderLeft => reset_layout(world, node_id, StyleType1::Position, |n|{n.set_border(YGEdge::YGEdgeLeft, undefined)}),
// //         StyleAttr::Top => reset_layout(world, node_id, StyleType1::Position, |n| {
// //             n.set_position(YGEdge::YGEdgeTop, undefined);
// //         }),
// //         StyleAttr::Right => reset_layout(world, node_id, StyleType1::Position, |n| {
// //             n.set_position(YGEdge::YGEdgeRight, undefined);
// //         }),
// //         StyleAttr::Bottom => reset_layout(world, node_id, StyleType1::Position, |n| {
// //             n.set_position(YGEdge::YGEdgeBottom, undefined);
// //         }),
// //         StyleAttr::Left => reset_layout(world, node_id, StyleType1::Position, |n| {
// //             n.set_position(YGEdge::YGEdgeLeft, undefined);
// //         }),

// //         StyleAttr::MinWidth => reset_layout(world, node_id, StyleType1::MinWidth, |n| {
// //             n.set_min_width(undefined);
// //         }),
// //         StyleAttr::MinHeight => reset_layout(world, node_id, StyleType1::MinHeight, |n| {
// //             n.set_min_height(undefined);
// //         }),
// //         StyleAttr::MaxHeight => reset_layout(world, node_id, StyleType1::MaxHeight, |n| {
// //             n.set_max_height(undefined)
// //         }),
// //         StyleAttr::MaxWidth => reset_layout(world, node_id, StyleType1::MaxWidth, |n| {
// //             n.set_max_width(undefined);
// //         }),

// //         StyleAttr::FlexBasis => reset_layout(world, node_id, StyleType1::FlexBasis, |n| {
// //             n.set_flex_basis_auto();
// //         }),
// //         StyleAttr::FlexShrink => reset_layout(world, node_id, StyleType1::FlexShrink, |n| {
// //             n.set_flex_shrink(0.0);
// //         }),
// //         StyleAttr::FlexGrow => reset_layout(world, node_id, StyleType1::FlexGrow, |n| {
// //             n.set_flex_grow(0.0)
// //         }),
// //         StyleAttr::PositionType => reset_layout(world, node_id, StyleType1::PositionType, |n| {
// //             n.set_position_type(YGPositionType::YGPositionTypeRelative);
// //         }),
// //         StyleAttr::FlexWrap => reset_layout(world, node_id, StyleType1::FlexWrap, |n| {
// //             n.set_flex_wrap(YGWrap::YGWrapWrap);
// //         }),
// //         StyleAttr::FlexDirection => reset_layout(world, node_id, StyleType1::FlexDirection, |n| {
// //             n.set_flex_direction(YGFlexDirection::YGFlexDirectionRow);
// //         }),
// //         StyleAttr::AlignContent => reset_layout(world, node_id, StyleType1::AlignContent, |n| {
// //             n.set_align_content(YGAlign::YGAlignFlexStart);
// //         }),
// //         StyleAttr::AlignItems => reset_layout(world, node_id, StyleType1::AlignItems, |n| {
// //             n.set_align_items(YGAlign::YGAlignFlexStart);
// //         }),
// //         StyleAttr::AlignSelf => reset_layout(world, node_id, StyleType1::AlignSelf, |n| {
// //             n.set_align_self(YGAlign::YGAlignFlexStart);
// //         }),
// //         StyleAttr::JustifyContent => {
// //             reset_layout(world, node_id, StyleType1::JustifyContent, |n| {
// //                 n.set_justify_content(YGJustify::YGJustifyFlexStart);
// //             })
// //         }

// //         // StyleAttr::Display => reset_layout(world, node_id, StyleType1::Position, |n|{n.set_width()}),
// //         // StyleAttr::Visibility => reset_value!(world, text_style, Visibility::default(), class_style1,local_style1, StyleType1::Visibility, visibility),
// //         // StyleAttr::Enable => reset_value!(world, text_style, Enable::default(), class_style1,local_style1, StyleType1::Enable, enable),
// //         StyleAttr::ZIndex => {
// //             let v = StyleType1::ZIndex;
// //             reset_value!(
// //                 world,
// //                 node_id,
// //                 z_index,
// //                 ZIndex::default(),
// //                 class_style1,
// //                 local_style1,
// //                 v
// //             );
// //         }
// //         StyleAttr::Transform => {
// //             let v = StyleType1::Transform;
// //             reset_value!(
// //                 world,
// //                 node_id,
// //                 transform,
// //                 Transform::default(),
// //                 class_style1,
// //                 local_style1,
// //                 v
// //             );
// //         }
// //         // StyleAttr::TransformWillChange => reset_value!(world, text_style, Filter::default(), class_style,local_style, StyleType::Filter, filter),
// //         StyleAttr::Overflow => {
// //             let v = StyleType1::Overflow;
// //             reset_value!(
// //                 world,
// //                 node_id,
// //                 overflow,
// //                 Overflow::default(),
// //                 class_style1,
// //                 local_style1,
// //                 v
// //             );
// //         }
// //         _ => (),
// //     }
// // }

// // fn reset_layout<T: FnOnce(&mut YgNode)>(world: u32, node_id: u32, ty: StyleType1, call_back: T) {
// //     // let edge = unsafe { transmute(edge) };
// //     let node_id = node_id as usize;
// //     let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
// //     let world = &mut world.gui;

// //     let ty = ty as usize;
// //     if (unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) }.local_style1 & ty) == ty {
// //         unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) }.local_style1 &=
// //             std::usize::MAX - ty;
// //         unsafe { world.yoga.lend_mut().get_unchecked_write(node_id) }.modify(|s| {
// //             call_back(s);
// //             true
// //         })
// //     }
// // }

// // enum StyleAttr {
// //     FontStyle,
// //     FontWeight,
// //     FontSize,
// //     FontFamily,
// //     LetterSpacing,
// //     WordSpacing,
// //     LineHeight,
// //     Indent,
// //     WhiteSpace,
// //     TextAlign,
// //     VerticalAlign,
// //     Color,
// //     Stroke,
// //     TextShadow,

// //     BackgroundColor,
// //     Image,
// //     ImageClip,
// //     ObjectFit,
// //     BorderImage,
// //     BorderImageClip,
// //     BorderImageSlice,
// //     BorderImageRepeat,
// //     BorderColor,
// //     BoxShadow,

// //     Opacity,
// //     BorderRadius,
// //     Filter,

// //     Width,
// //     Height,

// //     Margin,
// //     MarginTop,
// //     MarginRight,
// //     MarginBottom,
// //     MarginLeft,

// //     Padding,
// //     PaddingTop,
// //     PaddingRight,
// //     PaddingBottom,
// //     PaddingLeft,

// //     Border,
// //     BorderTop,
// //     BorderRight,
// //     BorderBottom,
// //     BorderLeft,

// //     Top,
// //     Right,
// //     Bottom,
// //     Left,

// //     MinWidth,
// //     MinHeight,

// //     MaxWidth,
// //     MaxHeight,

// //     FlexBasis,
// //     FlexShrink,
// //     FlexGrow,
// //     PositionType,
// //     FlexWrap,
// //     FlexDirection,
// //     AlignContent,
// //     AlignItems,
// //     AlignSelf,
// //     JustifyContent,

// //     Display,
// //     Visibility,
// //     Enable,
// //     Overflow,
// //     ZIndex,
// //     Transform,
// //     TransformWillChange,
// // }
