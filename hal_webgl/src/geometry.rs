use std::collections::{HashMap};
use std::sync::{Arc, Weak};
use hal_core::{Geometry, AttributeName};
use webgl_rendering_context::{WebGLRenderingContext, WebGLBuffer};
use stdweb::{UnsafeTypedArray};

#[derive(Debug)]
pub struct Attribute {
    pub size: u32,
    pub item_count: u32,
    pub is_updatable: bool,
    pub buffer: WebGLBuffer,    
}

#[derive(Debug)]
pub struct Indices {
    pub size: u32,
    pub is_updatable: bool,
    pub is_short_type: bool,
    pub buffer: WebGLBuffer,    
}

#[derive(Debug)]
pub struct WebGLGeometryImpl {
    gl: Weak<WebGLRenderingContext>,
    pub vertex_count: u32,
    pub indices: Option<Indices>,
    pub attributes: HashMap<AttributeName, Attribute>,
}

impl WebGLGeometryImpl {
    
    pub fn new(gl: &Arc<WebGLRenderingContext>) -> Self {
        WebGLGeometryImpl {
            gl: Arc::downgrade(gl),
            vertex_count: 0,
            indices: None,
            attributes: HashMap::new(),            
        }
    }
}

impl AsRef<Self> for WebGLGeometryImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl Geometry for WebGLGeometryImpl {

    fn has_attribute(&self, name: &AttributeName) -> bool {
        self.attributes.contains_key(name)
    }
  
    fn get_vertex_count(&self) -> u32 {
        self.vertex_count
    }

    fn set_vertex_count(&mut self, count: u32) {
        self.vertex_count = count;
    }

