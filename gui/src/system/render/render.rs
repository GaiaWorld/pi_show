/**
 *  渲染， 将渲染对象按照透明与不透明分类， 先渲染不透明物体， 再渲染透明物体， 不透明物体按照渲染管线的顺序渲染， 透明物体按照物体的深度顺序渲染
 */
use std::cmp::Ordering;
use std::default::{Default};
use std::marker::PhantomData;
use std::cell::RefCell;

use ecs::{
    CreateEvent, DeleteEvent, ModifyEvent, Runner, SingleCaseImpl, SingleCaseListener, MultiCaseImpl,
};
use ecs::monitor::{Event};
use hal_core::*;
use nalgebra::Orthographic3;
use ordered_float::OrderedFloat;
use share::Share;
use crate::Z_MAX;
use crate::component::user::{Aabb2, Matrix4, Point2, Vector3, Opacity};
use crate::component::calc::{WorldMatrix, RenderContext, ProjectMatrixUbo, FboParamter, Visibility};
use crate::entity::Node;

use crate::render::engine::{Engine, ShareEngine};
use crate::render::res::{SamplerRes, GeometryRes, BufferRes};
use crate::single::{PreRenderList, PostProcessContext, State, PostProcessObj};
use crate::single::{DirtyViewRect, IdTree, NodeRenderMap, Oct, ProjectionMatrix, RenderBegin, RenderObj, RenderObjs, Statistics, dyn_texture::DynAtlasSet};
use crate::system::util::{cal_uv_hash, create_uv_buffer, intersect};

pub struct RenderSys<C: HalContext + 'static> {
    program_dirtys: Vec<usize>,
    transparent_dirty: bool,
    opacity_dirty: bool,
    pub dirty: bool,
	default_sampler: Share<SamplerRes>,
	linner_sampler: Share<SamplerRes>,
	is_update_context_texture: bool,
	pub view_matrix_ubo: Share<dyn UniformBuffer>,
	pub projection_matrix_ubo: Share<dyn UniformBuffer>,
	pub render_count: usize,
	pub geometry: HalGeometry,
	pub uv_buffer: HalBuffer,
    marker: PhantomData<C>,
}


impl<C: HalContext + 'static> RenderSys<C> {
	pub fn new(engine: &mut Engine<C>, project_matrix: &ProjectionMatrix) -> Self {
		let mut sm = SamplerDesc::default();
		// 使用点采样，因为fbo的纹理部分和渲染的实际大小一致
		sm.u_wrap = TextureWrapMode::ClampToEdge;
		sm.v_wrap = TextureWrapMode::ClampToEdge;
		sm.min_filter = TextureFilterMode::Nearest;
		sm.mag_filter = TextureFilterMode::Nearest;

		let mut sm1 = SamplerDesc::default();
		sm1.u_wrap = TextureWrapMode::ClampToEdge;
		sm1.v_wrap = TextureWrapMode::ClampToEdge;

		let mut g = engine.create_geometry();
		let p_buffer = engine.create_buffer(
			BufferType::Attribute,
			8,
			Some(BufferData::Float(&vec![-1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0])),
			false,
		);
		let uv_buffer = engine.create_buffer(
			BufferType::Attribute,
			8,
			Some(BufferData::Float(&vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0])),
			false,
		);
		let indices = vec![0, 1, 2, 0, 2, 3];
		let i_buffer = engine.create_buffer(
			BufferType::Indices,
			indices.len(),
			Some(BufferData::Short(&indices)),
			false,
		);
		engine
			.gl
			.geometry_set_attribute(&g, &AttributeName::Position, &p_buffer, 2)
			.unwrap();
		engine
			.gl
			.geometry_set_attribute(&g, &AttributeName::UV0, &uv_buffer, 2)
			.unwrap();
		engine
			.gl
			.geometry_set_indices_short(&g, &i_buffer)
			.unwrap();

		let default_sampler = engine.create_sampler_res(sm);
		let sampler1 = engine.create_sampler_res(sm1);
		Self {
			
			program_dirtys: Vec::default(),
			transparent_dirty: false,
			opacity_dirty: false,
			dirty: false,
			is_update_context_texture: false,
			default_sampler: default_sampler,
			linner_sampler: sampler1,
			view_matrix_ubo: Share::new(ProjectMatrixUbo::new(
				UniformValue::MatrixV4(vec![1.0,0.0,0.0,0.0, 0.0,1.0,0.0,0.0, 0.0,0.0,1.0,0.0, 0.0,0.0,0.0,1.0]),
			)),
			projection_matrix_ubo: Share::new(ProjectMatrixUbo::new(
				UniformValue::MatrixV4(Vec::from(project_matrix.0.as_slice())),
			)),
			render_count: 0,
			geometry: g, 
			uv_buffer,
			marker: PhantomData,
		}
	}

	fn render(
		&self,
		gl: &C,
		obj: &RenderObj,
		// statistics: &mut Statistics,
		project_matrix: Option<&Share<dyn UniformBuffer>>,
		view_matrix: Option<&Share<dyn UniformBuffer>>,
		// dyn_atlas_set: &DynAtlasSet,
		// unit_geo: &HalGeometry,
		id: usize,
	){
		// if let Some(r) = obj.post_process {
		// 	if let Some(index) = r.result {
		// 		let target = dyn_atlas_set.get_target(index).unwrap();
		// 		let texture = gl.rt_get_color(target, 0).unwrap();
		// 		let param = FboParamter::default();
		// 		param.set_texture("texture", (texture, &self.default_sampler));
		// 		param.set_single_uniform("alpha", UniformValue::Float1(1.0));
		// 		param.set_value("worldMatrix", obj.paramter.get_value("worldMatrix").unwrap().clone());
		// 		render1(gl, &self.geometry, &Share::new(param), &obj.state, obj.program.as_ref().unwrap(), project_matrix, view_matrix, statistics);
		// 		return;
		// 	}
		// }
		let geometry = match &obj.geometry {
			None => return,
			Some(g) => g,
		};

		// log::warn!("context: {}", obj.context);
		if let Err(e) = render1(gl, &geometry.geo, &obj.paramter, &obj.state, obj.program.as_ref().unwrap(), project_matrix, view_matrix) {
			log::error!("render err, context:{:?}, render_obj:{:?}, vs: {:?}, fs: {:?}， error： {:?}", 
				obj.context,
				id,
				obj.vs_name,
				obj.fs_name,
				e,
			);
		}
	}
}

#[inline]
fn is_intersect(a: &Aabb2, b: &Aabb2) -> bool {
    if a.mins.x > b.maxs.x || a.mins.y > b.maxs.y || b.mins.x > a.maxs.x || b.mins.y > a.maxs.y {
        return false;
    } else {
        true
    }
}

