use hal_core::{Context, BlendState, DepthState, RasterState, StencilState, BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlot, convert_to_mut};
use implement::{WebGLBlendStateImpl, WebGLDepthStateImpl, WebGLRasterStateImpl, WebGLStencilStateImpl};

#[derive(Clone)]
pub struct WebGLBlendStateWrap(pub GLSlot<WebGLBlendStateImpl>);

impl BlendState for WebGLBlendStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &BlendStateDesc) -> Result<<Self::RContext as Context>::ContextBlendState, String> {
        let rimpl = WebGLBlendStateImpl(desc.clone());
        Ok(Self(GLSlot::new(&context.blend_state, rimpl)))
    }
    
    fn delete(&self) {
    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    fn get_desc(&self) -> &BlendStateDesc {
        let s = self.0.slab.get(self.0.index).unwrap();
        &s.0
    }
}

// ================================== 

#[derive(Clone)]
pub struct WebGLDepthStateWrap(pub GLSlot<WebGLDepthStateImpl>);

impl DepthState for WebGLDepthStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &DepthStateDesc) -> Result<<Self::RContext as Context>::ContextDepthState, String> {
       let rimpl = WebGLDepthStateImpl(desc.clone());
       Ok(Self(GLSlot::new(&context.depth_state, rimpl)))
    }
    
    fn delete(&self) {
    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    fn get_desc(&self) -> &DepthStateDesc {
        let s = self.0.slab.get(self.0.index).unwrap();
        &s.0
    }
}

// ================================== 

#[derive(Clone)]
pub struct WebGLRasterStateWrap(pub GLSlot<WebGLRasterStateImpl>);

impl RasterState for WebGLRasterStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &RasterStateDesc) -> Result<<Self::RContext as Context>::ContextRasterState, String> {
        let rimpl = WebGLRasterStateImpl(desc.clone());
        Ok(Self(GLSlot::new(&context.raster_state, rimpl)))
    }
    
    fn delete(&self) {
    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    fn get_desc(&self) -> &RasterStateDesc {
        let s = self.0.slab.get(self.0.index).unwrap();
        &s.0
    }
}

// ================================== 

#[derive(Clone)]
pub struct WebGLStencilStateWrap(pub GLSlot<WebGLStencilStateImpl>);

impl StencilState for WebGLStencilStateWrap {
    
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, desc: &StencilStateDesc) -> Result<<Self::RContext as Context>::ContextStencilState, String> {
        let rimpl = WebGLStencilStateImpl(desc.clone());
        Ok(Self(GLSlot::new(&context.stencil_state, rimpl)))
    }
    
    fn delete(&self) {
    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    fn get_desc(&self) -> &StencilStateDesc {
        let s = self.0.slab.get(self.0.index).unwrap();
        &s.0
    }
}