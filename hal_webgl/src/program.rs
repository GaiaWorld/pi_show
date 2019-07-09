use share::{Share};
use atom::{Atom};
use hal_core::*;
use webgl_rendering_context::{WebGLProgram, WebGLUniformLocation, WebGLRenderingContext};
use stdweb::unstable::TryInto;

use shader_cache::{ShaderCache, LayoutLocation};

pub struct SamplerUniform {
    // 针对UniformLayout的紧凑结构
    slot_uniform: usize,

    location: WebGLUniformLocation,
    value: Option<(HalTexture, HalSampler)>,
}

pub struct CommonUniform {
    // 针对UniformLayout的紧凑结构
    slot_ubo: usize,
    slot_uniform: usize,

    value: UniformValue,
    location: WebGLUniformLocation,
}

pub struct WebGLProgramImpl {
    pub handle: WebGLProgram,

    pub active_uniforms: Vec<CommonUniform>,
    pub active_textures: Vec<SamplerUniform>,
    
    pub last_ubos: Option<Share<dyn ProgramParamter>>, // 上次设置的Uniforms，对应接口的概念
}

impl WebGLProgramImpl {

    // TODO: 之后一定要问小燕，数据结构如何用str存储？
    pub fn new_with_vs_fs(gl: &WebGLRenderingContext, caps: &Capabilities, shader_cache: &mut ShaderCache, vs_id: u64, fs_id: u64, vs_name: &Atom, vs_defines: &[Option<&str>], fs_name: &Atom, fs_defines: &[Option<&str>], uniform_layout: &UniformLayout) -> Result<WebGLProgramImpl, String> {
        
        // 创建program
        let program_handle = gl.create_program().ok_or_else(|| String::from("unable to create shader object"))?;
        {
            let vs = shader_cache.compile_shader(gl, ShaderType::Vertex, vs_id, &vs_name, vs_defines);
            if let Err(e) = &vs {
                gl.delete_program(Some(&program_handle));
                return Err(e.clone());
            }
            let vs = vs.unwrap();
            gl.attach_shader(&program_handle, &vs.handle);
        }
        
        {
            let fs = shader_cache.compile_shader(gl, ShaderType::Fragment, fs_id, &fs_name, fs_defines);
            if let Err(e) = &fs {
                gl.delete_program(Some(&program_handle));
                return Err(e.clone());
            }
            let fs = fs.unwrap();
            gl.attach_shader(&program_handle, &fs.handle);
        }
        
        // 先绑定属性，再连接
        let max_attribute_count = std::cmp::min(AttributeName::get_builtin_count(), caps.max_vertex_attribs);
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

            gl.delete_program(Some(&program_handle));
            return Err(e);
        }

        let location_map = shader_cache.get_location_map(vs_name, fs_name, uniform_layout);
        
        // 初始化attribute和uniform
        match WebGLProgramImpl::init_uniform(gl, &program_handle, location_map, uniform_layout) {
            None => {
                gl.delete_program(Some(&program_handle));
                Err("WebGLProgramImpl failed, invalid uniforms".to_string())
            },
            Some((uniforms, textures)) => {
                
                Ok(WebGLProgramImpl {
                    handle: program_handle,
                    active_uniforms: uniforms,
                    active_textures: textures,
                    last_ubos: None,
                })
            }
        }
    }

    pub fn delete(&self, gl: &WebGLRenderingContext) {
        gl.delete_program(Some(&self.handle));
    }

    ////////////////////////////// 

    pub fn use_me(&mut self, gl: &WebGLRenderingContext) {
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

    fn init_uniform(gl: &WebGLRenderingContext, program: &WebGLProgram, location_map: &LayoutLocation, layout: &UniformLayout) -> Option<(Vec<CommonUniform>, Vec<SamplerUniform>)> {
        
        let uniform_num = gl
            .get_program_parameter(program, WebGLRenderingContext::ACTIVE_UNIFORMS)
            .try_into()
            .unwrap_or(0);

        let mut uniforms = vec![];
        let mut textures = vec![];

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
            let name = Atom::from(name);
            let loc = gl.get_uniform_location(program, &uniform.name()).unwrap();
            
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
                    match location_map.textures.get(&name) {
                        None => return None,
                        Some(i) => {
                            
                            textures.push(SamplerUniform {
                                value: None,
                                location: loc,
                                slot_uniform: *i,
                            });
                        }
                    }
                    continue;
                }
                _ => {
                    panic!("Invalid Uniform");
                }
            }

            match location_map.uniforms.get(&name) {
                None => return None,
                Some((i, j)) => {
                    uniforms.push(CommonUniform {
                        value: value,
                        location: loc,
                        slot_ubo: *i,
                        slot_uniform: *j,
                    });
                }
            }
        }

        return Some((uniforms, textures));
    }
}