use std::hash::{ Hash, Hasher };

use fx_hashmap::FxHashMap32;
use fxhash::FxHasher32;
use share::Share;

use hal_core::*;
use util::res_mgr::ResMgr;

pub struct Engine<C: HalContext + 'static>{
    pub gl: C,
    pub res_mgr: ResMgr,
    pub programs: FxHashMap32<u64, Share<HalProgram>>,
}

impl<C: HalContext + 'static> Engine<C> {
    pub fn new(gl: C, time: u32) -> Self {
        Engine{
            gl: gl,
            res_mgr: ResMgr::new(time),
            programs: FxHashMap32::default(),
        }
    }

    pub fn create_program(
        &mut self,
        vs_id: u64,
        fs_id: u64,
        vs_name: &str,
        vs_defines: &dyn Defines,
        fs_name: &str,
        fs_defines: &dyn Defines,
        paramter: &dyn ProgramParamter,
    ) -> Share<HalProgram>{
        let mut hasher = FxHasher32::default();
        vs_id.hash(&mut hasher);
        vs_defines.id().hash(&mut hasher);
        fs_id.hash(&mut hasher);
        fs_defines.id().hash(&mut hasher);
        let hash = hasher.finish();

        let gl = &self.gl;
        self.programs.entry(hash).or_insert_with(|| {
            let ubos = paramter.get_layout();
            let mut uniforms = Vec::with_capacity(ubos.len());
            for ubo in ubos.iter() {
                uniforms.push(paramter.get_value(ubo).unwrap().get_layout());
            }

            

            let uniform_layout = UniformLayout{
                ubos: ubos,
                uniforms: uniforms.as_slice(),
                textures: paramter.get_texture_layout(),
            };
            match gl.program_create_with_vs_fs(vs_id, fs_id, vs_name, vs_defines.list(), fs_name, fs_defines.list(), &uniform_layout) {
                Ok(r) => Share::new(r),
                Err(e) => panic!("create_program error: {:?}", e),
            }
        }).clone()
    }
}

unsafe impl<C: HalContext + 'static> Sync for Engine<C> {}
unsafe impl<C: HalContext + 'static> Send for Engine<C> {}