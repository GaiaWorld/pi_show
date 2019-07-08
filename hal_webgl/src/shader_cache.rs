
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use stdweb::unstable::TryInto;
use webgl_rendering_context::{
    WebGLShader,
    WebGLRenderingContext,
};
use fnv::FnvHashMap;

use share::{Share};
use atom::{Atom};
use hal_core::*;
use context::{WebglHalContext};

/**
 * GPU Shader
 */
#[derive(Debug)]
pub struct Shader {
    shader_type: ShaderType,
    handle: WebGLShader,
}

/**
 * 程序管理器，管理shader的创建和生命周期
 * 注：shader创建很费时间，而占的显存较小；
 * 而且游戏不大的话，总的shader不会太多；
 * 因此已经创建的shader全部缓存。
 */
pub struct ShaderCache {
    
    // 代码缓存
    code_caches: FnvHashMap<Atom, String>,

    // Shader缓存的键是：hash[shader名 + defines]
    shader_caches: FnvHashMap<u64, Shader>,
}

impl ShaderCache {
    
    /**
     * 创建一个管理器
     * 注：一个App可能存在多个gl环境，因此ProgramManager不能是单例
     */
    pub fn new() -> ShaderCache {
        ShaderCache {
            code_caches: FnvHashMap::default(),
            shader_caches: FnvHashMap::default(),
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
    pub fn compile_shader(&mut self, context: &WebglHalContext, shader_type: ShaderType, name: &Atom, defines: &[Atom]) -> Result<u64, String> {
        
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
}