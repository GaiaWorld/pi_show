use core::panic;
use std::cell::RefCell;
/// 处理渲染上下文（遮罩、半透明、裁剪、willchange都可以创建渲染上下文）
/// 目前只支持了遮罩，
///
/// 1.计算一个渲染上下文所有子节点的包围盒的交集范围，得到渲染上下文的包围盒（目前就是渲染上下文本节点的包围盒，没有递归子节点， TODO）
/// 2. 在共享纹理中分配一个该包围盒大小的区域，用于渲染上下文节点及其所有递归子节点。
///    * 此时，渲染这些节点的世界矩阵为原世界矩阵
///    * 由于不是渲染到最终目标上，而是渲染到中间目标上，其投影矩阵为：包围盒宽高对应的投影矩阵*将包围盒左上角移动到0，0位置的矩阵,  注意：包围盒应该考虑父设置的willchange
///    * 视图矩阵通常为单位矩阵，如果父存在willchange，则为willchangematrix
//
/// 3. 将渲染后的纹理的该区域，再次渲染到父渲染目标，
/// 

use std::marker::PhantomData;

use ecs::entity::Entity;
use share::Share;
use std::hash::{Hash, Hasher};

// use ordered_float::NotNan;
use hash::{DefaultHasher, XHashSet, XHashMap};

use atom::Atom;
use ecs::{CreateEvent, DeleteEvent, ModifyEvent, EntityListener, MultiCaseImpl, MultiCaseListener, Runner, SingleCaseImpl, SingleCaseListener};
use ecs::monitor::{Event, NotifyImpl};
use hal_core::*;
use map::vecmap::VecMap;

use crate::component::calc::{LayoutR, MaskTexture};
use crate::component::calc::*;
use crate::component::user::{ *, Opacity};
use crate::entity::Node;
use crate::render::engine::{Engine, ShareEngine};
use crate::render::res::*;
use crate::{single::*};
use crate::single::dyn_texture::DynAtlasSet;
use crate::single::{DirtyViewRect};
use crate::system::render::shaders::image::{ FBO_VS_SHADER_NAME, FBO_FS_SHADER_NAME};
use crate::system::util::constant::*;
use crate::system::util::{*, let_top_offset_matrix as let_top_offset_matrix1};
use crate::Z_MAX;

lazy_static! {
	static ref UV: Atom = Atom::from("UV");
	static ref POSITION: Atom = Atom::from("Position");
	static ref INDEX: Atom = Atom::from("Index");
}

const DIRTY_TY: usize = StyleType::Matrix as usize
	| StyleType::Opacity as usize
	| StyleType::Layout as usize;

const DIRTY_TY1: usize = StyleType1::MaskTexture as usize
	| StyleType1::MaskImageClip as usize
	| StyleType1::ContentBox as usize;

pub struct RenderContextSys<C> {
	dirty: XHashSet<usize>,
	render_map: VecMap<usize>,
	default_sampler: Share<SamplerRes>,
	uv1_sampler: Share<SamplerRes>,
	unit_geo: Share<GeometryRes>, // 含uv， index， pos
	default_paramter: FboParamter,

