/// 将设置文本属性的接口导出到js
use std::mem::transmute;

use gui::single::style_parse::parse_text_shadow;
use wasm_bindgen::prelude::*;
use web_sys::HtmlImageElement;
use js_sys::Object;
use smallvec::SmallVec;


use atom::Atom;
use data_view::GetView;
use ecs::{LendMut, Lend};
use gui::component::user::*;
use gui::font::font_sheet::{FontSheet, Glyph, TexFont};
use hal_core::*;
use crate::world::{GuiWorld, next_power_of_two};
use crate::index::create_texture;
use crate::index::PixelFormat;

#[macro_use()]
macro_rules! set_attr {
    ($world:ident, $node_id:ident, $name:ident, $name1:ident, $name2: expr, $value:expr, $key: ident) => {
        let node_id = $node_id as usize;
        let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
        let attr = world.gui.$key.lend_mut();
        let value = $value;
        $crate::paste::item! {
            let r = &mut attr[node_id];
            r.$name.$name1 = value;
            attr.get_notify_ref().modify_event(node_id, $name2, 0);
        }
    };
}

#[macro_use()]
macro_rules! get_attr {
    ($world:ident, $node_id:ident, $name:ident, $name1:ident, $name2: expr, $key: ident) => {
		{
			let node_id = $node_id as usize;
			let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
			let attr = world.gui.$key.lend();
			$crate::paste::item! {
				let r = &attr[node_id];
				&r.$name.$name1
			}
		}
    };
}

/// 设置字符间距
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_letter_spacing(world: u32, node_id: u32, value: f32) {
    set_attr!(
        world,
        node_id,
        text,
        letter_spacing,
        "letter_spacing",
        value,
        text_style
    );
}

/// 设置单词间距
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_word_spacing(world: u32, node_id: u32, value: f32) {
    set_attr!(
        world,
        node_id,
        text,
        word_spacing,
        "word_spacing",
        value,
        text_style
    );
}

/// 设置文字rgba颜色
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_text_rgba_color(world: u32, node_id: u32, r: f32, g: f32, b: f32, a: f32) {
    set_attr!(
        world,
        node_id,
        text,
        color,
        "color",
        Color::RGBA(CgColor::new(r, g, b, a)),
        text_style
    );
}

/// 设置文字渐变颜色
/// __jsObj: color_and_positions: [r, g, b, a, pos,   r, g, b, a, pos], direction: 0-360度
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_text_linear_gradient_color(world: u32, node_id: u32, direction: f32, color_and_positions: &[f32]) {
    let value = Color::LinearGradient(to_linear_gradient_color(
        color_and_positions,
        direction,
    ));
    set_attr!(world, node_id, text, color, "color", value, text_style);
}

/// 设置行高为normal
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_line_height_normal(world: u32, node_id: u32) {
    set_attr!(
        world,
        node_id,
        text,
        line_height,
        "line_height",
        LineHeight::Normal,
        text_style
    );
}

/// 设置行高的像素值
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_line_height(world: u32, node_id: u32, value: f32) {
    set_attr!(
        world,
        node_id,
        text,
        line_height,
        "line_height",
        LineHeight::Length(value),
        text_style
    );
}

/// 设置行高的百分比值
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_line_height_percent(world: u32, node_id: u32, value: f32) {
    set_attr!(
        world,
        node_id,
        text,
        line_height,
        "line_height",
        LineHeight::Percent(value),
        text_style
    );
}

/// 设置文字首行缩进的像素值
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_text_indent(world: u32, node_id: u32, value: f32) {
    set_attr!(
        world,
        node_id,
        text,
        indent,
        "text_indent",
        value,
        text_style
    );
}

/// 设置文本的水平对齐方式
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_text_align(world: u32, node_id: u32, value: u8) {
    set_attr!(
        world,
        node_id,
        text,
        text_align,
        "text_align",
        unsafe { transmute(value) },
        text_style
    );
}

