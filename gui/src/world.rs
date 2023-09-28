use std::cell::RefCell;
use std::mem::forget;
use std::sync::Arc;
use std::{default::Default, marker::PhantomData};

use pi_atom::Atom;
use hal_core::*;
use pi_null::Null;
use pi_style::style_parse::{parse_class_map_from_string, ClassMap};
use pi_style::style_type::ClassSheet;
use render::blur::{BlurSys, CellBlurSys};
use render::mask_texture::{CellMaskTextureSys, MaskTextureSys};

use crate::component::user::serialize::{ConvertToComponent, StyleTypeReader};
use crate::single::dyn_texture::DynAtlasSet;
use crate::single::IdTree;
use crate::single::fragment::{FragmentMap, Fragments, NodeTag};
use crate::system::render::opacity::{CellOpacitySys, OpacitySys};
use ecs::StdCell;
use ecs::*;
use res::ResMgr;
use share::Share;

use crate::component::calc;
use crate::component::user;
use crate::component::user::Overflow;
use crate::component::user::*;
use crate::component::{calc::LayoutR, calc::*};
use crate::entity::Node;
use crate::font::font_sheet::FontSheet;
use crate::render::engine::ShareEngine;
use crate::render::res::*;
use crate::single::DirtyViewRect;
use crate::single::*;
use crate::system::util::constant::*;
use crate::system::*;
use crate::Z_MAX;
use crate::component::calc:: Enable;

lazy_static! {
    pub static ref RENDER_DISPATCH: Atom = Atom::from("render_dispatch");
    pub static ref LAYOUT_DISPATCH: Atom = Atom::from("layout_dispatch");
    pub static ref CALC_DISPATCH: Atom = Atom::from("calc_dispatch");
    pub static ref CALC_GEO_DISPATCH: Atom = Atom::from("calc_geo_dispatch");
    pub static ref ZINDEX_N: Atom = Atom::from("z_index_sys");
    pub static ref SHOW_N: Atom = Atom::from("show_sys");
    pub static ref WORLD_MATRIX_N: Atom = Atom::from("world_matrix_sys");
    pub static ref OCT_N: Atom = Atom::from("oct_sys");
    pub static ref LYOUT_N: Atom = Atom::from("layout_sys");
    pub static ref TEXT_LAYOUT_N: Atom = Atom::from("text_layout_sys");
    pub static ref TEXT_LAYOUT_UPDATE_N: Atom = Atom::from("text_layout_update_sys");
    pub static ref MASK_TEXTURE_N: Atom = Atom::from("mask_texture_sys");
    pub static ref BLUR_N: Atom = Atom::from("blur_sys");
    pub static ref CLIP_N: Atom = Atom::from("clip_sys");
    pub static ref OPACITY_N: Atom = Atom::from("opacity_sys");
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
    pub static ref CLASS_SETTING_N: Atom = Atom::from("class_setting_sys");
    pub static ref TRANSFORM_WILL_CHANGE_N: Atom = Atom::from("transform_will_change_sys");
    pub static ref CONTENT_BOX_N: Atom = Atom::from("content_box_sys");
    pub static ref MASK_IMAGE_N: Atom = Atom::from("mask_image_sys");
    pub static ref RENDER_CONTEXT_N: Atom = Atom::from("render_context_sys");
	pub static ref CLIP_PATH_N: Atom = Atom::from("clip_path_sys");
	
}

/// 设置资源管理器
pub fn seting_res_mgr(res_mgr: &mut ResMgr) {
    res_mgr.register::<TextureRes>(10 * 1024 * 1024, 50 * 1024 * 1024, 5 * 60, 0, "TextureRes".to_string());
    res_mgr.register::<RenderBufferRes>(16 * 1024 * 1024, 32 * 1024 * 1024, 5 * 60, 0, "RenderBufferRes".to_string());
    res_mgr.register::<TexturePartRes>(10 * 1024 * 1024, 50 * 1024 * 1024, 5 * 60, 0, "TexturePartRes".to_string());
    res_mgr.register::<GeometryRes>(20 * 1024, 100 * 1024, 5 * 60, 0, "GeometryRes".to_string());
    res_mgr.register::<BufferRes>(20 * 1024, 100 * 1024, 5 * 60, 0, "BufferRes".to_string());

    res_mgr.register::<SamplerRes>(512, 1024, 60 * 60, 0, "SamplerRes".to_string());
    res_mgr.register::<RasterStateRes>(512, 1024, 60 * 60, 0, "RasterStateRes".to_string());
    res_mgr.register::<BlendStateRes>(512, 1024, 60 * 60, 0, "BlendStateRes".to_string());
    res_mgr.register::<StencilStateRes>(512, 1024, 60 * 60, 0, "StencilStateRes".to_string());
    res_mgr.register::<DepthStateRes>(512, 1024, 60 * 60, 0, "DepthStateRes".to_string());

    res_mgr.register::<UColorUbo>(4 * 1024, 8 * 1024, 60 * 60, 0, "UColorUbo".to_string());
    res_mgr.register::<HsvUbo>(1 * 1024, 2 * 1024, 60 * 60, 0, "HsvUbo".to_string());
    res_mgr.register::<MsdfStrokeUbo>(1 * 1024, 2 * 1024, 60 * 60, 0, "MsdfStrokeUbo".to_string());
    res_mgr.register::<CanvasTextStrokeColorUbo>(1 * 1024, 2 * 1024, 60 * 60, 0, "CanvasTextStrokeColorUbo".to_string());
}

