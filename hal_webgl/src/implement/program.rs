use atom::{Atom};
use hal_core::{ShaderType, UniformLayout, UniformValue, ProgramParamter, AttributeName};
use share::{Share};
use implement::context::{WebGLContextImpl}; 
use webgl_rendering_context::{WebGLProgram, WebGLUniformLocation, WebGLRenderingContext};
use stdweb::unstable::TryInto;

use wrap::{WebGLSamplerWrap, WebGLTextureWrap};

use fnv::FnvHashMap;

pub struct SamplerUniform {
    location: WebGLUniformLocation,
    value: Option<(WebGLSamplerWrap, WebGLTextureWrap)>,
}

pub struct CommonUniform {
    value: UniformValue,
    location: WebGLUniformLocation,
}

pub struct WebGLProgramImpl {
    pub context: Share<WebGLContextImpl>,

    pub handle: WebGLProgram,

    // 这里是所有宏展开后的全局的数据，有很多用不到的，统统填 None
    pub active_uniforms: Vec<(usize, Vec<(usize, CommonUniform)>)>,
    pub active_textures: Vec<(usize, SamplerUniform)>,
    
    pub last_ubos: Option<Share<dyn ProgramParamter<WebGLContextImpl>>>, // 上次设置的Uniforms，对应接口的概念
}

impl WebGLProgramImpl {

    pub fn new_with_vs_fs(context: &Share<WebGLContextImpl>, vs_name: &Atom, vs_defines: &[Atom], fs_name: &Atom, fs_defines: &[Atom], uniform_layout: &UniformLayout) -> Result<WebGLProgramImpl, String> {
        
        let vs = context.shader_cache.unwrap().get_shader(ShaderType::Vertex, vs_name, vs_defines);
        if let Err(e) = &vs {
            return Err(e.clone());
        }
        let vs = vs.unwrap();

        let fs = context.shader_cache.unwrap().get_shader(ShaderType::Fragment, fs_name, fs_defines);
        if let Err(e) = &fs {
            return Err(e.clone());
        }
        let fs = fs.unwrap();

        let gl = &context.context;
        
        // 创建program
        let program_handle = gl.create_program().ok_or_else(|| String::from("unable to create shader object"))?;
        gl.attach_shader(&program_handle, &vs.handle);
        gl.attach_shader(&program_handle, &fs.handle);

        // 先绑定属性，再连接
        let max_attribute_count = std::cmp::min(AttributeName::get_builtin_count(), context.caps.max_vertex_attribs);
        for i in 0..max_attribute_count {
            let (_attrib_name, name) = Self::get_attribute_by_location(i);
            debug_println!("Shader, link_program, attribute name = {:?}, location = {:?}", &name, i);
            gl.bind_attrib_location(&program_handle, i, name);
        }

        // 连接program
        gl.link_program(&program_handle);
        let is_link_ok = gl
            .get_program_parameter(&program_handle, WebGLRenderingContext::LINK_STATUS)
            .try_into()
            .unwrap_or(false);

        // 微信小游戏移动端环境，返回的是1-0，所以需要再来一次
        let is_link_ok = if is_link_ok { is_link_ok } else {
            let r = gl.get_program_parameter(&program_handle, WebGLRenderingContext::LINK_STATUS)
            .try_into()
            .unwrap_or(0);

            r != 0
        };

        if !is_link_ok {
            let e = gl
                .get_program_info_log(&program_handle)
                .unwrap_or_else(|| "unkown link error".into());
            debug_println!("Shader, link_program error, link failed, info = {:?}", &e);
            return Err(e);
        }

        let location_map = context.shader_cache.unwrap().get_location_map(vs_name, fs_name, uniform_layout);
        
        // 初始化attribute和uniform
        match Self::init_uniform(gl, &program_handle, location_map) {
            None => {
                gl.delete_program(Some(&program_handle));
                Err("WebGLProgramImpl failed, invalid uniforms".to_string())
            },
            Some((uniforms, textures)) => {
                Ok(WebGLProgramImpl {
                    context: context.clone(),
                    handle: program_handle,
                    active_uniforms: uniforms,
                    active_textures: textures,
                    last_ubos: None,
                })
            }
        }
    }

    pub fn delete(&self) {
        let gl = &self.context.context;
        gl.delete_program(Some(&self.handle));
    }

    pub fn get_shader_info(&self, stype: ShaderType) -> Option<(&Atom, &[Atom])> {
        None
    }

    ////////////////////////////// 

    pub fn use_me(&mut self) {
        let gl = &self.context.context;
        gl.use_program(Some(&self.handle));
    }

    fn get_attribute_by_location(index: u32) -> (AttributeName, &'static str) {
        match index {
            0 => (AttributeName::Position, "position"),
            1 => (AttributeName::Normal, "normal"),
            2 => (AttributeName::Color, "color"),
            3 => (AttributeName::UV0, "uv0"),
            4 => (AttributeName::UV1, "uv1"),
            5 => (AttributeName::SkinIndex, "skinIndex"),
            6 => (AttributeName::SkinWeight, "skinWeight"),
            7 => (AttributeName::Tangent, "tangent"),
            8 => (AttributeName::BiNormal, "binormal"),
            9 => (AttributeName::UV2, "uv2"),
            10 => (AttributeName::UV3, "uv3"),
            11 => (AttributeName::UV4, "uv4"),
            12 => (AttributeName::UV5, "uv5"),
            13 => (AttributeName::UV6, "uv6"),
            14 => (AttributeName::UV7, "uv7"),
            15 => (AttributeName::UV8, "uv8"),
            _ => {
                assert!(false, "no support");
                (AttributeName::Custom("no support".to_string()), "no support")
            }
        }
    }

