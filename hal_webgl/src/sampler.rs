use hal_core::*;

#[derive(Debug)]
pub struct WebGLSamplerImpl {
    pub min_filter: TextureFilterMode,
    pub mag_filter: TextureFilterMode,
    pub mip_filter: Option<TextureFilterMode>,

    pub u_wrap: TextureWrapMode,
    pub v_wrap: TextureWrapMode,
}

impl Sampler for WebGLSamplerImpl {
}

impl Drop for WebGLSamplerImpl {
    fn drop(&mut self) {
        println!("================= WebGLSamplerImpl Drop");
    }
}

impl AsRef<Self> for WebGLSamplerImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl WebGLSamplerImpl {
    pub fn new() -> Self {
        WebGLSamplerImpl {
            min_filter: TextureFilterMode::Linear,
            mag_filter: TextureFilterMode::Linear,
            mip_filter: None,

            u_wrap: TextureWrapMode::Repeat,
            v_wrap: TextureWrapMode::Repeat,
        }
    }
}