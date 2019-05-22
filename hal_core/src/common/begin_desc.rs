/** 
 * 开始渲染必要的数据
 * 一次渲染需要知道：渲染目标，视口，清空颜色-深度-模板
 * 
 * 用法：
 * 
 * let data = RenderBeginDesc::new();
 * data.set_***(...);
 */
pub struct RenderBeginDesc {
    pub viewport: (i32, i32, i32, i32),    // x, y, 宽, 高，左上角为原点
    pub clear_color: Option<(f32, f32, f32, f32)>, // r, g, b, a，范围：0-1，为None代表不更新颜色
    pub clear_depth: Option<f32>,   // 0-1，1代表最远，为None代表不更新深度
    pub clear_stencil: Option<u8>, // 0-255，为None代表不更新模板
}

impl RenderBeginDesc {

    /** 
     * 创建开始渲染数据
     * 默认状态：
     *    视口：渲染目标全屏
     *    清空颜色：白色
     *    清空深度：最远值，1.0
     *    不清空模板
     */
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        RenderBeginDesc {
            viewport: (x, y, width, height),
            clear_color: Some((1.0, 1.0, 1.0, 1.0)),
            clear_depth: Some(1.0),
            clear_stencil: None,
        }
    }

    /** 
     * 设置清空颜色
     * is_clear: 是否清空
     * r, g, b, a: 值在 [0, 1] 之间
     */
    pub fn set_clear_color(&mut self, is_clear: bool, r: f32, g: f32, b: f32, a: f32) {
        if is_clear {
            self.clear_color = Some((r, g, b, a));
        } else {
            self.clear_color = None;
        }
    }

    /** 
     * 设置清空深度
     * is_clear: 是否清空
     * depth：[0, 1]，1代表最远
     */
    pub fn set_clear_depth(&mut self, is_clear: bool, depth: f32) {
        self.clear_depth = if is_clear { Some(depth) } else { None };
    }

    /** 
     * 设置清空模板
     * is_clear: 是否清空
     * stencil：值在 [0, 255]
     */
    pub fn set_clear_stencil(&mut self, is_clear: bool, stencil: u8) {
        self.clear_stencil = if is_clear { Some(stencil) } else { None };
    }
}

impl AsRef<Self> for RenderBeginDesc {
    fn as_ref(&self) -> &Self {
        &self
    }
}