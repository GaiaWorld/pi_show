use std::cell::RefCell;
use std::rc::{Rc};

use web_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use wcs::world::{System};

use render::vector_sdf::{VectorSdf};
use render::gl::{compile_shader, link_program, GuiWorldViewProjection};
use component::component_def::{GuiComponentMgr};

pub struct Render(RefCell<RenderImpl>);

pub struct RenderImpl{
    gl: WebGlRenderingContext,
    pub view_projection: GuiWorldViewProjection,
    opaque_sdf_render: VectorSdfRender,
}

impl Render {
    pub fn init(component_mgr: &mut GuiComponentMgr, canvas: &HtmlCanvasElement) -> Result<Rc<Render>, JsValue> {
        let gl = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;
        gl.get_extension("OES_element_index_uint")?;
        gl.get_extension("ANGLE_instanced_arrays")?;
        gl.get_extension("OES_standard_derivatives")?;
        gl.get_extension("OES_texture_float")?;
        gl.get_extension("OES_texture_float_linear")?;
        gl.get_extension("OES_texture_half_float")?;
        gl.get_extension("OES_texture_half_float_linear")?;
        gl.get_extension("EXT_sRGB")?;
        gl.get_extension("OES_vertex_array_object")?;
        gl.get_extension("EXT_texture_filter_anisotropic")?;
        gl.get_extension("WEBKIT_EXT_texture_filter_anisotropic")?;
        gl.get_extension("EXT_frag_depth")?;
        gl.get_extension("WEBGL_depth_texture")?;
        gl.get_extension("WEBGL_color_buffer_float")?;
        gl.get_extension("EXT_color_buffer_half_float")?;
        gl.get_extension("EXT_shader_texture_lod")?;
        gl.get_extension("WEBGL_draw_buffers")?;
        gl.get_extension("GL_OES_standard_derivatives")?;
        gl.enable(WebGlRenderingContext::BLEND);
        gl.blend_func(WebGlRenderingContext::SRC_ALPHA, WebGlRenderingContext::ONE_MINUS_SRC_ALPHA);

        gl.clear_color(1.0, 0.0, 0.0, 1.0);
        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        let width = canvas.width() as f32;
        let height = canvas.height() as f32;
        let mut view_projection = GuiWorldViewProjection::new(width, height); // 视窗
        // component_mgr.opaque_vector = VectorSdf::with_size(8);
        let opaque_sdf_render = VectorSdfRender::init(&gl, &mut component_mgr.opaque_vector, width, height, &mut view_projection)?;
        Ok(Rc::new(Render(RefCell::new(RenderImpl{
            gl,
            view_projection,
            opaque_sdf_render
        }))))
    }
}

impl System<(), GuiComponentMgr> for Render{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        let mut render = self.0.borrow_mut();
        let len = match render.opaque_sdf_render.update(&render.gl, &mut component_mgr.opaque_vector) {
            Err(e) => panic!("{:?}", e),
            Ok(l) => l,
        };
        render.opaque_sdf_render.indices_buffer_len = len;
        render.opaque_sdf_render.render(&render.gl)
    }
}


pub struct VectorSdfRender{
    pub tex_buffer: WebGlTexture,
    pub positions_buffer: WebGlBuffer,
    pub mesh_index_buffer: WebGlBuffer,
    pub indices_buffer: WebGlBuffer,
    pub program: WebGlProgram,
    pub indices_buffer_len: u32,
    pub screen_size: (f32, f32),
    pub world_view: GuiWorldViewProjection,
    pub texture_size: (usize, usize),
}

