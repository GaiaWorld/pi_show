/**
 * 渲染对象的通用属性设置， 如alpha， hsv， visible， viewMatrix， projectMatrix
 */
use bevy_ecs::prelude::{EventWriter, Query, Res, Entity, ResMut, In};
use share::Share;

use atom::Atom;
use hal_core::*;
use res::{ResMap};

use crate::component::calc::*;
use crate::component::calc::Opacity;
use crate::render::engine::{UnsafeMut};
use crate::single::*;
use crate::single::DirtyViewRect;
use crate::single::oct::Oct;
use crate::system::util::*;
use crate::single::IdTree;
use crate::util::event::{EntityEvent, EventType, ImMessenger, RenderObjEvent, ResModifyEvent};

lazy_static! {
    static ref Z_DEPTH: Atom = Atom::from("zDepth");
    static ref HSV_MACRO: Atom = Atom::from("HSV");
    static ref HSV_ATTR: Atom = Atom::from("hsvValue");
}

// pub struct NodeAttrSys<C: HalContext + 'static> {
//     view_matrix_ubo: Option<Share<dyn UniformBuffer>>,
//     project_matrix_ubo: Option<Share<dyn UniformBuffer>>,
//     transform_will_change_matrix_dirtys: Vec<usize>,
//     hsv_ubo_map: UnsafeMut<ResMap<HsvUbo>>,
//     marker: PhantomData<C>,
// }

// impl<C: HalContext + 'static> NodeAttrSys<C> {
//     pub fn new(res_mgr: &ResMgr) -> Self {
//         NodeAttrSys {
//             view_matrix_ubo: None,
//             project_matrix_ubo: None,
//             transform_will_change_matrix_dirtys: Vec::default(),
//             hsv_ubo_map: UnsafeMut::new(res_mgr.fetch_map::<HsvUbo>(0).unwrap()),
//             marker: PhantomData,
//         }
//     }
// }

/// 处理视图矩阵的变化（视图矩阵目前应该不会改变，因此此方法目前只在首次初始化视图矩阵时调用）
pub fn handle_view<C: HalContext + 'static>(
	e: In<ResModifyEvent<ViewMatrix>>,
	view_matrix: Res<ViewMatrix>,
	mut view_matrix_ubo: ResMut<Share<ViewMatrixUbo>>,
	render_objs: ResMut<RenderObjs>,
){
	let slice: &[f32] = view_matrix.0.as_slice();
	let ubo = Share::new(ViewMatrixUbo::new(UniformValue::MatrixV4(Vec::from(slice))));
	for r in render_objs.iter() {
		r.1.paramter
			.set_value("viewMatrix", ubo.clone());
	}
	if render_objs.len() > 0 {
		// TODO
		// renderobjs_event_writer.send(EntityEvent::new_modify(1, "ubo", 0));
	}
	*view_matrix_ubo = ubo;
}

/// 处理投影矩阵的变化，递归设置所有渲染对象的投影矩阵
pub fn handle_project<C: HalContext + 'static>(
	e: In<ResModifyEvent<ProjectionMatrix>>,
	projection_matrix: Res<ProjectionMatrix>,
	mut render_objs: ResMut<RenderObjs>,
	mut project_matrix_ubo: ResMut<Share<ProjectMatrixUbo>>,
){
	let slice: &[f32] = projection_matrix.0.as_slice();
	let ubo = Share::new(ProjectMatrixUbo::new(UniformValue::MatrixV4(Vec::from(slice))));
	for r in render_objs.iter_mut() {
		r.1.paramter
			.set_value("projectMatrix", ubo.clone());
	}
	if render_objs.len() > 0 {
		// TODO
		// renderobjs_event_writer.send(EntityEvent::new_modify(1, "ubo", 0));
	}
	*project_matrix_ubo = ubo;
}

