use hal_core::{BufferData, BufferType};
use share::{Share};
use implement::context::{WebGLContextImpl};
use stdweb::{UnsafeTypedArray};
use webgl_rendering_context::{WebGLBuffer, WebGLRenderingContext};

pub struct WebGLBufferImpl {
    context: Share<WebGLContextImpl>,
    
    pub size: usize,        // buffer的字节数
    pub btype: BufferType,  // 类型
    pub is_updatable: bool, // 是否更新
    pub handle: WebGLBuffer,
}

impl WebGLBufferImpl {

    pub fn new(context: &Share<WebGLContextImpl>, btype: BufferType, count: usize, data: Option<BufferData>, is_updatable: bool) -> Result<WebGLBufferImpl, String> {
        let usage = if is_updatable { WebGLRenderingContext::DYNAMIC_DRAW } else { WebGLRenderingContext::STATIC_DRAW };
        
        let t = if btype == BufferType::Attribute { WebGLRenderingContext::ARRAY_BUFFER } else { WebGLRenderingContext::ELEMENT_ARRAY_BUFFER };

        let buffer = context.context.create_buffer();
        if buffer.is_none() {
            return Err("WebGLBufferImpl new failed".to_string());
        }
        let buffer = buffer.unwrap();
        context.context.bind_buffer(t, Some(&buffer));
        
        let size = match btype {
            BufferType::Attribute => 4 * count,
            BufferType::Indices => 2 * count,
        };

        match &data {
            Some(BufferData::Float(v)) => {
                debug_assert!(btype == BufferType::Attribute && v.len() == count, "WebGLBufferImpl new failed, invalid float btype");
                
                let b = unsafe { UnsafeTypedArray::new(v) };
                js! {
                    @{context.context.as_ref()}.bufferData(@{t}, @{b}, @{usage});
                }
            },
            Some(BufferData::Short(v)) => {
                debug_assert!(btype == BufferType::Attribute && v.len() == count, "WebGLBufferImpl new failed, invalid short btype");

                let b = unsafe { UnsafeTypedArray::new(v) };
                js! {
                    @{context.context.as_ref()}.bufferData(@{t}, @{b}, @{usage});
                }
            },
            None => {
                context.context.buffer_data(t, size as i64, usage);
            }
        };

        Ok(Self {
            context: context.clone(),
            btype: btype,
            size: size,
            is_updatable: is_updatable,
            handle: buffer,
        })
    }

    pub fn delete(&self) {
        self.context.context.delete_buffer(Some(&self.handle));
    }

    pub fn update(&mut self, offset: usize, data: BufferData) {
        
        let t = if self.btype == BufferType::Attribute { WebGLRenderingContext::ARRAY_BUFFER } else { WebGLRenderingContext::ELEMENT_ARRAY_BUFFER };

        self.context.context.bind_buffer(t, Some(&self.handle));
        match data {
            BufferData::Float(v) => {
                let offset = 4 * offset;
                debug_assert!(self.is_updatable && offset < self.size && offset + 4 * v.len() < self.size, "WebGLBufferImpl update failed");

                let buffer = unsafe { UnsafeTypedArray::new(v) };
                js! {
                    @{self.context.context.as_ref()}.bufferSubData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{offset as u32}, @{buffer});
                }
            }
            BufferData::Short(v) => {
                let offset = 2 * offset;
                debug_assert!(self.is_updatable && offset < self.size && offset + 2 * v.len() < self.size, "WebGLBufferImpl update failed");

                let buffer = unsafe { UnsafeTypedArray::new(v) };
                
                js! {
                    @{self.context.context.as_ref()}.bufferSubData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{offset as u32}, @{buffer});
                }
            }
        }
    }
}