// oct 改变需要立即通知，以根据旧的oct，更新脏区域 TODO

use std::any::TypeId;
use std::marker::PhantomData;
use std::{default::Default};

use bevy_ecs::component::{Component, ComponentId, StorageType};
use bevy_ecs::entity::EntityLocation;
use bevy_ecs::event::Events;
use bevy_ecs::prelude::{IntoExclusiveSystem, IntoSystem, Entity, Query, Res, In, ResMut, Schedule, System, SystemStage, World, ParallelSystemDescriptorCoercion};
// use bevy_ecs::system::IntoExclusiveSystem;
use bevy_ecs::schedule::{ShouldRun, Stage};
use atom::Atom;
use bevy_ecs::system::{CommandQueue, ResState};
use hal_core::*;

use crate::single::IdTree;
use crate::system::style_mark::{userstyle_mark};
use crate::util::event::{EntityEvent, EntityEventData, EventType, ImMessenger, add_listener, send_im_event};
use res::{ResMap, ResMgr};
use share::Share;

use crate::component::{calc::*, calc::LayoutR};
use crate::component::user::*;
use crate::component::user::{Overflow, Opacity as UserOpacity};
use crate::font::font_sheet::FontSheet;
use crate::render::engine::{ShareEngine, UnsafeMut};
use crate::render::res::*;
use crate::single::*;
use crate::single::DirtyViewRect;
use crate::system::util::constant::*;
use crate::Z_MAX;
use crate::util::cell::StdCell;
use crate::system::style_mark::{matrix_mark, layout_mark};
use crate::util::event::ImEvent;

lazy_static! {
    pub static ref RENDER_DISPATCH: Atom = Atom::from("render_dispatch");
	pub static ref LAYOUT_DISPATCH: Atom = Atom::from("layout_dispatch");
	pub static ref CALC_DISPATCH: Atom = Atom::from("calc_dispatch");
    pub static ref ZINDEX_N: Atom = Atom::from("z_index_sys");
    pub static ref SHOW_N: Atom = Atom::from("show_sys");
    pub static ref WORLD_MATRIX_N: Atom = Atom::from("world_matrix_sys");
    pub static ref OCT_N: Atom = Atom::from("oct_sys");
    pub static ref LYOUT_N: Atom = Atom::from("layout_sys");
	pub static ref TEXT_LAYOUT_N: Atom = Atom::from("text_layout_sys");
	pub static ref TEXT_LAYOUT_UPDATE_N: Atom = Atom::from("text_layout_update_sys");
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

pub trait ContextId {
	fn id() -> usize;
}


#[macro_use()]
macro_rules! gen_com_index_map {
    (@main $maxIndex:expr, $($ty:ident, $index:expr),*) => {
		$crate::paste::item! {
			
			pub struct ComContext([ComponentId; $maxIndex], [Option<usize>; $maxIndex], usize);

			impl ComContext {
				fn init(world: &mut World) -> Self {
					ComContext(
						[$(world.components_mut().get_or_insert_id::<$ty>()),*],
						[$(
							match world.get_resource::<ImEvent<EntityEvent<$ty>>>() {
								Some(r) => Some(r as *const ImEvent<EntityEvent<$ty>> as usize),
								None => None,
							}
						),*],
						world.get_resource::<ImEvent<EntityEventData>>().unwrap() as *const ImEvent<EntityEventData> as usize,
					)
				}

				#[inline]
				pub fn get_mut<'a, T: ContextId + Component>(&self, world: &'a World, entity: Entity) -> Option<&'a mut T> {
					ComponentRef(self.0[<T as ContextId>::id()], PhantomData).get_mut(world, entity)
				}

				// #[inline]
				// pub fn send_im_event<'a, T: ContextId + Component>(&self, world: &'a World, e: E) -> Option<&'a mut T> {
				// 	ComponentRef(self.0[<T as ContextId>::id()], PhantomData).get_mut(world, entity)
				// }

				pub fn send_modify_event<T: ContextId + Component>(&self, entity: bevy_ecs::prelude::Entity, ty: StyleIndex, world: &World) {
					unsafe{ &mut *(self.2 as *mut ImEvent<EntityEventData>)}.send(EntityEventData{ty: EventType::Modify, style_index: ty, id: entity}, unsafe{&mut *(world as *const World as usize as *mut World)});

					match self.1[<T as ContextId>::id()] {
						Some(r) => {
							unsafe{ &mut *(r as *mut ImEvent<EntityEvent<T>>)}.send(EntityEvent::<T>::new_modify(entity, ty), unsafe{&mut *(world as *const World as usize as *mut World)});
						},
						None => (),
					};
				}

				pub fn send_create_event<T: ContextId + Component>(&self, entity: bevy_ecs::prelude::Entity, ty: StyleIndex, world: &World) {
					unsafe{ &mut *(self.2 as *mut ImEvent<EntityEventData>)}.send(EntityEventData{ty: EventType::Create, style_index: ty, id: entity}, unsafe{&mut *(world as *const World as usize as *mut World)});

					match self.1[<T as ContextId>::id()] {
						Some(r) => {
							unsafe{ &mut *(r as *mut ImEvent<EntityEvent<T>>)}.send(EntityEvent::<T>::new_create(entity), unsafe{&mut *(world as *const World as usize as *mut World)});
						},
						None => (),
					};
				}

				pub fn send_delete_event<T: ContextId + Component>(&self, entity: bevy_ecs::prelude::Entity, ty: StyleIndex, world: &World) {
					unsafe{ &mut *(self.2 as *mut ImEvent<EntityEventData>)}.send(EntityEventData{ty: EventType::Delete, style_index: ty, id: entity}, unsafe{&mut *(world as *const World as usize as *mut World)});

					match self.1[<T as ContextId>::id()] {
						Some(r) => {
							unsafe{ &mut *(r as *mut ImEvent<EntityEvent<T>>)}.send(EntityEvent::<T>::new_delete(entity), unsafe{&mut *(world as *const World as usize as *mut World)});
						},
						None => (),
					};
				}
			}

			$(impl ContextId for $ty {
				fn id() -> usize {
					return $index;
				}
			})*
		}
    };
	($maxIndex:expr, $($ty:ident, $index:expr),*) => {
		gen_com_index_map!(@main $maxIndex,$($ty, $index),*);
	};
	($maxIndex:expr, $($ty:ident, $index:expr),*,) => {
		gen_com_index_map!(@main $maxIndex,$($ty, $index),*);
	};
}

