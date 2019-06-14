use std::sync::{Arc};
use hal_core::{Context, Buffer, BufferData, BufferType};
use wrap::{WebGLContextWrap};

pub struct WebGLBufferImpl {

}

impl WebGLBufferImpl {
    fn new(context: &Arc<WebGLContextWrap>, btype: BufferType, data: Option<BufferData>, is_updatable: bool) -> Result<Self, String> {
        Err("not implmentation".to_string())
    }

    fn delete(&self) {

    }

    fn get_id(&self) -> u64 {
        0
    }

    fn update(&self, offset: usize, data: BufferData) {

    }
}