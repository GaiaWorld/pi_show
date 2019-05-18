use std::rc::Rc;

use render::res::ResMgr;

pub struct Engine<C>{
    pub gl: Rc<C>,
    pub res_mgr: ResMgr,
}

impl<C> Engine<C> {
    pub fn new(gl: C) -> Self {
        Engine{
            gl: Rc::new(gl),
            res_mgr: ResMgr::new(),
        }
    }
}