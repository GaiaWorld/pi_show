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

pub struct CompressedTexAstcExtension {
    pub rgba_astc_4x4: u32,
}
pub struct CompressedTextureAstc(pub CompressedTexAstcExtension);

impl TryFrom<Value> for CompressedTextureAstc {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(CompressedTextureAstc(CompressedTexAstcExtension {
                // rgba_astc_4x4: u32::try_from(js! {@{v}.COMPRESSED_RGBA_ASTC_4x4_KHR}).unwrap(),
                rgba_astc_4x4: 1,
            })),
        }
    }
}

impl Extension for CompressedTextureAstc {
    const NAME: &'static str = "WEBGL_compressed_texture_astc";
}

// ===========================================

pub struct WebkitCompressedTextureAstc(pub CompressedTexAstcExtension);

impl TryFrom<Value> for WebkitCompressedTextureAstc {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(WebkitCompressedTextureAstc(CompressedTexAstcExtension {
                // rgba_astc_4x4: u32::try_from(js! {@{v}.COMPRESSED_RGBA_ASTC_4x4_KHR}).unwrap(),
                rgba_astc_4x4: 1,
            })),
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

pub struct CompressedTexPvrtcExtension {
    pub rgb_pvrtc_4bppv1: u32,
    pub rgba_pvrtc_4bppv1: u32,
    pub rgb_pvrtc_2bppv1: u32,
    pub rgba_pvrtc_2bppv1: u32,
}

pub struct CompressedTexturePvrtc(pub CompressedTexPvrtcExtension);

impl TryFrom<Value> for CompressedTexturePvrtc {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(CompressedTexturePvrtc(CompressedTexPvrtcExtension {
                // rgb_pvrtc_4bppv1: u32::try_from(js! {@{&v}.COMPRESSED_RGB_PVRTC_4BPPV1_IMG})
                //     .unwrap(),
                // rgba_pvrtc_4bppv1: u32::try_from(js! {@{&v}.COMPRESSED_RGBA_PVRTC_4BPPV1_IMG})
                //     .unwrap(),
                // rgb_pvrtc_2bppv1: u32::try_from(js! {@{&v}.COMPRESSED_RGB_PVRTC_2BPPV1_IMG})
                //     .unwrap(),
                // rgba_pvrtc_2bppv1: u32::try_from(js! {@{&v}.COMPRESSED_RGBA_PVRTC_2BPPV1_IMG})
                // 	.unwrap(),
                rgb_pvrtc_4bppv1: 1,
                rgba_pvrtc_4bppv1: 1,
                rgb_pvrtc_2bppv1: 1,
                rgba_pvrtc_2bppv1: 1,
            })),
        }
    }
}

impl Extension for CompressedTexturePvrtc {
    const NAME: &'static str = "WEBGL_compressed_texture_pvrtc";
}

pub struct WebkitCompressedTexturePvrtc(pub CompressedTexPvrtcExtension);

impl TryFrom<Value> for WebkitCompressedTexturePvrtc {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(WebkitCompressedTexturePvrtc(CompressedTexPvrtcExtension {
                // rgb_pvrtc_4bppv1: u32::try_from(js! {@{&v}.COMPRESSED_RGB_PVRTC_4BPPV1_IMG})
                //     .unwrap(),
                // rgba_pvrtc_4bppv1: u32::try_from(js! {@{&v}.COMPRESSED_RGBA_PVRTC_4BPPV1_IMG})
                //     .unwrap(),
                // rgb_pvrtc_2bppv1: u32::try_from(js! {@{&v}.COMPRESSED_RGB_PVRTC_2BPPV1_IMG})
                //     .unwrap(),
                // rgba_pvrtc_2bppv1: u32::try_from(js! {@{&v}.COMPRESSED_RGBA_PVRTC_2BPPV1_IMG})
                //     .unwrap(),
                rgb_pvrtc_4bppv1: 1,
                rgba_pvrtc_4bppv1: 1,
                rgb_pvrtc_2bppv1: 1,
                rgba_pvrtc_2bppv1: 1,
            })),
        }
    }
}

impl Extension for WebkitCompressedTexturePvrtc {
    const NAME: &'static str = "WEBKIT_WEBGL_compressed_texture_pvrtc";
}

// ===========================================

pub struct CompressedTexEtc1Extension {
    pub rgb_etc1: u32,
}

pub struct CompressedTextureEtc1(pub CompressedTexEtc1Extension);

impl TryFrom<Value> for CompressedTextureEtc1 {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(CompressedTextureEtc1(CompressedTexEtc1Extension {
                // rgb_etc1: u32::try_from(js! {@{v}.COMPRESSED_RGB_ETC1_WEBGL}).unwrap(),
                rgb_etc1: 1,
            })),
        }
    }
}

impl Extension for CompressedTextureEtc1 {
    const NAME: &'static str = "WEBGL_compressed_texture_etc1";
}

pub struct WebkitCompressedTextureEtc1(pub CompressedTexEtc1Extension);

impl TryFrom<Value> for WebkitCompressedTextureEtc1 {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(WebkitCompressedTextureEtc1(CompressedTexEtc1Extension {
                // rgb_etc1: u32::try_from(js! {@{v}.COMPRESSED_RGB_ETC1_WEBGL}).unwrap(),
                rgb_etc1: 1,
            })),
        }
    }
}

impl Extension for WebkitCompressedTextureEtc1 {
    const NAME: &'static str = "WEBKIT_WEBGL_compressed_texture_etc1";
}

// ===========================================

pub struct CompressedTexEtc2Extension {
    pub rgb8_etc2: u32,
    pub rgba8_etc2_eac: u32,
}

pub struct CompressedTextureEtc2(pub CompressedTexEtc2Extension);

impl TryFrom<Value> for CompressedTextureEtc2 {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(CompressedTextureEtc2(CompressedTexEtc2Extension {
                // rgb8_etc2: u32::try_from(js! {@{&v}.COMPRESSED_RGB8_ETC2}).unwrap(),
                // rgba8_etc2_eac: u32::try_from(js! {@{&v}.COMPRESSED_RGBA8_ETC2_EAC}).unwrap(),
                rgb8_etc2: 1,
                rgba8_etc2_eac: 1,
            })),
        }
    }
}

impl Extension for CompressedTextureEtc2 {
    const NAME: &'static str = "WEBGL_compressed_texture_etc";
}

pub struct WebkitCompressedTextureEtc2(pub CompressedTexEtc2Extension);

impl TryFrom<Value> for WebkitCompressedTextureEtc2 {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Undefined => Err(String::from("Undefined")),
            Value::Null => Err(String::from("Null")),
            _ => Ok(WebkitCompressedTextureEtc2(CompressedTexEtc2Extension {
                // rgb8_etc2: u32::try_from(js! {@{&v}.COMPRESSED_RGB8_ETC2}).unwrap(),
                // rgba8_etc2_eac: u32::try_from(js! {@{v}.COMPRESSED_RGBA8_ETC2_EAC}).unwrap(),
                rgb8_etc2: 1,
                rgba8_etc2_eac: 1,
            })),
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
