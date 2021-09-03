/**
 *  渲染， 将渲染对象按照透明与不透明分类， 先渲染不透明物体， 再渲染透明物体， 不透明物体按照渲染管线的顺序渲染， 透明物体按照物体的深度顺序渲染
 */
use std::cmp::Ordering;
use std::default::Default;

use bevy_ecs::prelude::{Res, ResMut, In};
use hal_core::*;
use crate::component::user::{Aabb2, Point2};

use crate::render::engine::ShareEngine;
use crate::single::{RenderBegin, RenderObj, RenderObjs, Statistics, DirtyViewRect, Oct};
use crate::util::event::RenderObjEvent;

pub fn renderobjs_listen(
	e: In<RenderObjEvent>,
	mut render_objs: ResMut<RenderObjs>,
	mut local: ResMut<RenderSys>,
) {
	let event = &e.0;
	local.dirty = true;
	let obj = match render_objs.get_mut(event.id) {
		Some(r) => r,
		None => return, // obj可能不存在
	};

	match event.field {
		"depth" => {
			if obj.is_opacity == false {
				local.transparent_dirty = true;
			}
		}
		"program_dirty" => {
			if obj.is_opacity == true {
				local.opacity_dirty = true;
			}
			if obj.program_dirty == false {
				local.program_dirtys.push(event.id);
				obj.program_dirty = true;
			}
		}
		"is_opacity" => {
			local.opacity_dirty = true;
			local.transparent_dirty = true;
		}
		"visibility" => {
			if obj.is_opacity {
				local.opacity_dirty = true;
			} else {
				local.transparent_dirty = true;
			}
		}
		_ => (),
	}
}

