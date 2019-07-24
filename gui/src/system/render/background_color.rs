/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use share::Share;
use std::hash::{ Hasher, Hash };

use ordered_float::NotNan;
use fxhash::FxHasher32;
use map::vecmap::VecMap;

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
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
use system::render::util::*;


lazy_static! {
    static ref GRADUAL: Atom = Atom::from("GRADUAL");
}

pub struct BackgroundColorSys<C: HalContext + 'static>{
    items: Items<usize>,
    share_ucolor_ubo: VecMap<Share<dyn UniformBuffer>>, // 如果存在BackgroundClass， 也存在对应的ubo
    mark: PhantomData<C>,
}

impl<C: HalContext + 'static> Default for BackgroundColorSys<C> {
    fn default() -> Self {
        BackgroundColorSys {
            items: Items::default(),
            share_ucolor_ubo: VecMap::default(),
            mark: PhantomData,
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

        &'a SingleCaseImpl<DefaultTable>,
        &'a SingleCaseImpl<ClassSheet>,
        &'a SingleCaseImpl<UnitQuad>,
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

            default_table,
            class_sheet,
            unit_quad,
        ) = read;
        let (render_objs, engine) = write;
        let default_transform = default_table.get::<Transform>().unwrap();
        for id in self.items.dirtys.iter() {
            let item = match self.items.render_map.get_mut(*id) {
                Some(r) => r,
                None => continue,
            };
            let dirty = item.dirty;
            let render_obj = unsafe {render_objs.get_unchecked_mut(item.index)};
            item.dirty = 0;

            let border_radius = border_radiuses.get(*id);
            let layout = unsafe {layouts.get_unchecked(*id)};
            let color;
            render_obj.program_dirty = render_obj.program_dirty | match background_colors.get(*id) {
                Some(bg_color) => {
                    color = bg_color;
                    // 如果Color脏， 或Opacity脏， 计算is_opacity
                    if dirty & DrityType::Color as usize != 0 || dirty & DrityType::Opacity as usize != 0 {
                        let opacity = unsafe {opacitys.get_unchecked(*id)}.0;
                        render_obj.is_opacity = background_is_opacity(opacity, bg_color);
                    }
                    // 尝试修改颜色， 以及颜色所对应的geo
                    modify_color(render_obj, bg_color, engine, dirty, layout, &unit_quad.0, border_radius)
                },
                None => {
                    let class_id = unsafe { classes.get_unchecked(*id) }.0;
                    let class = unsafe{ class_sheet.class.get_unchecked(class_id) };
                    let bg_color = unsafe { class_sheet.background_color.get_unchecked(class.background_color) };

                    // 如果Color脏， 或Opacity脏， 计算is_opacity
                    if dirty & DrityType::Color as usize != 0 || dirty & DrityType::Opacity as usize != 0 {
                        let opacity = unsafe {opacitys.get_unchecked(*id)}.0;
                        render_obj.is_opacity = background_is_opacity(opacity, bg_color);
                    }

                    color = bg_color;
                    // 尝试修改颜色， 以及颜色所对应的geo
                    modify_class_color(render_obj, bg_color, engine, dirty, &self.share_ucolor_ubo, class_id, layout, &unit_quad.0, border_radius)
                }
            };
            
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
            
            // 如果矩阵脏
            if dirty & DrityType::Matrix as usize != 0 || dirty & DrityType::Layout as usize != 0{
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
            }
        }
        self.items.dirtys.clear();
    }
}

// 插入渲染对象
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BackgroundColor, CreateEvent> for BackgroundColorSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a SingleCaseImpl<DefaultState>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, render_objs: Self::WriteData){
        // 如果已经存在渲染对象，设置颜色脏， 返回
        if self.items.render_map.get(event.id).is_some() {
            self.items.set_dirty(event.id, DrityType::BorderRadius as usize);
            return;
        }

        // 否则创建渲染对象
        let (z_depths, visibilitys, default_state) = read;
        let z_depth = unsafe {z_depths.get_unchecked(event.id)}.0;
        let visibility = unsafe {visibilitys.get_unchecked(event.id)}.0;
        self.create_render_obj(event.id, z_depth, visibility, render_objs, default_state);
    }
}

