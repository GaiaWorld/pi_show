use std::sync::{Arc};
use hal_core::{Context, Buffer, BufferData, BufferType};
use wrap::context::{WebGLContextWrap};
use implement::{WebGLBufferImpl};

pub struct WebGLBufferWrap {

}

impl Buffer for WebGLBufferWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Arc<Self::RContext>, btype: BufferType, data: Option<BufferData>, is_updatable: bool) -> Result<<Self::RContext as Context>::ContextBuffer, String> {
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

impl Clone for WebGLBufferWrap {
    fn clone(&self) -> Self {
        Self {

        }
    }
}