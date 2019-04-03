use std::cell::RefCell;
use std::rc::{Rc};

use webgl_rendering_context::{WebGLRenderingContext, WebGLTexture, WebGLBuffer, WebGLProgram};
use stdweb::web::html_element::CanvasElement;
use stdweb::{UnsafeTypedArray};

use wcs::world::{System};

use render::vector_sdf::{VectorSdf};
use render::extension::*;
use render::gl::{compile_shader, link_program, GuiWorldViewProjection};
use world::{DocumentMgr};

pub struct Render(RefCell<RenderImpl>);

pub struct RenderImpl{
    gl: WebGLRenderingContext,
    pub view_projection: GuiWorldViewProjection,
    opaque_sdf_render: VectorSdfRender,
}

impl Render {
    pub fn init(component_mgr: &mut DocumentMgr, canvas: &CanvasElement) -> Result<Rc<Render>, String> {
        let gl: WebGLRenderingContext = canvas.get_context().unwrap();
        gl.get_extension::<OESElementIndexUint>();
        gl.get_extension::<ANGLEInstancedArrays>();
        gl.get_extension::<OESStandardDerivatives>();
        gl.get_extension::<OESTextureFloat>();
        gl.get_extension::<OESTextureFloatLinear>();
        gl.get_extension::<OESTextureHalfFloat>();
        gl.get_extension::<OESTextureHalfFloatLinear>();
        gl.get_extension::<EXTSRGB>();
        gl.get_extension::<OESVertexArrayObject>();
        gl.get_extension::<EXTTextureFilterAnisotropic>();
        gl.get_extension::<WEBKITEXTTextureFilterAnisotropic>();
        gl.get_extension::<EXTFragDepth>();
        gl.get_extension::<WEBGLDepthTexture>();
        gl.get_extension::<WEBGLColorBufferFloat>();
        gl.get_extension::<EXTColorBufferHalfFloat>();
        gl.get_extension::<EXTShaderTextureLod>();
        gl.get_extension::<WEBGLDrawBuffers>();
        gl.get_extension::<GLOESStandardDerivatives>();
        gl.enable(WebGLRenderingContext::BLEND);
        gl.blend_func(WebGLRenderingContext::SRC_ALPHA, WebGLRenderingContext::ONE_MINUS_SRC_ALPHA);

        gl.clear_color(1.0, 0.0, 0.0, 1.0);
        gl.clear(WebGLRenderingContext::COLOR_BUFFER_BIT);

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

impl System<(), DocumentMgr> for Render{
    fn run(&self, _e: &(), component_mgr: &mut DocumentMgr){
        let mut render = self.0.borrow_mut();
        let len = render.opaque_sdf_render.update(&render.gl, &mut component_mgr.opaque_vector);
        render.opaque_sdf_render.indices_buffer_len = len;
        render.opaque_sdf_render.render(&render.gl)
    }
}


pub struct VectorSdfRender{
    pub tex_buffer: WebGLTexture,
    pub positions_buffer: WebGLBuffer,
    pub mesh_index_buffer: WebGLBuffer,
    pub indices_buffer: WebGLBuffer,
    pub program: WebGLProgram,
    pub indices_buffer_len: u32,
    pub screen_size: (f32, f32),
    pub world_view: GuiWorldViewProjection,
    pub texture_size: (usize, usize),
}

impl VectorSdfRender {
    pub fn init(gl: &WebGLRenderingContext, sdf: &mut VectorSdf, width: f32, height: f32, world_view: &mut GuiWorldViewProjection) -> Result<VectorSdfRender, String>{
        let vert_shader = compile_shader(
            &gl,
            WebGLRenderingContext::VERTEX_SHADER,
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
            WebGLRenderingContext::FRAGMENT_SHADER,
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
        gl.active_texture(WebGLRenderingContext::TEXTURE0);
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&tex_buffer));
        let b = sdf.use_tex_data().unwrap();
        let buffer = b.1;
        gl.tex_image2_d(WebGLRenderingContext::TEXTURE_2D, 0, WebGLRenderingContext::RGBA as i32, 8, 8, 0, WebGLRenderingContext::RGBA, WebGLRenderingContext::FLOAT, Some(buffer));
        // let buffer = unsafe{js_sys::Float32Array::view(b.1)};
        // console::log_1(&(b.1.len().to_string().into()));
        // gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(WebGLRenderingContext::TEXTURE_2D, 0, WebGLRenderingContext::RGBA as i32, 8, 8, 0, WebGLRenderingContext::RGBA, WebGLRenderingContext::FLOAT, Some(&buffer))?;
        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_MAG_FILTER, WebGLRenderingContext::NEAREST as i32);
        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_MIN_FILTER, WebGLRenderingContext::NEAREST as i32);
        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_WRAP_S, WebGLRenderingContext::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_WRAP_T, WebGLRenderingContext::CLAMP_TO_EDGE as i32);
        gl.uniform1i(Some(&location), 0);

        //设置vertex
        let positions_buffer = gl.create_buffer().ok_or("failed to create buffer")?;
        gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&positions_buffer));
        let position_local = gl.get_attrib_location(&program, "position");
        gl.vertex_attrib_pointer(position_local as u32, 3, WebGLRenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(position_local as u32);

        //设置mesh_index
        let mesh_index_buffer = gl.create_buffer().ok_or("failed to create buffer")?;
        let index_local = gl.get_attrib_location(&program, "index");
        gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&mesh_index_buffer));
        gl.vertex_attrib_pointer(index_local as u32, 1, WebGLRenderingContext::UNSIGNED_SHORT, false, 0, 0);
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

    pub fn set_uniforms(&self, gl: &WebGLRenderingContext) {
        //设置worldViewProjection
        let location = gl.get_uniform_location(&self.program, "worldViewProjection").unwrap();
        gl.uniform_matrix4fv(Some(&location), false, &(*self.world_view));

        //设置screenSize
        let location = gl.get_uniform_location(&self.program, "screenSize").unwrap();
        gl.uniform2f(Some(&location), self.screen_size.0, self.screen_size.1);

        //设置textureSize
        let location = gl.get_uniform_location(&self.program, "textureSize").unwrap();
        gl.uniform2f(Some(&location), self.texture_size.0 as f32, 3.0);
    }

    pub fn update(&self, gl: &WebGLRenderingContext, sdf: &mut VectorSdf) -> u32{
        let mut r = self.indices_buffer_len;
        match sdf.use_tex_data() {
            Some(tex_buffer) => {
                let buffer = tex_buffer.1;
                let rang = tex_buffer.0;
                gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&self.tex_buffer));
                gl.tex_sub_image2_d(WebGLRenderingContext::TEXTURE_2D, 0, rang.0 as i32, rang.1 as i32, rang.2 as i32, rang.3 as i32, WebGLRenderingContext::RGBA, WebGLRenderingContext::FLOAT, Some(buffer));
                let buffer = unsafe { UnsafeTypedArray::new( buffer ) };
                js!{
                    console.log("tex_data", @{buffer});
                }
            },
            None => ()
        }

        match sdf.use_positions_data(){
            Some(buffer) => {
                let buffer1 = unsafe { UnsafeTypedArray::new( buffer ) };
                let buffer = unsafe { UnsafeTypedArray::new( buffer ) };
                gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&self.positions_buffer));
                js!{
                    @{gl}.bufferData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{buffer}, @{WebGLRenderingContext::STATIC_DRAW});
                }
                js!{
                    console.log("position", @{buffer1});
                }
                // gl.buffer_data_1(
                //     WebGLRenderingContext::ARRAY_BUFFER,
                //     buffer,
                //     WebGLRenderingContext::STATIC_DRAW,
                // );
            },
            None => ()
        }

        match sdf.use_indexs_data(){
            Some(buffer) => {
                let mesh_index_buffer = buffer.0;
                let buffer1 = unsafe { UnsafeTypedArray::new( mesh_index_buffer ) };
                let mesh_index_buffer = unsafe { UnsafeTypedArray::new( mesh_index_buffer ) };
                gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&self.mesh_index_buffer));
                js!{
                    @{gl}.bufferData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{mesh_index_buffer}, @{WebGLRenderingContext::STATIC_DRAW});
                }
                js!{
                    console.log("mesh_index_buffer", @{buffer1});
                }
                // gl.buffer_data_1(
                //     WebGLRenderingContext::ARRAY_BUFFER,
                //     mesh_index_buffer,
                //     WebGLRenderingContext::STATIC_DRAW,
                // );

                let indices_buffer = buffer.1;
                let buffer1 = unsafe { UnsafeTypedArray::new( indices_buffer ) };
                let indices_buffer = unsafe { UnsafeTypedArray::new( indices_buffer ) };
                gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.indices_buffer));
                js!{
                    @{gl}.bufferData(@{WebGLRenderingContext::ELEMENT_ARRAY_BUFFER}, @{indices_buffer}, @{WebGLRenderingContext::STATIC_DRAW});
                }
                js!{
                    console.log("indices_buffer", @{buffer1});
                }
                // gl.buffer_data_1(
                //     WebGLRenderingContext::ELEMENT_ARRAY_BUFFER,
                //     indices_buffer,
                //     WebGLRenderingContext::STATIC_DRAW,
                // );
                r = buffer.1.len() as u32;
            },
            None => ()
        }

        r
    }

    pub fn render(&self, gl: &WebGLRenderingContext){
        //position
        gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&self.positions_buffer));
        let position_local = gl.get_attrib_location(&self.program, "position");
        gl.vertex_attrib_pointer(position_local as u32, 3, WebGLRenderingContext::FLOAT, false, 0, 0);

        //mesh_index
        let index_local = gl.get_attrib_location(&self.program, "index");
        gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&self.mesh_index_buffer));
        gl.vertex_attrib_pointer(index_local as u32, 1, WebGLRenderingContext::UNSIGNED_SHORT, false, 0, 0);

        //paramTexture
        gl.active_texture(WebGLRenderingContext::TEXTURE0);
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&self.tex_buffer));
        let location = gl.get_uniform_location(&self.program, "paramTexture").unwrap();
        gl.uniform1i(Some(&location), 0);

        //indices
        gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.indices_buffer));
        self.set_uniforms(gl);
        gl.draw_elements(
            WebGLRenderingContext::TRIANGLES,
            self.indices_buffer_len as i32 ,
            WebGLRenderingContext::UNSIGNED_SHORT,
            0,
        );
    }
}