/// 设置文字的描边属性
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_text_stroke(world: u32, node_id: u32, width: f32, r: f32, g: f32, b: f32, a: f32) {
    set_attr!(
        world,
        node_id,
        text,
        stroke,
        "stroke",
        Stroke {
            width,
            color: CgColor::new(r, g, b, a),
        },
        text_style
    );
}

/// 设置文字的空白处理方式
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_white_space(world: u32, node_id: u32, value: u8) {
    set_attr!(
        world,
        node_id,
        text,
        white_space,
        "white_space",
        unsafe { transmute(value) },
        text_style
    );
}

// /// 设置文字阴影
// #[allow(unused_attributes)]
// #[wasm_bindgen]
// pub fn set_text_shadow(
//     world: u32,
//     node_id: u32,
//     h: f32,
//     v: f32,
//     blur: f32,
//     r: f32,
//     g: f32,
//     b: f32,
//     a: f32,
// ) {
//     let value = TextShadow {
//         h: h,
//         v: v,
//         blur: blur,
//         color: CgColor::new(r, g, b, a),
//     };
//     let node_id = node_id as usize;
//     let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
//     let world = &mut world.gui;
//     let text_styles = world.text_style.lend_mut();
//     let r = &mut text_styles[node_id];
//     r.shadow = value;
//     text_styles
//         .get_notify_ref()
//         .modify_event(node_id, "text_shadow", 0);
//     debug_println!("set_text_shadow");
// }

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_text_shadow(
    world: u32,
    node_id: u32,
    s: &str,
) {
	let shadows = parse_text_shadow(s);
	if let Ok(value) = shadows {
		let node_id = node_id as usize;
		let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
		let world = &mut world.gui;
		let text_styles = world.text_style.lend_mut();
		let r = &mut text_styles[node_id];
		r.shadow = value;
		text_styles
			.get_notify_ref()
			.modify_event(node_id, "text_shadow", 0);
	}
}

/// 设置字体风格
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_font_style(world: u32, node_id: u32, value: u8) {
    set_attr!(
        world,
        node_id,
        font,
        style,
        "font_style",
        unsafe { transmute(value) },
        text_style
    );
}

/// 设置字体粗度
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_font_weight(world: u32, node_id: u32, value: u32) {
    set_attr!(
        world,
        node_id,
        font,
        weight,
        "font_weight",
        value as usize,
        text_style
    );
}

/// 设置字体尺寸为none（使用默认）
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_font_size_none(world: u32, node_id: u32) {
    set_attr!(
        world,
        node_id,
        font,
        size,
        "font_size",
        FontSize::None,
        text_style
    );
}

/// 设置字体尺寸的像素值
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_font_size(world: u32, node_id: u32, value: f32) {
    set_attr!(
        world,
        node_id,
        font,
        size,
        "font_size",
        FontSize::Length(value),
        text_style
    );
}

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn get_font_size(world: u32, node_id: u32) -> JsValue {
    let r = get_attr!(
        world,
        node_id,
        font,
        size,
        "font_size",
        text_style
    );
	JsValue::from_serde(r).unwrap()
}

/// 设置字体尺寸的百分比
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_font_size_percent(world: u32, node_id: u32, value: f32) {
    set_attr!(
        world,
        node_id,
        font,
        size,
        "font_size",
        FontSize::Percent(value),
        text_style
    );
}

/// 设置字体
/// __jsObj: family name
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_font_family(world: u32, node_id: u32, name: u32) {
    set_attr!(
        world,
        node_id,
        font,
        family,
        "font_family",
        name as usize,
        text_style
    );
}

