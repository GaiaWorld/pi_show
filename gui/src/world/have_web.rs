use std::ops::{Deref, DerefMut};

use webgl_rendering_context::{WebGLRenderingContext};

use deque::deque::{Node as DeNode};
use slab::{Slab};
use wcs::component::{SingleCase, SingleCaseWriteRef};
use wcs::world::{ComponentMgr};
use atom::Atom;
use cg::octree::*;
use cg::{Aabb3, Point3};

use component::node::*;
use component::math::{Point2};
use component::render::*;
use world::shader::{Shader, ShaderStore};
use shaders::*;
use render::engine::Engine;

pub const Z_MAX: f32 = 8388608.0;

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
        // gl: WebGLRenderingContext,
        sdf_shader: Shader,
        shader_store: ShaderStore,
        engine: Engine,
        #[component]
        render_obj: RenderObj,

        octree: Tree<f32, usize>,
        #[single_component]
        overflow: Overflow, // ([节点id 8个], [剪切矩形clip_rect 8个]), 每个矩形需要4个点定义。

        world_view: GuiWorldViewProjection,

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

        let mgr = GuiComponentMgr {
            node: NodeGroup::default(),
            node_container: Slab::default(),
            // render: Render::new(gl),
            // opaque_vector: VectorSdf::new(),
            root_id: 0,
            root_width: 0.0,
            root_height: 0.0,
            engine: Engine::new(gl),
            sdf_shader: sdf_shader,
            shader_store: shader_store,
            render_obj: RenderObjGroup::default(),
            octree: Tree::new(Aabb3::new(Point3::new(-1024f32,-1024f32,-8388608f32), Point3::new(3072f32,3072f32,8388608f32)), 0, 0, 0, 0),
            overflow: SingleCase::new(Overflow([0;8],[[Point2::default();4];8])),
            world_view: GuiWorldViewProjection::new(0.0, 0.0),
        };
        // let root = NodeBuilder::new()
        // .build(&mut mgr.node);

        // mgr.root_id = mgr.add_node(root).id;

        mgr
    }
}
#[derive(Debug)]
pub struct Overflow(pub [usize;8], pub [[Point2;4];8]);

impl QidContainer for GuiComponentMgr {
    fn get_qid_container(&mut self) -> &mut Slab<DeNode<usize>>{
        &mut self.node_container
    }
}

impl GuiComponentMgr {
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.root_width = width;
        self.root_height = height;
        self.world_view = GuiWorldViewProjection::new(width, height);
    }

    pub fn get_root(&mut self) -> NodeReadRef<Self> {
        self.get_node(self.root_id)
    }

    pub fn get_root_mut(&mut self) -> NodeWriteRef<Self> {
        self.get_node_mut(self.root_id)
    }
}

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