pub fn create_world<C: HalContext + 'static>(
    mut engine: ShareEngine<C>,
    width: f32,
    height: f32,
    font_measure: Box<dyn Fn(&Atom, usize, char) -> f32>,
    font_texture: Share<TextureRes>,
    cur_time: usize,

    share_class_sheet: Option<Share<StdCell<ClassSheet>>>,
    share_font_sheet: Option<Share<StdCell<FontSheet>>>,
	is_sdf_font: bool,
) -> World {
    let capacity = 2000;
    let mut world = World::default();
    world.capacity = capacity;
    let project_matrix = ProjectionMatrix::new(width, height, -Z_MAX - 1.0, Z_MAX + 1.0);

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
    engine.gl.geometry_set_attribute(&geo, &AttributeName::Position, &positions, 2).unwrap();
    engine.gl.geometry_set_indices_short(&geo, &indices).unwrap();
    let unit_quad = UnitQuad(Share::new(GeometryRes {
        geo: geo,
        buffers: vec![indices, positions],
    }));

    let default_state = DefaultState(CommonState::new(&engine.gl));
    let premulti_state = PremultiState::from_common(&default_state, &engine.gl);

    let charblock_sys = CellCharBlockSys::<C>::new(CharBlockSys::with_capacity(
        &mut engine,
        (font_texture.width, font_texture.height),
        capacity,
    ));
    let border_image_sys = BorderImageSys::<C>::with_capacity(&mut engine, capacity);
    let node_attr_sys = CellNodeAttrSys::<C>::new(NodeAttrSys::new(&engine.res_mgr.borrow()));

    let clip_sys = ClipSys::<C>::new();
    let image_sys = CellImageSys::new(ImageSys::with_capacity(&mut engine, capacity));
    let render_context_sys = CellRenderContextSys::new(RenderContextSys::with_capacity(&mut engine, capacity));
    let render_sys = CellRenderSys::<C>::new(RenderSys::new(&mut engine, &project_matrix));
    let blur_sys = CellBlurSys::new(BlurSys::with_capacity(&mut engine, 0));
    let mask_texture_sys = CellMaskTextureSys::new(MaskTextureSys::with_capacity(&mut engine, 0));
    let opacity_sys = CellOpacitySys::new(OpacitySys::with_capacity(&mut engine, 0));

    let mut sys_time = SystemTime::default();
    sys_time.cur_time = cur_time;
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

    // world.register_multi::<Node, CharBlock<L>>();
    world.register_multi::<Node, TextStyle>();
    world.register_multi::<Node, TextContent>();
    world.register_multi::<Node, Font>();
    world.register_multi::<Node, BorderRadius>();
    world.register_multi::<Node, BackgroundImage>();
    world.register_multi::<Node, BackgroundImageClip>();
    world.register_multi::<Node, MaskTexture>();
    world.register_multi::<Node, MaskImage>();
    world.register_multi::<Node, MaskImageClip>();
    world.register_multi::<Node, BackgroundImageMod>();
    world.register_multi::<Node, Hsi>();
    world.register_multi::<Node, ClassName>();
    world.register_multi::<Node, StyleMark>();
    world.register_multi::<Node, TransformWillChange>();
    world.register_multi::<Node, RectLayoutStyle>();
    world.register_multi::<Node, OtherLayoutStyle>();
    world.register_multi::<Node, NodeState>();
    world.register_multi::<Node, BlendMode>();
    world.register_multi::<Node, ContentBox>();
	world.register_multi::<Node, ClipPath>();

    //calc
    world.register_multi::<Node, ImageTexture>();
    world.register_multi::<Node, BorderImageTexture>();
    world.register_multi::<Node, ZDepth>();
    world.register_multi::<Node, Enable>();
    world.register_multi::<Node, Visibility>();
    world.register_multi::<Node, WorldMatrix>();
    world.register_multi::<Node, ByOverflow>();
    world.register_multi::<Node, calc::Opacity>();
    world.register_multi::<Node, LayoutR>();
    world.register_multi::<Node, HSV>();
    world.register_multi::<Node, Culling>();
    world.register_multi::<Node, TransformWillChangeMatrix>();
    world.register_multi::<Node, RenderContext>();
    // world.register_multi::<Node, ContextIndex>();
    world.register_multi::<Node, Blur>();
    world.register_multi::<Node, RenderContextMark>();

    let mut idtree = IdTree::with_capacity(capacity);
    idtree.set_statistics_count(true);
    //single
    world.register_single::<PreRenderList>(PreRenderList(Vec::new()));
    world.register_single::<Statistics>(Statistics::default());
    world.register_single::<IdTree>(idtree);
    world.register_single::<Oct>(Oct::with_capacity(capacity));
    world.register_single::<OverflowClip>(OverflowClip::default());
    world.register_single::<RenderObjs>(RenderObjs::with_capacity(capacity));
    world.register_single::<PixelRatio>(PixelRatio(1.0));
    world.register_single::<RootIndexs>(RootIndexs::default());
    world.register_single::<Share<RefCell<DynAtlasSet>>>(Share::new(RefCell::new(DynAtlasSet::new(
        engine.res_mgr.clone(),
        width as usize,
        height as usize,
    ))));
    world.register_single::<ShareEngine<C>>(engine);
    world.register_single::<RenderContextAttrCount>(RenderContextAttrCount::default());


    match share_font_sheet {
        Some(r) => world.register_single::<Share<StdCell<FontSheet>>>(r),
        None => world.register_single::<Share<StdCell<FontSheet>>>(Share::new(StdCell::new(FontSheet::new(font_texture, font_measure, is_sdf_font)))),
    }

    world.register_single::<ViewMatrix>(ViewMatrix(WorldMatrix(
        Matrix4::new_nonuniform_scaling(&Vector3::new(1.0, 1.0, 1.0)),
        false,
    )));
    world.register_single::<ProjectionMatrix>(project_matrix.clone());
    // world.register_single::<ProjectMatrixUbo>(ProjectMatrixUbo::new(UniformValue::MatrixV4(Vec::new())));
    // world.register_single::<ViewMatrixUbo>(ViewMatrixUbo::new(UniformValue::MatrixV4(Vec::new())));
    world.register_single::<RenderBegin>(RenderBegin(RenderBeginDesc::new(0, 0, width as i32, height as i32), None));

    world.register_single::<DirtyViewRect>(DirtyViewRect(0.0, 0.0, width as f32, height as f32, true));
    world.register_single::<RenderRect>(RenderRect {
        width: width as usize,
        height: height as usize,
        view_port: Aabb2::new(Point2::new(0.0, 0.0), Point2::new(width, height)),
		flex: (1.0, 1.0),
    });


    world.register_single::<NodeRenderMap>(NodeRenderMap::with_capacity(capacity));
    match share_class_sheet {
        Some(r) => world.register_single::<Share<StdCell<ClassSheet>>>(r),
        None => world.register_single::<Share<StdCell<ClassSheet>>>(Share::new(StdCell::new(ClassSheet::default()))),
    }

    world.register_single::<UnitQuad>(unit_quad);
    world.register_single::<DefaultState>(default_state);
    world.register_single::<PremultiState>(premulti_state);
    world.register_single::<ImageWaitSheet>(ImageWaitSheet::default());
    world.register_single::<DirtyList>(DirtyList::with_capacity(capacity));
    world.register_single::<SystemTime>(sys_time);
	world.register_single::<VertType>(VertType::default());
	world.register_single::<Share<StdCell<FragmentMap>>>(Share::new(StdCell::new(FragmentMap::default())));

	// default style
	world.register_single::<Transform>(Transform::default());
	world.register_single::<user::ZIndex>(user::ZIndex::default());
	world.register_single::<Overflow>(Overflow::default());
	world.register_single::<Show>(Show::default());
	world.register_single::<user::Opacity>(user::Opacity::default());
	world.register_single::<BackgroundColor>(BackgroundColor::default());
	world.register_single::<BoxShadow>(BoxShadow::default());
	world.register_single::<BorderColor>(BorderColor::default());
	world.register_single::<BorderImage>(BorderImage::default());
	world.register_single::<BorderImageClip>(BorderImageClip::default());
	world.register_single::<BorderImageSlice>(BorderImageSlice::default());
	world.register_single::<BorderImageRepeat>(BorderImageRepeat::default());
	world.register_single::<TextStyle>(TextStyle::default());
	world.register_single::<TextContent>(TextContent::default());
	world.register_single::<Font>(Font::default());
	world.register_single::<BorderRadius>(BorderRadius::default());
	world.register_single::<BackgroundImage>(BackgroundImage::default());
	world.register_single::<BackgroundImageClip>(BackgroundImageClip::default());
	world.register_single::<BackgroundImageMod>(BackgroundImageMod::default());
	world.register_single::<Hsi>(Hsi::default());
	world.register_single::<RectLayoutStyle>(RectLayoutStyle::default());
	world.register_single::<OtherLayoutStyle>(OtherLayoutStyle::default());
	world.register_single::<NodeState>(NodeState::default());
	world.register_single::<ClassName>(ClassName::default());
	world.register_single::<StyleMark>(StyleMark::default());
	world.register_single::<TransformWillChange>(TransformWillChange::default());
	world.register_single::<MaskImage>(MaskImage::default());
	world.register_single::<MaskImageClip>(MaskImageClip::default());
	world.register_single::<BlendMode>(BlendMode::default());
	world.register_single::<Blur>(Blur::default());
	world.register_single::<ClipPath>(ClipPath::default());

    world.register_system(ZINDEX_N.clone(), CellZIndexImpl::new(ZIndexImpl::with_capacity(capacity)));
    world.register_system(SHOW_N.clone(), CellShowSys::new(ShowSys::default()));
    world.register_system(FILTER_N.clone(), CellFilterSys::new(FilterSys::default()));
    // world.register_system(OPCITY_N.clone(), CellOpacitySys::new(OpacitySys::default()));
    world.register_system(LYOUT_N.clone(), CellLayoutSys::new(LayoutSys::default()));
    world.register_system(TEXT_LAYOUT_N.clone(), CellLayoutImpl::new(LayoutImpl::new()));
    world.register_system(WORLD_MATRIX_N.clone(), CellWorldMatrixSys::new(WorldMatrixSys::with_capacity(capacity)));
    world.register_system(OCT_N.clone(), CellOctSys::new(OctSys::default()));
    world.register_system(CONTENT_BOX_N.clone(), CellContentBoxSys::new(ContentBoxSys::default()));
    world.register_system(OVERFLOW_N.clone(), CellOverflowImpl::new(OverflowImpl::default()));
    world.register_system(IMAGE_N.clone(), image_sys);
    world.register_system(RENDER_CONTEXT_N.clone(), render_context_sys);
    world.register_system(CHAR_BLOCK_N.clone(), charblock_sys);
    world.register_system(TEXT_GLPHY_N.clone(), CellTextGlphySys::<C>::new(TextGlphySys(PhantomData)));
    world.register_system(
        TRANSFORM_WILL_CHANGE_N.clone(),
        CellTransformWillChangeSys::new(TransformWillChangeSys::default()),
    );

    // world.register_system(CHAR_BLOCK_SHADOW_N.clone(), CellCharBlockShadowSys::<L>::new(CharBlockShadowSys::new()));
    world.register_system(
        BG_COLOR_N.clone(),
        CellBackgroundColorSys::<C>::new(BackgroundColorSys::with_capacity(capacity)),
    );
    world.register_system(BR_COLOR_N.clone(), CellBorderColorSys::<C>::new(BorderColorSys::with_capacity(capacity)));
    world.register_system(BR_IMAGE_N.clone(), CellBorderImageSys::new(border_image_sys));
    world.register_system(BOX_SHADOW_N.clone(), CellBoxShadowSys::<C>::new(BoxShadowSys::with_capacity(capacity)));
    world.register_system(NODE_ATTR_N.clone(), node_attr_sys);
    world.register_system(RENDER_N.clone(), render_sys);
    world.register_system(CLIP_N.clone(), CellClipSys::new(clip_sys));
    // world.register_system(WORLD_MATRIX_RENDER_N.clone(), CellRenderMatrixSys::new(RenderMatrixSys::new()));
    // world.register_system(
    //     RES_RELEASE_N.clone(),
    //     CellResReleaseSys::<C>::new(ResReleaseSys::new()),
    // );
    world.register_system(STYLE_MARK_N.clone(), CellStyleMarkSys::<C>::new(StyleMarkSys::new()));
	
	let sys = CellClassSetting::<C>::new(ClassSetting::new(&mut world));
    world.register_system(CLASS_SETTING_N.clone(), sys);

    world.register_system(MASK_IMAGE_N.clone(), CellMaskImageSys::<C>::new(MaskImageSys::new()));

    world.register_system(TEXT_LAYOUT_UPDATE_N.clone(), CellTextLayoutUpdateSys::new(TextLayoutUpdateSys::default()));

    world.register_system(BLUR_N.clone(), blur_sys);

    world.register_system(MASK_TEXTURE_N.clone(), mask_texture_sys);

    world.register_system(OPACITY_N.clone(), opacity_sys);
	world.register_system(CLIP_PATH_N.clone(), CellClipPathSys::<C>::new(ClipPathSys::default()));

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("class_setting_sys, z_index_sys, show_sys, filter_sys, text_layout_sys, layout_sys, text_layout_update_sys, world_matrix_sys, text_glphy_sys, transform_will_change_sys, oct_sys, content_box_sys, overflow_sys, background_color_sys, box_shadow_sys, border_color_sys, image_sys, border_image_sys, charblock_sys, mask_image_sys, render_context_sys, blur_sys, opacity_sys, mask_texture_sys, clip_path_sys,  clip_sys, node_attr_sys, render_sys, style_mark_sys".to_string(), &world);
    world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);

    // let mut dispatch = SeqDispatcher::default();
    // dispatch.build("z_index_sys, show_sys, filter_sys, opacity_sys, layout_sys, text_layout_sys, world_matrix_sys, oct_sys, overflow_sys, clip_sys, world_matrix_render, background_color_sys, border_color_sys, box_shadow_sys, image_sys, border_image_sys, charblock_sys, charblock_shadow_sys, node_attr_sys, render_sys".to_string(), &world);
    // world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);

    let mut dispatch = SeqDispatcher::default();
    dispatch.build(
        "class_setting_sys, text_layout_sys, layout_sys, text_layout_update_sys, world_matrix_sys, oct_sys".to_string(),
        &world,
    );
    world.add_dispatcher(CALC_GEO_DISPATCH.clone(), dispatch);

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("class_setting_sys, text_layout_sys, layout_sys".to_string(), &world);
    world.add_dispatcher(LAYOUT_DISPATCH.clone(), dispatch);

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("class_setting_sys, z_index_sys, show_sys, filter_sys, text_layout_sys, layout_sys, text_layout_update_sys, world_matrix_sys, text_glphy_sys, transform_will_change_sys, oct_sys, content_box_sys, overflow_sys, background_color_sys, box_shadow_sys, border_color_sys, image_sys, border_image_sys, charblock_sys, mask_image_sys, render_context_sys, blur_sys, opacity_sys, mask_texture_sys, clip_path_sys, clip_sys, mask_image_sys, node_attr_sys, style_mark_sys".to_string(), &world);
    world.add_dispatcher(CALC_DISPATCH.clone(), dispatch);
    world
}

