
use std::sync::{Weak, Arc};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use stdweb::unstable::TryInto;
use webgl_rendering_context::{
    WebGLShader,
    WebGLProgram,
    WebGLUniformLocation,
    WebGLRenderingContext,
};
use fnv::FnvHashMap;

use atom::Atom;
use hal_core::*;
use context::{WebGLContextImpl};
use texture::{WebGLTextureImpl};
use sampler::{WebGLSamplerImpl};
use state::{State};
use debug_info::*;

/**
 * GPU Shader
 */
#[derive(Debug)]
pub struct Shader {
    shader_type: ShaderType,
    handle: WebGLShader,
}

/**
 * 着色器程序
 * 除了着色器的opengl句柄，还有着色器内部有效的attributes和uniforms
 */
pub struct Program {
    handle: WebGLProgram,
    gl: Weak<WebGLRenderingContext>,
    
    _attributes: FnvHashMap<AttributeName, u32>,  // 值是WebGL的Attrbitue Location

    all_uniforms: FnvHashMap<Atom, WebGLUniformImpl>, // Shader对应的所有Uniform，对应WebGL的概念

    // u32是上一次设置时候的 dirty_count
    last_uniforms: FnvHashMap<Atom, (u32, Arc<dyn AsRef<Uniforms<WebGLContextImpl>>>)>, // 上次设置的Uniforms，对应接口的概念
}

pub struct WebGLUniformImpl {
    value: UniformValue<WebGLContextImpl>,
    location: WebGLUniformLocation,
}

/**
 * 程序管理器，管理program和shader的创建和生命周期
 * 注：shader和program创建很费时间，而占的显存较小；
 * 而且游戏不大的话，总的shader和program不会太多；
 * 因此已经创建的shader和program全部缓存。
 */
pub struct ProgramManager {
    
    gl: Weak<WebGLRenderingContext>,

    // 代码缓存
    code_caches: FnvHashMap<Atom, String>,

    // Shader缓存的键是：hash[shader名 + defines]
    shader_caches: FnvHashMap<u64, Shader>,
    
    // Program缓存的键是：hash[vs名 + fs名 + defines]
    program_caches: FnvHashMap<u64, Program>,

    max_vertex_attribs: u32,
}

impl ProgramManager {
    
    /**
     * 创建一个管理器
     * 注：一个App可能存在多个gl环境，因此ProgramManager不能是单例
     */
    pub fn new(gl: &Arc<WebGLRenderingContext>, max_vertex_attribs: u32) -> ProgramManager {
        ProgramManager {
            gl: Arc::downgrade(gl),
            code_caches: FnvHashMap::default(),
            shader_caches: FnvHashMap::default(),
            program_caches: FnvHashMap::default(),
            max_vertex_attribs: max_vertex_attribs,
        }
    }

    /** 
     * 设置shader代码
     */
    pub fn set_shader_code<C: AsRef<str>>(&mut self, name: &Atom, code: &C) {
        self.code_caches.insert(name.clone(), code.as_ref().to_string());
    }
    
    /**
     * 编译shader，返回shader对应的hash
     */
    pub fn compile_shader(&mut self, shader_type: ShaderType, name: &Atom, defines: &[Atom]) -> Result<u64, String> {
        
        // 计算shader的哈希值，[名字+宏].hash
        let shader_hash = Self::get_hash(name, defines);

        // 如果能找到，返回
        if let Some(_) = self.shader_caches.get(&shader_hash) {
            return Ok(shader_hash);
        }

        let gl = self.gl.upgrade().unwrap();
        
        let shader = gl.create_shader(match shader_type {
            ShaderType::Vertex => WebGLRenderingContext::VERTEX_SHADER,
            ShaderType::Fragment => WebGLRenderingContext::FRAGMENT_SHADER,
        }).ok_or_else(|| String::from("Unable to create shader object"))?;

        let code = self.code_caches.get(name).ok_or_else(|| String::from("Unkown shader name"))?;

        // 将宏定义放到shader代码的开头
        let mut s = "".to_string();
        for d in defines {
            s += "#define ";
            s += d.as_ref();
            s += "\n";
        }
        let s = s + code;

        gl.shader_source(&shader, &s);
        gl.compile_shader(&shader);

        let is_compile_ok = gl.get_shader_parameter(&shader, WebGLRenderingContext::COMPILE_STATUS).try_into().unwrap_or(false);

        // 微信小游戏移动端环境，返回的是1-0，所以需要再来一次
        let is_compile_ok = if is_compile_ok { is_compile_ok } else {
            let r = gl
                .get_shader_parameter(&shader, WebGLRenderingContext::COMPILE_STATUS)
                .try_into()
                .unwrap_or(0);

            r != 0
        };

        if is_compile_ok {
            self.shader_caches.insert(shader_hash, Shader {
                shader_type: shader_type,
                handle: shader,
            });
            Ok(shader_hash)
        } else {
            let err = gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown error creating shader".into());

            debug_println!("Shader, compile_shader error, info = {:?}", &err);
            Err(err)
        }
    }

