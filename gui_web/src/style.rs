use std::mem::transmute;

use gui::single::style_parse::StyleParse;
use wasm_bindgen::prelude::*;
// use stdweb::unstable::TryInto;
// use stdweb::web::TypedArray;
use cssparser::{ParserInput, Parser};

use ecs::LendMut;
use hash::XHashMap;

use crate::world::GuiWorld;
use flex_layout::Rect;
use gui::component::user::{BlendMode as BlendMode1, *};
#[cfg(feature = "create_class_by_str")]
use gui::single::style_parse::parse_class_from_string;
use gui::single::*;
use gui::util::vecmap_default::VecMapWithDefault;

#[macro_use()]
macro_rules! insert_value {
    ($world:ident, $node_id:ident, $tt:ident, $value:expr, $key:ident) => {
        let node_id = $node_id as usize;
        let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
        let world = &mut world.gui;
        world.$key.lend_mut().insert(node_id, $tt($value));
    };
}

#[macro_use()]
macro_rules! insert_attr {
    ($world:ident, $node_id:ident, $tt:ident, $value:expr, $key:ident) => {
        let node_id = $node_id as usize;
        let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
        let world = &mut world.gui;
        world.$key.lend_mut().insert(node_id, $value);
    };
}
#[macro_use()]
macro_rules! set_show {
    ($world:ident, $node_id:ident, $name:ident, $value:expr, $field:expr) => {
        let node_id = $node_id as usize;
        let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
        let world = &mut world.gui;
        let show = world.show.lend_mut();
        let s = unsafe { show.get_unchecked_mut(node_id) };
        let old = s.clone();
        s.$name($value);
        if old != *s {
            show.get_notify_ref().modify_event(node_id, $field, 0);
        }
    };
}

/// 设置背景色的rgba
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_background_rgba_color(world: u32, node: u32, r: f32, g: f32, b: f32, a: f32) {
    insert_value!(world, node, BackgroundColor, Color::RGBA(CgColor::new(r, g, b, a)), background_color);
}

// // 设置一个径向渐变的背景颜色
// #[allow(unused_attributes)]
// #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
// pub fn set_background_radial_gradient_color(world: u32, node: u32, center_x: f32, center_y: f32, shape: u8, size: u8,color_and_positions: &[f32] ){
//     let value = Color::RadialGradient(to_radial_gradient_color(color_and_positions, center_x, center_y, shape, size));
//     insert_value!(world, node, BackgroundColor, value);
// }

/// 设置一个线性渐变的背景颜色
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_background_linear_gradient_color(world: u32, node: u32, direction: f32, color_and_positions: &[f32]) {
    let value = Color::LinearGradient(to_linear_gradient_color(color_and_positions, direction));
    insert_value!(world, node, BackgroundColor, value, background_color);
}

/// 设置边框颜色， 类型为rgba
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_border_color(world: u32, node: u32, r: f32, g: f32, b: f32, a: f32) {
    insert_value!(world, node, BorderColor, CgColor::new(r, g, b, a), border_color);
}

/// 设置边框圆角
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_border_radius(world: u32, node: u32, value: String) {
	let mut input = ParserInput::new(value.as_str());
    let mut parse = Parser::new(&mut input);
	match BorderRadius::parse(&mut parse) {
		Ok(v) => {
			insert_attr!(
				world,
				node,
				BorderRadius,
				v,
				border_radius
			);
		},
		Err(r) => {
			log::error!("set_border_radius invalid, {:?}", value);
		}
	}
}

/// 设置clip_path
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_clip_path_str(world: u32, node: u32, value: String) {
	let mut input = ParserInput::new(value.as_str());
    let mut parse = Parser::new(&mut input);
	match BaseShape::parse(&mut parse) {
		Ok(v) => {
			insert_attr!(
				world,
				node,
				ClipPath,
				ClipPath(v),
				clip_path
			);
		},
		Err(r) => {
			log::error!("set_clip_path_str invalid, {:?}", value);
		}
	}
}