fn update_geo<C: HalContext + 'static>(
	render_obj: &mut RenderObj,
	engine: &mut Engine<C>,
	uv: &Aabb2,
) {
	
	if let Some(r) = &render_obj.geometry {
		let uv_hash = cal_uv_hash(&uv.mins, &uv.maxs);
		let uv_buffer = create_uv_buffer(uv_hash, &uv.mins, &uv.maxs, engine);
		engine
			.gl
			.geometry_set_attribute(
				r,
				&AttributeName::UV0,
				&uv_buffer,
				2,
			)
			.unwrap();
	}
}

fn get_render_project_matrix(content_box: &Aabb2) -> WorldMatrix{
	WorldMatrix(
		Matrix4::new_translation(&Vector3::new(-content_box.mins.x,
			-content_box.mins.y,
			0.0)),
		false,
	)
}

// 137
// 46
// ]
// maxs:[
// 184
// 93
// #[test]
// fn xxx() {

// }
impl<'a, C: HalContext + 'static>  RenderSys<C> {
	
	fn recursive_list_by_intersect<'b>(&mut self, base: &RenderBase<'b>, 
		basemut: &mut RenderBaseMut<'b, C>, head: usize, parent_target: usize, render_context: &mut RenderContext) {
		let render_contexts1 = unsafe{ &mut *(base.render_contexts as *const MultiCaseImpl<Node, RenderContext> as usize as *mut MultiCaseImpl<Node, RenderContext>) };
		let (idtree, octree) = ( base.idtree.clone(),  base.octree.clone());
		for (id, node) in idtree.iter(head) {
			let oct = match octree.get(id) {
				Some(r) => r.0,
				None => {
					// log::warn!("render list fail, oct is not exist, id: {}", id);
					return;
				}
			};
			
			if let Some(r) = render_contexts1.get_mut(id) {
				self.render_context(base, basemut, id, parent_target,r);
				let is_render = match r.render_target {
					Some(r) => r > 0,
					None => false
				};

				if r.render_count > 0 && is_render && is_intersect(&base.dirty_rect, &r.content_box) {
					// log::info!("list===obj1==============={:?}, {:?}, {}", id, r.render_obj_index, basemut.render_objs[r.render_obj_index].context );

					if r.render_count > 0 && is_render {
						// render_post_process1
						let out = self.list_render_obj(base, basemut, render_context, r.render_obj_index, r.render_target.unwrap_or(0));
						let target = r.render_target.clone();
						let (render_index,sampler) = if let Some(post) = r.get_post_mut() {
							let result = self.render_post_process1(base, basemut, target.unwrap(), post, id);
							r.render_target = Some(0); // 它在后处理的过程中已经被释放
							(result, &self.linner_sampler)
						} else {
							(r.render_target.unwrap_or(0), &self.default_sampler)
						};
	
						if render_index > 0 {
							self.set_render_result(
								r.render_obj_index,
								render_index,
								sampler,
								basemut
							);
						}
						
					}
				}
				continue;
			}

			if let Some(r) = base.render_map.get(id) {
				if is_intersect(base.dirty_rect, oct) {
					for render_index in r.iter() {
						// if id == 279 {
						// 	log::info!("dirty_rect: {:?}", render_objs[*render_index].visibility);
						// }
						self.list_render_obj(base, basemut, render_context, *render_index, parent_target);
					}
				}
				
			}
			self.recursive_list_by_intersect(base, basemut, node.children().head, parent_target, render_context);
		}
	}

	fn recursive_list<'b>(&mut self, base: &RenderBase<'b>, 
		basemut: &mut RenderBaseMut<'b, C>,head: usize, parent_target: usize, render_context: &mut RenderContext) {
		let render_contexts1 = unsafe{ &mut *(base.render_contexts as *const MultiCaseImpl<Node, RenderContext> as usize as *mut MultiCaseImpl<Node, RenderContext>) };
		let idtree = base.idtree.clone();
		for (id, node) in idtree.iter(head) {
			if let Some(r) = render_contexts1.get_mut(id) {
				self.render_context(base, basemut, id, parent_target, r);
				let is_render = match r.render_target {
					Some(r) => r > 0,
					None => false
				};
				if r.render_count > 0 && is_render {
					// render_post_process1
					// log::info!("list===obj==============={:?}, {:?}, {}", id, r.render_obj_index, basemut.render_objs[r.render_obj_index].context );
					let out = self.list_render_obj(base, basemut, render_context, r.render_obj_index, r.render_target.unwrap_or(0));
					let target = r.render_target.clone();
					let (render_index,sampler) = if let Some(post) = r.get_post_mut() {
						let result = self.render_post_process1(base, basemut, target.unwrap(), post, id);
						r.render_target = Some(0); // 它在后处理的过程中已经被释放
						(result, &self.linner_sampler)
					} else {
						(r.render_target.unwrap_or(0), &self.default_sampler)
					};

					if render_index > 0 {
						self.set_render_result(
							r.render_obj_index,
							render_index,
							sampler,
							basemut
						);
					}
					
				}
				continue;
			}

			if let Some(r) = base.render_map.get(id) {
				for render_index in r.iter() {
					self.list_render_obj(base, basemut, render_context, *render_index, parent_target);
				}
			}
			self.recursive_list(base, basemut, node.children().head, parent_target, render_context);
		}
	}

	// post_process

	/// 渲染后处理
	fn render_post_process<'b>(&mut self, 
		base: &RenderBase<'b>, 
		basemut: &mut RenderBaseMut<'b, C>,
		render_index: usize,
		parent_target: usize,
		post_process_context: &mut PostProcessContext,
		) -> usize {
		let RenderBase {
			idtree,
			dirty_rect,
			is_reset,
			render_begin,
			render_contexts,
			..
		} = base;
		let RenderBaseMut{
			render_objs,
			engine,
			dyn_atlas_set
		} = basemut;
		let dyn_atlas_set1 = unsafe{&mut *( *dyn_atlas_set as *const DynAtlasSet as * mut DynAtlasSet)};
		let render_obj = &mut render_objs[render_index];
		let content_box = &post_process_context.content_box;
		let width = content_box.maxs.x-content_box.mins.x;
		let height = content_box.maxs.y-content_box.mins.y;
		let render_target = match post_process_context.render_target {
			Some(r) => r,
			None => {
				let target_index = dyn_atlas_set.update_or_add_rect(
					0,
					parent_target, 
					width,
					height,
					PixelFormat::RGBA, 
					DataFormat::UnsignedByte, 
					true, 
					1, 
					1, 
					&mut engine.gl);
				// log::info!("post update_or_add_rect============={}", target_index);
				target_index
			},
		};

		/// 渲染目标对象
		let mut target = dyn_atlas_set.get_target(render_target);
		let mut render_rect = dyn_atlas_set.get_rect(render_target).unwrap();
		let mut clear_rect = dyn_atlas_set.get_rect_with_border(render_target).unwrap();
		let cur_render_begin = self.calc_render_begin(&render_rect, &clear_rect, render_begin);

		// log::warn!("render_post_process1======================target, is_some: {:?}, {:?}, {:?}", rende_target, render_rect, &cur_render_begin. scissor);
		if cur_render_begin.viewport.2 == 0 || cur_render_begin.viewport.3 == 0 {
			// log::info!("render_post_process fail, scissor is zero, node:{}", render_obj.context);
			return 0;
		}
		// log::info!("begine====={:?}, {:?}, id:{}, dirty_rect: {:?}", begine.viewport, begine.scissor, id, dirty_rect);
		engine.gl.render_begin(
			target, 
			&cur_render_begin, 
			match self.render_count {
				0 => true,
				 _=> {engine.gl.render_reset_geometry();false}
				}
		);
		let view_ubo: Share<dyn UniformBuffer> = Share::new(ProjectMatrixUbo::new(
			UniformValue::MatrixV4(vec![1.0,0.0,0.0,0.0, 0.0,1.0,0.0,0.0, 0.0,0.0,1.0,0.0, 0.0,0.0,0.0,1.0]),
		));
		// log::warn!("get_projection_matrix======context={:?}, render_rect={:?}, content_box={:?}", render_obj.context, render_rect, content_box);
		let project_ubo = self.get_projection_matrix(&render_rect, content_box);
		// log::info!("post render======{}, vs:{:?}", obj.context, obj.vs_name);
		self.render(&engine.gl, render_obj, Some(&project_ubo), Some(&view_ubo), render_index);
		self.render_count += 1;

		if self.render_count > 0 { // 如果进行了一些渲染，则需要重置状态
			engine.gl.render_end();
			self.render_count = 0;
		}

		// log::info!("post_process======================{}", render_index);
		self.render_post_process1(base, basemut, render_target, post_process_context, render_index)
	}

	/// 渲染后处理
	fn render_post_process1<'b>(&mut self, 
		base: &RenderBase<'b>, 
		basemut: &mut RenderBaseMut<'b, C>,
		render_target: usize, // 后处理纹理
		post_process_context: &mut PostProcessContext,
		obj_index: usize,
		) -> usize {
		let RenderBase {
			idtree,
			dirty_rect,
			is_reset,
			render_begin,
			render_contexts,
			..
		} = base;
		let RenderBaseMut{
			render_objs,
			engine,
			dyn_atlas_set
		} = basemut;
		let dyn_atlas_set1 = unsafe{&mut *( *dyn_atlas_set as *const DynAtlasSet as * mut DynAtlasSet)};

		/// 渲染目标对象
		let mut source = dyn_atlas_set.get_target(render_target).unwrap();
		let mut render_rect = dyn_atlas_set.get_rect(render_target).unwrap();
		let mut clear_rect = dyn_atlas_set.get_rect_with_border(render_target).unwrap();
		let mut index = render_target;
		let mut target;
		
		/// 对目标对象进行后处理
		for post_processe in post_process_context.post_processes.iter_mut() {
			// log::warn!("post_processe======================post_processe, {}, {}", post_processe.render_size.width, post_processe.render_size.height);
			let uv = dyn_atlas_set.get_uv(index).unwrap();
			let target_index = dyn_atlas_set.update_or_add_rect(
				0,
				index, 
				post_processe.render_size.width,
				post_processe.render_size.height,
				PixelFormat::RGBA, 
				DataFormat::UnsignedByte, 
				true, 
				1, 
				1, 
				&mut engine.gl);
			// log::info!("post1 update_or_add_rect============={}, render_obj: {}", target_index, obj_index);
			target = dyn_atlas_set.get_target(target_index);
			render_rect = dyn_atlas_set.get_rect(target_index).unwrap();
			clear_rect = dyn_atlas_set.get_rect_with_border(target_index).unwrap();
			let cur_render_begin = self.calc_render_begin(&render_rect, &clear_rect, render_begin);
			// let project_ubo = self.get_projection_matrix(&render_rect, content_box);
			let obj = &mut post_processe.render_obj;

			if obj.geometry.is_none() {
				let mut g = engine.create_geometry();
				let p_buffer = engine.create_buffer(
					BufferType::Attribute,
					8,
					Some(BufferData::Float(&vec![-1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0])),
					false,
				);

				let indices = vec![0, 1, 2, 0, 2, 3];
				let i_buffer = engine.create_buffer(
					BufferType::Indices,
					indices.len(),
					Some(BufferData::Short(&indices)),
					false,
				);
				engine
					.gl
					.geometry_set_attribute(&g, &AttributeName::Position, &p_buffer, 2)
					.unwrap();
				engine
					.gl
					.geometry_set_indices_short(&g, &i_buffer)
					.unwrap();
				obj.geometry = Some(Share::new(GeometryRes{geo: g, buffers:vec![Share::new(BufferRes(p_buffer)), Share::new(BufferRes(i_buffer))]}));
			}

			let tt = dyn_atlas_set.get_target(index).unwrap();
			let t = engine.gl
					.rt_get_color(dyn_atlas_set.get_target(index).unwrap(), 0).unwrap();
			obj.paramter.set_texture("texture", (t, &self.linner_sampler));
			if obj.paramter.get_single_uniform("uvRegion").is_some() {
				let rect = dyn_atlas_set.get_rect(index).unwrap();
				obj.paramter.set_single_uniform(
					"uvRegion", 
					UniformValue::Float4(rect.mins.x, rect.mins.y, rect.maxs.x, rect.maxs.y),
				);
			}

			if obj.paramter.get_single_uniform("textureSize").is_some() {
				let size = dyn_atlas_set.get_target_size(index).unwrap();
				obj.paramter.set_single_uniform(
					"textureSize", 
					UniformValue::Float2(size.width as f32, size.height as f32),
				);
			}

			let uv = update_geo(obj, engine, &uv);

			if obj.program.is_none() || obj.program_dirty == true {
				let program = engine.create_program(
					obj.vs_name.get_hash() as u64,
					obj.fs_name.get_hash() as u64,
					&obj.vs_name,
					&*obj.vs_defines,
					&obj.fs_name,
					&*obj.fs_defines,
					obj.paramter.as_ref(),
				);
				obj.program = Some(program);
				obj.program_dirty == false;
			}	
			
			engine.gl.render_begin(
				target, 
				&cur_render_begin, 
				match self.render_count {
					0 => true,
					 _=> {engine.gl.render_reset_geometry();false}
					}
			);
			// uvRegion
			// log::info!("post render1======{}, vs:{:?}", obj.context, obj.vs_name);
			if let Err(e) = render1(
				&engine.gl,
				&obj.geometry.as_ref().unwrap(),
				&obj.paramter,
				&obj.state,
				obj.program.as_ref().unwrap(),
				None,
				None
			) {
				log::error!("render err, context:{:?}, render_obj:{:?}, vs: {:?}, fs: {:?}， error： {:?}", 
					obj.context,
					"post_process1",
					obj.vs_name,
					obj.fs_name,
					e,
				);
			}

			/// 释放分配（后处理分配以一些临时纹理）
			// log::info!("delete_rect render temp============={}", index);
			dyn_atlas_set1.delete_rect(index);

			self.render_count += 1;
			source = target.unwrap();
			index = target_index;

			if self.render_count > 0 { // 如果进行了一些渲染，则需要重置状态
				engine.gl.render_end();
				self.render_count = 0;
			}
		}
		let uv = dyn_atlas_set.get_uv(index).unwrap();
		// log::warn!("render_post_process======================end");

		/// 修改渲染结果的目标索引， 修改前释放旧的
		if let Some(r) = post_process_context.result {
			// log::info!("delete_rect render temp1============={}", r);
			dyn_atlas_set1.delete_rect(r);
		}

		post_process_context.result = Some(index);

		if post_process_context.copy > 0 {
			self.set_render_result(
				post_process_context.copy,
				index,
				&self.linner_sampler,
				basemut
			);
		}

		return index;
	}

	fn set_render_result<'b>(
		&self,
		target_obj_index: usize, 
		texture_index: usize, 
		sampler: &HalSampler,
		basemut: &mut RenderBaseMut<'b, C>
	) {
		let uv = basemut.dyn_atlas_set.get_uv(texture_index).unwrap();
		let copy = &mut basemut.render_objs[target_obj_index];
		let target = basemut.dyn_atlas_set.get_target(texture_index).unwrap();
		let t = basemut.engine.gl
					.rt_get_color(target, 0).unwrap();
		copy.paramter.set_texture("texture", (t, sampler));
		update_geo(copy, basemut.engine, &uv);
	}

	fn calc_render_begin(
		&self,
		rect: &Aabb2, 
		clear_rect: &Aabb2, 
		render_begin: &RenderBegin) -> RenderBeginDesc {
		let viewport = (
			rect.mins.x as i32, 
			rect.mins.y as i32, 
			(rect.maxs.x- rect.mins.x) as i32, 
			(rect.maxs.y - rect.mins.y) as i32);
		let scissor = (
			clear_rect.mins.x as i32, 
			clear_rect.mins.y as i32, 
			(clear_rect.maxs.x- clear_rect.mins.x) as i32, 
			(clear_rect.maxs.y - clear_rect.mins.y) as i32);

		RenderBeginDesc {
			viewport: viewport,
			scissor: scissor,
			clear_color: Some((OrderedFloat::from(0.0), OrderedFloat::from(0.0), OrderedFloat::from(0.0), OrderedFloat::from(0.0))),
			clear_depth: render_begin.0.clear_depth.clone(),
			clear_stencil: render_begin.0.clear_stencil.clone(),
		}
	}

	fn get_projection_matrix(&self, rect: &Aabb2, content_box: &Aabb2) -> Share<dyn UniformBuffer> {
		let project_martix = ProjectionMatrix(ProjectionMatrix::new(
			rect.maxs.x - rect.mins.x,
			rect.maxs.y - rect.mins.y,
			-Z_MAX - 1.0,
			Z_MAX + 1.0,
		).0 * get_render_project_matrix(content_box));
		let buffer = Vec::from(project_martix.0.as_slice());
		Share::new(ProjectMatrixUbo::new(
			UniformValue::MatrixV4(buffer),
		))
	}

	// 根据脏区域，列出渲染对象
	fn render_context<'b>(
		&mut self,
		base: &RenderBase<'b>,
		basemut: &mut RenderBaseMut<'b, C>,
		id: usize,
		parent_target: usize,
		render_context: &mut RenderContext,
	) {
		let RenderBase{
			render_map,
			idtree,
			render_contexts, 
			dirty_rect: &Aabb2,
			octree,
			render_begin,
			..
		} = base;
		let RenderBaseMut{
			render_objs,
			engine,
			dyn_atlas_set
		} = basemut;
		let is_reset = base.is_reset;

		let oldCount = render_context.opacity_list.len() + render_context.transparent_list.len();
		render_context.opacity_list.clear();
		render_context.transparent_list.clear();
		
		let mut render_target_change = false;
		let render_index_context = render_context.render_obj_index;
		
		if let Some(mut render_target) = render_context.render_target {
			let render_obj = &mut basemut.render_objs[render_index_context];
			// 重新赋值rendercontext render_obj的可见性，否则可能被overflow剔除（剔除是使用oct而不是content_box剔除）
			render_obj.visibility = base.visibilitys[id].0 && base.opacitys[id].0 > 0.0;
			if !render_obj.visibility { // 如果不可见，可以不用渲染
				return;
			}
			// log::info!("======================={:?}", render_obj.context);
			
			if self.is_update_context_texture || render_target == 0 {
				let content_box = render_context.content_box.clone();
				
				if content_box.maxs.x-content_box.mins.x <= 0.0 || content_box.maxs.y-content_box.mins.y <= 0.0 {
					return;
				}
				
				let target_index = basemut.dyn_atlas_set.update_or_add_rect(render_target,parent_target, content_box.maxs.x-content_box.mins.x,content_box.maxs.y-content_box.mins.y, PixelFormat::RGBA, DataFormat::UnsignedByte, true, 1, 1, &mut basemut.engine.gl);
				// log::info!("render_context update_or_add_rect============={}, old: {:?}", target_index, render_target);
				// 如果纹理区域修改了，则重新设置纹理，以及重新更新uv
				if target_index != 0 {
					render_context.render_target = Some(target_index);
					render_target = target_index;
					render_target_change = true;
				}
			}

			if render_target == 0 {
				return;
			}

			if render_target_change || render_context.geo_change {
				// let uv = basemut.dyn_atlas_set.get_uv(render_target).unwrap();
				let rect = basemut.dyn_atlas_set.get_rect(render_target).unwrap();
				let clear_rect = basemut.dyn_atlas_set.get_rect_with_border(render_target).unwrap();
				render_context.render_rect = rect.clone();
				render_context.clear_rect = clear_rect.clone();
				if rect.maxs.x - rect.mins.x <= 0.0 || rect.maxs.y - rect.mins.y <= 0.0 {
					return;
				}
				// 渲染目标矩形改变，投影矩阵也需要重新设置
				// 重新计算投影矩阵
				let project_martix = ProjectionMatrix(ProjectionMatrix::new(
					rect.maxs.x - rect.mins.x,
					rect.maxs.y - rect.mins.y,
					-Z_MAX - 1.0,
					Z_MAX + 1.0,
				).0 * get_render_project_matrix(&render_context.content_box));
				let buffer = Vec::from(project_martix.0.as_slice());
				render_context.projection_matrix_ubo = Some(Share::new(ProjectMatrixUbo::new(
					UniformValue::MatrixV4(buffer),
				)));
				render_context.projection_matrix = Some(project_martix);
				render_context.geo_change = false;
			}
		}

		let rect = &render_context.render_rect;
		let clear_rect = &render_context.clear_rect;
		let content_box = &render_context.content_box;
		let viewport = (rect.mins.x as i32, rect.mins.y as i32, (rect.maxs.x- rect.mins.x) as i32, (rect.maxs.y - rect.mins.y) as i32);
			
		// 修改裁剪区域
		let scissor = if is_reset == false && !render_target_change {
			// 渲染不全部重设，并且渲染目标未改变, 则根据脏区域重新计算裁剪区域
			// 渲染区域完全不在脏范围内，不需要重新渲染
			let intersect_rect = match intersect(content_box, &base.dirty_rect) {
				Some(r) => r,
				None => return,
			};
			// if id != 1 {
			// 	log::info!("oct: {:?}, {:?}", id, octree.get(id));
			// } else {
			// 	log::info!("scissor: {:?}, {:?}, {:?}, {:?}, {:?}", id, (
			// 		viewport.0 + (intersect_rect.mins.x - content_box.mins.x) as i32,
			// 		viewport.1 - (intersect_rect.maxs.y - content_box.maxs.y)as i32,
			// 		(intersect_rect.maxs.x - intersect_rect.mins.x) as i32,
			// 		(intersect_rect.maxs.y - intersect_rect.mins.y) as i32,
			// 	), viewport, dirty_rect, content_box);
			// }
			// log::info!("content_box====={:?}, {}, {:?}", content_box, id, rect);
			(
				viewport.0 + (intersect_rect.mins.x - content_box.mins.x) as i32,
				viewport.1 - (intersect_rect.maxs.y - content_box.maxs.y)as i32,
				(intersect_rect.maxs.x - intersect_rect.mins.x) as i32,
				(intersect_rect.maxs.y - intersect_rect.mins.y) as i32,
			)
		} else {
			// if id != 1 {
			// 	log::info!("oct1: {:?}, {:?}", id, octree.get(id));
			// } else {
			// 	log::info!("scissor1: {:?}, {:?}, {:?}, {:?}, {:?}", id, (rect.mins.x as i32, rect.mins.y as i32, (rect.maxs.x- rect.mins.x) as i32, (rect.maxs.y - rect.mins.y) as i32), viewport, dirty_rect, content_box);
			// }
			(clear_rect.mins.x as i32, clear_rect.mins.y as i32, (clear_rect.maxs.x- clear_rect.mins.x) as i32, (clear_rect.maxs.y - clear_rect.mins.y) as i32)
		};
		// log::info!("scissor====={:?}, {}, {:?}, {:?}", scissor, id, viewport, dirty_rect);

		let oct = match octree.get(id) {
			Some(r) => r.0,
			None => {
				// log::warn!("render list fail, oct is not exist, id: {}", id);
				return;
			}
		};
		if let Some(r) = render_map.get(id) {
			// 当渲染目标区域发生改变， 或渲染需要全部重置，或者节点包围盒与脏区域相交时，需要渲染该节点上的渲染对象
			if render_target_change || is_reset == true || (is_intersect(&base.dirty_rect, oct)) { 
				for render_obj_index in r.iter() {
					if *render_obj_index != render_index_context {
						// log::info!("list!!!==================={}, {}, {}", render_index_context, render_obj_index, id);
						self.list_render_obj(base, basemut, render_context, *render_obj_index, parent_target);
					}
				}
			}
		}

		if is_reset {
			self.recursive_list(
				base, basemut, idtree[id].children().head,
				match render_context.render_target {
					Some(r) => r,
					None => 0
				},
				render_context
			);
		} else {
			self.recursive_list_by_intersect(base, basemut, idtree[id].children().head,match render_context.render_target {
				Some(r) => r,
				None => 0
			}, render_context);
		}

		// 没有可渲染的物体，返回
		if render_context.opacity_list.len()==0 && render_context.transparent_list.len()==0 && oldCount == 0 { 
			render_context.render_count = 0;
			return;
		}
		render_context.render_count = render_context.opacity_list.len() + render_context.transparent_list.len();

		// 排序
		render_context.transparent_list.sort_by(|id1, id2| {
			let obj1 = &basemut.render_objs[*id1];
			let obj2 = &basemut.render_objs[*id2];
			if obj1.depth.is_nan() {
				log::info!("id1 id nan: {:?}, {:?}", obj1.context, obj2.vs_name);
			}
			if obj2.depth.is_nan() {
				log::info!("id2 id nan: {:?}, {:?}", obj2.context, obj2.vs_name);
			}	
			match obj1.depth.partial_cmp(&obj2.depth) {
				Some(r) => r,
				None => {
					log::info!("depth fail nan: {:?}, {:?}", obj1.depth, obj2.depth);
					panic!();
				}
			}
		});
		render_context.opacity_list.sort_by(|id1, id2| {
			let obj1 = &basemut.render_objs[*id1];
			let obj2 = &basemut.render_objs[*id2];
			if obj1.depth.is_nan() {
				log::info!("id1 id nan: {:?}, {:?}", obj1.context, obj2.vs_name);
			}
			if obj2.depth.is_nan() {
				log::info!("id2 id nan: {:?}, {:?}", obj2.context, obj2.vs_name);
			}
			if obj1.program.is_none() {
				log::info!("obj1 program is_none: {:?}", obj1.vs_name);
			}
			if obj2.program.is_none() {
				log::info!("obj2 program is_none: {:?}", obj2.vs_name);
			}

			match (obj1.program.as_ref().unwrap().item.index)
			.partial_cmp(&(obj2.program.as_ref().unwrap().item.index)) {
				Some(r) => r,
				None => {
					log::info!("program fail: {:?}, {:?}, {:?}, {:?}", obj1.program.as_ref().unwrap().item, obj2.program.as_ref().unwrap().item, obj1.vs_name, obj2.vs_name);
					panic!();
				}
			}
		});

		let render_context = render_contexts.get(id).unwrap();
		let ( 
			target,
			project_ubo,
			view_ubo,
			clear_color,
		) = (
			match render_context.render_target {
				Some(r) => basemut.dyn_atlas_set.get_target(r),
				None => match &render_begin.1 {
					Some(r) => Some(&**r),
					None => None,
				}
			},
			match &render_context.projection_matrix_ubo {
				Some(r) => r,
				None => &self.projection_matrix_ubo,
			},
			match &render_context.view_matrix_ubo {
				Some(r) => r,
				None => &self.view_matrix_ubo,
			},
			match render_context.render_target {
				Some(_r) => Some((OrderedFloat::from(0.0), OrderedFloat::from(0.0), OrderedFloat::from(0.0), OrderedFloat::from(0.0))),
				None => render_begin.0.clear_color.clone()
			}
		);

		let begine = RenderBeginDesc {
			viewport: viewport,
			scissor: scissor,
			clear_color: clear_color,
			clear_depth: render_begin.0.clear_depth.clone(),
			clear_stencil: render_begin.0.clear_stencil.clone(),
		};


		if scissor.2 == 0 {
			return;
		}

		// log::info!("render context============================={}", id);
	
		// log::info!("begine====={:?}, {:?}, id:{}, dirty_rect: {:?}", begine.viewport, begine.scissor, id, dirty_rect);
		basemut.engine.gl.render_begin(target, &begine, match self.render_count {0 => true, _=> {basemut.engine.gl.render_reset_geometry();false}});
		// log::info!("direct: {:?}, {:?}, {:?}", dirty_rect, self.opacity_list, self.transparent_list);
		for id in render_context.opacity_list.iter() {
			let obj = &basemut.render_objs[*id];
			// log::info!("render obj=============={:?}, {}, {:?}", id, obj.context, obj.vs_name);
			self.render(&basemut.engine.gl, obj, Some(project_ubo), Some(view_ubo), *id);
		}
		for id in render_context.transparent_list.iter() {
			let obj = &basemut.render_objs[*id];
			// log::info!("render obj=============={:?}, {}, {:?}", id, obj.context, obj.vs_name);
			self.render(&basemut.engine.gl, obj, Some(project_ubo), Some(view_ubo), *id);
		};
		self.render_count += 1;
	}

	fn list_render_obj<'b>(
		&mut self, 
		base: &RenderBase<'b>, 
		basemut: &mut RenderBaseMut<'b, C>, 
		render_context: &mut RenderContext,  
		index: usize, 
		parent_target: usize,
	) -> usize {
		let item = &basemut.render_objs[index];
		if item.visibility == true {
			if let Some(post_process) = &item.post_process {
				let post_mut = unsafe{ &mut *( &**post_process as *const PostProcessContext as usize as *mut PostProcessContext)};
				return self.render_post_process(base, basemut, index, parent_target, post_mut);
				;
			}
			if item.is_opacity == true {
				render_context.opacity_list.push(index);
			} else {
				render_context.transparent_list.push(index);
			}

			// log::info!("list===================index: {}, context: {}", index, item.context);

			if item.paramter.get_single_uniform("context_id").is_some() {
				let tt = item.context;
				item.paramter.set_single_uniform(
					"context_id", 
					UniformValue::Int1(tt as i32),
				);
			}
		}
		return 0;
	}

	// if dirty_view_rect.4 == true || dirty_view_rect.3 - dirty_view_rect.1 <= 0.0 {
}