// 静态展开组件，让外部更高效的修改组件（减少hash表的查找）
gen_com_index_map!(
	27,
	RectLayoutStyle, 0,
	OtherLayoutStyle, 1,
	BackgroundColor, 2,
	TextStyle, 3,
	Transform, 4,
	NodeState, 5,
	ZIndex, 6,
	Overflow, 7,
	Show, 8,
	UserOpacity, 9,
	BoxShadow, 10,
	BorderColor, 11,
	BorderImage, 12,
	BorderImageClip, 13,
	BorderImageSlice, 14,
	BorderImageRepeat, 15,
	TextContent, 16,
	Font, 17,
	BorderRadius, 18,
	Image, 19,
	ImageClip, 20,
	MaskImage, 21,
	MaskImageClip, 22,
	ObjectFit, 23,
	Filter, 24,
	ClassName, 25,
	TransformWillChange, 26,
);

#[derive(Debug, Clone, Copy)]
pub struct ComponentRef<T: Component>(pub (crate) ComponentId, PhantomData<T>);

impl<T: Component> ComponentRef<T> {
	pub fn get(self, world: &World, entity: Entity) -> Option<&T> {
		match world
		.entities()
		.get(entity){
			Some(location) => {
				unsafe{get_component(world, self.0, entity, location).map(|value| &*value.cast::<T>())}
			},
			None => None
		}
	}

	pub fn get_mut(self, world: &World, entity: Entity) -> Option<&mut T> {
		let world = unsafe {&mut *(world as *const World as usize as *mut World)};
		match world
		.entities()
		.get(entity){
			Some(location) => {
				unsafe{get_component(world, self.0, entity, location).map(|value| &mut *value.cast::<T>())}
			},
			None => None
		}
	}
}

unsafe fn get_component(
	world: &World,
	component_id: ComponentId,
	entity: Entity,
	location: EntityLocation,
) -> Option<*mut u8> {
	let archetype = &world.archetypes()[location.archetype_id];
	// SAFE: component_id exists and is therefore valid
	let component_info = world.components().get_info_unchecked(component_id);
	match component_info.storage_type() {
		StorageType::Table => {
			let table = &world.storages().tables[archetype.table_id()];
			let components = table.get_column(component_id)?;
			let table_row = archetype.entity_table_row(location.index);
			// SAFE: archetypes only store valid table_rows and the stored component type is T
			Some(components.get_data_unchecked(table_row))
		}
		StorageType::SparseSet => world
			.storages()
			.sparse_sets
			.get(component_id)
			.and_then(|sparse_set| sparse_set.get(entity)),
	}
}
// pub struct ComContext ();


// pub enum CoreStage {
//     /// Runs once at the beginning of the app.
//     Startup,
//     /// Name of app stage that runs before all other app stages
//     First,
//     /// Name of app stage responsible for performing setup before an update. Runs before UPDATE.
//     PreUpdate,
//     /// Name of app stage responsible for doing most app logic. Systems should be registered here
//     /// by default.
//     Update,
//     /// Name of app stage responsible for processing the results of UPDATE. Runs after UPDATE.
//     PostUpdate,
//     /// Name of app stage that runs after all other app stages
//     Last,
// }

/// 设置资源管理器
pub fn seting_res_mgr(res_mgr: &mut ResMgr) {
    res_mgr.register::<TextureRes>(
		10 * 1024 * 1024,
		50 * 1024 * 1024,
		5 * 60,
		0,
        "TextureRes".to_string(),
    );
    res_mgr.register::<GeometryRes>(
        20 * 1024, 100 * 1024, 5 * 60, 0,
        "GeometryRes".to_string(),
    );
    res_mgr.register::<BufferRes>(
        20 * 1024, 100 * 1024, 5 * 60, 0,
        "BufferRes".to_string(),
    );

    res_mgr.register::<SamplerRes>(
        512, 1024, 60 * 60, 0,
        "SamplerRes".to_string(),
    );
    res_mgr.register::<RasterStateRes>(
        512, 1024, 60 * 60, 0,
        "RasterStateRes".to_string(),
    );
    res_mgr.register::<BlendStateRes>(
        512, 1024, 60 * 60, 0,
        "BlendStateRes".to_string(),
    );
    res_mgr.register::<StencilStateRes>(
        512, 1024, 60 * 60, 0,
        "StencilStateRes".to_string(),
    );
    res_mgr.register::<DepthStateRes>(
        512, 1024, 60 * 60, 0,
        "DepthStateRes".to_string(),
    );

    res_mgr.register::<UColorUbo>(
        4 * 1024, 8 * 1024, 60 * 60, 0,
        "UColorUbo".to_string(),
    );
    res_mgr.register::<HsvUbo>(
        1 * 1024, 2 * 1024, 60 * 60, 0,
        "HsvUbo".to_string(),
    );
    res_mgr.register::<MsdfStrokeUbo>(
        1 * 1024, 2 * 1024, 60 * 60, 0,
        "MsdfStrokeUbo".to_string(),
    );
    res_mgr.register::<CanvasTextStrokeColorUbo>(
        1 * 1024, 2 * 1024, 60 * 60, 0,
        "CanvasTextStrokeColorUbo".to_string(),
    );
}

pub enum RunLabel {
	Layout = 1,
	Calc = 2,
	Render = 4,
}

pub struct RunType(pub u8);

pub fn should_layout(runType: Res<RunType>) -> ShouldRun {
	if runType.0 & 1 > 0 {
		ShouldRun::Yes
	} else {
		ShouldRun::No
	}
}
pub fn should_calc(runType: Res<RunType>) -> ShouldRun {
	if runType.0 & 2 > 0 {
		ShouldRun::Yes
	} else {
		ShouldRun::No
	}
}
pub fn should_render(runType: Res<RunType>) -> ShouldRun {
	if runType.0 & 4 > 0 {
		ShouldRun::Yes
	} else {
		ShouldRun::No
	}
}

