use std::mem::{transmute, transmute_copy};
use std::sync::Arc;

use stdweb::unstable::TryInto;
use stdweb::web::TypedArray;
use stdweb::Object;

use atom::Atom;
use ecs::{World, LendMut};
use hal_core::*;
use hal_webgl::*;

use component::user::*;
use entity::Node;
use render::res::{ TextureRes, Opacity };
use render::engine::Engine;
use font::sdf_font::{DefaultSdfFont, MSdfGenerator, Glyph, SdfFont};
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
    set_attr!(world, node_id, Font, size, FontSize::Length(value));
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
    let mut sdf_font = DefaultSdfFont::<WebGLContextImpl, FontGenerator>::new(texture_res.clone(), FontGenerator{font_name: name.clone()});
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
    
    font_sheet.set_face(Atom::from(family), oblique, size, weight, src);
}

// 动态字体纹理回调
// __jsObj: canvas __jsObj1: string, (字体名称)
#[allow(unused_attributes)]
#[no_mangle]
pub fn update_font_texture(world: u32, u: u32, v: u32, width: u32, height: u32){
    let world = unsafe {&mut *(world as usize as *mut World)};
    let font_name: String = js!(return __jsObj1;).try_into().unwrap();
    let font_name = Atom::from(font_name);
    let font_sheet = world.fetch_single::<FontSheet<WebGLContextImpl>>().unwrap();
    let font_sheet = font_sheet.lend_mut();
    let src = match font_sheet.get_src(&font_name) {
        Some(r) => r,
        None => panic!("update_font_texture error, font is not exist: {}", font_name.as_ref()),
    };

    match TryInto::<TypedArray<u8>>::try_into(js!{return __jsObj.getImageData(u, v, width, height)}) {
        Ok(data) => {
            let data = data.to_vec();
            src.texture().bind.update(u, v, width, height, &TextureData::U8(data.as_slice()));
        },
        Err(_) => panic!("update_font_texture error"),
    };
}

pub struct FontGenerator{
    font_name: Atom,
}

impl MSdfGenerator for FontGenerator{
    fn gen(&self, s: char) -> Glyph {
        let c: u32 = unsafe{transmute_copy(&s)};
        match TryInto::<TypedArray<u8>>::try_into(js!{return __gen_font(@{self.font_name.as_ref()}, @{c})}){
            Ok(buffer) => {
                let buffer = buffer.to_vec();
                let glyph = Glyph::parse(buffer.as_slice(), &mut 0);
                glyph
            },
            Err(_) => Glyph {
                id: s,
                x: 0.0,
                y: 0.0,
                ox: 0.0, 
                oy: 0.0,
                width: 0.0, 
                height: 0.0,
                advance: 0.0,
            }
        }
    }

    fn gen_mult(&self, _chars: &[char]) -> Vec<Glyph> {
        unimplemented!{}
    }
}