	marker: PhantomData<C>,
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for RenderContextSys<C> {
	type ReadData = (
		&'a MultiCaseImpl<Node, RenderContextMark>,
		&'a MultiCaseImpl<Node, LayoutR>,
		&'a MultiCaseImpl<Node, ZDepth>,
		&'a MultiCaseImpl<Node, WorldMatrix>,
		&'a MultiCaseImpl<Node, Transform>,
		&'a MultiCaseImpl<Node, StyleMark>,
		&'a MultiCaseImpl<Node, TransformWillChangeMatrix>,
		&'a MultiCaseImpl<Node, ContentBox>,
		&'a SingleCaseImpl<PremultiState>,
		&'a SingleCaseImpl<IdTree>,
		&'a SingleCaseImpl<Oct>,
		&'a SingleCaseImpl<RenderBegin>,
	);
	type WriteData = (
		&'a mut MultiCaseImpl<Node, RenderContext>,
		&'a mut SingleCaseImpl<RenderObjs>,
		&'a mut SingleCaseImpl<ShareEngine<C>>,
		&'a mut SingleCaseImpl<Share<RefCell<DynAtlasSet>>>,
	);
	fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
		let (
			render_context_marks,
			layouts,
			z_depths,
			world_matrixs,
			transforms,
			style_marks,
			willchange_matrixs,
			content_boxs,
			premulti_state,
			idtree,
			octree,
			render_begin,
		) = read;
		if self.dirty.len() == 0 {
			return;
		}

		let (
			render_contexts,
			render_objs,
			engine,
			dyn_atlas_set) = write;
		let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
		
		let mut dirty = std::mem::replace(&mut self.dirty, XHashSet::default());
		let mut render_target_change = false;
		for id in dirty.iter() {
			let style_mark = match style_marks.get(*id) { // 节点已经销毁，不做处理
				Some(r) => r,
				None => continue,
			};
			let node = &idtree[*id];
			if node.layer() == 0 {
				continue;
			}

			let dirty = style_mark.dirty;
			let dirty1 = style_mark.dirty1;
			
			// log::info!("is dirty: {:?}",dirty & DIRTY_TY == 0 && dirty1 & DIRTY_TY1 == 0);
			if dirty & DIRTY_TY == 0 && dirty1 & DIRTY_TY1 == 0 {
				continue;
			}
			
			// 无渲染上下文的标记，则删除RenderContext
			let render_context_mark = match render_context_marks.get(*id) {
				Some(mark) if !mark.is_empty() => mark,
				_ => {
					self.unbind_context(*id, render_contexts, &mut render_target_change);
					continue;
				}
			};

			render_target_change = true;

			let (render_context, is_create) = match render_contexts.get_mut(*id) {
				Some(r) => (r, false),
				None => {
					let (state, vs, fs) = (&***premulti_state, FBO_VS_SHADER_NAME.clone(), FBO_FS_SHADER_NAME.clone());
					let render_obj_index = self.create_render_obj(*id, render_objs, state, vs, fs);

					let aabb = content_boxs.get(*id).unwrap().0;
					
					render_contexts.insert(*id,
						RenderContext::new(1,
							aabb.clone(),
							aabb.clone(),
							Some(WorldMatrix(Matrix4::new_nonuniform_scaling(&Vector3::new(1.0,1.0,1.0)), false)),
							Some(ProjectionMatrix::new(10.0, 10.0,1.0, 2.0)),
							Some(Share::new(WorldMatrixUbo::default())),
							Some(Share::new(ProjectMatrixUbo::default())),
							DirtyViewRect(0.0, 0.0,0.0,0.0, false),
							Some(0),
							render_obj_index)
					);
					(render_contexts.get_mut(*id).unwrap(), true)
				},
			};
			
			let render_obj = &mut render_objs[render_context.render_obj_index];
			render_obj.is_opacity = false;

			let z_depth = z_depths[*id].0;
			let layout = &layouts[*id];

			let transform = &transforms[*id];
			let world_matrix = &world_matrixs[*id];

			if dirty & DIRTY_TY != 0 || dirty1 & DIRTY_TY1 != 0 {

				// 矩阵或布局发生改变， 需要更新fbo纹理，和uv
				let mut aabb = content_boxs.get(*id).unwrap().0;
				let viewport = render_begin.0.viewport;
				aabb = match intersect(&aabb, &Aabb2::new(Point2::new(0.0, 0.0), Point2::new(viewport.2 as f32, viewport.3 as f32))) {
					Some(r) => r,
					None => Aabb2::new(Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)),
				};

				if is_create || dirty1 & DIRTY_TY1 != 0 {
					update_geo_quad(render_obj, engine, &self.unit_geo);
					render_context.geo_change = true;
					render_context.content_box = aabb;
				}
				
				// 应该树contentbox发生改变时，TODO
				if is_create || dirty1 & StyleType1::ContentBox as usize != 0 {
					// 目的： 将其下渲染的字节，以本节点左上为原点对齐
					let (left_top_matrix, view_matrix) = let_top_offset_matrix(*id, 
						layout, world_matrix,transform, idtree, willchange_matrixs, &aabb
					);
					
					// render_context.projection_matrix = ProjectionMatrix(ProjectionMatrix::new(
					// 	rect.maxs.x - rect.mins.x,
					// 	rect.maxs.y - rect.mins.y,
					// 	-Z_MAX - 1.0,
					// 	Z_MAX + 1.0,
					// ).0 * left_top_matrix.clone());

					let buffer = Vec::from(view_matrix.0.as_slice());
					render_context.view_matrix_ubo = Some(Share::new(ViewMatrixUbo::new(
						UniformValue::MatrixV4(buffer),
					)));
					render_context.view_matrix = Some(view_matrix);
					// buffer = Vec::from(render_context.projection_matrix.0.as_slice());
					// render_context.projection_matrix_ubo = Share::new(ViewMatrixUbo::new(
					// 	UniformValue::MatrixV4(buffer),
					// ));

					modify_matrix(
						render_obj,
						z_depth,
						&aabb
					);
				}
				notify.modify_event(render_context.render_obj_index, "ubo", 0);
			}
			notify.modify_event(render_context.render_obj_index, "", 0);
		}