    /**
     * 找program
     */
    pub fn get_program(&mut self, vs_hash: u64, fs_hash: u64) -> Result<&mut Program, String> {
        
        if let Err(s) = self.link_program(vs_hash, fs_hash) {
            return Err(s);
        }

        let program_hash = Self::get_hash(&0, &[vs_hash, fs_hash]);
        if let Some(program) = self.program_caches.get_mut(&program_hash) {
            return Ok(program);
        } else {
            debug_println!("Shader, get_program error");
            return Err("Get Program Error!".to_string());
        }
    }

    /**
     * 连接program
     */
    pub fn link_program(&mut self, vs_hash: u64, fs_hash: u64) -> Result<(), String> {
        
        // 计算program的hash
        let program_hash = Self::get_hash(&0, &[vs_hash, fs_hash]);
        if let Some(_) = self.program_caches.get(&program_hash) {
            return Ok(());
        }

        // 确认shader存在，否则报错
        let vs = self.shader_caches.get(&vs_hash).ok_or_else(|| String::from("unknown vertex shader"))?;
        if vs.shader_type != ShaderType::Vertex {
            debug_println!("Shader, link_program error, not found vs = {:?}", vs);
            return Err(format!("{} isn't vertex shader", vs_hash));
        }

        let fs = self.shader_caches.get(&fs_hash).ok_or_else(|| String::from("unknown fragment shader"))?;
        if fs.shader_type != ShaderType::Fragment {
            debug_println!("Shader, link_program error, not found fs = {:?}", fs);
            return Err(format!("{} isn't fragment shader", fs_hash));
        }

        let gl = self.gl.upgrade().unwrap();
        
        // 创建program
        let program_handle = gl.create_program().ok_or_else(|| String::from("unable to create shader object"))?;
        gl.attach_shader(&program_handle, &vs.handle);
        gl.attach_shader(&program_handle, &fs.handle);

        // 先绑定属性，再连接
        let max_attribute_count = std::cmp::min(self.max_vertex_attribs, get_builtin_attribute_count());
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

        // 初始化attribute和uniform
        let attributes = ProgramManager::init_attribute(&gl, &program_handle);
        let all_uniforms = ProgramManager::init_uniform(&gl, &program_handle);
        
        // 将program加入缓存
        let program = Program {
            gl: self.gl.clone(),
            handle: program_handle,
            _attributes: attributes,
            all_uniforms: all_uniforms,
            last_uniforms: FnvHashMap::default(),
        };

        self.program_caches.insert(program_hash, program);

        Ok(())
    }

    fn init_attribute(gl: &WebGLRenderingContext, program: &WebGLProgram) -> FnvHashMap<AttributeName, u32> {
        
        // 为了减少状态切换，限制attribute前16个location必须用下面的名字

        let mut attributes = FnvHashMap::default();
        
        let attributes_num = gl
            .get_program_parameter(program, WebGLRenderingContext::ACTIVE_ATTRIBUTES)
            .try_into()
            .unwrap_or(0);

        for i in 0..attributes_num {
            match gl.get_active_attrib(program, i) {
                Some(info) => {
                    debug_println!("attribute name = {:?}, location = {:?}", &info.name(), i);
                    attributes.insert(AttributeName::from(Atom::from(info.name())), i);
                }
                None => {

                }
            }

        }

        return attributes;
    }

