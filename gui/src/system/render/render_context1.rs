// /**
// * 处理渲染上下文
// */
// use std::marker::PhantomData;

// use share::Share;
// use spatialtree::OctTree;
// use std::hash::{Hash, Hasher};

// // use ordered_float::NotNan;
// use hash::{DefaultHasher, XHashSet};

// use atom::Atom;
// use ecs::{CreateEvent, DeleteEvent, ModifyEvent, MultiCaseImpl, MultiCaseListener, Runner, SingleCaseImpl, SingleCaseListener};
// use ecs::monitor::NotifyImpl;
// use hal_core::*;
// use map::vecmap::VecMap;
// use map::Map;
// use polygon::*;
// use idtree::Node as IdtreeNode;

// use crate::component::calc::{Opacity, LayoutR, MaskTexture};
// use crate::component::calc::*;
// use crate::component::user::*;
// use crate::entity::Node;
// use crate::render::engine::{AttributeDecs, Engine, ShareEngine};
// use crate::render::res::*;
// use crate::{layout, single::*};
// use crate::single::dyn_texture::DynAtlasSet;
// use crate::single::{oct::Oct, DirtyViewRect};
// use crate::system::render::shaders::image::{ MASK_IMAGE_VS_SHADER_NAME, MASK_IMAGE_FS_SHADER_NAME};
// use crate::system::util::constant::*;
// use crate::system::util::{*, let_top_offset_matrix as let_top_offset_matrix1};
// use crate::Z_MAX;

// lazy_static! {
// 	static ref UV: Atom = Atom::from("UV");
// 	static ref POSITION: Atom = Atom::from("Position");
// 	static ref INDEX: Atom = Atom::from("Index");
// }

// const DIRTY_TY: usize = StyleType::Matrix as usize
// 	// | StyleType::Opacity as usize
// 	| StyleType::Layout as usize;

// const DIRTY_TY1: usize = StyleType1::MaskTexture as usize
// 	| StyleType1::MaskImageClip as usize;

// pub struct RenderContextSys<C> {
// 	list: XHashSet<usize>,
// 	dirty: XHashSet<usize>,
// 	render_map: VecMap<usize>,
// 	default_sampler: Share<SamplerRes>,
// 	unit_geo: Share<GeometryRes>, // 含uv， index， pos
// 	default_paramter: FboParamter,
// 	marker: PhantomData<C>,
// }

// // 将顶点数据改变的渲染对象重新设置索引流和顶点流
// impl<'a, C: HalContext + 'static> Runner<'a> for RenderContextSys<C> {
// 	type ReadData = (
// 		&'a MultiCaseImpl<Node, LayoutR>,
// 		&'a MultiCaseImpl<Node, ZDepth>,
// 		&'a MultiCaseImpl<Node, MaskImage>,
// 		&'a MultiCaseImpl<Node, MaskTexture>,
// 		&'a MultiCaseImpl<Node, MaskImageClip>,
// 		&'a MultiCaseImpl<Node, WorldMatrix>,
// 		&'a MultiCaseImpl<Node, Transform>,
// 		// &'a MultiCaseImpl<Node, Opacity>,
// 		&'a MultiCaseImpl<Node, StyleMark>,
// 		&'a SingleCaseImpl<Oct>,
// 		&'a SingleCaseImpl<DefaultState>,
// 		&'a SingleCaseImpl<PremultiState>,
// 		&'a SingleCaseImpl<IdTree>,
// 	);
// 	type WriteData = (
// 		&'a mut MultiCaseImpl<Node, RenderContext>,
// 		&'a mut SingleCaseImpl<RenderObjs>,
// 		&'a mut SingleCaseImpl<ShareEngine<C>>,
// 		&'a mut SingleCaseImpl<DynAtlasSet>,
// 		&'a mut SingleCaseImpl<RootIndexs>,
// 	);
// 	fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
// 		let (
// 			layouts,
// 			z_depths,
// 			mask_images,
// 			mask_textures,
// 			mask_image_clips,
// 			world_matrixs,
// 			transforms,
// 			// opacitys,
// 			style_marks,
// 			octs,
// 			default_state,
// 			premulti_state,
// 			idtree,
// 		) = read;
// 		if self.list.len() == 0 {
// 			return;
// 		}

