/** 
 * 抽象硬件层（HAL）的空实现
 * 
 */



use std::default::Default;
use std::sync::Arc;

use hal_core::{Context, RenderBeginDesc};
use atom::Atom;
use cgmath::One;


use ecs::*;
use ecs::idtree::IdTree;
use gui::component::user::*;
use gui::component::calc::*;
use gui::component::calc;
use gui::component::user;
use gui::single::*;
use gui::entity::Node;
use gui::render::engine::Engine;
use gui::system::*;
use gui::font::font_sheet::FontSheet;
use gui::Z_MAX;
use layout::YgNode;

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

pub fn create_world<C: Context + Sync + Send + 'static>(mut engine: Engine<C>, width: f32, height: f32) -> World{
    let mut world = World::default();

    let mut default_table = DefaultTable::new();
    default_table.set::<TextStyle>(TextStyle::default());
    default_table.set::<Transform>(Transform::default());
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
    world.register_multi::<Node, CharBlock<YgNode>>();
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
    world.register_multi::<Node, WorldMatrixRender>();
    
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
    world.register_single::<RenderBegin>(RenderBegin(Arc::new(RenderBeginDesc::new(0, 0, width as i32, height as i32))));
    world.register_single::<NodeRenderMap>(NodeRenderMap::new());
    world.register_single::<DefaultTable>(default_table);
    
    world.register_system(ZINDEX_N.clone(), CellZIndexImpl::new(ZIndexImpl::new()));
    world.register_system(SHOW_N.clone(), CellShowSys::new(ShowSys::default()));
    world.register_system(FILTER_N.clone(), CellFilterSys::new(FilterSys::default()));
    world.register_system(OPCITY_N.clone(), CellOpacitySys::new(OpacitySys::default()));
    world.register_system(LYOUT_N.clone(), CellLayoutSys::<YgNode>::new(LayoutSys::new())); // 10000 1.2ms
    world.register_system(TEXT_LAYOUT_N.clone(), CellLayoutImpl::new(LayoutImpl::<C, YgNode>::new())); // 0ms
    world.register_system(WORLD_MATRIX_N.clone(), CellWorldMatrixSys::new(WorldMatrixSys::default())); // 1.1ms
    world.register_system(OCT_N.clone(), CellOctSys::new(OctSys::default())); // 3.5ms
    world.register_system(OVERFLOW_N.clone(), CellOverflowImpl::new(OverflowImpl)); // 0.7ms
    world.register_system(CLIP_N.clone(), CellClipSys::new(clip_sys)); // 0.2ms
    world.register_system(IMAGE_N.clone(), CellImageSys::new(ImageSys::<C>::new()));
    world.register_system(CHAR_BLOCK_N.clone(), CellCharBlockSys::<C, YgNode>::new(CharBlockSys::new()));
    world.register_system(CHAR_BLOCK_SHADOW_N.clone(), CellCharBlockShadowSys::<C, YgNode>::new(CharBlockShadowSys::new()));
    world.register_system(BG_COLOR_N.clone(), CellBackgroundColorSys::new(BackgroundColorSys::<C>::new()));
    world.register_system(BR_COLOR_N.clone(), CellBorderColorSys::new(BorderColorSys::<C>::new()));
    world.register_system(BR_IMAGE_N.clone(), CellBorderImageSys::new(BorderImageSys::<C>::new()));
    world.register_system(BOX_SHADOW_N.clone(), CellBoxShadowSys::new(BoxShadowSys::<C>::new()));
    world.register_system(NODE_ATTR_N.clone(), CellNodeAttrSys::new(NodeAttrSys::<C>::new())); // 0.6ms
    world.register_system(RENDER_N.clone(), CellRenderSys::new(RenderSys::<C>::default()));
    world.register_system(WORLD_MATRIX_RENDER_N.clone(), CellRenderMatrixSys::new(RenderMatrixSys::<C>::new())); // 0.4ms
    
    // let mut dispatch = SeqDispatcher::default();
    // dispatch.build("world_matrix_sys".to_string(), &world);
    // world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);

    // let mut dispatch = SeqDispatcher::default();
    // dispatch.build("layout_sys, text_layout_sys, world_matrix_sys, oct_sys".to_string(), &world);
    // world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);

    let mut dispatch = SeqDispatcher::default();
    dispatch.build("z_index_sys, show_sys, filter_sys, opacity_sys, layout_sys, text_layout_sys, world_matrix_sys, oct_sys, overflow_sys, clip_sys, world_matrix_render, background_color_sys, border_color_sys, box_shadow_sys, image_sys, border_image_sys, charblock_sys, charblock_shadow_sys, node_attr_sys, render_sys".to_string(), &world);
    world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);

    // let mut dispatch = SeqDispatcher::default();
    // dispatch.build("layout_sys, world_matrix_sys, oct_sys".to_string(), &world);
    // world.add_dispatcher(LAYOUT_DISPATCH.clone(), dispatch);

    // let mut dispatch = SeqDispatcher::default();
    // dispatch.build("z_index_sys, layout_sys, world_matrix_sys ".to_string(), &world);
    // world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);
    world
}

