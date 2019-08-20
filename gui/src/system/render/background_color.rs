/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use share::Share;
use std::hash::{ Hasher, Hash };
use std::marker::PhantomData;

use ordered_float::NotNan;
use fxhash::FxHasher32;
use map::vecmap::VecMap;

use ecs::{CreateEvent, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, MultiCaseListener, DeleteEvent, Runner};
use hal_core::*;
use atom::Atom;
use polygon::*;

use component::user::*;
use component::calc::*;
use component::calc::{Opacity};
use entity::*;
use single::*;
use render::engine::Engine;
use render::res::*;
use system::util::*;
use system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};

lazy_static! {
    static ref GRADUAL: Atom = Atom::from("GRADUAL");
}

pub struct BackgroundColorSys<C: HalContext + 'static>{
    render_map: VecMap<usize>,
    dirty_ty: usize,
    share_ucolor_ubo: VecMap<Share<dyn UniformBuffer>>, // 如果存在BackgroundClass， 也存在对应的ubo
    marker: std::marker::PhantomData<C>,
}

impl<C: HalContext + 'static> Default for BackgroundColorSys<C> {
    fn default() -> Self {
        BackgroundColorSys {
            render_map: VecMap::default(),
            dirty_ty: StyleType::BackgroundColor as usize | StyleType::Matrix as usize | StyleType::BorderRadius as usize | StyleType::Opacity as usize | StyleType::Layout as usize,
            share_ucolor_ubo: VecMap::default(),
            marker: PhantomData,
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for BackgroundColorSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, BackgroundColor>,
        &'a MultiCaseImpl<Node, ClassName>,
        &'a MultiCaseImpl<Node, StyleMark>,

        &'a SingleCaseImpl<DefaultTable>,
        &'a SingleCaseImpl<ClassSheet>,
        &'a SingleCaseImpl<UnitQuad>,
        &'a SingleCaseImpl<DirtyList>,
        &'a SingleCaseImpl<DefaultState>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<Engine<C>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let (
            layouts,
            z_depths,
            world_matrixs,
            transforms,
            opacitys,
            border_radiuses,
            background_colors,
            classes,
            style_marks,

            default_table,
            _class_sheet,
            unit_quad,
            dirty_list,
            default_state,
        ) = read;
        let (render_objs, engine) = write;
        let default_transform = default_table.get::<Transform>().unwrap();
        let notify = render_objs.get_notify();
        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => continue,
            };

            let dirty = style_mark.dirty;
        
            // 不存在BuckgroundColor关心的脏, 跳过
            if dirty & self.dirty_ty == 0 {
                continue;
            }

            // 背景颜色脏， 如果不存在BacgroundColor的本地样式和class样式， 删除渲染对象
            let (render_index, color) = if dirty & StyleType::BackgroundColor as usize != 0 {
                if style_mark.local_style & StyleType::BackgroundColor as usize == 0 && style_mark.class_style & StyleType::BackgroundColor as usize == 0 {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                } else {
                    let render_index = match self.render_map.get_mut(*id) {
                        Some(r) => *r,
                        None => self.create_render_obj(*id, render_objs, default_state),
                    };
                    ( render_index, unsafe {background_colors.get_unchecked(*id)} )
                } 
            } else {
                let render_index = match self.render_map.get_mut(*id) {
                    Some(r) => *r,
                    None => continue,
                };
                ( render_index, unsafe {background_colors.get_unchecked(*id)} )
            };
            
            let render_obj = unsafe {render_objs.get_unchecked_mut(render_index)};
            let border_radius = border_radiuses.get(*id);
            let layout = unsafe {layouts.get_unchecked(*id)};

            // 如果Color脏， 或Opacity脏， 计算is_opacity
            if dirty & StyleType::BackgroundColor as usize != 0 || dirty & StyleType::Opacity as usize != 0 {
                let opacity = unsafe {opacitys.get_unchecked(*id)}.0;
                render_obj.is_opacity = background_is_opacity(opacity, color);
                notify.modify_event(render_index, "is_opacity", 0);
                modify_opacity(engine, render_obj);
            }

            let program_dirty = if style_mark.local_style & StyleType::BackgroundColor as usize != 0{ 
                // 尝试修改颜色， 以及颜色所对应的geo
                modify_color(render_obj, color, engine, dirty, layout, &unit_quad.0, border_radius)
            } else {
                let class_id = unsafe { classes.get_unchecked(*id) }.0;
                // 尝试修改颜色， 以及颜色所对应的geo
                modify_class_color(render_obj, color, engine, dirty, &self.share_ucolor_ubo, class_id, layout, &unit_quad.0, border_radius)
            };
            
            // program管线脏, 通知
            if program_dirty {
                notify.modify_event(render_index, "program_dirty", 0);
            }

            // 如果矩阵脏
            if dirty & StyleType::Matrix as usize != 0 || dirty & StyleType::Layout as usize != 0{
                let world_matrix = unsafe{world_matrixs.get_unchecked(*id)};
                let transform =  match transforms.get(*id) {
                    Some(r) => r,
                    None => default_transform,
                };
                let depth = unsafe{z_depths.get_unchecked(*id)}.0;
                let is_unit_geo = match &color.0 {
                    Color::RGBA(_) => {
                        let radius = cal_border_radius(border_radius, layout);
                        let g_b = geo_box(layout);
                        if radius.x <= g_b.min.x {
                            true
                        } else {
                            false
                        }
                    },
                    Color::LinearGradient(_) => false,
                };
                modify_matrix(render_obj, depth, world_matrix, transform, layout, is_unit_geo);
                notify.modify_event(render_index, "ubos", 0);
            }
        }
    }
}