		dirty.clear();
		self.dirty = dirty;

		if render_target_change {
			notify.modify_event(0, "context", 0);
		}
	}
}


impl<'a, C> MultiCaseListener<'a, Node, ContentBox, (CreateEvent, ModifyEvent)> for RenderContextSys<C>
	where
		C: HalContext + 'static {
	type ReadData = (
		&'a MultiCaseImpl<Node, RenderContextMark>,
		&'a SingleCaseImpl<Oct>, 
		&'a MultiCaseImpl<Node, ContentBox>
	);
	type WriteData = &'a mut MultiCaseImpl<Node, RenderContext>;
	fn listen(
		&mut self, 
		event: &Event, 
		(render_context_mark, octs, content_box): Self::ReadData, 
		render_contexts: Self::WriteData) {
		/// 根节点一定是一个renderContext
		if event.id == 1 {
			if let Some(r) = render_contexts.get_mut(1) {
				r.content_box = octs.get(1).unwrap().0.clone();
				r.content_box.mins.x = r.content_box.mins.x.floor();
				r.content_box.mins.y = r.content_box.mins.y.floor();
				r.content_box.maxs.x = r.content_box.maxs.x.ceil();
				r.content_box.maxs.y = r.content_box.maxs.y.ceil();
			}
			return;
		}

		// 如果该节点存在一个上下文，则设脏
		if let Some(mark) = render_context_mark.get(event.id) {
			if !mark.is_empty() {
				self.dirty.insert(event.id);
			}
		}
	}
}

/// RenderBegin发生变化，设置DynAtlasSet中默认的纹理宽高
/// 保持DynAtlasSet的宽高与屏幕一致，他们可以共用一张深度缓冲区
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderBegin, ModifyEvent> for RenderContextSys<C> {
	type ReadData = (&'a SingleCaseImpl<RenderBegin>, &'a SingleCaseImpl<ShareEngine<C>>);
	type WriteData = (&'a mut MultiCaseImpl<Node, RenderContext>, &'a mut SingleCaseImpl<Share<RefCell<DynAtlasSet>>>);
	fn listen(&mut self, _event: &Event, (render_begin, engine): Self::ReadData,(render_contexts, dyn_atlas_set): Self::WriteData) {
		if let Some(r) = render_contexts.get_mut(1) {
			r.render_rect = Aabb2::new(Point2::new(render_begin.0.viewport.0 as f32, render_begin.0.viewport.1 as f32), Point2::new(render_begin.0.viewport.0 as f32 + render_begin.0.viewport.2 as f32, render_begin.0.viewport.1 as f32 + render_begin.0.viewport.3 as f32));
		}
		let size = (render_begin.0.viewport.2 as u32, render_begin.0.viewport.3 as u32);
		dyn_atlas_set.borrow_mut().set_default_size(size.0 as usize, size.1 as usize);
	}
}