pub fn create_world<C: HalContext + 'static>(
	mut engine: ShareEngine<C>,
    width: f32,
    height: f32,
    font_measure: Box<dyn Fn(usize, usize, char) -> f32>,
	font_texture: Share<TextureRes>,
	_cur_time: usize,

	share_class_sheet: Option<Share<StdCell<ClassSheet>>>,
	share_font_sheet: Option<Share<StdCell<FontSheet>>>,
) -> App {

	let capacity = 2000;
	let mut world = World::default();
	// world.capacity = capacity;

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

	
	let hsv_ubo_res = {
		let res_mgr_ref = engine.res_mgr.borrow();
		UnsafeMut::new(res_mgr_ref.fetch_map::<HsvUbo>(0).unwrap())
	};

    // let charblock_sys = CellCharBlockSys::<C>::new(CharBlockSys::with_capacity(
    //     &mut engine,
	// 	(font_texture.width, font_texture.height),
	// 	capacity,
    // ));
    // let border_image_sys = BorderImageSys::<C>::with_capacity(&mut engine, capacity);
    // let node_attr_sys = CellNodeAttrSys::<C>::new(NodeAttrSys::new(&engine.res_mgr.borrow()));

    // let clip_sys = ClipSys::<C>::new();
	// let image_sys = CellImageSys::new(ImageSys::with_capacity(&mut engine, capacity));
	// let mut sys_time = SystemTime::default();
	// sys_time.cur_time = cur_time;
    //user
    // world.register_entity::<Node>();
    // world.register_multi::<Node, Transform>();
    // world.register_multi::<Node, user::ZIndex>();
    // world.register_multi::<Node, Overflow>();
    // world.register_multi::<Node, Show>();
    // world.register_multi::<Node, user::Opacity>();
    // world.register_multi::<Node, BackgroundColor>();
    // world.register_multi::<Node, BoxShadow>();
    // world.register_multi::<Node, BorderColor>();
    // world.register_multi::<Node, BorderImage>();
    // world.register_multi::<Node, BorderImageClip>();
    // world.register_multi::<Node, BorderImageSlice>();
    // world.register_multi::<Node, BorderImageRepeat>();

    // // world.register_multi::<Node, CharBlock<L>>();
    // world.register_multi::<Node, TextStyle>();
    // world.register_multi::<Node, TextContent>();
    // world.register_multi::<Node, Font>();
    // world.register_multi::<Node, BorderRadius>();
    // world.register_multi::<Node, Image>();
    // world.register_multi::<Node, ImageClip>();
	// world.register_multi::<Node, MaskImage>();
    // world.register_multi::<Node, MaskImageClip>();
    // world.register_multi::<Node, ObjectFit>();
    // world.register_multi::<Node, Filter>();
    // world.register_multi::<Node, ClassName>();
    // world.register_multi::<Node, StyleMark>();
	// world.register_multi::<Node, TransformWillChange>();
	// world.register_multi::<Node, RectLayoutStyle>();
	// world.register_multi::<Node, OtherLayoutStyle>();
	// world.register_multi::<Node, NodeState>();

    // //calc
    // world.register_multi::<Node, ZDepth>();
    // world.register_multi::<Node, Enable>();
    // world.register_multi::<Node, Visibility>();
    // world.register_multi::<Node, WorldMatrix>();
    // world.register_multi::<Node, ByOverflow>();
    // world.register_multi::<Node, calc::Opacity>();
    // world.register_multi::<Node, LayoutR>();
    // world.register_multi::<Node, HSV>();
    // world.register_multi::<Node, Culling>();
	// world.register_multi::<Node, TransformWillChangeMatrix>();
	world.insert_resource::<UnsafeMut<ResMap<HsvUbo>>>(hsv_ubo_res);

	let mut idtree = IdTree::with_capacity(capacity);
	idtree.set_statistics_count(true);
    //single
    world.insert_resource::<Statistics>(Statistics::default());
    world.insert_resource::<IdTree>(idtree);
    world.insert_resource::<Oct>(Oct::with_capacity(capacity));
    world.insert_resource::<OverflowClip>(OverflowClip::default());
    world.insert_resource::<RenderObjs>(RenderObjs::with_capacity(capacity));
	world.insert_resource::<ShareEngine<C>>(engine);

	match share_font_sheet {
		Some(r) => world.insert_resource::<Share<StdCell<FontSheet>>>(r),
		None => world.insert_resource::<Share<StdCell<FontSheet>>>(Share::new(StdCell::new(FontSheet::new(font_texture, font_measure)))),
	}
    
	world.insert_resource::<RunType>(RunType(0));

	// 视图矩阵
	let view_matrix = ViewMatrix(WorldMatrix(Matrix4::new_nonuniform_scaling(&Vector3::new(1.0,1.0,1.0)), false));
	let slice: &[f32] = view_matrix.0.as_slice();
	world.insert_resource::<Share<ViewMatrixUbo>>(Share::new(ViewMatrixUbo::new(UniformValue::MatrixV4(Vec::from(slice)))));
    world.insert_resource::<ViewMatrix>(view_matrix);

	// 投影矩阵
    let projection_matrix = ProjectionMatrix::new(
        width,
        height,
        -Z_MAX - 1.0,
        Z_MAX + 1.0,
    );
	let slice: &[f32] = projection_matrix.0.as_slice();
	world.insert_resource::<Share<ProjectMatrixUbo>>(Share::new(ProjectMatrixUbo::new(UniformValue::MatrixV4(Vec::from(slice)))));
	world.insert_resource::<ProjectionMatrix>(projection_matrix);

    world.insert_resource::<RenderBegin>(RenderBegin(
        RenderBeginDesc::new(0, 0, width as i32, height as i32),
		None,
	));

	world.insert_resource::<DirtyViewRect>(DirtyViewRect(0.0, 0.0, width as f32, height as f32, true));

    world.insert_resource::<NodeRenderMap>(NodeRenderMap::with_capacity(capacity));
	match share_class_sheet {
		Some(r) => world.insert_resource::<Share<StdCell<ClassSheet>>>(r),
		None => world.insert_resource::<Share<StdCell<ClassSheet>>>(Share::new(StdCell::new(ClassSheet::default()))),
	}

    world.insert_resource::<UnitQuad>(unit_quad);
    world.insert_resource::<DefaultState>(default_state);
    world.insert_resource::<ImageWaitSheet>(ImageWaitSheet::default());
	world.insert_resource::<DirtyList>(DirtyList::with_capacity(capacity));
	world.insert_resource::<SingleChangeType>(SingleChangeType(0));
	
	// world.insert_resource::<SystemTime>(sys_time);

	// 默认样式
	world.insert_resource::<TextStyle>(TextStyle::default());

	world.insert_resource::<crate::system::layout::LayoutSys>(crate::system::layout::LayoutSys::default());
	world.insert_resource::<crate::system::world_matrix::WorldMatrixSys>(crate::system::world_matrix::WorldMatrixSys::default());
	world.insert_resource::<crate::system::zindex::ZIndexImpl>(crate::system::zindex::ZIndexImpl::default());
	world.insert_resource::<crate::system::background_color::BackgroundColorSys>(crate::system::background_color::BackgroundColorSys::default());
	world.insert_resource::<crate::system::background_color::BackgroundColorSys>(crate::system::background_color::BackgroundColorSys::default());
	world.insert_resource::<crate::system::render::RenderSys>(crate::system::render::RenderSys::default());

	
	let mut layout_schedule = Schedule::default().with_run_criteria(should_layout.system());
	let mut calc_schedule = Schedule::default().with_run_criteria(should_calc.system());
	let mut render_schedule = Schedule::default().with_run_criteria(should_render.system());
	let mut schedule = Schedule::default();
	let mut calc_stage = SystemStage::single_threaded();
	let mut render_stage = SystemStage::single_threaded();
	let mut clear_stage = SystemStage::single_threaded();
	let layout_stage = layout_plugin(&mut world);

	calc_stage.add_system(crate::system::zindex::z_system.system());

	// 渲染system
	render_stage.add_system(crate::system::render::background_color::run::<C>.label("background_color").before("handle_dirty_view"));
	render_stage.add_system(crate::system::render::node_attr::handle_dirty_view::<C>.label("handle_dirty_view").before("draw"));
	render_stage.add_system(crate::system::render::render::draw::<C>.label("draw").before("clear_mark"));
	render_stage.add_system(crate::system::style_mark::clear_mark.label("clear_mark"));

	clear_stage.add_system(World::clear_trackers.exclusive_system());
	
	layout_schedule.add_stage("layout", layout_stage);
	calc_schedule.add_stage("calc", calc_stage);
	render_schedule.add_stage("render", render_stage);

	schedule.add_stage("layout", layout_schedule);
	schedule = schedule.with_stage_after("layout", "calc", calc_schedule)
	.with_stage_after("calc", "render", render_schedule)
	.with_stage_after("render", "clear", clear_stage);


	// dispatch.build("z_index_sys, show_sys, filter_sys, opacity_sys, text_layout_sys, layout_sys, text_layout_update_sys, world_matrix_sys, text_glphy_sys, transform_will_change_sys, oct_sys, overflow_sys, background_color_sys, box_shadow_sys, border_color_sys, image_sys, border_image_sys, charblock_sys, clip_sys, node_attr_sys, render_sys, style_mark_sys".to_string(), &world);

	// dispatch.build(
    //     "layout_sys, world_matrix_sys, oct_sys".to_string(),
    //     &world,
    // );

    // world.register_system(ZINDEX_N.clone(), CellZIndexImpl::new(ZIndexImpl::with_capacity(capacity)));
    // world.register_system(SHOW_N.clone(), CellShowSys::new(ShowSys::default()));
    // world.register_system(FILTER_N.clone(), CellFilterSys::new(FilterSys::default()));
    // world.register_system(OPCITY_N.clone(), CellOpacitySys::new(OpacitySys::default()));
    // world.register_system(LYOUT_N.clone(), CellLayoutSys::new(LayoutSys::default()));
    // world.register_system(
    //     TEXT_LAYOUT_N.clone(),
    //     CellLayoutImpl::new(LayoutImpl::new()),
    // );
    // world.register_system(
    //     WORLD_MATRIX_N.clone(),
    //     CellWorldMatrixSys::new(WorldMatrixSys::with_capacity(capacity)),
    // );
    // world.register_system(OCT_N.clone(), CellOctSys::new(OctSys::default()));
    // world.register_system(
    //     OVERFLOW_N.clone(),
    //     CellOverflowImpl::new(OverflowImpl::default()),
    // );
    // world.register_system(IMAGE_N.clone(), image_sys);
    // world.register_system(CHAR_BLOCK_N.clone(), charblock_sys);
    // world.register_system(
    //     TEXT_GLPHY_N.clone(),
    //     CellTextGlphySys::<C>::new(TextGlphySys(PhantomData)),
    // );
    // world.register_system(
    //     TRANSFORM_WILL_CHANGE_N.clone(),
    //     CellTransformWillChangeSys::new(TransformWillChangeSys::default()),
    // );

    // // world.register_system(CHAR_BLOCK_SHADOW_N.clone(), CellCharBlockShadowSys::<L>::new(CharBlockShadowSys::new()));
    // world.register_system(
    //     BG_COLOR_N.clone(),
    //     CellBackgroundColorSys::<C>::new(BackgroundColorSys::with_capacity(capacity)),
    // );
    // world.register_system(
    //     BR_COLOR_N.clone(),
    //     CellBorderColorSys::<C>::new(BorderColorSys::with_capacity(capacity)),
    // );
    // world.register_system(
    //     BR_IMAGE_N.clone(),
    //     CellBorderImageSys::new(border_image_sys),
    // );
    // world.register_system(
    //     BOX_SHADOW_N.clone(),
    //     CellBoxShadowSys::<C>::new(BoxShadowSys::with_capacity(capacity)),
    // );
    // world.register_system(NODE_ATTR_N.clone(), node_attr_sys);
    // world.register_system(
    //     RENDER_N.clone(),
    //     CellRenderSys::<C>::new(RenderSys::default()),
    // );
    // world.register_system(CLIP_N.clone(), CellClipSys::new(clip_sys));
    // // world.register_system(WORLD_MATRIX_RENDER_N.clone(), CellRenderMatrixSys::new(RenderMatrixSys::new()));
    // // world.register_system(
    // //     RES_RELEASE_N.clone(),
    // //     CellResReleaseSys::<C>::new(ResReleaseSys::new()),
    // // );
    // world.register_system(
    //     STYLE_MARK_N.clone(),
    //     CellStyleMarkSys::<C>::new(StyleMarkSys::new()),
	// );
	
	// world.register_system(
    //     TEXT_LAYOUT_UPDATE_N.clone(),
    //     CellTextLayoutUpdateSys::new(TextLayoutUpdateSys::default()),
    // );

    // let mut dispatch = SeqDispatcher::default();
    // dispatch.build("z_index_sys, show_sys, filter_sys, opacity_sys, text_layout_sys, layout_sys, text_layout_update_sys, world_matrix_sys, text_glphy_sys, transform_will_change_sys, oct_sys, overflow_sys, background_color_sys, box_shadow_sys, border_color_sys, image_sys, border_image_sys, charblock_sys, clip_sys, node_attr_sys, render_sys, style_mark_sys".to_string(), &world);
    // world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);

    // // let mut dispatch = SeqDispatcher::default();
    // // dispatch.build("z_index_sys, show_sys, filter_sys, opacity_sys, layout_sys, text_layout_sys, world_matrix_sys, oct_sys, overflow_sys, clip_sys, world_matrix_render, background_color_sys, border_color_sys, box_shadow_sys, image_sys, border_image_sys, charblock_sys, charblock_shadow_sys, node_attr_sys, render_sys".to_string(), &world);
    // // world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);

    // let mut dispatch = SeqDispatcher::default();
    // dispatch.build(
    //     "layout_sys, world_matrix_sys, oct_sys".to_string(),
    //     &world,
    // );
	// world.add_dispatcher(LAYOUT_DISPATCH.clone(), dispatch);
	
	// let mut dispatch = SeqDispatcher::default();
    // dispatch.build("z_index_sys, show_sys, filter_sys, opacity_sys, text_layout_sys, layout_sys, text_layout_update_sys, world_matrix_sys, text_glphy_sys, transform_will_change_sys, oct_sys, overflow_sys, background_color_sys, box_shadow_sys, border_color_sys, image_sys, border_image_sys, charblock_sys, clip_sys, node_attr_sys, style_mark_sys".to_string(), &world);
    // world.add_dispatcher(CALC_DISPATCH.clone(), dispatch);

	// 添加添加监听器
	add_listener(&mut world, userstyle_mark.system());
	add_listener(&mut world, crate::system::zindex::entity_listen.system());
	add_listener(&mut world, crate::system::zindex::idtree_listen.system());
	add_listener(&mut world, crate::system::zindex::zindex_listen.system());

	add_listener(&mut world, crate::system::world_matrix::transform_listen.system());
	add_listener(&mut world, crate::system::world_matrix::layout_listen.system());
	add_listener(&mut world, crate::system::world_matrix::idtree_listen.system());
	
	add_listener(&mut world, crate::system::layout::idtree_listen.system());

	add_listener(&mut world, crate::system::oct::matrix_listen.system());

	add_listener(&mut world, crate::system::background_color::bgcolor_listen.system());
	
	add_listener(&mut world, crate::system::render::node_attr::handle_view::<C>.system());
	add_listener(&mut world, crate::system::render::node_attr::handle_project::<C>.system());
	// add_listener(crate::system::render::node_attr::handle_will_change.system());
	add_listener(&mut world, crate::system::render::node_attr::handle_entity_create::<C>.system());
	add_listener(&mut world, crate::system::render::node_attr::handle_opacity::<C>.system());
	add_listener(&mut world, crate::system::render::node_attr::handle_visibility::<C>.system());
	add_listener(&mut world, crate::system::render::node_attr::handle_culling::<C>.system());
	add_listener(&mut world, crate::system::render::node_attr::handle_hsv::<C>.system());
	add_listener(&mut world, crate::system::render::node_attr::handle_oct::<C>.system());
	add_listener(&mut world, crate::system::render::node_attr::handle_renderobjs::<C>.system());
	add_listener(&mut world, crate::system::render::node_attr::handle_zdepth::<C>.system());

	add_listener(&mut world, crate::system::render::render::renderobjs_listen.system());
	
	let idtree = ResQuery::<IdTree>::init(&world);
	let com_context = ComContext::init(&mut world);
	let events = ResQuery::<Events<Entity>>::init(&world);
	let dirty_list = ResQuery::<DirtyList>::init(&world);
	App {
		world,
		schedule,
		command_queue: CommandQueue::default(),
		idtree,
		com_context,
		events,
		dirty_list
	}
}

