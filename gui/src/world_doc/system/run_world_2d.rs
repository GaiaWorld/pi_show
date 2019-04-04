//八叉树系统, 现在主要为事件做索引

use std::rc::{Rc};

use wcs::world::System;

use world_doc::WorldDocMgr;

pub struct RunWorld2d;

impl RunWorld2d {
    pub fn init(_component_mgr: &mut WorldDocMgr) -> Rc<RunWorld2d>{
        let r = Rc::new(RunWorld2d);
        r
    }
}

impl System<(), WorldDocMgr> for RunWorld2d{
    fn run(&self, _e: &(), component_mgr: &mut WorldDocMgr){
        component_mgr.world_2d.run(());
    }
}