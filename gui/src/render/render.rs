use webgl_rendering_context::{WebGLRenderingContext};

use render::vector_sdf::VectorSdf;
use world_doc::component::viewport::*;

pub struct Render{
    pub sdf: VectorSdf,
    //ext: 
    // image:
    pub view_port: ViewPort,
    pub view_port_dirty: bool,
    pub gl: WebGLRenderingContext,
}

impl Render {
    pub fn new (gl: WebGLRenderingContext) -> Self{
        Render {
            sdf: VectorSdf::new(),
            gl: gl,
            view_port: ViewPort {
                width: 10.0,
                height: 10.0,
            },
            view_port_dirty: true,
        }
    }

    pub fn set_view_port (&mut self, width: f32, height: f32) {
        self.view_port.width = width;
        self.view_port.height = height;
        self.view_port_dirty = true;
    }

    pub fn render(&self){
        
    }
}