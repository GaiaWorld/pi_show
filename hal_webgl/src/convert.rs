/**
 * 提供常用数据结构和WebGL的转换函数
 */
use hal_core::*;
use webgl_rendering_context::{WebGLRenderingContext};

pub fn get_attribute_location(name: &AttributeName) -> u32 {
    match name {
        AttributeName::Position => 0,
        AttributeName::Normal => 1,
        AttributeName::Color => 2,
        AttributeName::UV0 => 3,
        AttributeName::UV1 => 4,
        AttributeName::SkinIndex => 5,
        AttributeName::SkinWeight => 6,
        AttributeName::Tangent => 7,
        AttributeName::BiNormal => 8,
        AttributeName::UV2 => 9,
        AttributeName::UV3 => 10,
        AttributeName::UV4 => 11,
        AttributeName::UV5 => 12,
        AttributeName::UV6 => 13,
        AttributeName::UV7 => 14,
        AttributeName::UV8 => 15,
        _ => {
            assert!(false, "get_attribute_location failed!");
            10000
        }   
    }
}

pub fn get_shader_type(stype: ShaderType) -> u32 {
    match stype {
        ShaderType::Vertex => WebGLRenderingContext::VERTEX_SHADER,
        ShaderType::Fragment => WebGLRenderingContext::FRAGMENT_SHADER,
    }
}

/** 
 * 返回 (mag_mode, min_mode)
 */
pub fn get_texture_filter_mode(mag: TextureFilterMode, min: TextureFilterMode, mip: Option<TextureFilterMode>) -> (u32, u32) {
    let mag_mode = match mag {
        TextureFilterMode::Nearest => WebGLRenderingContext::NEAREST,
        TextureFilterMode::Linear => WebGLRenderingContext::LINEAR,
    };

    let min_mode = match mip {
        None => {
            match min {
                TextureFilterMode::Nearest => WebGLRenderingContext::NEAREST,
                TextureFilterMode::Linear => WebGLRenderingContext::LINEAR,
            }
        }
        Some(TextureFilterMode::Nearest) => {
            match min {
                TextureFilterMode::Nearest => WebGLRenderingContext::NEAREST_MIPMAP_NEAREST,
                TextureFilterMode::Linear => WebGLRenderingContext::LINEAR_MIPMAP_NEAREST,
            }
        }
        Some(TextureFilterMode::Linear) => {
            match min {
                TextureFilterMode::Nearest => WebGLRenderingContext::NEAREST_MIPMAP_LINEAR,
                TextureFilterMode::Linear => WebGLRenderingContext::LINEAR_MIPMAP_LINEAR,
            }
        }
    };

    (mag_mode, min_mode)
}

pub fn get_texture_wrap_mode(mode: TextureWrapMode) -> u32 { 
    match mode {
        TextureWrapMode::Repeat => WebGLRenderingContext::REPEAT,
        TextureWrapMode::ClampToEdge => WebGLRenderingContext::CLAMP_TO_EDGE,
        TextureWrapMode::MirroredRepeat => WebGLRenderingContext::MIRRORED_REPEAT,
    }
}

pub fn get_pixel_format(format: PixelFormat) -> u32 {
    match format {
        PixelFormat::RGB => WebGLRenderingContext::RGB,
        PixelFormat::RGBA => WebGLRenderingContext::RGBA,
        PixelFormat::ALPHA => WebGLRenderingContext::ALPHA,
        PixelFormat::DEPTH16 => WebGLRenderingContext::DEPTH_COMPONENT16,
    }
}

pub fn get_data_format(format: DataFormat) -> u32 {
    match format {
        DataFormat::Byte => WebGLRenderingContext::BYTE,
        DataFormat::UnsignedByte => WebGLRenderingContext::UNSIGNED_BYTE,
        DataFormat::Short => WebGLRenderingContext::SHORT,
        DataFormat::UnsignedShort => WebGLRenderingContext::UNSIGNED_SHORT,
        DataFormat::Int => WebGLRenderingContext::INT,
        DataFormat::UnsignedInt => WebGLRenderingContext::UNSIGNED_INT,
        DataFormat::Float => WebGLRenderingContext::FLOAT,
    }
}

