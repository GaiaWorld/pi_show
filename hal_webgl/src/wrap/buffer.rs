use std::sync::{Arc};
use hal_core::{Context, Buffer, BufferData, BufferType};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlot};
use implement::{WebGLBufferImpl};

#[derive(Clone)]
pub struct WebGLBufferWrap(GLSlot);

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