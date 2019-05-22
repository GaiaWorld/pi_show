
use std::rc::{Weak};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap};

use stdweb::unstable::TryInto;
use webgl_rendering_context::{
    WebGLShader,
    WebGLProgram,
    WebGLUniformLocation,
    WebGLRenderingContext,
};

use atom::Atom;
use hal_core::{UniformValue};

/**
 * Shader的类型
 */
#[derive(PartialEq)]
pub enum ShaderType {
    Vertex,   // 顶点着色器
    Fragment, // 片段着色器
}

/**
 * GPU Shader
 */
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
    attributes: FnvHashMap<Atom, u32>,
    uniforms: FnvHashMap<Atom, WebGLUniformImpl>,
}

pub struct WebGLUniformImpl {
    value: UniformValue,
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

        if is_compile_ok {
            self.shader_caches.insert(shader_hash, Shader {
                shader_type: shader_type,
                handle: shader,
            });
            Ok(shader_hash)
        } else {
            Err(gl
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown error creating shader".into()))
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
            return Err(format!("{} isn't vertex shader", vs_hash));
        }

        let fs = self.shader_caches.get(&fs_hash).ok_or_else(|| String::from("unknown fragment shader"))?;
        if fs.shader_type != ShaderType::Fragment {
            return Err(format!("{} isn't fragment shader", fs_hash));
        }

        // 创建program，并链接
        let gl = self.gl.upgrade().unwrap();

        let program_handle = gl.create_program().ok_or_else(|| String::from("unable to create shader object"))?;
        gl.attach_shader(&program_handle, &vs.handle);
        gl.attach_shader(&program_handle, &fs.handle);

        gl.link_program(&program_handle);
        let is_link_ok = gl
            .get_program_parameter(&program_handle, WebGLRenderingContext::LINK_STATUS)
            .try_into()
            .unwrap_or(false);

        if !is_link_ok {
            return Err(gl
                .get_program_info_log(&program_handle)
                .unwrap_or_else(|| "unkown link error".into()));
        }

        let attributes = ProgramManager::init_attribute(&gl, &program_handle, self.max_vertex_attribs);
        let uniforms = ProgramManager::init_uniform(&gl, &program_handle);

        // 将program加入缓存
        let program = Program {
            gl: self.gl.clone(),
            handle: program_handle,
            attributes: attributes,
            uniforms: uniforms
        };

        self.program_caches.insert(program_hash, program);

        Ok(())
    }

    fn init_attribute(gl: &WebGLRenderingContext, program: &WebGLProgram, max_vertex_attribs: u32) -> FnvHashMap<Atom, u32> {
        
        // 为了减少状态切换，限制attribute前16个location必须用下面的名字

        let mut attributes: FnvHashMap<Atom, u32> = FnvHashMap::default();
        let attribute_names = ["position", "normal", "color", "uv0", "uv1", "skinIndex", "skinWeight", "tangent","binormal", "uv2", "uv3", "uv4", "uv5", "uv6", "uv7", "uv8"];

        let mut max_attribute_count = max_vertex_attribs;
        if attribute_names.len() as u32 > max_attribute_count {
            max_attribute_count = attribute_names.len() as u32;
        }
        
        // 因为webgl有警告，所以这里就不记录类型和大小了。
        for i in 0..max_attribute_count {
            gl.bind_attrib_location(program, i, &attribute_names[i as usize]);
            attributes.insert(Atom::from(attribute_names[i as usize]), i);
        }

        return attributes;
    }