pub struct GuiWorld {
    pub node: Arc<CellEntity<Node>>,
    pub transform: Arc<CellMultiCase<Node, Transform>>,
    pub z_index: Arc<CellMultiCase<Node, gui::component::user::ZIndex>>,
    pub overflow: Arc<CellMultiCase<Node, Overflow>>,
    pub show: Arc<CellMultiCase<Node, Show>>,
    pub opacity: Arc<CellMultiCase<Node, gui::component::user::Opacity>>,
    pub background_color: Arc<CellMultiCase<Node, BackgroundColor>>,
    pub box_show: Arc<CellMultiCase<Node, BoxShadow>>,
    pub border_color: Arc<CellMultiCase<Node, BorderColor>>,
    pub border_image: Arc<CellMultiCase<Node, BorderImage<NullContextImpl>>>,
    pub border_image_clip: Arc<CellMultiCase<Node, BorderImageClip>>,
    pub border_image_slice: Arc<CellMultiCase<Node, BorderImageSlice>>,
    pub border_image_repeat: Arc<CellMultiCase<Node, BorderImageRepeat>>,
    pub text: Arc<CellMultiCase<Node, Text>>,
    pub text_style: Arc<CellMultiCase<Node, TextStyle>>,
    pub text_shadow: Arc<CellMultiCase<Node, TextShadow>>,
    pub font: Arc<CellMultiCase<Node, Font>>,
    pub border_radius: Arc<CellMultiCase<Node, BorderRadius>>,
    pub image: Arc<CellMultiCase<Node, Image<NullContextImpl>>>,
    pub image_clip: Arc<CellMultiCase<Node, ImageClip>>,
    pub object_fit: Arc<CellMultiCase<Node, ObjectFit>>,
    pub filter: Arc<CellMultiCase<Node, Filter>>,

    //calc
    pub z_depth: Arc<CellMultiCase<Node, ZDepth>>,
    pub enable: Arc<CellMultiCase<Node, Enable>>,
    pub visibility: Arc<CellMultiCase<Node, Visibility>>,
    pub world_matrix: Arc<CellMultiCase<Node, WorldMatrix>>,
    pub by_overflow: Arc<CellMultiCase<Node, ByOverflow>>,
    pub copacity: Arc<CellMultiCase<Node, gui::component::calc::Opacity>>,
    pub layout: Arc<CellMultiCase<Node, Layout>>,
    pub hsv: Arc<CellMultiCase<Node, HSV>>,
    
