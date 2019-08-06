
/** 
 * 定义渲染常用的数据结构
 */

/** 
 * 着色器的类型
 */
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

/** 
 * 纹理的过滤模式
 */
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum TextureFilterMode {
    Nearest,
    Linear,
}

/** 
 * 纹理环绕模式
 * 指：当纹理坐标不在[0, 1]范围时，如何处理
 */
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum TextureWrapMode {
    Repeat,        // 重复
    ClampToEdge,   // 截取
    MirroredRepeat // 镜像重复
}

/** 
 * 像素格式
 */
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum PixelFormat {
    RGB,
    RGBA,   
    ALPHA, 
    DEPTH16,
}

/** 
 * 数据格式
 */
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum DataFormat {
    Byte,
    UnsignedByte,
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    Float,
}

/** 
 * 光栅化时的剔除状态
 */
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum CullMode {
    Back,  // 背面剔除
    Front,  // 正面剔除
}

/** 
 * 混合操作
 */
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum BlendFunc {
    Add,
    Sub,
    ReverseSub,
}

/** 
 * 混合因子
 */
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
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
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
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
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
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
 * 目标的attach类型
 */
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum RTAttachementType {
    Color0,
    Depth,
}