    fn init_uniform(gl: &WebGLRenderingContext, program: &WebGLProgram, location_map: &FnvHashMap<Atom, (usize, usize)) -> Option<(Vec<Vec<Option<CommonUniform>>>, Vec<Option<SamplerUniform>>)> {
        
        let uniform_num = gl
            .get_program_parameter(program, WebGLRenderingContext::ACTIVE_UNIFORMS)
            .try_into()
            .unwrap_or(0);

        let mut uniforms = Vec::with_capacity(layout.uniforms.len());
        let mut textures = Vec::with_capacity(layout.textures.len());

        textures.resize_with(layout.textures.len(), || { None });

        for i in 0..uniform_num {
            let uniform = gl.get_active_uniform(program, i as u32).unwrap();
            let mut value;
            let mut name = uniform.name();
            
            let is_array = match uniform.name().find('[') {
                Some(index) => {
                    let n = uniform.name();
                    let (n, _v) = n.split_at(index);
                    name = n.to_string();
                    true
                },
                None => false
            };

            let mut is_texture = false;
            match uniform.type_() {
                WebGLRenderingContext::FLOAT => {
                    if is_array {
                        let size = 1 * uniform.size() as usize;
                        value = UniformValue::FloatV(1, vec![0.0; size]);
                    } else {
                        value = UniformValue::Float(1, 0.0, 0.0, 0.0, 0.0);
                    }
                }
                WebGLRenderingContext::FLOAT_VEC2 => {
                    if is_array {
                        let size = 2 * uniform.size() as usize;
                        value = UniformValue::FloatV(2, vec![0.0; size]);
                    } else {
                        value = UniformValue::Float(2, 0.0, 0.0, 0.0, 0.0);
                    }
                }
                WebGLRenderingContext::FLOAT_VEC3 => {
                    if is_array {
                        let size = 3 * uniform.size() as usize;
                        value = UniformValue::FloatV(3, vec![0.0; size]);
                    } else {
                        value = UniformValue::Float(3, 0.0, 0.0, 0.0, 0.0);
                    }
                }
                WebGLRenderingContext::FLOAT_VEC4 => {
                    if is_array {
                        let size = 4 * uniform.size() as usize;
                        value = UniformValue::FloatV(4, vec![0.0; size]);
                    } else {
                        value = UniformValue::Float(4, 0.0, 0.0, 0.0, 0.0);
                    }
                }
                WebGLRenderingContext::INT => {
                    if is_array {
                        let size = 1 * uniform.size() as usize;
                        value = UniformValue::IntV(1, vec![0; size]);
                    } else {
                        value = UniformValue::Int(1, 0, 0, 0, 0);
                    }
                }
                WebGLRenderingContext::INT_VEC2 => {
                    if is_array {
                        let size = 2 * uniform.size() as usize;
                        value = UniformValue::IntV(2, vec![0; size]);
                    } else {
                        value = UniformValue::Int(2, 0, 0, 0, 0);
                    }
                }
                WebGLRenderingContext::INT_VEC3 => {
                    if is_array {
                        let size = 3 * uniform.size() as usize;
                        value = UniformValue::IntV(3, vec![0; size]);
                    } else {
                        value = UniformValue::Int(3, 0, 0, 0, 0);
                    }
                }
                WebGLRenderingContext::INT_VEC4 => {
                    if is_array {
                        let size = 4 * uniform.size() as usize;
                        value = UniformValue::IntV(4, vec![0; size]);
                    } else {
                        value = UniformValue::Int(4, 0, 0, 0, 0);
                    }
                }
                WebGLRenderingContext::FLOAT_MAT2 => {
                    let size = 4 * uniform.size() as usize;
                    value = UniformValue::MatrixV(2, vec![0.0; size]);
                }
                WebGLRenderingContext::FLOAT_MAT3 => {
                    let size = 9 * uniform.size() as usize;
                    value = UniformValue::MatrixV(3, vec![0.0; size]);
                }
                WebGLRenderingContext::FLOAT_MAT4 => {
                    let size = 16 * uniform.size() as usize;
                    value = UniformValue::MatrixV(4, vec![0.0; size]);
                }
                WebGLRenderingContext::SAMPLER_2D => {
                    is_texture = true;
                }
                _ => {
                    panic!("Invalid Uniform");
                }
            }

            let loc = gl.get_uniform_location(program, &uniform.name()).unwrap();

            let name = Atom::from(name);
            if is_texture {
                let location = Self::get_sampler_location(layout, &name);
                if location < 0 || location >= textures.len() as i32 {
                    return None;
                } else {
                    textures[location as usize] = Some(SamplerUniform {
                        value: None,
                        location: loc,
                    });
                }
            } else {
                match location_map.get(&name) {
                    None => return None,
                    Some((i, j)) => {
                        uniforms[*i][*j] = Some(CommonUniform {
                            value: value,
                            location: loc,
                        });
                    }
                }
            }
        }

        return Some((uniforms, textures));
    }

     fn get_sampler_location(layout: &UniformLayout, name: &Atom) -> i32 {
        let mut location = 0;
        for t in layout.textures.iter() {
            if *t == *name {
                return location;
            }
            location += 1;
        }
        return -1;
    }
}