// 		let (
// 			render_contexts, 
// 			render_objs, 
// 			engine,
// 			dyn_atlas_set,
// 			root_indexs) = write;
// 		let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
		
// 		let dirty = std::mem::replace(&mut self.dirty, XHashSet::default());
// 		for id in dirty.iter() {
// 			let style_mark = match style_marks.get(*id) { // 节点已经销毁，不做处理
// 				Some(r) => r,
// 				None => continue,
// 			};

// 			let mut dirty = style_mark.dirty;

// 			let node = &idtree[*id];
// 			let render_context = render_contexts[*id];
// 			let render_obj = &mut render_objs[render_context.render_obj_index];
// 			let z_depth = z_depths[*id].0;
// 			let layout = &layouts[*id];
// 			let transform = &transforms[*id];
// 			let world_matrix = &world_matrixs[*id];

// 			// 矩阵或布局发生改变， 需要更新fbo纹理，和uv（TODO: 实际应该计算所有子节点的最大范围）
// 			let aabb = octs.get(*id).unwrap().0;
// 			let target_index = dyn_atlas_set.update_or_add_rect(render_context.render_target, aabb.maxs.x-aabb.mins.x,aabb.maxs.y-aabb.mins.y, &mut engine.gl);

// 			// 插入或更新成功（aabb发生了改变）
// 			if target_index != 0 {
// 				render_context.render_target = target_index;
// 				// 绑定纹理
// 				render_obj.paramter.set_texture(
// 					"texture",
// 					(&engine
// 						.gl
// 						.rt_get_color_texture(dyn_atlas_set.get_target(target_index).unwrap(), 0).unwrap(), &self.default_sampler),
// 				);

// 				let rect = dyn_atlas_set.get_rect(target_index).unwrap();
// 				let uv = dyn_atlas_set.get_uv(target_index).unwrap();
// 				log::info!("rect========={:?}, {:?}", rect, uv);
// 				render_context.render_rect = rect;
// 				update_geo_quad(render_obj, image_clip, engine, &self.unit_geo, &uv);
// 			}
			
			

// 			// 目的： 将其下渲染的字节，以本节点左上为原点对齐
// 			let left_top_matrix = let_top_offset_matrix(
// 				layout, world_matrix,transform
// 			);
// 			render_context.view_matrix = left_top_matrix;
// 			render_context.projection_matrix = ProjectionMatrix::new(
// 				rect.maxs.x - rect.mins.x,
// 				rect.maxs.y - rect.mins.y,
// 				-Z_MAX - 1.0,
// 				Z_MAX + 1.0,
// 			);
// 			let mut buffer = Vec::from(render_context.view_matrix.0.as_slice());
// 			render_context.view_matrix_ubo = Share::new(ViewMatrixUbo::new(
// 				UniformValue::MatrixV4(buffer),
// 			));
// 			buffer = Vec::from(render_context.projection_matrix.0.as_slice());
// 			render_context.projection_matrix_ubo = Share::new(ViewMatrixUbo::new(
// 				UniformValue::MatrixV4(buffer),
// 			));

// 			if dirty & StyleType::Matrix as usize != 0 {
// 				modify_matrix(
// 					render_obj,
// 					layout,
// 					z_depth,
// 					world_matrix,
// 					transform,
// 				);
// 			}

			

// 			// // src修改， 修改texture
// 			// if dirty1 & StyleType1::MaskTexture as usize != 0 {
// 			// 	render_obj.paramter.set_texture(
// 			// 		"maskTexture",
// 			// 		(&mask_texture.src.as_ref().unwrap().bind, &self.default_sampler),
// 			// 	);
// 			// }

// 			// notify.modify_event(render_context.render_obj_index, "geometry", 0);
// 			// notify.modify_event(render_context.render_obj_index, "ubo", 0);
// 			// dirty &= !(StyleType::Matrix as usize); // 已经计算了世界矩阵， 设置世界矩阵不脏