pub struct GuiWorldExt {
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
    pub background_image: Arc<CellMultiCase<Node, BackgroundImage>>,
    pub background_image_clip: Arc<CellMultiCase<Node, BackgroundImageClip>>,
    pub background_image_mod: Arc<CellMultiCase<Node, BackgroundImageMod>>,
    pub filter: Arc<CellMultiCase<Node, Hsi>>,
    pub rect_layout_style: Arc<CellMultiCase<Node, RectLayoutStyle>>,
    pub other_layout_style: Arc<CellMultiCase<Node, OtherLayoutStyle>>,
    pub node_state: Arc<CellMultiCase<Node, NodeState>>,
    pub class_name: Arc<CellMultiCase<Node, ClassName>>,
    pub transform_will_change: Arc<CellMultiCase<Node, TransformWillChange>>,
    pub mask_image: Arc<CellMultiCase<Node, MaskImage>>,
    pub mask_image_clip: Arc<CellMultiCase<Node, MaskImageClip>>,
    pub blend_mode: Arc<CellMultiCase<Node, BlendMode>>,
    pub blur: Arc<CellMultiCase<Node, Blur>>,
	pub clip_path: Arc<CellMultiCase<Node, ClipPath>>,

	pub style_mark: Arc<CellMultiCase<Node, StyleMark>>,