    fn init_uniform(gl: &WebGLRenderingContext, program: &WebGLProgram) -> FnvHashMap<Atom, Uniform> {
        // uniform的信息得从program里面取出来，一旦确定，就不会变了。
        let mut uniforms: FnvHashMap<Atom, Uniform> = FnvHashMap::default();
        
        let uniform_num = gl
            .get_program_parameter(program, WebGLRenderingContext::ACTIVE_UNIFORMS)
            .try_into()
            .unwrap_or(0);

        for i in 0..uniform_num {
            let uniform = gl.get_active_uniform(program, i).unwrap();
            let mut u_count = 0;
            let mut item_count = 0;
            let mut value;
            let mut name = uniform.name();
            
            let is_array = match uniform.name().find('[') {
                Some(index) => {
                    let n = uniform.name();
                    let (n, v) = n.split_at(index);
                    name = n.to_string();
                    true
                },
                None => false
            };

            match uniform.type_() {
                WebGLRenderingContext::FLOAT => {
                    if is_array {
                        item_count = 1;
                        u_count = 1 * uniform.size();
                        value = UniformValue::Floats(Vec::with_capacity(u_count as usize));
                    } else { 
                        u_count = 1;
                        item_count = 1;
                        value = UniformValue::Float(0.0, 0.0, 0.0, 0.0);
                    }
                }
                WebGLRenderingContext::FLOAT_VEC2 => {
                    if is_array {
                        item_count = 2;
                        u_count = 2 * uniform.size();
                        value = UniformValue::Floats(Vec::with_capacity(u_count as usize));
                    } else {
                        u_count = 2;
                        item_count = 2;
                        value = UniformValue::Float(0.0, 0.0, 0.0, 0.0);
                    }
                }
                WebGLRenderingContext::FLOAT_VEC3 => {
                    if is_array {
                        item_count = 3;
                        u_count = 3 * uniform.size();
                        value = UniformValue::Floats(Vec::with_capacity(u_count as usize));
                    } else {
                        u_count = 3;
                        item_count = 3;
                        value = UniformValue::Float(0.0, 0.0, 0.0, 0.0);
                    }
                }
                WebGLRenderingContext::FLOAT_VEC4 => {
                    if is_array {
                        item_count = 4;
                        u_count = 4 * uniform.size();
                        value = UniformValue::Floats(Vec::with_capacity(u_count as usize));
                    } else {
                        u_count = 4;
                        item_count = 4;
                        value = UniformValue::Float(0.0, 0.0, 0.0, 0.0);
                    }
                }
                WebGLRenderingContext::SAMPLER_2D => {
                    u_count = 1;
                    item_count = 1;
                    value = UniformValue::Int(0, 0, 0, 0);
                }
                WebGLRenderingContext::INT => {
                    if is_array {
                        item_count = 1;
                        u_count = 1 * uniform.size();
                        value = UniformValue::Ints(Vec::with_capacity(u_count as usize));
                    } else { 
                        u_count = 1;
                        item_count = 1;
                        value = UniformValue::Int(0, 0, 0, 0);
                    }
                }
                WebGLRenderingContext::INT_VEC2 => {
                    if is_array {
                        item_count = 2;
                        u_count = 2 * uniform.size();
                        value = UniformValue::Ints(Vec::with_capacity(u_count as usize));
                    } else { 
                        u_count = 2;
                        item_count = 2;
                        value = UniformValue::Int(0, 0, 0, 0);
                    }
                }
                WebGLRenderingContext::INT_VEC3 => {
                    if is_array {
                        item_count = 3;
                        u_count = 3 * uniform.size();
                        value = UniformValue::Ints(Vec::with_capacity(u_count as usize));
                    } else { 
                        u_count = 3;
                        item_count = 3;
                        value = UniformValue::Int(0, 0, 0, 0);
                    }
                }
                WebGLRenderingContext::INT_VEC4 => {
                    if is_array {
                        item_count = 4;
                        u_count = 4 * uniform.size();
                        value = UniformValue::Ints(Vec::with_capacity(u_count as usize));
                    } else { 
                        u_count = 4;
                        item_count = 4;
                        value = UniformValue::Int(0, 0, 0, 0);
                    }
                }
                WebGLRenderingContext::FLOAT_MAT2 => {
                    item_count = 4;
                    u_count = 4 * uniform.size();
                    value = UniformValue::Floats(Vec::with_capacity(u_count as usize));
                }
                WebGLRenderingContext::FLOAT_MAT3 => {
                    item_count = 9;
                    u_count = 9 * uniform.size();
                    value = UniformValue::Floats(Vec::with_capacity(u_count as usize));
                }
                WebGLRenderingContext::FLOAT_MAT4 => {
                    item_count = 16;
                    u_count = 16 * uniform.size();
                    value = UniformValue::Floats(Vec::with_capacity(u_count as usize));
                }
                _ => {
                    panic!("Invalid Uniform");
                }
            }

            let location = gl.get_uniform_location(program, &uniform.name()).unwrap();
            
            uniforms.insert(Atom::from(uniform.name()), Uniform {
                value: value,
                item_count: item_count,
                count: u_count as u32,
                location: location,
            });
        }

        return uniforms;
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

    pub fn uniform_i(&mut self, name: &Atom, v1: i32, v2: i32, v3: i32, v4: i32) -> Result<(), ()> {
        
        let u = self.uniforms.get_mut(name).ok_or(())?;
        let count = u.count;

        if let UniformValue::Int(a, b, c, d) = u.value {
            if count == 1 && a == v1 {
                return Ok( () );
            } else if count == 2 && a == v1 && b == v2 {
                return Ok( () );
            } else if count == 3 && a == v1 && b == v2 && c == v3 {
                return Ok( () );
            } else if a == v1 && b == v2 && c == v3 && d == v4 {
                return Ok( () );
            }
        } else {
            debug_assert!(false);
        }
        
        let gl = self.gl.upgrade().unwrap();
        
        if count == 1 {
            gl.uniform1i(Some(&u.location), v1);
            u.value = UniformValue::Int(v1, 0, 0, 0);
        } else if count == 2 {
            gl.uniform2i(Some(&u.location), v1, v2);
            u.value = UniformValue::Int(v1, v2, 0, 0);
        } else if count == 3 {
            gl.uniform3i(Some(&u.location), v1, v2, v3);
            u.value = UniformValue::Int(v1, v2, v3, 0);
        } else {
            gl.uniform4i(Some(&u.location), v1, v2, v3, v4);
            u.value = UniformValue::Int(v1, v2, v3, v4);
        }

        Ok(())
    }

    pub fn uniform_f(&mut self, name: &Atom, v1: f32, v2: f32, v3: f32, v4: f32) -> Result<(), ()> {
        let u = self.uniforms.get_mut(name).ok_or(())?;
        let count = u.count;

        if let UniformValue::Float(a, b, c, d) = u.value {
            if count == 1 && a == v1 {
                return Ok( () );
            } else if count == 2 && a == v1 && b == v2 {
                return Ok( () );
            } else if count == 3 && a == v1 && b == v2 && c == v3 {
                return Ok( () );
            } else if a == v1 && b == v2 && c == v3 && d == v4 {
                return Ok( () );
            }
        } else {
            debug_assert!(false);
        }
        
        let gl = self.gl.upgrade().unwrap();
        if count == 1 {
            gl.uniform1f(Some(&u.location), v1);
            u.value = UniformValue::Float(v1, 0.0, 0.0, 0.0);
        } else if count == 2 {
            gl.uniform2f(Some(&u.location), v1, v2);
            u.value = UniformValue::Float(v1, v2, 0.0, 0.0);
        } else if count == 3 {
            gl.uniform3f(Some(&u.location), v1, v2, v3);
            u.value = UniformValue::Float(v1, v2, v3, 0.0);
        } else {
            gl.uniform4f(Some(&u.location), v1, v2, v3, v4);
            u.value = UniformValue::Float(v1, v2, v3, v4);
        }
        Ok(())
    }

    pub fn uniform_iv(&mut self, name: &Atom, v: &[i32]) -> Result<(), ()> {
        let u = self.uniforms.get_mut(name).ok_or(())?;
        
        debug_assert!(v.len() == u.count as usize);

        if let UniformValue::Ints(ref old) = u.value {
            if old.as_slice().cmp(v) == Ordering::Equal {
                return Ok( () );
            }
        } else {
            debug_assert!(false);
        }
        
        let gl = self.gl.upgrade().unwrap();
        if u.item_count == 1 {
            gl.uniform1iv(Some(&u.location), v);
        } else if u.item_count == 2 {
            gl.uniform2iv(Some(&u.location), v);
        } else if u.item_count == 3 {
            gl.uniform3iv(Some(&u.location), v);
        } else {
            gl.uniform4iv(Some(&u.location), v);
        }

        if let UniformValue::Ints(ref mut target) = u.value {
            target.clone_from_slice(v);
        }

        Ok(())
    }

    pub fn uniform_fv(&mut self, name: &Atom, v: &[f32]) -> Result<(), ()> {
        let u = self.uniforms.get_mut(name).ok_or(())?;
        
        debug_assert!(v.len() == u.count as usize);

        if let UniformValue::Floats(ref old) = u.value {
            if old.as_slice().partial_cmp(v) == Some(Ordering::Equal) {
                return Ok( () );
            }
        } else {
            debug_assert!(false);
        }
        
        let gl = self.gl.upgrade().unwrap();
        if u.item_count == 1 {
            gl.uniform1fv(Some(&u.location), v);
        } else if u.item_count == 2 {
            gl.uniform2fv(Some(&u.location), v);
        } else if u.item_count == 3 {
            gl.uniform3fv(Some(&u.location), v);
        } else {
            gl.uniform4fv(Some(&u.location), v);
        }
        
        if let UniformValue::Floats(ref mut target) = u.value {
            target.clone_from_slice(v);
        }

        Ok(())
    }

    pub fn uniform_mat_v(&mut self, name: &Atom, v: &[f32]) -> Result<(), ()> {
        let u = self.uniforms.get_mut(name).ok_or(())?;
        
        debug_assert!(v.len() == u.count as usize);

        if let UniformValue::Floats(ref old) = u.value {
            if old.as_slice().partial_cmp(v) == Some(Ordering::Equal) {
                return Ok( () );
            }
        } else {
            debug_assert!(false);
        }
        
        let gl = self.gl.upgrade().unwrap();
        if u.item_count == 4 {
            gl.uniform_matrix2fv(Some(&u.location), false, v);
        } else if u.item_count == 9 {
            gl.uniform_matrix3fv(Some(&u.location), false, v);
        } else if u.item_count == 16 {
            gl.uniform_matrix4fv(Some(&u.location), false, v);
        }
        
        if let UniformValue::Floats(ref mut target) = u.value {
            target.clone_from_slice(v);
        }

        Ok(())
    }
}