use share::Share;
/**
 *  边框颜色渲染对象的构建及其属性设置
 */
use std::marker::PhantomData;

use ecs::{DeleteEvent, MultiCaseImpl, EntityListener, Runner, SingleCaseImpl};
use ecs::monitor::{NotifyImpl, Event};
use hal_core::*;
use map::vecmap::VecMap;
use map::Map;
use polygon::*;

use crate::component::calc::{Opacity, LayoutR};
use crate::component::calc::*;
use crate::component::user::*;
use crate::entity::Node;
use crate::render::engine::{AttributeDecs, Engine, ShareEngine};
use crate::render::res::GeometryRes;
use crate::single::*;
use crate::system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};
use crate::system::util::*;

const DIRTY_TY: usize = StyleType::Matrix as usize
    | StyleType::Opacity as usize
    | StyleType::Layout as usize
    | StyleType::BorderColor as usize
    | StyleType::BorderRadius as usize;

const GEO_DIRTY: usize = StyleType::Layout as usize | StyleType::BorderRadius as usize;

// lazy_static! {
//     // BorderColor几何属性的标志， 用于计算geometry的hash， 减弱hash冲突
//     pub static ref BORDER_COLOR: Atom = Atom::from("border_color");
// }

// 声明结构
/// BorderColor 操作
pub struct BorderColorSys<C: HalContext + 'static> {
    render_map: VecMap<usize>,
    default_paramter: ColorParamter,
    mark: PhantomData<C>,
}

// 实现 Runner
impl<'a, C: HalContext + 'static> Runner<'a> for BorderColorSys<C> {
    type ReadData = (
        &'a MultiCaseImpl<Node, LayoutR>,
        // &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, BorderColor>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a SingleCaseImpl<DirtyList>,
        &'a SingleCaseImpl<DefaultState>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<ShareEngine<C>>,
    );
    /// 将顶点数据改变的渲染对象重新设置索引流和顶点流
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        let (
            layouts,
            // opacitys,
            world_matrixs,
            transforms,
            border_radiuses,
            border_colors,
            style_marks,
            dirty_list,
            default_state,
        ) = read;

        if dirty_list.0.len() == 0 {
            return;
        }

        let (render_objs, engine) = write;

		let notify = unsafe { &* (render_objs.get_notify_ref() as *const NotifyImpl)} ;

        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                }
			};

			let color = match border_colors.get(*id) {
				Some(r) => &r.0,
				None => {
					self.remove_render_obj(*id, render_objs);
					continue;
				}
			};

			if style_mark.local_style & StyleType::BorderColor as usize == 0
				&& style_mark.class_style & StyleType::BorderColor as usize == 0
			{
				self.remove_render_obj(*id, render_objs);
				continue;
			}

            let mut dirty = style_mark.dirty;

            // 不存在BorderColor关心的脏, 跳过
            if dirty & DIRTY_TY == 0 {
                continue;
            }

            // 边框颜色脏， 如果不存在BorderColor的本地样式和class样式(边框颜色被删除)， 删除渲染对象
            let render_index = match self.render_map.get_mut(*id) {
				Some(r) => *r,
				None => {
					dirty |= DIRTY_TY;
					self.create_render_obj(*id, render_objs, default_state)
				}
			};

            let render_obj = &mut render_objs[render_index];
            let border_radius = border_radiuses.get(*id);
            let layout = &layouts[*id];

            // 如果Color脏， 或Opacity脏， 计算is_opacity
            if dirty & StyleType::BorderColor as usize != 0
                || dirty & StyleType::Opacity as usize != 0
            {
				// let opacity = opacitys[*id].0;
				let is_opacity_old = render_obj.is_opacity;
				render_obj.is_opacity = color.a >= 1.0;
				if render_obj.is_opacity != is_opacity_old {
					notify.modify_event(render_index, "is_opacity", 0);
				};
                modify_opacity(engine, render_obj, default_state);
            }

            // 颜色修改， 设置ucolor ubo
            if dirty & StyleType::BorderColor as usize != 0 {
                // to_ucolor_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut());
                render_obj
                    .paramter
                    .as_ref()
                    .set_value("uColor", engine.create_u_color_ubo(color));
            }

            // 布局或圆角修改， 重新创建geometry
            if dirty & GEO_DIRTY != 0 {
				let (radius_hash, border_radius) = match border_radius {
					Some(r) => {
						let border_radius = cal_border_radius(r, &layout.rect);
						render_obj.fs_defines.add(BORDER);
						let (width, height)  = (layout.rect.end - layout.rect.start, layout.rect.bottom - layout.rect.top);

						// 设置clipSdf
						render_obj
						.paramter
						.as_ref()
						.set_single_uniform("clipSdf", UniformValue::MatrixV4(vec![
							width/2.0, height/2.0, layout.border.bottom, layout.border.start,
							width/2.0, height/2.0, layout.border.top, layout.border.end,
							border_radius.y[0], border_radius.x[0], border_radius.x[1], border_radius.y[1],
							border_radius.y[2], border_radius.x[2], border_radius.x[3], border_radius.y[3],
						]));

						(calc_float_hash(&border_radius.y, calc_float_hash(&border_radius.x, 0)), Some(border_radius))
					},
					None => {
						render_obj.fs_defines.remove(BORDER);
						(0, None)
					}
				};
                render_obj.geometry = Some(create_geo(&border_radius, layout, engine));
            }

            // 如果矩阵脏， 更新worldMatrix ubo
            if dirty & StyleType::Matrix as usize != 0 {
                let world_matrix = &world_matrixs[*id];

                let transform = &transforms[*id];
                modify_matrix(
                    render_index,
                    create_let_top_offset_matrix(
                        layout,
                        world_matrix,
                        transform,
                        0.0,
                        0.0,
                        // render_obj.depth,
                    ),
                    render_obj,
                    &notify,
                );
            }

            notify.modify_event(render_index, "", 0);
        }
    }
}

