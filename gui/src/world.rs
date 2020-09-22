use std::default::Default;
use std::sync::Arc;

use atom::Atom;
use cgmath::One;
use hal_core::*;

use ecs::idtree::IdTree;
use ecs::*;
use res::ResMgr;
use share::Share;

use component::calc;
use component::calc::*;
use component::user;
use component::user::*;
use entity::Node;
use font::font_sheet::FontSheet;
use layout::FlexNode;
use render::engine::ShareEngine;
use render::res::*;
use single::*;
use system::util::constant::*;
use system::*;
use Z_MAX;

lazy_static! {
    pub static ref RENDER_DISPATCH: Atom = Atom::from("render_dispatch");
	pub static ref LAYOUT_DISPATCH: Atom = Atom::from("layout_dispatch");
	pub static ref CALC_DISPATCH: Atom = Atom::from("cal_dispatch");
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
    pub static ref TEXT_GLPHY_N: Atom = Atom::from("text_glphy_sys");
    pub static ref CHAR_BLOCK_SHADOW_N: Atom = Atom::from("charblock_shadow_sys");
    pub static ref NODE_ATTR_N: Atom = Atom::from("node_attr_sys");
    pub static ref FILTER_N: Atom = Atom::from("filter_sys");
    pub static ref WORLD_MATRIX_RENDER_N: Atom = Atom::from("world_matrix_render");
    pub static ref RES_RELEASE_N: Atom = Atom::from("res_release");
    pub static ref STYLE_MARK_N: Atom = Atom::from("style_mark_sys");
    pub static ref TRANSFORM_WILL_CHANGE_N: Atom = Atom::from("transform_will_change_sys");
}

pub fn create_res_mgr(total_capacity: usize) -> ResMgr {
    let mut res_mgr = if total_capacity > 0 {
        ResMgr::with_capacity(total_capacity)
    } else {
        ResMgr::default()
    };

    res_mgr.register::<TextureRes>(
		10 * 1024 * 1024,
		50 * 1024 * 1024,
		5 * 60000,
		0,
        "TextureRes".to_string(),
    );
    res_mgr.register::<GeometryRes>(
        20 * 1024, 100 * 1024, 5 * 60000, 0,
        "GeometryRes".to_string(),
    );
    res_mgr.register::<BufferRes>(
        20 * 1024, 100 * 1024, 5 * 60000, 0,
        "BufferRes".to_string(),
    );

    res_mgr.register::<SamplerRes>(
        512, 1024, 60 * 60000, 0,
        "SamplerRes".to_string(),
    );
    res_mgr.register::<RasterStateRes>(
        512, 1024, 60 * 60000, 0,
        "RasterStateRes".to_string(),
    );
    res_mgr.register::<BlendStateRes>(
        512, 1024, 60 * 60000, 0,
        "BlendStateRes".to_string(),
    );
    res_mgr.register::<StencilStateRes>(
        512, 1024, 60 * 60000, 0,
        "StencilStateRes".to_string(),
    );
    res_mgr.register::<DepthStateRes>(
        512, 1024, 60 * 60000, 0,
        "DepthStateRes".to_string(),
    );

    res_mgr.register::<UColorUbo>(
        4 * 1024, 8 * 1024, 60 * 60000, 0,
        "UColorUbo".to_string(),
    );
    res_mgr.register::<HsvUbo>(
        1 * 1024, 2 * 1024, 60 * 60000, 0,
        "HsvUbo".to_string(),
    );
    res_mgr.register::<MsdfStrokeUbo>(
        1 * 1024, 2 * 1024, 60 * 60000, 0,
        "MsdfStrokeUbo".to_string(),
    );
    res_mgr.register::<CanvasTextStrokeColorUbo>(
        1 * 1024, 2 * 1024, 60 * 60000, 0,
        "CanvasTextStrokeColorUbo".to_string(),
    );
    res_mgr
}

