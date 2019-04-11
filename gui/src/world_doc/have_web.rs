use std::os::raw::{c_void};
use std::rc::Rc;

use webgl_rendering_context::{WebGLRenderingContext};

use deque::deque::{Node as DeNode};
use slab::{Slab};
use wcs::component::{Builder};
use wcs::world::{ComponentMgr, World, System};
use cg::octree::*;
use cg::{Aabb3, Point3};

use world_doc::font::{FontSheet};
use world_doc::component::node::*;
use world_doc::system::{layout::Layout as LayoutSys, world_matrix::WorldMatrix as WorldMatrixSys, oct::Oct as OctSys, opacity::OpacitySys, decorate::BBSys , run_world_2d::RunWorld2d as RunWorld2dSys};
use world_2d::World2dMgr;
use world_2d;

pub const Z_MAX: f32 = 8388608.0;

pub fn create_world(gl: WebGLRenderingContext) -> World<WorldDocMgr, ()>{
    let mut mgr = WorldDocMgr::new(gl);

    let layout_sys = LayoutSys::init(&mut mgr);
    let world_matrix_sys = WorldMatrixSys::init(&mut mgr);
    let oct_sys = OctSys::init(&mut mgr);
    let opacity_sys = OpacitySys::init(&mut mgr);
    let bb_sys = BBSys::init(&mut mgr);
    let run_world_2d_sys = RunWorld2dSys::init(&mut mgr);

    let mut world = World::new(mgr);
    let systems: Vec<Rc<System<(), WorldDocMgr>>> = vec![layout_sys, world_matrix_sys, oct_sys, opacity_sys, bb_sys, run_world_2d_sys];
    world.set_systems(systems);

    world
}

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
    pub fn new(gl: WebGLRenderingContext) -> Self{
        let mut mgr = WorldDocMgr {
            node: NodeGroup::default(),
            node_container: Slab::default(),
            root_id: 0,
            font: FontSheet::default(),
            octree: Tree::new(Aabb3::new(Point3::new(-1024f32,-1024f32,-8388608f32), Point3::new(3072f32,3072f32,8388608f32)), 0, 0, 0, 0),
            world_2d: world_2d::create_world(gl),
        };

        let root = NodeBuilder::new()
        .build(&mut mgr.node);

        //插入根节点, 不抛出创建事件
        mgr.node._group.insert(root, 0); 
        mgr.root_id = 1;

        mgr.get_node_mut(mgr.root_id).set_parent(0); //为根的子组件设置正确的parent

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
        {
            let root = self.node._group.get(self.root_id);
            root.yoga.set_width(width);
            root.yoga.set_height(height);
        }
        
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