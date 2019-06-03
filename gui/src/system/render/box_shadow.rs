/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use std::sync::Arc;

use std::collections::HashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use map::{ vecmap::VecMap } ;
use hal_core::{Context, Uniforms, RasterState, BlendState, StencilState, DepthState, Geometry, AttributeName};
use atom::Atom;
use polygon::*;

use component::user::*;
use component::calc::{Opacity, ZDepth};
use entity::{Node};
use single::{RenderObjs, RenderObj};
use render::engine::{ Engine};
use system::util::*;
use system::util::constant::{COMMON};
use system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};


lazy_static! {
    static ref UCOLOR: Atom = Atom::from("UCOLOR");

    static ref BLUR: Atom = Atom::from("blur");
    static ref U_COLOR: Atom = Atom::from("uColor");
}

pub struct BoxShadowSys<C: Context + Share>{
    box_shadow_render_map: VecMap<Item>,
    geometry_dirtys: Vec<usize>,
    mark: PhantomData<C>,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
}

impl<C: Context + Share> BoxShadowSys<C> {
    pub fn new() -> Self{
        BoxShadowSys {
            box_shadow_render_map: VecMap::default(),
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
impl<'a, C: Context + Share> Runner<'a> for BoxShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, BoxShadow>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn run(&mut self, read: Self::ReadData, render_objs: Self::WriteData){
        println!("BoxShadowSys run, dirty_len:{}", self.geometry_dirtys.len());
        let map = &mut self.box_shadow_render_map;
        let (layouts, border_radius, z_depths, box_shadows) = read;
        for id in  self.geometry_dirtys.iter() {
            let item = unsafe { map.get_unchecked_mut(*id) };
            item.position_change = false;
            let border_radius = unsafe { border_radius.get_unchecked(*id) };
            let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;
            let layout = unsafe { layouts.get_unchecked(*id) };
            let box_shadow = unsafe { box_shadows.get_unchecked(*id) };
            let (positions, indices, colors) = get_geo_flow(border_radius, layout, z_depth - 0.2, box_shadow);

            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
            let geometry = unsafe {&mut *(render_obj.geometry.as_ref() as *const C::ContextGeometry as usize as *mut C::ContextGeometry)};

            let vertex_count: u32 = (positions.len()/3) as u32;
            if  vertex_count == 0 {
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
        }
        self.geometry_dirtys.clear();
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BoxShadow, CreateEvent> for BoxShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, BoxShadow>,
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
        let (box_shadows, border_radius, z_depths, layouts, opacitys) = read;
        let (render_objs, engine) = write;
        let box_shadow = unsafe { box_shadows.get_unchecked(event.id) };
        let _border_radius = unsafe { border_radius.get_unchecked(event.id) };
        let z_depth = unsafe { z_depths.get_unchecked(event.id) }.0;
        let _layout = unsafe { layouts.get_unchecked(event.id) };
        let _opacity = unsafe { opacitys.get_unchecked(event.id) }.0;

        let geometry = create_geometry(&mut engine.gl);
        let mut ubos: HashMap<Atom, Arc<Uniforms<C>>> = HashMap::default();
        let mut defines = Vec::new();
        defines.push(UCOLOR.clone());

        let mut common_ubo = engine.gl.create_uniforms();
        common_ubo.set_float_1(&BLUR, box_shadow.blur + 1.0);
        common_ubo.set_float_4(&U_COLOR, box_shadow.color.r, box_shadow.color.g, box_shadow.color.b, box_shadow.color.a);
        ubos.insert(COMMON.clone(), Arc::new(common_ubo)); // COMMON

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
        
        let render_obj: RenderObj<C> = RenderObj {
            depth: z_depth - 0.2,
            visibility: false,
            is_opacity: false,
            ubos: ubos,
            geometry: geometry,
            pipeline: pipeline.clone(),
            context: event.id,
            defines: defines,
        };

        let notify = render_objs.get_notify();
        let index = render_objs.insert(render_obj, Some(notify));
        self.box_shadow_render_map.insert(event.id, Item{index: index, position_change: true});
        self.geometry_dirtys.push(event.id);
    }
}

// 修改渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BoxShadow, ModifyEvent> for BoxShadowSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, BoxShadow>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, box_shadows: Self::ReadData, render_objs: Self::WriteData){
        let item = unsafe { self.box_shadow_render_map.get_unchecked_mut(event.id) };
        let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
   
        let box_shadow = unsafe { box_shadows.get_unchecked(event.id) };
        match event.field {
            "color" => {
                let common_ubo = Arc::make_mut(render_obj.ubos.get_mut(&COMMON).unwrap());
                debug_println!("box_shadow, id: {}, color: {:?}", event.id, box_shadow.color);
                common_ubo.set_float_4(&U_COLOR, box_shadow.color.r, box_shadow.color.g, box_shadow.color.b, box_shadow.color.a);
                return;
            },
            "blur" => {
                let common_ubo = Arc::make_mut(render_obj.ubos.get_mut(&COMMON).unwrap());
                debug_println!("box_shadow, id: {}, blur: {:?}", event.id, box_shadow.blur + 1.0);
                common_ubo.set_float_1(&BLUR, box_shadow.blur + 1.0);
                return;
            },
            "h" | "v" => {
                let item  = unsafe { self.box_shadow_render_map.get_unchecked_mut(event.id) };
                if item.position_change == false {
                    item.position_change = true;
                    self.geometry_dirtys.push(event.id);
                }
            },
            "" => {
                let common_ubo = Arc::make_mut(render_obj.ubos.get_mut(&COMMON).unwrap());
                debug_println!("box_shadow, id: {}, color: {:?}", event.id, box_shadow.color);
                common_ubo.set_float_4(&U_COLOR, box_shadow.color.r, box_shadow.color.g, box_shadow.color.b, box_shadow.color.a);
                debug_println!("box_shadow, id: {}, blur: {:?}", event.id, box_shadow.blur + 1.0);
                common_ubo.set_float_1(&BLUR, box_shadow.blur + 1.0);
                let item  = unsafe { self.box_shadow_render_map.get_unchecked_mut(event.id) };
                if item.position_change == false {
                    item.position_change = true;
                    self.geometry_dirtys.push(event.id);
                }
            },
            _ => (),
        };
    }
}

// 删除渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, BoxShadow, DeleteEvent> for BoxShadowSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, render_objs: Self::WriteData){
        let item = self.box_shadow_render_map.remove(event.id).unwrap();
        let notify = render_objs.get_notify();
        render_objs.remove(item.index, Some(notify));
        if item.position_change == true {
            self.geometry_dirtys.remove_item(&event.id);
        }
    }
}

