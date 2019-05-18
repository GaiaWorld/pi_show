use atom::{Atom};
use hal_core::{Geometry};

pub struct WebGLGeometryImpl {
    
}

impl Geometry for WebGLGeometryImpl {
    fn add_attribute(&mut self, _name: &Atom, _item_count: u32, _data: &[u8], _is_updatable: bool) -> Result<(), String> {
        Err("not impl".to_string())
    }
    
    /**
     * 设置索引数据，
     * 如果indices已经有数据，崩溃
     */
    fn set_indices_short(&mut self, _data: &[u16]) -> Result<(), String> {
        Err("not impl".to_string())
    }

    /**
     * 更新属性数据，
     * 不存在属性名，崩溃
     * is_updatable为false，崩溃
     * item_index + data.len() >= vertex_count，崩溃
     */
    fn update_attribute(&self, _name: &Atom, _item_index: u32, _data: &[u8]) {
        
    }
}

impl Drop for WebGLGeometryImpl {
    fn drop(&mut self) {
    }
}