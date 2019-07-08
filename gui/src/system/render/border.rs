/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use share::Share;
use std::hash::{ Hasher, Hash };

use fxhash::{FxHasher32};
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseImpl, MultiCaseImpl, Share as ShareTrait, Runner};
use map::{ vecmap::VecMap } ;
use hal_core::{Context, Uniforms, RasterState, BlendState, StencilState, DepthState, Geometry, AttributeName};
use atom::Atom;
use polygon::*;
use ordered_float::NotNan;

use component::user::*;
use component::calc::{Opacity, ZDepth, WorldMatrixRender};
use entity::{Node};
use single::*;
use render::engine::{ Engine};
use render::res::GeometryRes;
use system::util::*;
use system::util::constant::*;
use system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};
use FxHashMap32;

lazy_static! {
    static ref UCOLOR: Atom = Atom::from("UCOLOR");

    static ref BLUR: Atom = Atom::from("blur");
    static ref U_COLOR: Atom = Atom::from("uColor");

    static ref BORDER_COLOR: Atom = Atom::from("border_color");
}

pub struct BorderColorSys<C: Context + ShareTrait>{
    render_map: VecMap<Item>,
    geometry_dirtys: Vec<usize>,
    mark: PhantomData<C>,
    rs: Share<RasterState>,
    bs: Share<BlendState>,
    ss: Share<StencilState>,
    ds: Share<DepthState>,
}

impl<C: Context + ShareTrait> BorderColorSys<C> {
    pub fn new() -> Self{
        BorderColorSys {
            render_map: VecMap::default(),
            geometry_dirtys: Vec::new(),
            mark: PhantomData,
            rs: Share::new(RasterState::new()),
            bs: Share::new(BlendState::new()),
            ss: Share::new(StencilState::new()),
            ds: Share::new(DepthState::new()),
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: Context + ShareTrait> Runner<'a> for BorderColorSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, WorldMatrixRender>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let (layouts, border_radius, z_depths, world_matrixs) = read;
        let (render_objs, engine) = write;
        for id in  self.geometry_dirtys.iter() {
            let map = &mut self.render_map;
            let item = unsafe { map.get_unchecked_mut(*id) };
            item.position_change = false;
            let border_radius = unsafe { border_radius.get_unchecked(*id) };
            let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;
            let layout = unsafe { layouts.get_unchecked(*id) };
            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };

            let key = geometry_hash(border_radius, layout);
            match engine.res_mgr.get::<GeometryRes<C>>(&key) {
                Some(geometry) => render_obj.geometry = Some(geometry),
                None => {
                    let (positions, indices) = get_geo_flow(border_radius, layout, z_depth - 0.2);
                    if positions.len() == 0 {
                        render_obj.geometry = None;
                    } else {
                        let mut geometry = create_geometry(&mut engine.gl);
                        geometry.set_vertex_count((positions.len()/3) as u32);
                        geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false).unwrap();
                        geometry.set_indices_short(indices.as_slice(), false).unwrap();
                        render_obj.geometry = Some(engine.res_mgr.create::<GeometryRes<C>>(GeometryRes{name: key, bind: geometry}));
                    };
                }
            };
            
            render_objs.get_notify().modify_event(item.index, "geometry", 0);
            self.modify_matrix(*id, world_matrixs, render_objs);
        }
        self.geometry_dirtys.clear();
    }
}

// 删除渲染对象
impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, BorderColor, ModifyEvent> for BorderColorSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, BorderColor>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        self.change_is_opacity(event.id, read.0, read.1, write);
    }
}

// 插入渲染对象
impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, BorderColor, CreateEvent> for BorderColorSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, BorderColor>,
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
        let (border_colors, border_radius, z_depths, layouts, opacitys) = read;
        let (render_objs, engine) = write;
        let border_color = unsafe { border_colors.get_unchecked(event.id) };
        let _border_radius = unsafe { border_radius.get_unchecked(event.id) };
        let z_depth = unsafe { z_depths.get_unchecked(event.id) }.0;
        let _layout = unsafe { layouts.get_unchecked(event.id) };
        let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;

        let mut ubos: FxHashMap32<Atom, Share<Uniforms<C>>> = FxHashMap32::default();
        let mut defines = Vec::new();
        defines.push(UCOLOR.clone());

        let mut common_ubo = engine.gl.create_uniforms();
        common_ubo.set_float_1(&BLUR, 1.0);
        common_ubo.set_float_4(&U_COLOR, border_color.0.r, border_color.0.g, border_color.0.b,border_color.0.a);
        ubos.insert(COMMON.clone(), Share::new(common_ubo)); // COMMON

        let pipeline = engine.create_pipeline(0, &COLOR_VS_SHADER_NAME.clone(), &COLOR_FS_SHADER_NAME.clone(), defines.as_slice(), self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone());
        
        let is_opacity = if opacity < 1.0 || border_color.0.a < 1.0 {
            false
        }else {
            true
        };
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

