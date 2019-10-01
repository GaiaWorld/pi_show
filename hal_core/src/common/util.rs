
/** 
 * 定义渲染常用的数据结构
 */

/** 
 * 着色器的类型
 */
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

/** 
 * 纹理的过滤模式
 */
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum TextureFilterMode {
    Nearest,
    Linear,
}

/** 
 * 纹理环绕模式
 * 指：当纹理坐标不在[0, 1]范围时，如何处理
 */
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum TextureWrapMode {
    Repeat,        // 重复
    ClampToEdge,   // 截取
    MirroredRepeat // 镜像重复
}

/** 
 * 像素格式
 */
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum DataFormat {
    Byte, // 每分量一字节
    UnsignedByte, // 每分量一字节
    Short, // 每分量两字节
    UnsignedShort, // 每分量两字节
    Int, // 每分量4字节
    UnsignedInt, // 每分量4字节
    Float, // 每分量4字节
}

/** 
 * 光栅化时的剔除状态
 */
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum CullMode {
    Back,  // 背面剔除
    Front,  // 正面剔除
}

/** 
 * 混合操作
 */
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum BlendFunc {
    Add,
    Sub,
    ReverseSub,
}

/** 
 * 混合因子
 */
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum RTAttachementType {
    Color0,
    Depth,
}

// 每像素字节数
pub fn pixe_size(pformat: PixelFormat, dformat: DataFormat) -> usize {
    match (pformat, dformat) {
       (PixelFormat::RGBA, DataFormat::Byte) |
       (PixelFormat::RGBA, DataFormat::UnsignedByte) => 4,
       (PixelFormat::RGBA, DataFormat::Short) |
       (PixelFormat::RGBA, DataFormat::UnsignedShort) => 8,
       (PixelFormat::RGBA, DataFormat::Int) |
       (PixelFormat::RGBA, DataFormat::UnsignedInt) |
       (PixelFormat::RGBA, DataFormat::Float) => 16,

       (PixelFormat::RGB, DataFormat::Byte) |
       (PixelFormat::RGB, DataFormat::UnsignedByte) => 3,
       (PixelFormat::RGB, DataFormat::Short) |
       (PixelFormat::RGB, DataFormat::UnsignedShort) => 6,
       (PixelFormat::RGB, DataFormat::Int) |
       (PixelFormat::RGB, DataFormat::UnsignedInt) |
       (PixelFormat::RGB, DataFormat::Float) => 9,

       (PixelFormat::ALPHA, DataFormat::Byte) |
       (PixelFormat::ALPHA, DataFormat::UnsignedByte) => 1,
       (PixelFormat::ALPHA, DataFormat::Short) |
       (PixelFormat::ALPHA, DataFormat::UnsignedShort) => 2,
       (PixelFormat::ALPHA, DataFormat::Int) |
       (PixelFormat::ALPHA, DataFormat::UnsignedInt) |
       (PixelFormat::ALPHA, DataFormat::Float) => 4,

       (PixelFormat::DEPTH16, DataFormat::Byte) |
       (PixelFormat::DEPTH16, DataFormat::UnsignedByte) => 2,
       (PixelFormat::DEPTH16, DataFormat::Short) |
       (PixelFormat::DEPTH16, DataFormat::UnsignedShort) => 4,
       (PixelFormat::DEPTH16, DataFormat::Int) |
       (PixelFormat::DEPTH16, DataFormat::UnsignedInt) |
       (PixelFormat::DEPTH16, DataFormat::Float) => 8,
    }
}