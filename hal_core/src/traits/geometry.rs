use common::{AttributeName};

/** 
 * 几何数据：存放attribute，和index的地方
 */
pub trait Geometry: Drop {

    /** 
     * 是否有属性
     */   
    fn has_attribute(&self, name: &AttributeName) -> bool;

    /** 
     * 获取当前的顶点个数
     */   
    fn get_vertex_count(&self) -> u32;

    /** 
     * 设置顶点的个数
     * 注：一旦设置了顶点个数，就意味着老的attribute和indiecs无效，要全部重新设置
     */
    fn set_vertex_count(&mut self, count: u32);

    /**
     * 设置属性数据
     * item_count，每个顶点的该属性占多少个float
     * is_updatable尽量是false，以提高最优性能
     * 如果data为None，开辟一个长度是：vertex_count * item_count * 4大小的buffer
     * 如果用同name设置多次，会根据上次的is_updatable来决定是否需要用 buffer_sub_data 还是用 buffer_data
     */
    fn set_attribute(&mut self, name: &AttributeName, item_count: u32, data: Option<&[f32]>, is_updatable: bool) -> Result<(), String>;
     
    /**
     * 删除属性
     */
    fn remove_attribute(&mut self, name: &AttributeName);

    /**
     * 设置索引
     */
    fn set_indices_short(&mut self, data: &[u16], is_updatable: bool) -> Result<(), String>;

    /**
     * 删除索引
     */
    fn remove_indices(&mut self);

    /**
     * 更新属性数据，
     * 不存在属性名，崩溃
     * is_updatable为false，崩溃
     * item_offset + data.len() >= vertex_count * size，崩溃
     */
    fn update_attribute(&self, name: &AttributeName, item_offset: u32, data: &[f32]);
}