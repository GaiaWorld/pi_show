/**
 * Box Shadow
 */
use share::Share;
use std::slice;
use std::marker::PhantomData;
use ecs::{ SingleCaseImpl, MultiCaseImpl, Runner };
use map::{ vecmap::VecMap } ;
use hal_core::*;
use polygon::*;
use cg2d::{Point2, Polygon as Polygon2d, BooleanOperation};
use component::user::*;
use component::calc::*;
use component::calc::{ Opacity };
use entity::{ Node };
use single::*;
use render::engine::{ Engine };
use render::res::GeometryRes;
use system::util::*;
use system::render::shaders::color::{ COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME };

pub struct BoxShadowSys<C: HalContext + 'static> {
    render_map: VecMap<usize>,
    dirty_ty: usize,
    marker: std::marker::PhantomData<C>,
}

impl<C: HalContext + 'static> Default for BoxShadowSys<C> {
    fn default() -> Self {
        let dirty_ty = StyleType::BoxShadow as usize 
            | StyleType::Matrix as usize 
            | StyleType::BorderRadius as usize 
            | StyleType::Opacity as usize
            | StyleType::Layout as usize;

        Self {
            dirty_ty,
            render_map: VecMap::default(),
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
        &'a MultiCaseImpl<Node, Layout>,

        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, StyleMark>,

        &'a SingleCaseImpl<DefaultTable>,
        &'a SingleCaseImpl<ClassSheet>,
        &'a SingleCaseImpl<DirtyList>,
        &'a SingleCaseImpl<DefaultState>,
    );

    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<Engine<C>>);

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

            default_table,
            _class_sheet,
            dirty_list,
            default_state,
        ) = read;

        let (render_objs, engine) = write;
        
        let default_transform = default_table.get::<Transform>().unwrap();
        
        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => {
                    // 如果style_mark不存在， node也一定不存在， 应该删除对应的渲染对象
                    self.remove_render_obj(*id, render_objs);
                    continue;
                },
            };

            let dirty = style_mark.dirty;

            // 不存在BuckgroundColor关心的脏, 跳过
            if dirty & self.dirty_ty == 0 {
                continue;
            }

            // 阴影脏，如果不存在BoxShadow本地样式和class样式， 删除渲染对象
            if dirty & StyleType::BoxShadow as usize != 0 
            && style_mark.local_style & StyleType::BoxShadow as usize == 0 
            && style_mark.class_style & StyleType::BoxShadow as usize == 0 {
                self.remove_render_obj(*id, render_objs);
                continue;
            }

            // 不存在，则创建渲染对象
            let render_index = match self.render_map.get_mut(*id) {
                Some(r) => *r,
                None => self.create_render_obj(*id, 0.0, false, render_objs, default_state),
            };

            // 从组件中取出对应的数据
            let render_obj = unsafe {render_objs.get_unchecked_mut(render_index)};

            let border_radius = border_radiuses.get(*id);
            let layout = unsafe {layouts.get_unchecked(*id)};
            let shadow = unsafe {box_shadows.get_unchecked(*id)};

            // 如果Color脏， 或Opacity脏， 计算is_opacity
            if dirty & StyleType::Opacity as usize != 0
            || dirty & StyleType::BoxShadow as usize != 0 {
                let opacity = unsafe {opacitys.get_unchecked(*id)}.0;
                render_obj.is_opacity = color_is_opacity(opacity, &shadow.color);
            }

            // 如果阴影脏，或者边框半径改变，则重新创建geometry
            if style_mark.dirty & StyleType::BoxShadow as usize != 0
            || style_mark.dirty & StyleType::BorderRadius as usize != 0 {
                render_obj.program_dirty =  true;
                to_ucolor_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut());

                render_obj.paramter.as_ref().set_value("uColor", create_u_color_ubo(&shadow.color, engine));
                render_obj.geometry = create_shadow_geo(engine, layout, shadow, border_radius);
            }

            // 渲染管线脏， 创建渲染管线
            if render_obj.program_dirty {
                render_obj.paramter.as_ref().set_single_uniform("blur", UniformValue::Float1(1.0));
                render_obj.program = Some(engine.create_program(
                    COLOR_VS_SHADER_NAME.get_hash(),
                    COLOR_FS_SHADER_NAME.get_hash(),
                    COLOR_VS_SHADER_NAME.as_ref(),
                    &*render_obj.vs_defines,
                    COLOR_FS_SHADER_NAME.as_ref(),
                    &*render_obj.fs_defines,
                    render_obj.paramter.as_ref(),
                ));
            }
            
            // 矩阵脏，或者布局脏
            if dirty & StyleType::Matrix as usize != 0 
            || dirty & StyleType::Layout as usize != 0 {
                let world_matrix = unsafe { world_matrixs.get_unchecked(*id) };
                let transform =  match transforms.get(*id) {
                    Some(r) => r,
                    None => default_transform,
                };
                let depth = unsafe{z_depths.get_unchecked(*id)}.0;
                modify_matrix(render_obj, depth, world_matrix, transform, layout);
            }
        }
    }
}

