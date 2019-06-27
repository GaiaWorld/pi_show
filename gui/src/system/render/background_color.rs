/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use share::Share;
use std::hash::{ Hasher, Hash };
use std::collections::hash_map::DefaultHasher;

use fnv::FnvHashMap;
use ordered_float::NotNan;

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseImpl, MultiCaseImpl, Share as ShareTrait, Runner};
use map::{ vecmap::VecMap } ;
use hal_core::{Context, Uniforms, RasterState, BlendState, StencilState, DepthState, Geometry, AttributeName};
use atom::Atom;
use polygon::*;

use component::user::*;
use component::calc::{Opacity, ZDepth, WorldMatrixRender};
use entity::{Node};
use single::*;
use render::engine::{ Engine};
use render::res::GeometryRes;
use system::util::*;
use system::util::constant::*;
use system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};

lazy_static! {
    static ref UCOLOR: Atom = Atom::from("UCOLOR");
    static ref VERTEX_COLOR: Atom = Atom::from("VERTEX_COLOR");

    static ref BLUR: Atom = Atom::from("blur");
    static ref U_COLOR: Atom = Atom::from("uColor");
    static ref GRADUAL: Atom = Atom::from("gradual_change");
}

pub struct BackgroundColorSys<C: Context + ShareTrait>{
    render_map: VecMap<Item>,
    geometry_dirtys: Vec<usize>,
    mark: PhantomData<C>,
    rs: Share<RasterState>,
    bs: Share<BlendState>,
    ss: Share<StencilState>,
    ds: Share<DepthState>,
}

impl<C: Context + ShareTrait> BackgroundColorSys<C> {
    pub fn new() -> Self{
        BackgroundColorSys {
            render_map: VecMap::default(),
            geometry_dirtys: Vec::new(),
            mark: PhantomData,
            rs: Share::new(RasterState::new()),
            bs: Share::new(BlendState::new()),
            ss: Share::new(StencilState::new()),
            ds: Share::new(DepthState::new()),
        }
    }

    fn set_geometry_dirty(&mut self, id: usize) {
        if let Some(item) = self.render_map.get_mut(id) {
            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(id);
            }
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流和颜色属性流
impl<'a, C: Context + ShareTrait> Runner<'a> for BackgroundColorSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, BackgroundColor>,
        &'a MultiCaseImpl<Node, WorldMatrixRender>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let (layouts, border_radiuss, z_depths, background_colors, world_matrixs) = read;
        let (render_objs, engine) = write;
        for id in  self.geometry_dirtys.iter() {
            let item = unsafe { self.render_map.get_unchecked_mut(*id) };
            item.position_change = false;
            let border_radius = unsafe { border_radiuss.get_unchecked(*id) };
            let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;
            let layout = unsafe { layouts.get_unchecked(*id) };
            let background_color = unsafe { background_colors.get_unchecked(*id) };
            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };

            let key = geometry_hash(border_radius, layout, background_color);
            match engine.res_mgr.get::<GeometryRes<C>>(&key) {
                Some(geometry) => {
                    render_obj.geometry = Some(geometry);
                },
                None => {
                    let (positions, indices, colors) = get_geo_flow(border_radius, layout, z_depth - 0.2, background_color);
                    if positions.len() == 0 {
                        render_obj.geometry = None;
                    } else {
                        let mut geometry = create_geometry(&mut engine.gl);
                        geometry.set_vertex_count((positions.len()/3) as u32);
                        geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false).unwrap();
                        geometry.set_indices_short(indices.as_slice(), false).unwrap();
                        match colors {
                            Some(colors) => {
                                geometry.set_attribute(&AttributeName::Color, 4, Some(colors.as_slice()), false).unwrap()
                            },
                            None => geometry.set_attribute(&AttributeName::Color, 4, None, false).unwrap(),
                        };

                        render_obj.geometry = Some(engine.res_mgr.create::<GeometryRes<C>>(GeometryRes{name: key, bind: geometry}));
                    }
                },
            };  
            render_objs.get_notify().modify_event(item.index, "geometry", 0);

            self.modify_matrix(*id, world_matrixs, layouts, background_colors, border_radiuss, render_objs);
        }
        self.geometry_dirtys.clear();
    }
}

