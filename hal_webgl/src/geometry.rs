use hal_core::{AttributeName, HalBuffer};
use stdweb::{Object};
use stdweb::unstable::TryInto;
use webgl_rendering_context::{WebGLRenderingContext};
use buffer::{WebGLBufferImpl};
use convert::{get_attribute_location};

pub struct Attribute {
    pub offset: usize,      // handle的元素的索引
    pub count: usize,       // 元素的个数
    pub item_count: usize,  // 每个元素的个数
    pub stride: usize,      // 偏移
    pub handle: HalBuffer,
}

pub struct Indices {
    pub offset: usize, 
    pub count: usize,
    pub handle: HalBuffer,
}

pub struct WebGLGeometryImpl {
    pub vertex_count: u32,
    pub attributes: [Option<Attribute>; 16],  // 最多16个Attribute
    pub indices: Option<Indices>,
    
    pub vao: Option<Object>,
}

impl WebGLGeometryImpl  {

    pub fn new(gl: &WebGLRenderingContext, vao_extension: &Option<Object>) -> Result<WebGLGeometryImpl, String> {
         let vao = match vao_extension {
            None => None,
            Some(extension) => {
                match TryInto::<Object>::try_into(js! {
                    var vao = @{extension.as_ref()}.wrap.createVertexArrayOES();
                    // 因为小游戏的WebGL*不是Object，所以要包装一层
                    var vaoWrap = {
                        wrap: vao
                    };
                    return vaoWrap;
                }) {
                    Ok(object) => Some(object),
                    Err(_) => None,
                }
            }
        };

        let attributes = [
            None, None, None, None, 
            None, None, None, None, 
            None, None, None, None, 
            None, None, None, None
        ];

        Ok(Self {
            vertex_count: 0,
            attributes: attributes,
            indices: None,
            vao: vao,
        })
    }

    pub fn delete(&self, vao_extension: &Option<Object>) {
        if let Some(vao) = &self.vao {
            let extension = vao_extension.as_ref().unwrap().as_ref();
            js! {
                @{&extension}.wrap.deleteVertexArrayOES(@{vao}.wrap);
            }
        }
    }

    pub fn get_vertex_count(&self) -> u32 {
        self.vertex_count
    }

    pub fn set_vertex_count(&mut self, count: u32) {
        self.vertex_count = count;
    }
    pub fn set_attribute(&mut self, gl: &WebGLRenderingContext, vao_extension: &Option<Object>, name: &AttributeName, buffer: &WebGLBufferImpl, wrap: &HalBuffer, item_count: usize) -> Result<(), String> {
        let count = buffer.size / 4;
        self.set_attribute_with_offset(gl, vao_extension, name, buffer, wrap, item_count, 0, count, 0)
    }
    
    pub fn set_attribute_with_offset(&mut self, gl: &WebGLRenderingContext, vao_extension: &Option<Object>, name: &AttributeName, buffer: &WebGLBufferImpl, wrap: &HalBuffer, item_count: usize, offset: usize, count: usize, stride: usize) -> Result<(), String> {
        let location = get_attribute_location(name);
        self.attributes[location as usize] = Some(Attribute {
            offset: offset,
            count: count,
            item_count: item_count,
            stride: stride,
            handle: wrap.clone(),
        });

         if let Some(vao) = &self.vao {
            let extension = vao_extension.as_ref().unwrap().as_ref();
            js! {
                @{&extension}.wrap.bindVertexArrayOES(@{&vao}.wrap);
            }

            gl.enable_vertex_attrib_array(location as u32);
            gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&buffer.handle));
            gl.vertex_attrib_pointer(location as u32, item_count as i32, WebGLRenderingContext::FLOAT, false, stride as i32, offset as i64);

            js! {
                @{&extension}.wrap.bindVertexArrayOES(null);
            }
        }

        Ok(())
    }

    pub fn remove_attribute(&mut self, gl: &WebGLRenderingContext, vao_extension: &Option<Object>, name: &AttributeName) {
        let location = get_attribute_location(name);
        
        if let Some(vao) = &self.vao {
            let extension = vao_extension.as_ref().unwrap().as_ref();
            js! {
                @{&extension}.wrap.bindVertexArrayOES(@{&vao}.wrap);
            }
            
            gl.disable_vertex_attrib_array(location as u32);

            js! {
                @{&extension}.wrap.bindVertexArrayOES(null);
            }
        }
        
        self.attributes[location as usize] = None;
    }

    pub fn set_indices_short(&mut self, gl: &WebGLRenderingContext, vao_extension: &Option<Object>, buffer: &WebGLBufferImpl, wrap: &HalBuffer) -> Result<(), String> {

        let count = buffer.size / 2;
        self.set_indices_short_with_offset(gl, vao_extension, buffer, wrap, 0, count)
    }

    pub fn set_indices_short_with_offset(&mut self, gl: &WebGLRenderingContext, vao_extension: &Option<Object>, buffer: &WebGLBufferImpl, wrap: &HalBuffer, offset: usize, count: usize) -> Result<(), String> {
        self.indices = Some(Indices {
            offset: offset,
            count: count,
            handle: wrap.clone(),
        });

        if let Some(vao) = &self.vao {
            let extension = vao_extension.as_ref().unwrap().as_ref();
            js! {
                @{&extension}.wrap.bindVertexArrayOES(@{&vao}.wrap);
            }
            
            gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffer.handle));
            
            js! {
                @{&extension}.wrap.bindVertexArrayOES(null);
            }
        }

        Ok(())
    }

    // FIXME, TODO 
    // 注：有文章说，opengl无法在vao移除indices。
    // 出处：http://www.photoneray.com/opengl-vao-vbo/
    pub fn remove_indices(&mut self, gl: &WebGLRenderingContext, vao_extension: &Option<Object>) {
        if let Some(vao) = &self.vao {
            let extension = vao_extension.as_ref().unwrap().as_ref();
            js! {
                @{&extension}.wrap.bindVertexArrayOES(@{&vao}.wrap);
            }
            
            gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER, None);
            
            js! {
                @{&extension}.wrap.bindVertexArrayOES(null);
            }
        }
        self.indices = None;
    }
}