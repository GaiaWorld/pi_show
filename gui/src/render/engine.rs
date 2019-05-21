use std::sync::Arc;

use hal_core::{Context};
use render::res::{ResMgr, TextureRes};

pub struct Engine<C: Context>{
    pub gl: C,
    pub res_mgr: ResMgr<C>,
}

impl<C: Context> Engine<C> {
    pub fn new(gl: C) -> Self {
        Engine{
            gl: gl,
            res_mgr: ResMgr::new(),
        }
    }
}

unsafe impl<C: Context + Sync> Sync for Engine<C> {}
unsafe impl<C: Context + Send> Send for Engine<C> {}