// 			// // 世界矩阵脏， 设置世界矩阵ubo
// 			// if dirty & StyleType::Matrix as usize != 0 {
// 			// 	modify_matrix(
// 			// 		render_obj,
// 			// 		layout,
// 			// 		z_depth,
// 			// 		world_matrix,
// 			// 		transform,
// 			// 	);
// 			// 	notify.modify_event(render_context.render_obj_index, "ubo", 0);
// 			// }

// 			// notify.modify_event(render_context.render_obj_index, "", 0);
// 		}

// 		std::mem::replace(&mut self.list, list);
// 	}
// }

// impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, RenderContext, DeleteEvent> for RenderContextSys<C> {
// 	type ReadData = &'a SingleCaseImpl<IdTree>;
// 	type WriteData = &'a mut SingleCaseImpl<RootIndexs>;
// 	fn listen(&mut self, event: &DeleteEvent, idtree: Self::ReadData, root_indexs: Self::WriteData) {
// 		self.dirty.remove(&event.id);
// 		root_indexs.delete(event.id, idtree[event.id].layer());
// 	}
// }

// // RenderContext创建时，需要创建对应的渲染对象。
// impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, RenderContext, CreateEvent> for RenderContextSys<C> {
// 	type ReadData = (&'a SingleCaseImpl<IdTree>, &'a SingleCaseImpl<PremultiState>,);
// 	type WriteData = (
// 		&'a mut SingleCaseImpl<RootIndexs>, 
// 		&'a mut SingleCaseImpl<RenderObjs>,
// 		&'a mut MultiCaseImpl<Node, RenderContext>,
// 		&'a mut MultiCaseImpl<Node, Root>);
// 	fn listen(&mut self, event: &CreateEvent, (idtree, premulti_state): Self::ReadData, (root_indexs, render_objs, render_contexts, roots): Self::WriteData) {
// 		let id = event.id;
// 		root_indexs.mark(id, idtree[id].layer());
// 		root_indexs.1 = true;
// 		self.dirty.insert(id);

// 		let (state, vs, fs) = (&***premulti_state, MASK_IMAGE_VS_SHADER_NAME.clone(), MASK_IMAGE_FS_SHADER_NAME.clone());
// 		render_contexts[id].render_obj_index = self.create_render_obj(id, render_objs, state, vs, fs);

// 		set_root(event.id, roots, render_contexts, idtree);
// 	}
// }

// // IdTree创建时，设置实体的根。
// impl<'a, C: HalContext + 'static> SingleCaseListener<'a, IdTree, CreateEvent> for RenderContextSys<C> {
// 	type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, RenderContext>) ;
// 	type WriteData = &'a mut MultiCaseImpl<Node, Root>;
// 	fn listen(&mut self, event: &CreateEvent, (idtree, render_contexts): Self::ReadData, roots: Self::WriteData) {
// 		let id = event.id;
// 		set_root(event.id, roots, render_contexts, idtree);
// 	}
// }

// // 包围盒发生改变， 修改根的
// impl<'a, C: HalContext + 'static> SingleCaseListener<'a, Oct, ModifyEvent> for RenderContextSys<C> {
// 	type ReadData = &'a SingleCaseImpl<Oct> ;
// 	type WriteData = (&'a mut MultiCaseImpl<Node, Root>,  &'a mut MultiCaseImpl<Node, RenderContext>);
// 	fn listen(&mut self, event: &ModifyEvent, octs: Self::ReadData, (roots, render_contexts): Self::WriteData) {
// 		let id = event.id;
// 		let root = roots[id];
// 		if root.0 != 1 {
// 			self.dirty.insert(root.0);
// 		}
		

		
// 		// let root = roots[id];


// 		// // 计算包围盒
// 		// if let Some(render_context) = &mut render_contexts.get(id) {
// 		// 	let content_box = &mut render_context.content_box;
// 		// 	let oct = octs.get(id).unwrap().0;
// 		// 	content_box.mins.x = content_box.mins.x.min(oct.mins.x);
// 		// 	content_box.mins.y = content_box.mins.y.min(oct.mins.y);
// 		// 	content_box.maxs.x = content_box.maxs.x .max(oct.maxs.x);
// 		// 	content_box.maxs.y = content_box.maxs.y.max(oct.maxs.y);
// 		// }
// 	}
// }