    //calc
    pub z_depth: Arc<CellMultiCase<Node, ZDepth>>,
    pub enable: Arc<CellMultiCase<Node, crate::component::calc::Enable>>,
    pub visibility: Arc<CellMultiCase<Node, Visibility>>,
    pub world_matrix: Arc<CellMultiCase<Node, WorldMatrix>>,
    pub by_overflow: Arc<CellMultiCase<Node, ByOverflow>>,
    pub copacity: Arc<CellMultiCase<Node, calc::Opacity>>,
    pub layout: Arc<CellMultiCase<Node, LayoutR>>,
    pub hsv: Arc<CellMultiCase<Node, HSV>>,
    pub culling: Arc<CellMultiCase<Node, Culling>>,
    pub image_texture: Arc<CellMultiCase<Node, ImageTexture>>,

    //single
    pub idtree: Arc<CellSingleCase<IdTree>>,
    pub oct: Arc<CellSingleCase<Oct>>,
    pub overflow_clip: Arc<CellSingleCase<OverflowClip>>,
    pub render_objs: Arc<CellSingleCase<RenderObjs>>,
    pub font_sheet: Arc<CellSingleCase<Share<StdCell<FontSheet>>>>,
    pub class_sheet: Arc<CellSingleCase<Share<StdCell<ClassSheet>>>>,
    pub image_wait_sheet: Arc<CellSingleCase<ImageWaitSheet>>,
    pub dirty_list: Arc<CellSingleCase<DirtyList>>,
    pub system_time: Arc<CellSingleCase<SystemTime>>,
    pub dirty_view_rect: Arc<CellSingleCase<DirtyViewRect>>,
    pub dyn_atlas_set: Arc<CellSingleCase<Share<RefCell<DynAtlasSet>>>>,
	pub fragment:  Arc<CellSingleCase<Share<StdCell<FragmentMap>>>>,