// 插入渲染对象
impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, BackgroundColor, CreateEvent> for BackgroundColorSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, BackgroundColor>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, WorldMatrixRender>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs<C>>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (background_colors, border_radius, z_depths, layouts, opacitys, world_matrix) = read;
        let (render_objs, engine) = write;
        let background_color = unsafe { background_colors.get_unchecked(event.id) };
        let _border_radius = unsafe { border_radius.get_unchecked(event.id) };
        let z_depth = unsafe { z_depths.get_unchecked(event.id) }.0;
        let _layout = unsafe { layouts.get_unchecked(event.id) };
        let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;

        let mut ubos: FnvHashMap<Atom, Share<Uniforms<C>>> = FnvHashMap::default();
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
        ubos.insert(COMMON.clone(), Share::new(common_ubo)); // COMMON

        let mut world_matrix_ubo = engine.gl.create_uniforms();
        let slice: &[f32; 16] = unsafe { world_matrix.get_unchecked(event.id) }.0.as_ref();
        world_matrix_ubo.set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
        ubos.insert(WORLD.clone(), Share::new(world_matrix_ubo)); // WORLD_MATRIX

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
            geometry: None,
            pipeline: pipeline.clone(),
            context: event.id,
            defines: defines,
        };

        let notify = render_objs.get_notify();
        let index = render_objs.insert(render_obj, Some(notify));
        self.render_map.insert(event.id, Item{index: index, position_change: true});
        self.geometry_dirtys.push(event.id);
    }
}

// 修改渲染对象
impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, BackgroundColor, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, BackgroundColor>, &'a MultiCaseImpl<Node, Opacity>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (background_colors, opacitys) = read;
        let item = unsafe { self.render_map.get_unchecked_mut(event.id) };
        let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
   
        let background_color = unsafe { background_colors.get_unchecked(event.id) };
        match &background_color.0 {
            Color::RGBA(c) => {
                // 设置ubo
                let common_ubo = Share::make_mut(render_obj.ubos.get_mut(&COMMON).unwrap());
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
impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, BackgroundColor, DeleteEvent> for BackgroundColorSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, render_objs: Self::WriteData){
        let item = self.render_map.remove(event.id).unwrap();
        let notify = render_objs.get_notify();
        render_objs.remove(item.index, Some(notify));
        if item.position_change == true {
            self.geometry_dirtys.remove_item(&event.id);
        }
    }
}

//圆角修改， 需要重新计算世界矩阵和顶点流
impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, Layout, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        self.set_geometry_dirty(event.id);
    }
}

//圆角修改， 需要重新计算世界矩阵和顶点流
impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, BorderRadius, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        self.set_geometry_dirty(event.id);
    }
}

//不透明度变化， 设置ubo
impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, BackgroundColor>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        self.change_is_opacity(event.id, read.0, read.1, write);
    }
}


type MatrixRead<'a> = (
    &'a MultiCaseImpl<Node, WorldMatrixRender>,
    &'a MultiCaseImpl<Node, Layout>,
    &'a MultiCaseImpl<Node, BackgroundColor>,
    &'a MultiCaseImpl<Node, BorderRadius>,
);

impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, WorldMatrixRender, ModifyEvent> for BackgroundColorSys<C>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read.0, read.1, read.2, read.3, render_objs);
    }
}

impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, WorldMatrixRender, CreateEvent> for BackgroundColorSys<C>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read.0, read.1, read.2, read.3, render_objs);
    }
}