// // /// 计算渲染上下文的包围盒
// // /// 迭代所有的子节点，求所有子节点包围盒的最大范围
// // fn calc_context_box(id: usize,octs: &Oct, roots: &mut MultiCaseImpl<Node, Root>, render_contexts: &mut MultiCaseImpl<Node, RenderContext>, bound_box: &mut Aabb2) {
// // 	let root = roots[id];
// // 	if root.0 != 1 {
// // 		self.dir
// // 		// // 计算包围盒
// // 		// if let Some(render_context) = &mut render_contexts.get(id) {
// // 		// 	let oct = octs.get(id).unwrap().0;
// // 		// 	bound_box.mins.x = bound_box.mins.x.min(oct.mins.x);
// // 		// 	bound_box.mins.y = bound_box.mins.y.min(oct.mins.y);
// // 		// 	bound_box.maxs.x = bound_box.maxs.x .max(oct.maxs.x);
// // 		// 	bound_box.maxs.y = bound_box.maxs.y.max(oct.maxs.y);
// // 		// }
// // 	}
// // }

// // 继承父的根
// fn set_root(
// 	id: usize, 
// 	roots: &mut MultiCaseImpl<Node, Root>, 
// 	render_contexts: &MultiCaseImpl<Node, RenderContext>,
// 	idtree: &IdTree,) {
// 	let node = idtree[id];
	
// 	let root = match render_contexts.get(id) {
// 		Some(r) => id,
// 		None => {
// 			let parent = node.parent();
// 			if parent > 0 {
// 				roots[parent].0
// 			} else {
// 				1
// 			}
// 		}
// 	};
// 	let this_root = roots[id].0;

// 	if root != this_root {
// 		roots.insert(id, Root(root));

// 		if node.children().head == 0 {
// 			return;
// 		}

// 		for (id, node) in idtree.recursive_iter(node.children().head) {
// 			if None = render_contexts.get(id) {
// 				roots.insert(id, Root(root));
// 			}
// 		}
// 	}
// }

// impl<C: HalContext + 'static> RenderContextSys<C> {
// 	pub fn with_capacity(engine: &mut Engine<C>, capacity: usize) -> Self {
// 		let mut sm = SamplerDesc::default();
// 		sm.u_wrap = TextureWrapMode::ClampToEdge;
// 		sm.v_wrap = TextureWrapMode::ClampToEdge;
// 		// sm.min_filter = TextureFilterMode::Nearest;
// 		// sm.mag_filter = TextureFilterMode::Nearest;

// 		let default_sampler = engine.create_sampler_res(sm);

// 		let positions = engine
// 			.buffer_res_map
// 			.get(&(POSITIONUNIT.get_hash() as u64))
// 			.unwrap();
// 		let indices = engine
// 			.buffer_res_map
// 			.get(&(INDEXUNIT.get_hash() as u64))
// 			.unwrap();

// 		let geo = engine.create_geometry();
// 		engine
// 			.gl
// 			.geometry_set_attribute(&geo, &AttributeName::Position, &positions, 2)
// 			.unwrap();
// 		engine
// 			.gl
// 			.geometry_set_attribute(&geo, &AttributeName::UV0, &positions, 2)
// 			.unwrap();
// 		engine
// 			.gl
// 			.geometry_set_indices_short(&geo, &indices)
// 			.unwrap();

// 		RenderContextSys {
// 			dirty: XHashSet::default(),
// 			render_map: VecMap::with_capacity(capacity),
// 			default_sampler: default_sampler,
// 			unit_geo: Share::new(GeometryRes {
// 				geo: geo,
// 				buffers: vec![indices, positions.clone(), positions],
// 			}),
// 			default_paramter: FboParamter::default(),
// 			marker: PhantomData,
// 		}
// 	}