pub fn layout_plugin(world: &mut World) -> SystemStage {
	// 默认的其它组件
	world.insert_resource::<WorldMatrix>(WorldMatrix::default());
	world.insert_resource::<Transform>(Transform::default());
	world.insert_resource::<LayoutR>(LayoutR::default());

	let mut layout_stage = SystemStage::single_threaded();
	layout_stage.add_system(crate::system::layout::calc_layout.label("layout").before("world_matrix"));
	layout_stage.add_system(crate::system::world_matrix::calc_world_matrix.system().label("world_matrix").before("calc_oct"));
	layout_stage.add_system(crate::system::oct::calc_oct.system().label("calc_oct"));

	add_listener(world, matrix_mark.system());
	add_listener(world, layout_mark.system());

	layout_stage
}

pub struct App {
	schedule: Schedule,
	world: World,
	pub command_queue: CommandQueue,
	idtree: ResQuery<IdTree>,
	pub com_context: ComContext,
	pub events: ResQuery<Events<Entity>>,
	pub dirty_list: ResQuery<DirtyList>,
}

impl App {
	pub fn idtree(&self) -> &ResQuery<IdTree> {
		&self.idtree
	}
	pub fn idtree_mut(&mut self) -> &mut ResQuery<IdTree> {
		&mut self.idtree
	}
}

