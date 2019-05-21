use atom::{Atom};
use hal_core::{Geometry};

pub struct NullGeometryImpl {
    
}

impl Geometry for NullGeometryImpl {

    fn has_attribute(&self, _name: &Atom) -> bool {
        false
    }

    fn get_vertex_count(&self) -> u32 {
        0
    }

    fn set_vertex_count(&mut self, _count: u32) {

    }

    fn set_attribute(&mut self, _name: &Atom, _item_count: u32, _data: Option<&[f32]>, _is_updatable: bool) -> Result<(), String> {
        Err("not impl".to_string())
    }
     
    fn remove_attribute(&mut self, _name: &Atom) {

    }

    fn set_indices_short(&mut self, _data: &[u16], _is_updatable: bool) -> Result<(), String> {
        Err("not impl".to_string())
    }

    fn remove_indices(&mut self) {
        
    }

    fn update_attribute(&self, _name: &Atom, _item_offset: u32, _data: &[f32]) {
        
    }
}

impl Drop for NullGeometryImpl {
    fn drop(&mut self) {
    }
}