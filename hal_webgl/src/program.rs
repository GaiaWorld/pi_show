use fx_hashmap::{FxHashMap32};
use share::{Share};
use atom::{Atom};
use hal_core::*;
use webgl_rendering_context::{WebGLProgram, WebGLUniformLocation, WebGLRenderingContext};
use stdweb::unstable::TryInto;

use shader_cache::{ShaderCache, LayoutLocation};

pub struct SamplerUniform {
    // 针对UniformLayout的紧凑结构
    pub slot_uniform: usize,
    pub last: i32, // 上一次的值
    pub location: WebGLUniformLocation,
}

pub struct CommonUniform {
    pub slot_uniform: usize,
    pub last: UniformValue, // 上次设置的值
    pub location: WebGLUniformLocation,
}

pub struct CommonUbo {
    // 针对UniformLayout的紧凑结构
    pub slot_ubo: usize,
    pub values: Vec<CommonUniform>,
}

pub struct WebGLProgramImpl {
    pub handle: WebGLProgram,

    pub active_uniforms: Vec<CommonUbo>,
    pub active_textures: Vec<SamplerUniform>,
    
    pub last_pp: Option<Share<dyn ProgramParamter>>, // 上次设置的Uniforms，对应接口的概念
}

impl SamplerUniform {
    pub fn set_gl_uniform(&mut self, gl: &WebGLRenderingContext, unit: i32) {
        if unit != self.last {
            gl.uniform1i(Some(&self.location), unit);
            self.last = unit;
        }
    }
}

impl CommonUniform {
    pub fn set_gl_uniform(&mut self, gl: &WebGLRenderingContext, value: &UniformValue) {
        match (value, &mut self.last) {
            (UniformValue::Float(curr_count, curr_v0, curr_v1, curr_v2, curr_v3), UniformValue::Float(last_count, last_v0, last_v1, last_v2, last_v3))  => {
                if *curr_count != *last_count {
                    panic!("float uniform: count isn't match");
                }
                match *curr_count {
                    1 => {
                        if *curr_v0 != *last_v0 {
                            gl.uniform1f(Some(&self.location), *curr_v0);
                            *last_v0 = *curr_v0;
                        }
                    },
                    2 => {
                        if *curr_v0 != *last_v0 || *curr_v1 != *last_v1 {
                            gl.uniform2f(Some(&self.location), *curr_v0, *curr_v1);
                            *last_v0 = *curr_v0;
                            *last_v1 = *curr_v1;
                        }
                    },
                    3 => {
                        if *curr_v0 != *last_v0 || *curr_v1 != *last_v1 || *curr_v2 != *last_v2 {
                            gl.uniform3f(Some(&self.location), *curr_v0, *curr_v1, *curr_v2);
                            *last_v0 = *curr_v0;
                            *last_v1 = *curr_v1;
                            *last_v2 = *curr_v2;
                        }
                    },
                    4 => {
                        if *curr_v0 != *last_v0 || *curr_v1 != *last_v1 || *curr_v2 != *last_v2 || *curr_v3 != *last_v3 {
                            gl.uniform4f(Some(&self.location), *curr_v0, *curr_v1, *curr_v2, *curr_v3);
                            *last_v0 = *curr_v0;
                            *last_v1 = *curr_v1;
                            *last_v2 = *curr_v2;
                            *last_v3 = *curr_v3;
                        }
                    },
                    _ => {
                        panic!("float Invalid uniform");
                    }
                }
            },
            (UniformValue::Int(curr_count, curr_v0, curr_v1, curr_v2, curr_v3), UniformValue::Int(last_count, last_v0, last_v1, last_v2, last_v3)) => {
                if *curr_count != *last_count {
                    panic!("int uniform: count isn't match");
                }
                match *curr_count {
                    1 => {
                        if *curr_v0 != *last_v0 {
                            gl.uniform1i(Some(&self.location), *curr_v0);
                            *last_v0 = *curr_v0;
                        }
                    },
                    2 => {
                        if *curr_v0 != *last_v0 || *curr_v1 != *last_v1 {
                            gl.uniform2i(Some(&self.location), *curr_v0, *curr_v1);
                            *last_v0 = *curr_v0;
                            *last_v1 = *curr_v1;
                        }
                    },
                    3 => {
                        if *curr_v0 != *last_v0 || *curr_v1 != *last_v1 || *curr_v2 != *last_v2 {
                            gl.uniform3i(Some(&self.location), *curr_v0, *curr_v1, *curr_v2);
                            *last_v0 = *curr_v0;
                            *last_v1 = *curr_v1;
                            *last_v2 = *curr_v2;
                        }
                    },
                    4 => {
                        if *curr_v0 != *last_v0 || *curr_v1 != *last_v1 || *curr_v2 != *last_v2 || *curr_v3 != *last_v3 {
                            gl.uniform4i(Some(&self.location), *curr_v0, *curr_v1, *curr_v2, *curr_v3);
                            *last_v0 = *curr_v0;
                            *last_v1 = *curr_v1;
                            *last_v2 = *curr_v2;
                            *last_v3 = *curr_v3;
                        }
                    },
                    _ => {
                        panic!("int Invalid uniform");
                    }
                }
            },
            (UniformValue::FloatV(curr_count, curr_v), UniformValue::FloatV(last_count, _)) => {
                if *curr_count != *last_count {
                    panic!("floatv uniform: count isn't match");
                }
                match *curr_count {
                    1 => gl.uniform1fv(Some(&self.location), curr_v.as_slice()),
                    2 => gl.uniform2fv(Some(&self.location), curr_v.as_slice()),
                    3 => gl.uniform3fv(Some(&self.location), curr_v.as_slice()),
                    4 => gl.uniform4fv(Some(&self.location), curr_v.as_slice()),
                    _ => {
                        panic!("floatv Invalid uniform");
                    }
                }
            },
            (UniformValue::IntV(curr_count, curr_v), UniformValue::IntV(last_count, _))  => {
                if *curr_count != *last_count {
                    panic!("intv uniform: count isn't match");
                }
                match *curr_count {
                    1 => gl.uniform1iv(Some(&self.location), curr_v.as_slice()),
                    2 => gl.uniform2iv(Some(&self.location), curr_v.as_slice()),
                    3 => gl.uniform3iv(Some(&self.location), curr_v.as_slice()),
                    4 => gl.uniform4iv(Some(&self.location), curr_v.as_slice()),
                    _ => {
                        panic!("intv Invalid uniform");
                    }
                }
            },
            (UniformValue::MatrixV(curr_count, curr_v), UniformValue::MatrixV(last_count, _)) => {
                if *curr_count != *last_count {
                    panic!("intv uniform: count isn't match");
                }
                match *curr_count {
                    2 => gl.uniform_matrix2fv(Some(&self.location), false, curr_v.as_slice()),
                    3 => gl.uniform_matrix3fv(Some(&self.location), false, curr_v.as_slice()),
                    4 => gl.uniform_matrix4fv(Some(&self.location), false, curr_v.as_slice()),
                    _ => {
                        panic!("matrixv Invalid uniform");
                    }
                }
            },
            _ => {
                panic!(format!("Invalid Uniform: {:?}", value) )
            }
        }
    }
}

