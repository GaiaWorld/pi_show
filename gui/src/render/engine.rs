use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hasher, Hash };

use fnv::FnvHashMap;

use atom::Atom;
use hal_core::{Context, Pipeline, RasterState, BlendState, StencilState, DepthState, ShaderType};
use render::res::{ResMgr, TextureRes};

pub struct Engine<C: Context>{
    pub gl: C,
    pub res_mgr: ResMgr<C>,
    pub pipelines: FnvHashMap<u64, Arc<Pipeline>>,
}

impl<C: Context> Engine<C> {
    pub fn new(gl: C) -> Self {
        Engine{
            gl: gl,
            res_mgr: ResMgr::new(),
            pipelines: FnvHashMap::default(),
        }
    }

    pub fn create_pipeline(&mut self, start_hash: u64, vs_name: &Atom, fs_name: &Atom, defines: &[Atom], rs: Arc<RasterState>, bs: Arc<BlendState>, ss: Arc<StencilState>, ds: Arc<DepthState>) -> Arc<Pipeline> {
        let vs = match self.gl.compile_shader(ShaderType::Vertex, vs_name, defines) {
            Ok(r) => r,
            Err(s) => panic!("compile_vs_shader error"),
        };
        let fs = match self.gl.compile_shader(ShaderType::Fragment, fs_name, defines) {
            Ok(r) => r,
            Err(s) => panic!("compile_fs_shader error"),
        };

        let mut hasher = DefaultHasher::new();
        start_hash.hash(&mut hasher);
        vs.hash(&mut hasher);
        fs.hash(&mut hasher);
        let key = hasher.finish();

        let gl = &mut self.gl;
        let r = self.pipelines.entry(key).or_insert_with(|| {
            match gl.create_pipeline(vs, fs, rs, bs, ss, ds){
                Ok(r) => Arc::new(r),
                Err(_) => panic!("create_pipeline error"), 
            }
        });

        r.clone()
    }
}

unsafe impl<C: Context> Sync for Engine<C> {}
unsafe impl<C: Context> Send for Engine<C> {}