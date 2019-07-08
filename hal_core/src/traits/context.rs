use atom::{Atom};
use share::{Share};

use common::*;
use traits::uniform_buffer::{ProgramParamter};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HalBuffer(pub u32, pub u32);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HalGeometry(pub u32, pub u32);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HalProgram(pub u32, pub u32);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HalRenderTarget(pub u32, pub u32);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HalRenderBuffer(pub u32, pub u32);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HalRasterState(pub u32, pub u32);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HalDepthState(pub u32, pub u32);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HalStencilState(pub u32, pub u32);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HalBlendState(pub u32, pub u32);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HalSampler(pub u32, pub u32);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HalTexture(pub u32, pub u32);

#[derive(PartialEq, Clone, Copy, Debug, Hash)]
pub enum BufferType {
    Attribute,
    Indices,
}

pub enum BufferData<'a> {
    Float(&'a[f32]),
    Short(&'a[u16]),
}

/**
 * Uniform布局
 */
pub struct UniformLayout<'a> {
    pub ubos: &'a [Atom],
    pub uniforms: &'a [&'a [Atom]], 
    pub textures: &'a [Atom],
}

pub trait CustomTextureData {
    fn update(&self, texture: &HalTexture);
}

pub enum TextureData<'a> {
    F32(u32, u32, u32, u32, &'a[f32]),  // (x, y, w, h, data)
    U8(u32, u32, u32, u32, &'a[u8]),   // (x, y, w, h, data)
    Custom(Box<dyn CustomTextureData>),
}

pub trait HalContext {

    // ==================== HalBuffer
    
    /** 
     * count: data中有多少个具体的元素项(float/short/int)
     * is_updatable表示是否需要更新，根据这个来让显卡决定将该buffer放到不同的地方，以便显卡优化性能。
     * 
     * 注：Attribute --> float
     * 注：Indeices --> short
     */
    fn buffer_create(&self, btype: BufferType, count: usize, data: Option<BufferData>, is_updatable: bool) -> Result<HalBuffer, String>;

    fn buffer_destroy(&self, buffer: &HalBuffer);

    /**
     * 更新数据
     * 
     * 注：Attribute --> float
     * 注：Indeices --> short
     * 
     * offset：单位是BufferData对应的类型单位。
     *    如果BufferData是Float，那么offet的单位就是1个float
     * 注：如果一开始就要更新数据，那么new的时候，尽量使用 is_updatable = true 来创建buffer。
     * 注：偏移 + data的长度 <= 创建时候的大小
     */
    fn buffer_update(&self, buffer: &HalBuffer, offset: usize, data: BufferData);
    
    
    // ==================== HalGeometry

    fn geometry_create(&self) -> Result<HalGeometry, String>;

    fn geometry_destroy(&self, geometry: &HalGeometry);

    /** 
     * 获取当前的顶点个数
     */   
    fn geometry_get_vertex_count(&self, geometry: &HalGeometry) -> u32;

    /** 
     * 设置顶点的个数
     * 注：一旦设置了顶点个数，就意味着老的attribute和indiecs无效，要全部重新设置
     */
    fn geometry_set_vertex_count(&self, geometry: &HalGeometry, count: u32);

    /**
     * 设置属性数据
     * offset：该属性所在Buffer的索引，默认0
     * stride：该属性需要相隔多远才能取到下一个值，默认：0
     * count：该属性的一个元素占用Buffer的几个单位
     */
    fn geometry_set_attribute(&self, geometry: &HalGeometry, name: &AttributeName, buffer: &HalBuffer, item_count: usize) -> Result<(), String>;

    fn geometry_set_attribute_with_offset(&self, geometry: &HalGeometry, name: &AttributeName, buffer: &HalBuffer, item_count: usize, offset: usize, count: usize, stride: usize) -> Result<(), String>;
      
    /**
     * 删除属性
     */
    fn geometry_remove_attribute(&self, geometry: &HalGeometry, name: &AttributeName);

    /**
     * 设置索引
     * offset: 该索引从buffer的偏移量
     * count：该索引占用了buffer的多少个单位
     */
    fn geometry_set_indices_short(&self, geometry: &HalGeometry, buffer: &HalBuffer) -> Result<(), String>;
    
    fn geometry_set_indices_short_with_offset(&self, geometry: &HalGeometry, buffer: &HalBuffer, offset: usize, count: usize) -> Result<(), String>;

    /**
     * 删除索引
     */
    fn geometry_remove_indices(&self, geometry: &HalGeometry);



    // ==================== HalProgram

    /** 
     * 方便的构造函数，根据vs，fs创建对应的Program
     * ubo_layouts: 该Program的UBO的布局约定，索引就是该str
     * uniforms_layouts: 该Program的Uniform的布局约定，里面索引就是该str的槽
     * 注：compile，link内部有缓存表，已经编译过的shader和program不会再次编译
     */
    fn program_create_with_vs_fs(&self, vs_id: u64, fs_id: u64, vs_name: &str, vs_defines: &[Option<&str>], fs_name: &str, fs_defines: &[Option<&str>], uniform_layout: &UniformLayout) -> Result<HalProgram, String>;

    fn program_destroy(&self, program: &HalProgram);

    /** 
     * 返回指定类型的shader的名字和宏
     */
    fn program_get_shader_info(&self, program: &HalProgram, stype: ShaderType) -> Option<(&Atom, &[Atom])>;


    // ==================== HalRenderTarget

    fn rt_create(&self, w: u32, h: u32, pformat: PixelFormat, dformat: DataFormat, has_depth: bool) -> Result<HalRenderTarget, String>;
    
    fn rt_destroy(&self, rt: &HalRenderTarget);

    /** 
     * 取大小
     */
    fn rt_get_size(&self, rt: &HalRenderTarget) -> Option<(u32, u32)>;

    /**
     * 取渲染目标中特定通道的纹理
     */
    fn rt_get_color_texture(&self, rt: &HalRenderTarget, index: u32) -> Option<HalTexture>;

    // ==================== HalRenderBuffer
    fn rb_create(&self, w: u32, h: u32, pformat: PixelFormat) -> Result<HalRenderBuffer, String>;
    
    fn rb_destroy(&self, rb: &HalRenderBuffer);

    fn rb_get_size(&self, rb: &HalRenderBuffer) -> Option<(u32, u32)>;


    // ==================== HalTexture

    fn texture_create_2d(&self, mipmap_level: u32, width: u32, height: u32, pformat: PixelFormat, dformat: DataFormat, is_gen_mipmap: bool, data: Option<TextureData>) -> Result<HalTexture, String>;

    fn texture_destroy(&self, texture: &HalTexture);

    fn texture_get_size(&self, texture: &HalTexture) -> Option<(u32, u32)>;

    fn texture_get_render_format(&self, texture: &HalTexture) -> Option<PixelFormat>;

    fn texture_is_gen_mipmap(&self, texture: &HalTexture) -> bool;

    fn texture_update(&self, texture: &HalTexture, mipmap_level: u32, data: &TextureData);

    // ==================== HalSampler

    fn sampler_create(&self, desc: &SamplerDesc) -> Result<HalSampler, String>;

    fn sampler_destroy(&self, sampler: &HalSampler);

    fn sampler_get_desc(&self, sampler: &HalSampler) -> Option<&SamplerDesc>;

    // ==================== HalRasterState

    fn rs_create(&self, desc: &RasterStateDesc) -> Result<HalRasterState, String>;
    
    fn rs_destroy(&self, state: &HalRasterState);

    fn rs_get_desc(&self, state: &HalRasterState) -> Option<&RasterStateDesc>;

    // ==================== HalDepthState

    fn ds_create(&self, desc: &DepthStateDesc) -> Result<HalDepthState, String>;
    
    fn ds_destroy(&self, state: &HalDepthState);

    fn ds_get_desc(&self, state: &HalDepthState) -> Option<&DepthStateDesc>;

    // ==================== HalStencilState

    fn ss_create(&self, desc: &StencilStateDesc) -> Result<HalStencilState, String>;
    
    fn ss_destroy(&self, state: &HalStencilState);

    fn ss_get_desc(&self, state: &HalStencilState) -> Option<&StencilStateDesc>;

    // ==================== HalBlendState
    
    fn bs_create(&self, desc: &BlendStateDesc) -> Result<HalBlendState, String>;
    
    fn bs_destroy(&self, state: &HalBlendState);

    fn bs_get_desc(&self, state: &HalBlendState) -> Option<&BlendStateDesc>;

    // ==================== 上下文相关

    /**
     * 取特性
     */
    fn render_get_caps(&self) -> &Capabilities;

    /**
     * 取默认的渲染目标
     */
    fn render_get_default_target(&self) -> &HalRenderTarget;

    /** 
     * 设置shader代码
     */
    fn render_set_shader_code<C: AsRef<str>>(&self, name: &str, code: &C);

    /** 
     * 开始渲染：一次渲染指定一个 渲染目标，视口区域，清空策略
     * 注：begin-end之间，只能调用下面的几个方法，不能再调用任何创建和更新方法。
     * 注：所有的set_**和draw方法都要在begin_render和end_render之间调用，否则无效
     */
    fn render_begin(&self, render_target: &HalRenderTarget, data: &RenderBeginDesc);

    /** 
     * 结束渲染
     * 注：所有的set_**和draw方法都要在begin_render和end_render之间调用，否则无效
     */
    fn render_end(&self);

    /** 
     * 设置Program
     * 注：该方法都要在begin_render和end_render之间调用，否则无效
     */
    fn render_set_program(&self, program: &HalProgram);

    /** 
     * 设置State
     * 注：该方法都要在begin_render和end_render之间调用，否则无效
     */
    fn render_set_state(&self, bs: &HalBlendState, ds: &HalDepthState, rs: &HalRasterState, ss: &HalStencilState);

    /** 
     * 渲染物体
     * 注：该方法都要在begin_render和end_render之间调用，否则无效
     */
    fn render_draw(&self, geometry: &HalGeometry, parameter: &Share<dyn ProgramParamter>);
}