    fn init_uniform(gl: &WebGLRenderingContext, program: &WebGLProgram) -> FnvHashMap<Atom, WebGLUniformImpl> {
        
        let mut uniforms = FnvHashMap::default();
        
        let uniform_num = gl
            .get_program_parameter(program, WebGLRenderingContext::ACTIVE_UNIFORMS)
            .try_into()
            .unwrap_or(0);

        for i in 0..uniform_num {
            let uniform = gl.get_active_uniform(program, i).unwrap();
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
            
            match uniform.type_() {
                WebGLRenderingContext::FLOAT => {
                    if is_array {
                        let size = 1 * uniform.size() as usize;
                        value = UniformValue::<WebGLContextImpl>::FloatV(1, vec![0.0; size]);
                    } else {
                        value = UniformValue::<WebGLContextImpl>::Float(1, 0.0, 0.0, 0.0, 0.0);
                    }
                }
                WebGLRenderingContext::FLOAT_VEC2 => {
                    if is_array {
                        let size = 2 * uniform.size() as usize;
                        value = UniformValue::<WebGLContextImpl>::FloatV(2, vec![0.0; size]);
                    } else {
                        value = UniformValue::<WebGLContextImpl>::Float(2, 0.0, 0.0, 0.0, 0.0);
                    }
                }
                WebGLRenderingContext::FLOAT_VEC3 => {
                    if is_array {
                        let size = 3 * uniform.size() as usize;
                        value = UniformValue::<WebGLContextImpl>::FloatV(3, vec![0.0; size]);
                    } else {
                        value = UniformValue::<WebGLContextImpl>::Float(3, 0.0, 0.0, 0.0, 0.0);
                    }
                }
                WebGLRenderingContext::FLOAT_VEC4 => {
                    if is_array {
                        let size = 4 * uniform.size() as usize;
                        value = UniformValue::<WebGLContextImpl>::FloatV(4, vec![0.0; size]);
                    } else {
                        value = UniformValue::<WebGLContextImpl>::Float(4, 0.0, 0.0, 0.0, 0.0);
                    }
                }
                WebGLRenderingContext::INT => {
                    if is_array {
                        let size = 1 * uniform.size() as usize;
                        value = UniformValue::<WebGLContextImpl>::IntV(1, vec![0; size]);
                    } else {
                        value = UniformValue::<WebGLContextImpl>::Int(1, 0, 0, 0, 0);
                    }
                }
                WebGLRenderingContext::INT_VEC2 => {
                    if is_array {
                        let size = 2 * uniform.size() as usize;
                        value = UniformValue::<WebGLContextImpl>::IntV(2, vec![0; size]);
                    } else {
                        value = UniformValue::<WebGLContextImpl>::Int(2, 0, 0, 0, 0);
                    }
                }
                WebGLRenderingContext::INT_VEC3 => {
                    if is_array {
                        let size = 3 * uniform.size() as usize;
                        value = UniformValue::<WebGLContextImpl>::IntV(3, vec![0; size]);
                    } else {
                        value = UniformValue::Int(3, 0, 0, 0, 0);
                    }
                }
                WebGLRenderingContext::INT_VEC4 => {
                    if is_array {
                        let size = 4 * uniform.size() as usize;
                        value = UniformValue::<WebGLContextImpl>::IntV(4, vec![0; size]);
                    } else {
                        value = UniformValue::Int(4, 0, 0, 0, 0);
                    }
                }
                WebGLRenderingContext::FLOAT_MAT2 => {
                    let size = 4 * uniform.size() as usize;
                    value = UniformValue::<WebGLContextImpl>::MatrixV(2, vec![0.0; size]);
                }
                WebGLRenderingContext::FLOAT_MAT3 => {
                    let size = 9 * uniform.size() as usize;
                    value = UniformValue::<WebGLContextImpl>::MatrixV(3, vec![0.0; size]);
                }
                WebGLRenderingContext::FLOAT_MAT4 => {
                    let size = 16 * uniform.size() as usize;
                    value = UniformValue::<WebGLContextImpl>::MatrixV(4, vec![0.0; size]);
                }
                WebGLRenderingContext::SAMPLER_2D => {
                    value = UniformValue::<WebGLContextImpl>::Sampler(Weak::<WebGLSamplerImpl>::new(), Weak::<WebGLTextureImpl>::new());
                }
                _ => {
                    panic!("Invalid Uniform");
                }
            }

            let location = gl.get_uniform_location(program, &uniform.name()).unwrap();
            debug_println!("Shader, link_program, uniform name = {:?}, location = {:?}", &name, &location);
            uniforms.insert(Atom::from(uniform.name()), WebGLUniformImpl {
                value: value,
                location: location,
            });
        }

        return uniforms;
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
                (AttributeName::Custom(Atom::from("no support")), "no support")
            }
        }
    }

    fn get_hash<N: Hash, D: Hash>(name: &N, defines: &[D]) -> u64 {
        let mut hasher = DefaultHasher::new();
        
        name.hash(&mut hasher);
        
        for v in defines.iter() {
            v.hash(&mut hasher);
        }

        hasher.finish()
    }
}