/// 处理脏willchange的改变，重新设置视图矩阵
/// * 如果willchange变为Some，重新计算视图矩阵
/// * 如果willchange变为None，重新设置视图矩阵为默认视图矩阵
pub fn handle_will_change<C: HalContext + 'static>(
	e: In<EntityEvent<TransformWillChangeMatrix>>,
	willchange_matrixs: Query<Option<&TransformWillChangeMatrix>>,
	view_matrix: Res<ViewMatrix>,
	view_matrix_ubo: Res<Share<ViewMatrixUbo>>,
	node_render_map: Res<NodeRenderMap>,
	mut render_objs: ResMut<RenderObjs>,
	render_begin: Res<RenderBegin>,
	mut dirty_view_rect: ResMut<DirtyViewRect>,
	idtree: Res<IdTree>,
) {
	let ev = &e.0;
	let mut modify = false;
	let mut is_willchange = false;
	let view_matrix_ubo: Share<dyn UniformBuffer> = view_matrix_ubo.clone();
	let id = &ev.id;
	is_willchange = true;
	// if !nodes.is_exist(*id) { // TODO
	// 	continue;
	// }
	let willchange_matrix = match willchange_matrixs.get(id.clone()) {
		Ok(r) => r,
		Err(_r) => return,
	};
	match willchange_matrix {
		Some(transform_will_change_matrix) => {
			let m = &view_matrix.0 * &transform_will_change_matrix.0;
			let slice: &[f32] = m.0.as_slice();
			let view_matrix_ubo: Share<dyn UniformBuffer> = Share::new(ViewMatrixUbo::new(
				UniformValue::MatrixV4(Vec::from(slice)),
			));
			recursive_set_view_matrix(
				*id,
				&mut modify,
				&willchange_matrixs,
				&idtree,
				&view_matrix_ubo,
				&node_render_map,
				&mut render_objs,
			);
		}
		None => recursive_set_view_matrix(
			*id,
			&mut modify,
			&willchange_matrixs,
			&idtree,
			&view_matrix_ubo,
			&node_render_map,
			&mut render_objs,
		),
	}

	// 如果存在willchange改变，设置脏区域为最大渲染矩形
	// willchange存在时，包围盒的值不正确，不能正确计算脏区域的大小，因此直接暴力地设置脏区域为最大渲染矩形
	if is_willchange {
		set_max_view(&render_begin, &mut dirty_view_rect);
	}

	if modify {
		// TODO
		// renderobjs_event_writer.send(EntityEvent::Modify(0, "ubo", 0));
	}
}

/// 处理脏视图，遍历左右脏节点，计算脏区域
pub fn handle_dirty_view<C: HalContext + 'static>(
	idtree: Res<IdTree>,
	dirty_list: Res<DirtyList>,
	octree: Res<Oct>,
	render_begin: Res<RenderBegin>,
	mut dirty_view_rect: ResMut<DirtyViewRect>,
) {
	// 处理包围盒的脏，确定脏区域最大范围（任何改变都可能导致脏区域改变， 因此遍历所有脏，确定脏区域）
	for id in dirty_list.0.iter() {
		// dirty_view_rect已经是最大范围了，不需要再修改
		if dirty_view_rect.4 == true {
			break;
		}
		let node = match idtree.get(id.id() as usize) {
			Some(r) => r,
			None => continue,
		};
		if node.layer() == 0 {
			continue;
		}
		
		modify_dirty_view(id.id() as usize, &octree, &render_begin, &mut dirty_view_rect);
	}
}

/// 处理实体创建时间，创建对应的node_render_map
pub fn handle_entity_create<C: HalContext + 'static>(
	e: In<EntityEvent<Entity>>,
	mut node_render_map: ResMut<NodeRenderMap>
){
	let e = &e.0;
	node_render_map.create(e.id.id() as usize);
}