// unsafe
// 非安全 已经存在，且永不删除的Res的指针
pub struct ComponentQuery<T>(usize, PhantomData<T>);


impl<T: Component> ComponentQuery<T> {
	fn init(world: &World) -> Self {
		let resource = world.get_resource::<T>().unwrap();
		Self(resource as *const T as usize, PhantomData)
	}
}

impl<T> std::ops::Deref for ComponentQuery<T> {
	type Target = T;

    /// Dereferences the value.
    fn deref(&self) -> &Self::Target {
		unsafe{&*(self.0 as *const T)}
	}
}

impl<T> std::ops::DerefMut for ComponentQuery<T> {

    /// Dereferences the value.
    fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe{&mut *(self.0 as *mut T)}
	}
}


// unsafe
// 非安全 已经存在，且永不删除的Res的指针
pub struct ResQuery<T>(usize, PhantomData<T>);



impl<T: Component> ResQuery<T> {
	fn init(world: &World) -> Self {
		let resource = world.get_resource::<T>().unwrap();
		Self(resource as *const T as usize, PhantomData)
	}
}

impl<T> std::ops::Deref for ResQuery<T> {
	type Target = T;

    /// Dereferences the value.
    fn deref(&self) -> &Self::Target {
		unsafe{&*(self.0 as *const T)}
	}
}