/// 设置clip_path
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_clip_path(world: u32, node: u32, value: &BaseShape1) {
	insert_attr!(
		world,
		node,
		ClipPath,
		ClipPath(value.0.clone()),
		clip_path
	);
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[derive(Debug)]
pub struct BaseShape1 (BaseShape);

/// 对clip属性进行插值
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn interpolation_clip_path(value1: &BaseShape1, value2: &BaseShape1, process: f32) -> BaseShape1 {
	BaseShape1(value1.0.scale(1.0 - process).add(&value2.0.scale(process)))
}

/// 创建baseshape
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn create_base_shape(value: String) -> Option<BaseShape1> {
	let mut input = ParserInput::new(value.as_str());
    let mut parse = Parser::new(&mut input);
	match BaseShape::parse(&mut parse) {
		Ok(v) => Some(BaseShape1(v)),
		Err(r) => {
			log::error!("set_border_radius invalid, {:?}, reason: {:?}", value, r);
			None
		}
	}
}

pub trait AnimatableValue {
    fn add(&self, rhs: &Self) -> Self;
    fn scale(&self, other: f32) -> Self;
}

impl AnimatableValue for BaseShape {
    fn add(&self, rhs: &Self) -> Self {
        match (self, rhs) {
            (BaseShape::Circle{radius: radius1, center: center1}, BaseShape::Circle {radius: radius2, center: center2}) => BaseShape::Circle {
				radius: radius1.add(radius2),
				center: Center {x:  center1.x.add(&center2.x), y: center1.y.add(&center2.y)}
			},
            (BaseShape::Ellipse{rx: rx1, ry: ry1, center: center1}, BaseShape::Ellipse{rx: rx2, ry: ry2, center: center2}) => BaseShape::Ellipse {
				rx: rx1.add(rx2),
				ry: ry1.add(ry2),
				center: Center {x:  center1.x.add(&center2.x), y: center1.y.add(&center2.y)}
			},
			(BaseShape::Inset{rect_box: rect_box1, border_radius: border_radius1}, BaseShape::Inset {rect_box: rect_box2, border_radius: border_radius2}) => BaseShape::Inset {
				rect_box: [
					rect_box1[0].add(&rect_box2[0]), 
					rect_box1[1].add(&rect_box2[1]), 
					rect_box1[2].add(&rect_box2[2]), 
					rect_box1[3].add(&rect_box2[3]),
				],
				border_radius: BorderRadius { 
					x: [
						border_radius1.x[0].add(&border_radius2.x[0]), 
						border_radius1.x[1].add(&border_radius2.x[1]), 
						border_radius1.x[2].add(&border_radius2.x[2]), 
						border_radius1.x[3].add(&border_radius2.x[3]),
					], 
					y: [
						border_radius1.y[0].add(&border_radius2.y[0]), 
						border_radius1.y[1].add(&border_radius2.y[1]), 
						border_radius1.y[2].add(&border_radius2.y[2]), 
						border_radius1.y[3].add(&border_radius2.y[3]),
					]
				}
			},
			(BaseShape::Sector{angle: angle1, rotate: rotate1, radius: radius1,  center: center1}, BaseShape::Sector{angle: angle2, rotate: rotate2, radius: radius2, center: center2}) => BaseShape::Sector {
				angle: angle1 + angle2,
				rotate: rotate1 + rotate2,
				radius: radius1.add(radius2),
				center: Center {x:  center1.x.add(&center2.x), y: center1.y.add(&center2.y)}
			},
			(_, rhs) => rhs.clone()
        }
    }
    #[inline]
    fn scale(&self, other: f32) -> Self {
		match self {
			BaseShape::Circle { radius, center } => BaseShape::Circle { radius: radius.scale(other), center: Center {x:  center.x.scale(other), y: center.y.scale(other)} },
			BaseShape::Ellipse { rx, ry, center } => BaseShape::Ellipse { rx: rx.scale(other), ry: ry.scale(other), center: Center {x:  center.x.scale(other), y: center.y.scale(other)} },
			BaseShape::Inset { rect_box, border_radius } => BaseShape::Inset { rect_box: [
				rect_box[0].scale(other), 
				rect_box[1].scale(other), 
				rect_box[2].scale(other), 
				rect_box[3].scale(other),
			], border_radius: BorderRadius {
				x: [
					border_radius.x[0].scale(other), 
					border_radius.x[1].scale(other), 
					border_radius.x[2].scale(other), 
					border_radius.x[3].scale(other),
				], 
				y: [
					border_radius.y[0].scale(other), 
					border_radius.y[1].scale(other), 
					border_radius.y[2].scale(other), 
					border_radius.y[3].scale(other),
				]
			} } ,
			BaseShape::Sector { angle, rotate, radius, center } => BaseShape::Sector { angle: angle * other, rotate: rotate * other, radius: radius.scale(other), center: Center {x:  center.x.scale(other), y: center.y.scale(other)} },
		}
        // match self {
        //     LengthUnit::Pixel(r1) => LengthUnit::Pixel(r1 * other),
        //     LengthUnit::Percent(r1) => LengthUnit::Percent(r1 * other),
        // }
    }
}

impl AnimatableValue for LengthUnit {
    fn add(&self, rhs: &Self) -> Self {
        match (self, rhs) {
            (LengthUnit::Pixel(r1), LengthUnit::Pixel(r2)) => LengthUnit::Pixel(r1 + r2),
            (LengthUnit::Pixel(r1), LengthUnit::Percent(_)) => LengthUnit::Pixel(*r1),
            (LengthUnit::Percent(r1), LengthUnit::Pixel(_)) => LengthUnit::Percent(*r1),
            (LengthUnit::Percent(r1), LengthUnit::Percent(r2)) => LengthUnit::Percent(r1 + r2),
        }
    }
    #[inline]
    fn scale(&self, other: f32) -> Self {
        match self {
            LengthUnit::Pixel(r1) => LengthUnit::Pixel(r1 * other),
            LengthUnit::Percent(r1) => LengthUnit::Percent(r1 * other),
        }
    }
}


// // 设置阴影颜色
// #[allow(unused_attributes)]
// #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
// pub fn set_box_shadow_color(world: u32, node: u32, r: f32, g: f32, b: f32, a: f32){
//     let color = 0;
//     set_attr!(world, node, BoxShadow, color, CgColor::new(r, g, b, a), box_shadow);
// }

// #[allow(unused_attributes)]
// #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
// pub fn set_box_shadow_spread(world: u32, node: u32, value: f32){
//     let spread = 0;
//     set_attr!(world, node, BoxShadow, spread, value, box_shadow);
// }

// #[allow(unused_attributes)]
// #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
// pub fn set_box_shadow_blur(world: u32, node: u32, value: f32){
//     let blur = 0;
//     set_attr!(world, node, BoxShadow, blur, value, box_shadow);
// }

// // 设置阴影h
// #[allow(unused_attributes)]
// #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
// pub fn set_box_shadow_h(world: u32, node: u32, value: f32){
//     let h = 0;
//     set_attr!(world, node, BoxShadow, h, value, box_shadow);
// }

// // 设置阴影v
// #[allow(unused_attributes)]
// #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
// pub fn set_box_shadow_v(world: u32, node: u32, value: f32){
//     let v = 0;
//     set_attr!(world, node, BoxShadow, v, value, box_shadow);
// }

/// 设置阴影v
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_box_shadow(world: u32, node: u32, h: f32, v: f32, blur: f32, spread: f32, r: f32, g: f32, b: f32, a: f32) {
    // let v = 0;
    insert_attr!(
        world,
        node,
        BoxShadow,
        BoxShadow {
            h: h,
            v: v,
            blur: blur,
            spread: spread,
            color: CgColor::new(r, g, b, a)
        },
        box_shadow
    );
}

/// 设置object_fit
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_object_fit(world: u32, node: u32, value: u8) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let object_fits = world.gui.object_fit.lend_mut();
    if let None = object_fits.get(node as usize) {
        object_fits.insert_no_notify(node as usize, BackgroundImageOption::default());
    }
    object_fits[node as usize].object_fit = unsafe { transmute(value) };
    object_fits.get_notify_ref().modify_event(node as usize, "", 0);
}

