/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use std::sync::Arc;
use std::mem::transmute;

use std::collections::HashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use ecs::idtree::{ IdTree};
use map::{ vecmap::VecMap, Map } ;
use hal_core::{Context, Uniforms, RasterState, BlendState, StencilState, DepthState, BlendFunc, CullMode, ShaderType, Pipeline, Geometry, AttributeName};
use atom::Atom;
use polygon::*;

use component::user::*;
use component::calc::{Visibility, WorldMatrix, Opacity, ByOverflow, ZDepth};
use entity::{Node};
use single::{RenderObjs, RenderObjWrite, RenderObj, ViewMatrix, ProjectionMatrix, ClipUbo, ViewUbo, ProjectionUbo};
use render::engine::{ Engine , PipelineInfo};
use system::util::*;
use system::util::constant::{PROJECT_MATRIX, WORLD_MATRIX, VIEW_MATRIX, POSITION, COLOR, CLIP_INDEICES, ALPHA, CLIP, VIEW, PROJECT, WORLD, COMMON};
use system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};


lazy_static! {
    static ref UCOLOR: Atom = Atom::from("UCOLOR");
    static ref VERTEX_COLOR: Atom = Atom::from("VERTEX_COLOR");

    static ref BLUR: Atom = Atom::from("blur");
    static ref U_COLOR: Atom = Atom::from("uColor");
}

pub struct BackgroundColorSys<C: Context + Share>{
    background_color_render_map: VecMap<Item>,
    position_dirtys: Vec<usize>,
    mark: PhantomData<C>,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
    pipelines: HashMap<u64, Arc<PipelineInfo>>,
}

impl<C: Context + Share> BackgroundColorSys<C> {
    pub fn new() -> Self{
        BackgroundColorSys {
            background_color_render_map: VecMap::default(),
            position_dirtys: Vec::new(),
            mark: PhantomData,
            rs: Arc::new(RasterState::new()),
            bs: Arc::new(BlendState::new()),
            ss: Arc::new(StencilState::new()),
            ds: Arc::new(DepthState::new()),
            pipelines: HashMap::default(),
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流和颜色属性流
impl<'a, C: Context + Share> Runner<'a> for BackgroundColorSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, BackgroundColor>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn run(&mut self, read: Self::ReadData, render_objs: Self::WriteData){
        let map = &mut self.background_color_render_map;
        let (layouts, border_radius, z_depths, background_colors) = read;
        for id in  self.position_dirtys.iter() {
            let item = unsafe { map.get_unchecked_mut(*id) };
            item.position_change = false;
            let border_radius = unsafe { border_radius.get_unchecked(*id) };
            let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;
            let layout = unsafe { layouts.get_unchecked(*id) };
            let background_color = unsafe { background_colors.get_unchecked(*id) };
            let (positions, indices, colors) = get_geo_flow(border_radius, layout, z_depth - 0.1, background_color);

            let mut render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
            let geometry = Arc::get_mut(&mut render_obj.geometry).unwrap();

            let vertex_count: u32 = (positions.len()/3) as u32;
            if vertex_count != geometry.get_vertex_count() {
                geometry.set_vertex_count(vertex_count);
            }
            geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false);
            geometry.set_indices_short(indices.as_slice(), false);
            match colors {
                Some(colors) => geometry.set_attribute(&AttributeName::Color, 4, Some(colors.as_slice()), false),
                None => geometry.set_attribute(&AttributeName::Color, 4, None, false),
            };
        }
        self.position_dirtys.clear();
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BackgroundColor, CreateEvent> for BackgroundColorSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, BackgroundColor>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, Opacity>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs<C>>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (background_colors, border_radius, z_depths, layouts, opacitys) = read;
        let (render_objs, engine) = write;
        let background_color = unsafe { background_colors.get_unchecked(event.id) };
        let border_radius = unsafe { border_radius.get_unchecked(event.id) };
        let z_depth = unsafe { z_depths.get_unchecked(event.id) }.0;
        let layout = unsafe { layouts.get_unchecked(event.id) };
        let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;

        let mut geometry = create_geometry(&mut engine.gl);
        let mut ubos: HashMap<Atom, Arc<Uniforms<C>>> = HashMap::default();
        let mut defines = Vec::new();

        let mut common_ubo = engine.gl.create_uniforms();
        common_ubo.set_float_1(&BLUR, 1.0);
        common_ubo.set_float_1(&ALPHA, opacity);
        match &background_color.0 {
            Color::RGBA(c) => {
                common_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b,c.a);
                defines.push(U_COLOR.clone());
            },
            Color::LinearGradient(_) => {
                defines.push(VERTEX_COLOR.clone());
            },
            _ => (),
        }
        ubos.insert(COMMON.clone(), Arc::new(common_ubo)); // COMMON

        let pipeline = engine.create_pipeline(
            0,
            &COLOR_VS_SHADER_NAME.clone(),
            &COLOR_FS_SHADER_NAME.clone(),
            &[UCOLOR.clone()],
            self.rs.clone(),
            self.bs.clone(),
            self.ss.clone(),
            self.ds.clone(),
        );
        
        let is_opacity = background_is_opacity(opacity, background_color);
        let render_obj: RenderObj<C> = RenderObj {
            depth: z_depth - 1.0,
            visibility: false,
            is_opacity: is_opacity,
            ubos: ubos,
            geometry: geometry,
            pipeline: pipeline.clone(),
            context: event.id,
            defines: defines,
        };

        let notify = render_objs.get_notify();
        let index = render_objs.insert(render_obj, Some(notify));
        self.background_color_render_map.insert(event.id, Item{index: index, position_change: true});
        self.position_dirtys.push(event.id);
    }
}