    //single
    pub idtree: Arc<CellSingleCase<IdTree>>,
    pub oct: Arc<CellSingleCase<Oct>>,
    pub overflow_clip: Arc<CellSingleCase<OverflowClip>>,
    pub engine: Arc<CellSingleCase<Engine<NullContextImpl>>>,
    pub render_objs: Arc<CellSingleCase<RenderObjs<NullContextImpl>>>,
    pub font_sheet: Arc<CellSingleCase<FontSheet<NullContextImpl>>>,
    pub default_table: Arc<CellSingleCase<DefaultTable>>,

    pub world: World,
}

impl GuiWorld {
    pub fn new(world: World) -> GuiWorld{
        GuiWorld{
            node: world.fetch_entity::<Node>().unwrap(),
            transform: world.fetch_multi::<Node, Transform>().unwrap(),
            z_index: world.fetch_multi::<Node, gui::component::user::ZIndex>().unwrap(),
            overflow: world.fetch_multi::<Node, Overflow>().unwrap(),
            show: world.fetch_multi::<Node, Show>().unwrap(),
            opacity: world.fetch_multi::<Node, gui::component::user::Opacity>().unwrap(),
            background_color: world.fetch_multi::<Node, BackgroundColor>().unwrap(),
            box_show: world.fetch_multi::<Node, BoxShadow>().unwrap(),
            border_color: world.fetch_multi::<Node, BorderColor>().unwrap(),
            border_image: world.fetch_multi::<Node, BorderImage<NullContextImpl>>().unwrap(),
            border_image_clip: world.fetch_multi::<Node, BorderImageClip>().unwrap(),
            border_image_slice: world.fetch_multi::<Node, BorderImageSlice>().unwrap(),
            border_image_repeat: world.fetch_multi::<Node, BorderImageRepeat>().unwrap(),
            text: world.fetch_multi::<Node, Text>().unwrap(),
            text_style: world.fetch_multi::<Node, TextStyle>().unwrap(),
            text_shadow: world.fetch_multi::<Node, TextShadow>().unwrap(),
            font: world.fetch_multi::<Node, Font>().unwrap(),
            border_radius: world.fetch_multi::<Node, BorderRadius>().unwrap(),
            image: world.fetch_multi::<Node, Image<NullContextImpl>>().unwrap(),
            image_clip: world.fetch_multi::<Node, ImageClip>().unwrap(),
            object_fit: world.fetch_multi::<Node, ObjectFit>().unwrap(),
            filter: world.fetch_multi::<Node, Filter>().unwrap(),

            //calc
            z_depth: world.fetch_multi::<Node, ZDepth>().unwrap(),
            enable: world.fetch_multi::<Node, Enable>().unwrap(),
            visibility: world.fetch_multi::<Node, Visibility>().unwrap(),
            world_matrix: world.fetch_multi::<Node, WorldMatrix>().unwrap(),
            by_overflow: world.fetch_multi::<Node, ByOverflow>().unwrap(),
            copacity: world.fetch_multi::<Node, gui::component::calc::Opacity>().unwrap(),
            layout: world.fetch_multi::<Node, Layout>().unwrap(),
            hsv: world.fetch_multi::<Node, HSV>().unwrap(),
            
            //single
            idtree: world.fetch_single::<IdTree>().unwrap(),
            oct: world.fetch_single::<Oct>().unwrap(),
            overflow_clip: world.fetch_single::<OverflowClip>().unwrap(),
            engine: world.fetch_single::<Engine<NullContextImpl>>().unwrap(),
            render_objs: world.fetch_single::<RenderObjs<NullContextImpl>>().unwrap(),
            font_sheet: world.fetch_single::<FontSheet<NullContextImpl>>().unwrap(),
            default_table: world.fetch_single::<DefaultTable>().unwrap(),

            world: world,
        }
    }
}

// rust 世界矩阵run， 10000节点， 980微秒
// rust cal_oct， 10000节点， 980微秒

use hal_null::*;
use ecs::*;
use std::usize::MAX as UMAX;
use gui::layout::YGAlign;

