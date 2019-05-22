use ecs::{World};
use ecs::idtree::IdTree;
use component::user::*;
use component::calc::*;
use entity::Node;

pub fn create_world() -> World{
    let mut world = World::default();

    //user
    world.register_entity::<Node>();
    world.register_multi::<Node, Transform>();
    world.register_multi::<Node, WorldMatrix>();
    world.register_multi::<Node, ZIndex>();
    world.register_multi::<Node, BoxColor>();
    world.register_multi::<Node, BoxShadow>();
    // world.register_multi::<Node, BorderImage>();
    world.register_multi::<Node, BorderRadius>();
    world.register_multi::<Node, Overflow>();
    world.register_multi::<Node, Show>();

    //calc
    world.register_multi::<Node, ZDepth>();
    world.register_multi::<Node, Enable>();
    world.register_multi::<Node, Visibility>();
    world.register_multi::<Node, WorldMatrix>();
    world.register_multi::<Node, Layout>();

    //single
    world.register_single::<IdTree>(IdTree::default());
    
    world
}
