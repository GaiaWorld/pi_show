/**
 *  渲染， 将渲染对象按照透明与不透明分类， 先渲染不透明物体， 再渲染透明物体， 不透明物体按照渲染管线的顺序渲染， 透明物体按照物体的深度顺序渲染
 */
use std::cmp::Ordering;
use std::default::{Default};
use std::marker::PhantomData;

use ecs::{
    CreateEvent, DeleteEvent, ModifyEvent, Runner, SingleCaseImpl, SingleCaseListener, MultiCaseImpl,
};
use ecs::monitor::{Event};
use hal_core::*;
use ordered_float::OrderedFloat;
use share::Share;
use crate::Z_MAX;
use crate::component::user::{Aabb2, Matrix4, Point2, Vector3};
use crate::component::calc::{WorldMatrix, RenderContext, ProjectMatrixUbo};
use crate::entity::Node;

use crate::render::engine::{Engine, ShareEngine};
use crate::render::res::{SamplerRes};
use crate::single::{DirtyViewRect, IdTree, NodeRenderMap, Oct, ProjectionMatrix, RenderBegin, RenderObj, RenderObjs, Statistics, dyn_texture::DynAtlasSet};
use crate::system::util::{cal_uv_hash, create_uv_buffer};

pub struct RenderSys<C: HalContext + 'static> {
    program_dirtys: Vec<usize>,
    transparent_dirty: bool,
    opacity_dirty: bool,
    pub dirty: bool,
	default_sampler: Share<SamplerRes>,
	is_update_context_texture: bool,
	pub view_matrix_ubo: Share<dyn UniformBuffer>,
	pub projection_matrix_ubo: Share<dyn UniformBuffer>,
	pub render_count: usize,
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

		let default_sampler = engine.create_sampler_res(sm);
		Self {
			program_dirtys: Vec::default(),
			transparent_dirty: false,
			opacity_dirty: false,
			dirty: false,
			is_update_context_texture: false,
			default_sampler: default_sampler,
			view_matrix_ubo: Share::new(ProjectMatrixUbo::new(
				UniformValue::MatrixV4(vec![1.0,0.0,0.0,0.0, 0.0,1.0,0.0,0.0, 0.0,0.0,1.0,0.0, 0.0,0.0,0.0,1.0]),
			)),
			projection_matrix_ubo: Share::new(ProjectMatrixUbo::new(
				UniformValue::MatrixV4(Vec::from(project_matrix.0.as_slice())),
			)),
			render_count: 0,
			marker: PhantomData,
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

// 计算两个aabb的交集
#[inline]
fn intersect(a: &Aabb2, b: &Aabb2) -> Option<Aabb2> {
	let r = Aabb2::new(
		Point2::new(a.mins.x.max(b.mins.x), a.mins.y.max(b.mins.y)),
		Point2::new(a.maxs.x.min(b.maxs.x), a.maxs.y.min(b.maxs.y))
	);
	if r.maxs.x <= r.mins.x || r.maxs.y <= r.mins.y {
		return None
	}
	Some(r)
}


fn list_render_obj(render_context: &mut RenderContext, item: &RenderObj, index: usize) {
	if item.visibility == true {
		if item.is_opacity == true {
			render_context.opacity_list.push(index);
		} else {
			render_context.transparent_list.push(index);
		}
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
impl<'a, C: HalContext + 'static>  RenderSys<C> {
	
	fn recursive_list_by_intersect(&mut self, head: usize, render_map: &NodeRenderMap, render_objs: &mut RenderObjs, idtree: &IdTree, render_context: &mut RenderContext, render_contexts: &mut MultiCaseImpl<Node, RenderContext>, dirty_rect: &Aabb2, octree: &SingleCaseImpl<Oct>, dyn_atlas_set:&mut DynAtlasSet, engine: &mut Engine<C>, is_reset: bool, render_begin: &RenderBegin, statistics: &mut Statistics, parent_target: usize) {
		let render_contexts1 = unsafe{ &mut *(render_contexts as *const MultiCaseImpl<Node, RenderContext> as usize as *mut MultiCaseImpl<Node, RenderContext>) };
		for (id, node) in idtree.iter(head) {
			let oct = match octree.get(id) {
				Some(r) => r.0,
				None => {
					// log::warn!("render list fail, oct is not exist, id: {}", id);
					return;
				}
			};
			
			if let Some(r) = render_contexts1.get_mut(id) {
				if is_intersect(dirty_rect, oct) {
					list_render_obj(render_context, &render_objs[r.render_obj_index], r.render_obj_index);
				}

				// if let Some(r) = r.render_target {
				// 	if dyn_atlas_set.get_target(target_index).unwrap() == render_context.ren{

				// 	}
				// }
				
				self.render_context(id, render_map, render_objs, idtree, r, render_contexts, dirty_rect, octree, dyn_atlas_set, engine, is_reset, render_begin, statistics, parent_target);
				continue;
			}

			if let Some(r) = render_map.get(id) {
				if is_intersect(dirty_rect, oct) {
					for render_index in r.iter() {
						// if id == 279 {
						// 	log::info!("dirty_rect: {:?}", render_objs[*render_index].visibility);
						// }
						list_render_obj(render_context, &render_objs[*render_index], *render_index);
					}
				}
				
			}
			self.recursive_list_by_intersect(node.children().head, render_map, render_objs, idtree, render_context, render_contexts, dirty_rect, octree, dyn_atlas_set, engine, is_reset, render_begin, statistics, parent_target);
		}
	}

	fn recursive_list(&mut self, head: usize, render_map: &NodeRenderMap, render_objs: &mut RenderObjs, idtree: &IdTree, render_context: &mut RenderContext, render_contexts: &mut MultiCaseImpl<Node, RenderContext>, dirty_rect: &Aabb2, octree: &SingleCaseImpl<Oct>, dyn_atlas_set:&mut DynAtlasSet, engine: &mut Engine<C>, render_begin: &RenderBegin, statistics: &mut Statistics, parent_target: usize) {
		let render_contexts1 = unsafe{ &mut *(render_contexts as *const MultiCaseImpl<Node, RenderContext> as usize as *mut MultiCaseImpl<Node, RenderContext>) };
		for (id, node) in idtree.iter(head) {
			if let Some(r) = render_contexts1.get_mut(id) {
				list_render_obj(render_context, &render_objs[r.render_obj_index], r.render_obj_index);
				self.render_context(id, render_map, render_objs, idtree, r, render_contexts, dirty_rect, octree, dyn_atlas_set, engine, true, render_begin, statistics, parent_target);
				continue;
			}

			if let Some(r) = render_map.get(id) {
				for render_index in r.iter() {
					list_render_obj(render_context, &render_objs[*render_index], *render_index);
				}
			}
			self.recursive_list(node.children().head, render_map, render_objs, idtree, render_context, render_contexts, dirty_rect, octree, dyn_atlas_set, engine, render_begin, statistics, parent_target);
		}
	}

	// 根据脏区域，列出渲染对象
	fn render_context(&mut self, id: usize, render_map: &NodeRenderMap, render_objs: &mut RenderObjs, idtree: &IdTree, render_context: &mut RenderContext, render_contexts: &mut MultiCaseImpl<Node, RenderContext>, dirty_rect: &Aabb2, octree: &SingleCaseImpl<Oct>, dyn_atlas_set:&mut DynAtlasSet, engine: &mut Engine<C>, is_reset: bool, render_begin: &RenderBegin, statistics: &mut Statistics, parent_target: usize) {
		render_context.opacity_list.clear();
		render_context.transparent_list.clear();
		let id1 = id;
		
		let mut render_target_change = false;
		let render_index_context = render_context.render_obj_index;
		
		if let Some(mut render_target) = render_context.render_target {
			let render_obj = &mut render_objs[render_index_context];
			
			if self.is_update_context_texture {
				let content_box = render_context.content_box.clone();
				
				if content_box.maxs.x-content_box.mins.x <= 0.0 || content_box.maxs.y-content_box.mins.y <= 0.0 {
					return;
				}
				let target_index = dyn_atlas_set.update_or_add_rect(render_target,parent_target, content_box.maxs.x-content_box.mins.x,content_box.maxs.y-content_box.mins.y, &mut engine.gl);
				// 如果纹理区域修改了，则重新设置纹理，以及重新更新uv
				if target_index != 0 {
					render_context.render_target = Some(target_index);
					render_target = target_index;
					render_target_change = true;
					

					// 绑定纹理
					render_obj.paramter.set_texture(
						"texture",
						(&engine
							.gl
							.rt_get_color_texture(dyn_atlas_set.get_target(target_index).unwrap(), 0).unwrap(), &self.default_sampler),
					);
				}
			}

			if render_target == 0 {
				return;
			}

			if render_target_change || render_context.geo_change {
				let uv = dyn_atlas_set.get_uv(render_target).unwrap();
				let rect = dyn_atlas_set.get_rect(render_target).unwrap();
				render_context.render_rect = rect;
	
				// 设置uv
				update_geo(render_obj, engine, &uv);
				
				if render_context.geo_change {
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
		}

		let rect = &render_context.render_rect;
		let content_box = &render_context.content_box;
		let viewport = (rect.mins.x as i32, rect.mins.y as i32, (rect.maxs.x- rect.mins.x) as i32, (rect.maxs.y - rect.mins.y) as i32);
			
		// 修改裁剪区域
		let scissor = if is_reset == false && !render_target_change {
			// 渲染不全部重设，并且渲染目标未改变, 则根据脏区域重新计算裁剪区域
			// 渲染区域完全不在脏范围内，不需要重新渲染
			let intersect_rect = match intersect(content_box, &dirty_rect) {
				Some(r) => r,
				None => return,
			};
			// log::info!("content_box====={:?}, {}, {:?}", content_box, id, rect);
			(
				viewport.0 + (intersect_rect.mins.x - content_box.mins.x).floor() as i32,
				viewport.1 - (intersect_rect.maxs.y - content_box.maxs.y).ceil() as i32,
				(intersect_rect.maxs.x.ceil() - intersect_rect.mins.x.floor()) as i32,
				(intersect_rect.maxs.y.ceil() - intersect_rect.mins.y.floor()) as i32,
			)
		} else {
			(rect.mins.x as i32, rect.mins.y as i32, (rect.maxs.x- rect.mins.x) as i32, (rect.maxs.y - rect.mins.y) as i32)
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
			if render_target_change || is_reset == true || (is_intersect(&dirty_rect, oct)) { 
				for render_obj_index in r.iter() {
					if *render_obj_index != render_index_context {
						list_render_obj(render_context,&render_objs[*render_obj_index], *render_obj_index);
					}
				}
			}
		}

		if is_reset {
			self.recursive_list(idtree[id].children().head, render_map, render_objs, idtree, render_context, render_contexts, dirty_rect, octree, dyn_atlas_set, engine, render_begin, statistics, match render_context.render_target {
				Some(r) => r,
				None => 0
			});
		} else {
			self.recursive_list_by_intersect(idtree[id].children().head, render_map, render_objs, idtree, render_context, render_contexts, dirty_rect, octree, dyn_atlas_set, engine, is_reset, render_begin, statistics, match render_context.render_target {
				Some(r) => r,
				None => 0
			});
		}

		// 排序
		render_context.transparent_list.sort_by(|id1, id2| {
			let obj1 = &render_objs[*id1];
			let obj2 = &render_objs[*id2];
			obj1.depth.partial_cmp(&obj2.depth).unwrap()
		});
		render_context.opacity_list.sort_by(|id1, id2| {
			let obj1 = &render_objs[*id1];
			let obj2 = &render_objs[*id2];
			(obj1.program.as_ref().unwrap().item.index)
				.partial_cmp(&(obj2.program.as_ref().unwrap().item.index))
				.unwrap()
		});

		let render_context = render_contexts.get(id).unwrap();
		let ( 
			target,
			project_ubo,
			view_ubo,
			clear_color,
		) = (
			match render_context.render_target {
				Some(r) => dyn_atlas_set.get_target(r),
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
				Some(r) => Some((OrderedFloat::from(0.0), OrderedFloat::from(0.0), OrderedFloat::from(0.0), OrderedFloat::from(0.0))),
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

		// log::info!("begine====={:?}, {:?}, id:{}, dirty_rect: {:?}", begine.viewport, begine.scissor, id, dirty_rect);
		engine.gl.render_begin(target, &begine, match self.render_count {0 => true, _=> false});
		// log::info!("direct: {:?}, {:?}, {:?}", dirty_rect, self.opacity_list, self.transparent_list);
		for id in render_context.opacity_list.iter() {
			let obj = &render_objs[*id];
			render(&engine.gl, obj, statistics, project_ubo, view_ubo);
		}
		for id in render_context.transparent_list.iter() {
			let obj = &render_objs[*id];
			render(&engine.gl, obj, statistics, project_ubo, view_ubo);
		};
		self.render_count += 1;
	}

	// if dirty_view_rect.4 == true || dirty_view_rect.3 - dirty_view_rect.1 <= 0.0 {
}

impl<'a, C: HalContext + 'static> Runner<'a> for RenderSys<C> {
    type ReadData = (
		&'a SingleCaseImpl<Oct>,
		&'a SingleCaseImpl<IdTree>,
		&'a SingleCaseImpl<NodeRenderMap>,
	);
    type WriteData = (
		&'a mut SingleCaseImpl<DynAtlasSet>,
		&'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<ShareEngine<C>>,
		&'a mut SingleCaseImpl<Statistics>,
		&'a mut SingleCaseImpl<DirtyViewRect>,
		&'a mut SingleCaseImpl<RenderBegin>,
		&'a mut MultiCaseImpl<Node, RenderContext>,
	);
	
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
		let (
			octree, 
			idtree,
			render_map,) = read;
        let (dyn_atlas_set, render_objs, engine, statistics, dirty_view_rect, render_begin, render_contexts,) = write;

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

		statistics.drawcall_times = 0;
		// let gl = &engine.gl;
		// let mut del = Vec::new();

		// 脏区域
		let mut dirty_rect = Aabb2::new(
			Point2::new(dirty_view_rect.0 as f32, dirty_view_rect.1 as f32), Point2::new(dirty_view_rect.2 as f32, dirty_view_rect.3 as f32)
		);
		let is_reset = dirty_view_rect.4; // 是否全部重新渲染
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

		self.render_context(1, render_map, render_objs, idtree, render_context_root, render_contexts1, &dirty_rect, octree, dyn_atlas_set, engine, is_reset, render_begin, statistics, 0);
		
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

fn render<C: HalContext + 'static>(
	gl: &C,
	obj: &RenderObj,
	statistics: &mut Statistics,
	project_matrix: &Share<dyn UniformBuffer>,
	view_matrix: &Share<dyn UniformBuffer>,
) {
    let geometry = match &obj.geometry {
        None => return,
        Some(g) => g,
    };
	// if let Some(project_matrix) = project_matrix {
		obj.paramter.set_value("projectMatrix", project_matrix.clone());
	// }
	// if let Some(view_matrix) = view_matrix  {
		obj.paramter.set_value("viewMatrix", view_matrix.clone());
	// }
	
    // #[cfg(feature = "performance")]
    statistics.drawcall_times += 1;
    gl.render_set_program(obj.program.as_ref().unwrap());
    gl.render_set_state(&obj.state.bs, &obj.state.ds, &obj.state.rs, &obj.state.ss);
	// log::info!("render=============");
    gl.render_draw(&geometry.geo, &obj.paramter);
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
