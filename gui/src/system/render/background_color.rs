
use flex_layout::Size;
/**
 * 背景色渲染对象的构建及其属性设置
*/
use share::Share;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use hash::DefaultHasher;
use map::vecmap::VecMap;
use ordered_float::NotNan;

use pi_atom::Atom;
use ecs::{DeleteEvent, MultiCaseImpl, EntityListener, Runner, SingleCaseImpl};
use ecs::monitor::Event;
use ecs::monitor::NotifyImpl;
use hal_core::*;
use pi_polygon::*;

use crate::component::calc::LayoutR;
use crate::component::calc::*;
use crate::component::user::*;
use crate::entity::*;
use crate::render::engine::*;
use crate::render::res::*;
use crate::single::*;
use crate::system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};
use crate::system::util::*;

lazy_static! {
    static ref GRADUAL: Atom = Atom::from("GRADUAL");
	static ref DIRTY_TYPE: StyleBit = style_bit().set_bit(StyleType::BackgroundColor as usize)
	.set_bit(StyleType::BorderRadius as usize)
    .set_bit(StyleType::Opacity as usize);
	static ref OPACITY_TYPE: StyleBit = style_bit().set_bit(StyleType::BackgroundColor as usize)
		.set_bit(StyleType::Opacity as usize);
}



pub struct BackgroundColorSys<C: HalContext + 'static> {
    render_map: VecMap<usize>,
    default_paramter: ColorParamter,
    marker: std::marker::PhantomData<C>,
}

impl<C: HalContext + 'static> BackgroundColorSys<C> {
    pub fn new() -> Self {
        BackgroundColorSys {
            render_map: VecMap::default(),
            default_paramter: ColorParamter::default(),
            marker: PhantomData,
        }
	}
	
	pub fn with_capacity(capacity: usize) -> Self {
        BackgroundColorSys {
            render_map: VecMap::with_capacity(capacity),
            default_paramter: ColorParamter::default(),
            marker: PhantomData,
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for BackgroundColorSys<C> {
    type ReadData = (
        &'a MultiCaseImpl<Node, LayoutR>,
        &'a MultiCaseImpl<Node, ZRange>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        // &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, BackgroundColor>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a SingleCaseImpl<UnitQuad>,
        &'a SingleCaseImpl<DirtyList>,
        &'a SingleCaseImpl<DefaultState>,
		&'a SingleCaseImpl<VertType>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<ShareEngine<C>>,
    );
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        // 没有脏， 跳过
        if (read.8).0.len() == 0 {
            return;
        }

        let (
            layouts,
            z_depths,
            world_matrixs,
            transforms,
            // opacitys,
            border_radiuses,
            background_colors,
            style_marks,
            unit_quad,
            dirty_list,
            default_state,
			single_vert_type,
        ) = read;
        let (render_objs, engine) = write;

		let default_transform = Transform::default();
		let notify = unsafe { &* (render_objs.get_notify_ref() as *const NotifyImpl)} ;
		let time = cross_performance::now();
        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                }
            };

            // 不存在BuckgroundColor关心的脏, 跳过
            if !(style_mark.dirty & &*DIRTY_TYPE).any() && style_mark.dirty1 & GEO_DIRTY_TYPE == 0 {
                continue;
            }

            let dirty = &style_mark.dirty;
			let dirty1 = style_mark.dirty1;
			
			let color = match background_colors.get(*id) {
				Some(r) => r,
				None => {
					self.remove_render_obj(*id, render_objs);
					continue;
				}
			};
            // 背景颜色脏， 如果不存在BacgroundColor的本地样式和class样式， 删除渲染对象
            let render_index = if dirty[StyleType::BackgroundColor as usize] {
                if !style_mark.local_style[StyleType::BackgroundColor as usize]
                    && !style_mark.class_style[StyleType::BackgroundColor as usize]
                {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                } else {
                    match self.render_map.get_mut(*id) {
                        Some(r) => *r,
                        None => self.create_render_obj(*id, render_objs, default_state),
                    }
                }
            } else {
                match self.render_map.get_mut(*id) {
                    Some(r) => *r,
                    None => continue,
                }
            };

            let render_obj = &mut render_objs[render_index];
            let border_radius = border_radiuses.get(*id);
            let layout = &layouts[*id];

            // 如果Color脏， 或Opacity脏， 计算is_opacity
            if dirty[StyleType::BackgroundColor as usize]
                || dirty[StyleType::Opacity as usize]
            {
				// let opacity = opacitys[*id].0;
				let is_opacity_old = render_obj.is_opacity;
				render_obj.is_opacity = background_is_opacity(1.0, color) && border_radius.is_none();
				if render_obj.is_opacity != is_opacity_old {
					notify.modify_event(render_index, "is_opacity", 0);
				}
                modify_opacity(engine, render_obj, default_state);
            }

			let (program_dirty, vert_type) = modify_color(
                render_obj,
                color,
                engine,
                &dirty,
				dirty1,
                layout,
                &unit_quad.0,
            );

			if vert_type != render_obj.vert_type {
				render_obj.vert_type = vert_type;
				single_vert_type.get_notify_ref().modify_event(render_index, "", 0);
			}
            // 如果矩阵脏
            if dirty1 & GEO_DIRTY_TYPE != 0 {
                let world_matrix = &world_matrixs[*id];
                let transform = match transforms.get(*id) {
                    Some(r) => r,
                    None => &default_transform,
                };
                let depth = z_depths[*id].start as f32;

                match &color.0 {
                    Color::RGBA(_) => {
						modify_matrix(
							render_index,
							create_unit_matrix_by_layout(layout, world_matrix, transform, depth),
							render_obj,
							&notify,
						);
					},
                    Color::LinearGradient(_) => {
						modify_matrix(
							render_index,
							create_let_top_offset_matrix(
								layout,
								world_matrix,
								transform,
								0.0,
								0.0,
								// depth,
							),
							render_obj,
							&notify,
						);
					},
                };
            }
            notify.modify_event(render_index, "", 0);
        }
		// if dirty_list.0.len() > 0 {
		// 	log::info!("bg_color======={:?}", cross_performance::now() - time);
		// }
    }
}

