/**
 * 提供常用数据结构和WebGL的转换函数
 */
use hal_core::*;
use web_sys::WebGlRenderingContext;

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
        ShaderType::Vertex => WebGlRenderingContext::VERTEX_SHADER,
        ShaderType::Fragment => WebGlRenderingContext::FRAGMENT_SHADER,
    }
}

/**
 * 返回 (mag_mode, min_mode)
 */
pub fn get_texture_filter_mode(
    mag: TextureFilterMode,
    min: TextureFilterMode,
    mip: Option<TextureFilterMode>,
) -> (u32, u32) {
    let mag_mode = match mag {
        TextureFilterMode::Nearest => WebGlRenderingContext::NEAREST,
        TextureFilterMode::Linear => WebGlRenderingContext::LINEAR,
    };

    let min_mode = match mip {
        None => match min {
            TextureFilterMode::Nearest => WebGlRenderingContext::NEAREST,
            TextureFilterMode::Linear => WebGlRenderingContext::LINEAR,
        },
        Some(TextureFilterMode::Nearest) => match min {
            TextureFilterMode::Nearest => WebGlRenderingContext::NEAREST_MIPMAP_NEAREST,
            TextureFilterMode::Linear => WebGlRenderingContext::LINEAR_MIPMAP_NEAREST,
        },
        Some(TextureFilterMode::Linear) => match min {
            TextureFilterMode::Nearest => WebGlRenderingContext::NEAREST_MIPMAP_LINEAR,
            TextureFilterMode::Linear => WebGlRenderingContext::LINEAR_MIPMAP_LINEAR,
        },
    };

    (mag_mode, min_mode)
}

pub fn get_texture_wrap_mode(mode: TextureWrapMode) -> u32 {
    match mode {
        TextureWrapMode::Repeat => WebGlRenderingContext::REPEAT,
        TextureWrapMode::ClampToEdge => WebGlRenderingContext::CLAMP_TO_EDGE,
        TextureWrapMode::MirroredRepeat => WebGlRenderingContext::MIRRORED_REPEAT,
    }
}

pub fn get_pixel_format(format: PixelFormat) -> u32 {
    match format {
        PixelFormat::RGB => WebGlRenderingContext::RGB,
        PixelFormat::RGBA => WebGlRenderingContext::RGBA,
        PixelFormat::ALPHA => WebGlRenderingContext::ALPHA,
        PixelFormat::DEPTH16 => WebGlRenderingContext::DEPTH_COMPONENT16,
		PixelFormat::LUMINANCE => WebGlRenderingContext::LUMINANCE,
    }
}

// pub fn get_compressed_tex_extension(format: CmpressedTexFormat, extension: Extension) -> String {
//     match format {
//         CmpressedTexFormat::Astc => WEBGL_compressed_texture_astc,
//         CmpressedTexFormat::Atc => WEBGL_compressed_texture_atc,
//         CmpressedTexFormat::Etc => WEBGL_compressed_texture_etc,
//         CmpressedTexFormat::Pvrtc => WEBGL_compressed_texture_pvrtc,
//         CmpressedTexFormat::S3tc => WEBGL_compressed_texture_s3tc,
//         CmpressedTexFormat::S3tcSrgb => WEBGL_compressed_texture_s3tc_srgb,
//         _ => panic!("compressed_tex format invaild:{}", format),
//     }
// }

pub fn get_data_format(format: DataFormat) -> u32 {
    match format {
        DataFormat::Byte => WebGlRenderingContext::BYTE,
        DataFormat::UnsignedByte => WebGlRenderingContext::UNSIGNED_BYTE,
        DataFormat::Short => WebGlRenderingContext::SHORT,
        DataFormat::UnsignedShort => WebGlRenderingContext::UNSIGNED_SHORT,
        DataFormat::Int => WebGlRenderingContext::INT,
        DataFormat::UnsignedInt => WebGlRenderingContext::UNSIGNED_INT,
        DataFormat::Float => WebGlRenderingContext::FLOAT,
    }
}

pub fn get_cull_mode(mode: CullMode) -> u32 {
    match mode {
        CullMode::Back => WebGlRenderingContext::BACK,
        CullMode::Front => WebGlRenderingContext::FRONT,
    }
}

pub fn get_blend_func(func: BlendFunc) -> u32 {
    match func {
        BlendFunc::Add => WebGlRenderingContext::FUNC_ADD,
        BlendFunc::Sub => WebGlRenderingContext::FUNC_SUBTRACT,
        BlendFunc::ReverseSub => WebGlRenderingContext::FUNC_REVERSE_SUBTRACT,
    }
}

pub fn get_blend_factor(factor: BlendFactor) -> u32 {
    match factor {
        BlendFactor::Zero => WebGlRenderingContext::ZERO,
        BlendFactor::One => WebGlRenderingContext::ONE,
        BlendFactor::SrcColor => WebGlRenderingContext::SRC_COLOR,
        BlendFactor::OneMinusSrcColor => WebGlRenderingContext::ONE_MINUS_SRC_COLOR,
        BlendFactor::DstColor => WebGlRenderingContext::DST_COLOR,
        BlendFactor::OneMinusDstColor => WebGlRenderingContext::ONE_MINUS_DST_COLOR,
        BlendFactor::SrcAlpha => WebGlRenderingContext::SRC_ALPHA,
        BlendFactor::OneMinusSrcAlpha => WebGlRenderingContext::ONE_MINUS_SRC_ALPHA,
        BlendFactor::DstAlpha => WebGlRenderingContext::DST_ALPHA,
        BlendFactor::OneMinusDstAlpha => WebGlRenderingContext::ONE_MINUS_DST_ALPHA,
        BlendFactor::ConstantColor => WebGlRenderingContext::CONSTANT_COLOR,
        BlendFactor::OneMinusConstantColor => WebGlRenderingContext::ONE_MINUS_CONSTANT_COLOR,
        BlendFactor::ConstantAlpha => WebGlRenderingContext::CONSTANT_ALPHA,
        BlendFactor::OneMinusConstantAlpha => WebGlRenderingContext::ONE_MINUS_CONSTANT_ALPHA,
    }
}

pub fn get_compare_func(func: CompareFunc) -> u32 {
    match func {
        CompareFunc::Never => WebGlRenderingContext::NEVER,
        CompareFunc::Always => WebGlRenderingContext::ALWAYS,
        CompareFunc::Less => WebGlRenderingContext::LESS,
        CompareFunc::Equal => WebGlRenderingContext::EQUAL,
        CompareFunc::LEqual => WebGlRenderingContext::LEQUAL,
        CompareFunc::Greater => WebGlRenderingContext::GREATER,
        CompareFunc::GEqual => WebGlRenderingContext::GEQUAL,
        CompareFunc::NotEqual => WebGlRenderingContext::NOTEQUAL,
    }
}

pub fn get_stencil_op(op: StencilOp) -> u32 {
    match op {
        StencilOp::Keep => WebGlRenderingContext::KEEP,
        StencilOp::Zero => WebGlRenderingContext::ZERO,
        StencilOp::Replace => WebGlRenderingContext::REPLACE,
        StencilOp::Incr => WebGlRenderingContext::INCR,
        StencilOp::Decr => WebGlRenderingContext::DECR,
        StencilOp::Invert => WebGlRenderingContext::INVERT,
        StencilOp::IncrWrap => WebGlRenderingContext::INCR_WRAP,
        StencilOp::DecrWrap => WebGlRenderingContext::DECR_WRAP,
    }
}
