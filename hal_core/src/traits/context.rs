use std::sync::{Arc};
use std::collections::{HashMap};
use atom::Atom;
use std::convert::{AsRef};

use common::{Uniforms, ShaderType, Capabilities, Pipeline, RenderBeginDesc, PixelFormat, DataFormat, RasterState, DepthState, StencilState, BlendState};
use traits::texture::{Texture};
use traits::geometry::{Geometry};
use traits::sampler::{Sampler, SamplerDesc};
use traits::render_target::{RenderTarget, RenderBuffer};
use ShareRef;

/**
 * 渲染上下文，负责如下功能
 * 
 * 1. 创建资源
 * 2. 设置状态
 * 3. 渲染物体
 * 
 * 调用顺序如下：
 * 
 * let mut (context) = Context::new(gl);
 * 
 * // 初始化 shader
 * 
 * context.set_shader_code("hello_vs", "代码");
 * context.set_shader_code("hello_fs", "代码");
 
 * let vs = context.compile_shader(ShaderType.VS, "hello_vs", ["STROKE", "CLIP"]);
 * let fs = context.compile_shader(ShaderType.FS, "hello_fs", ["STROKE", "CLIP"]);
 * 
 * // 创建资源
 * let geometry = context.create_geometry(vertex_count).unwrap();
 * let texture = context.create_texture_2d(...).unwrap();
 * let sampler = context.create_sampler(...).unwrap();
 * 
 * let ss = RasterState::new().set_**();
 * let bs = BlendState::new().set_**();
 * let ss = StencilState::new().set_**();
 * let ds = DepthState::new().set_**();
 * let pipeline = context.create_pipeline(vs, fs, rs, bs, ss, ds);
 *
 * let u1 = Uniforms::new().set("world", ....).set("view", ...);
 * let u2 = Uniforms::new().set("abc", ....);
 * let u3 = Uniforms::new().set("def", ....);
 *
 * // 渲染循环
 * context_begin(渲染目标, 视口, 清空处理);
 *
 * context.set_pipeline(pipeline_1);
 * context.draw(geometry_1, [u1, u2, u3]);
 * context.draw(geometry_2, [u4, u2, u3]);
 * 
 * context.set_pipeline(pipeline_2);
 * context.draw(geometry_3, [u1, u2, u3]);
 * context.draw(geometry_4, [u4, u2, u3]);
 *
 * context_end();
 */

pub trait Context {
    type ContextSelf: Context;
    type ContextGeometry: Geometry;
    type ContextTexture: Texture;
    type ContextSampler: Sampler;
    type ContextRenderTarget: RenderTarget;
    type ContextRenderBuffer: RenderBuffer;

    /**
     * 取特性
     */
    fn get_caps(&self) -> Arc<Capabilities>;

    /**
     * 取默认的渲染目标
     */
    fn get_default_render_target(&self) -> Arc<Self::ContextRenderTarget>;

    /** 
     * 设置shader代码
     */
    fn set_shader_code<C: AsRef<str>>(&mut self, name: &Atom, code: &C);

    /**
     * 编译shader，返回shader对应的hash
     * Shader相关接口
     * 策略：底层握住所有的Shader句柄，不会释放
     * 注：Shader编译耗时，最好事先 编译 和 链接
     */
    fn compile_shader(&mut self, shader_type: ShaderType, name: &Atom, defines: &[Atom]) -> Result<u64, String>;

    /** 
     * 创建渲染管线
     */
    fn create_pipeline(&mut self, vs_hash: u64, fs_hash: u64, rs: ShareRef<RasterState>, bs: ShareRef<BlendState>, ss: ShareRef<StencilState>, ds: ShareRef<DepthState>) -> Result<Pipeline, String>;

    /** 
     * 创建Uniforms
     */
    fn create_uniforms(&mut self) -> Uniforms<Self> where Self: std::marker::Sized;

    /** 
     * 创建几何数据
     */
    fn create_geometry(&self) -> Result<Self::ContextGeometry, String>;

    /** 
     * 创建2D纹理
     * width: 宽
     * height: 高
     * format: 格式
     * is_gen_mipmap: 是否生成mipmap
     */
    fn create_texture_2d(&mut self, width: u32, height: u32, pformat: PixelFormat, dformat: DataFormat, is_gen_mipmap: bool, data: Option<&[u8]>) -> Result<Self::ContextTexture, String>;

    /** 
     * 用canvas创建2D纹理，尽用于webgl版本
     * width: 宽
     * height: 高
     * format: 格式
     * is_gen_mipmap: 是否生成mipmap
     */
    fn create_texture_2d_with_canvas(&mut self, width: u32, height: u32, pixel: PixelFormat, data: DataFormat, is_gen_mipmap: bool, canvas: *const isize) -> Result<Self::ContextTexture, String>;

    /** 
     * 创建采样器
     */
    fn create_sampler(&mut self, desc: ShareRef<SamplerDesc>) -> Result<Self::ContextSampler, String>;

    /** 
     * 创建渲染目标
     */
    fn create_render_target(&mut self) -> Result<Self::ContextRenderTarget, String>;

    /** 
     * 创建渲染缓冲区
     */
    fn create_render_buffer(&mut self, w: u32, h: u32, format: PixelFormat) -> Result<Self::ContextRenderBuffer, String>;
 
    /** 
     * 开始渲染：一次渲染指定一个 渲染目标，视口区域，清空策略
     * 注：所有的set_**和draw方法都要在begin_render和end_render之间调用，否则无效
     */
    fn begin_render(&mut self, render_target: &ShareRef<Self::ContextRenderTarget>, data: &ShareRef<RenderBeginDesc>);

    /** 
     * 结束渲染
     * 注：所有的set_**和draw方法都要在begin_render和end_render之间调用，否则无效
     */
    fn end_render(&mut self);

    /** 
     * 设置渲染管线
     * 注：该方法都要在begin_render和end_render之间调用，否则无效
     */
    fn set_pipeline(&mut self, pipeline: &ShareRef<Pipeline>);

    /** 
     * 渲染物体
     * 注：该方法都要在begin_render和end_render之间调用，否则无效
     */
    fn draw(&mut self, geometry: &ShareRef<Self::ContextGeometry>, values: &HashMap<Atom, ShareRef<Uniforms<Self>>>) where Self: std::marker::Sized;
}