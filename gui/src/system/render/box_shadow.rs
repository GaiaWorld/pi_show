use std::marker::PhantomData;
use std::slice;

use cg2d::{BooleanOperation, Polygon as Polygon2d};
use nalgebra::Point2;
use ecs::{DeleteEvent, MultiCaseImpl, MultiCaseListener, Runner, SingleCaseImpl};
use ecs::monitor::NotifyImpl;
use hal_core::*;
use map::vecmap::VecMap;
use map::Map;
use polygon::*;
use share::Share;

use crate::single::*;
use crate::system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};
use crate::system::util::*;
use crate::render::engine::{AttributeDecs, Engine, ShareEngine};
use crate::render::res::GeometryRes;
use crate::entity::Node;
use crate::component::calc::{Opacity, LayoutR};
use crate::component::calc::*;
use crate::component::user::*;

const DITY_TYPE: usize = StyleType::BoxShadow as usize
            | StyleType::Matrix as usize
            | StyleType::BorderRadius as usize
            | StyleType::Opacity as usize
			| StyleType::Layout as usize;
			
pub struct BoxShadowSys<C: HalContext + 'static> {
    render_map: VecMap<usize>,
    default_paramter: ColorParamter,
    marker: std::marker::PhantomData<C>,
}

impl<C: HalContext + 'static> BoxShadowSys<C> {
	pub fn with_capacity(capacity: usize) -> Self {
		BoxShadowSys {
			render_map: VecMap::with_capacity(capacity),
			default_paramter: ColorParamter::default(),
			marker: std::marker::PhantomData,
		}
	}
}

impl<C: HalContext + 'static> Default for BoxShadowSys<C> {
    fn default() -> Self {
        Self {
            render_map: VecMap::default(),
            default_paramter: ColorParamter::default(),
            marker: PhantomData,
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for BoxShadowSys<C> {
    type ReadData = (
        &'a MultiCaseImpl<Node, BoxShadow>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, LayoutR>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a SingleCaseImpl<DirtyList>,
        &'a SingleCaseImpl<DefaultState>,
    );

    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<ShareEngine<C>>,
    );

    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        let (
            box_shadows,
            world_matrixs,
            border_radiuses,
            opacitys,
            layouts,
            z_depths,
            transforms,
            style_marks,
            dirty_list,
            default_state,
        ) = read;

        if dirty_list.0.len() == 0 {
            return;
        }

		let (render_objs, engine) = write;
		let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };

        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                }
            };

            let dirty = style_mark.dirty;

            // 不存在BoxShadow关心的脏, 跳过
            if dirty & DITY_TYPE == 0 {
                continue;
			}

            // 阴影脏
            let render_index = if dirty & StyleType::BoxShadow as usize != 0 {
                // 如果不存在BoxShadow本地样式和class样式， 删除渲染对象
                if style_mark.local_style & StyleType::BoxShadow as usize == 0
                    && style_mark.class_style & StyleType::BoxShadow as usize == 0
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

            // 从组件中取出对应的数据
            let render_obj = &mut render_objs[render_index];

            let border_radius = border_radiuses.get(*id);
            let layout = &layouts[*id];
            let shadow = &box_shadows[*id];

            // 如果Color脏， 或Opacity脏， 计算is_opacity
            if dirty & StyleType::Opacity as usize != 0
                || dirty & StyleType::BoxShadow as usize != 0
            {
				let opacity = opacitys[*id].0;
				let is_opacity_old = render_obj.is_opacity;
                render_obj.is_opacity = color_is_opacity(opacity, &shadow.color, shadow.blur);
                if render_obj.is_opacity != is_opacity_old {
					notify.modify_event(render_index, "is_opacity", 0);
				}
                modify_opacity(engine, render_obj, default_state);
            }

            if style_mark.dirty & StyleType::BoxShadow as usize != 0 {
                render_obj
                    .paramter
                    .as_ref()
                    .set_value("uColor", engine.create_u_color_ubo(&shadow.color));
            }
            // 如果阴影脏，或者边框半径改变，则重新创建geometry
            if style_mark.dirty & StyleType::BoxShadow as usize != 0
                || style_mark.dirty & StyleType::BorderRadius as usize != 0
                || style_mark.dirty & StyleType::Layout as usize != 0
            {
                // render_obj.program_dirty =  true;
                // to_ucolor_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut());
                render_obj.geometry =
                    create_shadow_geo(engine, render_obj, layout, shadow, border_radius);
            }

            // 矩阵脏，或者布局脏
            if dirty & StyleType::Matrix as usize != 0 || dirty & StyleType::Layout as usize != 0 {
                let world_matrix = &world_matrixs[*id];
                let transform = &transforms[*id];
                let depth = z_depths[*id].0;
                modify_matrix(render_obj, depth, world_matrix, transform, layout);
            }
            notify.modify_event(render_index, "", 0);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BoxShadow, DeleteEvent>
    for BoxShadowSys<C>
{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, render_objs: Self::WriteData) {
        self.remove_render_obj(event.id, render_objs)
    }
}

impl<C: HalContext + 'static> BoxShadowSys<C> {
    #[inline]
    fn remove_render_obj(&mut self, id: usize, render_objs: &mut SingleCaseImpl<RenderObjs>) {
        match self.render_map.remove(id) {
            Some(index) => {
                let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
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
            -0.3,
            true,
            COLOR_VS_SHADER_NAME.clone(),
            COLOR_FS_SHADER_NAME.clone(),
            Share::new(self.default_paramter.clone()),
            default_state,
            render_objs,
            &mut self.render_map,
        );
        let render_obj = &mut render_objs[index];
        render_obj.fs_defines.add("UCOLOR");
        index
    }
}

