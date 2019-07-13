use stdweb::unstable::TryFrom;
use stdweb::Value;
/**
 * 扩展特性，用于caps模块
 */
use webgl_rendering_context::Extension;

pub struct OESElementIndexUint;

impl TryFrom<Value> for OESElementIndexUint {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(OESElementIndexUint),
        }
    }
}

impl Extension for OESElementIndexUint {
    const NAME: &'static str = "OES_element_index_uint";
}

pub struct ANGLEInstancedArrays;

impl TryFrom<Value> for ANGLEInstancedArrays {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(ANGLEInstancedArrays),
        }
    }
}

impl Extension for ANGLEInstancedArrays {
    const NAME: &'static str = "ANGLE_instanced_arrays";
}

pub struct OESStandardDerivatives;

impl TryFrom<Value> for OESStandardDerivatives {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(OESStandardDerivatives),
        }
    }
}

impl Extension for OESStandardDerivatives {
    const NAME: &'static str = "OES_standard_derivatives";
}

pub struct OESTextureFloat;

impl TryFrom<Value> for OESTextureFloat {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(OESTextureFloat),
        }
    }
}

impl Extension for OESTextureFloat {
    const NAME: &'static str = "OES_texture_float";
}

pub struct OESTextureFloatLinear;

impl TryFrom<Value> for OESTextureFloatLinear {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(OESTextureFloatLinear),
        }
    }
}

impl Extension for OESTextureFloatLinear {
    const NAME: &'static str = "OES_texture_float_linear";
}

pub struct OESTextureHalfFloat;

impl TryFrom<Value> for OESTextureHalfFloat {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(OESTextureHalfFloat),
        }
    }
}

impl Extension for OESTextureHalfFloat {
    const NAME: &'static str = "OES_texture_half_float";
}

pub struct OESTextureHalfFloatLinear;

impl TryFrom<Value> for OESTextureHalfFloatLinear {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(OESTextureHalfFloatLinear),
        }
    }
}

impl Extension for OESTextureHalfFloatLinear {
    const NAME: &'static str = "OES_texture_half_float_linear";
}

pub struct EXTSRGB;

impl TryFrom<Value> for EXTSRGB {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(EXTSRGB),
        }
    }
}

impl Extension for EXTSRGB {
    const NAME: &'static str = "EXT_sRGB";
}

pub struct OESVertexArrayObject;

impl TryFrom<Value> for OESVertexArrayObject {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(OESVertexArrayObject),
        }
    }
}

impl Extension for OESVertexArrayObject {
    const NAME: &'static str = "OES_vertex_array_object";
}

pub struct EXTTextureFilterAnisotropic;

impl TryFrom<Value> for EXTTextureFilterAnisotropic {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(EXTTextureFilterAnisotropic),
        }
    }
}

impl Extension for EXTTextureFilterAnisotropic {
    const NAME: &'static str = "EXT_texture_filter_anisotropic";
}

pub struct WEBKITEXTTextureFilterAnisotropic;

impl TryFrom<Value> for WEBKITEXTTextureFilterAnisotropic {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(WEBKITEXTTextureFilterAnisotropic),
        }
    }
}

impl Extension for WEBKITEXTTextureFilterAnisotropic {
    const NAME: &'static str = "WEBKIT_EXT_texture_filter_anisotropic";
}

pub struct EXTFragDepth;

impl TryFrom<Value> for EXTFragDepth {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(EXTFragDepth),
        }
    }
}

impl Extension for EXTFragDepth {
    const NAME: &'static str = "EXT_frag_depth";
}

pub struct WEBGLDepthTexture;

impl TryFrom<Value> for WEBGLDepthTexture {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(WEBGLDepthTexture),
        }
    }
}

impl Extension for WEBGLDepthTexture {
    const NAME: &'static str = "WEBGL_depth_texture";
}

pub struct WEBGLColorBufferFloat;

impl TryFrom<Value> for WEBGLColorBufferFloat {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(WEBGLColorBufferFloat),
        }
    }
}

impl Extension for WEBGLColorBufferFloat {
    const NAME: &'static str = "WEBGL_color_buffer_float";
}

pub struct EXTColorBufferHalfFloat;

impl TryFrom<Value> for EXTColorBufferHalfFloat {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(EXTColorBufferHalfFloat),
        }
    }
}

impl Extension for EXTColorBufferHalfFloat {
    const NAME: &'static str = "EXT_color_buffer_half_float";
}

