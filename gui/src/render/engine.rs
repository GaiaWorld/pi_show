use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hasher, Hash };

use fnv::FnvHashMap;

use atom::Atom;
use hal_core::{Context, Pipeline, RasterState, BlendState, StencilState, DepthState, ShaderType};
use render::res::{ResMgr};

pub struct PipelineInfo {
    pub pipeline: Arc<Pipeline>,
    pub vs: Atom,
    pub fs: Atom,
    pub defines: Vec<Atom>,
    pub rs: Arc<RasterState>,
    pub bs: Arc<BlendState>,
    pub ss: Arc<StencilState>,
    pub ds: Arc<DepthState>,
    pub start_hash: u64,
}

pub struct Engine<C: Context>{
    pub gl: C,
    pub res_mgr: ResMgr<C>,
    pub pipelines: FnvHashMap<u64, Arc<PipelineInfo>>,
}

impl<C: Context> Engine<C> {
    pub fn new(gl: C) -> Self {
        Engine{
            gl: gl,
            res_mgr: ResMgr::new(),
            pipelines: FnvHashMap::default(),
        }
    }

    pub fn create_pipeline(&mut self, start_hash: u64, vs_name: &Atom, fs_name: &Atom, defines: &[Atom], rs: Arc<RasterState>, bs: Arc<BlendState>, ss: Arc<StencilState>, ds: Arc<DepthState>) -> Arc<PipelineInfo> {
        let vs = match self.gl.compile_shader(ShaderType::Vertex, vs_name, defines) {
            Ok(r) => r,
            Err(s) => panic!("compile_vs_shader error: {:?}", s),
        };
        let fs = match self.gl.compile_shader(ShaderType::Fragment, fs_name, defines) {
            Ok(r) => r,
            Err(s) => panic!("compile_fs_shader error: {:?}", s),
        };
    
        let mut hasher = DefaultHasher::new();
        start_hash.hash(&mut hasher);
        vs.hash(&mut hasher);
        fs.hash(&mut hasher);
        for d in defines.iter() {
            d.hash(&mut hasher);
        }
        let key = hasher.finish();
        // println!("create_pipeline1, defines:{:?}, vs: {:?}, fs: {:?}, ds: {:?}, start_hash: {}, key:{}", defines, fs_name, fs, ds, start_hash, key);
        let gl = &mut self.gl;
        let r = self.pipelines.entry(key).or_insert_with(|| {
            match gl.create_pipeline(vs, fs, rs.clone(), bs.clone(), ss.clone(), ds.clone()){
                Ok(r) => {
                    println!("create_pipeline, defines:{:?}, vs: {:?}, fs: {:?}, ds: {:?}, start_hash: {}", defines, fs_name, fs, ds, start_hash);
                    let defines = Vec::from(defines);
                    Arc::new(PipelineInfo{
                        start_hash: start_hash,
                        pipeline: Arc::new(r),
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