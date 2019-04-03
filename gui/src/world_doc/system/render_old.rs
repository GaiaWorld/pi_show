use wcs::world::{System};

use world_doc::WorldDocMgr;

pub struct Render;

impl System<(), WorldDocMgr> for Render{
    fn run(&self, _e: &(), component_mgr: &mut WorldDocMgr){
        component_mgr.render.render();
    }
}