// 监听一个backgroundColorClass的创建， 如果backgroundColor是rgba类型， 创建一个对应的ubo
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, ClassSheet, CreateEvent> for BackgroundColorSys<C>{
    type ReadData = &'a SingleCaseImpl<ClassSheet>;
    type WriteData = &'a mut SingleCaseImpl<Engine<C>>;
    fn listen(&mut self, event: &CreateEvent, class_sheet: Self::ReadData, engine: Self::WriteData){
        let class = unsafe { class_sheet.class.get_unchecked(event.id)};

        if class.background_color > 0 {
            if let Color::RGBA(c) = unsafe { &class_sheet.background_color.get_unchecked(class.background_color).0 } {
                self.share_ucolor_ubo.insert(event.id, create_u_color_ubo(c, engine));
            }
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BackgroundColor, DeleteEvent> for BackgroundColorSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, render_objs: Self::WriteData){
        self.remove_render_obj(event.id, render_objs)
    }
}

impl<C: HalContext + 'static> BackgroundColorSys<C> {
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
    ) -> usize{
        let index = create_render_obj(
            id,
            -0.2,
            true,
            COLOR_VS_SHADER_NAME.clone(),
            COLOR_FS_SHADER_NAME.clone(),
            Share::new(ColorParamter::default()),
            default_state, render_objs,
            &mut self.render_map
        );
        unsafe{ render_objs.get_unchecked_mut(index) }.paramter.as_ref().set_single_uniform("blur", UniformValue::Float1(1.0));
        index
    }
}

fn create_rgba_geo<C: HalContext + 'static>(border_radius: Option<&BorderRadius>, layout: &Layout, unit_quad: &Share<GeometryRes>, engine: &mut Engine<C>) -> Option<Share<GeometryRes>>{
    let radius = cal_border_radius(border_radius, layout);
    let g_b = geo_box(layout);
    if g_b.min.x - g_b.max.x == 0.0 || g_b.min.y - g_b.max.y == 0.0 {
        return None;
    }

    if radius.x <= g_b.min.x {
        return Some(unit_quad.clone());
    } else {
        let mut hasher = FxHasher32::default();
        radius_quad_hash(&mut hasher, radius.x, layout.width, layout.height);
        let hash = hasher.finish();
        match engine.res_mgr.get::<GeometryRes>(&hash) {
            Some(r) => Some(r.clone()),
            None => {
                let r = split_by_radius(g_b.min.x, g_b.min.y, g_b.max.x - g_b.min.x, g_b.max.y - g_b.min.y, radius.x - g_b.min.x, None);
                if r.0.len() == 0 {
                    return None;
                } else {
                    let indices = to_triangle(&r.1, Vec::with_capacity(r.1.len()));
                    // 创建geo， 设置attribut
                    let positions = create_buffer(&engine.gl, BufferType::Attribute, r.0.len(), Some(BufferData::Float(r.0.as_slice())), false);
                    let indices = create_buffer(&engine.gl, BufferType::Indices, indices.len(), Some(BufferData::Short(indices.as_slice())), false);
                    let geo = create_geometry(&engine.gl);
                    engine.gl.geometry_set_vertex_count(&geo, (r.0.len()/2) as u32);
                    engine.gl.geometry_set_attribute(&geo, &AttributeName::Position, &positions, 2).unwrap();
                    engine.gl.geometry_set_indices_short(&geo, &indices).unwrap();

                    // 创建缓存
                    let geo_res = GeometryRes{geo: geo, buffers: vec![Share::new(positions), Share::new(indices)]};
                    let share_geo = engine.res_mgr.create(hash, geo_res);
                    return Some(share_geo);
                }
            }
        }
    }
}

