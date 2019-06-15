use atom::{Atom};
use common::{ShaderType};
use traits::context::{Context};

pub trait Program : Sized + Clone {
    type RContext: Context;

    fn new(context: &Self::RContext) -> Result<<Self::RContext as Context>::ContextProgram, String>;
    
    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    /** 
     * 方便的构造函数，根据vs，fs创建对应的Program
     * 注：compile，link内部有缓存表，已经编译过的shader和program不会再次编译
     */
    fn new_with_vs_fs(context: &Self::RContext, vs_name: &Atom, vs_defines: &[Atom], fs_name: &Atom, fs_defines: &[Atom]) -> Result<<Self::RContext as Context>::ContextProgram, String>;

    /** 
     * 返回指定类型的shader的名字和宏
     */
    fn get_shader_info(&self, stype: ShaderType) -> Option<(&Atom, &[Atom])>;
}