/// 处理渲染对象的创建和删除，进而创建对应的实体与渲染对象的索引
pub fn handle_renderobjs<C: HalContext + 'static>(
	e: In<RenderObjEvent>,
	query: Query<(
		&Opacity,
		&Visibility,
		&HSV,
		&ZDepth,
		&Culling)>,
	mut render_objs: ResMut<RenderObjs>,
	mut node_render_map: ResMut<NodeRenderMap>,
	view_matrix_ubo: Res<Share<ViewMatrixUbo>>,
	project_matrix_ubo: Res<Share<ProjectMatrixUbo>>,
	mut hsv_asset_map: ResMut<UnsafeMut<ResMap<HsvUbo>>>,
) {
	let e = &e.0;
	match e.ty {
		EventType::Create => {
			let render_obj = &mut render_objs[e.id];
			// let notify = unsafe { &*(node_render_map.get_notify_ref() as * const NotifyImpl) };
			node_render_map.add(render_obj.context.id() as usize, e.id);

			let paramter = &mut render_obj.paramter;

			paramter.set_value("viewMatrix", view_matrix_ubo.clone()); // VIEW_MATRIX
			paramter.set_value("projectMatrix", project_matrix_ubo.clone()); // PROJECT_MATRIX

			let (opacity, visibility, hsv, z_depth, culling) = match query.get(render_obj.context) {
				Ok(r) => r,
				Err(_) => return,
			};

			let z_depth = z_depth.0;
			let opacity = opacity.0;
			paramter.set_single_uniform("alpha", UniformValue::Float1(opacity)); // alpha
			debug_println!("id: {}, alpha: {:?}", render_obj.context, opacity);

			let visibility = visibility.0;
			let culling = culling.0;
			render_obj.visibility = visibility & !culling;

			render_obj.depth = z_depth + render_obj.depth_diff;

			if !(hsv.h == 0.0 && hsv.s == 0.0 && hsv.v == 0.0) {
				render_obj.fs_defines.add("HSV");
				paramter.set_value("hsvValue", create_hsv_ubo(hsv, &mut hsv_asset_map)); // hsv
			}
		},
		EventType::Delete => {
			let render_obj = &render_objs[e.id];
			node_render_map.remove(render_obj.context.id() as usize, e.id);
		},
		_ => (),
	}
}

/// 处理深度的变化, 设置渲染对象的深度值
pub fn handle_zdepth<C: HalContext + 'static>(
	e: In<EntityEvent<ZDepth>>,
	query: Query<&ZDepth>,
	mut render_objs: ResMut<RenderObjs>,
	node_render_map: Res<NodeRenderMap>,
	mut renderobjs_event_writer: ImMessenger<RenderObjEvent>,
) {
	let e = &e.0;
	match e.ty {
		EventType::Modify => {
			match node_render_map.get(e.id.id() as usize) {
				Some(obj_ids) => {
					let z_depth = match query.get(e.id){
						Ok(r) => r.0,
						Err(_) => return,
					};
		
					if obj_ids.len() > 0 {
						for id in obj_ids.iter() {
							let render_obj = &mut render_objs[*id];
							render_obj.depth = z_depth + render_obj.depth_diff;
							renderobjs_event_writer.send(RenderObjEvent::new_modify(*id, "depth", 0));
						}
					}
				},
				None => (),
			};
		},
		_ => (),
	}
}

/// 处理透明值的改变， 设置对应的ubo
pub fn handle_opacity<C: HalContext + 'static>(
	e: In<EntityEvent<Opacity>>,
	query: Query<&Opacity>,
	mut render_objs: ResMut<RenderObjs>,
	node_render_map: Res<NodeRenderMap>,
	mut renderobjs_event_writer: ImMessenger<RenderObjEvent>,
) {
	let e = &e.0;
	match e.ty {
		EventType::Create | EventType::Modify => {
			let opacity = query.get(e.id).unwrap().0;
			let obj_ids = &node_render_map[e.id.id() as usize];

			for id in obj_ids.iter() {
				let render_obj = &mut render_objs[*id];
				render_obj
					.paramter
					.as_ref()
					.set_single_uniform("alpha", UniformValue::Float1(opacity));
				renderobjs_event_writer.send(RenderObjEvent::new_modify(*id, "paramter", 0))
			}
		},
		_ => (),
	}
}

