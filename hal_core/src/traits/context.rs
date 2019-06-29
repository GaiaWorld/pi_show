use atom::Atom;
use share::Share;

use common::{Capabilities, RenderBeginDesc};
use traits::buffer::{Buffer};
use traits::geometry::{Geometry};
use traits::program::{Program};
use traits::render_target::{RenderTarget, RenderBuffer};
use traits::sampler::{Sampler};
use traits::state::{RasterState, DepthState, StencilState, BlendState};
use traits::texture::{Texture};
use traits::uniform_buffer::{ProgramParamter};

/**
 * 渲染上下文，负责如下功能
 * 
 * 1. 设置状态
 * 2. 渲染物体
 */

pub trait Context: Sized + Clone {
    type ContextSelf: Context;
    
    type ContextBuffer: Buffer<RContext = Self>;
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
    fn get_default_target(&self) -> &Self::ContextRenderTarget;

    /** 
     * 设置shader代码
     */
    fn set_shader_code<C: AsRef<str>>(&self, name: &Atom, code: &C);

    /**
     * 将渲染库底层的状态还原成状态机的状态
     * 目的：因为我们会和别的渲染引擎使用同一个底层渲染库，每个引擎的状态机，会导致底层状态机不一致，所以要有这个方法。
     * 保证一帧开始调用begin之前调用一次。
     */
    fn restore_state(&mut self);

    /** 
     * 开始渲染：一次渲染指定一个 渲染目标，视口区域，清空策略
     * 注：begin-end之间，只能调用下面的几个方法，不能再调用任何创建和更新方法。
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
    fn draw(&self, geometry: &Self::ContextGeometry, parameter: &Share<ProgramParamter<Self::ContextSelf>>);
}