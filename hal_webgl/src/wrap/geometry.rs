use hal_core::{AttributeName, Context, Geometry};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlot};
use implement::{WebGLGeometryImpl};

#[derive(Clone)]
pub struct WebGLGeometryWrap(GLSlot);

impl Geometry for WebGLGeometryWrap {

    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext) -> Result<<Self::RContext as Context>::ContextGeometry, String> {
        Err("not implmentation".to_string())
    }

    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn get_vertex_count(&self) -> u32 {
        0
    }

    fn set_vertex_count(&self, count: u32) {

    }

    fn set_attribute_with_offset(&self, name: &AttributeName, buffer: &<Self::RContext as Context>::ContextBuffer, offset: usize, count: usize, stride: usize) -> Result<(), String> {
        Err("not implmentation".to_string())
    }

    fn set_attribute(&self, name: &AttributeName, buffer: &<Self::RContext as Context>::ContextBuffer) -> Result<(), String> {
        Err("not implmentation".to_string())
    }

    fn remove_attribute(&self, name: &AttributeName) {

    }

    fn set_indices_short(&self, buffer: &<Self::RContext as Context>::ContextBuffer) -> Result<(), String> {
        Err("not implmentation".to_string())
    }

    fn set_indices_short_with_offset(&self, buffer: &<Self::RContext as Context>::ContextBuffer, offset: usize, count: usize) -> Result<(), String> {
        Err("not implmentation".to_string())
    }
    
    fn remove_indices(&self) {

    }
}