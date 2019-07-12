use std::default::Default;
use std::sync::Arc;

use hal_core::{Context, RenderBeginDesc};
use atom::Atom;
use cgmath::One;

use share::Share;
use ecs::*;
use ecs::idtree::IdTree;
use ecs::Share as ShareTrait;
use component::user::*;
use component::calc::*;
use component::calc;
use component::user;
use layout::FlexNode;
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
    pub static ref FILTER_N: Atom = Atom::from("filter_sys");
    pub static ref WORLD_MATRIX_RENDER_N: Atom = Atom::from("world_matrix_render");
}

pub fn create_world<C: Context + ShareTrait, L: FlexNode>(mut engine: Engine<C>, width: f32, height: f32) -> World{
    let mut world = World::default();

    let mut default_table = DefaultTable::new();
    default_table.set::<TextStyle>(TextStyle::default());
    default_table.set::<Transform>(Transform::default());
    let mut font = Font::default();
    font.family = Atom::from("__$common");
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
    world.register_multi::<Node, CharBlock<L>>();
    world.register_multi::<Node, Text>();
    world.register_multi::<Node, TextStyleClass>();
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
    world.register_multi::<Node, L>();
    world.register_multi::<Node, HSV>();
    world.register_multi::<Node, WorldMatrixRender>();
    
    //single
    world.register_single::<ClipUbo<C>>(ClipUbo(Share::new(engine.gl.create_uniforms())));
    world.register_single::<IdTree>(IdTree::default());
    world.register_single::<Oct>(Oct::new());
    world.register_single::<TextStyleClassMap>(TextStyleClassMap::default());
    world.register_single::<OverflowClip>(OverflowClip::default());
    world.register_single::<RenderObjs<C>>(RenderObjs::<C>::default());
    world.register_single::<Engine<C>>(engine);
    world.register_single::<FontSheet<C>>(FontSheet::<C>::default());
    world.register_single::<ViewMatrix>(ViewMatrix(Matrix4::one()));
    world.register_single::<ProjectionMatrix>(ProjectionMatrix::new(width, height, -Z_MAX - 1.0, Z_MAX + 1.0));
    world.register_single::<RenderBegin>(RenderBegin(Share::new(RenderBeginDesc::new(0, 0, width as i32, height as i32))));
    world.register_single::<NodeRenderMap>(NodeRenderMap::new());
    world.register_single::<DefaultTable>(default_table);
    
    world.register_system(ZINDEX_N.clone(), CellZIndexImpl::new(ZIndexImpl::new()));
    world.register_system(SHOW_N.clone(), CellShowSys::new(ShowSys::default()));
    world.register_system(FILTER_N.clone(), CellFilterSys::new(FilterSys::default()));
    world.register_system(OPCITY_N.clone(), CellOpacitySys::new(OpacitySys::default()));
    world.register_system(LYOUT_N.clone(), CellLayoutSys::<L>::new(LayoutSys::new()));
    world.register_system(TEXT_LAYOUT_N.clone(), CellLayoutImpl::new(LayoutImpl::<C, L>::new()));
    world.register_system(WORLD_MATRIX_N.clone(), CellWorldMatrixSys::new(WorldMatrixSys::default()));
    world.register_system(OCT_N.clone(), CellOctSys::new(OctSys::default()));
    world.register_system(OVERFLOW_N.clone(), CellOverflowImpl::new(OverflowImpl));
    world.register_system(CLIP_N.clone(), CellClipSys::new(clip_sys));
    world.register_system(IMAGE_N.clone(), CellImageSys::new(ImageSys::<C>::new()));
    world.register_system(CHAR_BLOCK_N.clone(), CellCharBlockSys::<C, L>::new(CharBlockSys::new()));
    // world.register_system(CHAR_BLOCK_SHADOW_N.clone(), CellCharBlockShadowSys::<C, L>::new(CharBlockShadowSys::new()));
    world.register_system(BG_COLOR_N.clone(), CellBackgroundColorSys::new(BackgroundColorSys::<C>::new()));
    world.register_system(BR_COLOR_N.clone(), CellBorderColorSys::new(BorderColorSys::<C>::new()));
    world.register_system(BR_IMAGE_N.clone(), CellBorderImageSys::new(BorderImageSys::<C>::new()));
    world.register_system(BOX_SHADOW_N.clone(), CellBoxShadowSys::new(BoxShadowSys::<C>::new()));
    world.register_system(NODE_ATTR_N.clone(), CellNodeAttrSys::new(NodeAttrSys::<C>::new()));
    world.register_system(RENDER_N.clone(), CellRenderSys::new(RenderSys::<C>::default()));
    world.register_system(WORLD_MATRIX_RENDER_N.clone(), CellRenderMatrixSys::new(RenderMatrixSys::<C>::new()));

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("z_index_sys, show_sys, filter_sys, opacity_sys, layout_sys, text_layout_sys, world_matrix_sys, oct_sys, overflow_sys, world_matrix_render, background_color_sys, border_color_sys, box_shadow_sys, image_sys, border_image_sys, charblock_sys, node_attr_sys, clip_sys, render_sys".to_string(), &world);
    world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("text_layout_sys, world_matrix_sys, oct_sys".to_string(), &world);
    world.add_dispatcher(LAYOUT_DISPATCH.clone(), dispatch);

    world
}

