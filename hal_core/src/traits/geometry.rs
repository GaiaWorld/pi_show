use common::{AttributeName};
use traits::context::{Context};

/** 
 * 几何数据：存放attribute，和index的地方
 */
pub trait Geometry : Sized + Clone {

    type RContext: Context;

    fn new(context: &Self::RContext) -> Result<<Self::RContext as Context>::ContextGeometry, String>;

    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    /** 
     * 获取当前的顶点个数
     */   
    fn get_vertex_count(&self) -> u32;

    /** 
     * 设置顶点的个数
     * 注：一旦设置了顶点个数，就意味着老的attribute和indiecs无效，要全部重新设置
     */
    fn set_vertex_count(&self, count: u32);

    /**
     * 设置属性数据
     * offset：该属性所在Buffer的索引，默认0
     * stride：该属性需要相隔多远才能取到下一个值，默认：0
     * count：该属性的一个元素占用Buffer的几个单位
     */
    fn set_attribute(&self, name: &AttributeName, buffer: &<Self::RContext as Context>::ContextBuffer, item_count: usize) -> Result<(), String>;

    fn set_attribute_with_offset(&self, name: &AttributeName, buffer: &<Self::RContext as Context>::ContextBuffer, item_count: usize, offset: usize, count: usize, stride: usize) -> Result<(), String>;
      
    /**
     * 删除属性
     */
    fn remove_attribute(&self, name: &AttributeName);

    /**
     * 设置索引
     * offset: 该索引从buffer的偏移量
     * count：该索引占用了buffer的多少个单位
     */
    fn set_indices_short(&self, buffer: &<Self::RContext as Context>::ContextBuffer) -> Result<(), String>;
    
    fn set_indices_short_with_offset(&self, buffer: &<Self::RContext as Context>::ContextBuffer, offset: usize, count: usize) -> Result<(), String>;

    /**
     * 删除索引
     */
    fn remove_indices(&self);
}