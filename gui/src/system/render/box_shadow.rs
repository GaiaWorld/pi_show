/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use share::Share;
use std::hash::{ Hasher, Hash };

use fnv::{FnvHashMap, FnvHasher};
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
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

    static ref BLUR: Atom = Atom::from("blur");
    static ref U_COLOR: Atom = Atom::from("uColor");
}

pub struct BoxShadowSys<C: Context>{
    render_map: VecMap<Item>,
    geometry_dirtys: Vec<usize>,
    mark: PhantomData<C>,
    rs: Share<RasterState>,
    bs: Share<BlendState>,
    ss: Share<StencilState>,
    ds: Share<DepthState>,
}

impl<C: Context> BoxShadowSys<C> {
    pub fn new() -> Self{
        BoxShadowSys {
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

// 将顶点数据改变的渲染对象重新设置索引流和顶点流和颜色属性流
impl<'a, C: Context> Runner<'a> for BoxShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, BoxShadow>,
        &'a MultiCaseImpl<Node, WorldMatrixRender>
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let (layouts, border_radiuss, z_depths, box_shadows, world_matrixs) = read;
        let (render_objs, engine) = write;
        for id in  self.geometry_dirtys.iter() {
            let map = &mut self.render_map;
            let item = unsafe { map.get_unchecked_mut(*id) };
            item.position_change = false;
            let border_radius = unsafe { border_radiuss.get_unchecked(*id) };
            let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;
            let layout = unsafe { layouts.get_unchecked(*id) };
            let box_shadow = unsafe { box_shadows.get_unchecked(*id) };
   
            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };

            let key = geometry_hash(border_radius, layout);
            match engine.res_mgr.get::<GeometryRes<C>>(&key) {
                Some(geometry) => {
                    render_obj.geometry = Some(geometry);
                },
                None => {
                    let (positions, indices) = get_geo_flow(border_radius, layout, z_depth - 0.3, box_shadow);
                    if positions.len() == 0 {
                        render_obj.geometry = None;
                    } else {
                        let mut geometry = create_geometry(&mut engine.gl);
                        geometry.set_vertex_count((positions.len()/3) as u32);
                        geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false).unwrap();
                        geometry.set_indices_short(indices.as_slice(), false).unwrap();
                        render_obj.geometry = Some(engine.res_mgr.create::<GeometryRes<C>>(GeometryRes{name: key, bind: geometry}));
                    }
                },
            };
            render_objs.get_notify().modify_event(item.index, "geometry", 0);

            self.modify_matrix(*id, world_matrixs, box_shadows, layouts, border_radiuss, render_objs)
        }
        self.geometry_dirtys.clear();
    }
}

// 插入渲染对象
impl<'a, C: Context> MultiCaseListener<'a, Node, BoxShadow, CreateEvent> for BoxShadowSys<C>{
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

        let mut ubos: FnvHashMap<Atom, Share<Uniforms<C>>> = FnvHashMap::default();
        let mut defines = Vec::new();
        defines.push(UCOLOR.clone());

        let mut common_ubo = engine.gl.create_uniforms();
        common_ubo.set_float_1(&BLUR, box_shadow.blur + 1.0);
        common_ubo.set_float_4(&U_COLOR, box_shadow.color.r, box_shadow.color.g, box_shadow.color.b, box_shadow.color.a);
        ubos.insert(COMMON.clone(), Share::new(common_ubo)); // COMMON

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
            depth: z_depth - 0.3,
            depth_diff: -0.3,
            visibility: false,
            is_opacity: false,
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
impl<'a, C: Context> MultiCaseListener<'a, Node, BoxShadow, ModifyEvent> for BoxShadowSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, BoxShadow>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, box_shadows: Self::ReadData, render_objs: Self::WriteData){
        let item = unsafe { self.render_map.get_unchecked_mut(event.id) };
        let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
   
        let box_shadow = unsafe { box_shadows.get_unchecked(event.id) };
        match event.field {
            "color" => {
                let common_ubo = Share::make_mut(render_obj.ubos.get_mut(&COMMON).unwrap());
                debug_println!("box_shadow, id: {}, color: {:?}", event.id, box_shadow.color);
                common_ubo.set_float_4(&U_COLOR, box_shadow.color.r, box_shadow.color.g, box_shadow.color.b, box_shadow.color.a);
                return;
            },
            "blur" => {
                let common_ubo = Share::make_mut(render_obj.ubos.get_mut(&COMMON).unwrap());
                debug_println!("box_shadow, id: {}, blur: {:?}", event.id, box_shadow.blur + 1.0);
                common_ubo.set_float_1(&BLUR, box_shadow.blur + 1.0);
                return;
            },
            "h" | "v" => {
                let item  = unsafe { self.render_map.get_unchecked_mut(event.id) };
                if item.position_change == false {
                    item.position_change = true;
                    self.geometry_dirtys.push(event.id);
                }
            },
            "" => {
                let common_ubo = Share::make_mut(render_obj.ubos.get_mut(&COMMON).unwrap());
                debug_println!("box_shadow, id: {}, color: {:?}", event.id, box_shadow.color);
                common_ubo.set_float_4(&U_COLOR, box_shadow.color.r, box_shadow.color.g, box_shadow.color.b, box_shadow.color.a);
                debug_println!("box_shadow, id: {}, blur: {:?}", event.id, box_shadow.blur + 1.0);
                common_ubo.set_float_1(&BLUR, box_shadow.blur + 1.0);
                let item  = unsafe { self.render_map.get_unchecked_mut(event.id) };
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
impl<'a, C: Context> MultiCaseListener<'a, Node, BoxShadow, DeleteEvent> for BoxShadowSys<C>{
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

//布局修改， 需要重新计算顶点
impl<'a, C: Context> MultiCaseListener<'a, Node, Layout, ModifyEvent> for BoxShadowSys<C>{
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

type MatrixRead<'a> = (
    &'a MultiCaseImpl<Node, WorldMatrixRender>,
    &'a MultiCaseImpl<Node, BoxShadow>,
    &'a MultiCaseImpl<Node, Layout>,
    &'a MultiCaseImpl<Node, BorderRadius>,
);

impl<'a, C: Context> MultiCaseListener<'a, Node, WorldMatrixRender, ModifyEvent> for BoxShadowSys<C>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read.0, read.1, read.2, read.3, render_objs);
    }
}

impl<'a, C: Context> MultiCaseListener<'a, Node, WorldMatrixRender, CreateEvent> for BoxShadowSys<C>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read.0, read.1, read.2, read.3, render_objs);
    }
}