    fn set_attribute(&mut self, name: &AttributeName, item_count: u32, data: Option<&[f32]>, is_updatable: bool) -> Result<(), String> {
        
        debug_println!("Shader, set_attribute_impl, name = {:?}, data = {:?}", name, data);

        assert!(self.vertex_count > 0 && item_count > 0, "WebGLGeometryImpl set_attribute failed, vertex_count or item_count invalid");
    
        if data.is_some() {
            assert!(self.vertex_count * item_count == data.unwrap().len() as u32, "WebGLGeometryImpl set_attribute failed, data.len invalid");
        }
        
        let gl = self.gl.upgrade();
        let gl: Option<&Arc<WebGLRenderingContext>> = gl.as_ref();
        if gl.is_none() {
            return Err("WebGLGeometryImpl set_attribute failed, gl han't exist".to_string());
        }

        let gl: &WebGLRenderingContext = gl.unwrap().as_ref();

        let is_first = !self.attributes.contains_key(name);
        if is_first {
            match gl.create_buffer() {
                None => {
                    return Err("WebGLGeometryImpl set_attribute failed".to_string());
                }
                Some(buffer) => {
                    self.attributes.insert(name.clone(), Attribute {
                        size: 0,
                        item_count: 0,
                        is_updatable: true,
                        buffer: buffer,
                    });
                }
            }
        }

        let usage = if is_updatable { WebGLRenderingContext::DYNAMIC_DRAW } else { WebGLRenderingContext::STATIC_DRAW };

        let attribute = self.attributes.get_mut(name).unwrap();
        match data {
            Some(data) => {
                gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&attribute.buffer));
                let buffer = unsafe { UnsafeTypedArray::new(data) };
            
                if !is_first && (attribute.is_updatable && is_updatable && data.len() as u32 == (self.vertex_count * item_count)) {
                    // println!("attribute: name = {:?}, size = {:?}, item_count = {:?}", &name, attribute.size, item_count);
                    js! {
                        console.log("bufferSubData = ", @{&buffer});
                        @{gl}.bufferSubData(@{WebGLRenderingContext::ARRAY_BUFFER}, 0, @{buffer});
                    }
                } else { 
                    
                    attribute.item_count = item_count;
                    attribute.is_updatable = is_updatable;
                    attribute.size = 4 * self.vertex_count * item_count;
                    // println!("attribute: name = {:?}, size = {:?}, item_count = {:?}", &name, attribute.size, item_count);
                    js! {
                        console.log("bufferData = ", @{&buffer});
                        @{gl}.bufferData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{buffer}, @{usage});
                    }
                }
            }
            None => {
                if attribute.size != 4 * self.vertex_count * item_count {
                    
                    attribute.item_count = item_count;
                    attribute.is_updatable = is_updatable;
                    attribute.size = 4 * self.vertex_count * item_count;

                    gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&attribute.buffer));
                    
                    gl.buffer_data(WebGLRenderingContext::ARRAY_BUFFER, attribute.size as i64, usage);
                }
            }
        }

        Ok(())
    }
    
    fn remove_attribute(&mut self, name: &AttributeName) {
        match (self.gl.upgrade(), self.attributes.remove(name)) {
            (Some(gl), Some(attribute)) => {
                gl.delete_buffer(Some(&attribute.buffer));
            }
            _ => { }
        }
    }

    fn set_indices_short(&mut self, data: &[u16], is_updatable: bool) -> Result<(), String> {
        debug_println!("Shader, set_indices_short, data = {:?}", data);

        assert!(self.vertex_count > 0 && data.len() > 0, "WebGLGeometryImpl set_indices_short failed, data.len invalid");

        let gl: Option<Arc<WebGLRenderingContext>> = self.gl.upgrade();
        
        if gl.is_none() {
            return Err("WebGLGeometryImpl set_indices_short failed, gl han't exist".to_string());
        }

        let gl: &WebGLRenderingContext = gl.as_ref().unwrap().as_ref();

        if self.indices.is_none() {
            match gl.create_buffer() {
                Some(buffer) => {
                    self.indices = Some(Indices {
                        size: 2 * data.len() as u32,
                        is_short_type: true,
                        buffer: buffer,
                        is_updatable: is_updatable,
                    });
                }
                None => {
                    return Err("WebGLGeometryImpl set_indices_short failed".to_string());
                }
            }
        }

        if let Some(indices) = &mut self.indices {
            gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&indices.buffer));
            
            let buffer = unsafe { UnsafeTypedArray::new(data) };
            
            // 老的索引数据标志可更新，而且长度相等，就直接更新
            if indices.is_updatable && is_updatable && indices.size == 2 * data.len() as u32 {
                js! {
                    @{gl}.bufferSubData(@{WebGLRenderingContext::ELEMENT_ARRAY_BUFFER}, 0, @{buffer});
                }
            } else {
                indices.size = 2 * data.len() as u32;
                indices.is_updatable = is_updatable;
                let usage = if is_updatable { WebGLRenderingContext::DYNAMIC_DRAW } else { WebGLRenderingContext::STATIC_DRAW };

                js! {
                    @{gl}.bufferData(@{WebGLRenderingContext::ELEMENT_ARRAY_BUFFER}, @{buffer}, @{usage});
                }
            }
        }

        Ok(())
    }

    fn remove_indices(&mut self) {
        match (self.gl.upgrade(), self.indices.take()) {
            (Some(gl), Some(indices)) => {
                gl.delete_buffer(Some(&indices.buffer));
            }
            _ => {}
        }
    }

    fn update_attribute(&self, name: &AttributeName, item_offset: u32, data: &[f32]) {
        if let Some(gl) = &self.gl.upgrade() {
            let gl = gl.as_ref();

            match self.attributes.get(name) {
                Some(attribute) if attribute.is_updatable && item_offset < attribute.size && item_offset + (4 * data.len() as u32) <= attribute.size => {
         
                    let item_offset = 4 * item_offset;
                    gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&attribute.buffer));
                    
                    let buffer = unsafe { UnsafeTypedArray::new(data) };
                    js! {
                        @{gl}.bufferSubData(@{WebGLRenderingContext::ELEMENT_ARRAY_BUFFER}, @{item_offset}, @{buffer});
                    }
                }
                _ => assert!(false, "WebGLGeometryImpl update_attribute failed")
            }
        }
    }
}

impl Drop for WebGLGeometryImpl {
    fn drop(&mut self) {
        if let Some(gl) = &self.gl.upgrade() {
            for (_, v) in self.attributes.iter() {
                gl.delete_buffer(Some(&v.buffer));
            }

            if let Some(indices) = &self.indices {
                gl.delete_buffer(Some(&indices.buffer));
            }
        }
    }
}