#[inline]
fn modify_matrix(
    render_obj: &mut RenderObj,
    depth: f32,
    world_matrix: &WorldMatrix,
    transform: &Transform,
    layout: &LayoutR,
) {
    let arr = create_let_top_offset_matrix(layout, world_matrix, transform, 0.0, 0.0, depth);
    render_obj.paramter.set_value(
        "worldMatrix",
        Share::new(WorldMatrixUbo::new(UniformValue::MatrixV4(arr))),
    );
}

#[inline]
fn color_is_opacity(opacity: f32, color: &CgColor, blur: f32) -> bool {
    opacity == 1.0 && color.a == 1.0 && blur == 0.0
}

fn create_shadow_geo<C: HalContext + 'static>(
    engine: &mut Engine<C>,
    render_obj: &mut RenderObj,
    layout: &LayoutR,
    shadow: &BoxShadow,
    border_radius: Option<&BorderRadius>,
) -> Option<Share<GeometryRes>> {
    let radius = cal_border_radius(border_radius, layout);
    let g_b = geo_box(layout);
    if g_b.mins.x - g_b.maxs.x == 0.0 || g_b.mins.y - g_b.maxs.y == 0.0 {
        return None;
    }

    let x = g_b.mins.x;
    let y = g_b.mins.y;
    let w = g_b.maxs.x - g_b.mins.x;
    let h = g_b.maxs.y - g_b.mins.y;
    let bg = split_by_radius(x, y, w, h, radius.x, Some(16));
    if bg.0.len() == 0 {
        return None;
    }

    let x = g_b.mins.x + shadow.h - shadow.spread - shadow.blur;
    let y = g_b.mins.y + shadow.v - shadow.spread - shadow.blur;
    let w = g_b.maxs.x - g_b.mins.x + 2.0 * shadow.spread + 2.0 * shadow.blur;
    let h = g_b.maxs.y - g_b.mins.y + 2.0 * shadow.spread + 2.0 * shadow.blur;
    let shadow_pts = split_by_radius(x, y, w, h, radius.x, Some(16));
    if shadow_pts.0.len() == 0 {
        return None;
    }

    let polygon_shadow = Polygon2d::new(convert_to_point(shadow_pts.0.as_slice()));
    let polygon_bg = Polygon2d::new(convert_to_point(bg.0.as_slice()));

    let mut curr_index = 0;
    let mut pts: Vec<f32> = vec![];
    let mut indices: Vec<u16> = vec![];
    for p in Polygon2d::boolean(&polygon_shadow, &polygon_bg, BooleanOperation::Difference) {
        pts.extend_from_slice(convert_to_f32(p.vertices.as_slice()));

        let tri_indices = p.triangulation();
        indices.extend_from_slice(
            tri_indices
                .iter()
                .map(|&v| (v + curr_index) as u16)
                .collect::<Vec<u16>>()
                .as_slice(),
        );

        curr_index += p.vertices.len();
    }

    if pts.len() == 0 {
        return None;
    }

    render_obj.paramter.as_ref().set_single_uniform(
        "blur",
        UniformValue::Float1(if shadow.blur > 0.0 { shadow.blur } else { 1.0 }),
    );

    if shadow.blur > 0.0 {
        render_obj.vs_defines.add("BOX_SHADOW_BLUR");
        render_obj.fs_defines.add("BOX_SHADOW_BLUR");

        let x = g_b.mins.x + shadow.h - shadow.spread;
        let y = g_b.mins.y + shadow.v - shadow.spread;
        let w = g_b.maxs.x - g_b.mins.x + 2.0 * shadow.spread;
        let h = g_b.maxs.y - g_b.mins.y + 2.0 * shadow.spread;
        render_obj
            .paramter
            .as_ref()
            .set_single_uniform("uRect", UniformValue::Float4(x, y, x + w, y + h));
    } else {
        render_obj.fs_defines.add("BOX_SHADOW_BLUR");
        render_obj.fs_defines.remove("BOX_SHADOW_BLUR");
    }

    debug_println!(
        "create_shadow_geo: pts = {:?}, indices = {:?}",
        pts.as_slice(),
        indices.as_slice()
    );
    Some(engine.create_geo_res(
        0,
        indices.as_slice(),
        &[AttributeDecs::new(
            AttributeName::Position,
            pts.as_slice(),
            2,
        )],
    ))
}

#[inline]
fn convert_to_point(pts: &[f32]) -> &[Point2<f32>] {
    let ptr = pts.as_ptr();
    let ptr = ptr as *const Point2<f32>;
    unsafe { slice::from_raw_parts(ptr, pts.len() / 2) }
}

#[inline]
fn convert_to_f32(pts: &[Point2<f32>]) -> &[f32] {
    let ptr = pts.as_ptr();
    let ptr = ptr as *const f32;
    unsafe { slice::from_raw_parts(ptr, 2 * pts.len()) }
}

impl_system! {
    BoxShadowSys<C> where [C: HalContext + 'static],
    true,
    {
        MultiCaseListener<Node, BoxShadow, DeleteEvent>
    }
}
