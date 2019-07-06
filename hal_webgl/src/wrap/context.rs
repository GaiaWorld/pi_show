use slab::{Slab};
use share::{Share};
use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::{Object};

use hal_core::{Context, ProgramParamter, Capabilities, RenderBeginDesc};
use wrap::buffer::{WebGLBufferWrap};
use wrap::geometry::{WebGLGeometryWrap};
use wrap::program::{WebGLProgramWrap};
use wrap::render_target::{WebGLRenderTargetWrap, WebGLRenderBufferWrap};
use wrap::sampler::{WebGLSamplerWrap};
use wrap::state::{WebGLRasterStateWrap, WebGLDepthStateWrap, WebGLStencilStateWrap, WebGLBlendStateWrap};
use wrap::texture::{WebGLTextureWrap};

use wrap::gl_slab::{GLSlot, convert_to_mut};
use implement::{
    WebGLBufferImpl, 
    WebGLContextImpl,
    WebGLGeometryImpl, 
    WebGLProgramImpl,
    WebGLRenderTargetImpl, WebGLRenderBufferImpl,
    WebGLSamplerImpl,
    WebGLRasterStateImpl, WebGLDepthStateImpl, WebGLStencilStateImpl, WebGLBlendStateImpl,
    WebGLTextureImpl,
};

#[derive(Clone)]
pub struct WebGLContextWrap {
    pub rimpl: Share<WebGLContextImpl>,
    pub default_rt: WebGLRenderTargetWrap,

    pub buffer: Share<Slab<WebGLBufferImpl>>,
    pub geometry: Share<Slab<WebGLGeometryImpl>>,
    pub texture: Share<Slab<WebGLTextureImpl>>,
    pub sampler: Share<Slab<WebGLSamplerImpl>>,
    pub render_target: Share<Slab<WebGLRenderTargetImpl>>,
    pub render_buffer: Share<Slab<WebGLRenderBufferImpl>>,
    pub blend_state: Share<Slab<WebGLBlendStateImpl>>,
    pub depth_state: Share<Slab<WebGLDepthStateImpl>>,
    pub raster_state: Share<Slab<WebGLRasterStateImpl>>,
    pub stencil_state: Share<Slab<WebGLStencilStateImpl>>,
    pub program: Share<Slab<WebGLProgramImpl>>,
}

impl Context for WebGLContextWrap {
    type ContextSelf = WebGLContextWrap;
    
    type ContextBuffer = WebGLBufferWrap;
    type ContextGeometry = WebGLGeometryWrap;
    type ContextTexture = WebGLTextureWrap;
    type ContextSampler = WebGLSamplerWrap;
    type ContextRenderTarget = WebGLRenderTargetWrap;
    type ContextRenderBuffer = WebGLRenderBufferWrap;
    type ContextBlendState = WebGLBlendStateWrap;
    type ContextDepthState = WebGLDepthStateWrap;
    type ContextRasterState = WebGLRasterStateWrap;
    type ContextStencilState = WebGLStencilStateWrap;
    type ContextProgram = WebGLProgramWrap;

    fn get_caps(&self) -> &Capabilities {
        &self.rimpl.caps
    }

    fn get_default_target(&self) -> &Self::ContextRenderTarget {
        &self.default_rt
    }

    fn set_shader_code<C: AsRef<str>>(&self, name: &str, code: &C) {
        let context = convert_to_mut(self.rimpl.as_ref());
        context.set_shader_code(name, code)
    }

    fn restore_state(&self) {
        let context = convert_to_mut(self.rimpl.as_ref());
        context.restore_state()
    }

    fn begin_render(&self, render_target: &Self::ContextRenderTarget, data: &RenderBeginDesc) {
        let rt = render_target.slot.get_mut();
        debug_assert!(rt.is_some(), "begin_render failed, rt can't found");
        let rt = rt.unwrap();
        let context = convert_to_mut(self.rimpl.as_ref());
        context.begin_render(rt, data)
    }

    fn end_render(&self) {
        let context = convert_to_mut(self.rimpl.as_ref());
        context.end_render();
    }

    fn set_program(&self, program: &Self::ContextProgram) {
        let program = program.0.get_mut();
        debug_assert!(program.is_some(), "set_program failed, program can't found");
        let program = program.unwrap();
        
        let context = convert_to_mut(self.rimpl.as_ref());
        context.set_program(program)
    }

    fn set_state(&self, bs: &Self::ContextBlendState, ds: &Self::ContextDepthState, rs: &Self::ContextRasterState, ss: &Self::ContextStencilState) {
        let bs = bs.0.get_mut();
        debug_assert!(bs.is_some(), "set_state failed, bs can't found");
        let bs = bs.unwrap();
        
        let ds = ds.0.get_mut();
        debug_assert!(ds.is_some(), "set_state failed, ds can't found");
        let ds = ds.unwrap();

        let ss = ss.0.get_mut();
        debug_assert!(ss.is_some(), "set_state failed, ss can't found");
        let ss = ss.unwrap();

        let rs = rs.0.get_mut();
        debug_assert!(rs.is_some(), "set_state failed, rs can't found");
        let rs = rs.unwrap();

        let context = convert_to_mut(self.rimpl.as_ref());
        context.set_state(bs, ds, rs, ss)
    }

    fn draw(&self, geometry: &Self::ContextGeometry, parameter: &Share<dyn ProgramParamter<Self::ContextSelf>>) {
        let geometry = geometry.0.get_mut();
        debug_assert!(geometry.is_some(), "draw failed, geometry can't found");
        let geometry = geometry.unwrap();

        let context = convert_to_mut(self.rimpl.as_ref());
        context.draw(geometry, parameter)
    }
}

impl WebGLContextWrap {
    pub fn new(context: WebGLRenderingContext, fbo: Option<Object>) -> Self {
        let buffer = Share::new(Slab::new());
        let geometry = Share::new(Slab::new());
        let texture = Share::new(Slab::new());
        let sampler = Share::new(Slab::new());
        let render_target = Share::new(Slab::new());
        let render_buffer = Share::new(Slab::new());
        let blend_state = Share::new(Slab::new());
        let depth_state = Share::new(Slab::new());
        let raster_state = Share::new(Slab::new());
        let stencil_state = Share::new(Slab::new());
        let program = Share::new(Slab::new());
        
        let rimpl = WebGLContextImpl::new(context);
        
        let default_rt = WebGLRenderTargetImpl::new_default(&rimpl, fbo, 0, 0);
        let default_rt = GLSlot::new(&render_target, default_rt);
        let default_rt = WebGLRenderTargetWrap::new_default(default_rt);
        
        Self {
            
            rimpl: rimpl,
            default_rt: default_rt,

            buffer: buffer,
            geometry: geometry,
            texture: texture,
            sampler: sampler,
            render_buffer: render_buffer,
            render_target: render_target,
            blend_state: blend_state,
            depth_state: depth_state,
            raster_state: raster_state,
            stencil_state: stencil_state,
            program: program,
        }
    }
}

