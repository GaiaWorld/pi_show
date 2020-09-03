use wasm_bindgen::prelude::*;
use js_sys::Object;

use web_sys::{WebGlRenderingContext as WebGlRenderingContext1, WebGlVertexArrayObject};

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(typescript_type = "OESVertexArrayObject")]
	pub type OESVertexArrayObject;
	#[wasm_bindgen(method)]
	pub fn createVertexArrayOES(this: &OESVertexArrayObject) -> WebGlVertexArrayObject;
	#[wasm_bindgen(method)]
	pub fn deleteVertexArrayOES(this: &OESVertexArrayObject, vao: &WebGlVertexArrayObject);
	#[wasm_bindgen(method)]
	pub fn bindVertexArrayOES(this: &OESVertexArrayObject, vao: Option<&WebGlVertexArrayObject>);
}

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(extends=WebGlRenderingContext1, js_name = WebGlRenderingContext)]
    pub type WebGlRenderingContext;
    #[wasm_bindgen(catch,method,structural,js_class = "WebGLRenderingContext",js_name = texImage2D )]
    pub fn tex_image_2d_with_obj(
        this: &WebGlRenderingContext,
        target: u32,
        level: i32,
        internalformat: i32,
        format: u32,
        type_: u32,
        pixels: &Object,
	) -> Result<(), JsValue>;

	#[wasm_bindgen(catch,method,structural,js_class = "WebGLRenderingContext",js_name = texSubImage2D )]
	pub fn tex_sub_image_2d_with_u32_and_u32_and_obj(
		this: &WebGlRenderingContext,
		target: u32,
		level: i32,
		xoffset: i32,
		yoffset: i32,
		format: u32,
		type_: u32,
		image: &Object
	) -> Result<(), JsValue>;

	#[wasm_bindgen(catch,method,structural,js_class = "WebGLRenderingContext",js_name = texImage2D )]
    pub fn tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_f32_array(
        this: &WebGlRenderingContext,
        target: u32,
        level: i32,
        internalformat: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        type_: u32,
        pixels: Option<&[f32]>,
	) -> Result<(), JsValue>;

	#[wasm_bindgen(catch,method,structural,js_class = "WebGLRenderingContext",js_name = compressedTexImage2D )]
    pub fn compressed_tex_image_2d_with_f32_array(
        this: &WebGlRenderingContext,
        target: u32,
		level: i32,
		internalformat: u32,
		width: i32,
		height: i32,
		border: i32,
		data: &[f32]
	) -> Result<(), JsValue>;

	#[wasm_bindgen(catch,method,structural,js_class = "WebGLRenderingContext",js_name = compressedTexSubImage2D )]
    pub fn tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_f32_array(
        this: &WebGlRenderingContext,
        target: u32,
		level: i32,
		xoffset: i32,
		yoffset: i32,
		width: i32,
		height: i32,
		format: u32,
		type_: u32,
		pixels: Option<&[f32]>
	) -> Result<(), JsValue>;
}