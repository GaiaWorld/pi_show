// use stdweb::web::html_element::CanvasElement;
use webgl_rendering_context::{WebGLRenderingContext};

use deque::deque::{Node as DeNode};
use slab::{Slab};
// use wcs::component::{ComponentGroupTree};
use wcs::world::{ComponentMgr};

use component::node::*;
// use component::style::style::*;
// use component::viewport::*;
use render::vector_sdf::VectorSdf;
use render::render::Render;

world!(
    struct GuiComponentMgr{
        #[component]
        node: Node,
        node_container: Slab<DeNode<usize>>,
        opaque_vector: VectorSdf,    //不透明渲染对象列表
        render:  Render,
        root_id: usize,
        root_width: f32,
        root_height: f32,
        // #[component]
        // view_port: ViewPort,
        // root: usize,
        // transparent_vector: VectorSdf,    //透明的矢量图形
    } 
);

impl QidContainer for GuiComponentMgr {
    fn get_qid_container(&mut self) -> &mut Slab<DeNode<usize>>{
        &mut self.node_container
    }
}

impl GuiComponentMgr {
    pub fn new(gl: WebGLRenderingContext) -> Self{
        GuiComponentMgr {
            node: NodeGroup::default(),
            node_container: Slab::default(),
            render: Render::new(gl),
            opaque_vector: VectorSdf::new(),
            root_id: 0,
            root_width: 0.0,
            root_height: 0.0,
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.root_width = width;
        self.root_height = height;
    }

    pub fn set_root(&mut self, id: usize) {
        self.root_id = id;
    }
}