/// 处理可见性的改变
pub fn handle_visibility<C: HalContext + 'static>(
	e: In<EntityEvent<Visibility>>,
	query: Query<(&Visibility, &Culling)>,
	oct: Res<Oct>,
	render_begin: Res<RenderBegin>,
	mut render_objs: ResMut<RenderObjs>,
	node_render_map: Res<NodeRenderMap>,
	mut dirty_view_rect: ResMut<DirtyViewRect>,
	mut renderobjs_event_writer: EventWriter<RenderObjEvent>,
) {
	let e = &e.0;
	match e.ty {
		EventType::Create | EventType::Modify => {
			modify_visible(e.id, &query, &mut render_objs, &node_render_map, &mut renderobjs_event_writer);
			// dirty_view_rect已经是最大范围了，不需要再修改
			if dirty_view_rect.4 == true {
				return;
			}

			modify_dirty_view(e.id.id() as usize, &oct, &render_begin, &mut dirty_view_rect);
		},
		_ => (),
	}
}
/// 处理裁剪性的改变
pub fn handle_culling<C: HalContext + 'static>(
	e: In<EntityEvent<Culling>>,
	query: Query<(&Visibility, &Culling)>,
	mut render_objs: ResMut<RenderObjs>,
	node_render_map: Res<NodeRenderMap>,
	mut renderobjs_event_writer: EventWriter<RenderObjEvent>,
) {
	let e = &e.0;
	match e.ty {
		EventType::Modify => {
			modify_visible(e.id, &query, &mut render_objs, &node_render_map, &mut renderobjs_event_writer);
		},
		_ => (),
	}
}

/// 处理hsv的改变，设置ubo
pub fn handle_hsv<C: HalContext + 'static>(
	e: In<EntityEvent<HSV>>,
	query: Query<&HSV>,
	mut render_objs: ResMut<RenderObjs>,
	node_render_map: Res<NodeRenderMap>,
	mut renderobjs_event_writer: EventWriter<RenderObjEvent>,
	mut hsv_asset_map: ResMut<UnsafeMut<ResMap<HsvUbo>>>,
) {
	let e = &e.0;
	match e.ty {
		EventType::Create | EventType::Modify => {
			let hsv = query.get(e.id).unwrap();
			modifyHsv(e.id.id() as usize, hsv, &mut hsv_asset_map, &mut render_objs, &node_render_map, &mut renderobjs_event_writer);
		},
		_ => (),
	}
}

/// 处理oct的改变，设置脏区域
pub fn handle_oct<C: HalContext + 'static>(
	e: In<EntityEvent<Oct>>,
	oct: Res<Oct>,
	render_begin: Res<RenderBegin>,
	mut dirty_view_rect: ResMut<DirtyViewRect>,
) {
	let e = &e.0;
	if dirty_view_rect.4 == true {
		return;
	}
	modify_dirty_view(e.id.id() as usize, &oct, &render_begin, &mut dirty_view_rect);
}

fn modify_visible(
	id: Entity, 
	query: &Query<(&Visibility, &Culling)>,
	render_objs: &mut ResMut<RenderObjs>,
	node_render_map: &Res<NodeRenderMap>,
	renderobjs_event_writer: &mut EventWriter<RenderObjEvent>,) {
	let (visibility, culling) = query.get(id).unwrap();
    let visibility = visibility.0;
    let culling = culling.0;
    let obj_ids = &node_render_map[id.id() as usize];

    for id in obj_ids.iter() {
		render_objs[*id].visibility = visibility & !culling;
		renderobjs_event_writer.send(RenderObjEvent::new_modify(*id, "", 0))
    }
}