pub fn get_cull_mode(mode: CullMode) -> u32 {
    match mode {
        CullMode::Back => WebGLRenderingContext::BACK,
        CullMode::Front => WebGLRenderingContext::FRONT,
    }
}

pub fn get_blend_func(func: BlendFunc) -> u32 {
    match func {
        BlendFunc::Add => WebGLRenderingContext::FUNC_ADD,
        BlendFunc::Sub => WebGLRenderingContext::FUNC_SUBTRACT,
        BlendFunc::ReverseSub => WebGLRenderingContext::FUNC_REVERSE_SUBTRACT,
    }
}

pub fn get_blend_factor(factor: BlendFactor) -> u32 {
    match factor {
        BlendFactor::Zero => WebGLRenderingContext::ZERO,
        BlendFactor::One => WebGLRenderingContext::ONE,
        BlendFactor::SrcColor => WebGLRenderingContext::SRC_COLOR,
        BlendFactor::OneMinusSrcColor => WebGLRenderingContext::ONE_MINUS_SRC_COLOR,
        BlendFactor::DstColor => WebGLRenderingContext::DST_COLOR,
        BlendFactor::OneMinusDstColor => WebGLRenderingContext::ONE_MINUS_DST_COLOR,
        BlendFactor::SrcAlpha => WebGLRenderingContext::SRC_ALPHA,
        BlendFactor::OneMinusSrcAlpha => WebGLRenderingContext::ONE_MINUS_SRC_ALPHA,
        BlendFactor::DstAlpha => WebGLRenderingContext::DST_ALPHA,
        BlendFactor::OneMinusDstAlpha => WebGLRenderingContext::ONE_MINUS_DST_ALPHA,
        BlendFactor::ConstantColor => WebGLRenderingContext::CONSTANT_COLOR,
        BlendFactor::OneMinusConstantColor => WebGLRenderingContext::ONE_MINUS_CONSTANT_COLOR,
        BlendFactor::ConstantAlpha => WebGLRenderingContext::CONSTANT_ALPHA,
        BlendFactor::OneMinusConstantAlpha => WebGLRenderingContext::ONE_MINUS_CONSTANT_ALPHA,
    }
}

pub fn get_compare_func(func: CompareFunc) -> u32 {
    match func {
        CompareFunc::Never => WebGLRenderingContext::NEVER,
        CompareFunc::Always => WebGLRenderingContext::ALWAYS,
        CompareFunc::Less => WebGLRenderingContext::LESS,
        CompareFunc::Equal => WebGLRenderingContext::EQUAL,
        CompareFunc::LEqual => WebGLRenderingContext::LEQUAL,
        CompareFunc::Greater => WebGLRenderingContext::GREATER,
        CompareFunc::GEqual => WebGLRenderingContext::GEQUAL,
        CompareFunc::NotEqual => WebGLRenderingContext::NOTEQUAL,
    }
}

pub fn get_stencil_op(op: StencilOp) -> u32 {
    match op {
        StencilOp::Keep => WebGLRenderingContext::KEEP,
        StencilOp::Zero => WebGLRenderingContext::ZERO,
        StencilOp::Replace => WebGLRenderingContext::REPLACE,
        StencilOp::Incr => WebGLRenderingContext::INCR,
        StencilOp::Decr => WebGLRenderingContext::DECR,
        StencilOp::Invert => WebGLRenderingContext::INVERT,
        StencilOp::IncrWrap => WebGLRenderingContext::INCR_WRAP,
        StencilOp::DecrWrap => WebGLRenderingContext::DECR_WRAP,
    }   
}

pub fn get_render_target_attachment(attachment: RTAttachment) -> u32 {
    match attachment {
        RTAttachment::Color0 => WebGLRenderingContext::COLOR_ATTACHMENT0,
        RTAttachment::Depth => WebGLRenderingContext::DEPTH_ATTACHMENT,
        RTAttachment::Stencil => WebGLRenderingContext::STENCIL_ATTACHMENT,
    }
}