// 监听实体销毁，删除索引
impl<'a, C: HalContext + 'static> EntityListener<'a, Node, DeleteEvent>
    for BorderColorSys<C>
{
    type ReadData = ();
    type WriteData = ();

    fn listen(&mut self, event: &Event, _read: Self::ReadData, _: Self::WriteData) {
		self.render_map.remove(event.id); // 移除索引
    }
}

/// 基本实现
impl<C: HalContext + 'static> BorderColorSys<C> {
    /// 创建方法
    #[inline]
    pub fn new() -> Self {
        BorderColorSys {
            render_map: VecMap::default(),
            default_paramter: ColorParamter::default(),
            mark: PhantomData,
        }
	}
	
	pub fn with_capacity(capacity: usize) -> Self {
		BorderColorSys {
            render_map: VecMap::with_capacity(capacity),
            default_paramter: ColorParamter::default(),
            mark: PhantomData,
        }
    }

    // 删除渲染对象
    #[inline]
    fn remove_render_obj(&mut self, id: usize, render_objs: &mut SingleCaseImpl<RenderObjs>) {
        match self.render_map.remove(id) {
            Some(index) => {
				let notify = unsafe { &* (render_objs.get_notify_ref() as *const NotifyImpl)} ;
                render_objs.remove(index, Some(notify));
            }
            None => (),
        };
    }

    /// 创建渲染数据对象
    #[inline]
    fn create_render_obj(
        &mut self,
        id: usize,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &DefaultState,
    ) -> usize {
        let index = create_render_obj(
            id,
            -0.1,
            true,
            COLOR_VS_SHADER_NAME.clone(),
            COLOR_FS_SHADER_NAME.clone(),
            Share::new(self.default_paramter.clone()),
            default_state,
            render_objs,
            &mut self.render_map,
        );
        let render_obj = &mut render_objs[index];
        render_obj
            .paramter
            .as_ref()
            .set_single_uniform("blur", UniformValue::Float1(1.0));
        render_obj.fs_defines.add("UCOLOR");
        index
    }
}

