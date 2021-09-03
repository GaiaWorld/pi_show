use bevy_ecs::prelude::{Entity, ResMut, With};
use bevy_ecs::prelude::{In, Query, Res};
/**
 * 背景色渲染对象的构建及其属性设置
*/
use share::Share;
use std::hash::{Hash, Hasher};

use hash::DefaultHasher;
use map::vecmap::VecMap;
use ordered_float::NotNan;

use atom::Atom;
// use ecs::{DeleteEvent, MultiCaseImpl, MultiCaseListener, Runner, SingleCaseImpl};
// use ecs::monitor::NotifyImpl;
use hal_core::*;
use polygon::*;

use crate::component::calc::LayoutR;
use crate::component::calc::Opacity;
use crate::component::calc::*;
use crate::component::user::*;
// use crate::entity::*;
use crate::render::engine::*;
use crate::render::res::*;
use crate::single::*;
use crate::system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};
use crate::system::util::*;
use crate::util::event::{EntityEvent, EventType, ImMessenger, RenderObjEvent};
use crate::util::util::get_or_default;

lazy_static! {
    static ref GRADUAL: Atom = Atom::from("GRADUAL");
}

const DIRTY_TYPE: usize = StyleType::BackgroundColor as usize
	| StyleType::BorderRadius as usize
	| StyleType::Opacity as usize
    | StyleType::Matrix as usize
    | StyleType::Layout as usize;

pub fn bgcolor_listen(
	e: In<EntityEvent<BackgroundColor>>,
	mut local: ResMut<BackgroundColorSys>,
	mut render_objs: ResMut<RenderObjs>,
	mut renderobjs_event_writer: ImMessenger<RenderObjEvent>,/*render_objs*/
) {

	let e = &e.0;
	// 删除已经删除的节点对应的渲染对象
	match e.ty {
		EventType::Delete => local.remove_render_obj(e.id, &mut render_objs, &mut renderobjs_event_writer),
		_ => ()
	}
	// 发送事件，TODO
}