pub fn create_world<L: FlexNode, C: HalContext + 'static>(
	mut engine: ShareEngine<C>,
    width: f32,
    height: f32,
    font_measure: Box<dyn Fn(&Atom, usize, char) -> f32>,
	font_texture: Share<TextureRes>,
	cur_time: u64,
) -> World {
    let mut world = World::default();

    let mut default_table = DefaultTable::new();
    default_table.set::<TextStyle>(TextStyle::default());
    default_table.set::<Transform>(Transform::default());
    let mut font = Font::default();
    font.family = Atom::from("__$common");
    default_table.set::<Font>(font);

    let positions = engine.create_buffer_res(
        POSITIONUNIT.get_hash() as u64,
        BufferType::Attribute,
        8,
        Some(BufferData::Float(&[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0])),
        false,
    );
    let indices = engine.create_buffer_res(
        INDEXUNIT.get_hash() as u64,
        BufferType::Indices,
        6,
        Some(BufferData::Short(&[0, 1, 2, 0, 2, 3])),
        false,
    );

    let geo = engine.create_geometry();
    engine
        .gl
        .geometry_set_attribute(&geo, &AttributeName::Position, &positions, 2)
        .unwrap();
    engine
        .gl
        .geometry_set_indices_short(&geo, &indices)
        .unwrap();
    let unit_quad = UnitQuad(Share::new(GeometryRes {
        geo: geo,
        buffers: vec![indices, positions],
    }));

    let default_state = DefaultState::new(&engine.gl);

    let charblock_sys = CellCharBlockSys::<L, C>::new(CharBlockSys::new(
        &mut engine,
        (font_texture.width, font_texture.height),
    ));
    let border_image_sys = BorderImageSys::<C>::new(&mut engine);
    let node_attr_sys = CellNodeAttrSys::<C>::new(NodeAttrSys::new(&engine.res_mgr));

    let clip_sys = ClipSys::<C>::new();
	let image_sys = CellImageSys::new(ImageSys::new(&mut engine));
	let mut sys_time = SystemTime::default();
	sys_time.start_time = cur_time;

    //user
    world.register_entity::<Node>();
    world.register_multi::<Node, Transform>();
    world.register_multi::<Node, user::ZIndex>();
    world.register_multi::<Node, Overflow>();
    world.register_multi::<Node, Show>();
    world.register_multi::<Node, user::Opacity>();
    world.register_multi::<Node, BackgroundColor>();
    world.register_multi::<Node, BoxShadow>();
    world.register_multi::<Node, BorderColor>();
    world.register_multi::<Node, BorderImage>();
    world.register_multi::<Node, BorderImageClip>();
    world.register_multi::<Node, BorderImageSlice>();
    world.register_multi::<Node, BorderImageRepeat>();
    world.register_multi::<Node, CharBlock<L>>();
    world.register_multi::<Node, TextStyle>();
    world.register_multi::<Node, TextContent>();
    world.register_multi::<Node, Font>();
    world.register_multi::<Node, BorderRadius>();
    world.register_multi::<Node, Image>();
    world.register_multi::<Node, ImageClip>();
    world.register_multi::<Node, ObjectFit>();
    world.register_multi::<Node, Filter>();
    world.register_multi::<Node, ClassName>();
    world.register_multi::<Node, StyleMark>();
    world.register_multi::<Node, TransformWillChange>();

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
    world.register_multi::<Node, Culling>();
    world.register_multi::<Node, TransformWillChangeMatrix>();

    //single
    world.register_single::<Statistics>(Statistics::default());
    world.register_single::<IdTree>(IdTree::default());
    world.register_single::<Oct>(Oct::new());
    world.register_single::<OverflowClip>(OverflowClip::default());
    world.register_single::<RenderObjs>(RenderObjs::default());
    world.register_single::<ShareEngine<C>>(engine);
    world.register_single::<FontSheet>(FontSheet::new(font_texture, font_measure));
    world.register_single::<ViewMatrix>(ViewMatrix(WorldMatrix(Matrix4::one(), false)));
    world.register_single::<ProjectionMatrix>(ProjectionMatrix::new(
        width,
        height,
        -Z_MAX - 1.0,
        Z_MAX + 1.0,
    ));
    world.register_single::<RenderBegin>(RenderBegin(
        RenderBeginDesc::new(0, 0, width as i32, height as i32),
        None,
    ));
    world.register_single::<NodeRenderMap>(NodeRenderMap::new());
    world.register_single::<DefaultTable>(default_table);
    world.register_single::<ClassSheet>(ClassSheet::default());
    world.register_single::<UnitQuad>(unit_quad);
    world.register_single::<DefaultState>(default_state);
    world.register_single::<ImageWaitSheet>(ImageWaitSheet::default());
	world.register_single::<DirtyList>(DirtyList::default());
	world.register_single::<SystemTime>(sys_time);

    world.register_system(ZINDEX_N.clone(), CellZIndexImpl::new(ZIndexImpl::new()));
    world.register_system(SHOW_N.clone(), CellShowSys::new(ShowSys::default()));
    world.register_system(FILTER_N.clone(), CellFilterSys::new(FilterSys::default()));
    world.register_system(OPCITY_N.clone(), CellOpacitySys::new(OpacitySys::default()));
    world.register_system(LYOUT_N.clone(), CellLayoutSys::<L>::new(LayoutSys::new()));
    world.register_system(
        TEXT_LAYOUT_N.clone(),
        CellLayoutImpl::new(LayoutImpl::<L>::new()),
    );
    world.register_system(
        WORLD_MATRIX_N.clone(),
        CellWorldMatrixSys::new(WorldMatrixSys::default()),
    );
    world.register_system(OCT_N.clone(), CellOctSys::new(OctSys::default()));
    world.register_system(
        OVERFLOW_N.clone(),
        CellOverflowImpl::new(OverflowImpl::default()),
    );
    world.register_system(IMAGE_N.clone(), image_sys);
    world.register_system(CHAR_BLOCK_N.clone(), charblock_sys);
    world.register_system(
        TEXT_GLPHY_N.clone(),
        CellTextGlphySys::<L>::new(TextGlphySys::new()),
    );
    world.register_system(
        TRANSFORM_WILL_CHANGE_N.clone(),
        CellTransformWillChangeSys::new(TransformWillChangeSys::default()),
    );

    // world.register_system(CHAR_BLOCK_SHADOW_N.clone(), CellCharBlockShadowSys::<L>::new(CharBlockShadowSys::new()));
    world.register_system(
        BG_COLOR_N.clone(),
        CellBackgroundColorSys::<C>::new(BackgroundColorSys::new()),
    );
    world.register_system(
        BR_COLOR_N.clone(),
        CellBorderColorSys::<C>::new(BorderColorSys::new()),
    );
    world.register_system(
        BR_IMAGE_N.clone(),
        CellBorderImageSys::new(border_image_sys),
    );
    world.register_system(
        BOX_SHADOW_N.clone(),
        CellBoxShadowSys::<C>::new(BoxShadowSys::default()),
    );
    world.register_system(NODE_ATTR_N.clone(), node_attr_sys);
    world.register_system(
        RENDER_N.clone(),
        CellRenderSys::<C>::new(RenderSys::default()),
    );
    world.register_system(CLIP_N.clone(), CellClipSys::new(clip_sys));
    // world.register_system(WORLD_MATRIX_RENDER_N.clone(), CellRenderMatrixSys::new(RenderMatrixSys::new()));
    world.register_system(
        RES_RELEASE_N.clone(),
        CellResReleaseSys::<C>::new(ResReleaseSys::new()),
    );
    world.register_system(
        STYLE_MARK_N.clone(),
        CellStyleMarkSys::<L, C>::new(StyleMarkSys::new()),
    );

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("z_index_sys, show_sys, filter_sys, opacity_sys, layout_sys, text_layout_sys, world_matrix_sys, text_glphy_sys, oct_sys, transform_will_change_sys, overflow_sys, background_color_sys, box_shadow_sys, border_color_sys, image_sys, border_image_sys, charblock_sys, clip_sys, node_attr_sys, render_sys, res_release, style_mark_sys".to_string(), &world);
    world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);

    // let mut dispatch = SeqDispatcher::default();
    // dispatch.build("z_index_sys, show_sys, filter_sys, opacity_sys, layout_sys, text_layout_sys, world_matrix_sys, oct_sys, overflow_sys, clip_sys, world_matrix_render, background_color_sys, border_color_sys, box_shadow_sys, image_sys, border_image_sys, charblock_sys, charblock_shadow_sys, node_attr_sys, render_sys".to_string(), &world);
    // world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);

    let mut dispatch = SeqDispatcher::default();
    dispatch.build(
        "text_layout_sys, world_matrix_sys, text_glphy_sys, oct_sys".to_string(),
        &world,
    );
	world.add_dispatcher(LAYOUT_DISPATCH.clone(), dispatch);
	
	// 用于计算首次劈分文字
	let mut dispatch = SeqDispatcher::default();
    dispatch.build(
        "z_index_sys, show_sys, filter_sys, opacity_sys, layout_sys, text_layout_sys, world_matrix_sys, text_glphy_sys, oct_sys, transform_will_change_sys, overflow_sys, background_color_sys, box_shadow_sys, border_color_sys, image_sys, border_image_sys, charblock_sys, clip_sys, node_attr_sys, res_release, style_mark_sys".to_string(),
        &world,
    );
	world.add_dispatcher(CALC_DISPATCH.clone(), dispatch);


    world
}

