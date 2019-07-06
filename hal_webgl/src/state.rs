use hal_core::{BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};

pub struct WebGLBlendStateImpl(pub BlendStateDesc);

pub struct WebGLDepthStateImpl(pub DepthStateDesc);

pub struct WebGLRasterStateImpl(pub RasterStateDesc);

pub struct WebGLStencilStateImpl(pub StencilStateDesc);