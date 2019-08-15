/// 将设置文本属性的接口导出到js
use std::mem::{transmute};

use stdweb::unstable::TryInto;
use stdweb::web::{ TypedArray };
use stdweb::{Object};

use atom::Atom;
use data_view::GetView;
use ecs::{LendMut};
use hal_core::*;
use gui::component::user::*;
use gui::font::font_sheet::{ FontSheet, TexFont, Glyph };
pub use gui::layout::{YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType};
use GuiWorld;

#[macro_use()]
macro_rules! set_attr {
    ($world:ident, $node_id:ident, $tt:ident, $name:ident, $name1:ident, $name2: expr, $value:expr, $key: ident) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut GuiWorld)};
		let world = &mut world.gui;
        let attr = world.$key.lend_mut();
        let value = $value;
        $crate::paste::item! {
            let r = match attr.get_mut(node_id) {
                Some(r) => r,
                None => {
                    attr.insert_no_notify(node_id, $tt::default());
                    unsafe { attr.get_unchecked_mut(node_id) }
                }
            };
            r.$name.$name1 = value;
            attr.get_notify_ref().modify_event(node_id, $name2, 0);
        }
        debug_println!("set_{}", $name2);
    };
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_letter_spacing(world: u32, node_id: u32, value: f32){
    set_attr!(world, node_id, TextStyle, text, letter_spacing, "letter_spacing", value, text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_rgba_color(world: u32, node_id: u32, r: f32, g: f32, b: f32, a: f32){
    set_attr!(world, node_id, TextStyle, text, color, "color", Color::RGBA(CgColor::new(r, g, b, a)), text_style);
}

// __jsObj: color_and_positions: [r, g, b, a, pos,   r, g, b, a, pos], direction: 0-360度
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_linear_gradient_color(world: u32, node_id: u32, direction: f32){
    let color_and_positions: TypedArray<f32> = js!(return __jsObj;).try_into().unwrap();
    let value = Color::LinearGradient(to_linear_gradient_color(color_and_positions.to_vec(), direction));
    set_attr!(world, node_id, TextStyle, text, color, "color", value, text_style);
}


#[allow(unused_attributes)]
#[no_mangle]
pub fn set_line_height_normal(world: u32, node_id: u32){
    set_attr!(world, node_id, TextStyle, text, line_height, "line_height", LineHeight::Normal, text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_line_height(world: u32, node_id: u32, value: f32){
    set_attr!(world, node_id, TextStyle, text, line_height, "line_height", LineHeight::Length(value), text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_line_height_percent(world: u32, node_id: u32, value: f32){
    set_attr!(world, node_id, TextStyle, text, line_height, "line_height", LineHeight::Percent(value), text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_indent(world: u32, node_id: u32, value: f32){
    set_attr!(world, node_id, TextStyle, text, indent, "text_indent", value, text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_align(world: u32, node_id: u32, value: u8){
    set_attr!(world, node_id, TextStyle, text, text_align, "text_align", unsafe { transmute(value) } , text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_stroke(world: u32, node_id: u32, width: f32, r: f32, g: f32, b: f32, a: f32){
    set_attr!(world, node_id, TextStyle, text, stroke, "stroke", Stroke {
        width,
        color: CgColor::new(r, g, b, a),
    }, text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_white_space(world: u32, node_id: u32, value: u8){
    set_attr!(world, node_id, TextStyle, text, white_space, "white_space", unsafe {transmute(value)}, text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_shadow(world: u32, node_id: u32, h: f32, v: f32, r: f32, g: f32, b: f32, a: f32, blur: f32){
    let value = TextShadow {
        h: h,
        v: v,
        blur: blur,
        color: CgColor::new(r, g, b, a),
    };
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
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
    text_styles.get_notify_ref().modify_event(node_id, "text_shadow", 0);
    debug_println!("set_text_shadow"); 
}


#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_style(world: u32, node_id: u32, value: u8){
    set_attr!(world, node_id, TextStyle, font, style, "font_style",  unsafe {transmute(value)}, text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_weight(world: u32, node_id: u32, value: u32){
    set_attr!(world, node_id, TextStyle, font, weight, "font_weight", value as usize, text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_size_none(world: u32, node_id: u32){
    set_attr!(world, node_id, TextStyle, font, size, "font_size", FontSize::None, text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_size(world: u32, node_id: u32, value: f32){
    set_attr!(world, node_id, TextStyle, font, size, "font_size", FontSize::Length(value), text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_size_percent(world: u32, node_id: u32, value: f32){
    set_attr!(world, node_id, TextStyle, font, size, "font_size", FontSize::Percent(value), text_style);
}

// __jsObj: family name
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_family(world: u32, node_id: u32){
    let value: String = js!(return __jsObj;).try_into().unwrap();
    set_attr!(world, node_id, TextStyle, font, family, "font_family", Atom::from(value), text_style);
    debug_println!("set_font_family"); 
}

// 图片            配置       
//__jsObj: image , __jsObj1: glyph cfg
#[allow(unused_attributes)]
#[no_mangle]
pub fn add_msdf_font_res(world_id: u32) {
    let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let cfg: TypedArray<u8> = js!(return __jsObj1;).try_into().unwrap();
    let cfg = cfg.to_vec();
    let width: u32 = js!(return __jsObj.width;).try_into().unwrap();
    let height: u32 = js!(return __jsObj.height;).try_into().unwrap();
    let font_sheet = world.font_sheet.lend_mut();
    
    if width > 2048 {
        println!("add_msdf_font_res fail, width > 2048");
    }

    update_text_texture(world_id, 0, 0, height);

    parse_msdf_font_res(cfg.as_slice(), font_sheet).unwrap();
}

// __jsObj 文字字符串
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_content(world_id: u32, node: u32){
    let value: String = js!(return __jsObj;).try_into().unwrap();
    let node = node as usize;
    let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    world.text_content.lend_mut().insert(node as usize, TextContent(value, Atom::from("")));
    debug_println!("set_text_content");  
}

//__jsObj1: name(String)
#[allow(unused_attributes)]
#[no_mangle]
pub fn add_canvas_font(world: u32, factor: f32) {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let name: String = js!(return __jsObj;).try_into().unwrap();
    let font_sheet = world.font_sheet.lend_mut();
    font_sheet.set_src(Atom::from(name), true, factor);
}

//          字体族名称                        字体名称（逗号分隔）     
// __jsObj: family_name(String), __jsObj1: src_name(String, 逗号分隔), 
#[allow(unused_attributes)]
#[no_mangle]
pub fn add_font_face(world: u32, oblique: f32, size: u32, weight: u32){
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let family: String = js!(return __jsObj;).try_into().unwrap();
    let src: String = js!(return __jsObj1;).try_into().unwrap();
    let font_sheet = world.font_sheet.lend_mut();
    font_sheet.set_face(Atom::from(family), oblique, size as usize, weight as usize, src);
}

// __jsObj: canvas
#[allow(unused_attributes)]
#[no_mangle]
pub fn update_text_texture(world: u32, u: u32, v: u32, height: u32) {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let font_sheet = world.font_sheet.lend_mut();
    let engine = world.engine.lend_mut();
    let texture = font_sheet.get_font_tex();
    
    let mut end_v = v + height;
    if end_v > texture.height as u32 {
        end_v = next_power_of_two(end_v);
        if end_v > 2048 {
            println!("update_canvas_text fail, height overflow");  
        }
        engine.gl.texture_extend(&texture.bind, texture.width as u32, end_v);
        texture.update_size(texture.width, end_v as usize);
        font_sheet.get_notify().modify_event(0, "", 0);
    }
    engine.gl.texture_update_webgl(&texture.bind, 0, u, v, &TryInto::<Object>::try_into(js!{return {wrap: __jsObj};}).unwrap());
}

#[derive(Debug, Serialize)]
pub struct TextInfo {
    pub font: String,
    pub font_size: f32,
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
    list: Vec<TextInfo>
}

js_serializable!( TextInfo );
js_serializable!( TextInfoList );

pub struct DrawTextSys{
    pub canvas: Object,
}

impl DrawTextSys {
    pub fn new() -> Self {
        let obj: Object = TryInto::try_into(js!{
            var c = document.createElement("canvas");
            // document.body.append(c);// 查看效果 
            var ctx = c.getContext("2d");
            return {canvas: c, ctx: ctx};
        }).unwrap();
        DrawTextSys{
            canvas: obj,
        }
    }

    pub fn run(&mut self, world_id: u32) {
        let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};
        let world = &mut world.gui;
        let font_sheet = world.font_sheet.lend_mut();

        let mut info_list = TextInfoList{list: Vec::default()};
        if font_sheet.wait_draw.chars.len() > 0 {
            info_list.list.push(
                TextInfo{
                    font: font_sheet.wait_draw.font.as_ref().to_string(),
                    font_size: font_sheet.wait_draw.font_size,
                    stroke_width: font_sheet.wait_draw.stroke_width,
                    weight: font_sheet.wait_draw.weight,
                    size: (font_sheet.wait_draw.size.x, font_sheet.wait_draw.size.y),
                    chars: unsafe { std::mem::transmute(std::mem::replace(&mut font_sheet.wait_draw.chars, Vec::default())) },
                }
            );
        }

        if font_sheet.wait_draw_list.len() > 0 {
            let list = std::mem::replace(&mut font_sheet.wait_draw_list, Vec::default());
            for mut wait_draw in list.into_iter() {
                info_list.list.push(
                    TextInfo{
                        font: wait_draw.font.as_ref().to_string(),
                        font_size: wait_draw.font_size,
                        stroke_width: wait_draw.stroke_width,
                        weight: wait_draw.weight,
                        size: (wait_draw.size.x, wait_draw.size.y),
                        chars: unsafe { std::mem::transmute(std::mem::replace(&mut wait_draw.chars, Vec::default())) },
                    }
                );
            }
        }
        
        if info_list.list.len() > 0 {
            js!{
                window.__draw_text_canvas(@{world_id}, @{info_list}, @{&self.canvas});
            }
        }
    }
}

fn next_power_of_two(value: u32) -> u32 {
    let mut value = value - 1;
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
    value += 1;
    value
}

// 解析msdf文字配置
#[inline]
fn parse_msdf_font_res(value: &[u8], font_sheet: &mut FontSheet) -> Result<(), String>{
    let mut offset = 12;
    let mut tex_font = TexFont{
        name: Atom::from(""),
        is_pixel: false,
        factor: 1.0,
    };

    match String::from_utf8(Vec::from(&value[0..11])) {
        Ok(s) => if s != "GLYPL_TABLE".to_string() {
            return Err("parse error, it's not GLYPL_TABLE".to_string());
        },
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
        let ch = unsafe{transmute( value.get_lu16(offset) as u32 )} ;
        offset += 2;
        let glyph = Glyph::parse(value, &mut offset);
        let index = font_sheet.char_slab.insert((ch, glyph));
        font_sheet.char_map.insert((tex_font.name.clone(), 0, 0, 0, ch), index);
    }
    font_sheet.src_map.insert(tex_font.name.clone(), tex_font);
    Ok(())
}