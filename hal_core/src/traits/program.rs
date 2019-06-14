use std::sync::{Arc};
use atom::{Atom};
use common::{ShaderType};
use traits::context::{Context};

pub trait Program {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>) -> Result<<Self::RContext as Context>::ContextProgram, String>;
    
    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    /** 
     * 方便的构造函数，根据vs，fs创建对应的Program
     * 注：compile，link内部有缓存表，已经编译过的shader和program不会再次编译
     */
    fn new_with_vs_fs(context: &Arc<Self::RContext>, vs_name: &Atom, vs_defines: &[Atom], fs_name: &Atom, fs_defines: &[Atom]) -> Result<<Self::RContext as Context>::ContextProgram, String>;

    /** 
     * 添加shader
     * 注：调用link之后，不能再次调用这个函数，否则返回错误
     */
    fn attach_shader(&self, shader_hash: u64) -> Result<(), String>;
    
    /** 
     * 链接program
     */
    fn link(&self);

    /** 
     * 返回指定类型的shader的名字和宏
     */
    fn get_shader_info(&self, stype: ShaderType) -> Option<(&Atom, &[Atom])>;

    /** 
     * 设置shader代码
     */
    fn set_shader_code<C: AsRef<str>>(conext: &Self::RContext, name: &Atom, code: &C);

    /**
     * 编译shader，返回shader对应的hash
     * Shader相关接口
     * 策略：底层握住所有的Shader句柄，不会释放
     * 注：Shader编译耗时，最好事先 编译 和 链接
     */
    fn compile_shader(context: &Self::RContext, shader_type: ShaderType, name: &Atom, defines: &[Atom]) -> Result<u64, String>;
}