	// DefaultComponent默认组件
	pub default_components: DefaultComponent,

    // pub world: World,
}

impl GuiWorldExt {
	pub fn new(world: &mut World) -> Self {
		Self {
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
			background_image: world.fetch_multi::<Node, BackgroundImage>().unwrap(),
			background_image_clip: world.fetch_multi::<Node, BackgroundImageClip>().unwrap(),
			background_image_mod: world.fetch_multi::<Node, BackgroundImageMod>().unwrap(),
			filter: world.fetch_multi::<Node, Hsi>().unwrap(),
			rect_layout_style: world.fetch_multi::<Node, RectLayoutStyle>().unwrap(),
			other_layout_style: world.fetch_multi::<Node, OtherLayoutStyle>().unwrap(),
			node_state: world.fetch_multi::<Node, NodeState>().unwrap(),
			class_name: world.fetch_multi::<Node, ClassName>().unwrap(),
			style_mark: world.fetch_multi::<Node, StyleMark>().unwrap(),
			transform_will_change: world.fetch_multi::<Node, TransformWillChange>().unwrap(),
			culling: world.fetch_multi::<Node, Culling>().unwrap(),
			mask_image: world.fetch_multi::<Node, MaskImage>().unwrap(),
			mask_image_clip: world.fetch_multi::<Node, MaskImageClip>().unwrap(),
			blend_mode: world.fetch_multi::<Node, BlendMode>().unwrap(),
			blur: world.fetch_multi::<Node, Blur>().unwrap(),
			clip_path: world.fetch_multi::<Node, ClipPath>().unwrap(),

			//calc
			z_depth: world.fetch_multi::<Node, ZDepth>().unwrap(),
			enable: world.fetch_multi::<Node, Enable>().unwrap(),
			visibility: world.fetch_multi::<Node, Visibility>().unwrap(),
			world_matrix: world.fetch_multi::<Node, WorldMatrix>().unwrap(),
			by_overflow: world.fetch_multi::<Node, ByOverflow>().unwrap(),
			copacity: world.fetch_multi::<Node, calc::Opacity>().unwrap(),
			layout: world.fetch_multi::<Node, LayoutR>().unwrap(),
			hsv: world.fetch_multi::<Node, HSV>().unwrap(),
			image_texture: world.fetch_multi::<Node, ImageTexture>().unwrap(),

			//single
			idtree: world.fetch_single::<IdTree>().unwrap(),
			oct: world.fetch_single::<Oct>().unwrap(),
			overflow_clip: world.fetch_single::<OverflowClip>().unwrap(),
			render_objs: world.fetch_single::<RenderObjs>().unwrap(),
			font_sheet: world.fetch_single::<Share<StdCell<FontSheet>>>().unwrap(),
			class_sheet: world.fetch_single::<Share<StdCell<ClassSheet>>>().unwrap(),
			image_wait_sheet: world.fetch_single::<ImageWaitSheet>().unwrap(),
			dirty_list: world.fetch_single::<DirtyList>().unwrap(),
			system_time: world.fetch_single::<SystemTime>().unwrap(),
			dirty_view_rect: world.fetch_single::<DirtyViewRect>().unwrap(),
			dyn_atlas_set: world.fetch_single::<Share<RefCell<DynAtlasSet>>>().unwrap(),
			fragment: world.fetch_single::<Share<StdCell<FragmentMap>>>().unwrap(),

			default_components: DefaultComponent { 
				transform: world.fetch_single::<Transform>().unwrap(),
				z_index: world.fetch_single::<user::ZIndex>().unwrap(),
				overflow: world.fetch_single::<Overflow>().unwrap(),
				show: world.fetch_single::<Show>().unwrap(),
				opacity: world.fetch_single::<user::Opacity>().unwrap(),
				background_color: world.fetch_single::<BackgroundColor>().unwrap(),
				box_shadow: world.fetch_single::<BoxShadow>().unwrap(),
				border_color: world.fetch_single::<BorderColor>().unwrap(),
				border_image: world.fetch_single::<BorderImage>().unwrap(),
				border_image_clip: world.fetch_single::<BorderImageClip>().unwrap(),
				border_image_slice: world.fetch_single::<BorderImageSlice>().unwrap(),
				border_image_repeat: world.fetch_single::<BorderImageRepeat>().unwrap(),
				text_style: world.fetch_single::<TextStyle>().unwrap(),
				text_content: world.fetch_single::<TextContent>().unwrap(),
				font: world.fetch_single::<Font>().unwrap(),
				border_radius: world.fetch_single::<BorderRadius>().unwrap(),
				background_image: world.fetch_single::<BackgroundImage>().unwrap(),
				background_image_clip: world.fetch_single::<BackgroundImageClip>().unwrap(),
				background_image_mod: world.fetch_single::<BackgroundImageMod>().unwrap(),
				filter: world.fetch_single::<Hsi>().unwrap(),
				rect_layout_style: world.fetch_single::<RectLayoutStyle>().unwrap(),
				other_layout_style: world.fetch_single::<OtherLayoutStyle>().unwrap(),
				node_state: world.fetch_single::<NodeState>().unwrap(),
				class_name: world.fetch_single::<ClassName>().unwrap(),
				style_mark: world.fetch_single::<StyleMark>().unwrap(),
				transform_will_change: world.fetch_single::<TransformWillChange>().unwrap(),
				mask_image: world.fetch_single::<MaskImage>().unwrap(),
				mask_image_clip: world.fetch_single::<MaskImageClip>().unwrap(),
				blend_mode: world.fetch_single::<BlendMode>().unwrap(),
				blur: world.fetch_single::<Blur>().unwrap(),
				clip_path: world.fetch_single::<ClipPath>().unwrap(),
			},
		}
	}
}

