use hal_core::{Context, Buffer, BufferData, BufferType};
use wrap::{WebGLContextWrap};

pub struct WebGLBufferImpl {

}

impl WebGLBufferImpl {
    pub fn new(context: &WebGLContextWrap, btype: BufferType, data: Option<BufferData>, is_updatable: bool) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }

    pub fn delete(&self) {

    }

    pub fn get_id(&self) -> u64 {
        0
    }

    pub fn update(&self, offset: usize, data: BufferData) {

    }
}