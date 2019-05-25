use std::sync::{Arc};

#[macro_use]
extern crate stdweb;

extern crate webgl_rendering_context;
extern crate atom;
extern crate hal_core;
extern crate hal_webgl;

mod test_shader;

use std::mem::forget;
use std::collections::{HashMap};

use stdweb::unstable::TryInto;
use stdweb::web::{
    IParentNode,
    document,
};
use stdweb::web::html_element::CanvasElement;

use webgl_rendering_context::{
    WebGLRenderingContext,
};

use atom::Atom;
use hal_core::*;
use hal_webgl::*;
use test_shader::{hello_vertex_shader, hello_fragment_shader};

struct RenderData {
    context: WebGLContextImpl,
    default_rt: Arc<WebGLRenderTargetImpl>,
    begin_data: Arc<RenderBeginDesc>,

    pipeline: Arc<Pipeline>,
    geometry: Arc<WebGLGeometryImpl>,
    uniforms1: HashMap<Atom, Arc<AsRef<Uniforms<WebGLContextImpl>>>>,
    uniforms2: HashMap<Atom, Arc<AsRef<Uniforms<WebGLContextImpl>>>>,
}

fn main() {
    stdweb::initialize();

    let canvas: CanvasElement = document().query_selector("#canvas").unwrap().unwrap().try_into().unwrap();

    // 注：gl只能有rust层创建，传到js去，由js创建3D场景；
    // 注：js没有办法通过asm.js传js object 到rust层，包括浏览器原生对象
    let gl: WebGLRenderingContext = canvas.get_context().unwrap();
    
    js! {
        js_init_gl(@{&gl});
    }

    let gl = Arc::new(gl);
    let mut webgl = create_hal_webgl(gl);

    // 初始化 shader
    let default_rt = webgl.get_default_render_target();
    let pipeline = init_pipeline(&mut webgl);
    let geometry = init_rect_geometry(&mut webgl);
    let (u1, u2) = init_uniforms(&mut webgl);
    let begin_data = init_begin_data();

    let data = Box::new(RenderData {
        context: webgl,
        default_rt: default_rt,
        begin_data: begin_data,
        pipeline: pipeline,
        geometry: geometry,
        uniforms1: u1,
        uniforms2: u2, 
    });
    
    let data = Box::into_raw(data) as u32;

    js! {
        Module._render(@{data});
    }

    stdweb::event_loop();
}

#[no_mangle]
fn render(data: *mut RenderData) {

    js! {
        requestAnimationFrame(function () {
            Module._render(@{data as u32});
        })
    }

    let mut data = unsafe { Box::from_raw(data) };

    let default_rt = data.default_rt.clone() as Arc<AsRef<WebGLRenderTargetImpl>>;
    let begin_desc = data.begin_data.clone() as Arc<AsRef<RenderBeginDesc>>;
    data.context.begin_render(&default_rt, &begin_desc);
    
    let pipeline = data.pipeline.clone() as Arc<AsRef<Pipeline>>;
    data.context.set_pipeline(&pipeline);

    let geometry = data.geometry.clone() as Arc<AsRef<WebGLGeometryImpl>>;
    data.context.draw(&geometry, &data.uniforms1);
    data.context.draw(&geometry, &data.uniforms2);

    data.context.end_render();

    forget(data);
}

fn init_begin_data() -> Arc<RenderBeginDesc> {
    let mut data = RenderBeginDesc::new(0, 0, 1024, 1024);
    data.set_clear_color(true, 0.0, 0.0, 1.0, 1.0);
    Arc::new(data)
}

fn init_pipeline(context: &mut WebGLContextImpl) -> Arc<Pipeline> {
    let (vs_hash, fs_hash) = init_hello_program(context);
    let rs = Arc::new(RasterState::new());
    let bs = Arc::new(BlendState::new()); 
    let ss = Arc::new(StencilState::new());
    let ds = Arc::new(DepthState::new());
    let pipeline = context.create_pipeline(vs_hash, fs_hash, rs, bs, ss, ds).unwrap();

    println!("init_pipeline");

    Arc::new(pipeline)
}

fn init_rect_geometry(context: &mut WebGLContextImpl) -> Arc<WebGLGeometryImpl> {
    let mut geometry = context.create_geometry().unwrap();

    geometry.set_vertex_count(4);

    let position: [f32; 12] = [
        -0.5, -0.5, 0.0, 
         0.5, -0.5, 0.0, 
         0.5,  0.5, 0.0, 
        -0.5,  0.5, 0.0, 
    ];

    let indices: [u16; 6] = [
        0, 2, 1,
        0, 3, 2,
    ];

    let _ = geometry.set_attribute(&AttributeName::Position, 3, Some(&position), false);
    let _ = geometry.set_indices_short(&indices, false);

    println!("init_geometry");
    
    Arc::new(geometry)
}

fn init_uniforms(context: &mut WebGLContextImpl) -> (HashMap<Atom, Arc<AsRef<Uniforms<WebGLContextImpl>>>>, HashMap<Atom, Arc<AsRef<Uniforms<WebGLContextImpl>>>>)  {
    
    let mut map1 = HashMap::new();
    let mut map2 = HashMap::new();

    let mut color = context.create_uniforms();
    color.set_float_4(&Atom::from("uColor"), 1.0, 1.0, 0.0, 1.0);
    let color = Arc::new(color);

    let mut position1 = context.create_uniforms();
    position1.set_float_3(&Atom::from("uPosition"), 0.0, 0.0, 0.0);
    let position1 = Arc::new(position1);

    let mut position2 = context.create_uniforms();
    position2.set_float_3(&Atom::from("uPosition"), 0.05, -0.05, 0.0);
    let position2 = Arc::new(position2);

    map1.insert(Atom::from("position"), position1 as Arc<AsRef<Uniforms<WebGLContextImpl>>>);
    map1.insert(Atom::from("color"), color.clone() as Arc<AsRef<Uniforms<WebGLContextImpl>>>);
    
    map2.insert(Atom::from("position"), position2 as Arc<AsRef<Uniforms<WebGLContextImpl>>>);
    map2.insert(Atom::from("color"), color as Arc<AsRef<Uniforms<WebGLContextImpl>>>);

    println!("init_uniforms");
    (map1, map2)
}

fn init_hello_program(context: &mut WebGLContextImpl) -> (u64, u64) {
    let vs_name = Atom::from("hello_vs");
    let fs_name = Atom::from("hello_fs");
    context.set_shader_code(&vs_name, &hello_vertex_shader());
    context.set_shader_code(&fs_name, &hello_fragment_shader());

    let vs = context.compile_shader(ShaderType::Vertex, &vs_name, &[]).unwrap();
    let fs = context.compile_shader(ShaderType::Fragment, &fs_name, &[]).unwrap();

    println!("init_hello_program");
    (vs, fs)
}