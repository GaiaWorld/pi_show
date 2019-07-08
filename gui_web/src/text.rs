use std::mem::{transmute};

use stdweb::unstable::TryInto;
use stdweb::web::{ TypedArray };
use stdweb::{Object, UnsafeTypedArray};

use share::Share;
use atom::Atom;
use ecs::{LendMut};
use hal_core::*;
use hal_webgl::*;

use gui::component::user::*;
use gui::render::res::{ TextureRes, Opacity };
use gui::font::sdf_font::{DefaultSdfFont, Glyph, SdfFont};
pub use gui::layout::{YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType};
use GuiWorld;

#[macro_use()]
macro_rules! set_attr {
    ($world:ident, $node_id:ident, $tt:ident, $name:ident, $value:expr, $key: ident) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut GuiWorld)};
        let attr = world.$key.lend_mut();
        let value = $value;
        $crate::paste::item! {
            match attr.get_write(node_id) {
                Some(mut r) => r.[<set_ $name>](value),
                _ =>{
                    let mut v = $tt::default();
                    v.$name = value;
                    attr.insert(node_id, v);
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
    world.text_shadow.lend_mut().insert(node_id, value);
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
    let world1 = unsafe {&mut *(world as usize as *mut GuiWorld)};
    let node = node_id as usize;
    let texts = world1.text.lend_mut();
    match texts.get(node) {
        Some(t) => look_text(world, node, t.0.as_str()),
        None => (),
    };
    
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
    let name: String = js!(return __jsObj2;).try_into().unwrap();
    let name = Atom::from(name);
    let cfg: TypedArray<u8> = js!(return __jsObj;).try_into().unwrap();
    let cfg = cfg.to_vec();
    let width: u32 = js!(return __jsObj1.width;).try_into().unwrap();
    let height: u32 = js!(return __jsObj1.height;).try_into().unwrap();
    let engine = world.engine.lend_mut();
    let font_sheet = world.font_sheet.lend_mut();

    println!("----------add_sdf_font_res{:?}", name);
    let texture = match TryInto::<Object>::try_into(js!{return {wrap: __jsObj1};}) {
        Ok(image_obj) => engine.gl.create_texture_2d_webgl(width, height, 0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &image_obj).unwrap(),
        Err(_) => panic!("set_src error"),
    };

    let texture_res = TextureRes::<WebGLContextImpl>::new(name.clone(), width as usize, height as usize, unsafe{transmute(Opacity::Translucent)}, unsafe{transmute(0 as u8)}, texture);
    let texture_res = engine.res_mgr.create(texture_res);
    // new_width_data
    let mut sdf_font = DefaultSdfFont::<WebGLContextImpl>::new(texture_res.clone(), dyn_type as usize);
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

    // 生成不存在的字体
    look_text(world_id, node, value.as_str());

    world.text.lend_mut().insert(node as usize, Text(Share::new(value)));
    debug_println!("set_text_content");  
}

// 动态字体纹理回调
// __jsObj: canvas __jsObj1: string, (字体名称)__jsObj2: u32Array, (draw 字符)
#[allow(unused_attributes)]
#[no_mangle]
pub fn update_font_texture(world: u32){
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
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
            src.texture().bind.update(u1 as u32, v1 as u32, width, height, &TextureData::U8(data.as_slice()));
            let render_objs = world.render_objs.lend_mut();
            render_objs.get_notify().modify_event(1, "", 0);
        },
        Err(_) => panic!("update_font_texture error"),
    };
}

