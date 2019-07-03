use hal_core::{AttributeName};
use share::{Share};
use implement::context::{WebGLContextImpl}; 
use implement::buffer::{WebGLBufferImpl};

pub struct WebGLGeometryImpl {
    context: Share<WebGLContextImpl>,
}

impl WebGLGeometryImpl  {

    pub fn new(context: &Share<WebGLContextImpl>) -> Result<WebGLGeometryImpl, String> {
        Err("not implmentation".to_string())
    }

    pub fn delete(&mut self) {

    }

    pub fn get_vertex_count(&self) -> u32 {
        0
    }

    pub fn set_vertex_count(&mut self, count: u32) {

    }

    pub fn set_attribute(&mut self, name: &AttributeName, buffer: &WebGLBufferImpl) -> Result<(), String> {
        Err("not implmentation".to_string())
    }
    
    pub fn set_attribute_with_offset(&self, name: &AttributeName, buffer: &WebGLBufferImpl, offset: usize, count: usize, stride: usize) -> Result<(), String> {
        Err("not implmentation".to_string())
    }

    pub fn remove_attribute(&mut self, name: &AttributeName) {

    }

    pub fn set_indices_short(&mut self, buffer: &WebGLBufferImpl) -> Result<(), String> {
        Err("not implmentation".to_string())
    }

    pub fn set_indices_short_with_offset(&self, buffer: &WebGLBufferImpl, offset: usize, count: usize) -> Result<(), String> {
        Err("not implmentation".to_string())
    }
    
    pub fn remove_indices(&mut self) {

    }
}