pub fn run<C: HalContext + 'static>(
	query: Query<(
		&LayoutR,
		&ZDepth,
		&WorldMatrix,
		Option<&Transform>,
		&Opacity,
		Option<&BorderRadius>,
		&BackgroundColor,
		&StyleMark), With<BackgroundColor>>,
	unit_quad: Res<UnitQuad>,
	dirty_list: Res<DirtyList>,
	default_transform: Res<Transform>,
	default_state: Res<DefaultState>,
	mut engine: ResMut<ShareEngine<C>>,
	mut render_objs: ResMut<RenderObjs>,
	mut renderobjs_event_writer: ImMessenger<RenderObjEvent>,/*render_objs*/
	mut local: ResMut<BackgroundColorSys>,) {

	if dirty_list.0.len() == 0 {
		return;
	}

	let time = cross_performance::now();
	// let notify = unsafe { &* (render_objs.get_notify_ref() as *const NotifyImpl)} ;
	for id in dirty_list.0.iter() {
		// 监听移除事件， TODO
		// let style_mark = match style_marks.get(*id) {
		// 	Some(r) => r,
		// 	None => {
		// 		self.remove_render_obj(*id, render_objs);
		// 		continue;
		// 	}
		// };
		let (layout, z_depth, world_matrix, transform, opacity, border_radius, color, style_mark) = match query.get(id.clone()){
			Ok(r) => r,
			Err(_r) => continue,
		};
		let transform = get_or_default(transform, &default_transform);

		// 不存在BuckgroundColor关心的脏, 跳过
		if style_mark.dirty[0] & DIRTY_TYPE == 0 {
			continue;
		}

		let dirty = style_mark.dirty[0];
		// 背景颜色脏， 如果不存在BacgroundColor的本地样式和class样式， 删除渲染对象
		let render_index = if style_mark.dirty[0] & StyleType::BackgroundColor as usize != 0 {
			if style_mark.local_style[0] & StyleType::BackgroundColor as usize == 0
				&& style_mark.class_style[0] & StyleType::BackgroundColor as usize == 0
			{
				local.remove_render_obj(*id, &mut render_objs, &mut renderobjs_event_writer);
				continue;
			} else {
				match local.render_map.get_mut(id.id() as usize) {
					Some(r) => *r,
					None => local.create_render_obj(*id, &mut render_objs, &default_state, &mut renderobjs_event_writer),
				}
			}
		} else {
			match local.render_map.get_mut(id.id() as usize) {
				Some(r) => *r,
				None => continue,
			}
		};

		// let color = &background_colors[*id];
		let render_obj = &mut render_objs[render_index];
		// let border_radius = border_radiuses.get(*id);
		// let layout = &layouts[*id];

		// 如果Color脏， 或Opacity脏， 计算is_opacity
		if dirty & StyleType::BackgroundColor as usize != 0
			|| dirty & StyleType::Opacity as usize != 0
		{
			let opacity = opacity.0;
			let is_opacity_old = render_obj.is_opacity;
			render_obj.is_opacity = background_is_opacity(opacity, color);
			if render_obj.is_opacity != is_opacity_old {
				renderobjs_event_writer.send(RenderObjEvent::new_modify(render_index, "is_opacity", 0));
			}
			modify_opacity(render_obj, &default_state);
		}

		let program_dirty = modify_color(
			render_obj,
			color,
			&mut engine,
			dirty,
			layout,
			&unit_quad.0,
			border_radius,
		);

		// program管线脏, 通知
		if program_dirty {
			renderobjs_event_writer.send(RenderObjEvent::new_modify(render_index, "program_dirty", 0));
		}

		// 如果矩阵脏
		if dirty & StyleType::Matrix as usize != 0 || dirty & StyleType::Layout as usize != 0 {
			// let world_matrix = &world_matrixs[*id];
			// let transform = match transforms.get(*id) {
			// 	Some(r) => r,
			// 	None => &default_transform,
			// };
			let depth = z_depth.0;
			let is_unit_geo = match &color.0 {
				Color::RGBA(_) => {
					let radius = cal_border_radius(border_radius, layout);
					let g_b = geo_box(layout);
					if radius.x <= g_b.mins.x {
						true
					} else {
						false
					}
				}
				Color::LinearGradient(_) => false,
			};
			if is_unit_geo {
				modify_matrix(
					render_index,
					create_unit_matrix_by_layout(layout, world_matrix, transform, depth),
					render_obj,
					&mut renderobjs_event_writer,
				);
			} else {
				modify_matrix(
					render_index,
					create_let_top_offset_matrix(
						layout,
						world_matrix,
						transform,
						0.0,
						0.0,
						depth,
					),
					render_obj,
					&mut renderobjs_event_writer,
				);
			}
		}
		renderobjs_event_writer.send(RenderObjEvent::new_modify(render_index, "", 0));
	}
	// if dirty_list.0.len() > 0 {
	// 	log::info!("bg_color======={:?}", cross_performance::now() - time);
	// }
}

#[derive(Default)]
pub struct BackgroundColorSys {
    render_map: VecMap<usize>,
    default_paramter: ColorParamter,
}

impl BackgroundColorSys {
    pub fn new() -> Self {
        BackgroundColorSys {
            render_map: VecMap::default(),
            default_paramter: ColorParamter::default(),
        }
	}
	
	pub fn with_capacity(capacity: usize) -> Self {
        BackgroundColorSys {
            render_map: VecMap::with_capacity(capacity),
            default_paramter: ColorParamter::default(),
        }
    }
}

// // 将顶点数据改变的渲染对象重新设置索引流和顶点流
// impl<'a, C: HalContext + 'static> Runner<'a> for BackgroundColorSys<C> {
//     type ReadData = (
//         &'a MultiCaseImpl<Node, LayoutR>,
//         &'a MultiCaseImpl<Node, ZDepth>,
//         &'a MultiCaseImpl<Node, WorldMatrix>,
//         &'a MultiCaseImpl<Node, Transform>,
//         &'a MultiCaseImpl<Node, Opacity>,
//         &'a MultiCaseImpl<Node, BorderRadius>,
//         &'a MultiCaseImpl<Node, BackgroundColor>,
//         &'a MultiCaseImpl<Node, StyleMark>,
//         &'a SingleCaseImpl<UnitQuad>,
//         &'a SingleCaseImpl<DirtyList>,
//         &'a SingleCaseImpl<DefaultState>,
//     );
//     type WriteData = (
//         &'a mut SingleCaseImpl<RenderObjs>,
//         &'a mut SingleCaseImpl<ShareEngine<C>>,
//     );
//     fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
//         // 没有脏， 跳过
//         if (read.9).0.len() == 0 {
//             return;
//         }

