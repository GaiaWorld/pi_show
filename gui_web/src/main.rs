#![feature(nll)] 
#![feature(proc_macro_hygiene)]
#![feature(rustc_const_unstable)] 
#![feature(core_intrinsics)]
#![feature(custom_attribute)] 
#![feature(type_ascription)]
#![feature(link_args)]
#![feature(vec_remove_item)]
#![allow(unused_attributes)]
#![allow(dead_code)]
#![feature(rustc_private)]
#![recursion_limit="512"]

extern crate serde;
extern crate stdweb_derive;
extern crate webgl_rendering_context;
#[macro_use]
extern crate stdweb;
extern crate lazy_static;
extern crate paste;
extern crate gui;
extern crate ecs;
#[macro_use]
extern crate ecs_derive;
extern crate map;
#[macro_use]
extern crate debug_info;
extern crate hal_core;
extern crate hal_webgl;
extern crate atom;
extern crate octree;
extern crate cg2d;
extern crate share;
extern crate fx_hashmap;
#[macro_use]
extern crate serde_derive;
extern crate ordered_float;

use std::mem::transmute;

use stdweb::unstable::TryInto;
use stdweb::Object;
use webgl_rendering_context::{WebGLRenderingContext};
use ordered_float::OrderedFloat;

use share::Share;
use atom::Atom;
use hal_webgl::*;
use hal_core::*;
use ecs::{ LendMut, Lend};
use gui::layout::{ YGAlign, FlexNode };
use gui::world::{ create_world, RENDER_DISPATCH, LAYOUT_DISPATCH };
use gui::component::user::*;
use gui::component::calc::Visibility;
use gui::single::RenderBegin;
use gui::render::engine::Engine;
use gui::world::GuiWorld as GuiWorld1;
use gui::render::res::TextureRes;


pub mod class;
pub mod style;
pub mod node;
pub mod text;
pub mod layout;
pub mod transform;
pub mod debug;
pub mod yoga;
pub mod bc;
pub mod world;

use bc::YgNode;
use text::{ DrawTextSys, define_draw_canvas};

pub struct GuiWorld {
    pub gui: GuiWorld1<WebglHalContext, YgNode>,
    pub draw_text_sys: DrawTextSys,
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn create_engine(mut res_cush_time: u32) -> u32{
    debug_println!("create_engine");
    let gl: WebGLRenderingContext = js!(return __gl;).try_into().unwrap();
    let fbo = TryInto::<Option<Object>>::try_into(js!(return __fbo?{wrap: __fbo}: undefined;)).unwrap();
    let gl = WebglHalContext::new(gl, fbo, true);

    if res_cush_time < 500 {
        res_cush_time = 500;
    }
    // let gl = WebglHalContext::new(Arc::new(gl), None);
    let engine = Engine::new(gl, res_cush_time);
    Box::into_raw(Box::new(engine)) as u32
}
#[allow(unused_attributes)]
#[no_mangle]
pub fn create_gui(engine: u32, width: f32, height: f32) -> u32{
    debug_println!("create_gui");
    let engine = *unsafe { Box::from_raw(engine as usize as *mut Engine<WebglHalContext>)}; // 安全隐患， 会消耗Engine的所有权， 一旦gui销毁，Engine也会销毁， 因此Engine无法共享， engine应该改为Rc
    let world = create_world::<_, YgNode>(engine, width, height);
    let world =  GuiWorld1::<WebglHalContext, YgNode>::new(world);
    let idtree = world.idtree.lend_mut();
    let node = world.node.lend_mut().create();
    let border_radius = world.border_radius.lend_mut();
    border_radius.insert(node, BorderRadius{x: LengthUnit::Pixel(0.0), y: LengthUnit::Pixel(0.0)});
    world.class_name.lend_mut().insert(node, ClassName(0));

    let visibilitys = world.visibility.lend_mut();
    visibilitys.insert(node, Visibility(true));

    let ygnode = world.yoga.lend_mut();
    let ygnode = unsafe { ygnode.get_unchecked_mut(node) };
    ygnode.set_width(width);
    ygnode.set_height(height);
    ygnode.set_align_items(YGAlign::YGAlignFlexStart);

    idtree.create(node);
    idtree.insert_child(node, 0, 0, None);
    let world = GuiWorld{
        gui: world,
        draw_text_sys: DrawTextSys::new(),
    };
    Box::into_raw(Box::new(world)) as u32
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_clear_color(world: u32, r: f32, g: f32, b: f32, a: f32){
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let render_begin = world.world.fetch_single::<RenderBegin>().unwrap();
    let render_begin = render_begin.lend_mut();
    Share::make_mut(&mut render_begin.0).clear_color = Some((OrderedFloat(r), OrderedFloat(g), OrderedFloat(b), OrderedFloat(a))); 
}

// 渲染gui
#[allow(unused_attributes)]
#[no_mangle]
pub fn render(world_id: u32){
    // debug_println!("gui render");
    let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};
    // let time = std::time::Instant::now();
    world.draw_text_sys.run(world_id);
    // println!("cal text canvas---------------{:?}",  std::time::Instant::now() - time);
	let world = &mut world.gui;
    load_image(world_id);
    world.world.run(&RENDER_DISPATCH);
}

// 计算布局
#[allow(unused_attributes)]
#[no_mangle]
pub fn cal_layout(world_id: u32){
    debug_println!("cal_layout");
    let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};
    // let time = std::time::Instant::now();
    world.draw_text_sys.run(world_id);
    // println!("cal text canvas1---------------{:?}",  std::time::Instant::now() - time);
	let world = &mut world.gui;
    world.world.run(&LAYOUT_DISPATCH);
}

