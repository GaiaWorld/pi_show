use wcs::world::{System};

use document::DocumentMgr;

pub struct Render;

impl System<(), DocumentMgr> for Render{
    fn run(&self, _e: &(), component_mgr: &mut DocumentMgr){
        component_mgr.render.render();
    }
}