// 删除渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BackgroundColor, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, BackgroundColor>, &'a MultiCaseImpl<Node, Opacity>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (background_colors, opacitys) = read;
        let item = unsafe { self.background_color_render_map.get_unchecked_mut(event.id) };
        let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
   
        let background_color = unsafe { background_colors.get_unchecked(event.id) };
        match &background_color.0 {
            Color::RGBA(c) => {
                // 设置ubo
                let common_ubo = Arc::make_mut(render_obj.ubos.get_mut(&COMMON).unwrap());
                common_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b, c.a);
                // 如果未找到UCOLOR宏， 表示修改之前的颜色不为RGBA， 应该删除VERTEX_COLOR宏， 添加UCOLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
                if find_item_from_vec(&render_obj.defines, &UCOLOR) == 0 {
                    render_obj.defines.remove_item(&VERTEX_COLOR);
                    render_obj.defines.push(UCOLOR.clone());
                    if item.position_change == false {
                        item.position_change = true;
                        self.position_dirtys.push(event.id);
                    }
                }
            },
            Color::LinearGradient(_) => {
                // 如果未找到VERTEX_COLOR宏， 表示修改之前的颜色不为LinearGradient， 应该删除UCOLOR宏， 添加VERTEX_COLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
                if find_item_from_vec(&render_obj.defines, &VERTEX_COLOR) == 0 {
                    render_obj.defines.remove_item(&UCOLOR);
                    render_obj.defines.push(VERTEX_COLOR.clone());      
                    if item.position_change == false {
                        item.position_change = true;
                        self.position_dirtys.push(event.id);
                    }
                }
            },
            _ => (),
        }

        let opacity = unsafe { opacitys.get_unchecked(event.id).0 };
        let is_opacity = background_is_opacity(opacity, background_color);
        let notify = render_objs.get_notify();
        unsafe { render_objs.get_unchecked_write(item.index, &notify).set_is_opacity(is_opacity)};
    }
}

// 删除渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BackgroundColor, DeleteEvent> for BackgroundColorSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, render_objs: Self::WriteData){
        let item = self.background_color_render_map.remove(event.id).unwrap();
        let notify = render_objs.get_notify();
        render_objs.remove(item.index, Some(notify));
        if item.position_change == true {
            self.position_dirtys.remove_item(&event.id);
        }
    }
}

