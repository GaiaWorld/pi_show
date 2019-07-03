use atom::{Atom};
use hal_core::{ShaderType, UniformLayout};
use share::{Share};
use implement::context::{WebGLContextImpl}; 

pub struct WebGLProgramImpl {
    context: Share<WebGLContextImpl>,
}

impl WebGLProgramImpl {

    pub fn delete(&self) {

    }

    pub fn new_with_vs_fs(context: &Share<WebGLContextImpl>, vs_name: &str, vs_defines: &[&str], fs_name: &str, fs_defines: &[&str], uniform_layout: &UniformLayout) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }

    pub fn get_shader_info(&self, stype: ShaderType) -> Option<(&Atom, &[Atom])> {
        None
    }
}