//         let (
//             layouts,
//             z_depths,
//             world_matrixs,
//             transforms,
//             opacitys,
//             border_radiuses,
//             background_colors,
//             style_marks,
//             unit_quad,
//             dirty_list,
//             default_state,
//         ) = read;
//         let (render_objs, engine) = write;

// 		let default_transform = Transform::default();
// 		let notify = unsafe { &* (render_objs.get_notify_ref() as *const NotifyImpl)} ;
//         for id in dirty_list.0.iter() {
//             let style_mark = match style_marks.get(*id) {
//                 Some(r) => r,
//                 None => {
//                     self.remove_render_obj(*id, render_objs);
//                     continue;
//                 }
//             };

//             // 不存在BuckgroundColor关心的脏, 跳过
//             if style_mark.dirty & DIRTY_TYPE == 0 {
//                 continue;
//             }

//             let dirty = style_mark.dirty;
//             // 背景颜色脏， 如果不存在BacgroundColor的本地样式和class样式， 删除渲染对象
//             let render_index = if dirty & StyleType::BackgroundColor as usize != 0 {
//                 if style_mark.local_style & StyleType::BackgroundColor as usize == 0
//                     && style_mark.class_style & StyleType::BackgroundColor as usize == 0
//                 {
//                     self.remove_render_obj(*id, render_objs);
//                     continue;
//                 } else {
//                     match self.render_map.get_mut(*id) {
//                         Some(r) => *r,
//                         None => self.create_render_obj(*id, render_objs, default_state),
//                     }
//                 }
//             } else {
//                 match self.render_map.get_mut(*id) {
//                     Some(r) => *r,
//                     None => continue,
//                 }
//             };

//             let color = &background_colors[*id];
//             let render_obj = &mut render_objs[render_index];
//             let border_radius = border_radiuses.get(*id);
//             let layout = &layouts[*id];

//             // 如果Color脏， 或Opacity脏， 计算is_opacity
//             if dirty & StyleType::BackgroundColor as usize != 0
//                 || dirty & StyleType::Opacity as usize != 0
//             {
// 				let opacity = opacitys[*id].0;
// 				let is_opacity_old = render_obj.is_opacity;
// 				render_obj.is_opacity = background_is_opacity(opacity, color);
// 				if render_obj.is_opacity != is_opacity_old {
// 					notify.modify_event(render_index, "is_opacity", 0);
// 				}
//                 modify_opacity(engine, render_obj, default_state);
//             }

//             let program_dirty = modify_color(
//                 render_obj,
//                 color,
//                 engine,
//                 dirty,
//                 layout,
//                 &unit_quad.0,
//                 border_radius,
//             );

//             // program管线脏, 通知
//             if program_dirty {
//                 notify.modify_event(render_index, "program_dirty", 0);
//             }

//             // 如果矩阵脏
//             if dirty & StyleType::Matrix as usize != 0 || dirty & StyleType::Layout as usize != 0 {
//                 let world_matrix = &world_matrixs[*id];
//                 let transform = match transforms.get(*id) {
//                     Some(r) => r,
//                     None => &default_transform,
//                 };
//                 let depth = z_depths[*id].0;
//                 let is_unit_geo = match &color.0 {
//                     Color::RGBA(_) => {
//                         let radius = cal_border_radius(border_radius, layout);
//                         let g_b = geo_box(layout);
//                         if radius.x <= g_b.mins.x {
//                             true
//                         } else {
//                             false
//                         }
//                     }
//                     Color::LinearGradient(_) => false,
//                 };
//                 if is_unit_geo {
//                     modify_matrix(
//                         render_index,
//                         create_unit_matrix_by_layout(layout, world_matrix, transform, depth),
//                         render_obj,
//                         &notify,
//                     );
//                 } else {
//                     modify_matrix(
//                         render_index,
//                         create_let_top_offset_matrix(
//                             layout,
//                             world_matrix,
//                             transform,
//                             0.0,
//                             0.0,
//                             depth,
//                         ),
//                         render_obj,
//                         &notify,
//                     );
//                 }
//             }
//             notify.modify_event(render_index, "", 0);
//         }
//     }
// }

// impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BackgroundColor, DeleteEvent>
//     for BackgroundColorSys<C>
// {
//     type ReadData = ();
//     type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
//     fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, render_objs: Self::WriteData) {
//         self.remove_render_obj(event.id, render_objs)
//     }
// }

