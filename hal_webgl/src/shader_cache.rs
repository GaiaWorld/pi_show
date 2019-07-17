
use stdweb::unstable::TryInto;
use webgl_rendering_context::{
    WebGLShader,
    WebGLRenderingContext,
};
use fx_hashmap::{FxHashMap32};

use atom::{Atom};
use hal_core::*;
use convert::{get_shader_type};

/**
 * GPU Shader
 */
#[derive(Debug)]
pub struct Shader {
    pub shader_type: ShaderType,
    pub handle: WebGLShader,
}

pub struct LayoutLocation {
    pub textures: FxHashMap32<Atom, usize>,
    pub uniforms: FxHashMap32<Atom, (usize, usize)>,
}

/**
 * 程序管理器，管理shader的创建和生命周期
 * 注：shader创建很费时间，而占的显存较小；
 * 而且游戏不大的话，总的shader不会太多；
 * 因此已经创建的shader全部缓存。
 */
pub struct ShaderCache {
    
    // 代码缓存
    code_caches: FxHashMap32<Atom, String>,

    shader_caches: FxHashMap32<u64, Shader>,

    location_caches: FxHashMap32<(Atom, Atom), LayoutLocation>,
}

impl ShaderCache {
    
    /**
     * 创建一个管理器
     * 注：一个App可能存在多个gl环境，因此ProgramManager不能是单例
     */
    pub fn new() -> ShaderCache {
        ShaderCache {
            code_caches: FxHashMap32::default(),
            shader_caches: FxHashMap32::default(),
            location_caches: FxHashMap32::default(),
        }
    }

    /** 
     * 设置shader代码
     */
    pub fn set_shader_code(&mut self, name: &str, code: &str) {
        self.code_caches.insert(Atom::from(name), code.to_string());
    }
    
    pub fn get_location_map(&mut self, vs_name: &Atom, fs_name: &Atom, layout: &UniformLayout) -> &LayoutLocation {
        let vs_fs = (vs_name.clone(), fs_name.clone());
        
        if self.location_caches.get(&vs_fs).is_none() {
            let mut uniforms = FxHashMap32::default();
            for (i, ubo) in layout.uniforms.iter().enumerate() {
                for (j, u) in ubo.iter().enumerate() {
                    uniforms.insert(Atom::from(*u), (i, j));
                }
            }
            
            let mut textures = FxHashMap32::default();
            for (i, u) in layout.textures.iter().enumerate() {
                textures.insert(Atom::from(*u), i);
            }

            self.location_caches.insert(vs_fs.clone(), LayoutLocation {
                uniforms, textures,
            });
        }

        self.location_caches.get(&vs_fs).unwrap()
    }

    /**
     * 编译shader，返回shader对应的hash
     */
    pub fn compile_shader(&mut self, gl: &WebGLRenderingContext, shader_type: ShaderType, id: u64, name: &Atom, defines: &[Option<&str>]) -> Result<&Shader, String> {
        
        // 如果能找到，返回
        if self.shader_caches.get(&id).is_none() {
            let stype = get_shader_type(shader_type);
            let shader = gl.create_shader(stype).ok_or_else(|| String::from("Unable to create shader object"))?;

            let code = self.code_caches.get(name).ok_or_else(|| String::from("Unkown shader name"))?;

            // 将宏定义放到shader代码的开头
            let mut s = "".to_string();
            for d in defines {
                if d.is_some() {
                    s += "#define ";
                    s += d.unwrap();
                    s += "\n";
                }
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
                self.shader_caches.insert(id, Shader {
                    shader_type: shader_type,
                    handle: shader,
                });
            } else {
                let err = gl.get_shader_info_log(&shader)
                    .unwrap_or_else(|| "Unknown error creating shader".into());

                debug_println!("Shader, compile_shader error, info = {:?}", &err);
                return Err(err);
            }
        }

        Ok(self.shader_caches.get(&id).unwrap())
    }
}