// 删除渲染对象
impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, BorderColor, DeleteEvent> for BorderColorSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, write: Self::WriteData){
        let item = self.render_map.remove(event.id).unwrap();
        let notify = write.get_notify();
        write.remove(item.index, Some(notify));
        if item.position_change == true {
            self.geometry_dirtys.remove_item(&event.id);
        }
    }
}

//布局修改， 需要重新计算顶点
impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, Layout, ModifyEvent> for BorderColorSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        if let Some(item) = self.render_map.get_mut(event.id) {
            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(event.id);
            }
        };
    }
}

//不透明度变化， 设置ubo
impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for BorderColorSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, BorderColor>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        self.change_is_opacity(event.id, read.0, read.1, write);
    }
}

type MatrixRead<'a> = &'a MultiCaseImpl<Node, WorldMatrixRender>;

impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, WorldMatrixRender, ModifyEvent> for BorderColorSys<C>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read, render_objs);
    }
}

impl<'a, C: Context + ShareTrait> MultiCaseListener<'a, Node, WorldMatrixRender, CreateEvent> for BorderColorSys<C>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read, render_objs);
    }
}

impl<'a, C: Context + ShareTrait> BorderColorSys<C> {
    fn change_is_opacity(&mut self, id: usize, opacitys: &MultiCaseImpl<Node, Opacity>, colors: &MultiCaseImpl<Node, BorderColor>, render_objs: &mut SingleCaseImpl<RenderObjs<C>>){
        if let Some(item) = self.render_map.get_mut(id) {
            let opacity = unsafe { opacitys.get_unchecked(id).0 };
            let border_color = unsafe { colors.get_unchecked(id) };

            let is_opacity = if opacity < 1.0 || border_color.0.a < 1.0 {
                false
            }else {
                true
            };

            let notify = render_objs.get_notify();
            unsafe { render_objs.get_unchecked_write(item.index, &notify).set_is_opacity(is_opacity)};
        }
    }

    fn modify_matrix(
        &self,
        id: usize,
        world_matrixs: &MultiCaseImpl<Node, WorldMatrixRender>,
        render_objs: &mut SingleCaseImpl<RenderObjs<C>>
    ){
        if let Some(item) = self.render_map.get(id) {
            let world_matrix = unsafe { world_matrixs.get_unchecked(id) };
            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };

            // 渲染物件的顶点不是一个四边形， 保持其原有的矩阵
            let ubos = &mut render_obj.ubos;
            let slice: &[f32; 16] = world_matrix.0.as_ref();
            Share::make_mut(ubos.get_mut(&WORLD).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
            debug_println!("border, id: {}, world_matrix: {:?}", render_obj.context, &slice[0..16]);
            render_objs.get_notify().modify_event(item.index, "ubos", 0);
        }
    }
}

struct Item {
    index: usize,
    position_change: bool,
}


fn geometry_hash(radius: &BorderRadius, layout: &Layout) -> u64{
    let radius = cal_border_radius(radius, layout);
    let mut hasher = FxHasher32::default();
    BORDER_COLOR.hash(&mut hasher);
    unsafe { NotNan::unchecked_new(radius.x).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(layout.width).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(layout.height).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(layout.border_left).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(layout.border_top).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(layout.border_right).hash(&mut hasher) };
    unsafe { NotNan::unchecked_new(layout.border_bottom).hash(&mut hasher) };
    return hasher.finish();
    
}

//取几何体的顶点流和属性流
fn get_geo_flow(radius: &BorderRadius, layout: &Layout, z_depth: f32) -> (Vec<f32>, Vec<u16>) {
    let radius = cal_border_radius(radius, layout);
    if radius.x == 0.0 {
        let border_start_x = layout.border_left;
        let border_start_y = layout.border_top;
        let border_end_x = layout.width - layout.border_right;
        let border_end_y = layout.height - layout.border_bottom;
        return (
            vec![
                0.0, 0.0, z_depth,
                0.0, layout.height, z_depth,
                layout.width, layout.height, z_depth,
                layout.width, 0.0, z_depth,

                border_start_x, border_start_y, z_depth,
                border_start_x, border_end_y, z_depth,
                border_end_x, border_end_y, z_depth,
                border_end_x, border_start_y, z_depth,
            ],
            vec![
                0, 1, 4,
                0, 4, 3,
                3, 4, 7,
                3, 7, 2,
                2, 7, 6,
                2, 6, 1,
                1, 6, 5,
                1, 5, 4,
            ],
        )
    }else {
        return split_by_radius_border(0.0, 0.0, layout.width, layout.height, radius.x, layout.border_left, z_depth, None);
    }
}

unsafe impl<C: Context + ShareTrait> Sync for BorderColorSys<C>{}
unsafe impl<C: Context + ShareTrait> Send for BorderColorSys<C>{}

impl_system!{
    BorderColorSys<C> where [C: Context + ShareTrait],
    true,
    {
        MultiCaseListener<Node, BorderColor, CreateEvent>
        MultiCaseListener<Node, BorderColor, ModifyEvent>
        MultiCaseListener<Node, BorderColor, DeleteEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
        MultiCaseListener<Node, WorldMatrixRender, CreateEvent>
        MultiCaseListener<Node, WorldMatrixRender, ModifyEvent>
    }
}