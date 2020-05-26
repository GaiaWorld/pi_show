use common::util::{CompareFunc, StencilOp};

/**
 * 模板状态
 */
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Hash)]
pub struct StencilStateDesc {
    pub is_stencil_test_enable: bool,

    pub stencil_test_func: CompareFunc,
    pub stencil_ref: i32,
    pub stencil_mask: u32,

    pub stencil_fail_op: StencilOp,
    pub stencil_zfail_op: StencilOp,
    pub stencil_zpass_op: StencilOp,
}

impl Default for StencilStateDesc {
    fn default() -> Self {
        Self::new()
    }
}

impl StencilStateDesc {
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
    pub fn set_func(&mut self, func: CompareFunc, sref: i32, mask: u32) {
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
