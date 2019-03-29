use webgl_rendering_context::{WebGLRenderingContext};

use deque::deque::{Node as DeNode};
use slab::{Slab};
// use wcs::component::{SingleCase, SingleCaseWriteRef};
use wcs::world::{ComponentMgr};
use atom::Atom;

use component::node::*;
use component::math::{Point2};
use component::render::*;
use world::shader::{Shader, ShaderStore};
use shaders::*;
use render::engine::Engine;

world!(
    struct GuiComponentMgr{
        #[component]
        node: Node,
        node_container: Slab<DeNode<usize>>,
        // opaque_vector: VectorSdf,    //不透明渲染对象列表
        // render:  Render,
        root_id: usize,
        root_width: f32,
        root_height: f32,
        overflow: Overflow, // ([节点id 8个], [剪切矩形clip_rect 8个]), 每个矩形需要4个点定义。
        // gl: WebGLRenderingContext,
        sdf_shader: Shader,
        shader_store: ShaderStore,
        engine: Engine,

        #[component]
        sdf_program: SdfProgram,

        // #[component]
        // view_port: ViewPort,
        // root: usize,
        // transparent_vector: VectorSdf,    //透明的矢量图形
    } 
);

impl GuiComponentMgr {
    pub fn new(gl: WebGLRenderingContext) -> Self{
        let sdf_shader = Shader::new(Atom::from("sdf_sharder"), Atom::from("sdf_fs_sharder"), Atom::from("sdf_vs_sharder"));
        let mut shader_store = ShaderStore::new();
        shader_store.store(&sdf_shader.fs, sdf_fragment_shader());
        shader_store.store(&sdf_shader.vs, sdf_vertex_shader());

        GuiComponentMgr {
            node: NodeGroup::default(),
            node_container: Slab::default(),
            // render: Render::new(gl),
            // opaque_vector: VectorSdf::new(),
            root_id: 0,
            root_width: 0.0,
            root_height: 0.0,
            overflow: Overflow([0;8],[[Point2::default();4];8]),
            engine: Engine::new(gl),
            sdf_shader: sdf_shader,
            shader_store: shader_store,
            sdf_program:SdfProgramGroup::default(),
        }
    }
}

pub struct Overflow([usize;8], [[Point2;4];8]);

impl QidContainer for GuiComponentMgr {
    fn get_qid_container(&mut self) -> &mut Slab<DeNode<usize>>{
        &mut self.node_container
    }
}

impl GuiComponentMgr {
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.root_width = width;
        self.root_height = height;
    }

    pub fn set_root(&mut self, id: usize) {
        self.root_id = id;
    }
}