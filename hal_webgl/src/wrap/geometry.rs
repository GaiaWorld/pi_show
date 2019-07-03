use hal_core::{AttributeName, Context, Geometry};
use wrap::context::{WebGLContextWrap};
use wrap::gl_slab::{GLSlot, convert_to_mut};
use implement::{WebGLGeometryImpl};

#[derive(Clone)]
pub struct WebGLGeometryWrap(GLSlot<WebGLGeometryImpl>);

impl Geometry for WebGLGeometryWrap {

    type RContext = WebGLContextWrap;

    fn new(context: &Self::RContext) -> Result<<Self::RContext as Context>::ContextGeometry, String> {
        match WebGLGeometryImpl::new(&context.rimpl) {
            Err(s) => Err(s),
            Ok(geometry) => {
                let slot = GLSlot::new(&context.geometry, geometry);
                Ok(Self(slot))
            }
        }
    }

    fn delete(&self) {
        let slab = convert_to_mut(self.0.slab.as_ref());
        let mut geometry = slab.remove(self.0.index);
        geometry.delete();
    }

    fn get_id(&self) -> u64 {
        self.0.index as u64
    }

    fn get_vertex_count(&self) -> u32 {
        match self.0.slab.get(self.0.index) {
            None => 0,
            Some(geometry) => geometry.get_vertex_count(),
        }
    }

    fn set_vertex_count(&self, count: u32) {
        let slab = convert_to_mut(self.0.slab.as_ref());
        match slab.get_mut(self.0.index) {
            None => {},
            Some(geometry) => geometry.set_vertex_count(count),
        }
    }

    fn set_attribute_with_offset(&self, name: &AttributeName, buffer: &<Self::RContext as Context>::ContextBuffer, offset: usize, count: usize, stride: usize) -> Result<(), String> {
        let gslab = convert_to_mut(self.0.slab.as_ref());
        let bslab = convert_to_mut(buffer.0.slab.as_ref());
        match (gslab.get_mut(self.0.index), bslab.get_mut(buffer.0.index)) {
            (Some(geometry), Some(buffer)) => geometry.set_attribute_with_offset(name, buffer, offset, count, stride),
            _ => Err("not found".to_string()),
        }
    }

    fn set_attribute(&self, name: &AttributeName, buffer: &<Self::RContext as Context>::ContextBuffer) -> Result<(), String> {
        let gslab = convert_to_mut(self.0.slab.as_ref());
        let bslab = convert_to_mut(buffer.0.slab.as_ref());
        match (gslab.get_mut(self.0.index), bslab.get_mut(buffer.0.index)) {
            (Some(geometry), Some(buffer)) => geometry.set_attribute(name, buffer),
            _ => Err("not found".to_string()),
        }
    }

    fn remove_attribute(&self, name: &AttributeName) {
        let slab = convert_to_mut(self.0.slab.as_ref());
        match slab.get_mut(self.0.index) {
            None => {},
            Some(geometry) => geometry.remove_attribute(name),
        }
    }

    fn set_indices_short(&self, buffer: &<Self::RContext as Context>::ContextBuffer) -> Result<(), String> {
        let gslab = convert_to_mut(self.0.slab.as_ref());
        let bslab = convert_to_mut(buffer.0.slab.as_ref());
        match (gslab.get_mut(self.0.index), bslab.get_mut(buffer.0.index)) {
            (Some(geometry), Some(buffer)) => geometry.set_indices_short(buffer),
            _ => Err("not found".to_string()),
        }
    }

    fn set_indices_short_with_offset(&self, buffer: &<Self::RContext as Context>::ContextBuffer, offset: usize, count: usize) -> Result<(), String> {
        let gslab = convert_to_mut(self.0.slab.as_ref());
        let bslab = convert_to_mut(buffer.0.slab.as_ref());
        match (gslab.get_mut(self.0.index), bslab.get_mut(buffer.0.index)) {
            (Some(geometry), Some(buffer)) => geometry.set_indices_short_with_offset(buffer, offset, count),
            _ => Err("not found".to_string()),
        }
    }
    
    fn remove_indices(&self) {
        let slab = convert_to_mut(self.0.slab.as_ref());
        match slab.get_mut(self.0.index) {
            None => {},
            Some(geometry) => geometry.remove_indices(),
        }
    }
}