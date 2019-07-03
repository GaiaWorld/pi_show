use hal_core::{BufferData, BufferType};
use share::{Share};
use implement::context::{WebGLContextImpl};

pub struct WebGLBufferImpl {
    context: Share<WebGLContextImpl>,
}

impl WebGLBufferImpl {
    pub fn new(context: &Share<WebGLContextImpl>, btype: BufferType, data: Option<BufferData>, is_updatable: bool) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }

    pub fn delete(&mut self) {

    }

    pub fn update(&mut self, offset: usize, data: BufferData) {

    }
}