impl<C: HalContext + 'static> GuiWorld<C> {
	// 设置默认样式
	pub fn set_default_style(&mut self, class: &str) {
		let class_sheet = self.world_ext.class_sheet.lend_mut();
		let mut class_sheet = class_sheet.borrow_mut();

		let mut c = class;
		let class_temp;
		if !class.starts_with(".c0") {
			class_temp = ".c0{".to_string() + class + "}";
			c = class_temp.as_str();
		}
        match parse_class_map_from_string(c, 0) {
            Ok(r) => {
                r.to_class_sheet(&mut class_sheet);

				if let Some(class) = class_sheet.class_map.get(&0) {
					let mut style_reader = StyleTypeReader::new(&class_sheet.style_buffer, class.start, class.end);
					while style_reader.write_to_default(&self.world_ext).is_some() {}
				}
            } // 触发DefaultStyle修改
            Err(e) => {
                log::error!("set_default_style_by_str fail, parse style err: {:?}", e);
                return;
            }
        };
	}

	/// 设置样式
	pub fn set_style<T: ConvertToComponent>(&mut self, entity: usize, value: T) {
		let style_mark = self.world_ext.style_mark.lend_mut();
		if let Some(style_mark) = style_mark.get_mut(entity){
			<T as ConvertToComponent>::set(&mut style_mark.local_style, &value as *const T as usize as *const u8, &self.world_ext, entity, false);
			let dirty_list = self.world_ext.dirty_list.lend_mut();
			// 设脏
			set_dirty(dirty_list, entity, T::get_type() as usize, style_mark);
		}
		forget(value);
	}

	// 创建class
    pub fn create_class_by_bin(&mut self, bin: &[u8]) {
        match postcard::from_bytes::<Vec<pi_style::style_parse::ClassMap>>(bin) {
            Ok(r) => {
				for item in r.into_iter() {
					self.create_class(item);
				}
            }
            Err(_e) => {
                return;
            }
        }
    }

	pub fn create_class(&mut self, class: ClassMap) {
       // 处理css
		let class_sheet_single = self.world_ext.class_sheet.lend_mut();
		let mut class_sheet_single = class_sheet_single.borrow_mut();
		let mut class_sheet = pi_style::style_type::ClassSheet::default();
		class.to_class_sheet(&mut class_sheet);
		class_sheet_single.extend_from_class_sheet(class_sheet);
    }


	// 添加模版
	pub fn add_fragment_by_bin(&mut self, value: Fragments) {
		let fragment = self.world_ext.fragment.lend_mut();
		fragment.borrow_mut().extend(value);
	}

	// 从模版创建节点, 返回创建的所有节点id
	pub fn create_from_fragment(&mut self, key: u32) -> Vec<usize> {
		let fragments = self.world_ext.fragment.lend();
		let fragments = fragments.borrow();
		let idtree = self.world_ext.idtree.lend_mut();
		let text_content = self.world_ext.text_content.lend_mut();
		let node_states = self.world_ext.node_state.lend_mut();

		let t = match fragments.map.get(&key) {
            Some(r) => r,
            _ => {
                return Vec::default();
            }
        };
		let mut entitys = Vec::with_capacity(t.end - t.start);
		for _ in t.start..t.end {
			let e = self.world.create_entity::<Node>();
			entitys.push(e);
			idtree.create(e);
		}


        for i in t.clone() {
            let n = &fragments.fragments[i];
            let node = entitys[i - t.start];

			// 初始化节点
			if n.tag == NodeTag::Span {
				text_content.insert(node, TextContent(pi_style::style::TextContent( "".to_string(), Atom::from(""))));
			} else if n.tag == NodeTag::VNode {
				node_states[node].0.set_vnode(true);
			}
			
			// 设置本地样式
            if n.style_meta.end > n.style_meta.start {
				let style_mark = self.world_ext.style_mark.lend_mut();
				let style_mark = &mut style_mark[node];

				let mut style_reader = StyleTypeReader::new(&fragments.style_buffer, n.style_meta.start, n.style_meta.end);
				while style_reader.write_to_component(&mut style_mark.local_style, node, &self.world_ext, true) {}

				// 设脏
				if style_mark.local_style.any() {
					self.world_ext.dirty_list.lend_mut().0.push(node);
				}
			}
			
			// 设置class
			if n.class.len() > 0 {
				self.world_ext.class_name.lend_mut().insert(node, n.class.clone());
			}
        }

		let idtree = self.world_ext.idtree.lend_mut();
		// 组织节点的父子关系
		for i in t.clone() {
            let n = &fragments.fragments[i];
            let node = entitys[i - t.start];
            log::debug!(
                "fragment_commands insertChild!!====================node={:?}, parent_index={:?}, parent_id={:?}",
                node,
                n.parent,
				entitys.get(n.parent)
            );
            if !n.parent.is_null(){
                // log::warn!("fragment_commands insertChild====================node：{:?}, parent {:?}", node, c.entitys[n.parent]);
                idtree.insert_child(node, entitys[n.parent], std::usize::MAX);
            }
        }

		entitys
	}
}


