use hal_core::{Context, Buffer, BufferData, BufferType};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlab, GLSlot, convert_to_mut};
use implement::{WebGLBufferImpl};

#[derive(Clone)]
pub struct WebGLBufferWrap(pub GLSlot);

impl Buffer for WebGLBufferWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, btype: BufferType, data: Option<BufferData>, is_updatable: bool) -> Result<<Self::RContext as Context>::ContextBuffer, String> {
        match WebGLBufferImpl::new(context, btype, data, is_updatable) {
            Err(s) => Err(s),
            Ok(buffer) => {
                let slab = convert_to_mut(&context.slabs.buffer);
                let slot = GLSlab::new_slice(context, slab, buffer);
                Ok(Self(slot))
            }
        }
    }

    fn delete(&self) {
        let slab = convert_to_mut(&self.0.context.slabs.buffer);
        let mut buffer = GLSlab::delete_slice(slab, &self.0);
        buffer.delete();
    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    fn update(&self, offset: usize, data: BufferData) {
        let slab = convert_to_mut(&self.0.context.slabs.buffer);
        match GLSlab::get_mut_slice(slab, &self.0) {
            None => {},
            Some(buffer) => buffer.update(offset, data),
        }
    }
}