//设置shader
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_shader(engine: u32){
    debug_println!("set_shader");
    let shader_name: String = js!(return __jsObj;).try_into().unwrap();
    let shader_code: String = js!(return __jsObj1;).try_into().unwrap();
    let engine = unsafe { &mut *(engine as usize as *mut Engine<WebglHalContext>)};
    engine.gl.render_set_shader_code(&shader_name, &shader_code);
}

#[no_mangle]
pub fn has_texture_res(world: u32, key: String) -> bool{
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let engine = world.engine.lend();
    let key = Atom::from(key);
    match engine.res_mgr.get::<TextureRes>(&key) {
        Some(_res) => true,
        None => false,
    }
}

#[no_mangle]
pub fn get_texture_res(world: u32, key: String) -> u32{
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let engine = world.engine.lend();
    let key = Atom::from(key);
    match engine.res_mgr.get::<TextureRes>(&key) {
        Some(res) => Box::into_raw(Box::new(res)) as u32,
        None => 0,
    }
}

#[no_mangle]
pub fn release_texture_res(texture: u32){
    unsafe { Box::from_raw(texture as usize as *mut Share<TextureRes>) };
}

//__jsObj: image,  __jsObj1: key
#[no_mangle]
pub fn create_texture_res(world: u32, opacity: u8, compress: u8) -> u32{
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let engine = world.engine.lend_mut();
    let name: String = js!{return __jsObj1}.try_into().unwrap();
    let name = Atom::from(name);

    let width: u32 = js!{return __jsObj.width}.try_into().unwrap();
    let height: u32 = js!{return __jsObj.height}.try_into().unwrap();

    let texture = match TryInto::<Object>::try_into(js!{return {wrap: __jsObj};}) {
        Ok(image_obj) => engine.gl.texture_create_2d_webgl(width, height, 0, PixelFormat::RGBA, DataFormat::UnsignedByte, false, &image_obj).unwrap(),
        Err(_) => panic!("set_src error"),
    };

    let res = engine.res_mgr.create::<TextureRes>(name, TextureRes::new(width as usize, height as usize, unsafe{transmute(opacity)}, unsafe{transmute(compress)}, Share::new(texture)) );
    Box::into_raw(Box::new(res)) as u32
}

// #[no_mangle]
// pub fn add_sdf_font_res(world: u32, value: u32){
//     let world = unsafe {&mut *(world as usize as *mut GuiWorld<WorldDocMgr, ()>)};
//     let res = *unsafe { Box::from_raw(value as usize as *mut Arc<SdfFont>) };
//     world.component_mgr.font.set_src(res.name(), res);
// }


#[no_mangle]
pub fn notify_timeout(f1: u32, f2: u32){
    let f: Box<dyn FnOnce()> = unsafe { transmute((f1 as usize, f2 as usize)) };
    f();
}

// __jsObj: image, __jsObj1: image_name(String)
#[no_mangle]
pub fn load_image_success(world_id: u32, opacity: u8, compress: u8){
    let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};
	let world = &mut world.gui;

    let name: String = js!{return __jsObj1}.try_into().unwrap();
    let name = Atom::from(name);
    let engine = world.engine.lend_mut();
    let width: u32 = js!{return __jsObj.width}.try_into().unwrap();
    let height: u32 = js!{return __jsObj.height}.try_into().unwrap();

    let texture = match TryInto::<Object>::try_into(js!{return {wrap: __jsObj};}) {
        Ok(image_obj) => engine.gl.texture_create_2d_webgl(width, height, 0, PixelFormat::RGBA, DataFormat::UnsignedByte, false, &image_obj).unwrap(),
        Err(s) => panic!("set_src error, {:?}", s),
    };
    let res = engine.res_mgr.create::<TextureRes>(name.clone(), TextureRes::new(width as usize, height as usize, unsafe{transmute(opacity)}, unsafe{transmute(compress)}, Share::new(texture)) );
    
    let image_wait_sheet = world.image_wait_sheet.lend_mut();
    match image_wait_sheet.wait.remove(&name) {
        Some(r) => {
            image_wait_sheet.finish.push((name, res, r));
        },
        None => (),
    };
    image_wait_sheet.get_notify().modify_event(0, "", 0);
}

fn load_image(world_id: u32) {
    let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};

    let image_wait_sheet = &mut world.gui.image_wait_sheet.lend_mut();
    for img_name in image_wait_sheet.loads.iter() {
        js!{
            if window.__load_image {
                window.__load_image(world_id, @{img_name.as_ref()});
            } else {
                console.log("__load_image is undefined");
            }
        }
    }
    image_wait_sheet.loads.clear();
}

// pub fn cancel_timeout(id: usize){
//     js!{
//         clearTimeout(@{id as u32});
//     }
// }

// pub fn set_timeout(ms: usize, f: Box<dyn FnOnce()>) -> usize{
//     let (x, y): (usize, usize) = unsafe { transmute(f) };
//     js!{
//         return setTimeout(function(){
//             Module._notify_timeout(@{x as u32}, @{y as u32});
//         }, @{ms as u32});
//     }
//     0
// }

// pub fn now_time() -> u64{
//     TryInto::<u64>::try_into(js!{
//         return Date.now();
//     }).unwrap()
// }

fn main(){
    define_draw_canvas();
}