fn modify_dirty_view(
	id: usize,
	octree: &Res<Oct>,
	render_begin: &Res<RenderBegin>,
	dirty_view_rect: &mut ResMut<DirtyViewRect>
) {
	let oct = match octree.get(id){
		Some(r) => r,
		None => return,
	};
	let oct = &oct.0;
	let viewport = render_begin.0.viewport;
	// println!("true2======================dirty_view_rect: {:?}", **dirty_view_rect);
	// 与包围盒求并
	dirty_view_rect.0 = dirty_view_rect.0.min(oct.mins.x.max(0.0));
	dirty_view_rect.1 = dirty_view_rect.1.min(oct.mins.y.max(0.0));
	dirty_view_rect.2 = dirty_view_rect.2.max(oct.maxs.x.min(viewport.2 as f32));
	dirty_view_rect.3 = dirty_view_rect.3.max(oct.maxs.y.min(viewport.3 as f32));
	// println!("true3======================dirty_view_rect: {:?}, oct: {:?}", **dirty_view_rect, oct);

	// 如果与视口一样大，则设置dirty_view_rect.4为true, 后面的包围盒改变，将不再重新计算dirty_view_rect
	// 由于包围盒改变事件通常是从父到子的顺序传递，因此如果界面有大范围的改变，能够很快的将dirty_view_rect.4设置为true
	// 因此在大范围改变时，具有较好的优化
	// 另外，dirty_view_rect.4被设计的另一个原因是，外部很多时候能够预计即将改变的界面将是大范围，可以提前设置该值，来优化掉后面的计算（尽管这种计算并不很费）
	if dirty_view_rect.0 == 0.0 && 
	dirty_view_rect.1 == 0.0 && 
	dirty_view_rect.2 == viewport.2 as f32 && 
	dirty_view_rect.3 == viewport.3  as f32 {

		// println!("true1======================oct: {:?}, dirty_view_rect:{:?}", oct, **dirty_view_rect);
		dirty_view_rect.4 = true;
	}
}

// 设置为最大视口
fn set_max_view(
	render_begin: &Res<RenderBegin>,
	dirty_view_rect: &mut ResMut<DirtyViewRect>
) {
	let viewport = render_begin.0.viewport;
	// 如果与视口一样大，则设置dirty_view_rect.4为true, 后面的包围盒改变，将不再重新计算dirty_view_rect
	// 由于包围盒改变事件通常是从父到子的顺序传递，因此如果界面有大范围的改变，能够很快的将dirty_view_rect.4设置为true
	// 因此在大范围改变时，具有较好的优化
	// 另外，dirty_view_rect.4被设计的另一个原因是，外部很多时候能够预计即将改变的界面将是大范围，可以提前设置该值，来优化掉后面的计算（尽管这种计算并不很费）
	if dirty_view_rect.4 != true {
		
		// 与包围盒求并
		dirty_view_rect.0 = 0.0;
		dirty_view_rect.1 = 0.0;
		dirty_view_rect.2 = viewport.2 as f32;
		dirty_view_rect.3 = viewport.3 as f32;

		dirty_view_rect.4 = true;
	}
}

fn recursive_set_view_matrix(
    entity: Entity,
    modify: &mut bool,
    transform_will_change_matrixs: &Query<Option<&TransformWillChangeMatrix>>,
    idtree: &IdTree,
    ubo: &Share<dyn UniformBuffer>,
    node_render_map: &Res<NodeRenderMap>,
    render_objs: &mut ResMut<RenderObjs>,
) {
    let obj_ids = match node_render_map.get(entity.id() as usize) {
		Some(r) => r,
		None => return,
	};
    for id in obj_ids.iter() {
        let render_obj = &mut render_objs[*id];
        render_obj.paramter.set_value("viewMatrix", ubo.clone());
        *modify = true;
    }

    let first = idtree[entity.id() as usize].children().head;
    for (child_id, child) in idtree.iter(first) {
        if let Ok(Some(_)) = transform_will_change_matrixs.get(to_entity(child_id, child.data)) {
            continue;
		}
        recursive_set_view_matrix(
            to_entity(child_id, child.data),
            modify,
            transform_will_change_matrixs,
            idtree,
            ubo,
            node_render_map,
            render_objs,
        );
    }
}

