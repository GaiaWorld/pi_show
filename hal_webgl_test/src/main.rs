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

struct RenderMesh {
    pipeline: Arc<Pipeline>,
    geometry: Arc<WebGLGeometryImpl>,
    uniforms: HashMap<Atom, Arc<AsRef<Uniforms<WebGLContextImpl>>>>,
}

struct RenderData {
    context: WebGLContextImpl,
    default_rt: Arc<WebGLRenderTargetImpl>,
    begin_data: Arc<RenderBeginDesc>,
    meshes: Vec<RenderMesh>,
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
    let webgl = create_hal_webgl(gl);

    let render_data = init_render_data(webgl);
    let data = Box::into_raw(render_data) as u32;

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
    
    for m in data.meshes.iter_mut() {
        let pipeline = m.pipeline.clone() as Arc<AsRef<Pipeline>>;
        data.context.set_pipeline(&pipeline);

        let geometry = m.geometry.clone() as Arc<AsRef<WebGLGeometryImpl>>;
        data.context.draw(&geometry, &m.uniforms);
    }
    
    data.context.end_render();

    forget(data);
}

fn init_render_data(mut webgl: WebGLContextImpl) -> Box<RenderData> {

    // 初始化 shader
    let (vs, fs1, fs2) = init_hello_program(&mut webgl);
    
    // 初始化 pipeline
    let rs = Arc::new(RasterState::new());
    
    let mut bs1 = BlendState::new(); 
    bs1.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
    let bs1 = Arc::new(bs1);
    
    let ss = Arc::new(StencilState::new());
    let ds1 = Arc::new(DepthState::new());
    
    let mut bs2 = BlendState::new();
    bs2.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
    let bs2 = Arc::new(bs2);

    let mut ds2 = DepthState::new();
    ds2.set_write_enable(false);
    let ds2 = Arc::new(ds2);

    let pipeline1 = webgl.create_pipeline(vs, fs1, rs.clone(), bs1, ss.clone(), ds1).unwrap();
    let pipeline2 = webgl.create_pipeline(vs, fs2, rs, bs2, ss, ds2).unwrap();

    // 初始化 geometry 
    let g1 = init_rect_geometry(&mut webgl, &[0.0, 0.0, 0.3, 0.5], 0.5);
    let g2 = init_rect_geometry(&mut webgl, &[0.2, -0.2, 0.4, 0.7], 0.3);

    // 初始化Uniforms

    let mut color = webgl.create_uniforms(false);
    color.set_float_4(&Atom::from("uColor"), 1.0, 1.0, 0.0, 1.0);
    let color = Arc::new(color);

    let u1 = init_uniforms(&mut webgl, &color, 1.0);
    let u2 = init_uniforms(&mut webgl, &color, 0.5);

    // 初始化网格
    let m1 = RenderMesh {
        pipeline: Arc::new(pipeline1),
        geometry: g1,
        uniforms: u1,
    };

    let m2 = RenderMesh {
        pipeline: Arc::new(pipeline2),
        geometry: g2,
        uniforms: u2,
    };

    let default_rt = webgl.get_default_render_target();
    let begin_data = init_begin_data();
    
    let data = Box::new(RenderData {
        context: webgl,
        default_rt: default_rt,
        begin_data: begin_data,
        meshes: vec![m1, m2],
    });
    
    data
}

fn init_begin_data() -> Arc<RenderBeginDesc> {
    let mut data = RenderBeginDesc::new(0, 0, 1024, 1024);
    data.set_clear_color(true, 0.0, 0.0, 1.0, 1.0);
    Arc::new(data)
}

fn init_rect_geometry(context: &mut WebGLContextImpl, rect: &[f32], z: f32) -> Arc<WebGLGeometryImpl> {
    let mut geometry = context.create_geometry().unwrap();

    geometry.set_vertex_count(4);

    let position: [f32; 12] = [
        rect[0] - rect[2] / 2.0, rect[1] - rect[3] / 2.0, z, 
        rect[0] + rect[2] / 2.0, rect[1] - rect[3] / 2.0, z, 
        rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0, z, 
        rect[0] - rect[2] / 2.0, rect[1] + rect[3] / 2.0, z, 
    ];

    let indices: [u16; 6] = [
        0, 1, 2,
        0, 2, 3,
    ];

    let _ = geometry.set_attribute(&AttributeName::Position, 3, Some(&position), false);
    let _ = geometry.set_indices_short(&indices, false);

    println!("init_geometry");
    
    Arc::new(geometry)
}

fn init_uniforms(context: &mut WebGLContextImpl, color: &Arc<Uniforms<WebGLContextImpl>>, alpha: f32) -> HashMap<Atom, Arc<AsRef<Uniforms<WebGLContextImpl>>>>  {
    
    let mut map = HashMap::new();

    let mut each = context.create_uniforms(false);
    each.set_float_3(&Atom::from("uPosition"), 0.0, 0.0, 0.0);
    if alpha != 1.0 {
        each.set_float_1(&Atom::from("uAlpha"), alpha);
    }
    
    let each = Arc::new(each);

    map.insert(Atom::from("each"), each as Arc<AsRef<Uniforms<WebGLContextImpl>>>);
    map.insert(Atom::from("color"), color.clone() as Arc<AsRef<Uniforms<WebGLContextImpl>>>);

    println!("init_uniforms");
    map
}

fn init_hello_program(context: &mut WebGLContextImpl) -> (u64, u64, u64) {
    let vs_name = Atom::from("hello_vs");
    let fs_name = Atom::from("hello_fs");
    context.set_shader_code(&vs_name, &hello_vertex_shader());
    context.set_shader_code(&fs_name, &hello_fragment_shader());

    let alpha = Atom::from("ALPHA");

    let vs = context.compile_shader(ShaderType::Vertex, &vs_name, &[]).unwrap();
    let fs1 = context.compile_shader(ShaderType::Fragment, &fs_name, &[]).unwrap();
    let fs2 = context.compile_shader(ShaderType::Fragment, &fs_name, &[alpha]).unwrap();

    println!("init_hello_program");
    (vs, fs1, fs2)
}