#[inline]
pub fn set_dirty(dirty_list: &mut DirtyList, id: usize, ty: usize, style_mark: &mut StyleMark) {
    if style_mark.dirty.not_any(){
        dirty_list.0.push(id);
    }

    style_mark.dirty.set(ty, true);
}

pub struct DefaultComponent {
	pub transform: Arc<CellSingleCase<Transform>>,
    pub z_index: Arc<CellSingleCase<user::ZIndex>>,
    pub overflow: Arc<CellSingleCase<Overflow>>,
    pub show: Arc<CellSingleCase<Show>>,
    pub opacity: Arc<CellSingleCase<user::Opacity>>,
    pub background_color: Arc<CellSingleCase<BackgroundColor>>,
    pub box_shadow: Arc<CellSingleCase<BoxShadow>>,
    pub border_color: Arc<CellSingleCase<BorderColor>>,
    pub border_image: Arc<CellSingleCase<BorderImage>>,
    pub border_image_clip: Arc<CellSingleCase<BorderImageClip>>,
    pub border_image_slice: Arc<CellSingleCase<BorderImageSlice>>,
    pub border_image_repeat: Arc<CellSingleCase<BorderImageRepeat>>,
    pub text_style: Arc<CellSingleCase<TextStyle>>,
    pub text_content: Arc<CellSingleCase<TextContent>>,
    pub font: Arc<CellSingleCase<Font>>,
    pub border_radius: Arc<CellSingleCase<BorderRadius>>,
    pub background_image: Arc<CellSingleCase<BackgroundImage>>,
    pub background_image_clip: Arc<CellSingleCase<BackgroundImageClip>>,
    pub background_image_mod: Arc<CellSingleCase<BackgroundImageMod>>,
    pub filter: Arc<CellSingleCase<Hsi>>,
    pub rect_layout_style: Arc<CellSingleCase<RectLayoutStyle>>,
    pub other_layout_style: Arc<CellSingleCase<OtherLayoutStyle>>,
    pub node_state: Arc<CellSingleCase<NodeState>>,
    pub class_name: Arc<CellSingleCase<ClassName>>,
    pub style_mark: Arc<CellSingleCase<StyleMark>>,
    pub transform_will_change: Arc<CellSingleCase<TransformWillChange>>,
    pub mask_image: Arc<CellSingleCase<MaskImage>>,
    pub mask_image_clip: Arc<CellSingleCase<MaskImageClip>>,
    pub blend_mode: Arc<CellSingleCase<BlendMode>>,
    pub blur: Arc<CellSingleCase<Blur>>,
	pub clip_path: Arc<CellSingleCase<ClipPath>>,
}

