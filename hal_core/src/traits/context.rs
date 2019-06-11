use atom::Atom;

use fnv::FnvHashMap;

use common::{Uniforms, Capabilities, RenderBeginDesc};
use traits::geometry::{Geometry};
use traits::program::{Program};
use traits::render_target::{RenderTarget, RenderBuffer};
use traits::sampler::{Sampler};
use traits::state::{RasterState, DepthState, StencilState, BlendState};
use traits::texture::{Texture};

/**
 * 渲染上下文，负责如下功能
 * 
 * 1. 设置状态
 * 2. 渲染物体
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
    fn get_default_target(&self) -> &Self::ContextRenderTarget;

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