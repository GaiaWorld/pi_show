/** 
 * 定义渲染常用的枚举
 */

/** 
 * 着色器的类型
 */
pub enum ShaderType {
    Vertex,
    Fragment,
}

/** 
 * 纹理的过滤模式
 */
pub enum TextureFilterMode {
    Nearest,
    Linear,
}

/** 
 * 纹理环绕模式
 * 指：当纹理坐标不在[0, 1]范围时，如何处理
 */
pub enum TextureWrapMode {
    Repeat,        // 重复
    ClampToEdge,   // 截取
    MirroredRepeat // 镜像重复
}

/** 
 * 像素格式
 */
pub enum PixelFormat {
    RGB,
    RGBA,   
    ALPHA, 
}

/** 
 * 数据格式
 */
pub enum DataFormat {
    Byte,
    UnsignedByte,
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    Float,
    Double,
}

/** 
 * 光栅化时的剔除状态
 */
#[derive(PartialEq, Clone, Copy)]
pub enum CullMode {
    None,  // 不剔除
    Back,  // 背面剔除
    Front,  // 正面剔除
}

/** 
 * 混合操作
 */
pub enum BlendFunc {
    Add,
    Sub,
    ReverseSub,
}

/** 
 * 混合因子
 */
pub enum BlendFactor {
    Zero,
    One,
    
    SrcColor,
    OneMinusSrcColor,
    
    DstColor,
    OneMinusDstColor,

    SrcAlpha,
    OneMinusSrcAlpha,

    DstAlpha,
    OneMinusDstAlpha,

    ConstantColor,
    OneMinusConstantColor,

    ConstantAlpha,
    OneMinusConstantAlpha,
}

/** 
 * 深度和模板的比较函数
 */
pub enum CompareFunc {
    Never,
    Always,
    Less,
    Equal,
    LEqual,
    Greater,
    GEqual,
    NotEqual,
}

/** 
 * 模板操作
 */
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    Incr,
    Decr,
    Invert,
    IncrWrap,
    DecrWrap, 
}

/**
 * 渲染目标的管道
 */
pub enum RTAttachment {
    Color0, // 第一个颜色缓冲区
    Depth,  // 深度缓冲区
    Stencil, // 模板缓冲区
}