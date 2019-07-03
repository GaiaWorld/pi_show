use hal_core::{Context, ShaderType, Program, UniformLayout};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlot, convert_to_mut};
use implement::{WebGLProgramImpl};

#[derive(Clone)]
pub struct WebGLProgramWrap(pub GLSlot<WebGLProgramImpl>);

impl Program for WebGLProgramWrap {
    type RContext = WebGLContextWrap;

    fn new_with_vs_fs(context: &Self::RContext, vs_name: &str, vs_defines: &[&str], fs_name: &str, fs_defines: &[&str], uniform_layout: &UniformLayout) -> Result<<Self::RContext as Context>::ContextProgram, String> {
        match WebGLProgramImpl::new_with_vs_fs(&context.rimpl, vs_name, vs_defines, fs_name, fs_defines, uniform_layout) {
            Err(s) => Err(s),
            Ok(program) => Ok(Self(GLSlot::new(&context.program, program))),
        }
    }

    fn delete(&self) {
        let slab = convert_to_mut(self.0.slab.as_ref());
        let mut program = slab.remove(self.0.index);
        program.delete();
    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    // TODO
    fn get_shader_info(&self, stype: ShaderType) -> Option<(&str, &[&str])> {
        None
    }
}