impl Program {

    pub fn use_me(&mut self) {
        if let Some(gl) = self.gl.upgrade() {
            // debug_println!("Shader, use_me, program = {:?}", &self.handle);
            gl.use_program(Some(&self.handle));
        }
    }

    pub fn set_uniforms(&mut self, state: &mut State, values: &FnvHashMap<Atom, Arc<dyn AsRef<Uniforms<WebGLContextImpl>>>>) {
        
        for (name, curr) in values.iter() {
            let is_old_same = match self.last_uniforms.get_mut(name) {
                None => {
                    self.set_uniforms_impl(state, &curr.as_ref().as_ref().values);
                    false
                }
                Some((last_dirty_count, old)) => {
                    let c = curr.as_ref().as_ref();
                    // 含有纹理的必须每帧更新，调用use_texture才行。
                    // 高层设置脏的，必须更新。
                    if c.has_texture || *last_dirty_count != c.dirty_count || !Arc::ptr_eq(old, curr) {
                        self.set_uniforms_impl(state, &c.values);
                        false
                    } else {
                        true
                    }
                }
            };
            
            if !is_old_same {
                // 更新 last_uniforms
                self.last_uniforms.insert(name.clone(), (curr.as_ref().as_ref().dirty_count, curr.clone()));
            }
        }
    }

    fn set_uniforms_impl(&mut self, state: &mut State, values: &FnvHashMap<Atom, UniformValue<WebGLContextImpl>>) {

        let gl = self.gl.upgrade();
        if gl.is_none() {
            return;
        }
        let gl = gl.as_ref().unwrap();

        for (name, v) in values.iter() {
            if let Some(u) = self.all_uniforms.get_mut(name) {
                if !Self::is_uniform_same(v, &mut u.value) {
                    // debug_println!("Shader, set_uniforms_impl, uniform name = {:?}, location = {:?}, value: {:?}", *name, &u.location, v);
                    Self::set_uniform(gl, state, &u.location, v);
                }
            } else {
                assert!(false, "set_uniforms failed, not exist in shader: {:?}", name);
            }
        }
    }

    fn is_uniform_same(curr: &UniformValue<WebGLContextImpl>, old: &mut UniformValue<WebGLContextImpl>) -> bool {
        match curr {
            UniformValue::<WebGLContextImpl>::Float(count, v0, v1, v2, v3) => match old {
                UniformValue::<WebGLContextImpl>::Float(old_count, old_v0, old_v1, old_v2, old_v3) if *old_count == *count => {
                    match *count {
                        1 => {
                            let r = *v0 == *old_v0;
                            if !r {
                                *old_v0 = *v0;
                            }
                            r
                        }
                        2 => {
                            let r = *v0 == *old_v0 && *v1 == *old_v1;
                            if !r {
                                *old_v0 = *v0;
                                *old_v1 = *v1;
                            }
                            r
                        }
                        3 => {
                            let r = *v0 == *old_v0 && *v1 == *old_v1 && *v2 == *old_v2;
                            if !r {
                                *old_v0 = *v0;
                                *old_v1 = *v1;
                                *old_v2 = *v2;
                            }
                            r
                        }
                        4 => {
                            let r = *v0 == *old_v0 && *v1 == *old_v1 && *v2 == *old_v2 && *v3 == *old_v3;
                            if !r {
                                *old_v0 = *v0;
                                *old_v1 = *v1;
                                *old_v2 = *v2;
                                *old_v3 = *v3;
                            }
                            r
                        }
                        _ => {
                            assert!(false, "invalid uniform");
                            false
                        }
                    }
                }
                _ => {
                    assert!(false, "invalid uniform");
                    false
                }
            }
            UniformValue::<WebGLContextImpl>::Int(count, v0, v1, v2, v3) => match old {
                UniformValue::<WebGLContextImpl>::Int(old_count, old_v0, old_v1, old_v2, old_v3) if *old_count == *count => {
                    match *count {
                        1 => {
                            let r = *v0 == *old_v0;
                            if !r {
                                *old_v0 = *v0;
                            }
                            r
                        }
                        2 => {
                            let r = *v0 == *old_v0 && *v1 == *old_v1;
                            if !r {
                                *old_v0 = *v0;
                                *old_v1 = *v1;
                            }
                            r
                        }
                        3 => {
                            let r = *v0 == *old_v0 && *v1 == *old_v1 && *v2 == *old_v2;
                            if !r {
                                *old_v0 = *v0;
                                *old_v1 = *v1;
                                *old_v2 = *v2;
                            }
                            r
                        }
                        4 => {
                            let r = *v0 == *old_v0 && *v1 == *old_v1 && *v2 == *old_v2 && *v3 == *old_v3;
                            if !r {
                                *old_v0 = *v0;
                                *old_v1 = *v1;
                                *old_v2 = *v2;
                                *old_v3 = *v3;
                            }
                            r
                        }
                        _ => {
                            assert!(false, "invalid uniform");
                            false
                        }
                    }
                }
                _ => {
                    assert!(false, "invalid uniform");
                    false
                }
            }
            UniformValue::<WebGLContextImpl>::FloatV(count, v) => match old {
                UniformValue::<WebGLContextImpl>::FloatV(old_count, old_v) if *old_count == *count && v.len() == old_v.len() => {
                    false
                }
                _ => {
                    assert!(false, "invalid uniform");
                    false
                }
            }
            UniformValue::<WebGLContextImpl>::IntV(count, v) => match old {
                UniformValue::<WebGLContextImpl>::IntV(old_count, old_v) if *old_count == *count && v.len() == old_v.len() => {
                    false
                }
                _ => {
                    assert!(false, "invalid uniform");
                    false
                }
            }
            UniformValue::<WebGLContextImpl>::MatrixV(count, v) => match old {
                UniformValue::<WebGLContextImpl>::MatrixV(old_count, old_v) if *old_count == *count && v.len() == old_v.len() => {
                    false
                }
                _ => {
                    assert!(false, "invalid uniform");
                    false
                }
            }
            UniformValue::<WebGLContextImpl>::Sampler(_s, _t) => {
                 false
            }
        }
    }