/// 添加一个msdf字体资源
/// 图片            配置       
///__jsObj: image , __jsObj1: glyph cfg
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn add_msdf_font_res(world_id: u32, image: HtmlImageElement, cfg: &[u8], name: u32) {
    let world1 = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let world = &mut world1.gui;
    let cfg = cfg.to_vec();
    let width: u32 = image.width();
    let height: u32 = image.height();
	let font_sheet = world.font_sheet.lend_mut();
	let font_sheet = &mut font_sheet.borrow_mut();

	if width > world1.max_texture_size {
		debug_println!("add_msdf_font_res fail");
	}

	let mut tex_font = TexFont {
        name: 0,
        is_pixel: false,
		factor_t: 0.0,
		factor_b: 0.0,
		textures: Some(Vec::new()),
		ascender: 0.0,
		descender: 0.0,
		font_size: 32.0
    };

	let (res, name) = create_texture(world_id, PixelFormat::RGBA, -1, 0, name, width, height, Object::from(image), width * height * 4);
	tex_font.textures.as_mut().unwrap().push(res);
	parse_msdf_font_res(cfg.as_slice(), font_sheet, name as u32, tex_font);
		// font_sheet.font_tex.last_v += height as f32;
	
	
}

/// 设置文本内容
/// __jsObj 文字字符串
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_text_content(world_id: u32, node: u32, content: String) {
    let node = node as usize;
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    world
        .text_content
        .lend_mut()
        .insert(node as usize, TextContent(content, Atom::from("")));
    debug_println!("set_text_content");
}

/// 设置文本内容
/// 文本内容为utf8编码的Uint8Array， 
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_text_content_utf8(world_id: u32, node: u32, content: Vec<u8>) {
	let content = unsafe{String::from_utf8_unchecked(content)};
    let node = node as usize;
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    world
        .text_content
        .lend_mut()
        .insert(node as usize, TextContent(content, Atom::from("")));
    debug_println!("set_text_content");
}

/// 添加一个canvas字体 
/// __jsObj1: name(String)
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn add_canvas_font(world: u32, factor_t: f32, factor_b: f32, name: u32) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
	let font_sheet = world.font_sheet.lend_mut();
    font_sheet.borrow_mut().set_src(name as usize, true, factor_t, factor_b, 0.0, 0.0);
}

/// 添加font-face
///          字体族名称                        字体名称（逗号分隔）     
/// __jsObj: family_name(String), __jsObj1: src_name(String, 逗号分隔),
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn add_font_face(world: u32, oblique: f32, size: u32, weight: u32, family: u32, src: Vec<usize>) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
	let font_sheet = world.font_sheet.lend_mut();
	// log::info!("add_font_face====================={:?}, {:?}", family, src);
    font_sheet.borrow_mut().set_face(
        family as usize,
        oblique,
        size as usize,
        weight as usize,
        src,
    );
}

/// 更新字体纹理
/// __jsObj: canvas
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn update_text_texture(world: u32, u: u32, v: u32, height: u32, img: HtmlImageElement) {
    let world1 = unsafe { &mut *(world as usize as *mut GuiWorld) };
	let world = &mut world1.gui;
	let single_font_sheet = world.font_sheet.lend_mut();
    let font_sheet = &mut single_font_sheet.borrow_mut();
    let engine = world.engine.lend_mut();
    let texture = font_sheet.get_font_tex();

    let mut end_v = v + height;
    if end_v > texture.height as u32 {
        end_v = next_power_of_two(end_v);
        if end_v > world1.max_texture_size {
            debug_println!("update_canvas_text fail, height overflow, height");
        }
        engine
            .gl
            .texture_extend(&texture.bind, texture.width as u32, end_v);
        texture.update_size(texture.width, end_v as usize);
        single_font_sheet.get_notify_ref().modify_event(0, "", 0);
	}
    engine.gl.texture_update_webgl(
        &texture.bind,
        0,
        u,
        v,
        &Object::from(img),
    );
}


// /// 绘制文字
// #[allow(unused_attributes)]
// #[no_mangle]
// #[js_export]
// pub fn draw_canvas_text(world_id: u32, data: u32) {
//     // let t = std::time::Instant::now();
//     let text_info_list = unsafe { Box::from_raw(data as usize as *mut Vec<TextInfo1>) };
//     let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
//     let canvas = &world.draw_text_sys.canvas;
//     let world = &mut world.gui;
//     let font_sheet = world.font_sheet.lend_mut();
//     let engine = world.engine.lend_mut();
//     let texture = font_sheet.get_font_tex();

