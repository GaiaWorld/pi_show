use share::Share;
/**
 *  边框颜色渲染对象的构建及其属性设置
 */
use std::marker::PhantomData;

use ecs::{DeleteEvent, MultiCaseImpl, MultiCaseListener, Runner, SingleCaseImpl};
use hal_core::*;
use map::vecmap::VecMap;
use polygon::*;

use component::calc::Opacity;
use component::calc::*;
use component::user::*;
use entity::Node;
use render::engine::{AttributeDecs, Engine, ShareEngine};
use render::res::GeometryRes;
use single::*;
use system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};
use system::util::*;

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
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, BorderColor>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a SingleCaseImpl<DefaultTable>,
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
            opacitys,
            world_matrixs,
            transforms,
            border_radiuses,
            border_colors,
            style_marks,
            default_table,
            dirty_list,
            default_state,
        ) = read;

        if dirty_list.0.len() == 0 {
            return;
        }

        let (render_objs, engine) = write;

        let default_transform = default_table.get::<Transform>().unwrap();
        let notify = render_objs.get_notify();

        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                }
            };

            let mut dirty = style_mark.dirty;

            // 不存在BorderColor关心的脏, 跳过
            if dirty & DIRTY_TY == 0 {
                continue;
            }

            // 边框颜色脏， 如果不存在BorderColor的本地样式和class样式(边框颜色被删除)， 删除渲染对象
            let render_index = if dirty & StyleType::BorderColor as usize != 0 {
                if style_mark.local_style & StyleType::BorderColor as usize == 0
                    && style_mark.class_style & StyleType::BorderColor as usize == 0
                {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                } else {
                    match self.render_map.get_mut(*id) {
                        Some(r) => *r,
                        None => {
                            dirty |= DIRTY_TY;
                            self.create_render_obj(*id, render_objs, default_state)
                        }
                    }
                }
            } else {
                match self.render_map.get_mut(*id) {
                    Some(r) => *r,
                    None => continue,
                }
            };

            let color = &border_colors[*id].0;
            let render_obj = &mut render_objs[render_index];
            let border_radius = border_radiuses.get(*id);
            let layout = &layouts[*id];

            // 如果Color脏， 或Opacity脏， 计算is_opacity
            if dirty & StyleType::BorderColor as usize != 0
                || dirty & StyleType::Opacity as usize != 0
            {
                let opacity = opacitys[*id].0;
                render_obj.is_opacity = color.a >= 1.0 && opacity >= 1.0;
                notify.modify_event(render_index, "is_opacity", 0);
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
                render_obj.geometry = Some(create_geo(border_radius, layout, engine));
            }

            // 如果矩阵脏， 更新worldMatrix ubo
            if dirty & StyleType::Matrix as usize != 0 {
                let world_matrix = &world_matrixs[*id];

                let transform = match transforms.get(*id) {
                    Some(r) => r,
                    None => default_transform,
                };
                modify_matrix(
                    render_index,
                    create_let_top_offset_matrix(
                        layout,
                        world_matrix,
                        transform,
                        0.0,
                        0.0,
                        render_obj.depth,
                    ),
                    render_obj,
                    &notify,
                );
            }

            notify.modify_event(render_index, "", 0);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderColor, DeleteEvent>
    for BorderColorSys<C>
{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, render_objs: Self::WriteData) {
        self.remove_render_obj(event.id, render_objs)
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

    // 删除渲染对象
    #[inline]
    fn remove_render_obj(&mut self, id: usize, render_objs: &mut SingleCaseImpl<RenderObjs>) {
        match self.render_map.remove(id) {
            Some(index) => {
                let notify = render_objs.get_notify();
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
    radiu: Option<&BorderRadius>,
    layout: &Layout,
    engine: &mut Engine<C>,
) -> Share<GeometryRes> {
    let buffer = get_geo_flow(radiu, layout);
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
fn get_geo_flow(radiu: Option<&BorderRadius>, layout: &Layout) -> (Vec<f32>, Vec<u16>) {
    let radius = cal_border_radius(radiu, layout);

    if radius.x == 0.0 {
        let border_start_x = layout.border_left;
        let border_start_y = layout.border_top;
        let border_end_x = layout.width - layout.border_right;
        let border_end_y = layout.height - layout.border_bottom;

        (
            vec![
                0.0,
                0.0,
                0.0,
                layout.height,
                layout.width,
                layout.height,
                layout.width,
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
    } else {
        split_by_radius_border(
            0.0,
            0.0,
            layout.width,
            layout.height,
            radius.x,
            layout.border_left,
            None,
        )
    }
}

unsafe impl<C: HalContext + 'static> Sync for BorderColorSys<C> {}
unsafe impl<C: HalContext + 'static> Send for BorderColorSys<C> {}

impl_system! {
    BorderColorSys<C> where [C: HalContext + 'static],
    true,
    {
        MultiCaseListener<Node, BorderColor, DeleteEvent>
    }
}