pub struct RenderBase<'a> {
	pub idtree: &'a IdTree,
	pub dirty_rect: &'a Aabb2,
	pub render_map: &'a NodeRenderMap,
	pub is_reset: bool, 
	pub render_begin: &'a RenderBegin, 
	pub octree: &'a SingleCaseImpl<Oct>,
	pub render_contexts: &'a MultiCaseImpl<Node, RenderContext>,
	pub visibilitys: &'a MultiCaseImpl<Node, Visibility>,
	pub opacitys: &'a MultiCaseImpl<Node, Opacity>
}
pub struct RenderBaseMut<'a, C: HalContext + 'static> {
	dyn_atlas_set:&'a mut DynAtlasSet,
	render_objs: &'a mut RenderObjs, 
	engine: &'a mut Engine<C>,
}



// pub struct RenderBase<'a, C: HalContext + 'static> {
// 	pub render_objs: &'a mut RenderObjs,
// 	pub idtree: &'a IdTree,
// 	pub dirty_rect: &'a Aabb2,
// 	pub render_map: &'a NodeRenderMap,
// 	pub dyn_atlas_set:&'a mut DynAtlasSet, 
// 	pub engine: &'a mut Engine<C>, 
// 	pub is_reset: bool, 
// 	pub render_begin: &'a RenderBegin, 
// 	pub octree: &'a SingleCaseImpl<Oct>,
// 	pub render_contexts: &'a MultiCaseImpl<Node, RenderContext>,
// }

