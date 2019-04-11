use std::default::Default;
use std::rc::Rc;

use webgl_rendering_context::{WebGLRenderingContext, WebGLTexture, WebGLFramebuffer};
use cg::{Matrix4, Point2 as CgPoint2, Ortho};

use wcs::world::{ComponentMgr, World, System};
use wcs::component::{SingleCase, SingleCaseWriteRef};
use world_2d::component::image::*;
use world_2d::component::sdf::*;
use world_2d::component::char_block::*;
use world_2d::shaders::*;
use world_2d::system::create_effect::CreateEffect;
use world_2d::system::create_sdf_program::CreateSdfProgram;
use world_2d::system::render::Render;
use world_2d::system::clip::ClipSys;
use component::math::{Point2};
use render::engine::Engine;

pub fn create_world(gl: WebGLRenderingContext) -> World<World2dMgr, ()> {
    let mut mgr = World2dMgr::new(gl);

    let create_effect = CreateEffect::init(&mut mgr);
    let create_sdf_program = CreateSdfProgram::init(&mut mgr);
    // let clip = ClipSys::init(&mut mgr);
    let render = Render::init(&mut mgr);

    let mut world = World::new(mgr);
    let systems: Vec<Rc<System<(), World2dMgr>>> = vec![create_effect, create_sdf_program, render];
    world.set_systems(systems);

    world
}
//创建world_2d的system

fn create_overflow() -> Overflow{
    Overflow([0;8],[[Point2(CgPoint2::new(100.0, 100.0)), Point2(CgPoint2::new(200.0, 100.0)), Point2(CgPoint2::new(200.0, 200.0)), Point2(CgPoint2::new(100.0, 200.0))];8])
}    


world!(
    struct World2dMgr{
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
        view: Matrix4<f32>, //v
        projection: GuiWorldViewProjection, //p

        #[single_component]
        overflow: Overflow,
        sdf_shader: Shader,
        word_shader: Shader,
        image_shader: Shader,
        clip_shader: Shader,

        overflow_texture: RenderTarget,

        engine: Engine,

        shader_store: ShaderStore,
    } 
);

impl World2dMgr {
    pub fn new(gl: WebGLRenderingContext) -> World2dMgr{
        let sdf_shader = Shader::new(Atom::from("sdf_sharder"), Atom::from("sdf_fs_sharder"), Atom::from("sdf_vs_sharder"));
        let image_shader = Shader::new(Atom::from("image_sharder"), Atom::from("image_fs_sharder"), Atom::from("image_vs_sharder"));
        let word_shader = Shader::new(Atom::from("word_sharder"), Atom::from("word_fs_sharder"), Atom::from("word_vs_sharder"));
        let clip_shader = Shader::new(Atom::from("clip_sharder"), Atom::from("clip_fs_sharder"), Atom::from("clip_vs_sharder"));
        let mut shader_store = ShaderStore::new();
        shader_store.store(&sdf_shader.fs, sdf_fragment_shader());
        shader_store.store(&sdf_shader.vs, sdf_vertex_shader());

        shader_store.store(&clip_shader.fs, clip_fragment_shader());
        shader_store.store(&clip_shader.vs, clip_vertex_shader());

        World2dMgr{
            sdf: SdfGroup::default(),
            word: CharBlockGroup::default(),
            image: ImageGroup::default(),

            sdf_effect: SdfEffectGroup::default(),

            width: 0.0,
            height: 0.0,
            overflow: SingleCase::new(Overflow([0;8],[[Point2::default();4];8])),
            // overflow: SingleCase::new(create_overflow()),
            projection: GuiWorldViewProjection::new(0.0, 0.0),
            view: Matrix4::new(1.0, 0.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 0.0, 1.0),
            sdf_shader: sdf_shader,
            image_shader: image_shader,
            word_shader: word_shader,
            clip_shader: clip_shader,

            overflow_texture: RenderTarget::create(&gl),

            engine: Engine::new(gl),
            shader_store: shader_store,
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.projection = GuiWorldViewProjection::new(width, height);
    }
}

#[derive(Debug)]
pub struct Overflow(pub [usize;8], pub [[Point2;4];8]);

#[derive(Clone)]
pub struct GuiWorldViewProjection(pub Matrix4<f32>);

impl GuiWorldViewProjection {
    pub fn new(width: f32, height: f32) -> GuiWorldViewProjection{
        // let ortho = Ortho {
        //     left: 0.0,
        //     right: width,
        //     bottom: height, 
        //     top: 0.0,
        //     near: -10000.0,
        //     far: 10000.0,
        // };
        // GuiWorldViewProjection(Matrix4::from(ortho))
        let (left, right, top, bottom, near, far) = (0.0, width, 0.0, height, -8388607.0, 8388608.0);
        GuiWorldViewProjection(Matrix4::new(
                2.0 / (right - left),                  0.0,                               0.0,                        0.0,
                    0.0,                     2.0 / (top - bottom),                       0.0,                        0.0,
                    0.0,                              0.0,                       -2.0 / (far - near),   -(far + near) / (far - near),
            -(right + left) / (right - left), -(top + bottom) / (top - bottom),           0.0,                        1.0
        ))
    }
}

// impl Deref for GuiWorldViewProjection{
//     type Target = [f32];
//     fn deref(&self) -> &[f32]{
//         &[]
//     }
// }

// impl DerefMut for GuiWorldViewProjection{
//     fn deref_mut(&mut self) -> &mut [f32]{
//         &mut self.0
//     }
// }

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

pub struct RenderTarget {
    pub frambuffer: WebGLFramebuffer,
    pub texture: WebGLTexture,
}

impl RenderTarget {
    pub fn create(gl: &WebGLRenderingContext) -> RenderTarget{
        let frambuffer = gl.create_framebuffer().unwrap();
        let texture = gl.create_texture().unwrap();
        gl.active_texture(WebGLRenderingContext::TEXTURE0);
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&texture));
        gl.tex_image2_d::<&[u8]>(WebGLRenderingContext::TEXTURE_2D, 0, WebGLRenderingContext::RGB as i32, 1024, 1024, 0, WebGLRenderingContext::RGB, WebGLRenderingContext::UNSIGNED_BYTE, None);

        gl.bind_framebuffer(WebGLRenderingContext::FRAMEBUFFER, Some(&frambuffer));
        gl.framebuffer_texture2_d(WebGLRenderingContext::FRAMEBUFFER,WebGLRenderingContext::COLOR_ATTACHMENT0, WebGLRenderingContext::TEXTURE_2D, Some(&texture), 0);
        gl.bind_framebuffer(WebGLRenderingContext::FRAMEBUFFER, None);

        RenderTarget {
            frambuffer, 
            texture
        }
    }
}