impl WebGLProgramImpl {

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

        println!("uniform_layout: {:?}", uniform_layout);
        let location_map = shader_cache.get_location_map(vs_name, fs_name, uniform_layout);
        
        // 初始化attribute和uniform
        match WebGLProgramImpl::init_uniform(gl, &program_handle, location_map) {
            None => {
                gl.delete_program(Some(&program_handle));
                Err("WebGLProgramImpl failed, invalid uniforms".to_string())
            },
            Some((uniforms, textures)) => {
                
                Ok(WebGLProgramImpl {
                    handle: program_handle,
                    active_uniforms: uniforms,
                    active_textures: textures,
                    last_pp: None,
                })
            }
        }
    }

    pub fn delete(&self, gl: &WebGLRenderingContext) {
        gl.delete_program(Some(&self.handle));
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

    fn init_uniform(gl: &WebGLRenderingContext, program: &WebGLProgram, location_map: &LayoutLocation) -> Option<(Vec<CommonUbo>, Vec<SamplerUniform>)> {
        
        let uniform_num = gl
            .get_program_parameter(program, WebGLRenderingContext::ACTIVE_UNIFORMS)
            .try_into()
            .unwrap_or(0);

        let mut uniforms = vec![];
        let mut textures = vec![];
        
        // 用于查找slot_ubo和Vec<CommonUbo>的对应关系的哈希表
        // 键是slot_ubo的索引，值是Vec<CommonUbo>的索引值
        let mut slot_map = FxHashMap32::default();
        
        for i in 0..uniform_num {
            let uniform = gl.get_active_uniform(program, i as u32).unwrap();
            let mut value;

            let mut name = uniform.name();
            let is_array = uniform.name().find('[').map_or(false, |index| {
                let n = uniform.name();
                let (n, _v) = n.split_at(index);
                name = n.to_string();
                true
            });

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
                                location: loc,
                                slot_uniform: *i,
                                last: 0,
                            });
                        }
                    }
                    continue;
                }
                _ => {
                    panic!(format!("Invalid Uniform: {:?}", name) );
                }
            }

            match location_map.uniforms.get(&name) {
                None => return None,
                Some((i, j)) => {
                    match slot_map.get(i) {
                        None => {
                            let mut ubo = CommonUbo {
                                slot_ubo: *i, 
                                values: vec![],
                            };
                            ubo.values.push(CommonUniform {
                                slot_uniform: *j,
                                last: value,
                                location: loc,
                            });
                            uniforms.push(ubo);
                            slot_map.insert(*i, uniforms.len() - 1);
                        }, 
                        Some(vi) => {
                            uniforms[*vi].values.push(CommonUniform {
                                slot_uniform: *j,
                                last: value,
                                location: loc,
                            });
                        }
                    }
                }
            }
        }
        
        // 排序，渲染的时候扫描起来更快
        textures.sort_by(|a, b| a.slot_uniform.partial_cmp(&b.slot_uniform).unwrap());

        for ubo in uniforms.iter_mut() {
            ubo.values.sort_by(|a, b| a.slot_uniform.partial_cmp(&b.slot_uniform).unwrap());
        }
        uniforms.sort_by(|a, b| a.slot_ubo.partial_cmp(&b.slot_ubo).unwrap());
        
        return Some((uniforms, textures));
    }
}