// /////////////////////////////////////////////////////////////////
// /// 静态方法


// 创建几何体， 没有缓冲几何体， 应该缓冲？TODO
#[inline]
fn create_geo<C: HalContext + 'static>(
    radius: &Option<BorderRadiusPixel>,
    layout: &LayoutR,
    engine: &mut Engine<C>,
) -> Share<GeometryRes> {
    let buffer = get_geo_flow(radius, layout);
    engine.create_geo_res(
        0,
        buffer.1.as_slice(),
        &[AttributeDecs::new(
            AttributeName::Position,
            buffer.0.as_slice(),
            2,
        )],
    )
}

#[inline]
/// 取几何体的顶点流和属性流
fn get_geo_flow(radius: &Option<BorderRadiusPixel>, layout: &LayoutR) -> (Vec<f32>, Vec<u16>) {

	let width = layout.rect.end - layout.rect.start;
	let height = layout.rect.bottom - layout.rect.top;
	let border = &layout.border;
    match radius {
		None => {
			let border_start_x = border.start;
			let border_start_y = border.top;
			let border_end_x = width - border.end;
			let border_end_y = height - border.bottom;

			(
				vec![
					0.0,
					0.0,
					0.0,
					height,
					width,
					height,
					width,
					0.0,
					border_start_x,
					border_start_y,
					border_start_x,
					border_end_y,
					border_end_x,
					border_end_y,
					border_end_x,
					border_start_y,
				],
				vec![
					0, 1, 4, 0, 4, 3, 3, 4, 7, 3, 7, 2, 2, 7, 6, 2, 6, 1, 1, 6, 5, 1, 5, 4,
				],
			)
		}
		Some(radius) => {
			let mut vert = Vec::new();
			let mut index = Vec::new();

			// 索引位置
			// 0         4      5       9
			//   ________|______|________
			//  |                        |
			//  |        3      6        |
			//  |     ___|______|___     |
			//  |    |              |    |
			//1 |-   |-2          7-|   -|8
			//  |    |              |    |
			//15|-   |-19        14-|   -|13
			//  |    |              |    |
			//  |    |___|_______|__|    |
			//  |        18     10       |
			//  |                        |
			//16|_______|_______|________|
			//         17     11     12
			vert.extend_from_slice(&[
				0.0, 0.0, // 0
				0.0, radius.y[0], // 1
				border.start, radius.y[0], // 2
				radius.x[0], border.top, // 3
				radius.x[0], 0.0, // 4

				width - radius.x[1], 0.0, // 5
				width - radius.x[1], border.top, // 6
				width - border.end, radius.y[1], // 7
				width, radius.y[1], // 8
				width, 0.0, // 9


				width - radius.x[2], height - border.bottom, // 10
				width - radius.x[2], height, // 11
				width, height, // 12
				width, height - radius.y[2], // 13
				width - border.end, height - radius.y[2], // 14

				0.0, height - radius.y[3], // 15
				0.0, height, // 16
				radius.x[3], height, // 17
				radius.x[3], height - border.bottom, // 18
				border.start, height - radius.y[3], // 19
				]);
			index.extend_from_slice(&[
				0, 1, 2, 0, 2, 3, 0, 3, 4, // 左上
				5, 6, 9, 6, 7, 9, 7, 8, 9, // 右上
				10, 11, 12, 10, 12, 14, 14, 12, 13, // 右下
				15, 16, 19, 19, 16, 18, 18, 16, 17, // 左下
				4, 3, 6, 4, 6, 5, // 上
				7, 14, 13, 7, 13, 8, // 右
				18, 17, 11, 18, 11, 10, // 下
				1, 15, 19, 1, 19, 2, // 左
				]);
			(vert, index)
		}
    }
}

const BORDER: &'static str = "BORDER";

impl_system! {
    BorderColorSys<C> where [C: HalContext + 'static],
    true,
    {
        EntityListener<Node, DeleteEvent>
    }
}
