use hal_core::{Geometry, AttributeName};

pub struct NullGeometryImpl {
    
}

impl AsRef<Self> for NullGeometryImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl Geometry for NullGeometryImpl {

    fn has_attribute(&self, _name: &AttributeName) -> bool {
        false
    }

    fn get_vertex_count(&self) -> u32 {
        0
    }

    fn set_vertex_count(&mut self, _count: u32) {

    }

    fn set_attribute(&mut self, _name: &AttributeName, _item_count: u32, _data: Option<&[f32]>, _is_updatable: bool) -> Result<(), String> {
        Ok(())
    }
     
    fn remove_attribute(&mut self, _name: &AttributeName) {

    }

    fn set_indices_short(&mut self, _data: &[u16], _is_updatable: bool) -> Result<(), String> {
        Ok(())
    }

    fn remove_indices(&mut self) {
        
    }

    fn update_attribute(&self, _name: &AttributeName, _item_offset: u32, _data: &[f32]) {
        
    }
}

impl Drop for NullGeometryImpl {
    fn drop(&mut self) {
    }
}