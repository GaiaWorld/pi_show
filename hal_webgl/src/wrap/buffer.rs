use hal_core::{Context, Buffer, BufferData, BufferType};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlab, GLSlot, convert_to_mut};
use implement::{WebGLBufferImpl};

#[derive(Clone)]
pub struct WebGLBufferWrap(GLSlot);

impl Buffer for WebGLBufferWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, btype: BufferType, data: Option<BufferData>, is_updatable: bool) -> Result<<Self::RContext as Context>::ContextBuffer, String> {
        let context = convert_to_mut(context);

        match WebGLBufferImpl::new(context, btype, data, is_updatable) {
            Err(s) => Err(s),
            Ok(buffer) => {
                let slot = GLSlab::new_slice(context, &context.0.slab.buffer, buffer);
                Ok(Self(slot))
            }
        }
    }

    fn delete(&self) {
        let context = convert_to_mut(&self.0.context);
        let buffer = GLSlab::delete_slice(&context.0.slab.buffer, &self.0);
        buffer.delete();
    }

    fn get_id(&self) -> u64 {
        let context = convert_to_mut(&self.0.context);
        match GLSlab::get_mut_slice(&context.0.slab.buffer, &self.0) {
            None => u64::max_value(),
            Some(buffer) => buffer.get_id()
        }
    }

    fn update(&self, offset: usize, data: BufferData) {
        let context = convert_to_mut(&self.0.context);
        let buffer = GLSlab::get_mut_slice(&context.0.slab.buffer, &self.0);
        match GLSlab::get_mut_slice(&context.0.slab.buffer, &self.0) {
            None => {},
            Some(buffer) => buffer.update(offset, data),
        }
    }
}