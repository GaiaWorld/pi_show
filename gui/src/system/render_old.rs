use wcs::world::{System};

use world::GuiComponentMgr;

pub struct Render;

impl System<(), GuiComponentMgr> for Render{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        component_mgr.render.render();
    }
}