pub fn create_hsv_ubo(hsv: &HSV, hsv_asset_map: &mut UnsafeMut<ResMap<HsvUbo>>) -> Share<dyn UniformBuffer> {
	let h = f32_3_hash(hsv.h, hsv.s, hsv.v);
	match hsv_asset_map.get(&h) {
		Some(r) => r,
		None => hsv_asset_map.create(
			h,
			HsvUbo::new(UniformValue::Float3(hsv.h, hsv.s, hsv.v)),
			0,
			0,
		), // TODO cost
	}
}

pub fn modifyHsv<'a>(id: usize, hsv: &HSV,
	hsv_asset_map: &mut UnsafeMut<ResMap<HsvUbo>>,
	render_objs: &'a mut ResMut<RenderObjs>,
	node_render_map: &'a Res<NodeRenderMap>,
	render_obj_event_writer: &'a mut EventWriter<RenderObjEvent>,
) {
	// let hsv = &hsvs[id];
	let obj_ids = match node_render_map.get(id) {
		Some(r) => r,
		None => return,
	};
	if !(hsv.h == 0.0 && hsv.s == 0.0 && hsv.v == 0.0) {
		for id in obj_ids.iter() {
			// let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
			let render_obj = &mut render_objs[*id];
			render_obj.fs_defines.add("HSV");
			render_obj
				.paramter
				.set_value("hsvValue", create_hsv_ubo(hsv, hsv_asset_map)); // hsv
			
			render_obj_event_writer.send(RenderObjEvent::new_modify(*id, "paramter", 0));
			render_obj_event_writer.send(RenderObjEvent::new_modify(*id, "program_dirty", 0));
		}
	} else {
		for id in obj_ids.iter() {
			let render_obj = &mut render_objs[*id];
			render_obj.fs_defines.remove("HSV");
			
			render_obj_event_writer.send(RenderObjEvent::new_modify(*id, "paramter", 0));
			render_obj_event_writer.send(RenderObjEvent::new_modify(*id, "program_dirty", 0));
		}
	}
}

#[derive(Deref, DerefMut)]
pub struct InnerMutRes<T>(pub T);

// impl_system! {
//     NodeAttrSys<C> where [C: HalContext + 'static],
//     true,
//     {
//         EntityListener<Node, CreateEvent>
//         SingleCaseListener<RenderObjs, CreateEvent>
// 		SingleCaseListener<RenderObjs, DeleteEvent>
// 		SingleCaseListener<ProjectionMatrix, ModifyEvent>
// 		SingleCaseListener<Oct, DeleteEvent>
// 		SingleCaseListener<Oct, CreateEvent>
// 		SingleCaseListener<Oct, ModifyEvent>
// 		MultiCaseListener<Node, Opacity, ModifyEvent>
// 		MultiCaseListener<Node, Visibility, ModifyEvent>
// 		MultiCaseListener<Node, Visibility, CreateEvent>
//         MultiCaseListener<Node, Culling, ModifyEvent>
// 		MultiCaseListener<Node, HSV, ModifyEvent>
// 		MultiCaseListener<Node, HSV, CreateEvent>
//         MultiCaseListener<Node, ZDepth, ModifyEvent>
//         MultiCaseListener<Node, TransformWillChangeMatrix, CreateEvent>
//         MultiCaseListener<Node, TransformWillChangeMatrix, ModifyEvent>
//         MultiCaseListener<Node, TransformWillChangeMatrix, DeleteEvent>
//     }
// }
