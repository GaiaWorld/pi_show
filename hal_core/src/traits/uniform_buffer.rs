use share::{Share};
use common::{UniformValue};
use traits::context::{Context};

/** 
 * UniformBuffer
 */
pub trait UniformBuffer {

    /** 
     * UniformBuffer的布局
     * 返回切片的索引表示高层约定uniform的槽
     */
    fn get_layout(&self) -> &[&str];

    /** 
     * 按布局顺序返回UniformValue数组
     */
    fn get_values(&self) -> &[UniformValue];

    fn set_value(&mut self, name: &str, value: &UniformValue);
}

/** 
 * Texture的Uniform和普通的Uniform要分开设置
 */
pub struct UniformTexture<RContext: Context> {
    pub texture: RContext::ContextTexture,
    pub sampler: RContext::ContextSampler,
}

/** 
 * Program的Uniform参数；
 * Texture的Uniform和普通的Uniform要分开设置
 * 布局，其中数组切片对应的下标意味着槽，避免哈希表。
 */
pub trait ProgramParamter<RContext: Context> {

    fn get_layout(&self) -> &[&str];
    fn get_texture_layout(&self) -> &[&str];

    fn get_values(&self) -> &[Share<UniformBuffer>];
    fn get_textures(&self) -> &[Share<UniformTexture<RContext>>];

    fn set_value(&mut self, name: &str, value: &Share<UniformBuffer>) -> bool;
    fn set_texture(&mut self, name: &str, value: &Share<UniformTexture<RContext>>) -> bool;

    fn get_value(&mut self, name: &str) -> Option<&Share<UniformBuffer>>;
    fn get_texture(&mut self, name: &str) -> Option<&Share<UniformTexture<RContext>>>;
}