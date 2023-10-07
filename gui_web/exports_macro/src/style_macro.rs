//! 将设置布局属性的接口导出到js


use std::mem::transmute;

// use pi_null::Null;
use gui::component::user::ClassName;
use ordered_float::NotNan;
use pi_flex_layout::prelude::*;
use hash::XHashMap;
use map::vecmap::VecMap;
use pi_style::style::*;
use pi_style::style_type::*;
use pi_style::style_parse::{parse_comma_separated, parse_text_shadow, parse_as_image, StyleParse};
use smallvec::SmallVec;
pub use crate::index::{OffsetDocument, Size, GuiWorld, Atom};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
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

#[macro_export]
macro_rules! style_out_export {
	(@dimension_box $attr_name:ident, $last_ty: ident) => {
		$crate::paste::item! {
			style_out_export!(@dimension_inner  [<$attr_name _percent>], $last_ty, Dimension::Percent(v),; v: f32, );
			style_out_export!(@dimension_inner $attr_name, $last_ty, Dimension::Points(v),; v: f32, );
			style_out_export!(@dimension_inner  [<$attr_name _auto>], $last_ty, Dimension::Auto,; );
		}
	};

	(@dimension $attr_name:ident, $last_ty: ident) => {
		$crate::paste::item! {
			style_out_export!(@expr  [<$attr_name _percent>], $last_ty, Dimension::Percent(v),; v: f32, );
			style_out_export!(@expr $attr_name, $last_ty, Dimension::Points(v),; v: f32, );
			style_out_export!(@expr  [<$attr_name _auto>], $last_ty, Dimension::Auto,; );
		}
	};

	(@cenum $attr_name:ident, $last_ty: ident) => {
		style_out_export!(@expr $attr_name, $last_ty, unsafe {transmute(v as u8)},; v: f64,);
	};

	(@expr $attr_name:ident, $last_ty: ident, $expr:expr, $($name_ref: ident: &$ty_ref: ty,)*; $($name: ident: $ty: ty,)*) => {
		$crate::paste::item! {

			#[cfg(target_arch="wasm32")]
			#[wasm_bindgen]
			#[allow(unused_attributes)]
			pub fn [<set_ $attr_name>](gui: u32, node_id: f64, $($name_ref: &$ty_ref,)* $($name: $ty,)*) {
				let node_id = node_id as usize;
				let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
				gui.gui.set_style(node_id, $last_ty($expr));
			}

			#[cfg(target_arch="wasm32")]
			#[wasm_bindgen]
			#[allow(unused_attributes)]
			pub fn [<reset_ $attr_name>](gui: u32, node_id: f64) {
				let node_id = node_id as usize;
				let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
				gui.gui.set_style(node_id, [<Reset $last_ty>]);
			}

		}
    };

	(@owner $attr_name:ident, $expr:expr, $resetexpr:expr, $($name_ref: ident: &$ty_ref: ty,)*; $($name: ident: $ty: ty,)*) => {
		$crate::paste::item! {

			#[cfg(target_arch="wasm32")]
			#[wasm_bindgen]
			#[allow(unused_attributes)]
			pub fn [<set_ $attr_name>](gui: u32, node_id: f64, $($name_ref: &$ty_ref,)* $($name: $ty,)*) {
				let node_id = node_id as usize;
				let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
				$expr;
			}

			#[cfg(target_arch="wasm32")]
			#[wasm_bindgen]
			#[allow(unused_attributes)]
			pub fn [<reset_ $attr_name>](gui: u32, node_id: f64) {
				let node_id = node_id as usize;
				let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
				$resetexpr;
			}
		}
	};

	(@dimension_inner $attr_name:ident, $last_ty: ident, $expr: expr, $($name_ref: ident: &$ty_ref: ty,)*; $($name: ident: $ty: ty,)*) => {
		$crate::paste::item! {

			#[cfg(target_arch="wasm32")]
			#[wasm_bindgen]
			#[allow(unused_attributes)]
			pub fn [<set_ $attr_name>](gui: u32, node_id: f64, edge: f64, $($name_ref: &$ty_ref,)* $($name: $ty,)*) {
				let node_id = node_id as usize;
				let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
				match unsafe {transmute(edge as u8)} {
					// Edge::All => gui.gui.set_style(node_id, [<$last_ty Type>]($last_ty(Rect {
					// 	top: $expr,
					// 	right: $expr,
					// 	bottom: $expr,
					// 	left: $expr,
					// }))),
					Edge::Top => gui.gui.set_style(node_id, [<$last_ty TopType>]($expr)),
					Edge::Right => gui.gui.set_style(node_id, [<$last_ty RightType>]($expr)),
					Edge::Bottom => gui.gui.set_style(node_id, [<$last_ty BottomType>]($expr)),
					Edge::Left => gui.gui.set_style(node_id, [<$last_ty LeftType>]($expr)),
					_ => return
				};
			}

			#[cfg(target_arch="wasm32")]
			#[wasm_bindgen]
			#[allow(unused_attributes)]
			pub fn [<reset_ $attr_name>](gui: u32, node_id: f64, edge: f64) {
				let node_id = node_id as usize;
				let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
				match unsafe {transmute(edge as u8)} {
					// Edge::All => gui.gui.set_style(node_id, [<Reset $last_ty Type>]),
					Edge::Top => gui.gui.set_style(node_id, [<Reset $last_ty TopType>]),
					Edge::Right => gui.gui.set_style(node_id, [<Reset $last_ty RightType>]),
					Edge::Bottom => gui.gui.set_style(node_id, [<Reset $last_ty BottomType>]),
					Edge::Left => gui.gui.set_style(node_id, [<Reset $last_ty LeftType>]),
					_ => return
				};
			}

		}
	};

	(@atom $attr_name:ident, $last_ty: ident, $expr:expr, $($name_ref: ident: &$ty_ref: ty,)*; $($name: ident: $ty: ty,)*) => {
		$crate::paste::item! {

			#[cfg(target_arch="wasm32")]
			#[wasm_bindgen]
			#[allow(unused_attributes)]
			pub fn [<set_ $attr_name>](gui: u32, node_id: f64, $($name_ref: &$ty_ref,)* $($name: $ty,)*) {
				let node_id = node_id as usize;
				let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
				gui.gui.set_style(node_id, $last_ty($expr));
			}

			#[cfg(target_arch="wasm32")]
			#[wasm_bindgen]
			#[allow(unused_attributes)]
			pub fn [<reset_ $attr_name>](gui: u32, node_id: f64) {
				let node_id = node_id as usize;
				let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
				gui.gui.set_style(node_id, [<Reset $last_ty>]);
			}
		}
    };
}

