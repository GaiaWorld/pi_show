use std::convert::TryFrom;

use atom::Atom;
use hal_core::*;
use hash::XHashMap;
use share::Share;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlUniformLocation};
use js_sys::{Boolean, Number};

use shader_cache::{LayoutLocation, ShaderCache};

pub struct SamplerUniform {
    // 针对UniformLayout的紧凑结构
    pub slot_uniform: usize,
    pub last: i32, // 上一次的值
    pub location: WebGlUniformLocation,
}

pub struct CommonUniform {
    pub slot_uniform: usize,
    pub last: UniformValue, // 上次设置的值
    pub name: Atom,
    pub location: WebGlUniformLocation,
}

pub struct CommonUbo {
    // 针对UniformLayout的紧凑结构
    pub slot_ubo: usize,
    pub values: Vec<CommonUniform>,
    pub last: Option<Share<dyn UniformBuffer>>,
}

pub struct WebGLProgramImpl {
    pub handle: WebGlProgram,

    pub active_uniforms: Vec<CommonUbo>,
    pub active_single_uniforms: Vec<CommonUniform>,
    pub active_textures: Vec<SamplerUniform>,
}

impl SamplerUniform {
    pub fn set_gl_uniform(&mut self, gl: &WebGlRenderingContext, unit: i32) {
        if unit != self.last {
            gl.uniform1i(Some(&self.location), unit);
            self.last = unit;
        }
    }
}

impl CommonUniform {
    pub fn set_gl_uniform(&mut self, gl: &WebGlRenderingContext, value: &UniformValue) -> Result<(), String> {
        match (value, &mut self.last) {
            (UniformValue::Float1(c1), UniformValue::Float1(o1)) => {
                if *c1 != *o1 {
                    gl.uniform1f(Some(&self.location), *c1);
                    *o1 = *c1;
                }
            }
            (UniformValue::Float2(c1, c2), UniformValue::Float2(o1, o2)) => {
                if *c1 != *o1 || *c2 != *o2 {
                    gl.uniform2f(Some(&self.location), *c1, *c2);
                    *o1 = *c1;
                    *o2 = *c2;
                }
            }
            (UniformValue::Float3(c1, c2, c3), UniformValue::Float3(o1, o2, o3)) => {
                if *c1 != *o1 || *c2 != *o2 || *c3 != *o3 {
                    gl.uniform3f(Some(&self.location), *c1, *c2, *c3);
                    *o1 = *c1;
                    *o2 = *c2;
                    *o3 = *c3;
                }
            }
            (UniformValue::Float4(c1, c2, c3, c4), UniformValue::Float4(o1, o2, o3, o4)) => {
                if *c1 != *o1 || *c2 != *o2 || *c3 != *o3 || *c4 != *o4 {
                    gl.uniform4f(Some(&self.location), *c1, *c2, *c3, *c4);
                    *o1 = *c1;
                    *o2 = *c2;
                    *o3 = *c3;
                    *o4 = *c4;
                }
            }
            (UniformValue::Int1(c1), UniformValue::Int1(o1)) => {
                if *c1 != *o1 {
                    gl.uniform1i(Some(&self.location), *c1);
                    *o1 = *c1;
                }
            }
            (UniformValue::Int2(c1, c2), UniformValue::Int2(o1, o2)) => {
                if *c1 != *o1 || *c2 != *o2 {
                    gl.uniform2i(Some(&self.location), *c1, *c2);
                    *o1 = *c1;
                    *o2 = *c2;
                }
            }
            (UniformValue::Int3(c1, c2, c3), UniformValue::Int3(o1, o2, o3)) => {
                if *c1 != *o1 || *c2 != *o2 || *c3 != *o3 {
                    gl.uniform3i(Some(&self.location), *c1, *c2, *c3);
                    *o1 = *c1;
                    *o2 = *c2;
                    *o3 = *c3;
                }
            }
            (UniformValue::Int4(c1, c2, c3, c4), UniformValue::Int4(o1, o2, o3, o4)) => {
                if *c1 != *o1 || *c2 != *o2 || *c3 != *o3 || *c4 != *o4 {
                    gl.uniform4i(Some(&self.location), *c1, *c2, *c3, *c4);
                    *o1 = *c1;
                    *o2 = *c2;
                    *o3 = *c3;
                    *o4 = *c4;
                }
            }
            (UniformValue::FloatV1(v), _) => {
                gl.uniform1fv_with_f32_array(Some(&self.location), v.as_slice());
            }
            (UniformValue::FloatV2(v), _) => {
                gl.uniform2fv_with_f32_array(Some(&self.location), v.as_slice());
            }
            (UniformValue::FloatV3(v), _) => {
                gl.uniform3fv_with_f32_array(Some(&self.location), v.as_slice());
            }
            (UniformValue::FloatV4(v), _) => {
                gl.uniform4fv_with_f32_array(Some(&self.location), v.as_slice());
            }
            (UniformValue::IntV1(v), _) => {
                gl.uniform1iv_with_i32_array(Some(&self.location), v.as_slice());
            }
            (UniformValue::IntV2(v), _) => {
                gl.uniform2iv_with_i32_array(Some(&self.location), v.as_slice());
            }
            (UniformValue::IntV3(v), _) => {
                gl.uniform3iv_with_i32_array(Some(&self.location), v.as_slice());
            }
            (UniformValue::IntV4(v), _) => {
                gl.uniform4iv_with_i32_array(Some(&self.location), v.as_slice());
            }
            (UniformValue::MatrixV2(v), _) => {
                gl.uniform_matrix2fv_with_f32_array(Some(&self.location), false, v.as_slice());
            }
            (UniformValue::MatrixV3(v), _) => {
                gl.uniform_matrix3fv_with_f32_array(Some(&self.location), false, v.as_slice());
            }
            (UniformValue::MatrixV4(v), _) => {
                gl.uniform_matrix4fv_with_f32_array(Some(&self.location), false, v.as_slice());
            }
            _ => {
				log::error!("Invalid Uniform, name: {:?}, value: {:?}", self.name.as_ref(), value);
				return Err(format!("Invalid Uniform, name: {:?}, value: {:?}", self.name.as_ref(), value));
			},
        }
		Ok(())
    }
}

