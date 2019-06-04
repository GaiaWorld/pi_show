use std::default::Default;
use std::sync::Arc;

use hal_core::{Context, RenderBeginDesc};
use atom::Atom;
use cgmath::One;

use ecs::{World, SeqDispatcher, Dispatcher};
use ecs::idtree::IdTree;
use component::user::*;
use component::calc::*;
use component::calc;
use component::user;
use layout::YgNode;
use single::*;
use entity::Node;
use render::engine::Engine;
use system::*;
use font::font_sheet::FontSheet;
use Z_MAX;

lazy_static! {

    pub static ref RENDER_DISPATCH: Atom = Atom::from("render_dispatch");
    pub static ref LAYOUT_DISPATCH: Atom = Atom::from("layout_dispatch");

    pub static ref ZINDEX_N: Atom = Atom::from("z_index_sys");
    pub static ref SHOW_N: Atom = Atom::from("show_sys");
    pub static ref WORLD_MATRIX_N: Atom = Atom::from("world_matrix_sys");
    pub static ref OCT_N: Atom = Atom::from("oct_sys");
    pub static ref LYOUT_N: Atom = Atom::from("layout_sys");
    pub static ref TEXT_LAYOUT_N: Atom = Atom::from("text_layout_sys");
    pub static ref CLIP_N: Atom = Atom::from("clip_sys");
    pub static ref OPCITY_N: Atom = Atom::from("opacity_sys");
    pub static ref OVERFLOW_N: Atom = Atom::from("overflow_sys");
    pub static ref RENDER_N: Atom = Atom::from("render_sys");
    pub static ref BG_COLOR_N: Atom = Atom::from("background_color_sys");
    pub static ref BOX_SHADOW_N: Atom = Atom::from("box_shadow_sys");
    pub static ref BR_COLOR_N: Atom = Atom::from("border_color_sys");
    pub static ref BR_IMAGE_N: Atom = Atom::from("border_image_sys");
    pub static ref IMAGE_N: Atom = Atom::from("image_sys");
    pub static ref CHAR_BLOCK_N: Atom = Atom::from("charblock_sys");
    pub static ref CHAR_BLOCK_SHADOW_N: Atom = Atom::from("charblock_shadow_sys");
    pub static ref NODE_ATTR_N: Atom = Atom::from("node_attr_sys");
}