impl BackgroundColorSys {
    #[inline]
    fn remove_render_obj(&mut self, entity: Entity, render_objs: &mut ResMut<RenderObjs>, renderobjs_event_writer: &mut ImMessenger<RenderObjEvent>) {
        match self.render_map.remove(entity.id() as usize) {
            Some(index) => {
                render_objs.remove(index);
				renderobjs_event_writer.send(RenderObjEvent::new_delete(index));
            }
            None => (),
        };
    }

    #[inline]
    fn create_render_obj(
        &mut self,
        entity: Entity,
        render_objs: &mut ResMut<RenderObjs>,
        default_state: &DefaultState,
		renderobjs_event_writer: &mut ImMessenger<RenderObjEvent>,/*render_objs*/
    ) -> usize {
        let index = create_render_obj(
            entity,
            -0.2,
            true,
            COLOR_VS_SHADER_NAME.clone(),
            COLOR_FS_SHADER_NAME.clone(),
            Share::new(self.default_paramter.clone()),
            default_state,
            render_objs,
            &mut self.render_map,
			renderobjs_event_writer,
        );
        render_objs[index]
            .paramter
            .as_ref()
            .set_single_uniform("blur", UniformValue::Float1(1.0));
        index
    }
}

// 背景颜色时rgba（而非渐变）， 创建geo
#[inline]
fn create_rgba_geo<C: HalContext + 'static>(
    border_radius: Option<&BorderRadius>,
    layout: &LayoutR,
    unit_quad: &Share<GeometryRes>,
    engine: &mut Engine<C>,
) -> Option<Share<GeometryRes>> {
    let radius = cal_border_radius(border_radius, layout);
    let g_b = geo_box(layout);
    if g_b.mins.x - g_b.maxs.x == 0.0 || g_b.mins.y - g_b.maxs.y == 0.0 {
        return None;
    }

    if radius.x <= g_b.mins.x {
        return Some(unit_quad.clone());
    } else {
        let mut hasher = DefaultHasher::default();
        radius_quad_hash(&mut hasher, radius.x, layout.rect.end - layout.rect.start - layout.border.start - layout.border.end, layout.rect.bottom - layout.rect.top-layout.border.bottom - layout.border.top);
        let hash = hasher.finish();
        match engine.geometry_res_map.get(&hash) {
            Some(r) => Some(r.clone()),
            None => {
                let r = split_by_radius(
                    g_b.mins.x,
                    g_b.mins.y,
                    g_b.maxs.x - g_b.mins.x,
                    g_b.maxs.y - g_b.mins.y,
                    radius.x - g_b.mins.x,
                    None,
                );
                if r.0.len() == 0 {
                    return None;
                } else {
                    let indices = to_triangle(&r.1, Vec::with_capacity(r.1.len()));
                    return Some(engine.create_geo_res(
                        hash,
                        indices.as_slice(),
                        &[AttributeDecs::new(
                            AttributeName::Position,
                            r.0.as_slice(),
                            2,
                        )],
                    ));
                }
            }
        }
    }
}