//     // 将在绘制在同一行的文字归类在一起， 以便一起绘制，一起更新
//     let mut end_v = 0;
//     let mut map: XHashMap<u32, (Vec<usize>, Vector2)> = XHashMap::default();
//     for i in 0..text_info_list.len() {
//         let text_info = &text_info_list[i];
//         let first = &text_info.chars[0];
//         let h = first.y + text_info.size.y as u32;
//         if h > end_v {
//             end_v = h;
//         }
//         match map.entry(first.y) {
//             Entry::Occupied(mut e) => {
//                 let r = e.get_mut();
//                 r.0.push(i);
//                 r.1.x += text_info.size.x;
//                 if text_info.size.y > r.1.y {
//                     r.1.y = text_info.size.y;
//                 }
//             }
//             Entry::Vacant(r) => {
//                 r.insert((vec![i], text_info.size.clone()));
//             }
//         };
//     }

//     // 扩展纹理
//     if end_v > texture.height as u32 {
//         end_v = next_power_of_two(end_v);
//         if end_v > 2048 {
//             debug_println!("update_canvas_text fail, height overflow");
//         }
//         engine
//             .gl
//             .texture_extend(&texture.bind, texture.width as u32, end_v);
//         texture.update_size(texture.width, end_v as usize);
//         font_sheet.get_notify().modify_event(0, "", 0);
//     }

//     for indexs in map.iter() {
//         js! {

//             var c = @{canvas};
//             var canvas = c.canvas;
//             canvas.width = @{(indexs.1).1.x as u32 };
//             canvas.height = @{(indexs.1).1.y as u32 };
//             var ctx = c.ctx;
//             ctx.fillStyle = "#00f";
//             ctx.fillRect(0, 0, canvas.width, canvas.height);
//         }
//         let mut start: (i32, i32) = (-1, -1);
//         for i in (indexs.1).0.iter() {
//             let text_info = &text_info_list[*i];
//             let first = &text_info.chars[0];
//             if start.0 == -1 {
//                 start.0 = first.x as i32;
//                 start.1 = first.y as i32;
//             }
//             let hal_stroke_width = text_info.stroke_width / 2;
//             let bottom = text_info.size.y as u32 - hal_stroke_width as u32;
//             js! {

//                 var c = @{canvas};
//                 var ctx = c.ctx;
//                 var weight;
//                 if (@{text_info.weight as u32} <= 300 ) {
//                     weight = "lighter";
//                 } else if (@{text_info.weight as u32} < 700 ) {
//                     weight = "normal";
//                 } else if (@{text_info.weight as u32} < 900 ) {
//                     weight = "bold";
//                 } else {
//                     weight = "bolder";
//                 }
//                 ctx.font = weight + " " + @{text_info.font_size as u32} + "px " + @{text_info.font.as_ref()};
//                 ctx.fillStyle = "#0f0";
//                 ctx.textBaseline = "top";
//             }
//             if text_info.stroke_width > 0 {
//                 js! {
//                     var c = @{canvas};
//                     c.ctx.lineWidth = @{text_info.stroke_width as u8};
//                     c.ctx.strokeStyle = "#f00";
//                 }
//                 for char_info in text_info.chars.iter() {
//                     let ch_code: u32 = unsafe { transmute(char_info.ch) };
//                     let x = char_info.x + hal_stroke_width as u32 - start.0 as u32;
//                     js! {
//                         var c = @{canvas};
//                         var ch = String.fromCharCode(@{ch_code});
//                         //fillText 和 strokeText 的顺序对最终效果会有影响， 为了与css text-stroke保持一致， 应该fillText在前
//                         c.ctx.strokeText(ch, @{x}, 0);
//                         c.ctx.fillText(ch, @{x}, 0);
//                     }
//                 }
//             } else {
//                 for char_info in text_info.chars.iter() {
//                     let ch_code: u32 = unsafe { transmute(char_info.ch) };
//                     let x = char_info.x - start.0 as u32;
//                     js! {
//                         var ch = String.fromCharCode(@{ch_code});
//                         @{canvas}.ctx.fillText(ch, @{x}, 0);
//                     }
//                 }
//             }
// 		}
// 		// 在华为Mate 20上，将canvas更新到纹理存在bug，因此这里将canvas的数据取到，然后跟新到纹理
// 		// 如果在后续迭代的过程中，所有手机都不存在该bug，应该删除该句，以节省性能（getImageData会拷贝数据）
// 		js!{
// 			@{canvas}.wrap = @{canvas}.ctx.getImageData(0, 0, @{canvas}.canvas.width, @{canvas}.canvas.height);
// 		}
//         engine
//             .gl
//             .texture_update_webgl(&texture.bind, 0, start.0 as u32, start.1 as u32, &canvas);
//     }

