use deque::deque::{Node as DeNode};
use slab::{Slab};
use wcs::component::{ComponentGroupTree};
use wcs::world::{ComponentMgr};

use component::node::*;
use component::viewport::*;
use render::vector_sdf::VectorSdf;

world!(
    struct GuiComponentMgr{
        #[component]
        node: Node,
        node_container: Slab<DeNode<usize>>,
        opaque_vector: VectorSdf,    //不透明渲染对象列表
        #[component]
        world_view: ViewPort,
        // transparent_vector: VectorSdf,    //透明的矢量图形
    } 
);

impl Default for GuiComponentMgr {
    fn default() -> Self {
        GuiComponentMgr{
            node: NodeGroup::default(),
            node_container: Slab::default(),
            opaque_vector: VectorSdf::new(),
            world_view: ViewPortGroup::default(),
        }
    }
}

impl QidContainer for GuiComponentMgr {
    fn get_qid_container(&mut self) -> &mut Slab<DeNode<usize>>{
        &mut self.node_container
    }
}