#[derive(Clone)]
pub struct RenderBase1<'a> {
	// pub post_process_context: &'a mut PostProcessContext,
	pub render_context: &'a RenderContext,
	pub parent_target: usize
}

impl<'a, C: HalContext + 'static> Runner<'a> for RenderSys<C> {
    type ReadData = (
		&'a SingleCaseImpl<Oct>,
		&'a SingleCaseImpl<IdTree>,
		&'a SingleCaseImpl<NodeRenderMap>,
		&'a MultiCaseImpl<Node, Visibility>,
		&'a MultiCaseImpl<Node, Opacity>,
	);
    type WriteData = (
		&'a mut SingleCaseImpl<PreRenderList>,
		&'a mut SingleCaseImpl<Share<RefCell<DynAtlasSet>>>,
		&'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<ShareEngine<C>>,
		// &'a mut SingleCaseImpl<Statistics>,
		&'a mut SingleCaseImpl<DirtyViewRect>,
		&'a mut SingleCaseImpl<RenderBegin>,
		&'a mut MultiCaseImpl<Node, RenderContext>,
	);
	
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
		let (
			octree, 
			idtree,
			render_map,
			visibilitys,
			opacitys) = read;
        let (
			pre_render_list, 
			dyn_atlas_set, 
			render_objs, engine, dirty_view_rect, render_begin, render_contexts,) = write;
		
		let mut dyn_atlas_set = dyn_atlas_set.borrow_mut();
		if pre_render_list.len() > 0 {
			for item in pre_render_list.iter_mut() {
				// 更新Program
				let render_obj = &mut item.obj;
				if render_obj.program.is_none() {

					let paramter = &render_obj.paramter;
					let ubos = paramter.get_layout();
					let mut uniforms = Vec::with_capacity(ubos.len());
					for ubo in ubos.iter() {
						uniforms.push(paramter.get_value(ubo).unwrap().get_layout());
					}

					let uniform_layout = UniformLayout {
						ubos: ubos,
						uniforms: uniforms.as_slice(),
						single_uniforms: paramter.get_single_uniform_layout(),
						textures: paramter.get_texture_layout(),
					};

					let program = engine.create_program(
						render_obj.vs_name.get_hash() as u64,
						render_obj.fs_name.get_hash() as u64,
						&render_obj.vs_name,
						&*render_obj.vs_defines,
						&render_obj.fs_name,
						&*render_obj.fs_defines,
						render_obj.paramter.as_ref(),
					);

					render_obj.program = Some(program);
				}
				
				// 渲染（每次渲染只有一个obj， 多个obj， TODO）
				let index = item.index;
				let rect = dyn_atlas_set.get_rect(index).unwrap();
				let clear_rect = dyn_atlas_set.get_rect_with_border(index).unwrap();
				let v = (rect.mins.x as i32, rect.mins.y as i32, (rect.maxs.x - rect.mins.x) as i32, (rect.maxs.y - rect.mins.y) as i32);
				let s = (clear_rect.mins.x as i32, clear_rect.mins.y as i32, (clear_rect.maxs.x - clear_rect.mins.x) as i32, (clear_rect.maxs.y - clear_rect.mins.y) as i32);
				// 不需要深度
				let begine = RenderBeginDesc {
					viewport: v.clone(),
					scissor: s,
					clear_color: Some((OrderedFloat::from(0.0), OrderedFloat::from(0.0), OrderedFloat::from(0.0), OrderedFloat::from(0.0))),
					clear_depth: render_begin.0.clear_depth.clone(),
					clear_stencil: render_begin.0.clear_stencil.clone(),
				};
				engine.gl.render_begin(
					Some(dyn_atlas_set.get_target(index).unwrap()), 
					&begine, 
					match self.render_count {
						0 => true,
						_=> {engine.gl.render_reset_geometry();false}
					}
				);
				self.render(&engine.gl, render_obj, None, None, 0);
				self.render_count += 1;
			}
			pre_render_list.clear();
			self.dirty = true;
		}

        for id in self.program_dirtys.iter() {
            let render_obj = match render_objs.get_mut(*id) {
                Some(render_obj) => render_obj,
                None => continue,
            };

            let program = engine.create_program(
                render_obj.vs_name.get_hash() as u64,
                render_obj.fs_name.get_hash() as u64,
                &render_obj.vs_name,
                &*render_obj.vs_defines,
                &render_obj.fs_name,
                &*render_obj.fs_defines,
                render_obj.paramter.as_ref(),
            );

            render_obj.program = Some(program);
            render_obj.program_dirty = false;
        }

        self.program_dirtys.clear();

        if self.dirty == false {
            return;
        }

		// log::info!("self.dirty: {:?}, root_indexs.1:{:?}", self.dirty, root_indexs.1);

		// statistics.drawcall_times = 0;
		// let gl = &engine.gl;
		// let mut del = Vec::new();

		// 脏区域
		let mut dirty_rect = Aabb2::new(
			Point2::new(dirty_view_rect.0 as f32, dirty_view_rect.1 as f32), Point2::new(dirty_view_rect.2 as f32, dirty_view_rect.3 as f32)
		);
		let is_reset = dirty_view_rect.4; // 是否全部重新渲染
		if !is_reset {
			dirty_rect.mins.x = dirty_rect.mins.x.floor();
			dirty_rect.mins.y = dirty_rect.mins.y.floor();
			dirty_rect.maxs.x = dirty_rect.maxs.x.ceil();
			dirty_rect.maxs.y = dirty_rect.maxs.y.ceil();
		}
		// if dirty_view_rect.4 == true {
		// 	dirty_rect = Aabb2::new(
		// 		Point2::new(0.0, 0.0), Point2::new(578.8295, 937.0)
		// 	);
		// 	is_reset = false;
		// }
		let render_begin_desc = &render_begin.0;
		let render_contexts1 = unsafe{ &mut *(render_contexts as *const MultiCaseImpl<Node, RenderContext> as usize as *mut MultiCaseImpl<Node, RenderContext>) };

		let root = 1;

		
		let render_context_root = &mut render_contexts[root];
		
		let base = RenderBase {
			render_map,
			idtree,
			dirty_rect: &dirty_rect,
			octree,
			is_reset,
			render_begin,
			render_contexts: render_contexts1,
			visibilitys,
			opacitys,
		};
		let mut basemut = RenderBaseMut{
			render_objs,
			dyn_atlas_set: &mut dyn_atlas_set,
			engine,
		};
		// let mut dyn_atlas_set = dyn_atlas_set.borrow_mut();
		
		self.render_context(&base, &mut basemut, 1,  0, render_context_root);
		
		if self.render_count > 0 { // 如果进行了一些渲染，则需要重置状态
			engine.gl.render_end();
			self.render_count = 0;
		}

		let viewport = render_begin_desc.viewport;
		// if dirty_view_rect.4 == true {
			dirty_view_rect.0 = viewport.2 as f32;
			dirty_view_rect.1 = viewport.3 as f32;
			dirty_view_rect.2 = 0.0;
			dirty_view_rect.3 = 0.0;
			dirty_view_rect.4 = false;
		// }
		self.is_update_context_texture = false;
		self.dirty = false;
    }
}


impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, CreateEvent> for RenderSys<C> {
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &Event, _: Self::ReadData, render_objs: Self::WriteData) {
        self.dirty = true;
        let obj = &mut render_objs[event.id];
        if obj.is_opacity == false {
            self.transparent_dirty = true;
        } else {
            self.opacity_dirty = true;
        }
        self.program_dirtys.push(event.id);
        obj.program_dirty = true;
    }
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, ModifyEvent> for RenderSys<C> {
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &Event, _: Self::ReadData, render_objs: Self::WriteData) {
        self.dirty = true;
		if event.id == 0 && event.field == "context" {
			self.is_update_context_texture = true;
		}
		
        let obj = match render_objs.get_mut(event.id) {
            Some(r) => r,
            None => return, // obj可能不存在
        };
        match event.field {
            "depth" => {
                if obj.is_opacity == false {
                    self.transparent_dirty = true;
                }
            }
            "program_dirty" => {
                if obj.is_opacity == true {
                    self.opacity_dirty = true;
                }
                if obj.program_dirty == false {
                    self.program_dirtys.push(event.id);
                    obj.program_dirty = true;
                }
            }
            "is_opacity" => {
                self.opacity_dirty = true;
                self.transparent_dirty = true;
            }
            "visibility" => {
                if obj.is_opacity {
                    self.opacity_dirty = true;
                } else {
                    self.transparent_dirty = true;
                }
            }
            _ => (),
        }
    }
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, DeleteEvent> for RenderSys<C> {
    type ReadData = &'a SingleCaseImpl<RenderObjs>;
    type WriteData = ();
    fn listen(&mut self, event: &Event, render_objs: Self::ReadData, _: Self::WriteData) {
        self.dirty = true;
        let obj = &render_objs[event.id];
        if obj.is_opacity == false {
            self.transparent_dirty = true;
        } else {
            self.opacity_dirty = true;
        }
    }
}