pub struct EXTShaderTextureLod;

impl TryFrom<Value> for EXTShaderTextureLod {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(EXTShaderTextureLod),
        }
    }
}

impl Extension for EXTShaderTextureLod {
    const NAME: &'static str = "EXT_shader_texture_lod";
}

pub struct WEBGLDrawBuffers;

impl TryFrom<Value> for WEBGLDrawBuffers {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(WEBGLDrawBuffers),
        }
    }
}

impl Extension for WEBGLDrawBuffers {
    const NAME: &'static str = "WEBGL_draw_buffers";
}

pub struct GLOESStandardDerivatives;

impl TryFrom<Value> for GLOESStandardDerivatives {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(GLOESStandardDerivatives),
        }
    }
}

impl Extension for GLOESStandardDerivatives {
    const NAME: &'static str = "GL_OES_standard_derivatives";
}

pub struct CompressedTextureAstc;

impl TryFrom<Value> for CompressedTextureAstc {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(CompressedTextureAstc),
        }
    }
}

impl Extension for CompressedTextureAstc {
    const NAME: &'static str = "WEBGL_compressed_texture_astc";
}

// ===========================================

pub struct WebkitCompressedTextureAstc;

impl TryFrom<Value> for WebkitCompressedTextureAstc {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(WebkitCompressedTextureAstc),
        }
    }
}

impl Extension for WebkitCompressedTextureAstc {
    const NAME: &'static str = "WEBKIT_WEBGL_compressed_texture_astc";
}

// ===========================================

//
pub struct CompressedTextureS3tc;

impl TryFrom<Value> for CompressedTextureS3tc {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(CompressedTextureS3tc),
        }
    }
}

impl Extension for CompressedTextureS3tc {
    const NAME: &'static str = "WEBGL_compressed_texture_s3tc";
}

pub struct WebkitCompressedTextureS3tc;

impl TryFrom<Value> for WebkitCompressedTextureS3tc {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(WebkitCompressedTextureS3tc),
        }
    }
}

impl Extension for WebkitCompressedTextureS3tc {
    const NAME: &'static str = "WEBKIT_WEBGL_compressed_texture_s3tc";
}

// ===========================================

pub struct CompressedTexturePvrtc;

impl TryFrom<Value> for CompressedTexturePvrtc {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(CompressedTexturePvrtc),
        }
    }
}

impl Extension for CompressedTexturePvrtc {
    const NAME: &'static str = "WEBGL_compressed_texture_pvrtc";
}

pub struct WebkitCompressedTexturePvrtc;

impl TryFrom<Value> for WebkitCompressedTexturePvrtc {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(WebkitCompressedTexturePvrtc),
        }
    }
}

impl Extension for WebkitCompressedTexturePvrtc {
    const NAME: &'static str = "WEBKIT_WEBGL_compressed_texture_pvrtc";
}

// ===========================================

pub struct CompressedTextureEtc1;

impl TryFrom<Value> for CompressedTextureEtc1 {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(CompressedTextureEtc1),
        }
    }
}

impl Extension for CompressedTextureEtc1 {
    const NAME: &'static str = "WEBGL_compressed_texture_etc1";
}

pub struct WebkitCompressedTextureEtc1;

impl TryFrom<Value> for WebkitCompressedTextureEtc1 {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(WebkitCompressedTextureEtc1),
        }
    }
}

impl Extension for WebkitCompressedTextureEtc1 {
    const NAME: &'static str = "WEBKIT_WEBGL_compressed_texture_etc1";
}

// ===========================================

pub struct CompressedTextureEtc2;

impl TryFrom<Value> for CompressedTextureEtc2 {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(CompressedTextureEtc2),
        }
    }
}

impl Extension for CompressedTextureEtc2 {
    const NAME: &'static str = "WEBGL_compressed_texture_etc";
}

pub struct WebkitCompressedTextureEtc2;

impl TryFrom<Value> for WebkitCompressedTextureEtc2 {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(WebkitCompressedTextureEtc2),
        }
    }
}

impl Extension for WebkitCompressedTextureEtc2 {
    const NAME: &'static str = "WEBKIT_WEBGL_compressed_texture_etc";
}

pub struct CompressedTextureEs3;

impl TryFrom<Value> for CompressedTextureEs3 {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(CompressedTextureEs3),
        }
    }
}

impl Extension for CompressedTextureEs3 {
    const NAME: &'static str = "WEBGL_compressed_texture_es3_0";
}