impl<T> std::ops::DerefMut for ResQuery<T> {

    /// Dereferences the value.
    fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe{&mut *(self.0 as *mut T)}
	}
}

impl std::ops::Deref for App {
	type Target = World;
	fn deref(&self) -> &Self::Target {
		&self.world
	}
}

impl std::ops::DerefMut for App {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.world
	}
}

impl App {
	pub fn run(&mut self) {
		self.schedule.run(&mut self.world);
	}
}

// pub fn userstyle_mark11(
// 	mut dirty_list: ResMut<DirtyList>,
// 	mut style_marks: Query<&mut StyleMark>,
// 	// mut class_names: Query<&ClassName>,
// ) {
// 	log::info!("xxxxxxxxxxxxxxxxxxxxxx");
// }

// #[test]
// fn test1() {
// 	let mut world = World::default();
// 	world.insert_resource::<DirtyList>(DirtyList::with_capacity(0));
// 	let entity = world.spawn().insert(StyleMark::default()).id();
// 	add_listener(&mut world, userstyle_mark.system());
// 	send_im_event(&mut world, EntityEventData{ty: EventType::Modify, style_index: StyleIndex::Add , id: entity});
// 	send_im_event(&mut world, EntityEventData{ty: EventType::Modify, style_index: StyleIndex::Add , id: entity});
// 	// let mut layout_stage = SystemStage::single_threaded();
// 	// layout_stage.add_system(userstyle_mark11.system());
// 	// layout_stage.run(&mut world);
// }

#[cfg(test)]
pub fn matri(
	query: Query<&mut WorldMatrix>,
	mut writer: ImMessenger<EntityEvent<WorldMatrix>>,
) {
	writer.send(EntityEvent::<WorldMatrix>::new_modify(to_entity(0, 0), StyleIndex::Add));
}
#[cfg(test)]
pub fn matri_listen(
	e: In<EntityEvent<WorldMatrix>>,
	query: Query<(Option<&Transform>, &WorldMatrix, Option<&LayoutR>)>,
) {
	println!("xxxxxxxxxxxxxxxxxxx");
	log::info!("xxxxx============:{:?}", query.get(to_entity(0, 0)).unwrap());
}

fn test_insert11(world: &mut World) {
	for i in 0..200 {
		world.spawn().insert_bundle((BorderRadius {
			x: LengthUnit::Pixel(0.0),
			y: LengthUnit::Pixel(0.0),
		}, 
		Visibility(true),
		RectLayoutStyle::default(),
		OtherLayoutStyle::default(),
		StyleMark::default(),
		ZDepth::default(),
		crate::component::calc::Opacity::default(),
		HSV::default(),
		LayoutR::default(),
		WorldMatrix::default(),
		Enable::default(),
		NodeState::default(),
		ByOverflow::default(),
		Culling::default()));
		// BackgroundColor::default();
	}

	for i in 0..70 {
		world.entity_mut(to_entity(i, 0)).insert(BackgroundColor::default());
	}

	for i in 0..200 {
		world.get_mut::<RectLayoutStyle>(to_entity(i, 0)).unwrap().size.width = flex_layout::Dimension::Points(32.0);
		world.get_mut::<RectLayoutStyle>(to_entity(i, 0)).unwrap().size.height = flex_layout::Dimension::Points(32.0);
		world.get_mut::<OtherLayoutStyle>(to_entity(i, 0)).unwrap().align_content = flex_layout::AlignContent::Center;
		world.get_mut::<OtherLayoutStyle>(to_entity(i, 0)).unwrap().align_items = flex_layout::AlignItems::Center;
		world.get_mut::<OtherLayoutStyle>(to_entity(i, 0)).unwrap().align_self = flex_layout::AlignSelf::Center;
	}
}

fn test_insert22(world: &mut World) {
	for i in 0..200 {

		world.spawn().insert_bundle((BorderRadius {
			x: LengthUnit::Pixel(0.0),
			y: LengthUnit::Pixel(0.0),
		}, 
		RectLayoutStyle::default(),
		OtherLayoutStyle::default()));
		// BackgroundColor::default();

		world.spawn().insert_bundle((
			Visibility(true),
			StyleMark::default(),
			ZDepth::default(),
			crate::component::calc::Opacity::default(),
			HSV::default(),
			LayoutR::default(),
			WorldMatrix::default(),
			Enable::default(),
			NodeState::default(),
			ByOverflow::default(),
			Culling::default()));
	}

	

	for i in 0..70 {
		world.entity_mut(to_entity(i * 2, 0)).insert(BackgroundColor::default());
	}

	for i in 0..200 {
		world.get_mut::<RectLayoutStyle>(to_entity(i * 2, 0)).unwrap().size.width = flex_layout::Dimension::Points(32.0);
		world.get_mut::<RectLayoutStyle>(to_entity(i * 2, 0)).unwrap().size.height = flex_layout::Dimension::Points(32.0);
		world.get_mut::<OtherLayoutStyle>(to_entity(i * 2, 0)).unwrap().align_content = flex_layout::AlignContent::Center;
		world.get_mut::<OtherLayoutStyle>(to_entity(i * 2, 0)).unwrap().align_items = flex_layout::AlignItems::Center;
		world.get_mut::<OtherLayoutStyle>(to_entity(i * 2, 0)).unwrap().align_self = flex_layout::AlignSelf::Center;
	}
}


