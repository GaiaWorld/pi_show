
use std::mem::{transmute, uninitialized};
use std::sync::Arc;

use stdweb::unstable::TryInto;
use stdweb::web::TypedArray;
use webgl_rendering_context::{WebGLRenderingContext};

use atom::Atom;
use hal_webgl::WebGLContextImpl;
use hal_core::Context;
use ecs::{World, idtree::IdTree, LendMut};

use component::user::{ BorderRadius, LengthUnit };
use render::engine::Engine;
use render::res::{ TextureRes, Opacity };
use world::{ create_world, RENDER_DISPATCH };
use font::sdf_font::{SdfFont, StaticSdfFont};
use entity::Node;
use layout::YgNode;

pub mod style;
pub mod node;
pub mod text;
pub mod layout;
pub mod transform;

#[no_mangle]
pub fn create_engine() -> u32{
    debug_println!("create_engine");
    let gl: WebGLRenderingContext = js!(return __gl;).try_into().unwrap();
    let gl = WebGLContextImpl::new(Arc::new(gl));
    let engine = Engine::new(gl);
    Box::into_raw(Box::new(engine)) as u32
}

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
#[no_mangle]
pub fn render(world: u32){
    debug_println!("gui render");
    let world = unsafe {&mut *(world as usize as *mut World)};
    world.run(&RENDER_DISPATCH);
}

// 计算布局
#[no_mangle]
pub fn cal_layout(world: u32){
    debug_println!("cal_layout");
    let world = unsafe {&mut *(world as usize as *mut World)};
    world.run(&Atom::from("cal_layout"));
}

//设置shader
#[no_mangle]
pub fn set_shader(engine: u32){
    debug_println!("set_shader");
    let shader_name: String = js!(return __jsObj;).try_into().unwrap();
    let shader_code: String = js!(return __jsObj1;).try_into().unwrap();
    let engine = unsafe { &mut *(engine as usize as *mut Engine<WebGLContextImpl>)};
    engine.gl.set_shader_code(&Atom::from(shader_name), &shader_code);
}
