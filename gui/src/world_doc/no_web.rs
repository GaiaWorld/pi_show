use std::os::raw::{c_void};

use deque::deque::{Node as DeNode};
use slab::{Slab};
use wcs::component::{Builder};
use wcs::world::{ComponentMgr, World};
use cg::octree::*;
use cg::{Aabb3, Point3};

use world_doc::font::{FontSheet};
use world_doc::component::node::*;
use world_2d::World2dMgr;

pub const Z_MAX: f32 = 4194304.0;

world!(
    struct WorldDocMgr{
        #[component]
        node: Node,
        node_container: Slab<DeNode<usize>>,

        root_id: usize,
        font: FontSheet,
        octree: Tree<f32, usize>,

        world_2d: World<World2dMgr, ()>,
    } 
);

impl WorldDocMgr {
    pub fn new() -> Self{
        let mut mgr = WorldDocMgr {
            node: NodeGroup::default(),
            node_container: Slab::default(),
            root_id: 0,
            font: FontSheet::default(),
            octree: Tree::new(Aabb3::new(Point3::new(-1024f32,-1024f32,-Z_MAX), Point3::new(3072f32,3072f32,Z_MAX)), 0, 0, 0, 0),
            world_2d: World::new(World2dMgr::new()),
        };

        let root = NodeBuilder::new()
        .build(&mut mgr.node);

        //插入根节点, 不抛出创建事件
        mgr.node._group.insert(root, 0); 
        mgr.root_id = 1;

        mgr
    }
}

impl QidContainer for WorldDocMgr {
    fn get_qid_container(&mut self) -> &mut Slab<DeNode<usize>>{
        &mut self.node_container
    }
}

impl WorldDocMgr {
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.world_2d.component_mgr.set_size(width, height);
    }
    pub fn get_root_id(&self) -> usize {
        1
    }
    pub fn get_root(&mut self) -> NodeReadRef<Self> {
        self.get_node(self.root_id)
    }

    pub fn get_root_mut(&mut self) -> NodeWriteRef<Self> {
        self.get_node_mut(self.root_id)
    }
}