style_out_export!(@cenum align_content, AlignContentType);
style_out_export!(@cenum align_items, AlignItemsType);
style_out_export!(@cenum justify_content, JustifyContentType);
style_out_export!(@cenum flex_direction, FlexDirectionType);
style_out_export!(@cenum flex_wrap, FlexWrapType);
style_out_export!(@cenum align_self, AlignSelfType);
style_out_export!(@cenum position_type, PositionTypeType);

style_out_export!(@expr flex_grow, FlexGrowType, v, ; v: f32,);
style_out_export!(@expr flex_shrink, FlexGrowType, v, ; v: f32,);

style_out_export!(@dimension flex_basis, FlexBasisType);
style_out_export!(@dimension width, WidthType);
style_out_export!(@dimension height, HeightType);
style_out_export!(@dimension min_width, MinWidthType);
style_out_export!(@dimension min_height, MinHeightType);
style_out_export!(@dimension max_width, MaxWidthType);
style_out_export!(@dimension max_height, MaxHeightType);

style_out_export!(@dimension_box padding, Padding);
style_out_export!(@dimension_box margin, Margin);
style_out_export!(@dimension_box border, Border);
style_out_export!(@dimension_box position, Position);

style_out_export!(@expr background_rgba_color, BackgroundColorType, Color::RGBA(CgColor::new(r, g, b, a)), ; r: f32, g: f32, b: f32, a: f32,);
style_out_export!(@expr 
	background_linear_color, 
	BackgroundColorType, 
	Color::LinearGradient(to_linear_gradient_color(
        color_and_positions.as_slice(),
        direction,
    )), ;
	direction: f32, color_and_positions: Vec<f32>,);

style_out_export!(@expr 
	border_color,
	BorderColorType,
	CgColor::new(r, g, b, a),;
	r: f32, g: f32, b: f32, a: f32,);

style_out_export!(@expr
	border_radius,
	BorderRadiusType,
	{
		let mut input = cssparser::ParserInput::new(s);
		let mut parse = cssparser::Parser::new(&mut input);

		let border_radius = pi_style::style_parse::parse_border_radius(&mut parse);
		if let Ok(value) = border_radius {
			value
		} else {
			Default::default()
		}
	},
	s: &str,; );