// 	#[inline]
// 	fn unbind_context(&mut self, id: usize, render_ctxs: &mut MultiCaseImpl<Node, RenderContext>, render_objs: &mut SingleCaseImpl<RenderObjs>) {
		
// 		match render_ctxs.get_mut(id) {
// 			Some(ctx) => {
// 				ctx.index_count -= 1;
// 				if ctx.index_count == 0 {
// 					match render_ctxs.delete(id){
// 						Some(r) => {
// 							self.remove_render_obj(r.render_obj_index, render_objs);
// 						},
// 						None => (),
// 					};
// 					// 通知
// 					render_objs.get_notify_ref().modify_event(0, "", 0);
// 				}
// 			}
// 			None => (),
// 		};
// 	}

// 	#[inline]
// fn remove_render_obj(&mut self, id: usize, render_objs: &mut SingleCaseImpl<RenderObjs>) {
// 	match self.render_map.remove(id) {
// 		Some(index) => {
// 			let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
// 			render_objs.remove(index, Some(notify));
// 		}
// 		None => (),
// 	};
// }

// 	#[inline]
// 	fn create_render_obj(
// 		&mut self,
// 		id: usize,
// 		render_objs: &mut SingleCaseImpl<RenderObjs>,
// 		default_state: &CommonState,
// 		vs: Atom,
// 		fs: Atom,
// 	) -> usize {
// 		create_render_obj(
// 			id,
// 			-0.1,
// 			true,
// 			vs,
// 			fs,
// 			Share::new(self.default_paramter.clone()),
// 			default_state,
// 			render_objs,
// 			&mut self.render_map,
// 		)
// 	}
// }

// fn update_geo_quad<C: HalContext + 'static>(
// 	render_obj: &mut RenderObj,
// 	engine: &mut Engine<C>,
// 	unit_geo: &Share<GeometryRes>,
// 	uv: &Aabb2,
// ) {
// 	log::info!("update_geo_quad: {:?}", clip);
// 	let (uv1, uv2) =  (clip.mins, clip.maxs);
// 	let uv_hash = cal_uv_hash(&uv.mins, &uv.maxs);
// 	let uv1_hash = cal_uv_hash(&uv1, &uv2);
// 	let geo_hash = unit_geo_hash(&uv_hash, &uv1_hash);
// 	match engine.geometry_res_map.get(&geo_hash) {
// 		Some(r) => render_obj.geometry = Some(r),
// 		None => {
// 			let uv_buffer = create_uv_buffer(uv_hash, &uv.mins, &uv.maxs, engine);
// 			let uv1_buffer = create_uv_buffer(uv1_hash, &uv1, &uv2, engine);
// 			let geo = engine.create_geometry();
// 			engine
// 				.gl
// 				.geometry_set_attribute(
// 					&geo,
// 					&AttributeName::Position,
// 					&unit_geo.buffers[1],
// 					2,
// 				)
// 				.unwrap();
// 			engine
// 				.gl
// 				.geometry_set_attribute(&geo, &AttributeName::UV0, &uv_buffer, 2)
// 				.unwrap();
// 			engine
// 				.gl
// 				.geometry_set_attribute(&geo, &AttributeName::UV1, &uv1_buffer, 2)
// 				.unwrap();
// 			engine
// 				.gl
// 				.geometry_set_indices_short(&geo, &unit_geo.buffers[0])
// 				.unwrap();
// 			let geo_res = GeometryRes {
// 				geo: geo,
// 				buffers: vec![
// 					unit_geo.buffers[0].clone(),
// 					unit_geo.buffers[1].clone(),
// 					uv_buffer,
// 					uv1_buffer,
// 				],
// 			};
// 			render_obj.geometry =
// 				Some(engine.geometry_res_map.create(geo_hash, geo_res, 0, 0));
// 		}
// 	};
// }

// #[inline]
// fn cal_uv_hash(uv1: &Point2, uv2: &Point2) -> u64 {
// 	let mut hasher = DefaultHasher::default();
// 	UV.hash(&mut hasher);
// 	f32_4_hash_(uv1.x, uv1.y, uv2.x, uv2.y, &mut hasher);
// 	hasher.finish()
// }