//     // println!("time: {:?}", std::time::Instant::now() - t);
//     set_render_dirty(world_id);
// }
// /// 绘制文字
// #[allow(unused_attributes)]
// #[no_mangle]
// #[js_export]
// pub fn draw_canvas_text(world_id: u32, data: u32) {
//     // let t = std::time::Instant::now();
//     let text_info_list = unsafe { Box::from_raw(data as usize as *mut Vec<TextInfo1>) };
//     let world1 = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
//     let canvas = &world1.draw_text_sys.canvas;
//     let world = &mut world1.gui;
// 	let single_font_sheet = &mut world.font_sheet.lend_mut();
// 	let font_sheet = &mut single_font_sheet.borrow_mut();
// 	font_sheet.tex_version += 1;
//     let engine = world.engine.lend_mut();
//     let texture = font_sheet.get_font_tex();

//     // 将在绘制在同一行的文字归类在一起， 以便一起绘制，一起更新
//     let mut end_v = 0;
//     let mut map: XHashMap<u32, (Vec<usize>, Vector2)> = XHashMap::default();
//     for i in 0..text_info_list.len() {
//         let text_info = &text_info_list[i];
//         let first = &text_info.chars[0];
//         let h = first.y + text_info.size.y as u32;
//         if h > end_v {
//             end_v = h;
//         }
//         match map.entry(first.y) {
//             Entry::Occupied(mut e) => {
//                 let r = e.get_mut();
//                 r.0.push(i);
//                 r.1.x += text_info.size.x;
//                 if text_info.size.y > r.1.y {
//                     r.1.y = text_info.size.y;
//                 }
//             }
//             Entry::Vacant(r) => {
//                 r.insert((vec![i], text_info.size.clone()));
//             }
//         };
//     }

//     // 扩展纹理
//     if end_v > texture.height as u32 {
//         end_v = next_power_of_two(end_v);
//         if end_v > world1.max_texture_size {
//             debug_println!("update_canvas_text fail, height overflow");
//         }
//         engine
//             .gl
//             .texture_extend(&texture.bind, texture.width as u32, end_v);
//         texture.update_size(texture.width, end_v as usize);
//         single_font_sheet.get_notify_ref().modify_event(0, "", 0);
//     }

//     for indexs in map.iter() {
//         js! {

//             var c = @{canvas};
//             var canvas = c.canvas;
//             canvas.width = @{(indexs.1).1.x as u32 };
//             canvas.height = @{(indexs.1).1.y as u32 };
//             var ctx = c.ctx;
//             ctx.fillStyle = "#00f";
//             ctx.fillRect(0, 0, canvas.width, canvas.height);
//         }
//         let mut start: (i32, i32) = (-1, -1);
//         for i in (indexs.1).0.iter() {
//             let text_info = &text_info_list[*i];
//             let first = &text_info.chars[0];
//             if start.0 == -1 {
//                 start.0 = first.x as i32;
//                 start.1 = first.y as i32;
//             }
//             let hal_stroke_width = text_info.stroke_width / 2;
//             let bottom = text_info.size.y as u32 - hal_stroke_width as u32;
//             js! {

