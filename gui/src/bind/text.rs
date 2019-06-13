use std::mem::{transmute};
use std::sync::Arc;

use stdweb::unstable::TryInto;
use stdweb::web::TypedArray;
use stdweb::{Object, UnsafeTypedArray};

use atom::Atom;
use ecs::{World, LendMut};
use hal_core::*;
use hal_webgl::*;

use component::user::*;
use single::{ RenderObjs, DefaultTable };
use entity::Node;
use render::res::{ TextureRes, Opacity };
use render::engine::Engine;
use font::sdf_font::{DefaultSdfFont, Glyph, SdfFont};
use font::font_sheet::FontSheet;
pub use layout::{YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType};

#[macro_use()]
macro_rules! set_attr {
    ($world:ident, $node_id:ident, $tt:ident, $name:ident, $value:expr) => {
        let node_id = $node_id as usize;
        let world = unsafe {&mut *($world as usize as *mut World)};
        let attr = world.fetch_multi::<Node, $tt>().unwrap();
        let attr = attr.lend_mut();
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
    set_attr!(world, node_id, TextStyle, letter_spacing, value);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_rgba_color(world: u32, node_id: u32, r: f32, g: f32, b: f32, a: f32){
    let color = 0;
    set_attr!(world, node_id, TextStyle, color, Color::RGBA(CgColor::new(r, g, b, a)));
}

// __jsObj: color_and_positions: [r, g, b, a, pos,   r, g, b, a, pos], direction: 0-360度
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_linear_gradient_color(world: u32, node_id: u32, direction: f32){
    let color_and_positions: TypedArray<f32> = js!(return __jsObj;).try_into().unwrap();
    let value = Color::LinearGradient(to_linear_gradient_color(color_and_positions.to_vec(), direction));
    let color = 0;
    set_attr!(world, node_id, TextStyle, color, value);
}


#[allow(unused_attributes)]
#[no_mangle]
pub fn set_line_height_normal(world: u32, node_id: u32){
    let line_height = 0;
    set_attr!(world, node_id, TextStyle, line_height, LineHeight::Normal);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_line_height(world: u32, node_id: u32, value: f32){
    let line_height = 0;
    set_attr!(world, node_id, TextStyle, line_height, LineHeight::Length(value));
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_line_height_percent(world: u32, node_id: u32, value: f32){
    let line_height = 0;
    set_attr!(world, node_id, TextStyle, line_height, LineHeight::Percent(value));
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_indent(world: u32, node_id: u32, value: f32){
    let indent = 0;
    set_attr!(world, node_id, TextStyle, indent, value);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_stroke(world: u32, node_id: u32, width: f32, r: f32, g: f32, b: f32, a: f32){
    let stroke = 0;
    set_attr!(world, node_id, TextStyle, stroke, Stroke {
        width,
        color: CgColor::new(r, g, b, a),
    });
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_white_space(world: u32, node_id: u32, value: u8){
    let white_space = 0;
    set_attr!(world, node_id, TextStyle, white_space, unsafe {transmute(value)});
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_shadow_h(world: u32, node_id: u32, value: f32){
    let h = 0;
    set_attr!(world, node_id, TextShadow, h, value);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_shadow_v(world: u32, node_id: u32, value: f32){
    let v = 0;
    set_attr!(world, node_id, TextShadow, v, value);
 
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_shadow_blur(world: u32, node_id: u32, value: f32){
    let blur = 0;
    set_attr!(world, node_id, TextShadow, blur, value);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_shadow_color(world: u32, node_id: u32, r: f32, g: f32, b: f32, a: f32){
    let color = 0;
    set_attr!(world, node_id, TextShadow, color, CgColor::new(r, g, b, a));
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
    let world = unsafe {&mut *(world as usize as *mut World)};
    let attr = world.fetch_multi::<Node, TextShadow>().unwrap();
    attr.lend_mut().insert(node_id, value);
    debug_println!("set_text_shadow"); 
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_style(world: u32, node_id: u32, value: u8){
    let style = 0;
    set_attr!(world, node_id, Font, style, unsafe {transmute(value)});
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_weight(world: u32, node_id: u32, value: f32){
    let weight = 0;
    set_attr!(world, node_id, Font, weight, value);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_size_none(world: u32, node_id: u32){
    let size = 0;
    set_attr!(world, node_id, Font, size, FontSize::None);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_size(world: u32, node_id: u32, value: f32){
    let size = 0;
    set_attr!(world, node_id, Font, size, FontSize::Length(value + 2.0));
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_size_percent(world: u32, node_id: u32, value: f32){
    let size = 0;
    set_attr!(world, node_id, Font, size, FontSize::Percent(value));
}

// __jsObj: family name
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_font_family(world: u32, node_id: u32){
    let value: String = js!(return __jsObj;).try_into().unwrap();

    // 生成不存在的字体
    let world1 = unsafe {&mut *(world as usize as *mut World)};
    let node = node_id as usize;
    let texts = world1.fetch_multi::<Node, Text>().unwrap();
    let texts = texts.lend_mut();
    match texts.get(node) {
        Some(t) => look_text(world, node, t.0.as_str()),
        None => (),
    };
    
    let family = 0;
    set_attr!(world, node_id, Font, family, Atom::from(value));
    debug_println!("set_font_family"); 
}

//           配置       图片                     图片名称
//__jsObj: uv cfg, __jsObj1: image | canvas, __jsObj2: name(String)
#[allow(unused_attributes)]
#[no_mangle]
pub fn add_sdf_font_res(world: u32) {
    let world = unsafe {&mut *(world as usize as *mut World)};
    let name: String = js!(return __jsObj2;).try_into().unwrap();
    let name = Atom::from(name);
    let cfg: TypedArray<u8> = js!(return __jsObj;).try_into().unwrap();
    let cfg = cfg.to_vec();
    let width: u32 = js!(return __jsObj1.width;).try_into().unwrap();
    let height: u32 = js!(return __jsObj1.height;).try_into().unwrap();
    let engine = world.fetch_single::<Engine<WebGLContextImpl>>().unwrap();
    let engine = engine.lend_mut();
    let font_sheet = world.fetch_single::<FontSheet<WebGLContextImpl>>().unwrap();
    let font_sheet = font_sheet.lend_mut();

    let texture = match TryInto::<Object>::try_into(js!{return __jsObj1}) {
        Ok(image_obj) => engine.gl.create_texture_2d_webgl(width, height, 0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &image_obj).unwrap(),
        Err(_) => panic!("set_src error"),
    };
    // let texture = match TryInto::<ImageElement>::try_into(js!{return __jsObj1}) {
    //     Ok(r) => engine.gl.create_texture_2d_webgl(0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &WebGLTextureData::Image(r)).unwrap(),
    //     Err(_s) => match TryInto::<CanvasElement>::try_into(js!{return __jsObj1}){
    //     Ok(r) => engine.gl.create_texture_2d_webgl(0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &WebGLTextureData::Canvas(r)).unwrap(),
    //     Err(s) => panic!("set_src error, {:?}", s),
    //     },
    // };
    let texture_res = TextureRes::<WebGLContextImpl>::new(name.clone(), width as usize, height as usize, unsafe{transmute(Opacity::Translucent)}, unsafe{transmute(0 as u8)}, texture);
    let texture_res = engine.res_mgr.textures.create(texture_res);
    // new_width_data
    let mut sdf_font = DefaultSdfFont::<WebGLContextImpl>::new(texture_res.clone());
    debug_println!("sdf_font parse start");
    sdf_font.parse(cfg.as_slice()).unwrap();
    debug_println!("sdf_font parse end: name: {:?}, {:?}", &sdf_font.name, &sdf_font.glyph_table);

    font_sheet.set_src(sdf_font.name(), Arc::new(sdf_font));
}

//          字体族名称                        字体名称（逗号分隔）     
// __jsObj: family_name(String), __jsObj1: src_name(String, 逗号分隔), 
#[allow(unused_attributes)]
#[no_mangle]
pub fn add_font_face(world: u32, oblique: f32, size: f32, weight: f32){
    let world = unsafe {&mut *(world as usize as *mut World)};
    let family: String = js!(return __jsObj;).try_into().unwrap();
    let src: String = js!(return __jsObj1;).try_into().unwrap();
    let font_sheet = world.fetch_single::<FontSheet<WebGLContextImpl>>().unwrap();
    let font_sheet = font_sheet.lend_mut();
    
    font_sheet.set_face(Atom::from(family), oblique, size + 2.0, weight, src);
}

// __jsObj 文字字符串
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_text_content(world_id: u32, node: u32){
    let value: String = js!(return __jsObj;).try_into().unwrap();
    let node = node as usize;
    let world = unsafe {&mut *(world_id as usize as *mut World)};

    // 生成不存在的字体
    look_text(world_id, node, value.as_str());

    let text = world.fetch_multi::<Node, Text>().unwrap();
    text.lend_mut().insert(node as usize, Text(Arc::new(value)));
    debug_println!("set_text_content");  
}

// 动态字体纹理回调
// __jsObj: canvas __jsObj1: string, (字体名称)__jsObj2: u32Array, (draw 字符)
#[allow(unused_attributes)]
#[no_mangle]
pub fn update_font_texture(world: u32){
    let world = unsafe {&mut *(world as usize as *mut World)};
    let font_name: String = js!(return __jsObj1;).try_into().unwrap();
    let font_name = Atom::from(font_name);
    let font_sheet = world.fetch_single::<FontSheet<WebGLContextImpl>>().unwrap();
    let font_sheet = font_sheet.lend_mut();
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
    println!("u1: {}, v1: {}, width: {}, height: {}", u1, v1, width, height);
    // 优化， getImageData性能不好， 应该直接更新canvas， TODO
    match TryInto::<TypedArray<u8>>::try_into(js!{return new Uint8Array(__jsObj.getContext("2d").getImageData(@{u1 as u32}, @{v1 as u32}, @{width}, @{height}).data.buffer);} ) {
        Ok(data) => {
            let data = data.to_vec();
            src.texture().bind.update(u1 as u32, v1 as u32, width, height, &TextureData::U8(data.as_slice()));
            let render_objs = world.fetch_single::<RenderObjs<WebGLContextImpl>>().unwrap();
            let render_objs = render_objs.lend_mut();
            render_objs.get_notify().modify_event(1, "", 0);
        },
        Err(_) => panic!("update_font_texture error"),
    };
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
            
            println!("buffer-------------------------------");
            glyphs
        },
        Err(_) => panic!("gen font error"),
    }
}

fn look_text(world_id: u32, node: usize, text: &str){
    let world = unsafe {&mut *(world_id as usize as *mut World)};
    let fonts = world.fetch_multi::<Node, Font>().unwrap();
    let fonts = fonts.lend_mut();
    let default_table = world.fetch_single::<DefaultTable>().unwrap();
    let default_table = default_table.lend_mut();

    let font = match fonts.get(node) {
        Some(r) => r,
        None => default_table.get_unchecked::<Font>(),
    };

    let font_sheet = world.fetch_single::<FontSheet<WebGLContextImpl>>().unwrap();
    let font_sheet = font_sheet.lend_mut();

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
                    None => i += 1,
                }
            }
            if chars.len() == 0 {
                return;
            }
            let chars: Vec<u32> = unsafe {transmute(chars)};
            let gl = gen_font(world_id, r.name().as_ref(), chars.as_slice());

            for v in gl.into_iter() {
                r.add_glyph(v.id, v);
            }
        },
        None => ()
    }
}