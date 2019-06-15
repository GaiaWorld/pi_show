use atom::{Atom};
use hal_core::{Context, ShaderType, Program};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlot};
use implement::{WebGLProgramImpl};

#[derive(Clone)]
pub struct WebGLProgramWrap(GLSlot);

impl Program for WebGLProgramWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext) -> Result<<Self::RContext as Context>::ContextProgram, String> {
        Err("not implmentation".to_string())
    }
    
    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn new_with_vs_fs(context: &Self::RContext, vs_name: &Atom, vs_defines: &[Atom], fs_name: &Atom, fs_defines: &[Atom]) -> Result<<Self::RContext as Context>::ContextProgram, String> {
        Err("not implmentation".to_string())
    }

    fn get_shader_info(&self, stype: ShaderType) -> Option<(&Atom, &[Atom])> {
        None
    }
}