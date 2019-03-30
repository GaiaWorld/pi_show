// use stdweb::web::html_element::CanvasElement;
#[cfg(feature = "web")]
use webgl_rendering_context::{WebGLRenderingContext};

use deque::deque::{Node as DeNode};
use slab::{Slab};
use wcs::component::{SingleCase, SingleCaseWriteRef};
use wcs::world::{ComponentMgr};

use component::node::*;
use component::math::{Point2};
// use component::style::style::*;
// use component::viewport::*;
// use render::vector_sdf::VectorSdf;
// use render::render::Render;

#[cfg(not(feature = "web"))]
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
        #[single_component]
        overflow: Overflow, // ([节点id 8个], [剪切矩形clip_rect 8个]), 每个矩形需要4个点定义。
        // #[component]
        // view_port: ViewPort,
        // root: usize,
        // transparent_vector: VectorSdf,    //透明的矢量图形
    } 
);

#[cfg(not(feature = "web"))]
impl GuiComponentMgr {
    pub fn new() -> Self{
        GuiComponentMgr {
            node: NodeGroup::default(),
            node_container: Slab::default(),
            // render: Render::new(gl),
            // opaque_vector: VectorSdf::new(),
            root_id: 0,
            root_width: 0.0,
            root_height: 0.0,
            overflow: SingleCase::new(Overflow([0;8],[[Point2::default();4];8])),
        }
    }
}

#[cfg(feature = "web")]
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
        #[single_component]
        overflow: Overflow, // ([节点id 8个], [剪切矩形clip_rect 8个]), 每个矩形需要4个点定义。
        gl: WebGLRenderingContext,
        // #[component]
        // view_port: ViewPort,
        // root: usize,
        // transparent_vector: VectorSdf,    //透明的矢量图形
    }
);


#[cfg(feature = "web")]
impl GuiComponentMgr {
    pub fn new(gl: WebGLRenderingContext) -> Self{
        GuiComponentMgr {
            node: NodeGroup::default(),
            node_container: Slab::default(),
            // render: Render::new(gl),
            // opaque_vector: VectorSdf::new(),
            root_id: 0,
            root_width: 0.0,
            root_height: 0.0,
            gl: gl,
            overflow: SingleCase::new(Overflow([0;8],[[Point2::default();4];8])),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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
    }

    pub fn set_root(&mut self, id: usize) {
        self.root_id = id;
    }
}