//布局修改， 需要重新计算顶点
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Layout, ModifyEvent> for BoxShadowSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        if let Some(item) = self.box_shadow_render_map.get_mut(event.id) {
            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(event.id);
            }
        };
    }
}

struct Item {
    index: usize,
    position_change: bool,
}

//取几何体的顶点流和索引流和color属性流
fn get_geo_flow(radius: &BorderRadius, layout: &Layout, z_depth: f32, box_shadow: &BoxShadow) -> (Vec<f32>, Vec<u16>, Option<Vec<f32>>) {
    let radius = cal_border_radius(radius, layout);
    let start_x = box_shadow.h;
    let start_y = box_shadow.v;
    let end_x = layout.width + box_shadow.h;
    let end_y = layout.height + box_shadow.v;
    let mut positions;
    let mut indices;
    if radius.x == 0.0 {
        positions = vec![
            start_x, start_y, z_depth, // left_top
            start_x, end_y, z_depth, // left_bootom
            end_x, end_y, z_depth, // right_bootom
            end_x, start_y, z_depth, // right_top
        ];
        indices = vec![0, 1, 2, 3];
    } else {
        let r = split_by_radius(start_x, start_y, end_x - box_shadow.h, end_y - box_shadow.v, radius.x, z_depth, None);
        positions = r.0;
        indices = r.1;
    }
    (positions, to_triangle(indices.as_slice(), Vec::new()), None)
}

unsafe impl<C: Context + Share> Sync for BoxShadowSys<C>{}
unsafe impl<C: Context + Share> Send for BoxShadowSys<C>{}

impl_system!{
    BoxShadowSys<C> where [C: Context + Share],
    true,
    {
        MultiCaseListener<Node, BoxShadow, CreateEvent>
        MultiCaseListener<Node, BoxShadow, ModifyEvent>
        MultiCaseListener<Node, BoxShadow, DeleteEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
    }
}