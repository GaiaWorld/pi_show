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