pub struct GuiWorld<C: Context + ShareTrait, L: FlexNode> {
    pub node: Arc<CellEntity<Node>>,
    pub transform: Arc<CellMultiCase<Node, Transform>>,
    pub z_index: Arc<CellMultiCase<Node, user::ZIndex>>,
    pub overflow: Arc<CellMultiCase<Node, Overflow>>,
    pub show: Arc<CellMultiCase<Node, Show>>,
    pub opacity: Arc<CellMultiCase<Node, user::Opacity>>,
    pub background_color: Arc<CellMultiCase<Node, BackgroundColor>>,
    pub box_shadow: Arc<CellMultiCase<Node, BoxShadow>>,
    pub border_color: Arc<CellMultiCase<Node, BorderColor>>,
    pub border_image: Arc<CellMultiCase<Node, BorderImage<C>>>,
    pub border_image_clip: Arc<CellMultiCase<Node, BorderImageClip>>,
    pub border_image_slice: Arc<CellMultiCase<Node, BorderImageSlice>>,
    pub border_image_repeat: Arc<CellMultiCase<Node, BorderImageRepeat>>,
    pub text: Arc<CellMultiCase<Node, Text>>,
    pub text_style_class: Arc<CellMultiCase<Node, TextStyleClass>>,
    pub text_style: Arc<CellMultiCase<Node, TextStyle>>,
    pub text_shadow: Arc<CellMultiCase<Node, TextShadow>>,
    pub font: Arc<CellMultiCase<Node, Font>>,
    pub border_radius: Arc<CellMultiCase<Node, BorderRadius>>,
    pub image: Arc<CellMultiCase<Node, Image<C>>>,
    pub image_clip: Arc<CellMultiCase<Node, ImageClip>>,
    pub object_fit: Arc<CellMultiCase<Node, ObjectFit>>,
    pub filter: Arc<CellMultiCase<Node, Filter>>,
    pub yoga: Arc<CellMultiCase<Node, L>>,

    //calc
    pub z_depth: Arc<CellMultiCase<Node, ZDepth>>,
    pub enable: Arc<CellMultiCase<Node, Enable>>,
    pub visibility: Arc<CellMultiCase<Node, Visibility>>,
    pub world_matrix: Arc<CellMultiCase<Node, WorldMatrix>>,
    pub by_overflow: Arc<CellMultiCase<Node, ByOverflow>>,
    pub copacity: Arc<CellMultiCase<Node, calc::Opacity>>,
    pub layout: Arc<CellMultiCase<Node, Layout>>,
    pub hsv: Arc<CellMultiCase<Node, HSV>>,
    
