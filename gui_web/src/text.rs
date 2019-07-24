use std::mem::{transmute};

use stdweb::unstable::TryInto;
use stdweb::web::{ TypedArray };
use stdweb::{Object, UnsafeTypedArray};

use share::Share;
use atom::Atom;
use ecs::{LendMut};
use hal_core::*;
use fx_hashmap::FxHashMap32;

use gui::component::user::*;
use gui::render::res::{ TextureRes, Opacity };
use gui::font::sdf_font::{DefaultSdfFont, Glyph, SdfFont};
use gui::font::font_sheet::FontSheet;
pub use gui::layout::{YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType};
use GuiWorld;

#[macro_use()]
macro_rules! set_attr {
    ($world:ident, $node_id:ident, $tt:ident, $name:ident, $value:expr, $key: ident) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut GuiWorld)};
		let world = &mut world.gui;
        let attr = world.$key.lend_mut();
        let value = $value;
        $crate::paste::item! {
            match attr.get_write(node_id) {
                Some(mut r) => r.[<set_ $name>](value),
                _ =>{
                    attr.insert(node_id, $tt::default());
                    unsafe { attr.get_unchecked_write(node_id).[<set_ $name>](value) };
                }
            }
        }
        debug_println!("set_{}", $name);
    };
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_letter_spacing(world: u32, node_id: u32, value: f32){
    let letter_spacing = 0;
    set_attr!(world, node_id, TextStyle, letter_spacing, value, text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_rgba_color(world: u32, node_id: u32, r: f32, g: f32, b: f32, a: f32){
    let color = 0;
    set_attr!(world, node_id, TextStyle, color, Color::RGBA(CgColor::new(r, g, b, a)), text_style);
}

// __jsObj: color_and_positions: [r, g, b, a, pos,   r, g, b, a, pos], direction: 0-360度
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_linear_gradient_color(world: u32, node_id: u32, direction: f32){
    let color_and_positions: TypedArray<f32> = js!(return __jsObj;).try_into().unwrap();
    let value = Color::LinearGradient(to_linear_gradient_color(color_and_positions.to_vec(), direction));
    let color = 0;
    set_attr!(world, node_id, TextStyle, color, value, text_style);
}


#[allow(unused_attributes)]
#[no_mangle]
pub fn set_line_height_normal(world: u32, node_id: u32){
    let line_height = 0;
    set_attr!(world, node_id, TextStyle, line_height, LineHeight::Normal, text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_line_height(world: u32, node_id: u32, value: f32){
    let line_height = 0;
    set_attr!(world, node_id, TextStyle, line_height, LineHeight::Length(value), text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_line_height_percent(world: u32, node_id: u32, value: f32){
    let line_height = 0;
    set_attr!(world, node_id, TextStyle, line_height, LineHeight::Percent(value), text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_indent(world: u32, node_id: u32, value: f32){
    let indent = 0;
    set_attr!(world, node_id, TextStyle, indent, value, text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_stroke(world: u32, node_id: u32, width: f32, r: f32, g: f32, b: f32, a: f32){
    let stroke = 0;
    set_attr!(world, node_id, TextStyle, stroke, Stroke {
        width,
        color: CgColor::new(r, g, b, a),
    }, text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_white_space(world: u32, node_id: u32, value: u8){
    let white_space = 0;
    set_attr!(world, node_id, TextStyle, white_space, unsafe {transmute(value)}, text_style);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_shadow_h(world: u32, node_id: u32, value: f32){
    let h = 0;
    set_attr!(world, node_id, TextShadow, h, value, text_shadow);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_shadow_v(world: u32, node_id: u32, value: f32){
    let v = 0;
    set_attr!(world, node_id, TextShadow, v, value, text_shadow);
 
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_shadow_blur(world: u32, node_id: u32, value: f32){
    let blur = 0;
    set_attr!(world, node_id, TextShadow, blur, value, text_shadow);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_shadow_color(world: u32, node_id: u32, r: f32, g: f32, b: f32, a: f32){
    let color = 0;
    set_attr!(world, node_id, TextShadow, color, CgColor::new(r, g, b, a), text_shadow);
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
    let ts = world.text_shadow.lend_mut();
    ts.insert(node_id, value);
    unsafe { ts.get_unchecked_write(node_id).modify(|_|{
        return true;
    }) };
    debug_println!("set_text_shadow"); 
}


#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_style(world: u32, node_id: u32, value: u8){
    let style = 0;
    set_attr!(world, node_id, Font, style, unsafe {transmute(value)}, font);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_weight(world: u32, node_id: u32, value: f32){
    let weight = 0;
    set_attr!(world, node_id, Font, weight, value, font);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_size_none(world: u32, node_id: u32){
    let size = 0;
    set_attr!(world, node_id, Font, size, FontSize::None, font);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_size(world: u32, node_id: u32, value: f32){
    let size = 0;
    set_attr!(world, node_id, Font, size, FontSize::Length(value), font);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_size_percent(world: u32, node_id: u32, value: f32){
    let size = 0;
    set_attr!(world, node_id, Font, size, FontSize::Percent(value), font);
}

// __jsObj: family name
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_family(world: u32, node_id: u32){
    let value: String = js!(return __jsObj;).try_into().unwrap();

    // 生成不存在的字体
    // let world1 = unsafe {&mut *(world as usize as *mut GuiWorld)};
    // let world1 = &mut world1.gui;
    // let node = node_id as usize;
    // let texts = world1.text.lend_mut();
    // match texts.get(node) {
    //     Some(t) => look_text(world, node, t.0.as_str()),
    //     None => (),
    // };
    
    let family = 0;
    set_attr!(world, node_id, Font, family, Atom::from(value), font);
    debug_println!("set_font_family"); 
}

//           配置       图片                     图片名称
//__jsObj: uv cfg, __jsObj1: image | canvas, __jsObj2: name(String)
#[allow(unused_attributes)]
#[no_mangle]
pub fn add_sdf_font_res(world: u32, dyn_type: u32) {
    if dyn_type > 0 {
        js!{
            var ctx = __jsObj1.getContext("2d");
            ctx.fillStyle = "#00f";
		    ctx.fillRect(0, 0, __jsObj1.width, __jsObj1.height);
        }
    }
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let name: String = js!(return __jsObj2;).try_into().unwrap();
    let name = Atom::from(name);
    let cfg: TypedArray<u8> = js!(return __jsObj;).try_into().unwrap();
    let cfg = cfg.to_vec();
    let width: u32 = js!(return __jsObj1.width;).try_into().unwrap();
    let height: u32 = js!(return __jsObj1.height;).try_into().unwrap();
    let engine = world.engine.lend_mut();
    let font_sheet = world.font_sheet.lend_mut();

    let texture = match TryInto::<Object>::try_into(js!{return {wrap: __jsObj1};}) {
        Ok(image_obj) => engine.gl.texture_create_2d_webgl(width, height, 0, PixelFormat::RGBA, DataFormat::UnsignedByte, false, &image_obj).unwrap(),
        Err(_) => panic!("set_src error"),
    };

    let texture_res = TextureRes::new(width as usize, height as usize, unsafe{transmute(Opacity::Translucent)}, unsafe{transmute(0 as u8)}, Share::new(texture));
    let texture_res = engine.res_mgr.create(name, texture_res);
    // new_width_data
    let mut sdf_font = DefaultSdfFont::new(texture_res, dyn_type as usize);
    sdf_font.parse(cfg.as_slice()).unwrap();
    // _println!("sdf_font parse end: name: {:?}, {:?}", &sdf_font.name, &sdf_font.glyph_table);

    font_sheet.set_src(sdf_font.name(), Share::new(sdf_font));
}

//          字体族名称                        字体名称（逗号分隔）     
// __jsObj: family_name(String), __jsObj1: src_name(String, 逗号分隔), 
#[allow(unused_attributes)]
#[no_mangle]
pub fn add_font_face(world: u32, oblique: f32, size: f32, weight: f32){
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let family: String = js!(return __jsObj;).try_into().unwrap();
    let src: String = js!(return __jsObj1;).try_into().unwrap();
    let font_sheet = world.font_sheet.lend_mut();
    
    font_sheet.set_face(Atom::from(family), oblique, size, weight, src);
}

// __jsObj 文字字符串
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_content(world_id: u32, node: u32){
    let value: String = js!(return __jsObj;).try_into().unwrap();
    let node = node as usize;
    let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};
    world.draw_text_sys.text_ids.push(node as usize);
    // look_text(world_id, node, value.as_str());
	let world = &mut world.gui;

    // 生成不存在的字体

    world.text.lend_mut().insert(node as usize, Text(value, Atom::from("")));
    debug_println!("set_text_content");  
}

// 动态字体纹理回调
// __jsObj: canvas __jsObj1: string, (字体名称)__jsObj2: u32Array, (draw 字符)
#[allow(unused_attributes)]
#[no_mangle]
pub fn update_font_texture(world: u32){
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let font_name: String = js!(return __jsObj1;).try_into().unwrap();
    let font_name = Atom::from(font_name);
    let font_sheet = world.font_sheet.lend_mut();
    let src = match font_sheet.get_src(&font_name) {
        Some(r) => r,
        None => panic!("update_font_texture error, font is not exist: {}", font_name.as_ref()),
    };

    let chars = TryInto::<TypedArray<u32>>::try_into(js!{return __jsObj2;}).unwrap().to_vec();
    if chars.len() == 0 {
        return;
    }
    let (mut u1, mut v1, mut u2, mut v2) = (std::f32::MAX, std::f32::MAX, std::f32::MIN, std::f32::MIN);
    for c in chars.into_iter() {
        let c: char = unsafe { transmute(c) };
        match src.get_glyph(&c) {
            Some(glyph) => {
                if u1 > glyph.x {
                    u1 = glyph.x;
                }
                if v1 > glyph.y {
                    v1 = glyph.y;
                }
                let g_u2 = glyph.x + glyph.width;
                let g_v2 = glyph.y + glyph.height;
                if u2 < g_u2 {
                    u2 = g_u2;
                }
                if v2 < g_v2 {
                    v2 = g_v2;
                }
            },
            None => panic!("glyph is not exist: {}", c),
        }
    }
    
    let width = (u2-u1) as u32;
    let height = (v2-v1) as u32;

    // 优化， getImageData性能不好， 应该直接更新canvas， TODO
    match TryInto::<TypedArray<u8>>::try_into(js!{return new Uint8Array(__jsObj.getContext("2d").getImageData(@{u1 as u32}, @{v1 as u32}, @{width}, @{height}).data.buffer);} ) {
        Ok(data) => {
            let data = data.to_vec();
            let engine = world.engine.lend_mut();
            engine.gl.texture_update(&src.texture().bind, 0, &TextureData::U8(u1 as u32, v1 as u32, width, height, data.as_slice()));
            let render_objs = world.render_objs.lend_mut();
            render_objs.get_notify().modify_event(1, "", 0);
        },
        Err(_) => panic!("update_font_texture error"),
    };
}

// __jsObj: canvas
fn update_font_texture1(world: u32, font_name: String, chars: &[u32], u: u32, v: u32) {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let font_name = Atom::from(font_name);
    let font_sheet = world.font_sheet.lend_mut();
    let engine = world.engine.lend_mut();
    let src = match font_sheet.get_src(&font_name) {
        Some(r) => r,
        None => panic!("update_font_texture error, font is not exist: {}", font_name.as_ref()),
    };

    if chars.len() == 0 {
        return;
    }

    engine.gl.texture_update_webgl(&src.texture().bind, 0, u, v, &TryInto::<Object>::try_into(js!{return {wrap: __jsObj};}).unwrap());
    let render_objs = world.render_objs.lend_mut();
    render_objs.get_notify().modify_event(1, "", 0);

    // 优化， getImageData性能不好， 应该直接更新canvas， TODO
    // match TryInto::<TypedArray<u8>>::try_into(js!{return new Uint8Array(__jsObj.getContext("2d").getImageData(0, 0, @{width}, @{height}).data.buffer);} ) {
    //     Ok(data) => {
    //         let data = data.to_vec();
    //         src.texture().bind.update_webgl(u, v, width, height, &TextureData::U8(data.as_slice()));
    //         let render_objs = world.render_objs.lend_mut();
    //         render_objs.get_notify().modify_event(1, "", 0);
    //     },
    //     Err(_) => panic!("update_font_texture error"),
    // };
}

// __jsObj: canvas
#[allow(unused_attributes)]
#[no_mangle]
pub fn update_font_texture2(world: u32, u: u32, v: u32) {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let font_name: String = js!(return __jsObj1;).try_into().unwrap();
    let font_name = Atom::from(font_name);
    let font_sheet = world.font_sheet.lend_mut();
    let engine = world.engine.lend_mut();
    let src = match font_sheet.get_src(&font_name) {
        Some(r) => r,
        None => panic!("update_font_texture error, font is not exist: {}", font_name.as_ref()),
    };
    
    engine.gl.texture_update_webgl(&src.texture().bind, 0, u, v, &TryInto::<Object>::try_into(js!{return {wrap: __jsObj};}).unwrap());
    let render_objs = world.render_objs.lend_mut();
    render_objs.get_notify().modify_event(1, "", 0);

    // 优化， getImageData性能不好， 应该直接更新canvas， TODO
    // match TryInto::<TypedArray<u8>>::try_into(js!{return new Uint8Array(__jsObj.getContext("2d").getImageData(0, 0, @{width}, @{height}).data.buffer);} ) {
    //     Ok(data) => {
    //         let data = data.to_vec();
    //         src.texture().bind.update_webgl(u, v, width, height, &TextureData::U8(data.as_slice()));
    //         let render_objs = world.render_objs.lend_mut();
    //         render_objs.get_notify().modify_event(1, "", 0);
    //     },
    //     Err(_) => panic!("update_font_texture error"),
    // };
}

// pub struct FontGenerator;

// impl MSdfGenerator for FontGenerator{
    

//     // fn gen_mult(&self, _chars: &[char]) -> Vec<Glyph> {
//     //     unimplemented!{}
//     // }
// }

pub fn gen_font(world: u32, name: &str, chars: &[u32]) -> Vec<Glyph> {
    let chars = unsafe { UnsafeTypedArray::<u32>::new(chars) };
    match TryInto::<TypedArray<u8>>::try_into(js!{return __gen_font(@{world}, @{name}, @{chars})}){
        Ok(buffer) => {
            let buffer = buffer.to_vec();
            let mut glyphs = Vec::new();
            let mut i = 0;
            loop {
                glyphs.push(Glyph::parse(buffer.as_slice(), &mut i));
                if i >= buffer.len() {
                    break;
                }
                // let glyph = Glyph::parse(buffer.as_slice(), &mut 0);
            }
            
            glyphs
        },
        Err(_) => panic!("gen font error"),
    }
}

// fn look_text(world_id: u32, node: usize, text: &str){
//     let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};
// 	let world = &mut world.gui;
//     let fonts = world.font.lend_mut();
//     let default_table = world.default_table.lend_mut();

//     let font = match fonts.get(node) {
//         Some(r) => r,
//         None => default_table.get_unchecked::<Font>(),
//     };
//     let font_sheet = world.font_sheet.lend_mut();

//     match font_sheet.get_first_font(&font.family) {
//         Some(r) => {
//             let mut chars: Vec<char> = text.chars().collect();
//             let mut i = 0;
//             loop {
//                 if i >= chars.len() {
//                     break;
//                 }

//                 match r.get_glyph(&chars[i]) {
//                     Some(_g) => {chars.swap_remove(i);},
//                     None => {
//                         let mut is_remove = false;
//                         for j in 0..i {
//                             if chars[j] == chars[i] {
//                                 chars.swap_remove(i);
//                                 is_remove = true;
//                                 break;
//                             }
//                         }
//                         if !is_remove {
//                             i += 1;
//                         }
//                     },
//                 }
//             }
//             if chars.len() == 0 {
//                 return;
//             }
//             let chars: Vec<u32> = unsafe {transmute(chars)};
        
//             if r.get_dyn_type() == 0 {
//                 let gl = gen_font(world_id, r.name().as_ref(), chars.as_slice());
//                 for v in gl.into_iter() {
//                     r.add_glyph(v.id, v);
//                 }
//             } else {
//                 // gen_canvas_text(world_id, &r, &chars, &self.canvas_obj)
//             } 
//         },
//         None => ()
//     }
// }


// 生成canvas字体
fn gen_canvas_text(world: u32, font: &Share<dyn SdfFont>, chars: &[u32], c: &Object) {
    let name = font.name();
    let font_name = name.as_ref();
    js!{
        @{c}.ctx.font = @{font_name};
    }
    let info = calc_canvas_text(&c, font, chars);
    let chars = TypedArray::<u32>::from(chars);
    js!{
        setTimeout(function () {
            window.__draw_text_canvas(@{world}, @{font_name}, @{font.stroke_width()}, @{font.line_height()}, @{chars}, @{info}, @{c});
        }, 0);
    }
    
    // draw_canvas_text(world, font_name, font.stroke_width(), font.line_height(), chars, info, c);
}

fn draw_canvas_text(world: u32, font_name: &str, stroke_width: f32, line_height: f32, chars: &[u32], info: Vec<TextInfo>, c: &Object) {
    let mut i = 0;
    for text_info in info.iter() {
        js!{
            @{c}.canvas.width = @{text_info.end.u - text_info.start.u};
            @{c}.canvas.height = @{text_info.end.v - text_info.start.v}; 
            @{c}.ctx.fillStyle = "#00f"; 
            @{c}.ctx.font = @{font_name};
		    @{c}.ctx.fillRect(0, 0, @{text_info.end.u - text_info.start.u}, @{text_info.end.v - text_info.start.v});
        }
        if stroke_width > 0.0 {
            js!{
                var ctx = @{c}.ctx;
                ctx.lineWidth = @{ stroke_width };
                ctx.fillStyle = "#0f0";
                ctx.strokeStyle = "#f00";
                ctx.textBaseline = "bottom";
            }
            for uv in text_info.list.iter() {
                js!{
                    @{c}.ctx.strokeText(String.fromCharCode(@{&chars[i]}), @{uv.u + stroke_width}, @{uv.v + line_height + stroke_width});
                    @{c}.ctx.fillText(String.fromCharCode(@{&chars[i]}), @{uv.u + stroke_width}, @{uv.v + line_height + stroke_width});
                }
                i += 1;
            }
        } else {
            js!{
                var ctx = @{c}.ctx;
                ctx.fillStyle = "#0f0";
                ctx.textBaseline = "bottom";
            }
            for uv in text_info.list.iter() {
                js!{
                    @{c}.ctx.fillText(String.fromCharCode(@{&chars[i]}), @{uv.u}, @{uv.v + line_height});
                }
                i += 1;
            }
        }
        
        js!{
            window.__jsObj = @{&c}.canvas;
            // document.body.append(@{&c}.canvas);// 查看效果 
        }
        update_font_texture1(
            world,
            font_name.to_string(),
            chars,
            text_info.start.u as u32,
            text_info.start.v as u32,
        );
    }
}

fn calc_canvas_text(
    cc: &Object,
    font: &Share<dyn SdfFont>,
    chars: &[u32],
) -> Vec<TextInfo>{
    let max_width = font.atlas_width() as f32;
    let max_height = font.atlas_width() as f32;
    let stroke_width = font.stroke_width() * 2.0;
    let (mut u, mut v) = font.curr_uv();
    let line_height = font.line_height() + stroke_width;
    let mut arr = Vec::new();

    let start_u = u;
    let mut info = TextInfo{
        list: Vec::new(),
        start: UV{u: u, v: v},
        end: UV{u: u, v: v + line_height},
    };
    let mut start_uv = UV{u: u, v: v};

    for c in chars.iter() {
        let w = TryInto::<f64>::try_into(js! { return @{cc}.ctx.measureText(String.fromCharCode(@{c})).width + @{stroke_width}; }).unwrap() as f32;
        
        // 换行
        if w > max_width - u {
            u = 0.0;
            v += line_height;
            if v + line_height > max_height {
                break;
            }
            if info.start.u == start_u {
                if info.list.len() > 0 {
                    arr.push(info);
                }
                info = TextInfo{
                    list: Vec::new(),
                    start: UV{u: u, v: v},
                    end: UV{u: u, v :v + line_height},
                };
                start_uv = UV{u: u, v: v};
            } else {
                info.end.v += line_height;
            }
        }

        let w1 = u + w;
        if info.end.u < w1 {
            info.end.u = w1;
        }
        info.list.push(UV{u: u - start_uv.u , v: v - start_uv.v});

        font.add_glyph(unsafe{transmute(*c)}, Glyph{
            id: unsafe{transmute(*c)},
            x: u,
            y: v,
            ox: 0.0, 
            oy: 0.0,
            width: w, 
            height: line_height,
            advance: w,
        });
        u += w;
    }
    if info.list.len() > 0 {
        arr.push(info);
    }
    font.set_curr_uv((u, v));
    arr
}

#[derive(Debug, Serialize)]
struct TextInfo {
    list: Vec<UV>,
    start: UV,
    end: UV,
}

js_serializable!( TextInfo );

#[derive(Debug, Serialize)]
struct UV {
    u: f32,
    v: f32,
}

js_serializable!( UV );

pub struct DrawTextSys{
    text_ids: Vec<usize>,
    canvas: Object,
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
            text_ids: Vec::new(),
            canvas: obj,
        }
    }
    pub fn run(&mut self, world_id: u32) {
        if self.text_ids.len() == 0 {
            return;
        }

        let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};
        let world = &mut world.gui;
        let fonts = world.font.lend_mut();
        let texts = world.text.lend_mut();
        // let default_table = world.default_table.lend_mut();
        // let default_font = default_table.get::<Font>();
        let font_sheet = world.font_sheet.lend_mut();
        let class_name = world.class_name.lend_mut();
        let class_sheet = world.class_sheet.lend_mut();

        let mut chars_catch = FxHashMap32::default();
        for id in self.text_ids.iter(){
            let text = match texts.get(*id) {
                Some(r) => r,
                None => continue,
            };
            let font = match fonts.get(*id) {
                Some(r) => r,
                None => {
                    match class_name.get(*id) {
                        Some(class_name) => match class_sheet.class.get(class_name.0) {
                            Some(class) => if class.text > 0 {
                                &unsafe { class_sheet.text.get_unchecked(class.text) }.font
                            }else {
                                continue;
                            },
                            None => continue,
                        },
                        None => continue
                    }
                    // match default_font {
                    //     Some(r) => r,
                    //     None => continue,
                    // }
                },
            };
            Self::look_text(font_sheet, text, font, &mut chars_catch);
        }
        for cache in chars_catch.into_iter() {
            if (cache.1).0.len() == 0 {
                continue;
            }
            let cache_char: Vec<u32> = unsafe {transmute((cache.1).0)};
            if (cache.1).1.get_dyn_type() == 0 {
                let gl = gen_font(world_id, (cache.1).1.name().as_str(), cache_char.as_slice());
                for v in gl.into_iter() {
                    (cache.1).1.add_glyph(v.id, v);
                }
            } else {
                gen_canvas_text(world_id, &(cache.1).1, cache_char.as_slice(), &self.canvas);
            }
        }
        self.text_ids.clear();
    }

    fn look_text(font_sheet: &FontSheet, text: &Text, font: &Font, char_cache: &mut FxHashMap32<Atom, (Vec<char>, Share<dyn SdfFont>)>){
        match font_sheet.get_first_font(&font.family) {
            Some(r) => {
                let char_cache = char_cache.entry(r.name()).or_insert((Vec::new(), r.clone()));
                for c in text.0.chars() {
                    match r.get_glyph(&c) {
                        Some(_) => continue,
                        None => (),
                    };

                    r.add_glyph(c, Glyph::default());
                    char_cache.0.push(c);
                }
            },
            None => ()
        }
    }
}

pub fn define_draw_canvas(){
    js!{
        window.__draw_text_canvas = function(world, font_name, stroke_width, line_height, chars, info, c){
            var k = 0;
            var canvas = c.canvas;
            var ctx = c.ctx;
            for (var i = 0; i < info.length; i++) {
                var text_info = info[i];
                canvas.width = text_info.end.u - text_info.start.u;
                canvas.height = text_info.end.v - text_info.start.v; 
                ctx.fillStyle = "#00f"; 
                ctx.font = font_name;
                ctx.textBaseline = "bottom";
                ctx.fillRect(0, 0, text_info.end.u - text_info.start.u, text_info.end.v - text_info.start.v);
                if (stroke_width > 0.0) {
                    ctx.lineWidth = stroke_width;
                    ctx.fillStyle = "#0f0";
                    ctx.strokeStyle = "#f00";

                    for (var j = 0; j < text_info.list.length; j++) {
                        var uv = text_info.list[j];
                        ctx.strokeText(String.fromCharCode(chars[k]), uv.u + stroke_width, uv.v + line_height + stroke_width);
                        ctx.fillText(String.fromCharCode(chars[k]), uv.u + stroke_width, uv.v + line_height + stroke_width);
                        k += 1;
                    }
                } else {
                    ctx.fillStyle = "#0f0";
                    ctx.textBaseline = "bottom";
                    
                    for (var j = 0; j < text_info.list.length; j++) {
                        var uv = text_info.list[j];
                        ctx.fillText(String.fromCharCode(chars[k]), uv.u, uv.v + line_height);
                        k += 1;
                    } 
                }
                window.__jsObj = canvas;
                window.__jsObj1 = font_name;
                Module._update_font_texture2(world,text_info.start.u,text_info.start.v);
            }
        };
    }
}