#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_background_repeat(world: u32, node: u32, x: u8, y: u8) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let object_fits = world.gui.object_fit.lend_mut();
    if let None = object_fits.get(node as usize) {
        object_fits.insert_no_notify(node as usize, BackgroundImageOption::default());
    }
    object_fits[node as usize].repeat = (unsafe { transmute(x) }, unsafe { transmute(y) });
    object_fits.get_notify_ref().modify_event(node as usize, "", 0);
}

// 设置图像裁剪
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_image_clip(world: u32, node: u32, u1: f32, v1: f32, u2: f32, v2: f32) {
    insert_value!(world, node, ImageClip, Aabb2::new(Point2::new(u1, v1), Point2::new(u2, v2)), image_clip);
}

// 设置图像裁剪
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_mask_image_clip(world: u32, node: u32, u1: f32, v1: f32, u2: f32, v2: f32) {
    insert_value!(
        world,
        node,
        MaskImageClip,
        Aabb2::new(Point2::new(u1, v1), Point2::new(u2, v2)),
        mask_image_clip
    );
}

// 设置图像裁剪
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_border_image_clip(world: u32, node: u32, u1: f32, v1: f32, u2: f32, v2: f32) {
    // println!("set_border_image_clip: {:?}, {}, {}, {}", u1, v1, u2, v2);
    insert_value!(
        world,
        node,
        BorderImageClip,
        Aabb2::new(Point2::new(u1, v1), Point2::new(u2, v2)),
        border_image_clip
    );
}
/// 设置border_image_slice
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_border_image_slice(world: u32, node: u32, top: f32, right: f32, bottom: f32, left: f32, fill: bool) {
    insert_attr!(
        world,
        node,
        BorderImageSlice,
        BorderImageSlice {
            top,
            right,
            bottom,
            left,
            fill
        },
        border_image_slice
    );
}
/// 设置border_image_slice
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_border_image_repeat(world: u32, node: u32, vertical: u8, horizontal: u8) {
    insert_attr!(
        world,
        node,
        BorderImageRepeat,
        BorderImageRepeat(unsafe { transmute(vertical) }, unsafe { transmute(horizontal) }),
        border_image_repeat
    );
}

