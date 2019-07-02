use hal_core::{Context, ShaderType, Program, UniformLayout};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlab, GLSlot, convert_to_mut};
use implement::{WebGLProgramImpl};

#[derive(Clone)]
pub struct WebGLProgramWrap(GLSlot);

impl Program for WebGLProgramWrap {
    type RContext = WebGLContextWrap;

    fn new_with_vs_fs(context: &Self::RContext, vs_name: &str, vs_defines: &[&str], fs_name: &str, fs_defines: &[&str], uniform_layout: &UniformLayout) -> Result<<Self::RContext as Context>::ContextProgram, String> {
        Err("not implmentation".to_string())
    }

    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    fn get_shader_info(&self, stype: ShaderType) -> Option<(&str, &[&str])> {
        None
    }
}