impl WebGLProgramImpl {
    pub fn restore_active_uniform(&mut self, index: usize) {
        for active_uniform in self.active_uniforms.iter_mut() {
            if active_uniform.slot_ubo == index {
                active_uniform.last = None;
            }
        }
    }

    pub fn new_with_vs_fs(
        gl: &WebGlRenderingContext,
        caps: &Capabilities,
        shader_cache: &mut ShaderCache,
        vs_id: u64,
        fs_id: u64,
        vs_name: &Atom,
        vs_defines: &[Option<&str>],
        fs_name: &Atom,
        fs_defines: &[Option<&str>],
        uniform_layout: &UniformLayout,
    ) -> Result<WebGLProgramImpl, String> {
        // 创建program
        let program_handle = gl
            .create_program()
            .ok_or_else(|| String::from("unable to create shader object"))?;
        {
            let vs =
                shader_cache.compile_shader(gl, ShaderType::Vertex, vs_id, &vs_name, vs_defines);
            if let Err(e) = &vs {
                gl.delete_program(Some(&program_handle));
                return Err(e.clone());
            }
            let vs = vs.unwrap();
            gl.attach_shader(&program_handle, &vs.handle);
        }

        {
            let fs =
                shader_cache.compile_shader(gl, ShaderType::Fragment, fs_id, &fs_name, fs_defines);
            if let Err(e) = &fs {
                gl.delete_program(Some(&program_handle));
                return Err(e.clone());
            }
            let fs = fs.unwrap();
            gl.attach_shader(&program_handle, &fs.handle);
        }

        // 先绑定属性，再连接
        let max_attribute_count =
            std::cmp::min(AttributeName::get_builtin_count(), caps.max_vertex_attribs);
        for i in 0..max_attribute_count {
            let (_attrib_name, name) = Self::get_attribute_by_location(i);
            debug_println!(
                "Shader, link_program, attribute name = {:?}, location = {:?}",
                &name,
                i
            );
            gl.bind_attrib_location(&program_handle, i, name);
        }

        // 连接program
		gl.link_program(&program_handle);
        let is_link_ok = match Boolean::try_from(gl
            .get_program_parameter(&program_handle, WebGlRenderingContext::LINK_STATUS)){
				Ok(r) => r.into(),
				Err(_) => false,
			};
			// .try_into().unwrap_or(false);
			// is_link_ok.into()

        // 微信小游戏移动端环境，返回的是1-0，所以需要再来一次
        let is_link_ok = if is_link_ok {
            is_link_ok
        } else {
			let r:f64 = match Number::try_from(gl
				.get_program_parameter(&program_handle, WebGlRenderingContext::LINK_STATUS)){
					Ok(r) => r.value_of() ,
					Err(_) => 0.0,
				};
            // let r = gl
            //     .get_program_parameter(&program_handle, WebGlRenderingContext::LINK_STATUS)
            //     .try_into()
            //     .unwrap_or(0);

            r != 0.0
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
        let (uniforms, single_uniforms, textures) =
            WebGLProgramImpl::init_uniform(gl, &program_handle, location_map, uniform_layout)?;
        Ok(WebGLProgramImpl {
            handle: program_handle,
            active_uniforms: uniforms,
            active_single_uniforms: single_uniforms,
            active_textures: textures,
        })
    }

    pub fn delete(&self, gl: &WebGlRenderingContext) {
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
                (
                    AttributeName::Custom("no support".to_string()),
                    "no support",
                )
            }
        }
    }

    fn init_uniform(
        gl: &WebGlRenderingContext,
        program: &WebGlProgram,
        location_map: &LayoutLocation,
		uniform_layout: &UniformLayout,
    ) -> Result<(Vec<CommonUbo>, Vec<CommonUniform>, Vec<SamplerUniform>), String> {
        let uniform_num = match Number::try_from(gl
            .get_program_parameter(program, WebGlRenderingContext::ACTIVE_UNIFORMS)) {
				Ok(r) => r.value_of() as i32,
				Err(_) => 0,
			};
            //.unwrap_or(0);

        let mut textures = vec![];
        let mut uniforms = vec![];
        let mut single_uniforms = vec![];

        // 用于查找slot_ubo和Vec<CommonUbo>的对应关系的哈希表
        // 键是slot_ubo的索引，值是Vec<CommonUbo>的索引值
        let mut slot_map = XHashMap::default();

        for i in 0..uniform_num {
            let uniform = gl.get_active_uniform(program, i as u32).unwrap();
            let value;

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
                WebGlRenderingContext::FLOAT => {
                    if is_array {
                        let size = 1 * uniform.size() as usize;
                        value = UniformValue::FloatV1(vec![0.0; size]);
                    } else {
                        value = UniformValue::Float1(0.0);
                    }
                }
                WebGlRenderingContext::FLOAT_VEC2 => {
                    if is_array {
                        let size = 2 * uniform.size() as usize;
                        value = UniformValue::FloatV2(vec![0.0; size]);
                    } else {
                        value = UniformValue::Float2(0.0, 0.0);
                    }
                }
                WebGlRenderingContext::FLOAT_VEC3 => {
                    if is_array {
                        let size = 3 * uniform.size() as usize;
                        value = UniformValue::FloatV3(vec![0.0; size]);
                    } else {
                        value = UniformValue::Float3(0.0, 0.0, 0.0);
                    }
                }
                WebGlRenderingContext::FLOAT_VEC4 => {
                    if is_array {
                        let size = 4 * uniform.size() as usize;
                        value = UniformValue::FloatV4(vec![0.0; size]);
                    } else {
                        value = UniformValue::Float4(0.0, 0.0, 0.0, 0.0);
                    }
                }
                WebGlRenderingContext::INT => {
                    if is_array {
                        let size = 1 * uniform.size() as usize;
                        value = UniformValue::IntV1(vec![0; size]);
                    } else {
                        value = UniformValue::Int1(0);
                    }
                }
                WebGlRenderingContext::INT_VEC2 => {
                    if is_array {
                        let size = 2 * uniform.size() as usize;
                        value = UniformValue::IntV2(vec![0; size]);
                    } else {
                        value = UniformValue::Int2(0, 0);
                    }
                }
                WebGlRenderingContext::INT_VEC3 => {
                    if is_array {
                        let size = 3 * uniform.size() as usize;
                        value = UniformValue::IntV3(vec![0; size]);
                    } else {
                        value = UniformValue::Int3(0, 0, 0);
                    }
                }
                WebGlRenderingContext::INT_VEC4 => {
                    if is_array {
                        let size = 4 * uniform.size() as usize;
                        value = UniformValue::IntV4(vec![0; size]);
                    } else {
                        value = UniformValue::Int4(0, 0, 0, 0);
                    }
                }
                WebGlRenderingContext::FLOAT_MAT2 => {
                    let size = 4 * uniform.size() as usize;
                    value = UniformValue::MatrixV2(vec![0.0; size]);
                }
                WebGlRenderingContext::FLOAT_MAT3 => {
                    let size = 9 * uniform.size() as usize;
                    value = UniformValue::MatrixV3(vec![0.0; size]);
                }
                WebGlRenderingContext::FLOAT_MAT4 => {
                    let size = 16 * uniform.size() as usize;
                    value = UniformValue::MatrixV4(vec![0.0; size]);
                }
                WebGlRenderingContext::SAMPLER_2D => {
                    match location_map.textures.get(&name) {
                        None => return Err(format!("init_uniform fail, location_map texture is not exist, name: {:?}, location_map: {:?}", name, location_map)),
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
					log::error!("Invalid Uniform: {:?}", name);
                    panic!();
                }
            }

            if let Some(index) = location_map.single_uniforms.get(&name) {
                single_uniforms.push(CommonUniform {
                    slot_uniform: *index,
                    name: name.clone(),
                    last: value,
                    location: loc,
                });
                continue;
            }

            match location_map.uniforms.get(&name) {
                None => return Err(format!("init_uniform fail, location_map uniform is not exist, name: {:?}, location_map: {:?}", name, location_map)),
                Some((i, j)) => {
                    match slot_map.get(i) {
                        None => {
                            let mut ubo = CommonUbo {
                                slot_ubo: *i, 
                                values: vec![],
                                last: None,
                            };
                            ubo.values.push(CommonUniform {
                                slot_uniform: *j,
                                name: name.clone(),
                                last: value,
                                location: loc,
                            });
                            uniforms.push(ubo);
                            slot_map.insert(*i, uniforms.len() - 1);
                        }, 
                        Some(vi) => {
                            uniforms[*vi].values.push(CommonUniform {
                                slot_uniform: *j,
                                name: name.clone(),
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
            ubo.values
                .sort_by(|a, b| a.slot_uniform.partial_cmp(&b.slot_uniform).unwrap());
        }
        uniforms.sort_by(|a, b| a.slot_ubo.partial_cmp(&b.slot_ubo).unwrap());

        single_uniforms.sort_by(|a, b| a.slot_uniform.partial_cmp(&b.slot_uniform).unwrap());

        return Ok((uniforms, single_uniforms, textures));
    }
}