/// 设置overflow
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_overflow(world: u32, node: u32, value: bool) {
    insert_value!(world, node, Overflow, value, overflow);
}

/// 设置不透明度
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_opacity(world: u32, node: u32, mut value: f32) {
    if value > 1.0 {
        value = 1.0;
    } else if value < 0.0 {
        value = 0.0;
    }
    insert_value!(world, node, Opacity, value, opacity);
}

/// 设置display
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_display(world: u32, node: u32, value: u8) {
    unsafe {
        let layouts = (&mut *(world as usize as *mut GuiWorld)).gui.other_layout_style.lend_mut();
        layouts[node as usize].display = transmute(value);
        layouts.get_notify_ref().modify_event(node as usize, "display", 0);
    }

    let value = unsafe { transmute(value) };
    set_show!(world, node, set_display, value, "display");
}

/// 设置visibility, true: visible, false: hidden,	默认true
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_visibility(world: u32, node: u32, value: bool) {
    set_show!(world, node, set_visibility, value, "visibility");
}

/// 设置enable
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_enable(world: u32, node: u32, value: u32) {
    set_show!(world, node, set_enable, unsafe { transmute(value as u8) }, "enable");
}

/// 取enable
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn get_enable(world: u32, node: u32) -> bool {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };

    let enables = world.gui.enable.lend_mut();
    match enables.get(node as usize) {
        Some(r) => r.0,
        None => false,
    }
}

// let enable = arg.enables[*bind].0;

/// 这只z_index
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_zindex(world: u32, node: u32, value: i32) {
    let value = value as isize;
    insert_value!(world, node, ZIndex, value, z_index);
}

#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_filter_blur(world: u32, node: u32, blur: f32) {
    insert_value!(world, node, Blur, blur, blur);
}

/// hsi, 效果与ps一致,  h: -180 ~ 180, s: -100 ~ 100, i: -100 ~ 100
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_filter_hsi(world: u32, node: u32, mut h: f32, mut s: f32, mut i: f32) {
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
    if i > 100.0 {
        i = 100.0;
    } else if i < -100.0 {
        i = -100.0
    }
    let value = Filter {
        hue_rotate: h / 360.0,
        saturate: s / 100.0,
        bright_ness: i / 100.0,
    };
    insert_attr!(world, node, Filter, value, filter);
}

/// __jsObj: image_name(String)
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_border_image(world: u32, node: u32, url: usize) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let border_images = world.gui.border_image.lend_mut();
    border_images.insert(node as usize, BorderImage { url });
}

/// __jsObj: image_name(String)
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_mask_image(world: u32, node: u32, url: usize) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let mask_images = world.gui.mask_image.lend_mut();
    mask_images.insert(node as usize, MaskImage::Path(url));
}

