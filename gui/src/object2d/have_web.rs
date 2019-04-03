use std::default::Default;
use std::ops::{Deref, DerefMut};

use webgl_rendering_context::{WebGLRenderingContext};

use wcs::world::{ComponentMgr};
use wcs::component::{SingleCase, SingleCaseWriteRef};
use object2d::component::image::*;
use object2d::component::sdf::*;
use object2d::component::char_block::*;
use object2d::shaders::*;
use generic_component::math::{Point2};
use render::engine::Engine;

world!(
    struct Object2dMgr{
        #[component]
        sdf: Sdf,

        #[component]
        word: CharBlock,

        #[component]
        image: Image,

        #[component]
        sdf_effect: SdfEffect,


        //全局数据
        width: f32,
        height: f32,
        world_view: GuiWorldViewProjection,
        #[single_component]
        overflow: Overflow,
        sdf_shader: Shader,
        word_shader: Shader,
        image_shader: Shader,

        engine: Engine,

        shader_store: ShaderStore,
    } 
);

impl Object2dMgr {
    pub fn new(gl: WebGLRenderingContext) -> Object2dMgr{
        let sdf_shader = Shader::new(Atom::from("sdf_sharder"), Atom::from("sdf_fs_sharder"), Atom::from("sdf_vs_sharder"));
        let image_shader = Shader::new(Atom::from("image_sharder"), Atom::from("image_fs_sharder"), Atom::from("image_vs_sharder"));
        let word_shader = Shader::new(Atom::from("word_sharder"), Atom::from("word_fs_sharder"), Atom::from("word_vs_sharder"));
        let mut shader_store = ShaderStore::new();
        shader_store.store(&sdf_shader.fs, sdf_fragment_shader());
        shader_store.store(&sdf_shader.vs, sdf_vertex_shader());

        Object2dMgr{
            sdf: SdfGroup::default(),
            word: CharBlockGroup::default(),
            image: ImageGroup::default(),

            sdf_effect: SdfEffectGroup::default(),

            width: 0.0,
            height: 0.0,
            overflow: SingleCase::new(Overflow([0;8],[[Point2::default();4];8])),
            world_view: GuiWorldViewProjection::new(0.0, 0.0),
            sdf_shader: sdf_shader,
            image_shader: image_shader,
            word_shader: word_shader,


            engine: Engine::new(gl),
            shader_store: shader_store,
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.world_view = GuiWorldViewProjection::new(width, height);
    }
}

#[derive(Debug)]
pub struct Overflow(pub [usize;8], pub [[Point2;4];8]);

#[derive(Clone)]
pub struct GuiWorldViewProjection([f32; 16]);

impl GuiWorldViewProjection {
    pub fn new(width: f32, height: f32) -> GuiWorldViewProjection{
        let (left, right, top, bottom, near, far) = (0.0, width, 0.0, height, 0.1, 1000.0);
        GuiWorldViewProjection([
                2.0 / (right - left),                  0.0,                               0.0,                        0.0,
                    0.0,                     2.0 / (top - bottom),                       0.0,                        0.0,
                    0.0,                              0.0,                       -2.0 / (far - near),   -(far + near) / (far - near),
            -(right + left) / (right - left), -(top + bottom) / (top - bottom),           0.0,                        1.0
            ]
        )
    }
}

impl Deref for GuiWorldViewProjection{
    type Target = [f32];
    fn deref(&self) -> &[f32]{
        &self.0
    }
}

impl DerefMut for GuiWorldViewProjection{
    fn deref_mut(&mut self) -> &mut [f32]{
        &mut self.0
    }
}

use std::hash::{Hash, Hasher};
use std::convert::AsRef;

use fnv::FnvHashMap;
use atom::Atom;

pub struct Shader {
    pub vs: Atom,
    pub fs: Atom,
    pub name: Atom,
}

impl Shader {
    pub fn new (name: Atom, vs: Atom, fs: Atom) -> Shader {
        Shader {
            vs,
            fs,
            name,
        }
    }
}

pub struct ShaderStore {
    shaders: FnvHashMap<Atom, ShaderCode>,
}

impl ShaderStore {
    pub fn new() -> ShaderStore {
        ShaderStore {
            shaders: FnvHashMap::default(),
        }
    }

    pub fn store(&mut self, name: &Atom, code: String){
        self.shaders.insert(name.clone(), ShaderCode::new(name.clone(), code));
    }

    pub fn remove(&mut self, name: &Atom) {
        self.shaders.remove(name);
    }

    pub fn get(&self, name: &Atom) -> Option<&ShaderCode> {
        self.shaders.get(name)
    }
}

pub struct ShaderCode {
    name: Atom,
    code: String,
}

impl ShaderCode {
    pub fn new (name: Atom, code: String) -> ShaderCode {
        ShaderCode {
            name,
            code,
        }
    }
}

impl Hash for ShaderCode{
    fn hash<H>(&self, state: &mut H) where H: Hasher{
        self.name.hash(state);
    }
}

impl AsRef<str> for ShaderCode {
    fn as_ref(&self) -> &str {
        &self.code
    }
}