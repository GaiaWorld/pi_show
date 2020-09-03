// use stdweb::unstable::TryFrom;
// use stdweb::JsValue;
// /**
//  * 扩展特性，用于caps模块
//  */
// use web_sys::Extension;
use wasm_bindgen::prelude::*;
use std::convert::TryFrom;

pub trait Extension {
	const NAME: &'static str;
}

#[macro_use()]
macro_rules! impl_extension_js_value {
    ($ty:ident, $strdesc: expr) => {
		pub struct $ty;
        impl TryFrom<JsValue> for $ty {
			type Error = String;
			fn try_from(v: JsValue) -> Result<Self, Self::Error> {
				if v.is_null() {
					Err(String::from("Undefined"))
				} else if v.is_undefined() {
					Err(String::from("Null"))
				} else {
					Ok($ty)
				}
			}
		}

		impl Extension for $ty {
			const NAME: &'static str = $strdesc;
		}
	}
}

#[macro_use()]
macro_rules! impl_extension_js_value1 {
    ($ty:ident, $strdesc: expr, $v: expr) => {
        impl TryFrom<JsValue> for $ty {
			type Error = String;
			fn try_from(v: JsValue) -> Result<Self, Self::Error> {
				if v.is_null() {
					Err(String::from("Undefined"))
				} else if v.is_undefined() {
					Err(String::from("Null"))
				} else {
					Ok($v)
				}
			}
		}

		impl Extension for $ty {
			const NAME: &'static str = $strdesc;
		}
	}
}

impl_extension_js_value!(OESElementIndexUint, "OES_element_index_uint");
impl_extension_js_value!(ANGLEInstancedArrays, "ANGLE_instanced_arrays");
impl_extension_js_value!(OESStandardDerivatives, "OES_standard_derivatives");
impl_extension_js_value!(OESTextureFloat, "OES_texture_float");
impl_extension_js_value!(OESTextureFloatLinear, "OES_texture_float_linear");
impl_extension_js_value!(OESTextureHalfFloat, "OES_texture_half_float");
impl_extension_js_value!(OESTextureHalfFloatLinear, "OES_texture_half_float_linear");
impl_extension_js_value!(EXTSRGB, "EXT_sRGB");
impl_extension_js_value!(OESVertexArrayObject, "OES_vertex_array_object");
impl_extension_js_value!(EXTTextureFilterAnisotropic, "EXT_texture_filter_anisotropic");
impl_extension_js_value!(WEBKITEXTTextureFilterAnisotropic, "WEBKIT_EXT_texture_filter_anisotropic");
impl_extension_js_value!(EXTFragDepth, "EXT_frag_depth");
impl_extension_js_value!(WEBGLDepthTexture, "WEBGL_depth_texture");
impl_extension_js_value!(WEBGLColorBufferFloat, "WEBGL_color_buffer_float");
impl_extension_js_value!(EXTColorBufferHalfFloat, "EXT_color_buffer_half_float");
impl_extension_js_value!(EXTShaderTextureLod, "EXT_shader_texture_lod");
impl_extension_js_value!(WEBGLDrawBuffers, "WEBGL_draw_buffers");
impl_extension_js_value!(GLOESStandardDerivatives, "GL_OES_standard_derivatives");

// 压缩纹理
impl_extension_js_value!(CompressedTextureS3tc, "WEBGL_compressed_texture_s3tc");
impl_extension_js_value!(WebkitCompressedTextureS3tc, "WEBKIT_WEBGL_compressed_texture_s3tc");
impl_extension_js_value!(CompressedTextureEs3, "WEBGL_compressed_texture_es3_0");

// 特殊压缩纹理
pub struct CompressedTexAstcExtension {
    pub rgba_astc_4x4: u32,
}
pub struct CompressedTextureAstc(pub CompressedTexAstcExtension);
impl_extension_js_value1!(
	CompressedTextureAstc,
	"WEBGL_compressed_texture_astc",
	CompressedTextureAstc(CompressedTexAstcExtension {
		// rgba_astc_4x4: u32::try_from(js! {@{v}.COMPRESSED_RGBA_ASTC_4x4_KHR}).unwrap(),
		rgba_astc_4x4: 1,
	})
);


// ===========================================

pub struct WebkitCompressedTextureAstc(pub CompressedTexAstcExtension);
impl_extension_js_value1!(
	WebkitCompressedTextureAstc,
	"WEBKIT_WEBGL_compressed_texture_astc",
	WebkitCompressedTextureAstc(CompressedTexAstcExtension {
		// rgba_astc_4x4: u32::try_from(js! {@{v}.COMPRESSED_RGBA_ASTC_4x4_KHR}).unwrap(),
		rgba_astc_4x4: 1,
	})
);
// ===========================================