style_out_export!(@expr 
	box_shadow,
	BoxShadowType,
	BoxShadow {
		h: h,
		v: v,
		blur: blur,
		spread: spread,
		color: CgColor::new(r, g, b, a)
	},;
	h: f32, v: f32, blur: f32, spread: f32, r: f32, g: f32 ,b: f32, a: f32,);
style_out_export!(@cenum object_fit, ObjectFitType);

style_out_export!(@expr background_repeat, BackgroundRepeatType, ImageRepeat {
		x: unsafe { transmute(x as u8) },
		y: unsafe { transmute(y as u8) },
	},;
	x: u8, y: u8, );

style_out_export!(@expr 
	mask_image_linear,
	MaskImageType,
	MaskImage::LinearGradient(to_linear_gradient_color(
        color_and_positions.as_slice(),
        direction,
    )),;
	direction: f32, color_and_positions: Vec<f32>, );

style_out_export!(@expr 
	image_clip,
	BackgroundImageClipType,
	NotNanRect::new(
		unsafe { NotNan::new_unchecked(v1) },
		unsafe { NotNan::new_unchecked(u2) },
		unsafe { NotNan::new_unchecked(v2) },
		unsafe { NotNan::new_unchecked(u1) },
	),;
	u1: f32, v1: f32, u2: f32, v2: f32,);

style_out_export!(@expr 
	mask_image_clip,
	MaskImageClipType,
	NotNanRect::new(
		unsafe { NotNan::new_unchecked(v1) },
		unsafe { NotNan::new_unchecked(u2) },
		unsafe { NotNan::new_unchecked(v2) },
		unsafe { NotNan::new_unchecked(u1) },
	),;
	u1: f32, v1: f32, u2: f32, v2: f32,);

style_out_export!(@expr 
	border_image_clip,
	BorderImageClipType,
	NotNanRect::new(
		unsafe { NotNan::new_unchecked(v1) },
		unsafe { NotNan::new_unchecked(u2) },
		unsafe { NotNan::new_unchecked(v2) },
		unsafe { NotNan::new_unchecked(u1) },
	),;
	u1: f32, v1: f32, u2: f32, v2: f32,);

style_out_export!(@expr 
	border_image_slice,
	BorderImageSliceType,
	BorderImageSlice {
		top: unsafe { NotNan::new_unchecked(top) },
		right: unsafe { NotNan::new_unchecked(right) },
		bottom: unsafe { NotNan::new_unchecked(bottom) },
		left: unsafe { NotNan::new_unchecked(left) },
		fill,
	},;
	top: f32, right: f32, bottom: f32, left: f32, fill: bool,);

style_out_export!(@expr 
	border_image_repeat,
	BorderImageRepeatType,
	ImageRepeat {
		x: unsafe { transmute(vertical as u8) },
		y: unsafe { transmute(horizontal as u8) },
	},;
	vertical: u8, horizontal: u8, );

style_out_export!(@expr overflow, OverflowType, v,; v: bool,);
style_out_export!(@expr opacity, OpacityType, v,; v: f32,);
style_out_export!(@cenum display, DisplayType);
style_out_export!(@expr visibility, VisibilityType, v,; v: bool,);
style_out_export!(@cenum enable, EnableType);
style_out_export!(@cenum blend_mode, BlendModeType);
style_out_export!(@expr zindex, ZIndexType, v as isize,; v: i32,);
// style_out_export!(@expr as_image, AsImageType, {
// 	let mut input = cssparser::ParserInput::new(value);
//     let mut parse = cssparser::Parser::new(&mut input);

//     match parse_as_image(&mut parse) {
//         Ok(r) => r,
//         Err(e) => {
//             log::error!("set_as_image fail, str: {}, err: {:?}", value, e);
//             return;
//         }
//     }
// }, value: &str,;);

// style_out_export!(@expr as_image, AsImageType, unsafe{transmute(v)},; v: u8,);
style_out_export!(@expr filter_blur, BlurType, v,; v: f32,);
style_out_export!(@expr transform_will_change, TransformWillChangeType, v,; v: bool,);