impl VectorSdfRender {
    pub fn init(gl: &WebGlRenderingContext, sdf: &mut VectorSdf, width: f32, height: f32, world_view: &mut GuiWorldViewProjection) -> Result<VectorSdfRender, JsValue>{
        let vert_shader = compile_shader(
            &gl,
            WebGlRenderingContext::VERTEX_SHADER,
            r#"
            precision highp float;

            // Attributes
            attribute float index;
            attribute vec3 position;
            
            // Uniforms
            uniform mat4 worldViewProjection;
            uniform sampler2D paramTexture;
            uniform vec2 textureSize;
            uniform vec2 screenSize;

            // Varying
            varying vec4 param1;
            varying vec4 param2;
            varying vec4 param3;

            vec2 getUV(float offset, float row) {
                return vec2(mod(offset, row) / row, floor(offset / row) / row);
            }

            void main(void) {
                gl_Position = worldViewProjection * vec4(position, 1.0);

                float offset = index * textureSize.y;
                param1 = texture2D(paramTexture, getUV(offset + 0.0, textureSize.x));
                param2 = texture2D(paramTexture, getUV(offset + 1.0, textureSize.x));
                param3 = texture2D(paramTexture, getUV(offset + 2.0, textureSize.x));

                // 注: Center一样需要做相机变换
                vec4 center = vec4(param1.xy, 0.0, 1.0);
                center = worldViewProjection * center;
                center.xy = center.xy / center.w;
                // center.xy 在这里 [-1, 1]
                param1.xy = center.xy * screenSize / 2.0;
            }
        "#,
        )?;
        let frag_shader = compile_shader(
            &gl,
            WebGlRenderingContext::FRAGMENT_SHADER,
            r#"
            #extension GL_OES_standard_derivatives : enable
            precision highp float;

            // Uniforms
            uniform vec2 screenSize;

            // Varying
            varying vec4 param1;
            varying vec4 param2;
            varying vec4 param3;

            float sdfEclipse(vec2 coord, vec2 center, vec2 radius) {
                float a2 = radius.x * radius.x;
                float b2 = radius.y * radius.y;
                float ab = a2 * b2;

                float x2 = coord.x - center.x;
                float y2 = coord.y - center.y;
                x2 = x2 * x2;
                y2 = y2 * y2; 
                
                return (b2 * x2 + a2 * y2 - ab) / ab;
            }

            float sdfRect(vec2 coord, vec2 center, vec2 size, float r) {
                vec2 d = abs(coord - center) - size;
                return min(max(d.x, d.y), 0.0) + length(max(d, 0.0)) - r;
            }

            void main(void) {
                vec2 center = param1.xy;
                vec2 radius = param1.zw;
                vec4 color = param2;

                // gl_FragCoord的范围是[0, screenSize)，需要变成 [-screenSize/2, screenSize/2]
                vec2 coord = gl_FragCoord.xy / gl_FragCoord.w - screenSize / 2.0;
                
                float d;
                if (param3.x == 1.0) { // 矩形
                    d = sdfRect(coord, center, radius, param3.y);
                } else { // 椭圆
                    d = sdfEclipse(coord, center, radius);
                }

                vec3 result = color.rgb;

                // 根据d计算alpha的值
                // d = clamp(d, 0.0, 1.0);
                // d = 1.0 - smoothstep(0.0, 0.1, d);

                // if (d > 0.0) result = vec3(0.0, 0.0, 0.0);
                
                // 抗锯齿：d变化的越快，抗锯齿的程度就越大
                float anti = 1.0 * fwidth(d);

                //float anti = 1.0;
                
                // 三次平滑插值
                d = 1.0 - smoothstep(-anti, anti, d);
                
                // 线性插值
                // float scale = 0.7; // 值越小，越平滑 [0.5, 1.0]
                // d = scale * (d + anti) / anti;
                // d = 1.0 - clamp(d, 0.0, 1.0);

                gl_FragColor = vec4(result, d * color.a);
            }
        "#,
        )?;

        let program = link_program(&gl, [vert_shader, frag_shader].iter())?;
        gl.use_program(Some(&program));

        gl.viewport(0, 0, width as i32, height as i32);

        //设置paramTexture
        let location = gl.get_uniform_location(&program, "paramTexture").unwrap();
        let tex_buffer = gl.create_texture().unwrap();
        gl.active_texture(WebGlRenderingContext::TEXTURE0);
        gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&tex_buffer));
        let b = sdf.use_tex_data().unwrap();
        let buffer = unsafe{js_sys::Float32Array::view(b.1)};
        // console::log_1(&(b.1.len().to_string().into()));
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(WebGlRenderingContext::TEXTURE_2D, 0, WebGlRenderingContext::RGBA as i32, 8, 8, 0, WebGlRenderingContext::RGBA, WebGlRenderingContext::FLOAT, Some(&buffer))?;
        gl.tex_parameteri(WebGlRenderingContext::TEXTURE_2D, WebGlRenderingContext::TEXTURE_MAG_FILTER, WebGlRenderingContext::NEAREST as i32);
        gl.tex_parameteri(WebGlRenderingContext::TEXTURE_2D, WebGlRenderingContext::TEXTURE_MIN_FILTER, WebGlRenderingContext::NEAREST as i32);
        gl.tex_parameteri(WebGlRenderingContext::TEXTURE_2D, WebGlRenderingContext::TEXTURE_WRAP_S, WebGlRenderingContext::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(WebGlRenderingContext::TEXTURE_2D, WebGlRenderingContext::TEXTURE_WRAP_T, WebGlRenderingContext::CLAMP_TO_EDGE as i32);
        gl.uniform1i(Some(&location), 0);

        //设置vertex
        let positions_buffer = gl.create_buffer().ok_or("failed to create buffer")?;
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&positions_buffer));
        let position_local = gl.get_attrib_location(&program, "position");
        gl.vertex_attrib_pointer_with_i32(position_local as u32, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(position_local as u32);

        //设置mesh_index
        let mesh_index_buffer = gl.create_buffer().ok_or("failed to create buffer")?;
        let index_local = gl.get_attrib_location(&program, "index");
        console::log_2(&("index".into()), &(index_local.to_string().into()));
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&mesh_index_buffer));
        gl.vertex_attrib_pointer_with_i32(index_local as u32, 1, WebGlRenderingContext::UNSIGNED_SHORT, false, 0, 0);
        gl.enable_vertex_attrib_array(index_local as u32);

        //设置indices
        let indices_buffer = gl.create_buffer().ok_or("failed to create buffer")?;

        Ok(VectorSdfRender{
            tex_buffer,
            positions_buffer,
            mesh_index_buffer,
            indices_buffer,
            program,
            indices_buffer_len: 0,
            screen_size: (width, height),
            world_view: world_view.clone(),
            texture_size: (sdf.size, sdf.size),
        })
        // // panic!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxc");
        // Err("mmmmmmmm".into())
    }

    pub fn set_uniforms(&self, gl: &WebGlRenderingContext) {
        //设置worldViewProjection
        let location = gl.get_uniform_location(&self.program, "worldViewProjection").unwrap();
        let mut world_view = self.world_view.clone();
        gl.uniform_matrix4fv_with_f32_array(Some(&location), false, &mut world_view);
        let buffer = unsafe{js_sys::Float32Array::view(&world_view)};
        console::log_2(&("worldViewProjection".into()), &buffer);

        //设置screenSize
        let location = gl.get_uniform_location(&self.program, "screenSize").unwrap();
        gl.uniform2f(Some(&location), self.screen_size.0, self.screen_size.1);

        //设置textureSize
        let location = gl.get_uniform_location(&self.program, "textureSize").unwrap();
        gl.uniform2f(Some(&location), self.texture_size.0 as f32, 3.0);
    }

    pub fn update(&self, gl: &WebGlRenderingContext, sdf: &mut VectorSdf) -> Result<u32, JsValue>{
        console::log_1(&("update".into()));
        let mut r = self.indices_buffer_len;
        match sdf.use_tex_data() {
            Some(tex_buffer) => {
                let buffer = unsafe{js_sys::Float32Array::view(tex_buffer.1)};
                console::log_1(&("use_tex_data".into()));
                console::log_1(&buffer);
                let rang = tex_buffer.0;
                gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&self.tex_buffer));
                gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(WebGlRenderingContext::TEXTURE_2D, 0, rang.0 as i32, rang.1 as i32, rang.2 as i32, rang.3 as i32, WebGlRenderingContext::RGBA, WebGlRenderingContext::FLOAT, Some(&buffer))?;
            },
            None => ()
        }

        match sdf.use_positions_data(){
            Some(buffer) => {
                let buffer = unsafe{js_sys::Float32Array::view(buffer)};
                console::log_1(&("use_positions_data".into()));
                console::log_1(&buffer);
                gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.positions_buffer));
                gl.buffer_data_with_array_buffer_view(
                    WebGlRenderingContext::ARRAY_BUFFER,
                    &buffer,
                    WebGlRenderingContext::STATIC_DRAW,
                );
            },
            None => ()
        }

        match sdf.use_indexs_data(){
            Some(buffer) => {
                let mesh_index_buffer = unsafe{js_sys::Uint16Array::view(buffer.0)};
                console::log_1(&("use_mesh_index_data".into()));
                console::log_1(&mesh_index_buffer);
                gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.mesh_index_buffer));
                gl.buffer_data_with_array_buffer_view(
                    WebGlRenderingContext::ARRAY_BUFFER,
                    &mesh_index_buffer,
                    WebGlRenderingContext::STATIC_DRAW,
                );

                let indices_buffer = unsafe{js_sys::Uint16Array::view(buffer.1)};
                console::log_1(&("use_indices_data".into()));
                console::log_1(&indices_buffer);
                gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.indices_buffer));
                gl.buffer_data_with_array_buffer_view(
                    WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
                    &indices_buffer,
                    WebGlRenderingContext::STATIC_DRAW,
                );
                r = buffer.1.len() as u32;
            },
            None => ()
        }

        console::log_1(&("update end".into()));

        Ok(r)
    }

    pub fn render(&self, gl: &WebGlRenderingContext){
        console::log_1(&("render start".into()));
        //position
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.positions_buffer));
        let position_local = gl.get_attrib_location(&self.program, "position");
        console::log_1(&("render start1".into()));
        gl.vertex_attrib_pointer_with_i32(position_local as u32, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
        console::log_1(&("render start2".into()));

        //mesh_index
        let index_local = gl.get_attrib_location(&self.program, "index");
        console::log_2(&("index".into()), &(index_local.to_string().into()));
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.mesh_index_buffer));
        console::log_1(&("render start3".into()));
        gl.vertex_attrib_pointer_with_i32(index_local as u32, 1, WebGlRenderingContext::UNSIGNED_SHORT, false, 0, 0);
        console::log_1(&("render start4".into()));

        //paramTexture
        gl.active_texture(WebGlRenderingContext::TEXTURE0);
        gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&self.tex_buffer));
        let location = gl.get_uniform_location(&self.program, "paramTexture").unwrap();
        gl.uniform1i(Some(&location), 0);

        //indices
        gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.indices_buffer));
        self.set_uniforms(gl);
        gl.draw_elements_with_i32(
            WebGlRenderingContext::TRIANGLES,
            self.indices_buffer_len as i32 ,
            WebGlRenderingContext::UNSIGNED_SHORT,
            0,
        );
        console::log_1(&("render end".into()));
    }
}



