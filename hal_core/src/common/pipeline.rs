use std::sync::{Arc};
use common::{CullMode, CompareFunc, BlendFunc, BlendFactor, StencilOp};

/** 
 * 渲染管线
 */
pub struct Pipeline {
    pub vs_hash: u64,
    pub fs_hash: u64,

    pub raster_state: Option<Arc<RasterState>>,
    pub depth_state: Option<Arc<DepthState>>,
    pub stencil_state: Option<Arc<StencilState>>,
    pub blend_state: Option<Arc<BlendState>>,
}

/** 
 * 光栅化状态
 */
pub struct RasterState {
    pub cull_mode: Option<CullMode>,   // 默认：None
    pub is_front_face_ccw: bool,       // 默认：true
    pub polygon_offset: (f32, f32),    // 成员分别是factor和units，默认：(0, 0)
}

/** 
 * 深度状态
 */
pub struct DepthState {
    pub is_depth_test_enable: bool,
    pub is_depth_write_enable: bool,
    pub depth_test_func: CompareFunc,
}

/** 
 * 模板状态
 */
pub struct StencilState {
    pub is_stencil_test_enable: bool,
    
    pub stencil_test_func: CompareFunc,
    pub stencil_ref: i32,
    pub stencil_mask: i32,
    
    pub stencil_fail_op: StencilOp,
    pub stencil_zfail_op: StencilOp,
    pub stencil_zpass_op: StencilOp,
}

/** 
 * 混合状态
 * 注：src和dst因子不能同时填 常量 颜色
 */
pub struct BlendState {
    pub rgb_equation: BlendFunc,
    pub alpha_equation: BlendFunc,
    
    pub src_rgb_factor: BlendFactor,
    pub dst_rgb_factor: BlendFactor,
    
    pub src_alpha_factor: BlendFactor,
    pub dst_alpha_factor: BlendFactor,

    pub const_rgba: (f32, f32, f32, f32),
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            vs_hash: 0,
            fs_hash: 0,

            raster_state: None,
            depth_state: None,
            stencil_state: None,
            blend_state: None,
        }
    }
}

/** 
 * RasterState的操作
 * 
 * let state = RasterState::new();
 * state.set_***();
 * state.set_***();
 */
impl RasterState {
    
    pub fn new() -> Self {
        Self {
            cull_mode: None,
            is_front_face_ccw: true,
            polygon_offset: (0.0, 0.0),
        }
    }

    /** 
     * 设置光栅化时剔除的面
     * 默认：不剔除
     */
    pub fn set_cull_mode(&mut self, cull_mode: Option<CullMode>) {
        self.cull_mode = cull_mode;
    }

    /** 
     * 设置 正面是否是逆时针
     * 默认：是逆时针
     */
    pub fn set_front_face_ccw(&mut self, is_front_face_ccw: bool) {
        self.is_front_face_ccw = is_front_face_ccw;
    }
    
    /** 
     * 设置 多边形深度偏移
     * 默认：不会做任何偏移
     */
    pub fn set_polygon_offset(&mut self, factor: f32, unit: f32) {
        self.polygon_offset = (factor, unit);
    }
}

/** 
 * 创建 BlendState的操作
 * 
 * let state = BlendState::new();
 * state.set_***();
 * state.set_***();
 */
impl BlendState {
    
    pub fn new() -> Self {
        Self {
            rgb_equation: BlendFunc::Add,
            alpha_equation: BlendFunc::Add,
            
            src_rgb_factor: BlendFactor::One,
            dst_rgb_factor: BlendFactor::Zero,
            
            src_alpha_factor: BlendFactor::One,
            dst_alpha_factor: BlendFactor::Zero,

            const_rgba: (1.0, 1.0, 1.0, 1.0),
        }
    }

    /** 
     * 设置rgb的混合操作
     * src_factor * src.rgb op dst_factor * dst.rgb
     * 默认：加法
     */
    pub fn set_rgb_equation(&mut self, func: BlendFunc) {
        self.rgb_equation = func;
    }

    /** 
     * 设置alpha的混合操作
     * src_factor * src.alpha op dst_factor * dst.alpha
     * 默认：加法
     */
    pub fn set_alpha_equation(&mut self, func: BlendFunc) {
        self.alpha_equation = func;
    }

    /** 
     * 设置rgb的因子
     * 默认：src_factor是1，dst_factor是0
     */
    pub fn set_rgb_factor(&mut self, src: BlendFactor, dst: BlendFactor) {
        self.src_rgb_factor = src;
        self.dst_rgb_factor = dst;
    }

    /** 
     * 设置alpha的因子
     * 默认：src_factor是1，dst_factor是0
     */
    pub fn set_alpha_factor(&mut self, src: BlendFactor, dst: BlendFactor) {
        self.src_alpha_factor = src;
        self.dst_alpha_factor = dst;
    }

    /** 
     * 设置常量
     * 默认是白色
     */
    pub fn set_const_rgba(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.const_rgba = (r, g, b, a);
    }
}

/** 
 * 创建 DepthState的操作
 * 
 * let state = DepthState::new();
 * state.set_***();
 * state.set_***();
 */
impl DepthState {
    pub fn new() -> Self {
        Self {
            is_depth_test_enable: true,
            is_depth_write_enable: true,
            depth_test_func: CompareFunc::Less,
        }
    }

    /** 
     * 开启深度检测
     * 默认：开启
     */
    pub fn set_test_enable(&mut self, is_enable: bool) {
        self.is_depth_test_enable = is_enable;
    }

    /** 
     * 开启写深度
     * 默认：开启
     */
    pub fn set_write_enable(&mut self, is_enable: bool) {
        self.is_depth_write_enable = is_enable;
    }

    /** 
     * 深度检测函数
     * 离相机更近的物体，深度值越小！
     * 默认：src.z < 深度缓冲区的值，通过测试
     */
    pub fn set_test_func(&mut self, func: CompareFunc) {
        self.depth_test_func = func;
    }
}

/** 
 * 创建 StencilState的操作
 * 
 * let state = StencilState::new();
 * state.set_***();
 * state.set_***();
 */
impl StencilState {

    pub fn new() -> Self {
        Self {
            is_stencil_test_enable: false,
            stencil_test_func: CompareFunc::Never,
            stencil_ref: 0,
            stencil_mask: 0,
            stencil_fail_op: StencilOp::Keep,
            stencil_zfail_op: StencilOp::Keep,
            stencil_zpass_op: StencilOp::Keep,
        }
    }

    /** 
     * 开启模板测试
     * 默认：关闭
     */
    pub fn set_enable(&mut self, enable: bool) {
        self.is_stencil_test_enable = enable;
    }

    /** 
     * 设置模板测试函数
     * 默认：永远不通过，ref和mask都是0
     */
    pub fn set_func(&mut self, func: CompareFunc, sref: i32, mask: i32) {
        self.stencil_test_func = func;
        self.stencil_ref = sref;
        self.stencil_mask = mask;
    }


    /** 
     * 设置模板测试之后的操作
     * 默认：不管哪种操作，都是保留原值。
     */
    pub fn set_op(&mut self, sfail: StencilOp, zfail: StencilOp, zpass: StencilOp) {
        self.stencil_fail_op = sfail;
        self.stencil_zfail_op = zfail;
        self.stencil_zpass_op = zpass;
    }
}