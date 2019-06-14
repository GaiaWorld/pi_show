use ordered_float::{OrderedFloat};
use common::util::{CullMode};

/** 
 * 光栅化状态
 */
#[derive(Debug, Clone, Hash)]
pub struct RasterStateDesc {
    pub cull_mode: Option<CullMode>,   // 默认：None
    pub is_front_face_ccw: bool,       // 默认：true
    pub polygon_offset: (OrderedFloat<f32>, OrderedFloat<f32>),    // 成员分别是factor和units，默认：(0, 0)
}

impl RasterStateDesc {
    
    pub fn new() -> Self {
        Self {
            cull_mode: None, // Some(CullMode::Back),
            is_front_face_ccw: true,
            polygon_offset: (OrderedFloat(0.0), OrderedFloat(0.0)),
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
        self.polygon_offset = (OrderedFloat(factor), OrderedFloat(unit));
    }
}