//                 var c = @{canvas};
//                 var ctx = c.ctx;
//                 var weight;
//                 if (@{text_info.weight as u32} <= 300 ) {
//                     weight = "lighter";
//                 } else if (@{text_info.weight as u32} < 700 ) {
//                     weight = "normal";
//                 } else if (@{text_info.weight as u32} < 900 ) {
//                     weight = "bold";
//                 } else {
//                     weight = "bolder";
//                 }
//                 ctx.font = weight + " " + @{text_info.font_size as u32} + "px " + @{text_info.font.as_ref()};
//                 ctx.fillStyle = "#0f0";
//                 ctx.textBaseline = "top";
//             }
//             if text_info.stroke_width > 0 {
//                 js! {
//                     var c = @{canvas};
//                     c.ctx.lineWidth = @{text_info.stroke_width as u8};
//                     c.ctx.strokeStyle = "#f00";
//                 }
//                 for char_info in text_info.chars.iter() {
//                     let ch_code: u32 = unsafe { transmute(char_info.ch) };
//                     let x = char_info.x + hal_stroke_width as u32 - start.0 as u32;
//                     js! {
//                         var c = @{canvas};
//                         var ch = String.fromCharCode(@{ch_code});
//                         //fillText 和 strokeText 的顺序对最终效果会有影响， 为了与css text-stroke保持一致， 应该fillText在前
//                         c.ctx.strokeText(ch, @{x}, @{text_info.top as u32});
//                         c.ctx.fillText(ch, @{x}, @{text_info.top as u32});
//                     }
//                 }
//             } else {
//                 for char_info in text_info.chars.iter() {
//                     let ch_code: u32 = unsafe { transmute(char_info.ch) };
//                     let x = char_info.x - start.0 as u32;
//                     js! {
//                         var ch = String.fromCharCode(@{ch_code});
//                         @{canvas}.ctx.fillText(ch, @{x}, @{text_info.top as u32});
//                     }
//                 }
//             }
// 		}
// 		// // 在华为Mate 20上，将canvas更新到纹理存在bug，因此这里将canvas的数据取到，然后跟新到纹理
// 		// // 如果在后续迭代的过程中，所有手机都不存在该bug，应该删除该句，以节省性能（getImageData会拷贝数据）
// 		// js!{
// 		// 	@{canvas}.wrap = @{canvas}.ctx.getImageData(0, 0, @{canvas}.canvas.width, @{canvas}.canvas.height);
// 		// }
//         engine
//             .gl
//             .texture_update_webgl(&texture.bind, 0, start.0 as u32, start.1 as u32, &canvas);
//     }

//     // println!("time: {:?}", std::time::Instant::now() - t);
// 	// println!("set_render_dirty11============={}", world_id);
//     set_render_dirty(world_id);
// }

// #[derive(Debug, Serialize)]
// pub struct TextInfo {
//     pub font: String,
//     pub font_size: usize,
//     pub stroke_width: usize,
//     pub weight: usize,
//     pub size: (f32, f32),
// 	pub chars: Vec<WaitChar>,
// 	pub top: usize,
// }

// #[derive(Debug, Serialize)]
// pub struct WaitChar {
//     ch: char,
//     width: f32,
//     x: u32,
//     y: u32,
// }

// #[derive(Debug, Serialize)]
// pub struct TextInfoList {
//     list: Vec<TextInfo>,
// }
// js_serializable!(TextInfo);
// js_serializable!(TextInfoList);

// pub struct DrawTextSys {
//     pub canvas: Object,
// }

// impl DrawTextSys {
//     pub fn new() -> Self {
//         let obj: Object = TryInto::try_into(js! {
// 			var c = document.createElement("canvas");
// 			// c.style.position = "absolute";
//             // document.body.append(c);// 查看效果
//             var ctx = c.getContext("2d");
//             return {canvas: c, ctx: ctx, wrap: c};
//         })
//         .unwrap();
//         DrawTextSys { canvas: obj }
//     }