#[test]
fn test_insert() {
	let mut all_time1 = std::time::Duration::from_millis(0);
	for i in 0..100 {
		let mut world = World::default();
		let t = std::time::Instant::now();
		test_insert11(&mut world);
		all_time1 = all_time1 + (std::time::Instant::now() - t);
	}
	
	println!("time1: {:?}", all_time1/100);

	let mut all_time2 = std::time::Duration::from_millis(0);
	for i in 0..100 {
		let mut world = World::default();
		let t = std::time::Instant::now();
		test_insert22(&mut world);
		all_time2 = all_time2 + (std::time::Instant::now() - t);
	}
	
	println!("time2: {:?}", all_time2/100);
}

#[test]
fn test_matrix() {
	let mut world = World::default();
	let mut stage = SystemStage::single_threaded();
	stage.add_system(matri.system());
	add_listener(&mut world, matri_listen.system());

	let entity = world.spawn().insert(WorldMatrix::default()).id();
	log::info!("entity: {:?}", entity);

	stage.run(&mut world);

	// world.insert_resource::<DirtyList>(DirtyList::with_capacity(0));
	// let entity = world.spawn().insert(StyleMark::default()).id();
	// add_listener(&mut world, userstyle_mark.system());
	// send_im_event(&mut world, EntityEventData{ty: EventType::Modify, style_index: StyleIndex::Add , id: entity});
	// send_im_event(&mut world, EntityEventData{ty: EventType::Modify, style_index: StyleIndex::Add , id: entity});
	// let mut layout_stage = SystemStage::single_threaded();
	// layout_stage.add_system(userstyle_mark11.system());
	// layout_stage.run(&mut world);
	
}

fn init_world() -> App {
	let mut world = World::default();

	let mut idtree = IdTree::with_capacity(1000);
	idtree.set_statistics_count(true);
    //single
    world.insert_resource::<Statistics>(Statistics::default());
    world.insert_resource::<IdTree>(idtree);
    world.insert_resource::<Oct>(Oct::with_capacity(1000));
    world.insert_resource::<OverflowClip>(OverflowClip::default());
    world.insert_resource::<RenderObjs>(RenderObjs::with_capacity(1000));

	world.insert_resource::<RunType>(RunType(0));

	// 视图矩阵
	let view_matrix = ViewMatrix(WorldMatrix(Matrix4::new_nonuniform_scaling(&Vector3::new(1.0,1.0,1.0)), false));
	let slice: &[f32] = view_matrix.0.as_slice();
	world.insert_resource::<Share<ViewMatrixUbo>>(Share::new(ViewMatrixUbo::new(UniformValue::MatrixV4(Vec::from(slice)))));
    world.insert_resource::<ViewMatrix>(view_matrix);

	let width = 1000.0;
	let height = 2000.0;
	// 投影矩阵
    let projection_matrix = ProjectionMatrix::new(
        width,
        height,
        -Z_MAX - 1.0,
        Z_MAX + 1.0,
    );
	let slice: &[f32] = projection_matrix.0.as_slice();
	world.insert_resource::<Share<ProjectMatrixUbo>>(Share::new(ProjectMatrixUbo::new(UniformValue::MatrixV4(Vec::from(slice)))));
	world.insert_resource::<ProjectionMatrix>(projection_matrix);

    world.insert_resource::<RenderBegin>(RenderBegin(
        RenderBeginDesc::new(0, 0, width as i32, height as i32),
		None,
	));

	world.insert_resource::<DirtyViewRect>(DirtyViewRect(0.0, 0.0, width as f32, height as f32, true));

    world.insert_resource::<NodeRenderMap>(NodeRenderMap::with_capacity(1000));
	

    world.insert_resource::<ImageWaitSheet>(ImageWaitSheet::default());
	world.insert_resource::<DirtyList>(DirtyList::with_capacity(1000));
	world.insert_resource::<SingleChangeType>(SingleChangeType(0));
	
	// world.insert_resource::<SystemTime>(sys_time);

	// 默认样式
	world.insert_resource::<TextStyle>(TextStyle::default());

	world.insert_resource::<crate::system::layout::LayoutSys>(crate::system::layout::LayoutSys::default());
	world.insert_resource::<crate::system::world_matrix::WorldMatrixSys>(crate::system::world_matrix::WorldMatrixSys::default());
	world.insert_resource::<crate::system::zindex::ZIndexImpl>(crate::system::zindex::ZIndexImpl::default());
	world.insert_resource::<crate::system::background_color::BackgroundColorSys>(crate::system::background_color::BackgroundColorSys::default());
	world.insert_resource::<crate::system::background_color::BackgroundColorSys>(crate::system::background_color::BackgroundColorSys::default());
	world.insert_resource::<crate::system::render::RenderSys>(crate::system::render::RenderSys::default());
	world.insert_resource::<Events<Entity>>(Events::default());

	
	let mut layout_schedule = Schedule::default().with_run_criteria(should_layout.system());
	let mut calc_schedule = Schedule::default().with_run_criteria(should_calc.system());
	let mut render_schedule = Schedule::default().with_run_criteria(should_render.system());
	let mut schedule = Schedule::default();
	let mut calc_stage = SystemStage::single_threaded();
	let mut render_stage = SystemStage::single_threaded();
	let mut clear_stage = SystemStage::single_threaded();
	let layout_stage = layout_plugin(&mut world);

	calc_stage.add_system(crate::system::zindex::z_system.system());

	render_stage.add_system(crate::system::style_mark::clear_mark.label("clear_mark"));

	clear_stage.add_system(World::clear_trackers.exclusive_system());
	
	layout_schedule.add_stage("layout", layout_stage);
	calc_schedule.add_stage("calc", calc_stage);
	render_schedule.add_stage("render", render_stage);

	schedule.add_stage("layout", layout_schedule);
	schedule = schedule.with_stage_after("layout", "calc", calc_schedule)
	.with_stage_after("calc", "render", render_schedule)
	.with_stage_after("render", "clear", clear_stage);

	// 添加添加监听器
	add_listener(&mut world, userstyle_mark.system());
	add_listener(&mut world, crate::system::zindex::entity_listen.system());
	add_listener(&mut world, crate::system::zindex::idtree_listen.system());
	add_listener(&mut world, crate::system::zindex::zindex_listen.system());

	add_listener(&mut world, crate::system::world_matrix::transform_listen.system());
	add_listener(&mut world, crate::system::world_matrix::layout_listen.system());
	add_listener(&mut world, crate::system::world_matrix::idtree_listen.system());
	
	add_listener(&mut world, crate::system::layout::idtree_listen.system());

	add_listener(&mut world, crate::system::oct::matrix_listen.system());

	add_listener(&mut world, crate::system::background_color::bgcolor_listen.system());

	add_listener(&mut world, crate::system::render::render::renderobjs_listen.system());
	
	let idtree = ResQuery::<IdTree>::init(&world);
	let events = ResQuery::<Events<Entity>>::init(&world);
	let dirty_list = ResQuery::<DirtyList>::init(&world);
	let com_context = ComContext::init(&mut world);
	App {
		world,
		schedule,
		command_queue: CommandQueue::default(),
		idtree,
		com_context,
		events,
		dirty_list
	}
	
}

