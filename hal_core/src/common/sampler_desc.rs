use common::util::{TextureFilterMode, TextureWrapMode};

/** 
 * 纹理采样器描述，sampler，纹理的采样状态
 */
#[derive(Debug, Clone, Hash)]
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

    pub fn set_filter_mode(&mut self, mag: TextureFilterMode, min: TextureFilterMode, mip: Option<TextureFilterMode>) {
        self.min_filter = min;
        self.mag_filter = mag;
        self.mip_filter = mip;
    }

    pub fn set_wrap_mode(&mut self, u_wrap: TextureWrapMode, v_wrap: TextureWrapMode) {
        self.u_wrap = u_wrap;
        self.v_wrap = v_wrap;
    }
}