pub fn draw<'a, C: HalContext + 'static>(
	octree: Res<Oct>,
	mut render_objs: ResMut<RenderObjs>,
	mut engine: ResMut<ShareEngine<C>>,
	mut dirty_view_rect: ResMut<DirtyViewRect>,
	render_begin: Res<RenderBegin>,
	mut statistics: ResMut<Statistics>,
	// mut renderobjs_event_render: EventReader<RenderObjEvent>,
	mut local: ResMut<RenderSys>,
) {
	// for e in renderobjs_event_render.iter() {
	// 	if local.opacity_dirty || local.transparent_dirty {
	// 		break;
	// 	}

	if local.dirty == false {
		return;
	}
	// let mut program_dirtys = std::mem::replace(&mut render_objs.program_dirtys, Vec::new());

	for id in local.program_dirtys.iter() {
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
	local.program_dirtys.clear();

	if local.dirty == false {
		return;
	}
	local.dirty = false;

	// #[cfg(feature = "performance")]
	// js! {
	//     __time = performance.now();
	// }

	let mut visibility = Vec::new();
	if local.transparent_dirty && local.opacity_dirty {
		local.opacity_list.clear();
		local.transparent_list.clear();
		for item in render_objs.iter() {
			if item.1.visibility == true {
				if item.1.is_opacity == true {
					local.opacity_list.push(item.0);
				} else {
					local.transparent_list.push(item.0);
				}
			} else {
				visibility.push(1);
			}
		}
		local.transparent_list.sort_by(|id1, id2| {
			let obj1 = &render_objs[*id1];
			let obj2 = &render_objs[*id2];
			obj1.depth.partial_cmp(&obj2.depth).unwrap()
		});
		local.opacity_list.sort_by(|id1, id2| {
			let obj1 = &render_objs[*id1];
			let obj2 = &render_objs[*id2];
			(obj1.program.as_ref().unwrap().item.index)
				.partial_cmp(&(obj2.program.as_ref().unwrap().item.index))
				.unwrap()
		});
	} else if local.transparent_dirty {
		local.transparent_list.clear();
		for item in render_objs.iter() {
			if item.1.visibility == true {
				if item.1.is_opacity != true {
					local.transparent_list.push(item.0);
				}
			} else {
				visibility.push(1);
			}
		}
		local.transparent_list.sort_by(|id1, id2| {
			let obj1 = &render_objs[*id1];
			let obj2 = &render_objs[*id2];
			obj1.depth.partial_cmp(&obj2.depth).unwrap()
		});
	} else if local.opacity_dirty {
		local.opacity_list.clear();
		for item in render_objs.iter() {
			if item.1.visibility == true {
				if item.1.is_opacity == true {
					local.opacity_list.push(item.0);
				}
			} else {
				visibility.push(1);
			}
		}
		local.opacity_list.sort_by(|id1, id2| {
			let obj1 = &render_objs[*id1];
			let obj2 = &render_objs[*id2];
			obj1.program
				.as_ref()
				.unwrap()
				.item
				.index
				.partial_cmp(&obj2.program.as_ref().unwrap().item.index)
				.unwrap()
		});
	}

	let target = match &render_begin.1 {
		Some(r) => Some(&**r),
		None => None,
	};
	let gl = &engine.gl;
	let render_begin_desc = &render_begin.0;
	// #[cfg(feature = "performance")]
	statistics.drawcall_times = 0;
	let viewport = render_begin_desc.viewport;
	// 如果局部视口就是最大视口，则按最大视口来渲染
	if dirty_view_rect.4 == true || dirty_view_rect.3 - dirty_view_rect.1 <= 0.0 {
		
		gl.render_begin(target, &render_begin_desc);
		dirty_view_rect.4 = false;
		for id in local.opacity_list.iter() {
			let obj = &render_objs[*id];
			render(gl, obj, &mut statistics);
		}
		for id in local.transparent_list.iter() {
			let obj = &render_objs[*id];
			render(gl, obj, &mut statistics);
		}
	} else {
		// let root_matrix = &world_matrixs[1];
		// // 将渲染视口(这个视口的原点是根节点的0,0点)，转换到-1~1范围，再将其转换为裁剪区域（以渲染目标的左上角为原点）
		// let left_top = &(projection_matrix.0).0 * &root_matrix.0 * &Vector4::new(dirty_view_rect.0 as f32, dirty_view_rect.1 as f32, 0.0, 0.0);
		// let right_bottom = &(projection_matrix.0).0 * &root_matrix.0 * &Vector4::new(dirty_view_rect.2 as f32, dirty_view_rect.3 as f32, 0.0, 0.0);
		// let scissor_left_top = (
		// 	((left_top.x + 1.0)/2.0 * viewport.2 as f32) as i32,
		// 	((1.0 - (left_top.y + 1.0)/2.0) * viewport.3 as f32) as i32,);
		
		
		// let scissor = (
		// 	scissor_left_top.0,
		// 	scissor_left_top.1,
		// 	((right_bottom.x + 1.0)/2.0 * viewport.2 as f32) as i32 - scissor_left_top.0,
		// 	((1.0 - (right_bottom.y + 1.0)/2.0) * viewport.3 as f32) as i32 - scissor_left_top.1,
		// );
		let scissor = (
			render_begin.0.viewport.0 + dirty_view_rect.0.floor() as i32,
			render_begin.0.viewport.1 + render_begin.0.viewport.3 - dirty_view_rect.3.ceil() as i32,
			(dirty_view_rect.2.ceil() - dirty_view_rect.0.floor()) as i32,
			(dirty_view_rect.3.ceil() - dirty_view_rect.1.floor()) as i32,
		);

		gl.render_begin(target, &RenderBeginDesc{
			viewport: viewport.clone(),
			scissor: scissor,
			clear_color: render_begin_desc.clear_color.clone(),
			clear_depth: render_begin_desc.clear_depth.clone(),
			clear_stencil: render_begin_desc.clear_stencil.clone(),
		});

		// 视口的Aabb，用于剔除视口之外的渲染对象
		let viewPortAabb = Aabb2::new(
			Point2::new(dirty_view_rect.0 as f32, dirty_view_rect.1 as f32), Point2::new(dirty_view_rect.2 as f32, dirty_view_rect.3 as f32)
		);

		for id in local.opacity_list.iter() {
			let obj = &render_objs[*id];
			if let None = octree.get(obj.context.id() as usize) {
				log::info!("obj.context: {:?}",obj.context);
			}
			// 如果相交才渲染
			if is_intersect(&viewPortAabb, &unsafe { octree.get_unchecked(obj.context.id() as usize) }.0) {
				render(gl, obj, &mut statistics);
			}
		}
		for id in local.transparent_list.iter() {
			let obj = &render_objs[*id];
			if let None = octree.get(obj.context.id() as usize) {
				log::info!("obj.context: {:?}",obj.context);
			}
			if is_intersect(&viewPortAabb, &unsafe { octree.get_unchecked(obj.context.id() as usize) }.0) {
				render(gl, obj, &mut statistics);
			}
		}
	}

	gl.render_end();
	
	dirty_view_rect.0 = viewport.2 as f32;
	dirty_view_rect.1 = viewport.3 as f32;
	dirty_view_rect.2 = 0.0;
	dirty_view_rect.3 = 0.0;

}

fn render<C: HalContext + 'static>(gl: &C, obj: &RenderObj, statistics: &mut Statistics) {
    let geometry = match &obj.geometry {
        None => return,
        Some(g) => g,
    };
    // #[cfg(feature = "performance")]
    statistics.drawcall_times += 1;
    gl.render_set_program(obj.program.as_ref().unwrap());
    gl.render_set_state(&obj.state.bs, &obj.state.ds, &obj.state.rs, &obj.state.ss);
    gl.render_draw(&geometry.geo, &obj.paramter);
}


pub struct RenderSys {
    program_dirtys: Vec<usize>,
    transparent_dirty: bool,
    opacity_dirty: bool,
    pub dirty: bool,
    opacity_list: Vec<usize>,
    transparent_list: Vec<usize>,
	// program_dirtys: Vec<usize>,
}

impl Default for RenderSys{
    fn default() -> Self {
        Self {
            program_dirtys: Vec::default(),
            transparent_dirty: false,
            opacity_dirty: false,
            dirty: false,
            opacity_list: Vec::new(),
            transparent_list: Vec::new(),
            // transparent_list: BTreeMap::new(),
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

