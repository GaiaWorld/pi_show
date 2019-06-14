use std::sync::{Arc};
use atom::{Atom};
use hal_core::{Context, ShaderType, Program};
use wrap::context::{WebGLContextWrap};
use implement::{WebGLProgramImpl};

pub struct WebGLProgramWrap {
}

impl Program for WebGLProgramWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Arc<Self::RContext>) -> Result<<Self::RContext as Context>::ContextProgram, String> {
        Err("not implmentation".to_string())
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn new_with_vs_fs(context: &Arc<Self::RContext>, vs_name: &Atom, vs_defines: &[Atom], fs_name: &Atom, fs_defines: &[Atom]) -> Result<<Self::RContext as Context>::ContextProgram, String> {
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

    fn set_shader_code<C: AsRef<str>>(conext: &Self::RContext, name: &Atom, code: &C) {

    }

    fn compile_shader(context: &Self::RContext, shader_type: ShaderType, name: &Atom, defines: &[Atom]) -> Result<u64, String> {
        Err("not implmentation".to_string())
    }
}

impl Clone for WebGLProgramWrap {
    fn clone(&self) -> Self {
        Self {
            
        }
    }
}