pub struct GuiWorld<L: FlexNode, C: HalContext + 'static> {
    pub node: Arc<CellEntity<Node>>,
    pub transform: Arc<CellMultiCase<Node, Transform>>,
    pub z_index: Arc<CellMultiCase<Node, user::ZIndex>>,
    pub overflow: Arc<CellMultiCase<Node, Overflow>>,
    pub show: Arc<CellMultiCase<Node, Show>>,
    pub opacity: Arc<CellMultiCase<Node, user::Opacity>>,
    pub background_color: Arc<CellMultiCase<Node, BackgroundColor>>,
    pub box_shadow: Arc<CellMultiCase<Node, BoxShadow>>,
    pub border_color: Arc<CellMultiCase<Node, BorderColor>>,
    pub border_image: Arc<CellMultiCase<Node, BorderImage>>,
    pub border_image_clip: Arc<CellMultiCase<Node, BorderImageClip>>,
    pub border_image_slice: Arc<CellMultiCase<Node, BorderImageSlice>>,
    pub border_image_repeat: Arc<CellMultiCase<Node, BorderImageRepeat>>,
    pub text_style: Arc<CellMultiCase<Node, TextStyle>>,
    pub text_content: Arc<CellMultiCase<Node, TextContent>>,
    pub font: Arc<CellMultiCase<Node, Font>>,
    pub border_radius: Arc<CellMultiCase<Node, BorderRadius>>,
    pub image: Arc<CellMultiCase<Node, Image>>,
    pub image_clip: Arc<CellMultiCase<Node, ImageClip>>,
    pub object_fit: Arc<CellMultiCase<Node, ObjectFit>>,
    pub filter: Arc<CellMultiCase<Node, Filter>>,
    pub yoga: Arc<CellMultiCase<Node, L>>,
    pub class_name: Arc<CellMultiCase<Node, ClassName>>,
    pub style_mark: Arc<CellMultiCase<Node, StyleMark>>,
    pub transform_will_change: Arc<CellMultiCase<Node, TransformWillChange>>,

    //calc
    pub z_depth: Arc<CellMultiCase<Node, ZDepth>>,
    pub enable: Arc<CellMultiCase<Node, Enable>>,
    pub visibility: Arc<CellMultiCase<Node, Visibility>>,
    pub world_matrix: Arc<CellMultiCase<Node, WorldMatrix>>,
    pub by_overflow: Arc<CellMultiCase<Node, ByOverflow>>,
    pub copacity: Arc<CellMultiCase<Node, calc::Opacity>>,
    pub layout: Arc<CellMultiCase<Node, Layout>>,
    pub hsv: Arc<CellMultiCase<Node, HSV>>,
    pub culling: Arc<CellMultiCase<Node, Culling>>,

    //single
    pub idtree: Arc<CellSingleCase<IdTree>>,
    pub oct: Arc<CellSingleCase<Oct>>,
    pub overflow_clip: Arc<CellSingleCase<OverflowClip>>,
    pub engine: Arc<CellSingleCase<ShareEngine<C>>>,
    pub render_objs: Arc<CellSingleCase<RenderObjs>>,
    pub font_sheet: Arc<CellSingleCase<FontSheet>>,
    pub default_table: Arc<CellSingleCase<DefaultTable>>,
    pub class_sheet: Arc<CellSingleCase<ClassSheet>>,
    pub image_wait_sheet: Arc<CellSingleCase<ImageWaitSheet>>,
	pub dirty_list: Arc<CellSingleCase<DirtyList>>,
	pub system_time: Arc<CellSingleCase<SystemTime>>,

    pub world: World,
}