pub struct GuiWorld<C: HalContext + 'static> {

	pub world_ext: GuiWorldExt,

	pub engine: Arc<CellSingleCase<ShareEngine<C>>>,
    pub renderSys: Arc<CellRenderSys<C>>,

    pub world: World,
}

impl<C: HalContext + 'static> GuiWorld<C> {
    pub fn new(mut world: World) -> GuiWorld<C> {
        GuiWorld {
			world_ext: GuiWorldExt::new(&mut world),
			engine: world.fetch_single::<ShareEngine<C>>().unwrap(),
			renderSys: world.fetch_sys::<CellRenderSys<C>>(&RENDER_N).unwrap(),
            world: world,
        }
    }
}

// #[test]
fn test_insert11() -> std::time::Duration {
    let mut world = World::default();
    world.register_entity::<Node>();
    world.register_multi::<Node, BorderRadius>();
    world.register_multi::<Node, user::ZIndex>();
    world.register_multi::<Node, Visibility>();
    world.register_multi::<Node, RectLayoutStyle>();
    world.register_multi::<Node, OtherLayoutStyle>();
    world.register_multi::<Node, StyleMark>();
    world.register_multi::<Node, ZDepth>();
    world.register_multi::<Node, calc::Opacity>();
    world.register_multi::<Node, HSV>();
    world.register_multi::<Node, LayoutR>();
    world.register_multi::<Node, WorldMatrix>();
    world.register_multi::<Node, Enable>();
    world.register_multi::<Node, NodeState>();
    world.register_multi::<Node, ByOverflow>();
    world.register_multi::<Node, Culling>();
    world.register_multi::<Node, BackgroundColor>();

    let nodes = world.fetch_entity::<Node>().unwrap();

    let opacity = world.fetch_multi::<Node, calc::Opacity>().unwrap();
    let border_radius = world.fetch_multi::<Node, BorderRadius>().unwrap();
    let rect_layout_style = world.fetch_multi::<Node, RectLayoutStyle>().unwrap();
    let other_layout_style = world.fetch_multi::<Node, OtherLayoutStyle>().unwrap();
    let node_state = world.fetch_multi::<Node, NodeState>().unwrap();
    let style_mark = world.fetch_multi::<Node, StyleMark>().unwrap();
    let culling = world.fetch_multi::<Node, Culling>().unwrap();
    let z_depth = world.fetch_multi::<Node, ZDepth>().unwrap();
    let enable = world.fetch_multi::<Node, Enable>().unwrap();
    let visibility = world.fetch_multi::<Node, Visibility>().unwrap();
    let world_matrix = world.fetch_multi::<Node, WorldMatrix>().unwrap();
    let by_overflow = world.fetch_multi::<Node, ByOverflow>().unwrap();
    let layout = world.fetch_multi::<Node, LayoutR>().unwrap();
    let hsv = world.fetch_multi::<Node, HSV>().unwrap();
    let bg_color = world.fetch_multi::<Node, BackgroundColor>().unwrap();

    let t = std::time::Instant::now();
    for _i in 0..200 {
        let entity = nodes.lend_mut().create();
        opacity.lend_mut().insert(entity, calc::Opacity::default());
        border_radius.lend_mut().insert(entity, BorderRadius::default());
        rect_layout_style.lend_mut().insert(entity, RectLayoutStyle::default());
        other_layout_style.lend_mut().insert(entity, OtherLayoutStyle::default());
        node_state.lend_mut().insert(entity, NodeState::default());
        style_mark.lend_mut().insert(entity, StyleMark::default());
        culling.lend_mut().insert(entity, Culling::default());
        z_depth.lend_mut().insert(entity, ZDepth::default());
        enable.lend_mut().insert(entity, Enable::default());
        visibility.lend_mut().insert(entity, Visibility::default());
        world_matrix.lend_mut().insert(entity, WorldMatrix::default());
        by_overflow.lend_mut().insert(entity, ByOverflow::default());
        layout.lend_mut().insert(entity, LayoutR::default());
        hsv.lend_mut().insert(entity, HSV::default());
    }
    for i in 1..71 {
        bg_color.lend_mut().insert(i, BackgroundColor::default());
    }

    for i in 1..201 {
        rect_layout_style.lend_mut().get_mut(i).unwrap().size.width = flex_layout::Dimension::Points(32.0);
        rect_layout_style.lend_mut().get_mut(i).unwrap().size.height = flex_layout::Dimension::Points(32.0);
        other_layout_style.lend_mut().get_mut(i).unwrap().align_content = flex_layout::AlignContent::Center;
        other_layout_style.lend_mut().get_mut(i).unwrap().align_items = flex_layout::AlignItems::Center;
        other_layout_style.lend_mut().get_mut(i).unwrap().align_self = flex_layout::AlignSelf::Center;
    }
    std::time::Instant::now() - t
}

#[test]
fn test_insert() {
    let mut all_time = std::time::Duration::from_millis(0);
    for i in 0..100 {
        all_time = all_time + test_insert11();
    }
    println!("time: {:?}", all_time / 100);
}