// 监听实体销毁，删除索引
impl<'a, C: HalContext + 'static> EntityListener<'a, Node, DeleteEvent>
    for BackgroundColorSys<C>
{
    type ReadData = ();
    type WriteData = ();

    fn listen(&mut self, event: &Event, _read: Self::ReadData, _: Self::WriteData) {
		self.render_map.remove(event.id); // 移除索引
    }
}

impl<C: HalContext + 'static> BackgroundColorSys<C> {
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

    #[inline]
    fn create_render_obj(
        &mut self,
        id: usize,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &DefaultState,
    ) -> usize {
        let index = create_render_obj(
            id,
            -0.2,
            true,
            COLOR_VS_SHADER_NAME.clone(),
            COLOR_FS_SHADER_NAME.clone(),
            Share::new(self.default_paramter.clone()),
            default_state,
            render_objs,
            &mut self.render_map,
        );
        render_objs[index]
            .paramter
            .as_ref()
            .set_single_uniform("blur", UniformValue::Float1(1.0));
        index
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
    dirty: &StyleBit,
	dirty1: usize,
    layout: &LayoutR,
    unit_quad: &Share<GeometryRes>,
) -> (bool, VertType) {
    let mut change = false;
	let mut vert_type = render_obj.vert_type;
	if dirty[StyleType::BackgroundColor as usize] {
		match &background_color.0 {
			Color::RGBA(c) => {
				change = to_ucolor_defines(
					render_obj.vs_defines.as_mut(),
					render_obj.fs_defines.as_mut(),
				);
				render_obj
					.paramter
					.as_ref()
					.set_value("uColor", engine.create_u_color_ubo(c));
				vert_type = VertType::ContentRect;
			}
			Color::LinearGradient(c) => {
				change = to_vex_color_defines(
					render_obj.vs_defines.as_mut(),
					render_obj.fs_defines.as_mut(),
				);
				vert_type = VertType::ContentNone;
			}
		};
	}
	
	if change || dirty1 & CalcType::Layout as usize != 0 {
		let mut hasher = DefaultHasher::default();
		background_color.0.hash(&mut hasher);
		radius_quad_hash(&mut hasher, 0.0, layout.rect.right - layout.rect.left - layout.border.left - layout.border.right, layout.rect.bottom - layout.rect.top-layout.border.bottom - layout.border.top);
		let hash = hasher.finish();

		match engine.geometry_res_map.get(&hash) {
			Some(r) => render_obj.geometry = Some(r.clone()),
			None => {
				if let Color::LinearGradient(color) = &background_color.0 {
					let rect = get_content_rect(layout);
					let size = Size {width: rect.right - rect.left, height: rect.bottom - rect.top};
					let (positions, indices) = (
						vec![
							*rect.left, *rect.top, // left_top
							*rect.left, *rect.bottom, // left_bootom
							*rect.right, *rect.bottom, // right_bootom
							*rect.right, *rect.top, // right_top
						],
						vec![0, 1, 2, 3],
					);
					
					let (positions, colors, indices) = linear_gradient_split(color, positions, indices, &size);
	
					render_obj.geometry = Some(engine.create_geo_res(
						hash,
						indices.as_slice(),
						&[
							AttributeDecs::new(AttributeName::Position, positions.as_slice(), 2),
							AttributeDecs::new(AttributeName::Color, colors.as_slice(), 4),
						],
					))
				} else {
					render_obj.geometry = Some(unit_quad.clone());
				}
			}
		}
	}
    (change, vert_type)
}

pub fn linear_gradient_split(color: &LinearGradientColor, positions: Vec<f32>, indices: Vec<u16>, size: &Size<NotNan<f32>>) -> (Vec<f32>, Vec<f32>, Vec<u16>) {
	let mut lg_pos = Vec::with_capacity(color.list.len());
	let mut colors = Vec::with_capacity(color.list.len() * 4);
	for v in color.list.iter() {
		lg_pos.push(v.position);
		colors.extend_from_slice(&[v.rgba.x, v.rgba.y, v.rgba.z, v.rgba.w]);
	}

	//渐变端点
	let endp = find_lg_endp(
		&[
			0.0,
			0.0,
			0.0,
			*size.height,
			*size.width,
			*size.height,
			*size.width,
			0.0,
		],
		color.direction,
	);

	let (positions1, indices1) = split_by_lg(
		positions,
		indices,
		lg_pos.as_slice(),
		endp.0.clone(),
		endp.1.clone(),
	);

	let mut colors = interp_mult_by_lg(
		positions1.as_slice(),
		&indices1,
		vec![Vec::new()],
		vec![LgCfg {
			unit: 4,
			data: colors,
		}],
		lg_pos.as_slice(),
		endp.0,
		endp.1,
	);

	let indices = mult_to_triangle(&indices1, Vec::new());
	let colors = colors.pop().unwrap();

	(positions1, colors, indices)
}


impl_system! {
    BackgroundColorSys<C> where [C: HalContext + 'static],
    true,
    {
        EntityListener<Node, DeleteEvent>
    }
}
