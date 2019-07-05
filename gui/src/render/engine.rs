use share::Share;
use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hasher, Hash };

use fnv::FnvHashMap;

use atom::Atom;
use hal_core::{Context, Pipeline, RasterState, BlendState, StencilState, DepthState, ShaderType};
use util::res_mgr::ResMgr;

pub struct PipelineInfo {
    pub pipeline: Share<Pipeline>,
    pub vs: Atom,
    pub fs: Atom,
    pub defines: Vec<Atom>,
    pub rs: Share<RasterState>,
    pub bs: Share<BlendState>,
    pub ss: Share<StencilState>,
    pub ds: Share<DepthState>,
    pub start_hash: u64,
}

pub struct Engine<C: Context>{
    pub gl: C,
    pub res_mgr: ResMgr,
    pub pipelines: FnvHashMap<u64, Share<PipelineInfo>>,
}

impl<C: Context> Engine<C> {
    pub fn new(gl: C) -> Self {
        Engine{
            gl: gl,
            res_mgr: ResMgr::new(36000),
            pipelines: FnvHashMap::default(),
        }
    }

    pub fn create_pipeline(&mut self, start_hash: u64, vs_name: &Atom, fs_name: &Atom, defines: &[Atom], rs: Share<RasterState>, bs: Share<BlendState>, ss: Share<StencilState>, ds: Share<DepthState>) -> Share<PipelineInfo> {
        // pipeline hash
        let mut hasher = DefaultHasher::new();
        start_hash.hash(&mut hasher);
        vs_name.hash(&mut hasher);
        fs_name.hash(&mut hasher);
        for d in defines.iter() {
            d.hash(&mut hasher);
        }
        let key = hasher.finish();

        let gl = &mut self.gl;
        let r = self.pipelines.entry(key).or_insert_with(|| {
            let vs = match gl.compile_shader(ShaderType::Vertex, vs_name, defines) {
                Ok(r) => r,
                Err(s) => panic!("compile_vs_shader error: {:?}", s),
            };
            let fs = match gl.compile_shader(ShaderType::Fragment, fs_name, defines) {
                Ok(r) => r,
                Err(s) => panic!("compile_fs_shader error: {:?}", s),
            };
            match gl.create_pipeline(vs, fs, rs.clone(), bs.clone(), ss.clone(), ds.clone()){
                Ok(r) => {
                    let defines = Vec::from(defines);
                    Share::new(PipelineInfo{
                        start_hash: start_hash,
                        pipeline: Share::new(r),
                        vs: vs_name.clone(),
                        fs: fs_name.clone(),
                        defines: defines,
                        rs: rs.clone(),
                        bs: bs.clone(),
                        ss: ss.clone(),
                        ds: ds.clone(),
                    })
                },
                Err(e) => panic!("create_pipeline error: {:?}", e), 
            }
        });

        r.clone()
    }
}

unsafe impl<C: Context> Sync for Engine<C> {}
unsafe impl<C: Context> Send for Engine<C> {}