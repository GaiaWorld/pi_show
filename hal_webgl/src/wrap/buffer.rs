use hal_core::{Context, Buffer, BufferData, BufferType};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlot, convert_to_mut};
use implement::{WebGLBufferImpl};

#[derive(Clone)]
pub struct WebGLBufferWrap(pub GLSlot<WebGLBufferImpl>);

impl Buffer for WebGLBufferWrap {
    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext, btype: BufferType, count: usize, data: Option<BufferData>, is_updatable: bool) -> Result<<Self::RContext as Context>::ContextBuffer, String> {
        match WebGLBufferImpl::new(&context.rimpl, btype, count, data, is_updatable) {
            Err(s) => Err(s),
            Ok(buffer) => {
                let slot = GLSlot::new(&context.buffer, buffer);
                Ok(Self(slot))
            }
        }
    }

    fn delete(&self) {
        let slab = convert_to_mut(self.0.slab.as_ref());
        let mut buffer = slab.remove(self.0.index);
        buffer.delete();
    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    fn update(&self, offset: usize, data: BufferData) {
        let slab = convert_to_mut(self.0.slab.as_ref());
        match slab.get_mut(self.0.index) {
            None => {},
            Some(buffer) => buffer.update(offset, data),
        }
    }
}