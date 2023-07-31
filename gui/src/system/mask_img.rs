
use std::marker::PhantomData;
use std::hash::Hash;
use std::hash::Hasher;
use std::cell::RefCell;

use ecs::SingleCaseListener;
//八叉树系统
use ecs::{CreateEvent, Event, ModifyEvent, MultiCaseImpl, Runner, SingleCaseImpl, MultiCaseListener}; 

use flex_layout::Size;
use hal_core::*;
use hash::{ DefaultHasher};
use atom::Atom;
use ordered_float::NotNan;
use share::Share;

use crate::Z_MAX;
use crate::component::calc::ViewMatrixUbo;
use crate::component::calc::MaskTexture;
use crate::component::user::*;
use crate::entity::Node;
use crate::render::engine::AttributeDecs;
use crate::render::engine::ShareEngine;
use crate::render::res::GeometryRes;
use crate::single::Oct;
use crate::single::dyn_texture::DynAtlasSet;
use crate::single::{IdTree, CommonState, State, RenderObj, PreRenderList};
use crate::single::PreRenderItem;
use crate::render::res::TexturePartRes;
use crate::system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};
use crate::component::calc::ColorParamter;
use crate::single::DefaultState;
use crate::component::calc::WorldMatrixUbo;
use crate::single::ProjectionMatrix;
use crate::component::calc::ProjectMatrixUbo;

use super::linear_gradient_split;
use super::util::calc_float_hash;
use super::util::calc_hash;
use super::util::new_render_obj;
use super::util::to_vex_color_defines;
lazy_static! {
    // 四边形几何体的hash值
    pub static ref QUAD_GEO_HASH: u64 = 0;
	static ref MASK_IMAGE_TEXTURE: Atom = Atom::from("MASK_IMAGE_TEXTURE");
}

#[derive(Deref, DerefMut)]
pub struct MaskImageSys<C: HalContext + 'static>(Vec<usize>, PhantomData<C>);

impl<C: HalContext + 'static> MaskImageSys<C> {
	pub fn new() -> Self {
		MaskImageSys(Vec::new(), PhantomData)
	}
}


impl<'a, C: HalContext + 'static> Runner<'a> for MaskImageSys<C> {
	type ReadData = (
		&'a SingleCaseImpl<IdTree>, 
		&'a MultiCaseImpl<Node, MaskImage>,
		&'a SingleCaseImpl<DefaultState>,
		&'a SingleCaseImpl<Oct>,
	);
    type WriteData = (
		&'a mut MultiCaseImpl<Node, MaskTexture>, 
		&'a mut SingleCaseImpl<Share<RefCell<DynAtlasSet>>>,
		&'a mut SingleCaseImpl<ShareEngine<C>>,
		// &'a mut SingleCaseImpl<RenderObjs>,
		&'a mut SingleCaseImpl<PreRenderList>,
	);
	fn run(&mut self, (idtree, mask_images, default_state, octree): Self::ReadData, (mask_texture, dyn_atlas_set, mut engine, pre_render_list): Self::WriteData) {
		if self.0.len() == 0 {
			return;
		}

		for id in self.0.iter() {
			let id = *id;
			if let Some(node) = idtree.get(id) {
				if node.layer() == 0 {
					return;
				}
	
				let mask_image = match mask_images.get(id) {
					Some(r) => r,
					None => return,
				};
	
				if let MaskImage::LinearGradient(color) = mask_image {
					let oct = octree.get(id).unwrap();
					let size = calc_size(oct.0, color) as u32;
					let mut hasher = DefaultHasher::default();
					MASK_IMAGE_TEXTURE.hash(&mut hasher);
					color.hash(&mut hasher);
					size.hash(&mut hasher);
					let hash = hasher.finish();
					
					let texture = match engine.texture_part_res_map.get(&hash) {
						Some(r) => r,
						None => {
							let size = size as f32;
							let index = dyn_atlas_set.borrow_mut().update_or_add_rect(0, 0, size, size, PixelFormat::RGB, DataFormat::UnsignedByte, false, 1, 2, &mut engine.gl);
							// log::info!("mask update_or_add_rect============={}", index);
							let texture = TexturePartRes::new(index, dyn_atlas_set.clone());
							let cost = texture.cost();
							
							// 创建render_obj
							let obj = create_render_obj(
										color,
										&texture,
								COLOR_VS_SHADER_NAME.clone(),
								COLOR_FS_SHADER_NAME.clone(),
								Share::new(ColorParamter::default()),
								default_state,
								
								&mut engine
							);

							pre_render_list.push(PreRenderItem{index, obj});
	
							engine.texture_part_res_map.create(hash, texture, cost, 0)
						}
					};
					mask_texture.insert(id, MaskTexture::Part(texture));
				}
			}
		}

		self.0.clear();
	}
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, MaskImage, (CreateEvent, ModifyEvent)>
    for MaskImageSys<C>
{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &Event, _: Self::ReadData, _: Self::WriteData) {
		self.0.push(event.id);
	}
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, IdTree, ModifyEvent>
    for MaskImageSys<C>
{
    type ReadData = &'a MultiCaseImpl<Node, MaskImage>;
    type WriteData = ();
    fn listen(&mut self, event: &Event, mask_images: Self::ReadData, _: Self::WriteData) {
		if event.field == "totree" {
			if let Some(r) = mask_images.get(event.id) {
				if let MaskImage::LinearGradient(_r) = r {
					self.0.push(event.id);
				}
			}
		}
	}
}

