use std::sync::{Arc};
use hal_core::{AttributeName};
use wrap::{WebGLContextWrap};
use implement::buffer::{WebGLBufferImpl};

pub struct WebGLGeometryImpl {
}

impl WebGLGeometryImpl  {

    fn new(context: &Arc<WebGLContextWrap>) -> Result<Self, String> {
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

    fn set_attribute(&self, name: &AttributeName, buffer: &WebGLBufferImpl, offset: usize, count: usize, stride: usize) -> Result<(), String> {
        Err("not implmentation".to_string())
    }
     
    fn remove_attribute(&self, name: &AttributeName) {

    }

    fn set_indices_short(&self, buffer: &WebGLBufferImpl, offset: usize, count: usize) -> Result<(), String> {
        Err("not implmentation".to_string())
    }

    fn remove_indices(&self) {

    }
}