// hsi, 效果与ps一致,  h: -180 ~ 180, s: -100 ~ 100, i: -100 ~ 100
style_out_export!(@expr 
	filter_hsi,
	HsiType,
	{
		let (mut h, mut s, mut _i) = (h, s, _i);
		if h > 180.0 {
			h = 180.0;
		} else if h < -180.0 {
			h = -180.0
		}
		if s > 100.0 {
			s = 100.0;
		} else if s < -100.0 {
			s = -100.0
		}
		if _i > 100.0 {
			_i = 100.0;
		} else if _i < -100.0 {
			_i = -100.0
		}
		Hsi {
			hue_rotate: h / 360.0,
			saturate: s / 100.0,
			bright_ness: _i / 100.0,
		}
	},;
	h: f32, s: f32, _i: f32, );
/************************************ Transform **************************************/
style_out_export!(@expr 
	translate, 
	TranslateType, 
	{
		let mut input = cssparser::ParserInput::new(s);
		let mut parse = cssparser::Parser::new(&mut input);

		let translate = pi_style::style_parse::parse_mult(&mut parse, [LengthUnit::default(), LengthUnit::default()], pi_style::style_parse::parse_len_or_percent);
		if let Ok(translate) = translate {
			translate
		} else {
			Default::default()
		}
	},
	s: &str,; );
style_out_export!(@expr 
	scale, 
	ScaleType, 
	{
		let mut input = cssparser::ParserInput::new(s);
		let mut parse = cssparser::Parser::new(&mut input);

		let scale = pi_style::style_parse::parse_mult(&mut parse, [1.0f32, 1.0f32], pi_style::style_parse::parse_number);
		if let Ok(scale) = scale {
			scale
		} else {
			Default::default()
		}

	},
	s: &str,; );
style_out_export!(@expr 
	rotate, 
	RotateType, 
	{
		let mut input = cssparser::ParserInput::new(s);
		let mut parse = cssparser::Parser::new(&mut input);

		let rotate = pi_style::style_parse::parse_angle(&mut parse);
		if let Ok(rotate) = rotate {
			rotate
		} else {
			Default::default()
		}

	},
	s: &str,; );

style_out_export!(@expr 
	transform, 
	TransformType, 
	{
		let mut input = cssparser::ParserInput::new(s);
		let mut parse = cssparser::Parser::new(&mut input);

		let transform = pi_style::style_parse::parse_transform(&mut parse);
		if let Ok(transform) = transform {
			transform
		} else {
			Default::default()
		}
	},
	s: &str,; );

style_out_export!(@expr 
	transform_origin, 
	TransformOriginType, 
	{
		let x_ty = unsafe { transmute(x_ty as u8) };
		let y_ty = unsafe { transmute(y_ty as u8) };
		let x_value = match x_ty {
			LengthUnitType::Pixel => LengthUnit::Pixel(x),
			LengthUnitType::Percent => LengthUnit::Percent(x),
		};
		let y_value = match y_ty {
			LengthUnitType::Pixel => LengthUnit::Pixel(y),
			LengthUnitType::Percent => LengthUnit::Percent(y),
		};
		TransformOrigin::XY(x_value, y_value)
	},;
	x_ty: f64, x: f32, y_ty: f64, y: f32,);

// 设置transform为None TODO

style_out_export!(@expr letter_spacing, LetterSpacingType, v,; v: f32,);
style_out_export!(@expr word_spacing, WordSpacingType, v,; v: f32,);

style_out_export!(@expr text_rgba_color, ColorType, Color::RGBA(CgColor::new(r, g, b, a)),; r: f32, g: f32, b: f32, a: f32,);
style_out_export!(@expr 
	text_linear_gradient_color, 
	ColorType, 
	Color::LinearGradient(to_linear_gradient_color(
		color_and_positions.as_slice(),
		direction,
	)),; direction: f32, color_and_positions: Vec<f32>, );
style_out_export!(@expr line_height_normal, LineHeightType, LineHeight::Normal,;);
style_out_export!(@expr line_height, LineHeightType, LineHeight::Length(value),; value: f32,);
style_out_export!(@expr line_height_percent, LineHeightType, LineHeight::Percent(value), ;value: f32,);
style_out_export!(@expr text_indent, TextIndentType, v,; v: f32,);
// style_out_export!(@cenum text_align, TextAlignType);
style_out_export!(
	@owner 
	text_align, 
	{	
		let v: TextAlign = unsafe {transmute(v as u8)};
		gui.gui.set_style(node_id, TextAlignType(v));
		gui.gui.set_style(node_id, JustifyContentType(match v {
			TextAlign::Left => JustifyContent::FlexStart,
			TextAlign::Right => JustifyContent::FlexEnd,
			TextAlign::Center => JustifyContent::Center,
			TextAlign::Justify => JustifyContent::SpaceBetween,
		}));
	},
	{
		gui.gui.set_style(node_id, ResetTextAlignType);
		gui.gui.set_style(node_id, ResetJustifyContentType);
	},;
	v: f64,
);

