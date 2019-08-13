use common::util::{CompareFunc};

/** 
 * 深度状态
 */
#[derive(Debug, Clone, Hash)]
pub struct DepthStateDesc {
    pub is_depth_test_enable: bool,
    pub is_depth_write_enable: bool,
    pub depth_test_func: CompareFunc,
}

impl Default for DepthStateDesc {
    fn default() -> Self {
        Self::new()
    }
}

impl DepthStateDesc {
    pub fn new() -> Self {
        Self {
            is_depth_test_enable: true,
            is_depth_write_enable: true,
            depth_test_func: CompareFunc::LEqual,
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