/// 监听渲染上下文的删除事件，释放渲染抓住的渲染对象，并释放其分配的纹理空间
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, RenderContext, DeleteEvent> for RenderContextSys<C> {
	type ReadData = &'a MultiCaseImpl<Node, RenderContext>;
	type WriteData = (&'a mut SingleCaseImpl<Share<RefCell<DynAtlasSet>>>, &'a mut SingleCaseImpl<RenderObjs>);
	fn listen(&mut self, event: &Event, render_contexts: Self::ReadData, (dyn_atlas_set, render_objs): Self::WriteData) {
		match render_contexts.get(event.id) {
			Some(ctx) => {
				if let Some(render_target) = ctx.render_target {
					self.remove_render_obj(ctx.render_obj_index, render_objs);
					// log::info!("delete_rect rendercontext remove============={}", render_target);
					dyn_atlas_set.borrow_mut().delete_rect(render_target);
				}

				if let Some(post_process) = &ctx.post_process {
					if let Some(result) = post_process.result {
						// log::info!("delete_rect rendercontext remove1============={}", result);
						dyn_atlas_set.borrow_mut().delete_rect(result);
					}
				}
			}, 
			None => ()
		};
	}
}



/// 监听实体销毁，释放渲染抓住的渲染对象，并释放其分配的纹理空间
/// 实体删除，不会再通知组件的删除事件，所以除了监听组件的RenderContext组件的删除，还要监听实体的删除
impl<'a, C: HalContext + 'static> EntityListener<'a, Node, ModifyEvent>
    for RenderContextSys<C>
{
	type ReadData = &'a MultiCaseImpl<Node, RenderContext>;
	type WriteData = (&'a mut SingleCaseImpl<Share<RefCell<DynAtlasSet>>>, &'a mut SingleCaseImpl<RenderObjs>);
	fn listen(&mut self, event: &Event, render_contexts: Self::ReadData, (dyn_atlas_set, render_objs): Self::WriteData) {
		match render_contexts.get(event.id) {
			Some(ctx) => {
				if let Some(render_target) = ctx.render_target {
					self.remove_render_obj(ctx.render_obj_index, render_objs);
					// log::info!("delete_rect Node remove============={}", render_target);
					dyn_atlas_set.borrow_mut().delete_rect(render_target);
				}

				if let Some(post_process) = &ctx.post_process {
					if let Some(result) = post_process.result {
						// log::info!("delete_rect Node remove1============={}", result);
						dyn_atlas_set.borrow_mut().delete_rect(result);
					}
				}
			}, 
			None => ()
		};
	}
}

// idtree创建时，递归遍历，如果子节点中存在MaskImage，记脏
// 注意，不需要处理IdTree的add事件，不在树上，创建RenderContext也没有意义
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, RenderContextMark, (CreateEvent, DeleteEvent, ModifyEvent)> for RenderContextSys<C> {
	type ReadData = ();
	type WriteData = ();
	fn listen(
		&mut self, 
		event: &Event, 
		_: Self::ReadData, 
		_: Self::WriteData) {
		self.dirty.insert(event.id);
	}
}

// idtree创建时，递归遍历，如果子节点中存在MaskImage，记脏
// 注意，不需要处理IdTree的add事件，不在树上，创建RenderContext也没有意义
impl<'a, C: HalContext + 'static> SingleCaseListener <'a, IdTree, CreateEvent> for RenderContextSys<C> {
	type ReadData = (
		&'a SingleCaseImpl<IdTree>, 
		&'a MultiCaseImpl<Node, RenderContextMark>, 
		&'a SingleCaseImpl<RenderBegin>, 
		&'a SingleCaseImpl<RenderRect>);
	type WriteData = &'a mut MultiCaseImpl<Node, RenderContext>;
	fn listen(
		&mut self, 
		event: &Event, 
		(
			idtree, 
			render_context_marks, 
			render_begin, 
			render_rect
		): Self::ReadData, 
		render_contexts: Self::WriteData) {
		if event.id == 1 { 
			// 如果id是1, 则直接创建渲染上下文
			render_contexts.insert(1,
				RenderContext::new(1,
					Aabb2::new(Point2::new(render_begin.0.viewport.0 as f32, render_begin.0.viewport.1 as f32), Point2::new(render_begin.0.viewport.0 as f32 + render_begin.0.viewport.2 as f32, render_begin.0.viewport.1 as f32 + render_begin.0.viewport.3 as f32)),
					Aabb2::new(Point2::new(0.0, 0.0), Point2::new(render_rect.width as f32, render_rect.height as f32)),
					None,
					None,
					None,
					None,
					DirtyViewRect(0.0, 0.0,0.0,0.0, false),
					None,
					0)
			);
			return;
		}

		let node = &idtree[event.id];
		// 插入自身到脏列表，因为除了对其脏列表中的节点插入渲染上下文，也需要为节点设置自己的渲染上下文的索引
		// 自身没有opacity、maskImage等，也需要设置渲染上下文
		self.dirty.insert(event.id);

		for (id, _node) in idtree.recursive_iter(node.children().head) {
			if let Some(mark) = render_context_marks.get(id) {
				if !mark.is_empty() {
					self.dirty.insert(id);
				}
			}
		}
	}
}