// 设置mask_image为渐变色
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_mask_image_linenear(world: u32, node: u32, direction: f32, color_and_positions: &[f32]) {
    let value = to_linear_gradient_color(color_and_positions, direction);

    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let mask_images = world.gui.mask_image.lend_mut();
    // log::info!("set_mask_image_linenear========={}, {:?}", node, value);
    mask_images.insert(node as usize, MaskImage::LinearGradient(value));
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub enum BlendMode {
    Normal,
    AlphaAdd,
    Subtract,
    Multiply,
    OneOne,
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_blend_mode(world: u32, node: u32, blend_mode: u8) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let blend_modes = world.gui.blend_mode.lend_mut();
    blend_modes.insert(node as usize, unsafe { transmute(blend_mode) });
}

/// 设置默认样式, 暂支持布局属性、 文本属性的设置
/// __jsObj: class样式的二进制描述， 如".0{color:red}"生成的二进制， class名称必须是“0”
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_default_style_by_str(world: u32, value: &str) {
	let default_style = match parse_class_from_string(value) {
		Ok(r) => r,
		Err(_e) => return,
	};
    // let mut map: XHashMap<usize, Class> = match bincode::deserialize(bin) {
    //     Ok(r) => r,
    //     Err(e) => {
    //         debug_println!("deserialize_class_map error: {:?}", e);
    //         return;
    //     }
    // };

    // let default_style = map.remove(&0).unwrap();
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    set_default_style1(world, default_style);
}

#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_default_style_by_bin(world: u32, bin: &[u8]) {
    let mut map: XHashMap<usize, Class> = match bincode::deserialize(bin) {
        Ok(r) => r,
        Err(e) => {
            debug_println!("deserialize_class_map error: {:?}", e);
            return;
        }
    };

    let default_style = map.remove(&0).unwrap();
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    set_default_style1(world, default_style);
}

/// 设置默认样式, 暂支持布局属性、 文本属性的设置
/// __jsObj: class样式的文本描述
#[cfg(feature = "create_class_by_str")]
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_default_style(world: u32, css: &str) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let r = match parse_class_from_string(css) {
        Ok(r) => r,
        Err(e) => {
            debug_println!("set_default_style error, {:?}", e);
            return;
        }
    };
    set_default_style1(world, r);
    // world.default_layout_attr = r.1;
}

/// 设置transform_will_change
#[allow(unused_attributes)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn set_transform_will_change(world: u32, node_id: u32, value: u8) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let node_id = node_id as usize;
    let transforms = world.gui.transform.lend_mut();
    let transform_will_changes = world.gui.transform_will_change.lend_mut();
    if value == 0 {
        if transform_will_changes.get(node_id).is_some() {
            transforms.insert(node_id, transform_will_changes.delete(node_id).unwrap().0);
        }
    } else {
        if transforms.get(node_id).is_some() {
            transform_will_changes.insert(node_id, TransformWillChange(transforms.delete(node_id).unwrap()));
        } else if transform_will_changes.get(node_id).is_none() {
            transform_will_changes.insert(node_id, TransformWillChange(Transform::default()));
        }
    }
}