//布局修改， 需要重新计算顶点
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Layout, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        if let Some(item) = unsafe { self.background_color_render_map.get_mut(event.id) } {
            if item.position_change == false {
                item.position_change = true;
                self.position_dirtys.push(event.id);
            }
        };
    }
}

//不透明度变化， 设置ubo
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, BackgroundColor>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        self.change_is_opacity(event.id, read.0, read.1, write);
    }
}

impl<'a, C: Context + Share> BackgroundColorSys<C> {
    fn change_is_opacity(&mut self, id: usize, opacitys: &MultiCaseImpl<Node, Opacity>, colors: &MultiCaseImpl<Node, BackgroundColor>, render_objs: &mut SingleCaseImpl<RenderObjs<C>>){
        if let Some(item) = unsafe { self.background_color_render_map.get_mut(id) } {
            let opacity = unsafe { opacitys.get_unchecked(id).0 };
            let background_color = unsafe { colors.get_unchecked(id) };

            let is_opacity = background_is_opacity(opacity, background_color);

            let notify = render_objs.get_notify();
            unsafe { render_objs.get_unchecked_write(item.index, &notify).set_is_opacity(is_opacity)};
        }
    }
}

struct Item {
    index: usize,
    position_change: bool,
}

//取集合体的顶点流和索引流和color属性流
fn get_geo_flow(radius: &BorderRadius, layout: &Layout, z_depth: f32, color: &BackgroundColor) -> (Vec<f32>, Vec<u16>, Option<Vec<f32>>) {
    let radius = cal_border_radius(radius, layout);
    let start = layout.border;
    let end_x = layout.width - layout.border;
    let end_y = layout.height - layout.border;
    let mut positions;
    if radius.x == 0.0 {
        positions = vec![
            start, start, z_depth, // left_top
            start, end_y, z_depth, // left_bootom
            end_x, end_y, z_depth, // right_bootom
            end_x, start, z_depth, // right_top
        ];
    } else {
        positions = split_by_radius(start, start, end_x - layout.border, end_y - layout.border, radius.x - layout.border, z_depth);
    }
    match &color.0 {
        &Color::RGBA(_) => {
            let indices = create_increase_vec(positions.len());
            (positions, to_triangle(indices.as_slice()), None)
        },
        &Color::LinearGradient(ref bg_colors) => {
            let mut lg_pos = Vec::with_capacity(bg_colors.list.len());
            let mut color = Vec::with_capacity(bg_colors.list.len() * 4);
            for v in bg_colors.list.iter() {
                lg_pos.push(v.position);
                color.extend_from_slice(&[v.rgba.r, v.rgba.g, v.rgba.b, v.rgba.a]);
            }

            //渐变端点
            let endp = find_lg_endp(&[0.0, 0.0, layout.width, layout.height], bg_colors.direction);
            let (positions, indeices) = split_by_lg(positions, lg_pos.as_slice(), endp.0.clone(), endp.1.clone());
            let mut colors = interp_by_lg(positions.as_slice(), vec![LgCfg{unit:4, data: color}], lg_pos.as_slice(), endp.0, endp.0);
            let colors = colors.pop().unwrap();
            (positions, indeices, Some(colors))
        },
    }
}

fn background_is_opacity(is_opacity: f32, background_color: &BackgroundColor) -> bool{
    if is_opacity < 1.0 {
        return false;
    }
    match &background_color.0 {
        &Color::RGBA(ref c) => if c.a < 1.0 {
            return false;
        },
        &Color::LinearGradient(ref r) => {
            for c in r.list.iter() {
                if c.rgba.a < 1.0 {
                    return false;
                }
            }
        },
    };
    return true;
}

unsafe impl<C: Context + Share> Sync for BackgroundColorSys<C>{}
unsafe impl<C: Context + Share> Send for BackgroundColorSys<C>{}

impl_system!{
    BackgroundColorSys<C> where [C: Context + Share],
    false,
    {
        MultiCaseListener<Node, BackgroundColor, CreateEvent>
        MultiCaseListener<Node, BackgroundColor, ModifyEvent>
        MultiCaseListener<Node, BackgroundColor, DeleteEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
    }
}