#[allow(unused_attributes)]
#[no_mangle]
pub fn test_time() {
    let engine = Engine::new(NullContextImpl::new());
    let world = create_world(engine, 1000.0, 100.0);

    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.lend_mut();
    let node = world.create_entity::<Node>();
    let border_radius = world.fetch_multi::<Node, BorderRadius>().unwrap();
    let border_radius = border_radius.lend_mut();
    border_radius.insert(node, BorderRadius{x: LengthUnit::Pixel(0.0), y: LengthUnit::Pixel(0.0)});

    let visibilitys = world.fetch_multi::<Node, Visibility>().unwrap();
    let visibilitys = visibilitys.lend_mut();
    visibilitys.insert(node, Visibility(true));

    // let ygnode = world.fetch_multi::<Node, YgNode>().unwrap();
    // let ygnode = ygnode.lend_mut();
    // let ygnode = unsafe { ygnode.get_unchecked_mut(node) };
    // ygnode.set_width(width);
    // ygnode.set_height(height);
    // ygnode.set_align_items(YGAlign::YGAlignFlexStart);

    idtree.create(node);
    idtree.insert_child(node, 0, 0, None);


    let node2 = create(&world);
    append_child(&world, node2 as u32, 1);
    let node3 = create(&world);

    let now = std::time::Instant::now();
    for i in 4..100 {
        create(&world);
    }
    println!("create node: {:?}", std::time::Instant::now() - now);

    let now = std::time::Instant::now();
    for i in 4..100 {
        append_child(&world, i, node3 as u32);
    }
    println!("append node: {:?}", std::time::Instant::now() - now);

    let now = std::time::Instant::now();
    append_child(&world, node3 as u32, node2 as u32);
    println!("append node3: {:?}", std::time::Instant::now() - now);

    let now = std::time::Instant::now();
    world.run(&RENDER_DISPATCH);
    println!("run: {:?}", std::time::Instant::now() - now);

    println!("---------------------------------------------");
    let node4 = create(&world);
    let now = std::time::Instant::now();
    for _ in 1..10000 {
        create(&world);
    }
    println!("create node------------: {:?}", std::time::Instant::now() - now);

    let now = std::time::Instant::now();
    for i in 1..10000 {
        append_child(&world, i + 100, node3 as u32);
    }
    println!("append node------------: {:?}", std::time::Instant::now() - now);

    let now = std::time::Instant::now();
    append_child(&world, node4 as u32, node2 as u32);
    println!("append node4------------: {:?}", std::time::Instant::now() - now);

    let now = std::time::Instant::now();
    world.run(&RENDER_DISPATCH);
    println!("run------------: {:?}", std::time::Instant::now() - now);
}

fn create(world: &World) -> usize {
    let idtree = world.fetch_single::<IdTree>().unwrap(); // 10000 节点， 139us
    let idtree = idtree.lend_mut(); // 10000 节点， 64us
    let node = world.create_entity::<Node>(); // 10000 节点， 300us （一次hash表查询， 一次lend_mut, 一次slab插入）
    let border_radius = world.fetch_multi::<Node, BorderRadius>().unwrap(); // 10000 节点， 150us
    let border_radius = border_radius.lend_mut(); // 10000 节点， 90us
    border_radius.insert(node, BorderRadius{x: LengthUnit::Pixel(0.0), y: LengthUnit::Pixel(0.0)}); // 470 us   （单独测试的插入没有这个费， 因为内存缓冲？）
    idtree.create(node); // 400us  (只是vecmap插入， 不应该这么慢， 正常时间应该在40us左右)
    node
}

fn append_child(world: &World, child: u32, parent: u32){
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.lend_mut();
    let notify = idtree.get_notify();
    idtree.insert_child(child as usize, parent as usize, 0, Some(&notify));
}


fn main(){
    test_time();
}