impl<'a, C: HalContext + 'static> SingleCaseListener<'a, ProjectionMatrix, ModifyEvent>
    for RenderSys<C>
{
    type ReadData = &'a SingleCaseImpl<ProjectionMatrix>;
    type WriteData = ();
    fn listen(
        &mut self,
        _event: &Event,
        projection_matrix: Self::ReadData,
		_: Self::WriteData,
    ) {
        let slice: &[f32] = projection_matrix.0.as_slice();
        let project_matrix_ubo =
            ProjectMatrixUbo::new(UniformValue::MatrixV4(Vec::from(slice)));
        self.projection_matrix_ubo = Share::new(project_matrix_ubo);
		self.dirty = true;
    }
}

fn render1<C: HalContext + 'static>(
	gl: &C,
	geo: &HalGeometry,
	paramter: &Share<dyn ProgramParamter>,

	state: &State,
	program: &HalProgram,
	project_matrix: Option<&Share<dyn UniformBuffer>>,
	view_matrix: Option<&Share<dyn UniformBuffer>>,
) -> Result<(), String> {
	gl.render_set_program(program);
	if let Some(project_matrix) = project_matrix {
		paramter.set_value("projectMatrix", project_matrix.clone());
	}
	if let Some(view_matrix) = view_matrix  {
		paramter.set_value("viewMatrix", view_matrix.clone());
	}

    // statistics.drawcall_times += 1;
    gl.render_set_state(&state.bs, &state.ds, &state.rs, &state.ss);
    gl.render_draw(geo, paramter)?;
	Ok(())
}
// fn render<C: HalContext + 'static>(gl: &C, obj: &RenderObj, statistics: &mut Statistics) {
//     let geometry = match &obj.geometry {
//         None => return,
//         Some(g) => g,
//     };
//     // #[cfg(feature = "performance")]
//     statistics.drawcall_times += 1;
//     gl.render_set_program(obj.program.as_ref().unwrap());
//     gl.render_set_state(&obj.state.bs, &obj.state.ds, &obj.state.rs, &obj.state.ss);
//     gl.render_draw(&geometry.geo, &obj.paramter);
// }

