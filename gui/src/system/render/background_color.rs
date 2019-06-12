/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use std::sync::Arc;

use fnv::FnvHashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use map::{ vecmap::VecMap } ;
use hal_core::{Context, Uniforms, RasterState, BlendState, StencilState, DepthState, Geometry, AttributeName};
use atom::Atom;
use polygon::*;

use component::user::*;
use component::calc::{Opacity, ZDepth};
use entity::{Node};
use single::{RenderObjs, RenderObjWrite, RenderObj};
use render::engine::{ Engine};
use system::util::*;
use system::util::constant::{COMMON};
use system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};


lazy_static! {
    static ref UCOLOR: Atom = Atom::from("UCOLOR");
    static ref VERTEX_COLOR: Atom = Atom::from("VERTEX_COLOR");

    static ref BLUR: Atom = Atom::from("blur");
    static ref U_COLOR: Atom = Atom::from("uColor");
}

pub struct BackgroundColorSys<C: Context + Share>{
    background_color_render_map: VecMap<Item>,
    geometry_dirtys: Vec<usize>,
    mark: PhantomData<C>,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
}

impl<C: Context + Share> BackgroundColorSys<C> {
    pub fn new() -> Self{
        BackgroundColorSys {
            background_color_render_map: VecMap::default(),
            geometry_dirtys: Vec::new(),
            mark: PhantomData,
            rs: Arc::new(RasterState::new()),
            bs: Arc::new(BlendState::new()),
            ss: Arc::new(StencilState::new()),
            ds: Arc::new(DepthState::new()),
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
        for id in  self.geometry_dirtys.iter() {
            let item = unsafe { map.get_unchecked_mut(*id) };
            item.position_change = false;
            let border_radius = unsafe { border_radius.get_unchecked(*id) };
            let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;
            let layout = unsafe { layouts.get_unchecked(*id) };
            let background_color = unsafe { background_colors.get_unchecked(*id) };
            let (positions, indices, colors) = get_geo_flow(border_radius, layout, z_depth - 0.2, background_color);

            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
            let geometry = unsafe {&mut *(render_obj.geometry.as_ref() as *const C::ContextGeometry as usize as *mut C::ContextGeometry)};

            let vertex_count: u32 = (positions.len()/3) as u32;
            if  vertex_count == 0 {
                geometry.set_vertex_count(vertex_count);
                continue;
            }
            if vertex_count != geometry.get_vertex_count() {
                geometry.set_vertex_count(vertex_count);
            }
            geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false).unwrap();
            geometry.set_indices_short(indices.as_slice(), false).unwrap();
            match colors {
                Some(colors) => {
                    geometry.set_attribute(&AttributeName::Color, 4, Some(colors.as_slice()), false).unwrap()
                },
                None => geometry.set_attribute(&AttributeName::Color, 4, None, false).unwrap(),
            };

            render_objs.get_notify().modify_event(item.index, "geometry", 0);
        }
        self.geometry_dirtys.clear();
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
        let _border_radius = unsafe { border_radius.get_unchecked(event.id) };
        let z_depth = unsafe { z_depths.get_unchecked(event.id) }.0;
        let _layout = unsafe { layouts.get_unchecked(event.id) };
        let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;

        let geometry = create_geometry(&mut engine.gl);
        let mut ubos: FnvHashMap<Atom, Arc<Uniforms<C>>> = FnvHashMap::default();
        let mut defines = Vec::new();

        let mut common_ubo = engine.gl.create_uniforms();
        common_ubo.set_float_1(&BLUR, 1.0);
        match &background_color.0 {
            Color::RGBA(c) => {
                debug_println!("bg_color, id: {}, color: {:?}", event.id, c);
                common_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b,c.a);
                defines.push(UCOLOR.clone());
            },
            Color::LinearGradient(_) => {
                defines.push(VERTEX_COLOR.clone());
            },
        }
        ubos.insert(COMMON.clone(), Arc::new(common_ubo)); // COMMON

        // println!("ds----------------{:?}", self.ds);
        let pipeline = engine.create_pipeline(
            0,
            &COLOR_VS_SHADER_NAME.clone(),
            &COLOR_FS_SHADER_NAME.clone(),
            defines.as_slice(),
            self.rs.clone(),
            self.bs.clone(),
            self.ss.clone(),
            self.ds.clone(),
        );
        
        let is_opacity = background_is_opacity(opacity, background_color);
        // debug_println!("xxxxxxxxxxxxxxxxxcreate color{}， is_opacity: {}", event.id, is_opacity);
        let render_obj: RenderObj<C> = RenderObj {
            depth: z_depth - 0.2,
            depth_diff: -0.2,
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
        self.geometry_dirtys.push(event.id);
    }
}

// 修改渲染对象
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
                // debug_println!("bg_color, id: {}, color: {:?}", event.id, c);
                common_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b, c.a);

                // 如果未找到UCOLOR宏， 表示修改之前的颜色不为RGBA， 应该删除VERTEX_COLOR宏， 添加UCOLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
                if find_item_from_vec(&render_obj.defines, &UCOLOR) == 0 {
                    render_obj.defines.remove_item(&VERTEX_COLOR);
                    render_obj.defines.push(UCOLOR.clone());
                    if item.position_change == false {
                        item.position_change = true;
                        self.geometry_dirtys.push(event.id);
                    }
                }
                render_objs.get_notify().modify_event(item.index, "", 0);
            },
            Color::LinearGradient(_) => {
                // 如果未找到VERTEX_COLOR宏， 表示修改之前的颜色不为LinearGradient， 应该删除UCOLOR宏， 添加VERTEX_COLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
                if find_item_from_vec(&render_obj.defines, &VERTEX_COLOR) == 0 {
                    render_obj.defines.remove_item(&UCOLOR);
                    render_obj.defines.push(VERTEX_COLOR.clone());      
                    if item.position_change == false {
                        item.position_change = true;
                        self.geometry_dirtys.push(event.id);
                    }
                }
            },
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
            self.geometry_dirtys.remove_item(&event.id);
        }
    }
}