impl<'a, C: Context> BoxShadowSys<C>{
    fn modify_matrix(
        &self,
        id: usize,
        world_matrixs: &MultiCaseImpl<Node, WorldMatrixRender>,
        box_shadows: &MultiCaseImpl<Node, BoxShadow>,
        layouts: &MultiCaseImpl<Node, Layout>,
        border_radiuss: &MultiCaseImpl<Node, BorderRadius>,
        render_objs: &mut SingleCaseImpl<RenderObjs<C>>
    ){
        if let Some(item) = self.render_map.get(id) {
            let layout = unsafe { layouts.get_unchecked(id) };
            let box_shadow = unsafe { box_shadows.get_unchecked(id) };
            let world_matrix = unsafe { world_matrixs.get_unchecked(id) };
            let border_radius = cal_border_radius(unsafe { border_radiuss.get_unchecked(id) }, layout);
            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };

            let mut world_matrix = world_matrix.0 * Matrix4::from_translation(Vector3::new(box_shadow.h, box_shadow.v, 1.0));
            if border_radius.x == 0.0 {
                // 渲染物件的顶点是一个四边形， 将其宽高乘在世界矩阵上
                world_matrix = world_matrix * Matrix4::from_nonuniform_scale(
                    layout.width - layout.border_right - layout.border_left,
                    layout.height - layout.border_top - layout.border_bottom,
                    1.0
                );
            }
            let ubos = &mut render_obj.ubos;
            let slice: &[f32; 16] = world_matrix.as_ref();
            Share::make_mut(ubos.get_mut(&WORLD).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
            debug_println!("box_shadow, id: {}, world_matrix_shadow: {:?}", render_obj.context, &slice[0..16]);
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
    let mut hasher = FnvHasher::default();
    if radius.x == 0.0 {
        QUAD_POSITION_INDEX.hash(&mut hasher);           
    } else {
        radius_quad_hash(&mut hasher, radius.x, layout.width, layout.height);
    }
    return hasher.finish();  
}

//取几何体的顶点流和索引流和color属性流
fn get_geo_flow(radius: &BorderRadius, layout: &Layout, z_depth: f32, box_shadow: &BoxShadow) -> (Vec<f32>, Vec<u16>) {
    if layout.width == 0.0 && layout.height == 0.0 {
        return (Vec::new(), Vec::new());
    }
    let radius = cal_border_radius(radius, layout);
    let start_x = box_shadow.h;
    let start_y = box_shadow.v;
    let end_x = layout.width + box_shadow.h;
    let end_y = layout.height + box_shadow.v;
    let mut positions;
    let mut indices;
    if radius.x == 0.0 {
        let r = create_quad_geo();
        positions = r.0;
        indices = r.1;
        // (positions, to_triangle(indices.as_slice(), Vec::new()), None)
        // positions = vec![
        //     start_x, start_y, z_depth, // left_top
        //     start_x, end_y, z_depth, // left_bootom
        //     end_x, end_y, z_depth, // right_bootom
        //     end_x, start_y, z_depth, // right_top
        // ];
        // indices = vec![0, 1, 2, 3];
    } else {
        let r = split_by_radius(start_x, start_y, end_x - box_shadow.h, end_y - box_shadow.v, radius.x, z_depth, None);
        positions = r.0;
        indices = r.1;
    }
    (positions, to_triangle(indices.as_slice(), Vec::new()))
}

unsafe impl<C: Context> Sync for BoxShadowSys<C>{}
unsafe impl<C: Context> Send for BoxShadowSys<C>{}

impl_system!{
    BoxShadowSys<C> where [C: Context],
    true,
    {
        MultiCaseListener<Node, BoxShadow, CreateEvent>
        MultiCaseListener<Node, BoxShadow, ModifyEvent>
        MultiCaseListener<Node, BoxShadow, DeleteEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        MultiCaseListener<Node, WorldMatrixRender, CreateEvent>
        MultiCaseListener<Node, WorldMatrixRender, ModifyEvent>
    }
}