/// 将设置文本属性的接口导出到js
use std::mem::transmute;

use wasm_bindgen::prelude::*;
use web_sys::HtmlImageElement;
use js_sys::Object;


use atom::Atom;
use data_view::GetView;
use ecs::LendMut;
use gui::component::user::*;
use gui::font::font_sheet::{FontSheet, Glyph, TexFont};
use hal_core::*;
use world::{GuiWorld, next_power_of_two};

#[macro_use()]
macro_rules! set_attr {
    ($world:ident, $node_id:ident, $name:ident, $name1:ident, $name2: expr, $value:expr, $key: ident) => {
        let node_id = $node_id as usize;
        let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
        let attr = world.gui.$key.lend_mut();
        let value = $value;
        $crate::paste::item! {
            let r = unsafe { attr.get_unchecked_mut(node_id) };
            r.$name.$name1 = value;
            attr.get_notify_ref().modify_event(node_id, $name2, 0);
        }
        debug_println!("set_{}", $name2);
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

/// 设置文字阴影
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_text_shadow(
    world: u32,
    node_id: u32,
    h: f32,
    v: f32,
    blur: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    let value = TextShadow {
        h: h,
        v: v,
        blur: blur,
        color: CgColor::new(r, g, b, a),
    };
    let node_id = node_id as usize;
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let text_styles = world.text_style.lend_mut();
    let r = match text_styles.get_mut(node_id) {
        Some(r) => r,
        None => {
            text_styles.insert_no_notify(node_id, TextStyle::default());
            unsafe { text_styles.get_unchecked_mut(node_id) }
        }
    };
    r.shadow = value;
    text_styles
        .get_notify_ref()
        .modify_event(node_id, "text_shadow", 0);
    debug_println!("set_text_shadow");
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
pub fn set_font_family(world: u32, node_id: u32, name: String) {
    set_attr!(
        world,
        node_id,
        font,
        family,
        "font_family",
        Atom::from(name),
        text_style
    );
}

/// 添加一个msdf字体资源
/// 图片            配置       
///__jsObj: image , __jsObj1: glyph cfg
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn add_msdf_font_res(world_id: u32, image: HtmlImageElement, cfg: &[u8]) {
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let cfg = cfg.to_vec();
    let width: u32 = image.width();
    let height: u32 = image.height();
    let font_sheet = world.font_sheet.lend_mut();

    if width > 2048 {
        debug_println!("add_msdf_font_res fail, width > 2048");
    }

    update_text_texture(world_id, 0, 0, height,image);

    parse_msdf_font_res(cfg.as_slice(), font_sheet).unwrap();
    font_sheet.font_tex.last_v += height as f32;
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

/// 添加一个canvas字体
/// __jsObj1: name(String)
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn add_canvas_font(world: u32, factor: f32, name: String) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let font_sheet = world.font_sheet.lend_mut();
    font_sheet.set_src(Atom::from(name), true, factor);
}

/// 添加font-face
///          字体族名称                        字体名称（逗号分隔）     
/// __jsObj: family_name(String), __jsObj1: src_name(String, 逗号分隔),
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn add_font_face(world: u32, oblique: f32, size: u32, weight: u32, family: String, src: String) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let font_sheet = world.font_sheet.lend_mut();
    font_sheet.set_face(
        Atom::from(family),
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
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let font_sheet = world.font_sheet.lend_mut();
    let engine = world.engine.lend_mut();
    let texture = font_sheet.get_font_tex();

    let mut end_v = v + height;
    if end_v > texture.height as u32 {
        end_v = next_power_of_two(end_v);
        if end_v > 2048 {
            debug_println!("update_canvas_text fail, height overflow");
        }
        engine
            .gl
            .texture_extend(&texture.bind, texture.width as u32, end_v);
        texture.update_size(texture.width, end_v as usize);
        font_sheet.get_notify().modify_event(0, "", 0);
	}
    engine.gl.texture_update_webgl(
        &texture.bind,
        0,
        u,
        v,
        &Object::from(img),
    );
}

#[derive(Debug, Serialize)]
pub struct TextInfo {
    pub font: String,
    pub font_size: usize,
    pub stroke_width: usize,
    pub weight: usize,
    pub size: (f32, f32),
    pub chars: Vec<WaitChar>,
}

#[derive(Debug, Serialize)]
pub struct WaitChar {
    ch: char,
    width: f32,
    x: u32,
    y: u32,
}

#[derive(Debug, Serialize)]
pub struct TextInfoList {
    list: Vec<TextInfo>,
}
// 解析msdf文字配置
#[inline]
fn parse_msdf_font_res(value: &[u8], font_sheet: &mut FontSheet) -> Result<(), String> {
    let mut offset = 12;
    let mut tex_font = TexFont {
        name: Atom::from(""),
        is_pixel: false,
        factor: 1.0,
    };

    match String::from_utf8(Vec::from(&value[0..11])) {
        Ok(s) => {
            if s != "GLYPL_TABLE".to_string() {
                return Err("parse error, it's not GLYPL_TABLE".to_string());
            }
        }
        Err(s) => return Err(s.to_string()),
    };

    let name_len = value.get_u8(offset);
    offset += 1;
    let name_str = match String::from_utf8(Vec::from(&value[offset..offset + name_len as usize])) {
        Ok(s) => s,
        Err(s) => return Err(s.to_string()),
    };
    offset += name_len as usize;
    tex_font.name = Atom::from(name_str);

    offset += 13; // 遵循 旧的配置表结构， 若配置表结构更新， 再来改此处 TODO
                  //字符uv表
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