// 创建一个线性渐变背景色的geo
#[inline]
fn create_linear_gradient_geo<C: HalContext + 'static>(
    color: &LinearGradientColor,
    border_radius: Option<&BorderRadius>,
    layout: &LayoutR,
    engine: &mut Engine<C>,
) -> Option<Share<GeometryRes>> {
    let radius = cal_border_radius(border_radius, layout);
    let g_b = geo_box(layout);
    if g_b.mins.x - g_b.maxs.x == 0.0 || g_b.mins.y - g_b.maxs.y == 0.0 {
        return None;
    }

    // 圆角 + 渐变hash
    let mut hasher = DefaultHasher::default();
    GRADUAL.hash(&mut hasher);
    NotNan::new(color.direction).unwrap().hash(&mut hasher);
    for c in color.list.iter() {
        NotNan::new(c.position).unwrap().hash(&mut hasher);
        NotNan::new(c.rgba.r).unwrap().hash(&mut hasher);
        NotNan::new(c.rgba.g).unwrap().hash(&mut hasher);
        NotNan::new(c.rgba.b).unwrap().hash(&mut hasher);
        NotNan::new(c.rgba.a).unwrap().hash(&mut hasher);
    }
    NotNan::new(color.direction).unwrap().hash(&mut hasher);
    radius_quad_hash(
        &mut hasher,
        radius.x - g_b.mins.x,
        g_b.maxs.x - g_b.mins.x,
        g_b.maxs.y - g_b.mins.y,
    );
    let hash = hasher.finish();

    match engine.geometry_res_map.get(&hash) {
        Some(r) => Some(r.clone()),
        None => {
            let (positions, indices) = if radius.x <= g_b.mins.x {
                (
                    vec![
                        g_b.mins.x, g_b.mins.y, // left_top
                        g_b.mins.x, g_b.maxs.y, // left_bootom
                        g_b.maxs.x, g_b.maxs.y, // right_bootom
                        g_b.maxs.x, g_b.mins.y, // right_top
                    ],
                    vec![0, 1, 2, 3],
                )
            } else {
                split_by_radius(
                    g_b.mins.x,
                    g_b.mins.y,
                    g_b.maxs.x - g_b.mins.x,
                    g_b.maxs.y - g_b.mins.y,
                    radius.x - g_b.mins.x,
                    None,
                )
            };

            let mut lg_pos = Vec::with_capacity(color.list.len());
            let mut colors = Vec::with_capacity(color.list.len() * 4);
            for v in color.list.iter() {
                lg_pos.push(v.position);
                colors.extend_from_slice(&[v.rgba.r, v.rgba.g, v.rgba.b, v.rgba.a]);
            }

			let width = layout.rect.end - layout.rect.start - layout.border.start - layout.border.end;
			let height = layout.rect.bottom - layout.rect.top-layout.border.bottom - layout.border.top;
            //渐变端点
            let endp = find_lg_endp(
                &[
                    0.0,
                    0.0,
                    0.0,
                    height,
                    width,
                    height,
                    width,
                    0.0,
                ],
                color.direction,
            );

            let (positions, indices_arr) = split_by_lg(
                positions,
                indices,
                lg_pos.as_slice(),
                endp.0.clone(),
                endp.1.clone(),
            );

            let mut colors = interp_mult_by_lg(
                positions.as_slice(),
                &indices_arr,
                vec![Vec::new()],
                vec![LgCfg {
                    unit: 4,
                    data: colors,
                }],
                lg_pos.as_slice(),
                endp.0,
                endp.1,
            );

            let indices = mult_to_triangle(&indices_arr, Vec::new());
            let colors = colors.pop().unwrap();
            // 创建geo， 设置attribut
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

#[inline]
fn background_is_opacity(opacity: f32, background_color: &BackgroundColor) -> bool {
    if opacity < 1.0 {
        return false;
    }
    background_color.0.is_opaque()
}

// 修改颜色， 返回是否存在宏的修改(不是class中的颜色)
#[inline]
fn modify_color<C: HalContext + 'static>(
    render_obj: &mut RenderObj,
    background_color: &BackgroundColor,
    engine: &mut Engine<C>,
    dirty: usize,
    layout: &LayoutR,
    unit_quad: &Share<GeometryRes>,
    border_radius: Option<&BorderRadius>,
) -> bool {
    let mut change = false;
    match &background_color.0 {
        Color::RGBA(c) => {
            if dirty & StyleType::BackgroundColor as usize != 0 {
                change = to_ucolor_defines(
                    render_obj.vs_defines.as_mut(),
                    render_obj.fs_defines.as_mut(),
                );
                render_obj
                    .paramter
                    .as_ref()
                    .set_value("uColor", engine.create_u_color_ubo(c));
            }
            // 如果颜色类型改变（纯色改为渐变色， 或渐变色改为纯色）或圆角改变， 需要重新创建geometry
            if change
                || dirty & StyleType::BorderRadius as usize != 0
                || dirty & StyleType::Layout as usize != 0
            {
                render_obj.geometry = create_rgba_geo(border_radius, layout, unit_quad, engine);
            }
        }
        Color::LinearGradient(c) => {
            if dirty & StyleType::BackgroundColor as usize != 0 {
                change = to_vex_color_defines(
                    render_obj.vs_defines.as_mut(),
                    render_obj.fs_defines.as_mut(),
                );
            }

            // 如果颜色类型改变（纯色改为渐变色， 或渐变色改为纯色）或圆角改变， 需要重新创建geometry
            if change
                || dirty & StyleType::BorderRadius as usize != 0
                || dirty & StyleType::Layout as usize != 0
            {	

                render_obj.geometry = create_linear_gradient_geo(c, border_radius, layout, engine);
            }
        }
    };
    change
}

// impl_system! {
//     BackgroundColorSys<C> where [C: HalContext + 'static],
//     true,
//     {
//         MultiCaseListener<Node, BackgroundColor, DeleteEvent>
//     }
// }
