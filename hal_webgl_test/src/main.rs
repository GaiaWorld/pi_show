use std::sync::{Arc};

#[macro_use]
extern crate stdweb;

extern crate webgl_rendering_context;
extern crate atom;
extern crate hal_core;
extern crate hal_webgl;

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
use hal_webgl::{create_hal_webgl};

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

    // 初始化 shader
    let pipeline = Arc::into_raw(init_pipeline());
    let geometry = Arc::into_raw(init_geometry());
    let uniforms = Arc::into_raw(init_uniforms());
    
    js! {
        requestAnimationFrame(function () {
            Module._render(pipeline, geometry, uniforms);
        })
    }

    stdweb::event_loop();
}

#[no_mangle]
fn render(pipeline: *const T, geometry: *const T, uniforms: *const T) {
    let pipeline = unsafe { Arc::into_raw(pipeline) };
    let geometry = unsafe { Arc::into_raw(geometry) };
    let uniforms = unsafe { Arc::into_raw(uniforms) };

    println!("render");
}

fn init_pipeline() -> Arc<Pipeline> {

}

fn init_geometry() -> Arc<Geometry> {

}

fn init_uniforms() -> Arc<Uniforms> {

}