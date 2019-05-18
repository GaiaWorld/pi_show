use atom::{Atom};

/** 
 * 几何数据：存放attribute，和index的地方
 */
pub trait Geometry {

    /**
     * 设置属性数据
     * item_count，每个顶点的该属性占多少个float
     * is_updatable尽量是false，以提高最优性能
     */
    fn add_attribute(&mut self, name: &Atom, item_count: u32, data: &[u8], is_updatable: bool) -> Result<(), String>;
    
    /**
     * 设置索引数据，
     * 如果indices已经有数据，崩溃
     */
    fn set_indices_short(&mut self, data: &[u16]) -> Result<(), String>;

    /**
     * 更新属性数据，
     * 不存在属性名，崩溃
     * is_updatable为false，崩溃
     * item_index + data.len() >= vertex_count，崩溃
     */
    fn update_attribute(&self, name: &Atom, item_index: u32, data: &[u8]);
}