    fn set_uniform(gl: &WebGLRenderingContext, state: &mut State, location: &WebGLUniformLocation, value: &UniformValue<WebGLContextImpl>) {
        match value {
            UniformValue::<WebGLContextImpl>::Float(count, v0, v1, v2, v3) => {
                match *count {
                    1 => gl.uniform1f(Some(location), *v0),
                    2 => gl.uniform2f(Some(location), *v0, *v1),
                    3 => gl.uniform3f(Some(location), *v0, *v1, *v2),
                    4 => gl.uniform4f(Some(location), *v0, *v1, *v2, *v3),
                    _ => {
                        assert!(false, "no support");
                    }
                }
            }
            UniformValue::<WebGLContextImpl>::Int(count, v0, v1, v2, v3) => {
                match *count {
                    1 => gl.uniform1i(Some(location), *v0),
                    2 => gl.uniform2i(Some(location), *v0, *v1),
                    3 => gl.uniform3i(Some(location), *v0, *v1, *v2),
                    4 => gl.uniform4i(Some(location), *v0, *v1, *v2, *v3),
                    _ => {
                        assert!(false, "no support");
                    }
                }
            }
            UniformValue::<WebGLContextImpl>::FloatV(count, v) => {
                match *count {
                    1 => gl.uniform1fv(Some(location), v.as_slice()),
                    2 => gl.uniform2fv(Some(location), v.as_slice()),
                    3 => gl.uniform3fv(Some(location), v.as_slice()),
                    4 => gl.uniform4fv(Some(location), v.as_slice()),
                    _ => {
                        assert!(false, "no support");
                    }
                }
            }
            UniformValue::<WebGLContextImpl>::IntV(count, v) => {
                match *count {
                    1 => gl.uniform1iv(Some(location), v.as_slice()),
                    2 => gl.uniform2iv(Some(location), v.as_slice()),
                    3 => gl.uniform3iv(Some(location), v.as_slice()),
                    4 => gl.uniform4iv(Some(location), v.as_slice()),
                    _ => {
                        assert!(false, "no support");
                    }
                }
            }
            UniformValue::<WebGLContextImpl>::MatrixV(count, v) => {
                match *count {
                    2 => gl.uniform_matrix2fv(Some(location), false, v.as_slice()),
                    3 => gl.uniform_matrix3fv(Some(location), false, v.as_slice()),
                    4 => gl.uniform_matrix4fv(Some(location), false, v.as_slice()),
                    _ => {
                        assert!(false, "no support");
                    }
                }
            }
            UniformValue::<WebGLContextImpl>::Sampler(s, t) => {
                let unit = state.use_texture(t, s);
                if unit > 0 {
                    gl.uniform1i(Some(location), unit as i32);
                } else {
                    assert!(false, "no support");
                }
            }
        }
    }
}