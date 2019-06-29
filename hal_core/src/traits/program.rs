use atom::{Atom};
use common::{ShaderType};
use traits::context::{Context};


/**
 * Uniform布局
 */
pub struct UniformLayout<'a> {
    pub ubos: &'a [Atom],
    pub uniforms: &'a [&'a [Atom]], 
    pub textures: &'a [Atom],
}

pub trait Program : Sized + Clone {
    type RContext: Context;

    /** 
     * 方便的构造函数，根据vs，fs创建对应的Program
     * ubo_layouts: 该Program的UBO的布局约定，索引就是该Atom的槽
     * uniforms_layouts: 该Program的Uniform的布局约定，里面索引就是该Atom的槽
     * 注：compile，link内部有缓存表，已经编译过的shader和program不会再次编译
     */
    fn new_with_vs_fs(context: &Self::RContext, vs_name: &Atom, vs_defines: &[Atom], fs_name: &Atom, fs_defines: &[Atom], uniform_layout: &UniformLayout) -> Result<<Self::RContext as Context>::ContextProgram, String>;

    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    /** 
     * 返回指定类型的shader的名字和宏
     */
    fn get_shader_info(&self, stype: ShaderType) -> Option<(&Atom, &[Atom])>;
}