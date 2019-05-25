use std::default::Default;

use hal_core::Context;
use atom::Atom;

use ecs::{World, SeqDispatcher, Dispatcher};
use ecs::idtree::IdTree;
use component::user::*;
use component::calc::*;
use component::calc;
use component::user;
use single::{ RenderObjs, oct::Oct };
use entity::Node;
use render::engine::Engine;
use system::*;

lazy_static! {

    pub static ref RENDER_DISPATCH: Atom = Atom::from("render_dispatch");
    pub static ref LAYOUT_DISPATCH: Atom = Atom::from("layout_dispatch");

    pub static ref ZINDEX_N: Atom = Atom::from("z_idnex_sys");
    pub static ref SHOW_N: Atom = Atom::from("show_sys");
    pub static ref WORLD_MATRIX_N: Atom = Atom::from("world_matrix_sys");
    pub static ref OCT_N: Atom = Atom::from("oct_sys");
    pub static ref LYOUT_N: Atom = Atom::from("layout_sys");
    pub static ref TEXT_LAYOUT_N: Atom = Atom::from("text_layout_sys");
    pub static ref CLIP_N: Atom = Atom::from("clip_sys");
    pub static ref OPCITY_N: Atom = Atom::from("opcity_sys");
    pub static ref OVERFLOW_N: Atom = Atom::from("overflow_sys");
    pub static ref RENDER_N: Atom = Atom::from("render_sys");
    pub static ref BG_COLOR_N: Atom = Atom::from("background_color_sys");
    pub static ref BR_COLOR_N: Atom = Atom::from("border_color_sys");
    pub static ref CHAR_BLOCK_N: Atom = Atom::from("charblock_sys");
}

pub fn create_world<C: Context + Sync + Send + 'static>(engine: Engine<C>, width: f32, height: f32) -> World{
    let mut world = World::default();

    //user
    world.register_entity::<Node>();
    world.register_multi::<Node, Transform>();;
    world.register_multi::<Node, user::ZIndex>();
    world.register_multi::<Node, Overflow>();
    world.register_multi::<Node, Show>();
    world.register_multi::<Node, BackgroundColor>();
    world.register_multi::<Node, BoxShadow>();
    world.register_multi::<Node, BorderColor>();
    world.register_multi::<Node, BorderImage<C>>();
    world.register_multi::<Node, BorderImageClip>();
    world.register_multi::<Node, BorderImageSlice>();
    world.register_multi::<Node, BorderImageRepeat>();
    world.register_multi::<Node, BorderRadius>();
    world.register_multi::<Node, Image<C>>();
    world.register_multi::<Node, ImageClip>();
    world.register_multi::<Node, ObjectFit>();

    //calc
    world.register_multi::<Node, ZDepth>();
    world.register_multi::<Node, Enable>();
    world.register_multi::<Node, Visibility>();
    world.register_multi::<Node, WorldMatrix>();
    world.register_multi::<Node, Layout>();
    world.register_multi::<Node, ByOverflow>();
    world.register_multi::<Node, calc::Opacity>();
    
    //single
    world.register_single::<IdTree>(IdTree::default());
    world.register_single::<Oct>(Oct::new());
    world.register_single::<RenderObjs<C>>(RenderObjs::<C>::default());

    world.register_system(ZINDEX_N.clone(), CellZIndexImpl::new(ZIndexImpl::new()));
    world.register_system(SHOW_N.clone(), CellShowSys::new(ShowSys::default()));
    world.register_system(OPCITY_N.clone(), CellOpacitySys::new(OpacitySys::default()));
    world.register_system(LYOUT_N.clone(), CellLayoutSys::new(LayoutSys));
    world.register_system(TEXT_LAYOUT_N.clone(), CellLayoutImpl::new(LayoutImpl::<C>::new()));
    world.register_system(WORLD_MATRIX_N.clone(), CellWorldMatrixSys::new(WorldMatrixSys::default()));
    world.register_system(OCT_N.clone(), CellOctSys::new(OctSys::default()));
    world.register_system(OVERFLOW_N.clone(), CellOverflowImpl::new(OverflowImpl));
    // world.register_system(CLIP_N.clone(), ClipSys::<C>::default());
    // world.register_system(SDF_N.clone(), CellSdfSys::new(SdfSys::<C>::new()));
    // world.register_system(IMAGE_N.clone(), CellImageSys::new(ImageSys::<C>::new()));
    // world.register_system(CHAR_BLOCK_N.clone(), CharBlockSys::<C>::default());
    world.register_system(BG_COLOR_N.clone(), CellBackgroundColorSys::new(BackgroundColorSys::<C>::new()));
    world.register_system(BR_COLOR_N.clone(), CellBorderColorSys::new(BorderColorSys::<C>::new()));
    world.register_system(RENDER_N.clone(), CellRenderSys::new(RenderSys::<C>::default()));

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("z_index_sys, show_sys, opcity_sys, layout_sys, text_layout_sys, world_matrix_sys, oct_sys, background_color_sys, border_color_sys, render_sys".to_string(), &world);
    world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);
    world
}