#[inline]
pub fn create_render_obj<C: HalContext + 'static>(
    color: &LinearGradientColor,
	texture: &TexturePartRes,
    vs_name: Atom,
    fs_name: Atom,
    paramter: Share<dyn ProgramParamter>,
    default_state: &CommonState,
	engine: &mut ShareEngine<C>
) -> RenderObj {
    let state = State {
        bs: default_state.df_bs.clone(),
        rs: default_state.df_rs.clone(),
        ss: default_state.df_ss.clone(),
        ds: default_state.df_ds.clone(),
    };
	let rect = texture.get_rect();
	let mut render_obj = new_render_obj(
		0, 0.0, true, vs_name, fs_name, paramter, state,
	);

	to_vex_color_defines(
		render_obj.vs_defines.as_mut(),
		render_obj.fs_defines.as_mut(),
	);
	let matrix = vec![
		1.0, 0.0, 0.0, 0.0, 
		0.0, 1.0, 0.0, 0.0,
		0.0, 0.0, 1.0, 0.0,
		0.0, 0.0, 0.0, 1.0
	];
	render_obj.paramter.set_value(
        "worldMatrix",
        Share::new(WorldMatrixUbo::new(UniformValue::MatrixV4(matrix.clone()))),
	);
	render_obj.paramter.set_value(
        "viewMatrix",
        Share::new(ViewMatrixUbo::new(UniformValue::MatrixV4(matrix))),
	);
	
	let project_martix = ProjectionMatrix::new(
		rect.maxs.x - rect.mins.x,
		rect.maxs.y - rect.mins.y,
		-Z_MAX - 1.0,
		Z_MAX + 1.0,
	);
	let buffer = Vec::from(project_martix.0.as_slice());
	render_obj.paramter.set_value(
        "projectMatrix",
        Share::new(ProjectMatrixUbo::new(UniformValue::MatrixV4(buffer))),
	);
	render_obj.paramter.set_single_uniform(
        "depth",
        UniformValue::Float1(0.0),
	);
	render_obj.paramter.set_single_uniform(
        "blur",
        UniformValue::Float1(1.0),
	);
	// alpha, depth

	render_obj.geometry = create_linear_gradient_geo(
        &rect,
		color,
        engine,
    );

	render_obj
}

fn create_linear_gradient_geo<C: HalContext + 'static>(rect: &Aabb2, color: &LinearGradientColor, engine: &mut ShareEngine<C>) -> Option<Share<GeometryRes>>{
	let size = Size {width: NotNan::new(rect.maxs.x - rect.mins.x).unwrap(), height: NotNan::new(rect.maxs.y - rect.mins.y).unwrap()};
	let (positions, indices) = (
		vec![
			rect.mins.x, rect.mins.y, // left_top
			rect.mins.x, rect.maxs.y, // left_bootom
			rect.maxs.x, rect.maxs.y, // right_bootom
			rect.maxs.x, rect.mins.x, // right_top
		],
		vec![0, 1, 2, 3],
	);

	let hash = calc_hash(&"linear_gradient geo", calc_hash(color, calc_float_hash(&positions.as_slice(), 0)));
	match engine.geometry_res_map.get(&hash) {
		Some(r) => Some(r),
		None => {
			let (positions, colors, indices) = linear_gradient_split(color, positions, indices, &size);
			Some(engine.create_geo_res(
				hash,
				indices.as_slice(),
				&[
					AttributeDecs::new(AttributeName::Position, positions.as_slice(), 2),
					AttributeDecs::new(AttributeName::Color, colors.as_slice(), 4),
				],
			))
		}
	}
}