impl<L: FlexNode, C: HalContext + 'static> GuiWorld<L, C> {
    pub fn new(world: World) -> GuiWorld<L, C> {
        GuiWorld {
            node: world.fetch_entity::<Node>().unwrap(),
            transform: world.fetch_multi::<Node, Transform>().unwrap(),
            z_index: world.fetch_multi::<Node, user::ZIndex>().unwrap(),
            overflow: world.fetch_multi::<Node, Overflow>().unwrap(),
            show: world.fetch_multi::<Node, Show>().unwrap(),
            opacity: world.fetch_multi::<Node, user::Opacity>().unwrap(),
            background_color: world.fetch_multi::<Node, BackgroundColor>().unwrap(),
            box_shadow: world.fetch_multi::<Node, BoxShadow>().unwrap(),
            border_color: world.fetch_multi::<Node, BorderColor>().unwrap(),
            border_image: world.fetch_multi::<Node, BorderImage>().unwrap(),
            border_image_clip: world.fetch_multi::<Node, BorderImageClip>().unwrap(),
            border_image_slice: world.fetch_multi::<Node, BorderImageSlice>().unwrap(),
            border_image_repeat: world.fetch_multi::<Node, BorderImageRepeat>().unwrap(),
            text_content: world.fetch_multi::<Node, TextContent>().unwrap(),
            text_style: world.fetch_multi::<Node, TextStyle>().unwrap(),
            font: world.fetch_multi::<Node, Font>().unwrap(),
            border_radius: world.fetch_multi::<Node, BorderRadius>().unwrap(),
            image: world.fetch_multi::<Node, Image>().unwrap(),
            image_clip: world.fetch_multi::<Node, ImageClip>().unwrap(),
            object_fit: world.fetch_multi::<Node, ObjectFit>().unwrap(),
            filter: world.fetch_multi::<Node, Filter>().unwrap(),
            yoga: world.fetch_multi::<Node, L>().unwrap(),
            class_name: world.fetch_multi::<Node, ClassName>().unwrap(),
            style_mark: world.fetch_multi::<Node, StyleMark>().unwrap(),
            transform_will_change: world.fetch_multi::<Node, TransformWillChange>().unwrap(),
            culling: world.fetch_multi::<Node, Culling>().unwrap(),

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
            engine: world.fetch_single::<ShareEngine<C>>().unwrap(),
            render_objs: world.fetch_single::<RenderObjs>().unwrap(),
            font_sheet: world.fetch_single::<FontSheet>().unwrap(),
            default_table: world.fetch_single::<DefaultTable>().unwrap(),
            class_sheet: world.fetch_single::<ClassSheet>().unwrap(),
            image_wait_sheet: world.fetch_single::<ImageWaitSheet>().unwrap(),
			dirty_list: world.fetch_single::<DirtyList>().unwrap(),
			system_time: world.fetch_single::<SystemTime>().unwrap(),

            world: world,
        }
    }
}