//     pub fn run(&mut self, world_id: u32) {
//         let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
//         let world = &mut world.gui;
//         let font_sheet = world.font_sheet.lend_mut();
//         if font_sheet.wait_draw_list.len() == 0 {
//             return;
//         }

//         let list = std::mem::replace(&mut font_sheet.wait_draw_list, Vec::default());
//         let ptr = Box::into_raw(Box::new(list)) as usize as u32;

//         font_sheet.wait_draw_map.clear();
//         js! {
//             var p = @{ptr};
//             setTimeout(function(){
//                 Module._draw_canvas_text(@{world_id}, p);
//             },0);
//         }
//     }
// }


// pub struct DrawTextSys {
//     pub canvas: Object,
// }

// impl DrawTextSys {
//     pub fn new() -> Self {
//         let obj: Object = TryInto::try_into(js! {
// 			var c = document.createElement("canvas");
// 			// c.style = "position:absolute;bottom:20px;z-index:100000";
//             // document.body.append(c);// 查看效果
//             var ctx = c.getContext("2d");
//             return {canvas: c, ctx: ctx, wrap: c};
//         })
//         .unwrap();
//         DrawTextSys { canvas: obj }
//     }

//     pub fn run(&mut self, world_id: u32) {
//         let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
//         let world = &mut world.gui;
// 		let font_sheet = world.font_sheet.lend_mut();
// 		let font_sheet = &mut font_sheet.borrow_mut();
//         if font_sheet.wait_draw_list.len() == 0 {
//             return;
//         }

//         let list = std::mem::replace(&mut font_sheet.wait_draw_list, Vec::default());
//         let ptr = Box::into_raw(Box::new(list)) as usize as u32;

//         font_sheet.wait_draw_map.clear();
//         js! {
//             var p = @{ptr};
//             setTimeout(function(){
//                 Module._draw_canvas_text(@{world_id}, p);
//             },0);
//         }
//     }
// }

// 解析msdf文字配置
#[inline]
fn parse_msdf_font_res(value: &[u8], font_sheet: &mut FontSheet, name: u32, mut tex_font: TexFont) -> Result<(), String> {
    let mut offset = 12;

    match String::from_utf8(Vec::from(&value[0..11])) {
        Ok(s) => {
            if s != "GLYPL_TABLE".to_string() {
                return Err("parse error, it's not GLYPL_TABLE".to_string());
            }
        }
        Err(s) => return Err(s.to_string()),
    };

	tex_font.ascender = value.get_lf32(offset);
	offset += 4;
	tex_font.descender = value.get_lf32(offset);
	offset += 4;

	tex_font.font_size = value.get_lf32(offset);
	offset += 4;
	// log::info!("font_size=================={:?}", tex_font.font_size);
	

    let name_len = value.get_u8(offset);
    offset += 1;
    // let name_str = match String::from_utf8(Vec::from(&value[offset..offset + name_len as usize])) {
    //     Ok(s) => s,
    //     Err(s) => return Err(s.to_string()),
    // };
    offset += name_len as usize;

	// 基数变偶数
	if offset % 2 > 0 {
		offset += 1;
	}

	// log::info!("load================= {:?}", name_str);

	tex_font.name = name as usize;



	// 跳过不解析
	// line_height: 2字节,
	// atlas_width: 2字节,
	// atlas_height: 2字节,
	// padding: 2字节,
    offset += 8;
	// log::info!("offset=================={:?}",offset);

    loop {
        if offset >= value.len() {
            break;
        }
        let ch = unsafe { transmute(value.get_lu16(offset) as u32) };
        offset += 2;
        let glyph = Glyph::parse(value, &mut offset);
        let index = font_sheet.char_slab.insert((ch, glyph));
        font_sheet
            .char_map
            .insert((tex_font.name.clone(), 0, 0, 0, ch), index);
    }
	
    font_sheet.src_map.insert(tex_font.name.clone(), tex_font);

    Ok(())
}
