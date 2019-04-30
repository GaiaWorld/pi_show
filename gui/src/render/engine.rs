use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use fnv::FnvHashMap;

use webgl_rendering_context::{WebGLRenderingContext, WebGLShader, WebGLProgram, WebGLUniformLocation};
use stdweb::unstable::TryInto;

use atom::Atom;

use render::extension::*;
use render::res::ResMgr;

pub struct Engine {
    pub gl: WebGLRenderingContext,
    pub res_mgr: ResMgr,
    compiled_programs: FnvHashMap<u64, Program>,
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
            res_mgr: ResMgr::new(),
            compiled_programs: FnvHashMap::default(),
        }
    }

    pub fn lookup_program(&self, id: u64) -> Option<&Program>{
        self.compiled_programs.get(&id)
    }

    pub fn lookup_program_mut(&mut self, id: u64) -> Option<&mut Program>{
        self.compiled_programs.get_mut(&id)
    }

    pub fn create_program<C: Hash + AsRef<str>, D: Hash + AsRef<str>>(&mut self, vertex_code: &C, fragment_code: &C, defines: &Vec<D>) -> Result<u64, String>{
        println!("engine create_program----------------------------");
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
                // println!("defines12----------------------------{}", hash);
                let shader_program = self.create_shader_program(vertex_code, fragment_code, defines)?;
                let program = Program{
                    program: shader_program,
                    uniform_locations: FnvHashMap::default(),
                    attr_locations: FnvHashMap::default(),
                };
                self.compiled_programs.insert(hash, program);
                Ok(hash)
            },
        }
    }

    pub fn create_shader_program<C: AsRef<str>, D: AsRef<str>>(&self, vertex_code: &C, fragment_code: &C, defines: &Vec<D>) -> Result<WebGLProgram, String> {
        println!("engine create_shader_program----------------------------");
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
            s += "#define ";
            s += v.as_ref();
            s += "\n";
        }
        // println!("ssssss----------------------------{:?}", s.clone() + source.as_ref());
        self.compile_raw_shader(&(s + source.as_ref()), ty)
    }

    pub fn compile_raw_shader<C: AsRef<str>>(&self, source: &C, ty: ShaderType) -> Result<WebGLShader, String> {
        println!("compile_raw_shader----------------------------");
        let gl = &self.gl;
        js!{
            console.log("gl-------------------", @{&gl});
        }
        js!{
            console.log("WebGLRenderingContext::VERTEX_SHADER-------------------", @{WebGLRenderingContext::VERTEX_SHADER});
        }
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
            println!("Unknown error creating shader----------------------------");
            Err(gl
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown error creating shader".into()))
        }
    }

    fn _create_shader_program(&self, vertex_shader: &WebGLShader, fragment_shader: &WebGLShader) -> Result<WebGLProgram, String> {
        println!("engine _create_shader_program----------------------------");
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

pub struct Program {
    pub program: WebGLProgram,
    pub uniform_locations: FnvHashMap<Atom, WebGLUniformLocation>,
    pub attr_locations: FnvHashMap<Atom, u32>,
}

pub fn get_uniform_location(gl: &WebGLRenderingContext,program: &WebGLProgram ,name: &Atom) -> WebGLUniformLocation{
    match gl.get_uniform_location(program, name) {
        Some(v) => v,
        None => panic!("get_uniform_location is None: {:?}", name),
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
    gl.enable(WebGLRenderingContext::BLEND);
    gl.blend_func(WebGLRenderingContext::SRC_ALPHA, WebGLRenderingContext::ONE_MINUS_SRC_ALPHA);

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(WebGLRenderingContext::COLOR_BUFFER_BIT);

    gl.enable(WebGLRenderingContext::DEPTH_TEST);
    gl.enable(WebGLRenderingContext::DEPTH_WRITEMASK);
}