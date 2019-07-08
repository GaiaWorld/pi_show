use hal_core::{BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};

pub struct WebGLBlendStateImpl(pub BlendStateDesc);

pub struct WebGLDepthStateImpl(pub DepthStateDesc);

pub struct WebGLRasterStateImpl(pub RasterStateDesc);

pub struct WebGLStencilStateImpl(pub StencilStateDesc);

impl WebGLBlendStateImpl {
    pub fn new(desc: &BlendStateDesc) -> Self {
        Self(desc.clone())
    }
}

impl WebGLDepthStateImpl {
    pub fn new(desc: &DepthStateDesc) -> Self {
        Self(desc.clone())
    }
}

impl WebGLRasterStateImpl {
    pub fn new(desc: &RasterStateDesc) -> Self {
        Self(desc.clone())
    }
}

impl WebGLStencilStateImpl {
    pub fn new(desc: &StencilStateDesc) -> Self {
        Self(desc.clone())
    }
}