fn set_default_style1(world: &mut GuiWorld, r: Class) {
    let mut text_style = TextStyle::default();
    let mut rect_layout_style = RectLayoutStyle::default();
    let mut other_layout_style = OtherLayoutStyle::default();
    // let mut border_color = BorderColor::default();
    // let mut bg_color = BackgroundColor::default();
    // let mut box_shadow = BoxShadow::default();

    for attr in r.attrs1.into_iter() {
        match attr {
            // Attribute::BGColor(r) => bg_color = r,
            Attribute1::TextAlign(r) => text_style.text.text_align = r,
            Attribute1::WhiteSpace(r) => text_style.text.white_space = r,

            Attribute1::PositionType(r) => other_layout_style.position_type = r,
            Attribute1::FlexWrap(r) => other_layout_style.flex_wrap = r,
            Attribute1::FlexDirection(r) => other_layout_style.flex_direction = r,
            Attribute1::AlignContent(r) => other_layout_style.align_content = r,
            Attribute1::AlignItems(r) => other_layout_style.align_items = r,
            Attribute1::AlignSelf(r) => other_layout_style.align_self = r,
            Attribute1::JustifyContent(r) => other_layout_style.justify_content = r,

            // Attribute::BorderColor(r) => border_color = r,
            _ => debug_println!("set_class error"),
        }
    }

    for attr in r.attrs2.into_iter() {
        match attr {
            Attribute2::LetterSpacing(r) => text_style.text.letter_spacing = r,
            Attribute2::WordSpacing(r) => text_style.text.word_spacing = r,
            Attribute2::LineHeight(r) => text_style.text.line_height = r,
            Attribute2::TextIndent(r) => text_style.text.indent = r,

            Attribute2::FontWeight(r) => text_style.font.weight = r as usize,
            Attribute2::FontSize(r) => text_style.font.size = r,
            Attribute2::FontFamily(r) => {
                text_style.font.family = r;
            }

            Attribute2::Width(r) => rect_layout_style.size.width = r,
            Attribute2::Height(r) => rect_layout_style.size.height = r,
            Attribute2::MarginLeft(r) => rect_layout_style.margin.start = r,
            Attribute2::MarginTop(r) => rect_layout_style.margin.top = r,
            Attribute2::MarginBottom(r) => rect_layout_style.margin.bottom = r,
            Attribute2::MarginRight(r) => rect_layout_style.margin.end = r,
            Attribute2::Margin(r) => {
                rect_layout_style.margin = Rect {
                    start: r,
                    end: r,
                    bottom: r,
                    top: r,
                }
            }

            Attribute2::FlexShrink(r) => other_layout_style.flex_shrink = r,
            Attribute2::FlexGrow(r) => other_layout_style.flex_grow = r,
            Attribute2::PaddingLeft(r) => other_layout_style.padding.start = r,
            Attribute2::PaddingTop(r) => other_layout_style.padding.top = r,
            Attribute2::PaddingBottom(r) => other_layout_style.padding.bottom = r,
            Attribute2::PaddingRight(r) => other_layout_style.padding.end = r,
            Attribute2::Padding(r) => {
                other_layout_style.padding = Rect {
                    start: r,
                    end: r,
                    bottom: r,
                    top: r,
                }
            }
            Attribute2::BorderLeft(r) => other_layout_style.border.start = r,
            Attribute2::BorderTop(r) => other_layout_style.border.top = r,
            Attribute2::BorderBottom(r) => other_layout_style.border.bottom = r,
            Attribute2::BorderRight(r) => other_layout_style.border.end = r,
            Attribute2::Border(r) => {
                other_layout_style.border = Rect {
                    start: r,
                    end: r,
                    bottom: r,
                    top: r,
                }
            }
            Attribute2::MinWidth(r) => other_layout_style.min_size.width = r,
            Attribute2::MinHeight(r) => other_layout_style.min_size.height = r,
            Attribute2::MaxHeight(r) => other_layout_style.max_size.width = r,
            Attribute2::MaxWidth(r) => other_layout_style.max_size.width = r,
            Attribute2::FlexBasis(r) => other_layout_style.flex_basis = r,
            Attribute2::PositionLeft(r) => other_layout_style.position.start = r,
            Attribute2::PositionTop(r) => other_layout_style.position.top = r,
            Attribute2::PositionRight(r) => other_layout_style.position.end = r,
            Attribute2::PositionBottom(r) => other_layout_style.position.bottom = r,

            // Attribute::BorderColor(r) => border_color = r,
            _ => debug_println!("set_class error"),
        }
    }

    for attr in r.attrs3.into_iter() {
        match attr {
            Attribute3::Color(r) => text_style.text.color = r,
            Attribute3::TextShadow(r) => text_style.shadow = r,
            Attribute3::TextStroke(r) => text_style.text.stroke = r,
            // Attribute::BorderColor(r) => border_color = r,
            _ => debug_println!("set_class error"),
        }
    }

    let text_styles = unsafe {
        &mut *(world.gui.text_style.lend_mut().get_storage() as *const VecMapWithDefault<TextStyle> as usize as *mut VecMapWithDefault<TextStyle>)
    };
    let flex_rect_styles = unsafe {
        &mut *(world.gui.rect_layout_style.lend_mut().get_storage() as *const VecMapWithDefault<RectLayoutStyle> as usize
            as *mut VecMapWithDefault<RectLayoutStyle>)
    };
    let flex_other_styles = unsafe {
        &mut *(world.gui.other_layout_style.lend_mut().get_storage() as *const VecMapWithDefault<OtherLayoutStyle> as usize
            as *mut VecMapWithDefault<OtherLayoutStyle>)
    };

    flex_rect_styles.set_default(rect_layout_style);
    flex_other_styles.set_default(other_layout_style);
    text_styles.set_default(text_style);
}
