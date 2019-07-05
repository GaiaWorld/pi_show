use hal_core::{BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};
use share::{Share};
use implement::context::{WebGLContextImpl}; 

pub struct WebGLBlendStateImpl {
    context: Share<WebGLContextImpl>,
    desc: BlendStateDesc,
}

impl WebGLBlendStateImpl {
    
    pub fn new(context: &Share<WebGLContextImpl>, desc: &BlendStateDesc) -> Result<Self, String> {
        Ok(Self {
            context: context.clone(),
            desc: desc.clone(),
        })
    }
    
    pub fn delete(&mut self) {

    }
}

// ================================== 

pub struct WebGLDepthStateImpl {
    context: Share<WebGLContextImpl>,
    desc: DepthStateDesc,
}

impl WebGLDepthStateImpl {

    pub fn new(context: &Share<WebGLContextImpl>, desc: &DepthStateDesc) -> Result<Self, String> {
        Ok(Self {
            context: context.clone(),
            desc: desc.clone(),
        })
    }
    
    pub fn delete(&mut self) {

    }
}

// ================================== 

pub struct WebGLRasterStateImpl {
    context: Share<WebGLContextImpl>,
    desc: RasterStateDesc,
}

impl WebGLRasterStateImpl {
    
    pub fn new(context: &Share<WebGLContextImpl>, desc: &RasterStateDesc) -> Result<Self, String> {
        Ok(Self {
            context: context.clone(),
            desc: desc.clone(),
        })
    }
    
    pub fn delete(&mut self) {

    }
}

// ================================== 

pub struct WebGLStencilStateImpl {
    context: Share<WebGLContextImpl>,
    desc: StencilStateDesc,
}

impl WebGLStencilStateImpl {

    pub fn new(context: &Share<WebGLContextImpl>, desc: &StencilStateDesc) -> Result<Self, String> {
        Ok(Self {
            context: context.clone(),
            desc: desc.clone(),
        })
    }
    
    pub fn delete(&mut self) {

    }
}