#[test]
fn test_world() {
	let mut all_time1 = std::time::Duration::from_millis(0);
	for i in 0..100 {
		let mut app = init_world();
		let t = std::time::Instant::now();
		set_world(&mut app);
		all_time1 = all_time1 + (std::time::Instant::now() - t);
	}
	
	println!("time1: {:?}", all_time1/100);

	let mut all_time2 = std::time::Duration::from_millis(0);
	for i in 0..100 {
		let mut app = init_world();
		let t = std::time::Instant::now();
		set_world_sample(&mut app);
		all_time2 = all_time2 + (std::time::Instant::now() - t);
	}
	
	println!("time2: {:?}", all_time2/100);
	

}

fn set_world(world: &mut App) {
	for i in 0..200 {
		let entity = world.spawn().insert_bundle((BorderRadius {
			x: LengthUnit::Pixel(0.0),
			y: LengthUnit::Pixel(0.0),
		}, 
		Visibility(true),
		RectLayoutStyle::default(),
		OtherLayoutStyle::default(),
		StyleMark::default(),
		ZDepth::default(),
		crate::component::calc::Opacity::default(),
		HSV::default(),
		LayoutR::default(),
		WorldMatrix::default(),
		Enable::default(),
		NodeState::default(),
		ByOverflow::default(),
		Culling::default())).id();
		// BackgroundColor::default();
		world.idtree_mut().create(entity.id() as usize, entity.generation());
		send_im_event(world, EntityEvent::<Entity>::new_create(entity));
		
	}

	for i in 0..70 {
		world.entity_mut(to_entity(i, 0)).insert(BackgroundColor::default());
		// world.com_context.send_modify_event::<BackgroundColor>(to_entity(i, 0), StyleIndex::BackgroundColor, world);
	}

	for i in 0..200 {
		world.get_mut::<RectLayoutStyle>(to_entity(i, 0)).unwrap().size.width = flex_layout::Dimension::Points(32.0);
		// world.com_context.send_modify_event::<RectLayoutStyle>(to_entity(i, 0), StyleIndex::Width, world);
		world.get_mut::<RectLayoutStyle>(to_entity(i, 0)).unwrap().size.height = flex_layout::Dimension::Points(32.0);
		// world.com_context.send_modify_event::<RectLayoutStyle>(to_entity(i, 0), StyleIndex::Height, world);
		world.get_mut::<OtherLayoutStyle>(to_entity(i, 0)).unwrap().align_content = flex_layout::AlignContent::Center;
		// world.com_context.send_modify_event::<OtherLayoutStyle>(to_entity(i, 0), StyleIndex::AlignContent, world);
		world.get_mut::<OtherLayoutStyle>(to_entity(i, 0)).unwrap().align_items = flex_layout::AlignItems::Center;
		// world.com_context.send_modify_event::<OtherLayoutStyle>(to_entity(i, 0), StyleIndex::AlignItems, world);
		world.get_mut::<OtherLayoutStyle>(to_entity(i, 0)).unwrap().align_self = flex_layout::AlignSelf::Center;
		// world.com_context.send_modify_event::<OtherLayoutStyle>(to_entity(i, 0), StyleIndex::AlignSelf, world);
	}
}

fn set_world_sample(world: &mut App) {
	for i in 0..200 {
		let entity = world.spawn().insert_bundle((BorderRadius {
			x: LengthUnit::Pixel(0.0),
			y: LengthUnit::Pixel(0.0),
		}, 
		Visibility(true),
		RectLayoutStyle::default(),
		OtherLayoutStyle::default(),
		StyleMark::default(),
		ZDepth::default(),
		crate::component::calc::Opacity::default(),
		HSV::default(),
		LayoutR::default(),
		WorldMatrix::default(),
		Enable::default(),
		NodeState::default(),
		ByOverflow::default(),
		Culling::default())).id();
		// BackgroundColor::default();
		world.idtree_mut().create(entity.id() as usize, entity.generation());
		world.events.send(entity);
	}

	for i in 0..70 {
		world.entity_mut(to_entity(i, 0)).insert(BackgroundColor::default());
		world.com_context.send_modify_event::<BackgroundColor>(to_entity(i, 0), StyleIndex::BackgroundColor, world);
	}

	for i in 0..200 {
		world.get_mut::<RectLayoutStyle>(to_entity(i, 0)).unwrap().size.width = flex_layout::Dimension::Points(32.0);
		world.get_mut::<RectLayoutStyle>(to_entity(i, 0)).unwrap().size.height = flex_layout::Dimension::Points(32.0);
		world.get_mut::<OtherLayoutStyle>(to_entity(i, 0)).unwrap().align_content = flex_layout::AlignContent::Center;
		world.get_mut::<OtherLayoutStyle>(to_entity(i, 0)).unwrap().align_items = flex_layout::AlignItems::Center;
		world.get_mut::<OtherLayoutStyle>(to_entity(i, 0)).unwrap().align_self = flex_layout::AlignSelf::Center;
	}
}