// 修改渲染对象
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BackgroundColor, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, BackgroundColor>, &'a MultiCaseImpl<Node, Opacity>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &ModifyEvent, _: Self::ReadData, _: Self::WriteData){
        self.items.set_dirty(event.id, DrityType::Color as usize);
    }
}

// 删除渲染对象
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BackgroundColor, DeleteEvent> for BackgroundColorSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ClassName>, 
        &'a SingleCaseImpl<ClassSheet>,  
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (class_names, class_sheet) = read;
        // 如果class中存在backgroundColor， 设置color脏
        let class_name = unsafe { class_names.get_unchecked(event.id) };
        if let Some(class) = class_sheet.class.get(class_name.0) {
            if let Some(_) = class_sheet.background_color.get(class.background_color) {
                self.items.set_dirty(event.id, DrityType::Color as usize);
                return;
            }
        }
        let item = self.items.render_map.remove(event.id).unwrap();
        let notify = render_objs.get_notify();
        render_objs.remove(item.index, Some(notify));
    }
}

// 修改class （不监听class的创建， 应该在创建node的同时创建class， 创建的class没有意义）
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ClassName, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, ClassName>,
        &'a MultiCaseImpl<Node, BackgroundColor>,

        &'a SingleCaseImpl<ClassSheet>,
        &'a SingleCaseImpl<DefaultState>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){

        // 如果class中含有backgrooundColor的描述， 创建一个渲染对象
        let (z_depths, visibilitys,class_names, background_colors, class_sheet, default_state) = read;
        if let Some(class_name) = class_names.get(event.id) {
            if let Some(class) = class_sheet.class.get(class_name.0) {
                if class.background_color > 0 {
                    // 如果已经存在backgroundColor， 设置color脏 返回
                    if background_colors.get(event.id).is_some() {
                        self.items.set_dirty(event.id, DrityType::Color as usize);
                        return;
                    }

                    // 如果不存在渲染对象， 创建渲染对象
                    if self.items.render_map.get(event.id).is_none() {
                        let z_depth = unsafe {z_depths.get_unchecked(event.id)}.0;
                        let visibility = unsafe {visibilitys.get_unchecked(event.id)}.0;
                        self.create_render_obj(event.id, z_depth, visibility, render_objs, default_state);
                    }
   
                    self.items.set_dirty(event.id, DrityType::Color as usize);
                    return;
                }
            }
        }

        if background_colors.get(event.id).is_some()  {
            return;
        }

        // 如果class中不存在backgroundColor， style中也不存在， 应该删除渲染对象
        if let Some(item) = self.items.render_map.remove(event.id) {
            let notify = render_objs.get_notify();
            render_objs.remove(item.index, Some(notify));
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

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Layout, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        self.items.set_dirty(event.id, DrityType::Layout as usize);
    }
}

//圆角修改， 需要重新计算世界矩阵和顶点流
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderRadius, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, _: Self::ReadData, _: Self::WriteData){
        self.items.set_dirty(event.id, DrityType::BorderRadius as usize);
    }
}

//不透明度变化， 设置ubo
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, _: Self::ReadData, _: Self::WriteData){
        self.items.set_dirty(event.id, DrityType::Opacity as usize);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, _: Self::ReadData, _: Self::WriteData){
        self.items.set_dirty(event.id, DrityType::Matrix as usize);
    }
}