impl<C: HalContext + 'static> RenderContextSys<C> {
	pub fn with_capacity(engine: &mut Engine<C>, capacity: usize) -> Self {
		let mut sm = SamplerDesc::default();
		// 使用点采样，因为fbo的纹理部分和渲染的实际大小一致
		sm.u_wrap = TextureWrapMode::ClampToEdge;
		sm.v_wrap = TextureWrapMode::ClampToEdge;
		sm.min_filter = TextureFilterMode::Nearest;
		sm.mag_filter = TextureFilterMode::Nearest;

		let default_sampler = engine.create_sampler_res(sm);

		let mut sm1 = SamplerDesc::default();
		sm1.u_wrap = TextureWrapMode::ClampToEdge;
		sm1.v_wrap = TextureWrapMode::ClampToEdge;
		let default_sampler1 = engine.create_sampler_res(sm1);

		let positions = engine
			.buffer_res_map
			.get(&(POSITIONUNIT.get_hash() as u64))
			.unwrap();
		let indices = engine
			.buffer_res_map
			.get(&(INDEXUNIT.get_hash() as u64))
			.unwrap();

		let geo = engine.create_geometry();
		engine
			.gl
			.geometry_set_attribute(&geo, &AttributeName::Position, &positions, 2)
			.unwrap();
		engine
			.gl
			.geometry_set_attribute(&geo, &AttributeName::UV0, &positions, 2)
			.unwrap();
		engine
			.gl
			.geometry_set_indices_short(&geo, &indices)
			.unwrap();

		RenderContextSys {
			dirty: XHashSet::default(),
			render_map: VecMap::with_capacity(capacity),
			default_sampler: default_sampler,
			uv1_sampler: default_sampler1,
			unit_geo: Share::new(GeometryRes {
				geo: geo,
				buffers: vec![indices, positions.clone(), positions],
			}),
			default_paramter: FboParamter::default(),
			marker: PhantomData,
		}
	}

	#[inline]
	fn unbind_context(&mut self, id: usize, render_ctxs: &mut MultiCaseImpl<Node, RenderContext>, render_target_change: &mut bool) -> bool {
		match render_ctxs.get_mut(id) {
			Some(_ctx) => {
				render_ctxs.delete(id);
				*render_target_change = true;
				true
			}
			None => false,
		}
	}

	#[inline]
	fn remove_render_obj(&mut self, id: usize, render_objs: &mut SingleCaseImpl<RenderObjs>) {
		let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
		render_objs.remove(id, Some(notify));
	}

	#[inline]
	fn create_render_obj(
		&mut self,
		id: usize,
		render_objs: &mut SingleCaseImpl<RenderObjs>,
		default_state: &CommonState,
		vs: Atom,
		fs: Atom,
	) -> usize {
		create_render_obj(
			id,
			-0.1,
			true,
			vs,
			fs,
			Share::new(self.default_paramter.clone()),
			default_state,
			render_objs,
			&mut self.render_map,
		)
	}
}

