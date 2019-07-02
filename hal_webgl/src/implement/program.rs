use atom::{Atom};
use hal_core::{ShaderType};
use wrap::{WebGLContextWrap};

pub struct WebGLProgramImpl {
}

impl WebGLProgramImpl {

    pub fn delete(&self) {

    }

    pub fn new_with_vs_fs(context: &WebGLContextWrap, vs_name: &Atom, vs_defines: &[Atom], fs_name: &Atom, fs_defines: &[Atom]) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }

    pub fn attach_shader(&self, shader_hash: u64) -> Result<(), String> {
        Err("not implmentation".to_string())
    }
    
    pub fn link(&self) {

    }

    pub fn get_shader_info(&self, stype: ShaderType) -> Option<(&Atom, &[Atom])> {
        None
    }

    fn set_shader_code<C: AsRef<str>>(conext: &WebGLContextWrap, name: &Atom, code: &C) {

    }

    pub fn compile_shader(context: &WebGLContextWrap, shader_type: ShaderType, name: &Atom, defines: &[Atom]) -> Result<u64, String> {
        Err("not implmentation".to_string())
    }
}