style_out_export!(
	@owner 
	vertical_align, 
	{	
		let v: VerticalAlign = unsafe {transmute(v as u8)};
		gui.gui.set_style(node_id, VerticalAlignType(v));
		gui.gui.set_style(node_id, AlignSelfType(match v {
			VerticalAlign::Top => AlignSelf::FlexStart,
			VerticalAlign::Bottom => AlignSelf::FlexEnd,
			VerticalAlign::Middle => AlignSelf::Center,
		}));
	},
	{
		gui.gui.set_style(node_id, ResetVerticalAlignType);
		gui.gui.set_style(node_id, ResetAlignSelfType);
	},;
	v: f64,
);

style_out_export!(@expr $attr_name, $last_ty, unsafe {transmute(v as u8)},; v: f64,);

style_out_export!(@expr text_stroke, TextStrokeType, Stroke {
	width: NotNan::new(width).expect("stroke width is nan"),
	color: CgColor::new(r, g, b, a),
},; width: f32, r: f32, g: f32, b: f32, a: f32,);
style_out_export!(@cenum white_space, WhiteSpaceType);
style_out_export!(@cenum font_style, FontStyleType);
style_out_export!(@expr font_weight, FontWeightType, v as usize,; v: f64,);
style_out_export!(@expr font_size_none, FontSizeType, FontSize::None,;);
style_out_export!(@expr font_size, FontSizeType, FontSize::Length(value as usize),; value: f64,);
style_out_export!(@expr font_size_percent, FontSizeType, FontSize::Percent(value),; value: f32,);
style_out_export!(@expr text_content_utf8, TextContentType, {
	let content = unsafe{String::from_utf8_unchecked(content)};
	TextContent(content, pi_atom::Atom::from(""))
},; content: Vec<u8>,);
style_out_export!(@expr clip_path_str, ClipPathType, {
	let mut input = cssparser::ParserInput::new(value);
    let mut parse = cssparser::Parser::new(&mut input);

    match BaseShape::parse(&mut parse) {
        Ok(r) => r,
        Err(e) => {
            log::error!("set_animation_str fail, animation: {}, err: {:?}", value, e);
            return;
        }
    }
}, value: &str,;);

style_out_export!(@atom 
	mask_image,
	MaskImageType,
	MaskImage::Path((**image_hash).clone()),
	image_hash: &Atom,; );

style_out_export!(@atom 
	background_image,
	BackgroundImageType,
	(**image_hash).clone(),
	image_hash: &Atom,; );
style_out_export!(@atom 
	border_image,
	BorderImageType,
	(**image_hash).clone(),
	image_hash: &Atom,; );
style_out_export!(@expr text_shadow, TextShadowType, {
	let mut input = cssparser::ParserInput::new(s);
	let mut parse = cssparser::Parser::new(&mut input);

	let shadows = parse_text_shadow(&mut parse);
	if let Ok(value) = shadows {
		value
	} else {
		Default::default()
	}
}, s: &str,;);
style_out_export!(@atom font_family, FontFamilyType, (**name).clone(), name: &Atom,;);
style_out_export!(@expr text_content, TextContentType,  TextContent(content, pi_atom::Atom::from("")), ;content: String,);


pub enum LengthUnitType {
    Pixel,
    Percent,
}

pub fn to_linear_gradient_color(
	color_and_positions: &[f32],
	direction: f32,
) -> LinearGradientColor {
	let arr = color_and_positions;
	let len = arr.len();
	let count = len / 5;
	let mut list = Vec::with_capacity(count);
	for i in 0..count {
		let start = i * 5;
		let color_pos = ColorAndPosition {
			rgba: CgColor::new(
				arr[start],
				arr[start + 1],
				arr[start + 2],
				arr[start + 3],
			),
			position: arr[start + 4],
		};
		list.push(color_pos);
	}
	LinearGradientColor {
		direction: direction,
		list: list,
	}
}