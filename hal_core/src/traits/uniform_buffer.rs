use share::{Share};
use common::{UniformValue};
use traits::context::{HalTexture, HalSampler};

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

    fn get_value(&self, name: &str) -> Option<&UniformValue>;

    fn set_value(&mut self, name: &str, value: UniformValue) -> bool;
}

pub struct NullUniformBuffer;

impl UniformBuffer for NullUniformBuffer {
    #[inline]
    fn get_layout(&self) -> &[&str] {
        &[]
    }

    #[inline]
    fn get_values(&self) -> &[UniformValue] {
        &[]
    }

    fn get_value(&self, _name: &str) -> Option<&UniformValue> {
        None
    }

    fn set_value(&mut self, _name: &str, _value: UniformValue) -> bool {
        false
    }
}

/** 
 * Program的Uniform参数；
 * Texture的Uniform和普通的Uniform要分开设置
 * 布局，其中数组切片对应的下标意味着槽，避免哈希表。
 */
pub trait ProgramParamter {

    fn get_layout(&self) -> &[&str];
    fn get_texture_layout(&self) -> &[&str];

    fn get_values(&self) -> &[Share<dyn UniformBuffer>];
    fn get_textures(&self) -> &[(Share<HalTexture>, Share<HalSampler>)];

    fn set_value(&self, name: &str, value: Share<dyn UniformBuffer>) -> bool;
    fn set_texture(&self, name: &str, value: (Share<HalTexture>, Share<HalSampler>)) -> bool;

    fn get_value(&self, name: &str) -> Option<&Share<dyn UniformBuffer>>;
    fn get_texture(&self, name: &str) -> Option<&(Share<HalTexture>, Share<HalSampler>)>;
}

pub trait Defines {
    fn add(&mut self, value: &'static str) -> Option<&'static str>;
    fn remove(&mut self, value: &'static str) -> Option<&'static str>;
    fn list(&self) -> &[Option<&str>];
    fn id(&self) -> u32;
}