fn update_geo_quad<C: HalContext + 'static>(
	render_obj: &mut RenderObj,
	engine: &mut Engine<C>,
	unit_geo: &Share<GeometryRes>,
	// uv: &Aabb2,
) {
	// let uv_hash = cal_uv_hash(&uv.mins, &uv.maxs);
	let geo = engine.create_geometry();
	engine
		.gl
		.geometry_set_attribute(
			&geo,
			&AttributeName::Position,
			&unit_geo.buffers[1],
			2,
		)
		.unwrap();
	// engine
	// 	.gl
	// 	.geometry_set_attribute(&geo, &AttributeName::UV0, &uv_buffer, 2)
	// 	.unwrap();
	engine
		.gl
		.geometry_set_indices_short(&geo, &unit_geo.buffers[0])
		.unwrap();
	let geo_res = GeometryRes {
		geo: geo,
		buffers: vec![
			unit_geo.buffers[0].clone(),
			unit_geo.buffers[1].clone(),
			// uv_buffer,
		],
	};
	render_obj.geometry = Some(Share::new(geo_res));
}

#[inline]
fn unit_geo_hash(uv_hash: &u64, uv1_hash: &u64) -> u64 {
	let mut hasher = DefaultHasher::default();
	uv_hash.hash(&mut hasher);
	uv1_hash.hash(&mut hasher);
	POSITIONUNIT.hash(&mut hasher);
	INDEXUNIT.hash(&mut hasher);
	hasher.finish()
}

#[inline]
pub fn let_top_offset_matrix(
	id: usize,
	layout: &LayoutR,
	matrix: &WorldMatrix,
	transform: &Transform,
	idtree: &IdTree,
	willchange_matrixs: &MultiCaseImpl<Node, TransformWillChangeMatrix>,
	content_box: &Aabb2,
) -> (WorldMatrix, WorldMatrix) {
	let mut matrix = let_top_offset_matrix1(layout, matrix,transform, 0.0, 0.0 );
	let mut view_matrix = WorldMatrix(Matrix4::new_nonuniform_scaling(&Vector3::new(1.0,1.0,1.0)), false);
	if let Some(willchange) = find_transfrom_will_change(id, idtree, willchange_matrixs) {
		matrix = &willchange.0 * matrix;
		view_matrix = willchange.0.clone();
	}
	// let offsetX = matrix.m14;
	// let offsetY = matrix.m24;

	(WorldMatrix(
		Matrix4::new_translation(&Vector3::new(-content_box.mins.x,
			-content_box.mins.y,
			0.0)),
		false,
	), view_matrix)
}

#[inline]
fn modify_matrix(
	render_obj: &mut RenderObj,
	depth: f32,
	content_box: &Aabb2,
) {

    let matrix = WorldMatrix(
			Matrix4::new(
				content_box.maxs.x - content_box.mins.x,0.0,0.0,content_box.mins.x,
                0.0,content_box.maxs.y - content_box.mins.y,0.0,content_box.mins.y,
                0.0,0.0,1.0,0.0,
                0.0,0.0,0.0,1.0,
            ),
            false,
        );
    let slice: &[f32] = matrix.as_slice();
    let mut arr = Vec::from(slice);

	render_obj.paramter.set_value(
		"worldMatrix",
		Share::new(WorldMatrixUbo::new(UniformValue::MatrixV4(arr))),
	);
}

// 找父的transfrom_will_change（如果存在）
fn find_transfrom_will_change<'a>(
	mut id: usize,
	idtree: &'a IdTree,
	willchange_matrixs: &'a MultiCaseImpl<Node, TransformWillChangeMatrix>,
) -> Option<&'a TransformWillChangeMatrix> {
	loop {
		if id == 0 {
			return None;
		}
	
		if let Some(node) = idtree.get(id) {
			if let Some(matrix) = willchange_matrixs.get(id) {
				return Some(matrix);
			}
			id = node.parent();
		} else {
			return None;
		}
	}
}

impl_system! {
	RenderContextSys<C> where [C: HalContext + 'static],
	true,
	{
		MultiCaseListener<Node, RenderContextMark, (CreateEvent, DeleteEvent, ModifyEvent)>
		MultiCaseListener<Node, ContentBox, (CreateEvent, ModifyEvent)>
		MultiCaseListener<Node, RenderContext, DeleteEvent>
		EntityListener<Node, ModifyEvent>
		SingleCaseListener<IdTree, CreateEvent>
		SingleCaseListener<RenderBegin, ModifyEvent>
	}
}