// fn create_uv_buffer<C: HalContext + 'static>(
// 	uv_hash: u64,
// 	uv1: &Point2,
// 	uv2: &Point2,
// 	engine: &mut Engine<C>,
// ) -> Share<BufferRes> {
// 	log::info!("create_uv_buffer====={:?}, {:?}", uv1, uv2);
// 	match engine.buffer_res_map.get(&uv_hash) {
// 		Some(r) => r,
// 		None => {
// 			let uvs = [uv1.x, uv1.y, uv1.x, uv2.y, uv2.x, uv2.y, uv2.x, uv1.y];
// 			log::info!("uvxxxxx====={:?}", uvs);
// 			engine.create_buffer_res(
// 				uv_hash,
// 				BufferType::Attribute,
// 				8,
// 				Some(BufferData::Float(&uvs[..])),
// 				false,
// 			)
// 		}
// 	}
// }

// #[inline]
// fn unit_geo_hash(uv_hash: &u64, uv1_hash: &u64) -> u64 {
// 	let mut hasher = DefaultHasher::default();
// 	uv_hash.hash(&mut hasher);
// 	uv1_hash.hash(&mut hasher);
// 	POSITIONUNIT.hash(&mut hasher);
// 	INDEXUNIT.hash(&mut hasher);
// 	hasher.finish()
// }

// #[inline]
// pub fn let_top_offset_matrix(
// 	layout: &LayoutR,
// 	matrix: &WorldMatrix,
// 	transform: &Transform,
// ) -> WorldMatrix {
// 	let matrix = let_top_offset_matrix1(layout, matrix,transform, 0.0, 0.0, 0.0 );
// 	let offsetX = matrix.m14;
// 	let offsetY = matrix.m24;

// 	WorldMatrix(
// 		Matrix4::new_translation(&Vector3::new(-offsetX,
// 			-offsetY,
// 			0.0)),
// 		false,
// 	)
// }

// #[inline]
// fn modify_matrix(
// 	render_obj: &mut RenderObj,
// 	layout: &LayoutR,
// 	depth: f32,
// 	world_matrix: &WorldMatrix,
// 	transform: &Transform,
// ) {
// 	let arr = create_unit_offset_matrix(
// 		layout.rect.end - layout.rect.start,
// 		layout.rect.bottom - layout.rect.top,
// 		layout.border.start,
// 		layout.border.top,
// 		layout,
// 		world_matrix,
// 		transform,
// 		depth,
// 	);
// 	render_obj.paramter.set_value(
// 		"worldMatrix",
// 		Share::new(WorldMatrixUbo::new(UniformValue::MatrixV4(arr))),
// 	);
// }