struct OpacityOrd<'a>(&'a RenderObj, usize);

impl<'a> PartialOrd for OpacityOrd<'a> {
    fn partial_cmp(&self, other: &OpacityOrd<'a>) -> Option<Ordering> {
        self.0
            .program
            .as_ref()
            .unwrap()
            .item
            .index
            .partial_cmp(&other.0.program.as_ref().unwrap().item.index)
    }
}

impl<'a> PartialEq for OpacityOrd<'a> {
    fn eq(&self, other: &OpacityOrd<'a>) -> bool {
        self.0.program.as_ref().unwrap().item.index.eq(&other
            .0
            .program
            .as_ref()
            .unwrap()
            .item
            .index)
    }
}

impl<'a> Eq for OpacityOrd<'a> {}

impl<'a> Ord for OpacityOrd<'a> {
    fn cmp(&self, other: &OpacityOrd<'a>) -> Ordering {
        let r = self.partial_cmp(&other).unwrap();
        r
    }
}

struct TransparentOrd<'a>(&'a RenderObj, usize);

impl<'a> PartialOrd for TransparentOrd<'a> {
    fn partial_cmp(&self, other: &TransparentOrd<'a>) -> Option<Ordering> {
        (self.0.depth + other.0.depth_diff).partial_cmp(&(other.0.depth + other.0.depth_diff))
    }
}

