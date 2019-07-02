use hal_core::{Context, Buffer, BufferData, BufferType};
use wrap::{WebGLContextWrap};

pub struct WebGLBufferImpl {

}

impl WebGLBufferImpl {
    pub fn new(context: &WebGLContextWrap, btype: BufferType, data: Option<BufferData>, is_updatable: bool) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }

    pub fn delete(&mut self) {

    }

    pub fn update(&mut self, offset: usize, data: BufferData) {

    }
}