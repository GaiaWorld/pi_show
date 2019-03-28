use std::ops::{Deref, DerefMut};
use webgl_rendering_context::{WebGLRenderingContext, WebGLShader, WebGLProgram};
use stdweb::unstable::TryInto;

pub fn compile_shader(
    gl: &WebGLRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGLShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    let parameter: bool = gl.get_shader_parameter(&shader, WebGLRenderingContext::COMPILE_STATUS).try_into().unwrap_or(false);
    if parameter{
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error creating shader".into()))
    }
}

pub fn link_program<'a, T: IntoIterator<Item = &'a WebGLShader>>(
    gl: &WebGLRenderingContext,
    shaders: T,
) -> Result<WebGLProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    for shader in shaders {
        gl.attach_shader(&program, shader)
    }
    gl.link_program(&program);

    let parameter: bool = gl.get_program_parameter(&program, WebGLRenderingContext::LINK_STATUS).try_into().unwrap_or(false);
    if parameter{
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error creating program object".into()))
    }
}

#[derive(Clone)]
pub struct GuiWorldViewProjection([f32; 16]);

impl GuiWorldViewProjection {
    pub fn new(width: f32, height: f32) -> GuiWorldViewProjection{
        let (left, right, top, bottom, near, far) = (0.0, width, 0.0, height, 0.1, 1000.0);
        GuiWorldViewProjection([
                2.0 / (right - left),                  0.0,                               0.0,                        0.0,
                    0.0,                     2.0 / (top - bottom),                       0.0,                        0.0,
                    0.0,                              0.0,                       -2.0 / (far - near),   -(far + near) / (far - near),
            -(right + left) / (right - left), -(top + bottom) / (top - bottom),           0.0,                        1.0
            ]
        )
    }
}

impl Deref for GuiWorldViewProjection{
    type Target = [f32];
    fn deref(&self) -> &[f32]{
        &self.0
    }
}

impl DerefMut for GuiWorldViewProjection{
    fn deref_mut(&mut self) -> &mut [f32]{
        &mut self.0
    }
}

// pub struct WebGlBuffer(Rc<usize>);

// impl WebGlBuffer {
//     pub fn new(index: usize) -> WebGlBuffer{
//         WebGlBuffer(Rc::new(index))
//     }
// }

// impl Deref for WebGlBuffer{
//     type Target = usize;
//     fn deref(&self) -> &usize{
//         &self.0
//     }
// }
