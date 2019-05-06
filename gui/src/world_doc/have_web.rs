use std::os::raw::{c_void};

use deque::deque::{Node as DeNode};
use slab::{Slab};
use wcs::component::{Builder};
use wcs::world::{ComponentMgr, World};
use cg::octree::*;
use cg::{Aabb3, Point3};
use atom::Atom;

use font::font_sheet::{FontSheet};
use world_doc::component::node::*;
use world_doc::system::{layout::Layout as LayoutSys, world_matrix::WorldMatrix as WorldMatrixSys, oct::Oct as OctSys, opacity::OpacitySys, decorate::BBSys , run_world_2d::RunWorld2d as RunWorld2dSys};
use world_doc::system::node_count::NodeCountSys;
use world_doc::system::zindex::ZIndexSys;
use world_doc::system::overflow::OverflowSys;
use world_doc::system::image::ImageSys;
use world_doc::system::visibility::VisibilitySys;
use world_doc::system::enable::EnableSys;
use world_2d::World2dMgr;
use world_2d;
use render::engine::Engine;

pub const Z_MAX: f32 = 4194304.0;
lazy_static! {
    pub static ref LAYOUT_SYS: Atom = Atom::from("Layout_sys");
    pub static ref ALL: Atom = Atom::from("All");
}

pub fn create_world(engine: Engine, width: f32, height: f32) -> World<WorldDocMgr, ()>{
    let mut mgr = WorldDocMgr::new(engine, width, height);

    let node_count_sys = NodeCountSys::init(&mut mgr);
    let layout_sys = LayoutSys::init(&mut mgr);
    let world_matrix_sys = WorldMatrixSys::init(&mut mgr);
    let oct_sys = OctSys::init(&mut mgr);
    let opacity_sys = OpacitySys::init(&mut mgr);
    let bb_sys = BBSys::init(&mut mgr);
    let run_world_2d_sys = RunWorld2dSys::init(&mut mgr);
    let z_index_sys = ZIndexSys::init(&mut mgr);
    let overflow_sys = OverflowSys::init(&mut mgr);
    let image_sys = ImageSys::init(&mut mgr);
    let visibility_sys = VisibilitySys::init(&mut mgr);
    let enable_sys = EnableSys::init(&mut mgr);
    
    let system_names = [
        Atom::from("NodeCountSys"),
        Atom::from("ZIndexSys"),
        Atom::from("OverflowSys"),
        Atom::from("LayoutSys"),
        Atom::from("WorldMatrixSys"),
        Atom::from("OctSys"),
        Atom::from("OpacitySys"),
        Atom::from("VisibilitySys"),   
        Atom::from("EnableSys"),   
        Atom::from("BBSys"),
        Atom::from("ImageSys"),
        Atom::from("RunWorld2dSys"),
    ];

    let mut world = World::new(mgr);
    {
        world.register_system(system_names[0].clone(), node_count_sys);
        world.register_system(system_names[1].clone(), z_index_sys);
        world.register_system(system_names[2].clone(), overflow_sys);
        world.register_system(system_names[3].clone(), layout_sys);
        world.register_system(system_names[4].clone(), world_matrix_sys);
        world.register_system(system_names[5].clone(), oct_sys);
        world.register_system(system_names[6].clone(), opacity_sys);
        world.register_system(system_names[7].clone(), visibility_sys);
        world.register_system(system_names[8].clone(), enable_sys);
        world.register_system(system_names[9].clone(), bb_sys);
        world.register_system(system_names[10].clone(), image_sys);
        world.register_system(system_names[11].clone(), run_world_2d_sys);
    }
    world.add_systems(ALL.clone(), &mut system_names.iter()).unwrap();

    let layout_names = [
        system_names[3].clone(),
        system_names[4].clone(),
    ];

    world.add_systems(LAYOUT_SYS.clone(), &mut layout_names.iter()).unwrap();

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
    pub fn new(engine: Engine, width: f32, heigth: f32) -> Self{
        let mut mgr = WorldDocMgr {
            node: NodeGroup::default(),
            node_container: Slab::default(),
            root_id: 0,
            font: FontSheet::default(),
            octree: Tree::new(Aabb3::new(Point3::new(-1024f32,-1024f32,-Z_MAX), Point3::new(3072f32,3072f32,Z_MAX)), 0, 0, 0, 0),
            world_2d: world_2d::create_world(engine, -Z_MAX - 1.0, Z_MAX + 1.0, width, heigth),
        };

        let root = NodeBuilder::new()
        .build(&mut mgr.node);

        root.yoga.set_context(1 as *mut c_void);

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