impl<'a, C: Context + ShareTrait> BackgroundColorSys<C> {
    fn change_is_opacity(&mut self, id: usize, opacitys: &MultiCaseImpl<Node, Opacity>, colors: &MultiCaseImpl<Node, BackgroundColor>, render_objs: &mut SingleCaseImpl<RenderObjs<C>>){
        if let Some(item) = self.render_map.get_mut(id) {
            let opacity = unsafe { opacitys.get_unchecked(id).0 };
            let background_color = unsafe { colors.get_unchecked(id) };

            let is_opacity = background_is_opacity(opacity, background_color);
            
            let notify = render_objs.get_notify();
            // unsafe { render_objs.get_unchecked_mut(item.index)}.is_opacity = is_opacity;
            // debug_println!("set_is_opacity color{}， is_opacity: {}", id, is_opacity);
            unsafe { render_objs.get_unchecked_write(item.index, &notify)}.set_is_opacity(is_opacity);
        }
    }

    fn modify_matrix(
        &self,
        id: usize,
        world_matrixs: &MultiCaseImpl<Node, WorldMatrixRender>,
        layouts: &MultiCaseImpl<Node, Layout>,
        background_colors: &MultiCaseImpl<Node, BackgroundColor>,
        border_radiuss: &MultiCaseImpl<Node, BorderRadius>,
        render_objs: &mut SingleCaseImpl<RenderObjs<C>>
    ){
        if let Some(item) = self.render_map.get(id) {
            let background_color = unsafe { background_colors.get_unchecked(id) };
            let layout = unsafe { layouts.get_unchecked(id) };
            let world_matrix = unsafe { world_matrixs.get_unchecked(id) };
            let border_radius = cal_border_radius(unsafe { border_radiuss.get_unchecked(id) }, layout);
            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
            match background_color.0 {
                Color::RGBA(_) => if border_radius.x == 0.0 {
                    // 渲染物件的顶点是一个四边形， 将其宽高乘在世界矩阵上
                    let world_matrix = world_matrix.0 * Matrix4::from_nonuniform_scale(
                        layout.width - layout.border_right - layout.border_left,
                        layout.height - layout.border_top - layout.border_bottom,
                        1.0
                    );
                    let ubos = &mut render_obj.ubos;
                    let slice: &[f32; 16] = world_matrix.as_ref();
                    Share::make_mut(ubos.get_mut(&WORLD).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
                    debug_println!("id: {}, world_matrix: {:?}", render_obj.context, &slice[0..16]);
                    render_objs.get_notify().modify_event(item.index, "ubos", 0);
                    return;
                }
                _ => ()
            }

            // 渲染物件的顶点不是一个四边形， 保持其原有的矩阵
            let ubos = &mut render_obj.ubos;
            let slice: &[f32; 16] = world_matrix.0.as_ref();
            Share::make_mut(ubos.get_mut(&WORLD).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
            debug_println!("background_color, id: {}, world_matrix: {:?}", render_obj.context, &slice[0..16]);
            render_objs.get_notify().modify_event(item.index, "ubos", 0);
            
        }
    }
}

struct Item {
    index: usize,
    position_change: bool,
}

fn geometry_hash(radius: &BorderRadius, layout: &Layout, color: &BackgroundColor) -> u64{
    let radius = cal_border_radius(radius, layout);
    let mut hasher = DefaultHasher::new();
    match &color.0 {
        Color::RGBA(_) => {
            if radius.x == 0.0 {
                QUAD_POSITION_INDEX.hash(&mut hasher);           
            } else {
                radius_quad_hash(&mut hasher, radius.x, layout.width - layout.border_right - layout.border_left, layout.height - layout.border_top - layout.border_bottom);
            }
        },
        Color::LinearGradient(color) => {
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
            radius_quad_hash(&mut hasher, radius.x, layout.width - layout.border_right - layout.border_left, layout.height - layout.border_top - layout.border_bottom);
        },
    }
    return hasher.finish();
    
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
    match &color.0 {
        &Color::RGBA(_) => {
            if radius.x <= start_x {
                let r = create_quad_geo();
                positions = r.0;
                indices = r.1;
            } else {
                let r = split_by_radius(start_x, start_y, end_x - start_x, end_y - start_y, radius.x - start_x, z_depth, None);
                positions = r.0;
                indices = r.1;
            }
            (positions, to_triangle(indices.as_slice(), Vec::new()), None)
            
        },
        &Color::LinearGradient(ref bg_colors) => {
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

unsafe impl<C: Context + ShareTrait> Sync for BackgroundColorSys<C>{}
unsafe impl<C: Context + ShareTrait> Send for BackgroundColorSys<C>{}

impl_system!{
    BackgroundColorSys<C> where [C: Context + ShareTrait],
    true,
    {
        MultiCaseListener<Node, BackgroundColor, CreateEvent>
        MultiCaseListener<Node, BackgroundColor, ModifyEvent>
        MultiCaseListener<Node, BackgroundColor, DeleteEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
        MultiCaseListener<Node, WorldMatrixRender, CreateEvent>
        MultiCaseListener<Node, WorldMatrixRender, ModifyEvent>
        MultiCaseListener<Node, BorderRadius, ModifyEvent>
    }
}