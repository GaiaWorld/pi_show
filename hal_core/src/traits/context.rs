use atom::Atom;

use fnv::FnvHashMap;

use common::{Uniforms, ShaderType, Capabilities, RenderBeginDesc};
use traits::geometry::{Geometry};
use traits::program::{Program};
use traits::render_target::{RenderTarget, RenderBuffer};
use traits::sampler::{Sampler};
use traits::state::{RasterState, DepthState, StencilState, BlendState};
use traits::texture::{Texture};

/**
 * 渲染上下文，负责如下功能
 * 
 * 1. 创建资源
 * 2. 设置状态
 * 3. 渲染物体
 */

pub trait Context: Sized {
    type ContextSelf: Context;
    type ContextGeometry: Geometry<RContext = Self>;
    type ContextTexture: Texture<RContext = Self>;
    type ContextSampler: Sampler<RContext = Self>;
    type ContextRenderTarget: RenderTarget<RContext = Self>;
    type ContextRenderBuffer: RenderBuffer<RContext = Self>;
    type ContextBlendState: BlendState<RContext = Self>;
    type ContextDepthState: DepthState<RContext = Self>;
    type ContextRasterState: RasterState<RContext = Self>;
    type ContextStencilState: StencilState<RContext = Self>;
    type ContextProgram: Program<RContext = Self>;

    /**
     * 取特性
     */
    fn get_caps(&self) -> &Capabilities;

    /**
     * 取默认的渲染目标
     */
    fn get_default_render_target(&self) -> &Self::ContextRenderTarget;

    /** 
     * 设置shader代码
     */
    fn set_shader_code<C: AsRef<str>>(&self, name: &Atom, code: &C);

    /**
     * 编译shader，返回shader对应的hash
     * Shader相关接口
     * 策略：底层握住所有的Shader句柄，不会释放
     * 注：Shader编译耗时，最好事先 编译 和 链接
     */
    fn compile_shader(&self, shader_type: ShaderType, name: &Atom, defines: &[Atom]) -> Result<u64, String>;

    /** 
     * 开始渲染：一次渲染指定一个 渲染目标，视口区域，清空策略
     * 注：所有的set_**和draw方法都要在begin_render和end_render之间调用，否则无效
     */
    fn begin_render(&self, render_target: &Self::ContextRenderTarget, data: &RenderBeginDesc);

    /** 
     * 结束渲染
     * 注：所有的set_**和draw方法都要在begin_render和end_render之间调用，否则无效
     */
    fn end_render(&self);

    /** 
     * 设置Program
     * 注：该方法都要在begin_render和end_render之间调用，否则无效
     */
    fn set_program(&self, program: &Self::ContextProgram);

    /** 
     * 设置State
     * 注：该方法都要在begin_render和end_render之间调用，否则无效
     */
    fn set_state(&self, bs: &Self::ContextBlendState, ds: &Self::ContextDepthState, rs: &Self::ContextRasterState, ss: &Self::ContextStencilState);

    /** 
     * 渲染物体
     * 注：该方法都要在begin_render和end_render之间调用，否则无效
     */
    fn draw(&self, geometry: &Self::ContextGeometry, values: &FnvHashMap<Atom, Uniforms>, samplers: &[Self::ContextSampler]);
}