pub fn create_world<C: Context + Sync + Send + 'static>(mut engine: Engine<C>, width: f32, height: f32) -> World{
    let mut world = World::default();

    let mut default_table = DefaultTable::new();
    default_table.set::<TextStyle>(TextStyle::default());
    let mut font = Font::default();
    font.family = Atom::from("common");
    default_table.set::<Font>(font);

    let clip_sys = ClipSys::<C>::new(&mut engine, width as u32, height as u32);

    //user
    world.register_entity::<Node>();
    world.register_multi::<Node, Transform>();;
    world.register_multi::<Node, user::ZIndex>();
    world.register_multi::<Node, Overflow>();
    world.register_multi::<Node, Show>();
    world.register_multi::<Node, user::Opacity>();
    world.register_multi::<Node, BackgroundColor>();
    world.register_multi::<Node, BoxShadow>();
    world.register_multi::<Node, BorderColor>();
    world.register_multi::<Node, BorderImage<C>>();
    world.register_multi::<Node, BorderImageClip>();
    world.register_multi::<Node, BorderImageSlice>();
    world.register_multi::<Node, BorderImageRepeat>();
    world.register_multi::<Node, CharBlock>();
    world.register_multi::<Node, Text>(); 
    world.register_multi::<Node, TextStyle>();
    world.register_multi::<Node, TextShadow>();
    world.register_multi::<Node, Font>();
    world.register_multi::<Node, BorderRadius>();
    world.register_multi::<Node, Image<C>>();
    world.register_multi::<Node, ImageClip>();
    world.register_multi::<Node, ObjectFit>();
    world.register_multi::<Node, Filter>();


    //calc
    world.register_multi::<Node, ZDepth>();
    world.register_multi::<Node, Enable>();
    world.register_multi::<Node, Visibility>();
    world.register_multi::<Node, WorldMatrix>();
    world.register_multi::<Node, ByOverflow>();
    world.register_multi::<Node, calc::Opacity>();
    world.register_multi::<Node, Layout>();
    world.register_multi::<Node, YgNode>();
    world.register_multi::<Node, HSV>();
    
    //single
    world.register_single::<ClipUbo<C>>(ClipUbo(Arc::new(engine.gl.create_uniforms())));
    world.register_single::<IdTree>(IdTree::default());
    world.register_single::<Oct>(Oct::new());
    world.register_single::<OverflowClip>(OverflowClip::default());
    world.register_single::<RenderObjs<C>>(RenderObjs::<C>::default());
    world.register_single::<Engine<C>>(engine);
    world.register_single::<FontSheet<C>>(FontSheet::<C>::default());
    world.register_single::<ViewMatrix>(ViewMatrix(Matrix4::one()));
    world.register_single::<ProjectionMatrix>(ProjectionMatrix::new(width, height, -Z_MAX - 1.0, Z_MAX + 1.0));
    world.register_single::<ViewPort>(ViewPort(Arc::new(RenderBeginDesc::new(0, 0, width as i32, height as i32))));
    world.register_single::<NodeRenderMap>(NodeRenderMap::new());
    world.register_single::<DefaultTable>(default_table);
    
    world.register_system(ZINDEX_N.clone(), CellZIndexImpl::new(ZIndexImpl::new()));
    world.register_system(SHOW_N.clone(), CellShowSys::new(ShowSys::default()));
    world.register_system(OPCITY_N.clone(), CellOpacitySys::new(OpacitySys::default()));
    world.register_system(LYOUT_N.clone(), CellLayoutSys::new(LayoutSys));
    world.register_system(TEXT_LAYOUT_N.clone(), CellLayoutImpl::new(LayoutImpl::<C>::new()));
    world.register_system(WORLD_MATRIX_N.clone(), CellWorldMatrixSys::new(WorldMatrixSys::default()));
    world.register_system(OCT_N.clone(), CellOctSys::new(OctSys::default()));
    world.register_system(OVERFLOW_N.clone(), CellOverflowImpl::new(OverflowImpl));
    world.register_system(CLIP_N.clone(), CellClipSys::new(clip_sys));
    // // world.register_system(SDF_N.clone(), CellSdfSys::new(SdfSys::<C>::new()));
    world.register_system(IMAGE_N.clone(), CellImageSys::new(ImageSys::<C>::new()));
    world.register_system(CHAR_BLOCK_N.clone(), CellCharBlockSys::<C>::new(CharBlockSys::new()));
    world.register_system(CHAR_BLOCK_SHADOW_N.clone(), CellCharBlockShadowSys::<C>::new(CharBlockShadowSys::new()));
    world.register_system(BG_COLOR_N.clone(), CellBackgroundColorSys::new(BackgroundColorSys::<C>::new()));
    world.register_system(BR_COLOR_N.clone(), CellBorderColorSys::new(BorderColorSys::<C>::new()));
    world.register_system(BR_IMAGE_N.clone(), CellBorderImageSys::new(BorderImageSys::<C>::new()));
    world.register_system(BOX_SHADOW_N.clone(), CellBoxShadowSys::new(BoxShadowSys::<C>::new()));
    world.register_system(NODE_ATTR_N.clone(), CellNodeAttrSys::new(NodeAttrSys::<C>::new()));
    world.register_system(RENDER_N.clone(), CellRenderSys::new(RenderSys::<C>::default()));

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("z_index_sys, show_sys, opacity_sys, layout_sys, text_layout_sys, world_matrix_sys, oct_sys, overflow_sys, clip_sys, background_color_sys, border_color_sys, box_shadow_sys, image_sys, border_image_sys, charblock_sys, charblock_shadow_sys, node_attr_sys, render_sys".to_string(), &world);
    world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("layout_sys, world_matrix_sys, oct_sys".to_string(), &world);
    world.add_dispatcher(LAYOUT_DISPATCH.clone(), dispatch);
    world
}
