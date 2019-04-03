use std::ops::{Deref, DerefMut};
use std::os::raw::{c_void};

use webgl_rendering_context::{WebGLRenderingContext};

use deque::deque::{Node as DeNode};
use slab::{Slab};
use wcs::component::{Builder};
use wcs::world::{ComponentMgr, World};
use cg::octree::*;
use cg::{Aabb3, Point3};

use document::component::node::*;
use object2d::Object2dMgr;

pub const Z_MAX: f32 = 8388608.0;

world!(
    struct DocumentMgr{
        #[component]
        node: Node,
        node_container: Slab<DeNode<usize>>,

        root_id: usize,

        octree: Tree<f32, usize>,

        object2d: World<Object2dMgr, ()>,
    } 
);

impl DocumentMgr {
    pub fn new(gl: WebGLRenderingContext) -> Self{
        let mut mgr = DocumentMgr {
            node: NodeGroup::default(),
            node_container: Slab::default(),
            root_id: 0,
            octree: Tree::new(Aabb3::new(Point3::new(-1024f32,-1024f32,-8388608f32), Point3::new(3072f32,3072f32,8388608f32)), 0, 0, 0, 0),
            object2d: World::new(Object2dMgr::new(gl)),
        };

        let root = NodeBuilder::new()
        .build(&mut mgr.node);

        //设置yoga的上下文
        let yoga_context = Box::into_raw(Box::new(YogaContex {
            node_id: 1,
            mgr: &mgr as *const DocumentMgr as usize,
        })) as usize;
        root.yoga.set_context(yoga_context as *mut c_void);

        //插入根节点, 不抛出创建事件
        mgr.node._group.insert(root, 0); 
        mgr.root_id = 1;

        mgr
    }
}

impl QidContainer for DocumentMgr {
    fn get_qid_container(&mut self) -> &mut Slab<DeNode<usize>>{
        &mut self.node_container
    }
}

impl DocumentMgr {
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.object2d.component_mgr.set_size(width, height);
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