use self::util::{TextureFilterMode, TextureWrapMode};

/** 
 * 纹理采样器描述，sampler，纹理的采样状态
 */
#[derive(Debug, Clone)]
pub struct SamplerDesc {
    pub min_filter: TextureFilterMode,
    pub mag_filter: TextureFilterMode,
    pub mip_filter: Option<TextureFilterMode>,

    pub u_wrap: TextureWrapMode,
    pub v_wrap: TextureWrapMode,
}

impl SamplerDesc {
    
    pub fn new() -> Self {
        Self {
            min_filter: TextureFilterMode::Linear,
            mag_filter: TextureFilterMode::Linear,
            mip_filter: None,

            u_wrap: TextureWrapMode::Repeat,
            v_wrap: TextureWrapMode::Repeat,
        }
    }
}