/**
 * Box Shadow
 */
use share::Share;
use std::slice;
use std::marker::PhantomData;
use ecs::{ SingleCaseImpl, MultiCaseImpl, MultiCaseListener, DeleteEvent, Runner };
use map::{ vecmap::VecMap } ;
use hal_core::*;
use polygon::*;
use cg2d::{Point2, Polygon as Polygon2d, BooleanOperation};
use component::user::*;
use component::calc::*;
use component::calc::{ Opacity };
use entity::{ Node };
use single::*;
use render::engine::{ ShareEngine, Engine, AttributeDecs };
use render::res::{GeometryRes};
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

    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<ShareEngine<C>>);

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

        if dirty_list.0.len() == 0 {
            return;
        }

        let (render_objs, engine) = write;
        let notify = render_objs.get_notify();
        let default_transform = default_table.get::<Transform>().unwrap();
        
        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => {
                    self.remove_render_obj(*id, render_objs);
                    continue
                },
            };

            let dirty = style_mark.dirty;

            // 不存在BuckgroundColor关心的脏, 跳过
            if dirty & self.dirty_ty == 0 {
                continue;
            }

            // 阴影脏
            let render_index = if dirty & StyleType::BoxShadow as usize != 0 {
                // 如果不存在BoxShadow本地样式和class样式， 删除渲染对象
                if style_mark.local_style & StyleType::BoxShadow as usize == 0 && style_mark.class_style & StyleType::BoxShadow as usize == 0 {
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
            let render_obj = unsafe {render_objs.get_unchecked_mut(render_index)};

            let border_radius = border_radiuses.get(*id);
            let layout = unsafe {layouts.get_unchecked(*id)};
            let shadow = unsafe {box_shadows.get_unchecked(*id)};

            // 如果Color脏， 或Opacity脏， 计算is_opacity
            if dirty & StyleType::Opacity as usize != 0
            || dirty & StyleType::BoxShadow as usize != 0 {
                let opacity = unsafe {opacitys.get_unchecked(*id)}.0;
                render_obj.is_opacity = color_is_opacity(opacity, &shadow.color, shadow.blur);
                notify.modify_event(render_index, "is_opacity", 0);
                modify_opacity(engine, render_obj);
            }
            
            // 如果阴影脏，或者边框半径改变，则重新创建geometry
            if style_mark.dirty & StyleType::BoxShadow as usize != 0
            || style_mark.dirty & StyleType::BorderRadius as usize != 0 || style_mark.dirty & StyleType::Layout as usize != 0{
                render_obj.program_dirty =  true;
                to_ucolor_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut());

                render_obj.paramter.as_ref().set_value("uColor", engine.create_u_color_ubo(&shadow.color));
                render_obj.geometry = create_shadow_geo(engine, render_obj, layout, shadow, border_radius);
            }

            // // 渲染管线脏， 创建渲染管线
            // if render_obj.program_dirty {                
            //     render_obj.program = Some(ProgramRes(engine.create_program(
            //         COLOR_VS_SHADER_NAME.get_hash(),
            //         COLOR_FS_SHADER_NAME.get_hash(),
            //         COLOR_VS_SHADER_NAME.as_ref(),
            //         &*render_obj.vs_defines,
            //         COLOR_FS_SHADER_NAME.as_ref(),
            //         &*render_obj.fs_defines,
            //         render_obj.paramter.as_ref(),
            //     )));
            // }
            
            // 矩阵脏，或者布局脏
            if dirty & StyleType::Matrix as usize != 0 
            || dirty & StyleType::Layout as usize != 0{
                let world_matrix = unsafe { world_matrixs.get_unchecked(*id) };
                let transform =  match transforms.get(*id) {
                    Some(r) => r,
                    None => default_transform,
                };
                let depth = unsafe{z_depths.get_unchecked(*id)}.0;
                modify_matrix(render_obj, depth, world_matrix, transform, layout);
            }
            notify.modify_event(render_index, "", 0);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BoxShadow, DeleteEvent> for BoxShadowSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, render_objs: Self::WriteData){
        self.remove_render_obj(event.id, render_objs)
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

    #[inline]
    fn create_render_obj(
        &mut self,
        id: usize,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &DefaultState,
    ) -> usize {
        create_render_obj(
            id,
            -0.3,
            true,
            COLOR_VS_SHADER_NAME.clone(),
            COLOR_FS_SHADER_NAME.clone(),
            Share::new(ColorParamter::default()),
            default_state, render_objs,
            &mut self.render_map
        )
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
fn color_is_opacity(opacity: f32, color: &CgColor, blur: f32) -> bool {
    opacity == 1.0 && color.a == 1.0 && blur == 0.0
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

fn create_shadow_geo<C: HalContext + 'static>(
    engine: &mut Engine<C>,
    render_obj: &mut RenderObj,
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
    let bg = split_by_radius(x, y, w, h, radius.x, Some(16));
    if bg.0.len() == 0 {
        return None;
    }
    
    let x = g_b.min.x + shadow.h - shadow.spread - shadow.blur;
    let y = g_b.min.y + shadow.v - shadow.spread - shadow.blur;
    let w = g_b.max.x - g_b.min.x + 2.0 * shadow.spread + 2.0 * shadow.blur;
    let h = g_b.max.y - g_b.min.y + 2.0 * shadow.spread + 2.0 * shadow.blur;
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
        pts.extend_from_slice( convert_to_f32(p.vertices.as_slice()) );
        
        let tri_indices = p.triangulation();
        indices.extend_from_slice(tri_indices.iter().map(|&v| (v + curr_index) as u16).collect::<Vec<u16>>().as_slice());
        
        curr_index += p.vertices.len();
    }

    if pts.len() == 0 {
        return None;
    }
    
    render_obj.paramter.as_ref().set_single_uniform("blur", UniformValue::Float1(if shadow.blur > 0.0 { shadow.blur } else { 1.0 } ));

    if shadow.blur > 0.0 {
        render_obj.vs_defines.add("BOX_SHADOW_BLUR");
        render_obj.fs_defines.add("BOX_SHADOW_BLUR");
        
        let x = g_b.min.x + shadow.h - shadow.spread;
        let y = g_b.min.y + shadow.v - shadow.spread;
        let w = g_b.max.x - g_b.min.x + 2.0 * shadow.spread;
        let h = g_b.max.y - g_b.min.y + 2.0 * shadow.spread;
        render_obj.paramter.as_ref().set_single_uniform("uRect", UniformValue::Float4(x, y, x + w, y + h));
    } else {
        render_obj.fs_defines.add("BOX_SHADOW_BLUR");
        render_obj.fs_defines.remove("BOX_SHADOW_BLUR");
    }
    
    debug_println!("create_shadow_geo: pts = {:?}, indices = {:?}", pts.as_slice(), indices.as_slice());
    Some(engine.create_geo_res(0, indices.as_slice(), &[AttributeDecs::new(AttributeName::Position, pts.as_slice(), 2)]))
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