impl<C: HalContext + 'static> BackgroundColorSys<C> {

    #[inline]
    fn create_render_obj(
        &mut self,
        id: usize,
        z_depth: f32,
        visibility: bool,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &DefaultState,
    ) -> usize{
        
        let render_obj = RenderObj {
            depth: z_depth - 0.2,
            depth_diff: -0.2,
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
        self.items.create(id, index);
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
                println!("g_b: {:?}, radius.x - g_b.min.x: {}", g_b, radius.x - g_b.min.x);
                let r = split_by_radius(g_b.min.x, g_b.min.y, g_b.max.x - g_b.min.x, g_b.max.y - g_b.min.y, radius.x - g_b.min.x, None);
                println!("r: {:?}", r);
                if r.0.len() == 0 {
                    return None;
                } else {
                    let indices = to_triangle(&r.1, Vec::with_capacity(r.1.len()));
                    println!("indices: {:?}", indices);
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

            let position_len = positions.len();
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
fn geo_box(layout: &Layout) -> Aabb2{
    Aabb2::new(Point2::new(layout.border_left, layout.border_top), Point2::new(layout.width - layout.border_right, layout.height - layout.border_bottom))
}

#[inline]
fn background_is_opacity(opacity: f32, background_color: &BackgroundColor) -> bool{
    if opacity < 1.0 {
        return false;
    }
    background_color.0.is_opaque()
}

#[inline]
fn create_u_color_ubo<C: HalContext + 'static>(c: &CgColor, engine: &mut Engine<C>) -> Share<dyn UniformBuffer> {
    let h = f32_4_hash(c.r, c.g, c.b, c.a);
    match engine.res_mgr.get::<UColorUbo>(&h) {
        Some(r) => r,
        None => engine.res_mgr.create(h, UColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a))),
    }
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
            if dirty & DrityType::Color as usize != 0 {
                change = to_ucolor_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut());
                render_obj.paramter.as_ref().set_value("uColor", create_u_color_ubo(c, engine));
            }

            // 如果颜色类型改变（纯色改为渐变色， 或渐变色改为纯色）或圆角改变， 需要重新创建geometry
            if change || dirty & DrityType::BorderRadius as usize != 0 {    
                render_obj.geometry = create_rgba_geo(border_radius, layout, unit_quad, engine);
            }
        },
        Color::LinearGradient(c) => {
            if dirty & DrityType::Color as usize != 0{
                change = to_vex_color_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut());
            }

            // 如果颜色类型改变（纯色改为渐变色， 或渐变色改为纯色）或圆角改变， 需要重新创建geometry
            if change || dirty & DrityType::BorderRadius as usize != 0 || dirty & DrityType::Layout as usize != 0 {
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
            if dirty & DrityType::Color as usize != 0{
                change = to_ucolor_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut());
                let u_color_ubo = unsafe {share_bg.get_unchecked(class_id)};
                render_obj.paramter.as_ref().set_value("uColor", u_color_ubo.clone());
            }

            // 如果颜色类型改变（纯色改为渐变色， 或渐变色改为纯色）或圆角改变， 需要重新创建geometry
            if change || dirty & DrityType::BorderRadius as usize != 0 {
                render_obj.geometry = create_rgba_geo(border_radius, layout, unit_quad, engine);
            }
        },
        Color::LinearGradient(c) => {
            if dirty & DrityType::Color as usize != 0{
                change = to_vex_color_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut());
            }

            // 如果颜色类型改变（纯色改为渐变色， 或渐变色改为纯色）或圆角改变， 需要重新创建geometry
            if change || dirty & DrityType::BorderRadius as usize != 0 || dirty & DrityType::Layout as usize != 0 {
                render_obj.geometry = create_linear_gradient_geo(c, border_radius, layout, engine);
            }
        },
    };
    change
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
fn to_vex_color_defines(vs_defines: &mut dyn Defines, fs_defines: &mut dyn Defines) -> bool {
    match vs_defines.add("VERTEX_COLOR") {
        Some(_) => false,
        None => {
            fs_defines.add("VERTEX_COLOR");
            fs_defines.remove("UCOLOR");
            true
        }
    }
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
        let arr = create_unit_matrix(
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

enum DrityType {
    Color = 1,
    BorderRadius = 4,
    Matrix = 16,
    Opacity = 32,
    Layout = 64,
}


unsafe impl<C: HalContext + 'static> Sync for BackgroundColorSys<C>{}
unsafe impl<C: HalContext + 'static> Send for BackgroundColorSys<C>{}

impl_system!{
    BackgroundColorSys<C> where [C: HalContext + 'static],
    true,
    {
        MultiCaseListener<Node, BackgroundColor, CreateEvent>
        MultiCaseListener<Node, BackgroundColor, ModifyEvent>
        MultiCaseListener<Node, BackgroundColor, DeleteEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
        MultiCaseListener<Node, WorldMatrix, ModifyEvent>
        MultiCaseListener<Node, BorderRadius, ModifyEvent>
        MultiCaseListener<Node, ClassName, ModifyEvent>
    }
}