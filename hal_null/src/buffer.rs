use hal_core::{BufferData, BufferType};
use web_sys::{WebGlBuffer, WebGlRenderingContext};
use js_sys::{Float32Array, Uint16Array, Object};

pub struct WebGLBufferImpl {
    pub size: usize,        // buffer的字节数
    pub btype: BufferType,  // 类型
    pub is_updatable: bool, // 是否更新
    pub handle: WebGlBuffer,
}

impl WebGLBufferImpl {
    pub fn new(
        gl: &WebGlRenderingContext,
        btype: BufferType,
        count: usize,
        data: Option<BufferData>,
        is_updatable: bool,
    ) -> Result<WebGLBufferImpl, String> {
        let usage = if is_updatable {
            WebGlRenderingContext::DYNAMIC_DRAW
        } else {
            WebGlRenderingContext::STATIC_DRAW
        };

        let t = if btype == BufferType::Attribute {
            WebGlRenderingContext::ARRAY_BUFFER
        } else {
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER
        };

        let buffer = gl.create_buffer();
        if buffer.is_none() {
            return Err("WebGLBufferImpl new failed".to_string());
        }
        let buffer = buffer.unwrap();
        gl.bind_buffer(t, Some(&buffer));

        let size = match btype {
            BufferType::Attribute => 4 * count,
            BufferType::Indices => 2 * count,
        };

        match &data {
            Some(BufferData::Float(v)) => {
                debug_assert!(
                    btype == BufferType::Attribute && v.len() == count,
                    "WebGLBufferImpl new failed, invalid float btype, len: {}, count: {}",
                    v.len(),
                    count
                );

                // let b = unsafe { UnsafeTypedArray::new(v) };
                // js! {
                //     @{gl.as_ref()}.bufferData(@{t}, @{b}, @{usage});
				// }
				gl.buffer_data_with_array_buffer_view(t, &Object::from(unsafe{Float32Array::view(v)}) , usage);
            }
            Some(BufferData::Short(v)) => {
                debug_assert!(
                    btype == BufferType::Indices && v.len() == count,
                    format!(
                        "WebGLBufferImpl new failed, invalid short btype, len: {}, count: {}",
                        v.len(),
                        count
                    )
                );
				gl.buffer_data_with_array_buffer_view(t, &Object::from(unsafe{Uint16Array::view(v)}), usage);
                // let b = unsafe { UnsafeTypedArray::new(v) };
                // js! {
                //     @{gl.as_ref()}.bufferData(@{t}, @{b}, @{usage});
                // }
            }
            None => {
				gl.buffer_data_with_i32(t, size as i32, usage);
            }
        };

        Ok(Self {
            btype: btype,
            size: size,
            is_updatable: is_updatable,
            handle: buffer,
        })
    }

    pub fn delete(&self, gl: &WebGlRenderingContext) {
        gl.delete_buffer(Some(&self.handle));
    }

    pub fn update(&mut self, gl: &WebGlRenderingContext, offset: usize, data: BufferData) {
        let t = if self.btype == BufferType::Attribute {
            WebGlRenderingContext::ARRAY_BUFFER
        } else {
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER
        };

        gl.bind_buffer(t, Some(&self.handle));
        match data {
            BufferData::Float(v) => {
                let offset = 4 * offset;
                debug_assert!(self.is_updatable, format!("WebGLBufferImpl update failed, is_updatable: {}, offset: {}, size: {}, len: {}", self.is_updatable, offset, self.size, v.len()));

                let buffer = Object::from(unsafe{Float32Array::view(v)});

                if offset < self.size && offset + 4 * v.len() <= self.size {
					gl.buffer_sub_data_with_i32_and_array_buffer_view(t, offset as i32, &buffer);
                    // js! {
                    //     @{gl.as_ref()}.bufferSubData(@{t}, @{offset as u32}, @{buffer});
                    // }
                } else if offset == 0 {
                    let usage = WebGlRenderingContext::DYNAMIC_DRAW;
                    // js! {
                    //     @{gl.as_ref()}.bufferData(@{t}, @{buffer}, @{usage});
					// }
					gl.buffer_data_with_array_buffer_view(t, &buffer, usage);
                    self.size = 4 * v.len();
                } else {
                    debug_assert!(false, format!("WebGLBufferImpl update failed, is_updatable: {}, offset: {}, size: {}, len: {}", self.is_updatable, offset, self.size, v.len()));
                }
            }
            BufferData::Short(v) => {
                let offset = 2 * offset;
                debug_assert!(self.is_updatable, format!("WebGLBufferImpl update failed, is_updatable: {}, offset: {}, size: {}, len: {}", self.is_updatable, offset, self.size, v.len()));

				// let buffer = unsafe { UnsafeTypedArray::new(v) };
				let buffer = Object::from(unsafe{Uint16Array::view(v)});

                println!(
                    "offset:{}, size:{}, len:{}",
                    offset,
                    self.size,
                    offset + 2 * v.len()
                );
                if offset < self.size && offset + 2 * v.len() <= self.size {
                    // js! {
                    //     @{gl.as_ref()}.bufferSubData(@{t}, @{offset as u32}, @{buffer});
					// }
					gl.buffer_sub_data_with_i32_and_array_buffer_view(t, offset as i32, &buffer);
                } else if offset == 0 {
                    let usage = WebGlRenderingContext::DYNAMIC_DRAW;
                    // js! {
                    //     @{gl.as_ref()}.bufferData(@{t}, @{buffer}, @{usage});
					// }
					gl.buffer_data_with_array_buffer_view(t, &buffer, usage);
                    self.size = 2 * v.len();
                } else {
                    debug_assert!(false, format!("WebGLBufferImpl update failed, is_updatable: {}, offset: {}, size: {}, len: {}", self.is_updatable, offset, self.size, v.len()));
                }
            }
        }
    }
}
