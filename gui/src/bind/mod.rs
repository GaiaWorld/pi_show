use std::sync::Arc;
use std::mem::transmute;

use stdweb::unstable::TryInto;
use stdweb::Object;
use webgl_rendering_context::{WebGLRenderingContext};

use atom::Atom;
use hal_webgl::*;
use hal_core::*;
use ecs::{World, idtree::IdTree, LendMut, Lend};

use component::user::{ BorderRadius, LengthUnit };
use component::calc::Visibility;
use single::RenderBegin;
use render::engine::Engine;
use render::res::TextureRes;
use world::{ create_world, RENDER_DISPATCH, LAYOUT_DISPATCH };
use entity::Node;
use layout::{ YgNode, YGAlign };

pub mod style;
pub mod node;
pub mod text;
pub mod layout;
pub mod transform;
pub mod debug;

#[allow(unused_attributes)]
#[no_mangle]
pub fn create_engine() -> u32{
    debug_println!("create_engine");
    let gl: WebGLRenderingContext = js!(return __gl;).try_into().unwrap();
    let fbo = TryInto::<Option<Object>>::try_into(js!(return __fbo?{wrap: __fbo}: undefined;)).unwrap();
    let gl = WebGLContextImpl::new(Arc::new(gl), fbo);

    // let gl = WebGLContextImpl::new(Arc::new(gl), None);
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
    ygnode.set_align_items(YGAlign::YGAlignFlexStart);

    idtree.create(node);
    idtree.insert_child(node, 0, 0, None);
    Box::into_raw(Box::new(world)) as u32
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_clear_color(world: u32, r: f32, g: f32, b: f32, a: f32){
    let world = unsafe {&mut *(world as usize as *mut World)};
    let render_begin = world.fetch_single::<RenderBegin>().unwrap();
    let render_begin = render_begin.lend_mut();
    Arc::make_mut(&mut render_begin.0).clear_color = Some((r, g, b, a)); 
}

// 渲染gui
#[allow(unused_attributes)]
#[no_mangle]
pub fn render(world: u32){
    // debug_println!("gui render");
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
    match engine.res_mgr.get::<TextureRes<WebGLContextImpl>>(&key) {
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

// #[no_mangle]
// pub fn add_sdf_font_res(world: u32, value: u32){
//     let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
//     let res = *unsafe { Box::from_raw(value as usize as *mut Arc<SdfFont>) };
//     world.component_mgr.font.set_src(res.name(), res);
// }

#[no_mangle]
pub fn notify_timeout(f1: u32, f2: u32){
    let f: Box<dyn FnOnce()> = unsafe { transmute((f1 as usize, f2 as usize)) };
    f();
}

pub fn cancel_timeout(id: usize){
    js!{
        clearTimeout(@{id as u32});
    }
}

pub fn set_timeout(ms: usize, f: Box<dyn FnOnce()>) -> usize{
    let (x, y): (usize, usize) = unsafe { transmute(f) };
    js!{
        return setTimeout(function(){
            Module._notify_timeout(@{x as u32}, @{y as u32});
        }, @{ms as u32});
    }
    0
}

pub fn now_time() -> u64{
    TryInto::<u64>::try_into(js!{
        return Date.now();
    }).unwrap()
}