fn calc_size(oct: &Aabb2, linear: &LinearGradientColor) -> u32 {
	let width = oct.maxs.x - oct.mins.x;
	let height = oct.maxs.y - oct.mins.y;

	let l =  (width * width + height * height).sqrt();
	let mut min: f32 = 1.0;
	let mut pre_pos: f32 = 0.0;
	for item in linear.list.iter() {
		let diff = item.position - pre_pos;
		if diff != 0.0 {
			min = min.min(diff);
			pre_pos = item.position;
		}
	}

	if min == 1.0 {
		return 10;
	}

	// 保证渐变百分比中，渐变端点之间的距离至少两个像素
	let at_least =  (2.0_f32.min((min * l).ceil() + 1.0)/min).min(width.max(height) / 4.0);
	// 渐变颜色渲染尺寸为20的整数倍，使得不同大小的渐变色，可以共用同一张纹理
	// 加2，使得分配的纹理四周可以扩充一个像素，避免采样问题导致边界模糊 TODO
	return ((at_least/10.0).ceil() * 10.0) as u32;
}

impl_system! {
    MaskImageSys<C> where [C: HalContext + 'static],
    true,
    {
		MultiCaseListener<Node, MaskImage, (CreateEvent, ModifyEvent)>
		SingleCaseListener<IdTree, ModifyEvent>
    }
}

// #[cfg(test)]
// use atom::Atom;
// #[cfg(test)]
// use component::calc::{ZDepth};
// #[cfg(test)]
// use component::user::{TransformFunc, TransformWrite};
// #[cfg(test)]
// use ecs::{Dispatcher, LendMut, SeqDispatcher, World};
// #[cfg(test)]
// use system::world_matrix::{CellWorldMatrixSys, WorldMatrixSys};
// #[cfg(test)]
// use flex_layout::Rect;

// #[test]
// fn test() {
//     let world = new_world();

//     let idtree = world.fetch_single::<IdTree>().unwrap();
//     let idtree = LendMut::lend_mut(&idtree);
//     let oct = world.fetch_single::<Oct>().unwrap();
//     let oct = LendMut::lend_mut(&oct);
//     let notify = idtree.get_notify();
//     let transforms = world.fetch_multi::<Node, Transform>().unwrap();
//     let transforms = LendMut::lend_mut(&transforms);
//     let layouts = world.fetch_multi::<Node, LayoutR>().unwrap();
//     let layouts = LendMut::lend_mut(&layouts);
//     let world_matrixs = world.fetch_multi::<Node, WorldMatrix>().unwrap();
//     let _world_matrixs = LendMut::lend_mut(&world_matrixs);
//     let zdepths = world.fetch_multi::<Node, ZDepth>().unwrap();
//     let zdepths = LendMut::lend_mut(&zdepths);

//     let e0 = world.create_entity::<Node>();

//     idtree.create(e0);
//     transforms.insert(e0, Transform::default());
//     zdepths.insert(e0, ZDepth::default());
//     layouts.insert(
//         e0,
//         LayoutR {
// 			rect: Rect{start: 0.0, end: 900.0, top: 0.0, bottom: 900.0},
// 			border: Rect{start: 0.0, end: 900.0, top: 0.0, bottom: 900.0},
// 			padding: Rect{start: 0.0, end: 900.0, top: 0.0, bottom: 900.0},
//         },
//     );
//     idtree.insert_child(e0, 0, 0); //根

//     let e00 = world.create_entity::<Node>();
//     let e01 = world.create_entity::<Node>();
//     let e02 = world.create_entity::<Node>();
//     idtree.create(e00);
//     transforms.insert(e00, Transform::default());
//     zdepths.insert(e00, ZDepth::default());
//     layouts.insert(
//         e00,
//         LayoutR {
// 			rect: Rect{start: 0.0, end: 300.0, top: 0.0, bottom: 900.0},
// 			border: Rect{start: 0.0, end: 300.0, top: 0.0, bottom: 900.0},
// 			padding: Rect{start: 0.0, end: 300.0, top: 0.0, bottom: 900.0},
//         },
//     );
//     idtree.insert_child(e00, e0, 1);

//     idtree.create(e01);
//     layouts.insert(
//         e01,
//         Layout {
//             left: 300.0,
//             top: 0.0,
//             width: 300.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     transforms.insert(e01, Transform::default());
//     zdepths.insert(e01, ZDepth::default());
//     idtree.insert_child(e01, e0, 2);

//     idtree.create(e02);
//     transforms.insert(e02, Transform::default());
//     zdepths.insert(e02, ZDepth::default());
//     layouts.insert(
//         e02,
//         Layout {
//             left: 600.0,
//             top: 0.0,
//             width: 300.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     idtree.insert_child(e02, e0, 3);

//     let e000 = world.create_entity::<Node>();
//     let e001 = world.create_entity::<Node>();
//     let e002 = world.create_entity::<Node>();
//     idtree.create(e000);
//     layouts.insert(
//         e000,
//         Layout {
//             left: 0.0,
//             top: 0.0,
//             width: 100.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     transforms.insert(e000, Transform::default());
//     zdepths.insert(e000, ZDepth::default());
//     idtree.insert_child(e000, e00, 1);

//     idtree.create(e001);
//     transforms.insert(e001, Transform::default());
//     zdepths.insert(e001, ZDepth::default());
//     layouts.insert(
//         e001,
//         Layout {
//             left: 100.0,
//             top: 0.0,
//             width: 100.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     idtree.insert_child(e001, e00, 2);

//     idtree.create(e002);
//     transforms.insert(e002, Transform::default());
//     zdepths.insert(e002, ZDepth::default());
//     layouts.insert(
//         e002,
//         Layout {
//             left: 200.0,
//             top: 0.0,
//             width: 100.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     idtree.insert_child(e002, e00, 3);

//     let e010 = world.create_entity::<Node>();
//     let e011 = world.create_entity::<Node>();
//     let e012 = world.create_entity::<Node>();
//     idtree.create(e010);
//     layouts.insert(
//         e010,
//         Layout {
//             left: 0.0,
//             top: 0.0,
//             width: 100.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     transforms.insert(e010, Transform::default());
//     zdepths.insert(e010, ZDepth::default());
//     idtree.insert_child(e010, e01, 1);

//     idtree.create(e011);
//     transforms.insert(e011, Transform::default());
//     zdepths.insert(e011, ZDepth::default());
//     layouts.insert(
//         e011,
//         Layout {
//             left: 100.0,
//             top: 0.0,
//             width: 100.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     idtree.insert_child(e011, e01, 2);

//     idtree.create(e012);
//     transforms.insert(e012, Transform::default());
//     zdepths.insert(e012, ZDepth::default());
//     layouts.insert(
//         e012,
//         Layout {
//             left: 200.0,
//             top: 0.0,
//             width: 100.0,
//             height: 900.0,
//             border_left: 0.0,
//             border_top: 0.0,
//             border_right: 0.0,
//             border_bottom: 0.0,
//             padding_left: 0.0,
//             padding_top: 0.0,
//             padding_right: 0.0,
//             padding_bottom: 0.0,
//         },
//     );
//     idtree.insert_child(e012, e01, 3);

//     transforms.get_write(e0).unwrap().modify(|transform: &mut Transform| {
//         transform.funcs.push(TransformFunc::TranslateX(50.0));
//         true
//     });
//     world.run(&Atom::from("test_oct_sys"));
//     debug_println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
//         oct.get(e0).unwrap(),
//         oct.get(e00).unwrap(),
//         oct.get(e01).unwrap(),
//         oct.get(e02).unwrap(),
//         oct.get(e000).unwrap(),
//         oct.get(e001).unwrap(),
//         oct.get(e002).unwrap(),
//         oct.get(e010).unwrap(),
//         oct.get(e011).unwrap(),
//         oct.get(e012).unwrap(),
//     );
// }

// #[cfg(test)]
// fn new_world() -> World {
//     let mut world = World::default();

//     world.register_entity::<Node>();
//     world.register_multi::<Node, Layout>();
//     world.register_multi::<Node, Transform>();
//     world.register_multi::<Node, ZDepth>();
//     world.register_multi::<Node, WorldMatrix>();
//     world.register_single::<IdTree>(IdTree::default());
//     world.register_single::<Oct>(Oct::new());

//     let system = CellContentBoxSys::new(ContentBoxSys::default());
//     world.register_system(Atom::from("oct_system"), system);
//     let system = CellWorldMatrixSys::new(WorldMatrixSys::default());
//     world.register_system(Atom::from("world_matrix_system"), system);

//     let mut dispatch = SeqDispatcher::default();
//     dispatch.build("oct_system, world_matrix_system".to_string(), &world);

//     world.add_dispatcher(Atom::from("test_oct_sys"), dispatch);
//     world
// }
