
/** 
 * 定义渲染常用的数据结构
 */

use atom::{Atom};

/** 
 * 着色器的类型
 */
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

/** 
 * Attribute的名字，类型可以更改，
 * 注：请尽量使用内置的Attribute名，以便于内部加速
 */
#[derive(PartialEq, Hash, Eq, Clone, Debug)]
pub enum AttributeName {
    Position,   // shader attribute：position，一般是vec3
    Normal,     // shader attribute：normal，一般是vec3 
    Color,      // shader attribute：color，一般是vec4
    UV0,        // shader attribute：uv0，一般是vec2
    UV1,        // shader attribute：uv1，一般是vec2
    SkinIndex,  // shader attribute：skinIndex，一般是vec4
    SkinWeight, // shader attribute：skinWeight，一般是vec4
    Tangent,    // shader attribute：tangent，一般是vec3
    BiNormal,   // shader attribute：binormal，一般是vec3
    UV2,        // shader attribute：uv2，一般是vec2
    UV3,        // shader attribute：uv3，一般是vec2
    UV4,        // shader attribute：uv4，一般是vec2
    UV5,        // shader attribute：uv5，一般是vec2
    UV6,        // shader attribute：uv6，一般是vec2
    UV7,        // shader attribute：uv7，一般是vec2
    UV8,        // shader attribute：uv8，一般是vec2
    Custom(Atom), // 自定义名字，无非必要，最好不用,
}

/** 
 * 内置Attribute名字的就是上面的16个
 */
pub fn get_builtin_attribute_count() -> u32 {
    16
}

impl From<Atom> for AttributeName {
    fn from(name: Atom) -> AttributeName {
        match name.as_ref() {
            "position" => AttributeName::Position,
            "normal" => AttributeName::Normal,
            "color" => AttributeName::Color,
            "uv0" => AttributeName::UV0,
            "uv1" => AttributeName::UV1,
            "skinIndex" => AttributeName::SkinIndex,
            "skinWeight" => AttributeName::SkinWeight,
            "tangent" => AttributeName::Tangent,
            "binormal" => AttributeName::BiNormal,
            "uv2" => AttributeName::UV2,
            "uv3" => AttributeName::UV3,
            "uv4" => AttributeName::UV4,
            "uv5" => AttributeName::UV5,
            "uv6" => AttributeName::UV6,
            "uv7" => AttributeName::UV7,
            "uv8" => AttributeName::UV8,
            n @ _  => AttributeName::Custom(Atom::from(n)),
        }
    }
}

impl Into<Atom> for AttributeName {
    fn into(self) -> Atom {
        match self {
            AttributeName::Position => Atom::from("position"),
            AttributeName::Normal => Atom::from("normal"),
            AttributeName::Color => Atom::from("color"),
            AttributeName::UV0 => Atom::from("uv0"),
            AttributeName::UV1 => Atom::from("uv1"),
            AttributeName::SkinIndex => Atom::from("skinIndex"),
            AttributeName::SkinWeight => Atom::from("skinWeight"),
            AttributeName::Tangent => Atom::from("tangent"),
            AttributeName::BiNormal => Atom::from("binormal"),
            AttributeName::UV2 => Atom::from("uv2"),
            AttributeName::UV3 => Atom::from("uv3"),
            AttributeName::UV4 => Atom::from("uv4"),
            AttributeName::UV5 => Atom::from("uv5"),
            AttributeName::UV6 => Atom::from("uv6"),
            AttributeName::UV7 => Atom::from("uv7"),
            AttributeName::UV8 => Atom::from("uv8"),
            AttributeName::Custom(n) => n.clone(),
        }       
    }
}

/** 
 * 纹理的过滤模式
 */
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TextureFilterMode {
    Nearest,
    Linear,
}

/** 
 * 纹理环绕模式
 * 指：当纹理坐标不在[0, 1]范围时，如何处理
 */
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TextureWrapMode {
    Repeat,        // 重复
    ClampToEdge,   // 截取
    MirroredRepeat // 镜像重复
}

/** 
 * 像素格式
 */
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PixelFormat {
    RGB,
    RGBA,   
    ALPHA, 
    DEPTH16,
}

/** 
 * 数据格式
 */
#[derive(PartialEq, Clone, Copy, Debug)]
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
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum CullMode {
    Back,  // 背面剔除
    Front,  // 正面剔除
}

/** 
 * 混合操作
 */
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum BlendFunc {
    Add,
    Sub,
    ReverseSub,
}

/** 
 * 混合因子
 */
#[derive(PartialEq, Clone, Copy, Debug)]
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
#[derive(PartialEq, Clone, Copy, Debug)]
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
#[derive(PartialEq, Clone, Copy, Debug)]
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
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum RTAttachment {
    Color0, // 第一个颜色缓冲区
    Depth,  // 深度缓冲区
    Stencil, // 模板缓冲区
}