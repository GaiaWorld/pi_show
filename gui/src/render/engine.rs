use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use fnv::FnvHashMap;

use webgl_rendering_context::{WebGLRenderingContext, WebGLShader, WebGLProgram};
use stdweb::unstable::TryInto;

use render::extension::*;

pub struct Engine {
    pub gl: WebGLRenderingContext,
    compiled_programs: FnvHashMap<u64, WebGLProgram>,
}

pub enum ShaderType{
    Vertex,
    Fragment,
}

pub trait GetCode {
    fn get_code(&self) -> &str;
}


impl Engine {
    pub fn new(gl: WebGLRenderingContext) -> Engine {
        init_gl(&gl);
        Engine{
            gl,
            compiled_programs: FnvHashMap::default(),
        }
    }

    pub fn create_program<C: Hash + AsRef<str>, D: Hash + AsRef<str>>(&mut self, vertex_code: &C, fragment_code: &C, defines: &Vec<D>) -> Result<u64, String>{
        let mut hasher = DefaultHasher::new();
        vertex_code.hash(&mut hasher);
        fragment_code.hash(&mut hasher);
        for v in defines.iter() {
            v.hash(&mut hasher);
        }
        let hash = hasher.finish();
        match self.compiled_programs.get(&hash) {
            Some(_) => Ok(hash),
            None => {
                let shader_program = self.create_shader_program(vertex_code, fragment_code, defines)?;
                self.compiled_programs.insert(hash, shader_program);
                Ok(hash)
            },
        }
    }

    pub fn create_shader_program<C: AsRef<str>, D: AsRef<str>>(&self, vertex_code: &C, fragment_code: &C, defines: &Vec<D>) -> Result<WebGLProgram, String> {
        let vertex_shader = self.compile_shader(vertex_code, ShaderType::Vertex, defines)?;
        let fragment_shader = self.compile_shader(fragment_code, ShaderType::Fragment, defines)?;

        self._create_shader_program(&vertex_shader, &fragment_shader)
    }

    pub fn create_raw_shader_program<C: AsRef<str>, D: AsRef<str>>(&self, vertex_code: &C, fragment_code: &C) -> Result<WebGLProgram, String>{
        let vertex_shader = self.compile_raw_shader(vertex_code, ShaderType::Vertex)?;
        let fragment_shader = self.compile_raw_shader(fragment_code, ShaderType::Fragment)?;

        self._create_shader_program(&vertex_shader, &fragment_shader)
    }

    pub fn compile_shader<C: AsRef<str>, D: AsRef<str>>(&self, source: &C, ty: ShaderType, defines: &Vec<D>) -> Result<WebGLShader, String> {
        let mut s = "".to_string();
        for v in defines.iter() {
            s += "define ";
            s += v.as_ref();
            s += ";";
        }
        self.compile_raw_shader(&(s + source.as_ref()), ty)
    }

    pub fn compile_raw_shader<C: AsRef<str>>(&self, source: &C, ty: ShaderType) -> Result<WebGLShader, String> {
        let gl = &self.gl;
        let shader = gl.create_shader(match ty {
            ShaderType::Vertex => WebGLRenderingContext::VERTEX_SHADER,
            ShaderType::Fragment => WebGLRenderingContext::FRAGMENT_SHADER,
        }).ok_or_else(|| String::from("Unable to create shader object"))?;

        gl.shader_source(&shader, source.as_ref());
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

    fn _create_shader_program(&self, vertex_shader: &WebGLShader, fragment_shader: &WebGLShader) -> Result<WebGLProgram, String> {
        let gl = &self.gl;
        let shader_program = gl.create_program().ok_or_else(|| String::from("Unable to create shader object"))?;

        gl.attach_shader(&shader_program, vertex_shader);
        gl.attach_shader(&shader_program, fragment_shader);

        gl.link_program(&shader_program);

        let parameter: bool = gl.get_program_parameter(&shader_program, WebGLRenderingContext::LINK_STATUS).try_into().unwrap_or(false);
        if parameter{
            Ok(shader_program)
        } else {
            Err(gl
                .get_program_info_log(&shader_program)
                .unwrap_or_else(|| "Unknown error creating program object".into()))
        }
    }
}

fn init_gl(gl: &WebGLRenderingContext){
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
}