fn create_linear_gradient_geo<C: HalContext + 'static>(color: &LinearGradientColor, border_radius: Option<&BorderRadius>, layout: &Layout, engine: &mut Engine<C>) -> Option<Share<GeometryRes>>{
    let radius = cal_border_radius(border_radius, layout);
    let g_b = geo_box(layout);
    if g_b.min.x - g_b.max.x == 0.0 || g_b.min.y - g_b.max.y == 0.0 {
        return None;
    }

    // 圆角 + 渐变hash
    let mut hasher = FxHasher32::default();
    GRADUAL.hash(&mut hasher);
    unsafe { NotNan::unchecked_new(color.direction).hash(&mut hasher) };
    for c in color.list.iter() {
        unsafe { NotNan::unchecked_new(c.position).hash(&mut hasher) };
        unsafe { NotNan::unchecked_new(c.rgba.r).hash(&mut hasher) };
        unsafe { NotNan::unchecked_new(c.rgba.g).hash(&mut hasher) };
        unsafe { NotNan::unchecked_new(c.rgba.b).hash(&mut hasher) };
        unsafe { NotNan::unchecked_new(c.rgba.a).hash(&mut hasher) };
    }
    unsafe { NotNan::unchecked_new(color.direction).hash(&mut hasher)};
    radius_quad_hash(&mut hasher, radius.x - g_b.min.x, g_b.max.x - g_b.min.x, g_b.max.y - g_b.min.y);
    let hash = hasher.finish();

    match engine.res_mgr.get::<GeometryRes>(&hash) {
        Some(r) => Some(r.clone()),
        None => {
            let (positions, indices) = if radius.x <= g_b.min.x {
                (
                    vec![
                        g_b.min.x, g_b.min.y, // left_top
                        g_b.min.x, g_b.max.y, // left_bootom
                        g_b.max.x, g_b.max.y, // right_bootom
                        g_b.max.x, g_b.min.y, // right_top
                    ],
                    vec![0, 1, 2, 3]
                )
            }else {
                split_by_radius(g_b.min.x, g_b.min.y, g_b.max.x - g_b.min.x, g_b.max.y - g_b.min.y, radius.x - g_b.min.x, None)
            };
                
            let mut lg_pos = Vec::with_capacity(color.list.len());
            let mut colors = Vec::with_capacity(color.list.len() * 4);
            for v in color.list.iter() {
                lg_pos.push(v.position);
                colors.extend_from_slice(&[v.rgba.r, v.rgba.g, v.rgba.b, v.rgba.a]);
            }

            //渐变端点
            let endp = find_lg_endp(&[
                0.0, 0.0,
                0.0, layout.height,
                layout.width, layout.height,
                layout.width, 0.0,
            ], color.direction);
            let (positions, indices_arr) = split_by_lg(positions, indices, lg_pos.as_slice(), endp.0.clone(), endp.1.clone());
            let mut colors = interp_mult_by_lg(positions.as_slice(), &indices_arr, vec![Vec::new()], vec![LgCfg{unit:4, data: colors}], lg_pos.as_slice(), endp.0, endp.1);
            let indices = mult_to_triangle(&indices_arr, Vec::new());
            let colors = colors.pop().unwrap();

            // 创建geo， 设置attribut
            let positions = create_buffer(&engine.gl, BufferType::Attribute, positions.len(), Some(BufferData::Float(positions.as_slice())), false);
            let colors = create_buffer(&engine.gl, BufferType::Attribute, colors.len(), Some(BufferData::Float(colors.as_slice())), false);
            let indices = create_buffer(&engine.gl, BufferType::Indices, indices.len(), Some(BufferData::Short(indices.as_slice())), false);
            let geo = create_geometry(&engine.gl);
            engine.gl.geometry_set_attribute(&geo, &AttributeName::Position, &positions, 2).unwrap();
            engine.gl.geometry_set_attribute(&geo, &AttributeName::Color, &colors, 4).unwrap();
            engine.gl.geometry_set_indices_short(&geo, &indices).unwrap();

            // 创建缓存
            let geo_res = GeometryRes{geo: geo, buffers: vec![Share::new(positions), Share::new(indices)]};
            let share_geo = engine.res_mgr.create(hash, geo_res);
            return Some(share_geo);
        }
    }
}