// fn use_layout_pos<C: HalContext + 'static>(
// 	render_obj: &mut RenderObj,
// 	uv: Aabb2,
// 	layout: &LayoutR,
// 	radius: &Point2,
// 	engine: &mut Engine<C>,
// ) {
// 	let width = layout.rect.end - layout.rect.start;
// 	let height = layout.rect.bottom - layout.rect.top;
// 	let start_x = layout.border.start;
// 	let start_y = layout.border.top;
// 	let end_x = width - layout.border.end;
// 	let end_y = height - layout.border.bottom;
// 	let (positions, indices) = if radius.x == 0.0 || width == 0.0 || height == 0.0 {
// 		(
// 			vec![
// 				start_x, start_y, start_x, end_y, end_x, end_y, end_x, start_y,
// 			],
// 			vec![0, 1, 2, 3],
// 		)
// 	} else {
// 		split_by_radius(
// 			start_x,
// 			start_y,
// 			end_x - start_x,
// 			end_y - start_y,
// 			radius.x - start_x,
// 			None,
// 		)
// 	};
// 	// debug_println!("indices: {:?}", indices);
// 	// debug_println!("split_by_lg,  positions:{:?}, indices:{:?}, top_percent: {}, bottom_percent: {}, start: ({}, {}) , end: ({}, {})", positions, indices, 0.0, 1.0, 0.0, 0.0, 0.0, layout.height);
// 	let (positions, indices_arr) = split_by_lg(
// 		positions,
// 		indices,
// 		&[0.0, 1.0],
// 		(0.0, 0.0),
// 		(0.0, height),
// 	);
// 	// debug_println!("split_mult_by_lg, positions: {:?}, indices_arr: {:?}, cfg: {:?}, percent: [{}, {}], start: [{}, {}], end: [{}, {}]",  &positions, indices_arr, vec![LgCfg{unit: 1, data: vec![uv.min.x, uv.max.x]}], 0.0, 1.0, 0.0, 0.0, layout.width, 0.0);
// 	let (positions, indices_arr) = split_mult_by_lg(
// 		positions,
// 		indices_arr,
// 		&[0.0, 1.0],
// 		(0.0, 0.0),
// 		(width, 0.0),
// 	);
// 	let indices = mult_to_triangle(&indices_arr, Vec::new());
// 	// debug_println!("u positions: {:?}, indices_arr: {:?}, cfg: {:?}, percent: [{}, {}], start: [{}, {}], end: [{}, {}]",  &positions, indices_arr, vec![LgCfg{unit: 1, data: vec![uv.min.x, uv.max.x]}], 0.0, 1.0, 0.0, 0.0, layout.width, 0.0);
// 	let u = interp_mult_by_lg(
// 		&positions,
// 		&indices_arr,
// 		vec![Vec::new()],
// 		vec![LgCfg {
// 			unit: 1,
// 			data: vec![uv.mins.x, uv.maxs.x],
// 		}],
// 		&[0.0, 1.0],
// 		(0.0, 0.0),
// 		(width, 0.0),
// 	);
// 	let v = interp_mult_by_lg(
// 		&positions,
// 		&indices_arr,
// 		vec![Vec::new()],
// 		vec![LgCfg {
// 			unit: 1,
// 			data: vec![uv.mins.y, uv.maxs.y],
// 		}],
// 		&[0.0, 1.0],
// 		(0.0, 0.0),
// 		(0.0, height),
// 	);
// 	// debug_println!("v positions: {:?}, indices_arr: {:?}, cfg: {:?}, percent: [}, {}], start: [{}, {}], end: [{}, {}]",  &positions, indices_arr, vec![LgCfg{unit: 1, data: vec![uv.min.y, uv.max.y]}], 0.0, 1.0, 0.0, 0.0, 0.0, layout.height);
// 	let mut uvs = Vec::with_capacity(u[0].len());
// 	for i in 0..u[0].len() {
// 		uvs.push(u[0][i]);
// 		uvs.push(v[0][i]);
// 	}

// 	render_obj.geometry = Some(engine.create_geo_res(
// 		0,
// 		indices.as_slice(),
// 		&[
// 			AttributeDecs::new(AttributeName::Position, positions.as_slice(), 2),
// 			AttributeDecs::new(AttributeName::UV0, uvs.as_slice(), 2),
// 		],
// 	));
// }

// // impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, RenderContext, CreateEvent> for RenderContextSys<C> {
// // 	type ReadData = &'a SingleCaseImpl<IdTree>;
// // 	type WriteData = &'a mut SingleCaseImpl<RootIndexs>;
// // 	fn listen(&mut self, event: &CreateEvent, idtree: Self::ReadData, root_indexs: Self::WriteData) {
// // 		match idtree.get(event.id) {
// // 			Some(r) => {
// // 				root_indexs.delete(event.id, idtree.get(event.id).layer());
// // 			},
// // 			None => (),
// // 		}
// // 		root_indexs.1 = true;
// // 	}
// // }

// impl_system! {
// 	RenderContextSys<C> where [C: HalContext + 'static],
// 	true,
// 	{
// 		MultiCaseListener<Node, RenderContext, CreateEvent>
// 		MultiCaseListener<Node, RenderContext, DeleteEvent>
// 		MultiCaseListener<Node, RenderContext, ModifyEvent>
// 		SingleCaseListener<IdTree, CreateEvent>
// 		// SingleCaseListener<Oct, CreateEvent>
// 	}
// }