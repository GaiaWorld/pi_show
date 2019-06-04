
use std::mem::{transmute};
use std::sync::Arc;

use stdweb::unstable::TryInto;
use stdweb::web::TypedArray;
use stdweb::web::html_element::{ImageElement, CanvasElement};
use webgl_rendering_context::{WebGLRenderingContext};

use atom::Atom;
use hal_webgl::*;
use hal_core::*;
use ecs::{World, idtree::IdTree, LendMut, Lend};

use component::user::{ BorderRadius, LengthUnit };
use component::calc::Visibility;
use render::engine::Engine;
use render::res::{ TextureRes, Opacity };
use world::{ create_world, RENDER_DISPATCH, LAYOUT_DISPATCH };
use font::sdf_font::{SdfFont, StaticSdfFont};
use font::font_sheet::FontSheet;
use entity::Node;
use layout::YgNode;

pub mod style;
pub mod node;
pub mod text;
pub mod layout;
pub mod transform;

#[allow(unused_attributes)]
#[no_mangle]
pub fn create_engine() -> u32{
    debug_println!("create_engine");
    let gl: WebGLRenderingContext = js!(return __gl;).try_into().unwrap();
    let gl = WebGLContextImpl::new(Arc::new(gl));
    let engine = Engine::new(gl);
    Box::into_raw(Box::new(engine)) as u32
}
#[allow(unused_attributes)]
#[no_mangle]
pub fn create_gui(engine: u32, width: f32, height: f32) -> u32{
    debug_println!("create_gui");
    let engine = *unsafe { Box::from_raw(engine as usize as *mut Engine<WebGLContextImpl>)}; // 安全隐患， 会消耗Engine的所有权， 一旦gui销毁，Engine也会销毁， 因此Engine无法共享， engine应该改为Rc
    let world = create_world(engine, width, height);
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.lend_mut();
    let node = world.create_entity::<Node>();
    let border_radius = world.fetch_multi::<Node, BorderRadius>().unwrap();
    let border_radius = border_radius.lend_mut();
    border_radius.insert(node, BorderRadius{x: LengthUnit::Pixel(0.0), y: LengthUnit::Pixel(0.0)});

    let visibilitys = world.fetch_multi::<Node, Visibility>().unwrap();
    let visibilitys = visibilitys.lend_mut();
    visibilitys.insert(node, Visibility(true));

    let ygnode = world.fetch_multi::<Node, YgNode>().unwrap();
    let ygnode = ygnode.lend_mut();
    let ygnode = unsafe { ygnode.get_unchecked_mut(node) };
    ygnode.set_width(width);
    ygnode.set_height(height);

    idtree.create(node);
    idtree.insert_child(node, 0, 0, None);
    Box::into_raw(Box::new(world)) as u32
}

// 渲染gui
#[allow(unused_attributes)]
#[no_mangle]
pub fn render(world: u32){
    debug_println!("gui render");
    let world = unsafe {&mut *(world as usize as *mut World)};
    world.run(&RENDER_DISPATCH);
}

// 计算布局
#[allow(unused_attributes)]
#[no_mangle]
pub fn cal_layout(world: u32){
    debug_println!("cal_layout");
    let world = unsafe {&mut *(world as usize as *mut World)};
    world.run(&LAYOUT_DISPATCH);
}

//设置shader
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_shader(engine: u32){
    debug_println!("set_shader");
    let shader_name: String = js!(return __jsObj;).try_into().unwrap();
    let shader_code: String = js!(return __jsObj1;).try_into().unwrap();
    let engine = unsafe { &mut *(engine as usize as *mut Engine<WebGLContextImpl>)};
    engine.gl.set_shader_code(&Atom::from(shader_name), &shader_code);
}


#[no_mangle]
pub fn get_texture_res(world: u32, key: String) -> u32{
    let world = unsafe {&mut *(world as usize as *mut World)};
    let engine = world.fetch_single::<Engine<WebGLContextImpl>>().unwrap();
    let engine = engine.lend();
    let key = Atom::from(key);
    match engine.res_mgr.textures.get(&key) {
        Some(res) => Box::into_raw(Box::new(res)) as u32,
        None => 0,
    }
}

// #[no_mangle]
// pub fn create_texture_res(engine: u32, key: String, width: u32, height: u32, opacity: u8, compress: u32) -> u32{
//     let engine = unsafe {&mut *(engine as usize as *mut Engine)};
//     let bind = js!(return __jsObj;).try_into().unwrap();
//     let key = Atom::from(key);
//     let r = Box::into_raw(Box::new( engine.res_mgr.textures.create(TextureRes::new(key, width as usize, height as usize, unsafe{transmute(opacity)}, compress as usize, bind, engine.gl.clone()) ))) as u32;
//     // js!{
//     //     console.log("create_texture_res src:", @{r});
//     // };
//     r
// }

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

    let texture = match TryInto::<ImageElement>::try_into(js!{return __jsObj1}) {
        Ok(r) => engine.gl.create_texture_2d_webgl(0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &WebGLTextureData::Image(r)).unwrap(),
        Err(_s) => match TryInto::<CanvasElement>::try_into(js!{return __jsObj1}){
        Ok(r) => engine.gl.create_texture_2d_webgl(0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &WebGLTextureData::Canvas(r)).unwrap(),
        Err(_s) => panic!("set_src error"),
        },
    };
    let texture_res = TextureRes::<WebGLContextImpl>::new(name.clone(), width as usize, height as usize, unsafe{transmute(Opacity::Translucent)}, unsafe{transmute(0 as u8)}, texture);
    let texture_res = engine.res_mgr.textures.create(texture_res);
    // new_width_data
    let mut sdf_font = StaticSdfFont::<WebGLContextImpl>::new(texture_res.clone());
    debug_println!("sdf_font parse start");
    sdf_font.parse(cfg.as_slice()).unwrap();
    debug_println!("sdf_font parse end: name: {:?}, {:?}", &sdf_font.name, &sdf_font.glyph_table);

    font_sheet.set_src(sdf_font.name(), Arc::new(sdf_font));
}

// #[no_mangle]
// pub fn add_sdf_font_res(world: u32, value: u32){
//     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
//     let res = *unsafe { Box::from_raw(value as usize as *mut Arc<SdfFont>) };
//     world.component_mgr.font.set_src(res.name(), res);
// }

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