// __jsObj: canvas
fn update_font_texture1(world: u32, font_name: String, chars: &Vec<u32>, u: u32, v: u32, width: u32, height: u32) {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
    let font_name = Atom::from(font_name);
    let font_sheet = world.font_sheet.lend_mut();
    let src = match font_sheet.get_src(&font_name) {
        Some(r) => r,
        None => panic!("update_font_texture error, font is not exist: {}", font_name.as_ref()),
    };

    if chars.len() == 0 {
        return;
    }

    
    src.texture().bind.update_webgl(u, v, width, height, &TryInto::<Object>::try_into(js!{return {wrap: __jsObj};}).unwrap() );
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

fn look_text(world_id: u32, node: usize, text: &str){
    let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};
    let fonts = world.font.lend_mut();
    let default_table = world.default_table.lend_mut();

    let font = match fonts.get(node) {
        Some(r) => r,
        None => default_table.get_unchecked::<Font>(),
    };
    let font_sheet = world.font_sheet.lend_mut();

    match font_sheet.get_first_font(&font.family) {
        Some(r) => {
            let mut chars: Vec<char> = text.chars().collect();
            let mut i = 0;
            loop {
                if i >= chars.len() {
                    break;
                }

                match r.get_glyph(&chars[i]) {
                    Some(_g) => {chars.swap_remove(i);},
                    None => {
                        let mut is_remove = false;
                        for j in 0..i {
                            if chars[j] == chars[i] {
                                chars.swap_remove(i);
                                is_remove = true;
                                break;
                            }
                        }
                        if !is_remove {
                            i += 1;
                        }
                    },
                }
            }
            if chars.len() == 0 {
                return;
            }
            let chars: Vec<u32> = unsafe {transmute(chars)};
        
            if r.get_dyn_type() == 0 {
                let gl = gen_font(world_id, r.name().as_ref(), chars.as_slice());
                for v in gl.into_iter() {
                    r.add_glyph(v.id, v);
                }
            } else {
                gen_canvas_text(world_id, &r, &chars)
            } 
        },
        None => ()
    }
}


// 生成canvas字体
fn gen_canvas_text(world: u32, font: &Share<dyn SdfFont<Ctx = WebGLContextImpl>>, chars: &Vec<u32>) {
    let name = font.name();
    let font_name = name.as_ref();
    let c: Object = js!{
        var c = document.createElement("canvas");
        var ctx = c.getContext("2d");
        ctx.font = @{font_name};
        return {canvas: c, ctx: ctx};
    }.try_into().unwrap();
    let info = calc_canvas_text(&c, font, chars);

    draw_canvas_text(world, font_name, font.stroke_width(), font.line_height(), chars, info);
}

fn draw_canvas_text(world: u32, font_name: &str, stroke_width: f32, line_height: f32, chars: &Vec<u32>, info: Vec<TextInfo>) {
    let mut i = 0;
    for text_info in info.iter() {
        let c: Object = js!{
            var canvas = document.createElement("canvas");
            canvas.width = @{text_info.end.u - text_info.start.u};
            canvas.height = @{text_info.end.v - text_info.start.v};
            var ctx = canvas.getContext("2d");
            ctx.font = @{font_name};       
            return {canvas: canvas, ctx: ctx};
        }.try_into().unwrap();

        js!{
            var ctx = @{&c}.ctx;
            ctx.fillStyle = "#00f";
		    ctx.fillRect(0, 0, @{text_info.end.u - text_info.start.u}, @{text_info.end.v - text_info.start.v});
        }
        if stroke_width > 0.0 {
            js!{
                var ctx = @{&c}.ctx;
                ctx.lineWidth = @{ stroke_width };
                ctx.strokeStyle = "#ff0000";
                ctx.fillStyle = "#00ff00";
                ctx.textBaseline = "bottom";
            }
            for uv in text_info.list.iter() {
                js!{
                    @{&c}.ctx.strokeText(String.fromCharCode(@{&chars[i]}), @{uv.u + stroke_width}, @{uv.v + line_height + stroke_width});
                    @{&c}.ctx.fillText(String.fromCharCode(@{&chars[i]}), @{uv.u + stroke_width}, @{uv.v + line_height + stroke_width});
                }
                i += 1;
            }
        } else {
            js!{
                var ctx = @{&c}.ctx;
                ctx.lineWidth = @{ stroke_width };
                ctx.fillStyle = "#00ff00";
                ctx.textBaseline = "bottom";
            }
            for uv in text_info.list.iter() {
                js!{
                    @{&c}.ctx.fillText(String.fromCharCode(@{&chars[i]}), @{uv.u}, @{uv.v + line_height});
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
            (text_info.end.u - text_info.start.u) as u32,
            (text_info.end.v - text_info.start.v) as u32
        );
    }
}

fn calc_canvas_text(
    cc: &Object,
    font: &Share<dyn SdfFont<Ctx = WebGLContextImpl>>,
    chars: &Vec<u32>,
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
        let w = TryInto::<u32>::try_into(js! { return @{cc}.ctx.measureText(String.fromCharCode(@{c})).width + @{stroke_width}; }).unwrap() as f32;
        
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

#[derive(Debug)]
struct TextInfo {
    list: Vec<UV>,
    start: UV,
    end: UV,
}

#[derive(Debug)]
struct UV {
    u: f32,
    v: f32,
}