#[inline]
fn background_is_opacity(opacity: f32, background_color: &BackgroundColor) -> bool{
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
    layout: &Layout,
    unit_quad: &Share<GeometryRes>,
    border_radius: Option<&BorderRadius>,
) -> bool {
    let mut change = false;
    match &background_color.0 {
        Color::RGBA(c) => {
            if dirty & StyleType::BackgroundColor as usize != 0 {
                change = to_ucolor_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut());
                render_obj.paramter.as_ref().set_value("uColor", create_u_color_ubo(c, engine));
            }
            // 如果颜色类型改变（纯色改为渐变色， 或渐变色改为纯色）或圆角改变， 需要重新创建geometry
            if change || dirty & StyleType::BorderRadius as usize != 0 {    
                render_obj.geometry = create_rgba_geo(border_radius, layout, unit_quad, engine);
            }
        },
        Color::LinearGradient(c) => {
            if dirty & StyleType::BackgroundColor as usize != 0{
                change = to_vex_color_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut());
            }

            // 如果颜色类型改变（纯色改为渐变色， 或渐变色改为纯色）或圆角改变， 需要重新创建geometry
            if change || dirty & StyleType::BorderRadius as usize != 0 || dirty & StyleType::Layout as usize != 0 {
                render_obj.geometry = create_linear_gradient_geo(c, border_radius, layout, engine);
            }
        },
    };
    change
}

// class bgcolor
#[inline]
fn modify_class_color<C: HalContext + 'static>(
    render_obj: &mut RenderObj,
    background_color: &BackgroundColor,
    engine: &mut Engine<C>,
    dirty: usize,
    share_bg: &VecMap<Share<dyn UniformBuffer>>,
    class_id: usize,
    layout: &Layout,
    unit_quad: &Share<GeometryRes>,
    border_radius: Option<&BorderRadius>,
) -> bool {
    let mut change = false;
    match &background_color.0 {
        Color::RGBA(_) => {
            if dirty & StyleType::BackgroundColor as usize != 0{
                change = to_ucolor_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut());
                let u_color_ubo = unsafe {share_bg.get_unchecked(class_id)};
                render_obj.paramter.as_ref().set_value("uColor", u_color_ubo.clone());
            }

            // 如果颜色类型改变（纯色改为渐变色， 或渐变色改为纯色）或圆角改变， 需要重新创建geometry
            if change || dirty & StyleType::BorderRadius as usize != 0 {
                render_obj.geometry = create_rgba_geo(border_radius, layout, unit_quad, engine);
            }
        },
        Color::LinearGradient(c) => {
            if dirty & StyleType::BackgroundColor as usize != 0{
                change = to_vex_color_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut());
            }

            // 如果颜色类型改变（纯色改为渐变色， 或渐变色改为纯色）或圆角改变， 需要重新创建geometry
            if change || dirty & StyleType::BorderRadius as usize != 0 || dirty & StyleType::Layout as usize != 0 {
                render_obj.geometry = create_linear_gradient_geo(c, border_radius, layout, engine);
            }
        },
    };
    change
}

#[inline]
fn modify_matrix(
    render_obj: &mut RenderObj,
    depth: f32,
    world_matrix: &WorldMatrix,
    transform: &Transform,
    layout: &Layout,
    is_unity_geo: bool,
){
    if is_unity_geo {
        let arr = create_unit_matrix_by_layout(
            layout,
            world_matrix,
            transform,
            depth,
        );
        
        render_obj.paramter.set_value("worldMatrix", Share::new( WorldMatrixUbo::new(UniformValue::MatrixV4(arr)) ));
    } else {
        let arr = create_let_top_offset_matrix(layout, world_matrix, transform, 0.0, 0.0, depth);
        render_obj.paramter.set_value("worldMatrix", Share::new(  WorldMatrixUbo::new(UniformValue::MatrixV4(arr)) ));
    }
}

impl_system!{
    BackgroundColorSys<C> where [C: HalContext + 'static],
    true,
    {
        SingleCaseListener<ClassSheet, CreateEvent>
        MultiCaseListener<Node, BackgroundColor, DeleteEvent>
    }
}