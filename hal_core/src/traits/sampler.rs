
use common::{TextureFilterMode, TextureWrapMode};

pub struct SamplerDesc {
    pub min_filter: TextureFilterMode,
    pub mag_filter: TextureFilterMode,
    pub mip_filter: Option<TextureFilterMode>,

    pub u_wrap: TextureWrapMode,
    pub v_wrap: TextureWrapMode,
}

pub trait Sampler: Drop + AsRef<Self> {
}

impl Default for SamplerDesc {
    fn default() -> Self {
        SamplerDesc {
            min_filter: TextureFilterMode::Linear,
            mag_filter: TextureFilterMode::Linear,
            mip_filter: None,

            u_wrap: TextureWrapMode::Repeat,
            v_wrap: TextureWrapMode::Repeat,
        }
    }
}