//布局修改， 需要重新计算顶点
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Layout, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        if let Some(item) = self.background_color_render_map.get_mut(event.id) {
            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(event.id);
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
        if let Some(item) = self.background_color_render_map.get_mut(id) {
            let opacity = unsafe { opacitys.get_unchecked(id).0 };
            let background_color = unsafe { colors.get_unchecked(id) };

            let is_opacity = background_is_opacity(opacity, background_color);
            
            let notify = render_objs.get_notify();
            // debug_println!("set_is_opacity color{}， is_opacity: {}", id, is_opacity);
            unsafe { render_objs.get_unchecked_write(item.index, &notify).set_is_opacity(is_opacity)};
        }
    }
}

struct Item {
    index: usize,
    position_change: bool,
}

//取几何体的顶点流和索引流和color属性流
fn get_geo_flow(radius: &BorderRadius, layout: &Layout, z_depth: f32, color: &BackgroundColor) -> (Vec<f32>, Vec<u16>, Option<Vec<f32>>) {
    let radius = cal_border_radius(radius, layout);
    let start_x = layout.border_left;
    let start_y = layout.border_top;
    let end_x = layout.width - layout.border_right;
    let end_y = layout.height - layout.border_bottom;
    let mut positions;
    let mut indices;
    debug_println!("radius:{:?}", radius);
    if radius.x <= start_x {
        positions = vec![
            start_x, start_y, z_depth, // left_top
            start_x, end_y, z_depth, // left_bootom
            end_x, end_y, z_depth, // right_bootom
            end_x, start_y, z_depth, // right_top
        ];
        indices = vec![0, 1, 2, 3];
    } else {
        debug_println!("bg_color, split_by_radius----x:{}, y: {}, width: {}, height: {}, radius: {}, z_depth: {}",
            start_x, start_y, end_x - start_x, end_y - start_y, radius.x - start_x, z_depth);
        let r = split_by_radius(start_x, start_y, end_x - start_x, end_y - start_y, radius.x - start_x, z_depth, None);
        positions = r.0;
        indices = r.1;
    }
    match &color.0 {
        &Color::RGBA(_) => {
            (positions, to_triangle(indices.as_slice(), Vec::new()), None)
        },
        &Color::LinearGradient(ref bg_colors) => {
            let mut lg_pos = Vec::with_capacity(bg_colors.list.len());
            let mut color = Vec::with_capacity(bg_colors.list.len() * 4);
            for v in bg_colors.list.iter() {
                lg_pos.push(v.position);
                color.extend_from_slice(&[v.rgba.r, v.rgba.g, v.rgba.b, v.rgba.a]);
            }

            //渐变端点
            debug_println!("layout:{:?}, direction: {}", layout, bg_colors.direction);
            let endp = find_lg_endp(&[
                0.0, 0.0,
                0.0, layout.height,
                layout.width, layout.height,
                layout.width, 0.0,
            ], bg_colors.direction);
            debug_println!("split_by_lg------------------positions:{:?}, indices: {:?}, lg_pos:{:?}, start: ({}, {}), end: ({}, {})", positions, indices, lg_pos, (endp.0).0, (endp.0).1, (endp.1).0, (endp.1).1);
            let (positions, indices_arr) = split_by_lg(positions, indices, lg_pos.as_slice(), endp.0.clone(), endp.1.clone());
            debug_println!("indices_arr------------------{:?}", indices_arr);
            // println!("interp_mult_by_lg------------------positions:{:?}, indices_arr: {:?}, lg_pos:{:?}, cfg: {:?}, start: ({}, {}), end: ({}, {})", positions, indices_arr, lg_pos, vec![LgCfg{unit:4, data: &color}], (endp.0).0, (endp.0).1, (endp.1).0, (endp.1).1);
            let mut colors = interp_mult_by_lg(positions.as_slice(), &indices_arr, vec![Vec::new()], vec![LgCfg{unit:4, data: color}], lg_pos.as_slice(), endp.0, endp.1);
            let indices = mult_to_triangle(&indices_arr, Vec::new());
            let colors = colors.pop().unwrap();
            (positions, indices, Some(colors))
        },
    }
}

fn background_is_opacity(opacity: f32, background_color: &BackgroundColor) -> bool{
    if opacity < 1.0 {
        // println!("cccccccccccccccccccccccccc opacity:{}", opacity);
        return false;
    }
    match &background_color.0 {
        &Color::RGBA(ref c) => if c.a < 1.0 {
            // println!("cccccccccccccccccccccccccc:{}", c.a);
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
    // println!("cccccccccccccccccccccccccc11111");
    return true;
}

unsafe impl<C: Context + Share> Sync for BackgroundColorSys<C>{}
unsafe impl<C: Context + Share> Send for BackgroundColorSys<C>{}

impl_system!{
    BackgroundColorSys<C> where [C: Context + Share],
    true,
    {
        MultiCaseListener<Node, BackgroundColor, CreateEvent>
        MultiCaseListener<Node, BackgroundColor, ModifyEvent>
        MultiCaseListener<Node, BackgroundColor, DeleteEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
    }
}