impl<C: HalContext + 'static> BoxShadowSys<C> {

    #[inline]
    fn remove_render_obj(&mut self, id: usize, render_objs: &mut SingleCaseImpl<RenderObjs>) {
        match self.render_map.remove(id) {
            Some(index) => {
                let notify = render_objs.get_notify();
                render_objs.remove(index, Some(notify));
            },
            None => ()
        };
    }

    fn create_render_obj(
        &mut self,
        id: usize,
        z_depth: f32,
        visibility: bool,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &DefaultState,
    ) -> usize {
        
        let render_obj = RenderObj {
            depth: z_depth - 0.3,
            depth_diff: -0.3,
            visibility: visibility,
            is_opacity: true,
            vs_name: COLOR_VS_SHADER_NAME.clone(),
            fs_name: COLOR_FS_SHADER_NAME.clone(),
            vs_defines: Box::new(VsDefines::default()),
            fs_defines: Box::new(FsDefines::default()),
            paramter: Share::new(ColorParamter::default()),
            program_dirty: true,

            program: None,
            geometry: None,
            state: State {
                bs: default_state.df_bs.clone(),
                rs: default_state.df_rs.clone(),
                ss: default_state.df_ss.clone(),
                ds: default_state.df_ds.clone(),
            },
            context: id,
        };

        let notify = render_objs.get_notify();
        let index = render_objs.insert(render_obj, Some(notify));
        // 创建RenderObj与Node实体的索引关系， 并设脏
        self.render_map.insert(id, index);
        index
    }
}

#[inline]
fn modify_matrix(
    render_obj: &mut RenderObj,
    depth: f32,
    world_matrix: &WorldMatrix,
    transform: &Transform,
    layout: &Layout) {
        let arr = create_let_top_offset_matrix(layout, world_matrix, transform, 0.0, 0.0, depth);
        render_obj.paramter.set_value("worldMatrix", Share::new(  WorldMatrixUbo::new(UniformValue::MatrixV4(arr)) ));
}

#[inline]
fn color_is_opacity(opacity: f32, color: &CgColor) -> bool {
    opacity == 1.0 && color.a == 1.0
}

#[inline]
fn to_ucolor_defines(vs_defines: &mut dyn Defines, fs_defines: &mut dyn Defines) -> bool {
    match fs_defines.add("UCOLOR") {
        Some(_) => false,
        None => {
            vs_defines.remove("VERTEX_COLOR");
            fs_defines.remove("VERTEX_COLOR");
            true
        },
    }
}

#[inline]
fn create_u_color_ubo<C: HalContext + 'static>(c: &CgColor, engine: &mut Engine<C>) -> Share<dyn UniformBuffer> {
    let h = f32_4_hash(c.r, c.g, c.b, c.a);
    match engine.res_mgr.get::<UColorUbo>(&h) {
        Some(r) => r,
        None => engine.res_mgr.create(h, UColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a))),
    }
}

fn create_shadow_geo<C: HalContext + 'static>(
    engine: &mut Engine<C>,
    layout: &Layout,
    shadow: &BoxShadow,
    border_radius: Option<&BorderRadius>) -> Option<Share<GeometryRes>> {
    
    let radius = cal_border_radius(border_radius, layout);
    let g_b = geo_box(layout);
    if g_b.min.x - g_b.max.x == 0.0 || g_b.min.y - g_b.max.y == 0.0 {
        return None;
    }

    let x = g_b.min.x;
    let y = g_b.min.y;
    let w = g_b.max.x - g_b.min.x;
    let h = g_b.max.y - g_b.min.y;
    let bg = split_by_radius(x, y, w, h, radius.x, None);
    if bg.0.len() == 0 {
        return None;
    }
    
    let x = g_b.min.x + shadow.h;
    let y = g_b.min.y + shadow.v;
    let w = g_b.max.x - g_b.min.x + 2.0 * shadow.spread;
    let h = g_b.max.y - g_b.min.y + 2.0 * shadow.spread;
    let shadow = split_by_radius(x, y, w, h, radius.x, None);
    if shadow.0.len() == 0 {
        return None;
    }
    
    let polygon_shadow = Polygon2d::new(convert_to_point(shadow.0.as_slice()));
    let polygon_bg = Polygon2d::new(convert_to_point(bg.0.as_slice()));
    
    let mut curr_index = 0;
    let mut pts: Vec<f32> = vec![];
    let mut indices: Vec<u16> = vec![];
    for p in Polygon2d::boolean(&polygon_shadow, &polygon_bg, BooleanOperation::Difference) {
        pts.extend_from_slice( convert_to_f32(p.vertices.as_slice()) );
        
        let tri_indices = p.triangulation();
        indices.extend_from_slice(tri_indices.iter().map(|&v| (v + curr_index) as u16).collect::<Vec<u16>>().as_slice());
        
        curr_index += p.vertices.len();
    }

    if pts.len() == 0 {
        return None;
    }

    let geo = create_p_i_geometry(pts.as_slice(), indices.as_slice(), engine);
    Some(Share::new(geo))
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
    }
}