pub struct CompressedTexPvrtcExtension {
    pub rgb_pvrtc_4bppv1: u32,
    pub rgba_pvrtc_4bppv1: u32,
    pub rgb_pvrtc_2bppv1: u32,
    pub rgba_pvrtc_2bppv1: u32,
}

pub struct CompressedTexturePvrtc(pub CompressedTexPvrtcExtension);
impl_extension_js_value1!(
	CompressedTexturePvrtc,
	"WEBGL_compressed_texture_pvrtc",
	CompressedTexturePvrtc(CompressedTexPvrtcExtension {
		// rgba_astc_4x4: u32::try_from(js! {@{v}.COMPRESSED_RGBA_ASTC_4x4_KHR}).unwrap(),
		rgb_pvrtc_4bppv1: 1,
		rgba_pvrtc_4bppv1: 1,
		rgb_pvrtc_2bppv1: 1,
		rgba_pvrtc_2bppv1: 1,
	})
);

pub struct WebkitCompressedTexturePvrtc(pub CompressedTexPvrtcExtension);
impl_extension_js_value1!(
	WebkitCompressedTexturePvrtc,
	"WEBKIT_WEBGL_compressed_texture_pvrtc",
	WebkitCompressedTexturePvrtc(CompressedTexPvrtcExtension {
		// rgba_astc_4x4: u32::try_from(js! {@{v}.COMPRESSED_RGBA_ASTC_4x4_KHR}).unwrap(),
		rgb_pvrtc_4bppv1: 1,
		rgba_pvrtc_4bppv1: 1,
		rgb_pvrtc_2bppv1: 1,
		rgba_pvrtc_2bppv1: 1,
	})
);

// ===========================================

pub struct CompressedTexEtc1Extension {
    pub rgb_etc1: u32,
}

pub struct CompressedTextureEtc1(pub CompressedTexEtc1Extension);
impl_extension_js_value1!(
	CompressedTextureEtc1,
	"WEBGL_compressed_texture_etc1",
	CompressedTextureEtc1(CompressedTexEtc1Extension {
		// rgba_astc_4x4: u32::try_from(js! {@{v}.COMPRESSED_RGBA_ASTC_4x4_KHR}).unwrap(),
		rgb_etc1: 1,
	})
);

pub struct WebkitCompressedTextureEtc1(pub CompressedTexEtc1Extension);
impl_extension_js_value1!(
	WebkitCompressedTextureEtc1,
	"WEBKIT_WEBGL_compressed_texture_etc1",
	WebkitCompressedTextureEtc1(CompressedTexEtc1Extension {
		// rgba_astc_4x4: u32::try_from(js! {@{v}.COMPRESSED_RGBA_ASTC_4x4_KHR}).unwrap(),
		rgb_etc1: 1,
	})
);
// ===========================================

pub struct CompressedTexEtc2Extension {
    pub rgb8_etc2: u32,
    pub rgba8_etc2_eac: u32,
}
pub struct CompressedTextureEtc2(pub CompressedTexEtc2Extension);
impl_extension_js_value1!(
	CompressedTextureEtc2,
	"WEBGL_compressed_texture_etc",
	CompressedTextureEtc2(CompressedTexEtc2Extension {
		// rgb8_etc2: u32::try_from(js! {@{&v}.COMPRESSED_RGB8_ETC2}).unwrap(),
		// rgba8_etc2_eac: u32::try_from(js! {@{&v}.COMPRESSED_RGBA8_ETC2_EAC}).unwrap(),
		rgb8_etc2: 1,
		rgba8_etc2_eac: 1,
	})
);

pub struct WebkitCompressedTextureEtc2(pub CompressedTexEtc2Extension);
impl_extension_js_value1!(
	WebkitCompressedTextureEtc2,
	"WEBKIT_WEBGL_compressed_texture_etc",
	WebkitCompressedTextureEtc2(CompressedTexEtc2Extension {
		// rgb8_etc2: u32::try_from(js! {@{&v}.COMPRESSED_RGB8_ETC2}).unwrap(),
		// rgba8_etc2_eac: u32::try_from(js! {@{&v}.COMPRESSED_RGBA8_ETC2_EAC}).unwrap(),
		rgb8_etc2: 1,
		rgba8_etc2_eac: 1,
	})
);
