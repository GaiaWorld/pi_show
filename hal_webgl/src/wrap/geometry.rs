use std::sync::{Arc};
use hal_core::{AttributeName, Context, Geometry};
use wrap::context::{WebGLContextWrap};
use implement::{WebGLGeometryImpl};

pub struct WebGLGeometryWrap {
}

impl Geometry for WebGLGeometryWrap {

    type RContext = WebGLContextWrap;

    fn new(context: &Arc<Self::RContext>) -> Result<<Self::RContext as Context>::ContextGeometry, String> {
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

    fn set_attribute(&self, name: &AttributeName, buffer: &<Self::RContext as Context>::ContextBuffer, offset: usize, count: usize, stride: usize) -> Result<(), String> {
        Err("not implmentation".to_string())
    }
     
    fn remove_attribute(&self, name: &AttributeName) {

    }

    fn set_indices_short(&self, buffer: &<Self::RContext as Context>::ContextBuffer, offset: usize, count: usize) -> Result<(), String> {
        Err("not implmentation".to_string())
    }

    fn remove_indices(&self) {

    }
}

impl Clone for WebGLGeometryWrap {
    fn clone(&self) -> Self {
        Self {
            
        }
    }
}