impl<'a> PartialEq for TransparentOrd<'a> {
    fn eq(&self, other: &TransparentOrd<'a>) -> bool {
        (self.0.depth + other.0.depth_diff).eq(&(other.0.depth + other.0.depth_diff))
    }
}

impl<'a> Eq for TransparentOrd<'a> {}

impl<'a> Ord for TransparentOrd<'a> {
    fn cmp(&self, other: &TransparentOrd<'a>) -> Ordering {
        let r = self.partial_cmp(&other).unwrap();
        r
    }
}

impl_system! {
    RenderSys<C> where [C: HalContext + 'static],
    true,
    {
        SingleCaseListener<RenderObjs, CreateEvent>
        SingleCaseListener<RenderObjs, ModifyEvent>
        SingleCaseListener<RenderObjs, DeleteEvent>
		SingleCaseListener<ProjectionMatrix, ModifyEvent>
    }
}

#[cfg(test)]
use crate::component::user::Vector4;

#[test]
fn test() {
	let rect = Aabb2::new(Point2::new(0.0, 0.0), Point2::new(46.0, 46.0));
	let content_box = Aabb2::new(Point2::new(133.0, 44.0), Point2::new(179.0, 46.0));
	
	let project_martix = ProjectionMatrix(ProjectionMatrix::new(
		rect.maxs.x - rect.mins.x,
		rect.maxs.y - rect.mins.y,
		-Z_MAX - 1.0,
		Z_MAX + 1.0,
	).0 * get_render_project_matrix(&content_box));
	println!("{:?}", project_martix.0.as_slice());

	
   	let ortho = Orthographic3::new(133.0, 46.0, 46.0, 44.0, -Z_MAX - 1.0, Z_MAX + 1.0);
	let project_martix1 = 	WorldMatrix(Matrix4::from(ortho), false);
	println!("project_martix 1 {:?}", project_martix1.as_slice());

	println!("0 project_martix 1 {:?}", (&project_martix1) * Vector4::new(0.0, 0.0, 0.0, 1.0));


	let mm = WorldMatrix(Matrix4::new(
		44.6097, 0.0, 0.0, 133.8290,
		0.0, 44.6777, 0.0, 44.6777,
		0.0, 0.0, 1.0, 0.0,
		0.0, 0.0, 0.0, 1.0
	), false);

	println!("1 project_martix 1 {:?}", &((&project_martix1) * mm) * Vector4::new(0.0, 0.0, 0.0, 1.0));

	let wm = WorldMatrix(Matrix4::new(
		112.2750, 0.0, 0.0, 337.0,
		0.0, 156.7172, 0.0, 305.0,
		0.0, 0.0, 1.0, -0.3275,
		0.0,0.0,0.0,1.0,
	), false);

	let pm1 = WorldMatrix(Matrix4::new(
		0.5000, 0.0, 0.0, -1.0,
  	    0.0, -0.5000, 0.0, 1.0, 
		0.0, 0.0, -0.0000, 0.0, 
		0.0, 0.0, 0.0, 1.0), false);
	
	let v1 = Vector4::new(0.0, 0.0,0.0,1.0);
	let v2 = Vector4::new(4.0, 4.0,0.0,1.0);
	let v3 = Vector4::new(1.0, 1.0,0.0,1.0);
	let v4 = Vector4::new(0.0, 1.0,0.0,1.0);

	let m = wm.clone();
	// println!("{:?}, {:?}, {:?}, {:?}", m.0 * v1, m.0 * v2,m.0 * v3,m.0 * v4);
	println!("{:?}", pm1.0 * v1);
	
	let m = project_martix.0 * wm;
	println!("{:?}", pm1.0 * v2);

}