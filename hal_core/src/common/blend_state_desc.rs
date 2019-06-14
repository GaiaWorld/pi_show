use ordered_float::{OrderedFloat};
use common::util::{BlendFunc, BlendFactor};

/** 
 * 混合状态
 * 注：src和dst因子不能同时填 常量 颜色
 */
#[derive(Debug, Clone, Hash)]
pub struct BlendStateDesc {
    pub rgb_equation: BlendFunc,
    pub alpha_equation: BlendFunc,
    
    pub src_rgb_factor: BlendFactor,
    pub dst_rgb_factor: BlendFactor,
    
    pub src_alpha_factor: BlendFactor,
    pub dst_alpha_factor: BlendFactor,

    pub const_rgba: (OrderedFloat<f32>, OrderedFloat<f32>, OrderedFloat<f32>, OrderedFloat<f32>),
}

impl BlendStateDesc {
    
    pub fn new() -> Self {
        Self {
            rgb_equation: BlendFunc::Add,
            alpha_equation: BlendFunc::Add,
            
            src_rgb_factor: BlendFactor::One,
            dst_rgb_factor: BlendFactor::Zero,
            
            src_alpha_factor: BlendFactor::One,
            dst_alpha_factor: BlendFactor::One,

            const_rgba: (OrderedFloat(1.0), OrderedFloat(1.0), OrderedFloat(1.0), OrderedFloat(1.0)),
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
        self.const_rgba = (OrderedFloat(r), OrderedFloat(g), OrderedFloat(b), OrderedFloat(a));
    }
}