    //single
    pub idtree: Arc<CellSingleCase<IdTree>>,
    pub oct: Arc<CellSingleCase<Oct>>,
    pub overflow_clip: Arc<CellSingleCase<OverflowClip>>,
    pub engine: Arc<CellSingleCase<Engine<C>>>,
    pub render_objs: Arc<CellSingleCase<RenderObjs<C>>>,
    pub font_sheet: Arc<CellSingleCase<FontSheet<C>>>,
    pub default_table: Arc<CellSingleCase<DefaultTable>>,

    pub world: World,
}

impl<C: Context + ShareTrait, L: FlexNode> GuiWorld<C, L> {
    pub fn new(world: World) -> GuiWorld<C, L>{
        GuiWorld{
            node: world.fetch_entity::<Node>().unwrap(),
            transform: world.fetch_multi::<Node, Transform>().unwrap(),
            z_index: world.fetch_multi::<Node, user::ZIndex>().unwrap(),
            overflow: world.fetch_multi::<Node, Overflow>().unwrap(),
            show: world.fetch_multi::<Node, Show>().unwrap(),
            opacity: world.fetch_multi::<Node, user::Opacity>().unwrap(),
            background_color: world.fetch_multi::<Node, BackgroundColor>().unwrap(),
            box_shadow: world.fetch_multi::<Node, BoxShadow>().unwrap(),
            border_color: world.fetch_multi::<Node, BorderColor>().unwrap(),
            border_image: world.fetch_multi::<Node, BorderImage<C>>().unwrap(),
            border_image_clip: world.fetch_multi::<Node, BorderImageClip>().unwrap(),
            border_image_slice: world.fetch_multi::<Node, BorderImageSlice>().unwrap(),
            border_image_repeat: world.fetch_multi::<Node, BorderImageRepeat>().unwrap(),
            text: world.fetch_multi::<Node, Text>().unwrap(),
            text_style_class: world.fetch_multi::<Node, TextStyleClass>().unwrap(),
            text_style: world.fetch_multi::<Node, TextStyle>().unwrap(),
            text_shadow: world.fetch_multi::<Node, TextShadow>().unwrap(),
            font: world.fetch_multi::<Node, Font>().unwrap(),
            border_radius: world.fetch_multi::<Node, BorderRadius>().unwrap(),
            image: world.fetch_multi::<Node, Image<C>>().unwrap(),
            image_clip: world.fetch_multi::<Node, ImageClip>().unwrap(),
            object_fit: world.fetch_multi::<Node, ObjectFit>().unwrap(),
            filter: world.fetch_multi::<Node, Filter>().unwrap(),
            yoga: world.fetch_multi::<Node, L>().unwrap(),

            //calc
            z_depth: world.fetch_multi::<Node, ZDepth>().unwrap(),
            enable: world.fetch_multi::<Node, Enable>().unwrap(),
            visibility: world.fetch_multi::<Node, Visibility>().unwrap(),
            world_matrix: world.fetch_multi::<Node, WorldMatrix>().unwrap(),
            by_overflow: world.fetch_multi::<Node, ByOverflow>().unwrap(),
            copacity: world.fetch_multi::<Node, calc::Opacity>().unwrap(),
            layout: world.fetch_multi::<Node, Layout>().unwrap(),
            hsv: world.fetch_multi::<Node, HSV>().unwrap(),
            
            //single
            idtree: world.fetch_single::<IdTree>().unwrap(),
            oct: world.fetch_single::<Oct>().unwrap(),
            overflow_clip: world.fetch_single::<OverflowClip>().unwrap(),
            engine: world.fetch_single::<Engine<C>>().unwrap(),
            render_objs: world.fetch_single::<RenderObjs<C>>().unwrap(),
            font_sheet: world.fetch_single::<FontSheet<C>>().unwrap(),
            default_table: world.fetch_single::<DefaultTable>().unwrap(),

            world: world,
        }
    }
}