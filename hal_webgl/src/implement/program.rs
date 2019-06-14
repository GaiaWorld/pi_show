use std::sync::{Arc};
use atom::{Atom};
use hal_core::{ShaderType};
use wrap::{WebGLContextWrap};

pub struct WebGLProgramImpl {
}

impl WebGLProgramImpl {

    fn new(context: &Arc<WebGLContextWrap>) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn new_with_vs_fs(context: &Arc<WebGLContextWrap>, vs_name: &Atom, vs_defines: &[Atom], fs_name: &Atom, fs_defines: &[Atom]) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }

    fn attach_shader(&self, shader_hash: u64) -> Result<(), String> {
        Err("not implmentation".to_string())
    }
    
    fn link(&self) {

    }

    fn get_shader_info(&self, stype: ShaderType) -> Option<(&Atom, &[Atom])> {
        None
    }

    fn set_shader_code<C: AsRef<str>>(conext: &WebGLContextWrap, name: &Atom, code: &C) {

    }

    fn compile_shader(context: &WebGLContextWrap, shader_type: ShaderType, name: &Atom, defines: &[Atom]) -> Result<u64, String> {
        Err("not implmentation".to_string())
    }
}