/**
 *  
 */
use std::marker::PhantomData;
use share::Share;

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
use hal_core::*;
use atom::Atom;

use system::util::*;
use component::calc::*;
use entity::{Node};
use single::*;
use render::engine::Engine;

lazy_static! {
    static ref Z_DEPTH: Atom = Atom::from("zDepth");
    static ref HSV_MACRO: Atom = Atom::from("HSV");
    static ref HSV_ATTR: Atom = Atom::from("hsvValue");
}

pub struct NodeAttrSys<C: HalContext + 'static>{
    view_matrix_ubo: Option<Share<dyn UniformBuffer>>,
    project_matrix_ubo: Option<Share<dyn UniformBuffer>>,
    marker: PhantomData<C>,
}

impl<C: HalContext + 'static> NodeAttrSys<C> {
    pub fn new() -> Self{
        NodeAttrSys {
            view_matrix_ubo: None,
            project_matrix_ubo: None,
            marker: PhantomData,
        }
    }
}

impl<'a, C: HalContext + 'static> Runner<'a> for NodeAttrSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<ViewMatrix>,
        &'a SingleCaseImpl<ProjectionMatrix>,
    );
    type WriteData =  &'a mut SingleCaseImpl<Engine<C>>;
    fn run(&mut self, _read: Self::ReadData, _write: Self::WriteData){
    }
    fn setup(&mut self, read: Self::ReadData, _: Self::WriteData){
        let (view_matrix, projection_matrix) = read;

        let slice: &[f32; 16] = view_matrix.0.as_ref();
        let view_matrix_ubo = ViewMatrixUbo::new(UniformValue::MatrixV(4, Vec::from(&slice[..])));
        debug_println!("view_matrix: {:?}", &slice[..]);

        let slice: &[f32; 16] = projection_matrix.0.as_ref();
        let project_matrix_ubo = ViewMatrixUbo::new(UniformValue::MatrixV(4, Vec::from(&slice[..])));
        debug_println!("projection_matrix: {:?}", &slice[..]);

        self.view_matrix_ubo = Some(Share::new(view_matrix_ubo));
        self.project_matrix_ubo = Some(Share::new(project_matrix_ubo));
    }
}

impl<'a, C: HalContext + 'static> EntityListener<'a, Node, CreateEvent> for NodeAttrSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<NodeRenderMap>;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, node_render_map: Self::WriteData){
        node_render_map.create(event.id);
    }
}

impl<'a, C: HalContext + 'static> EntityListener<'a, Node, DeleteEvent> for NodeAttrSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<NodeRenderMap>;
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, node_render_map: Self::WriteData){
        unsafe { node_render_map.destroy_unchecked(event.id) };
    }
}

//创建索引
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, CreateEvent> for NodeAttrSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, HSV>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<Engine<C>>, &'a mut SingleCaseImpl<NodeRenderMap>);
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (opacitys, visibilitys, hsvs) = read;
        let (render_objs, engine, node_render_map) = write;
        let render_obj = unsafe { render_objs.get_unchecked_mut(event.id) };
        let notify = node_render_map.get_notify();
        unsafe{ node_render_map.add_unchecked(render_obj.context, event.id, &notify) };
        
        let paramter = &mut render_obj.paramter;

        paramter.set_value("view_matrix", self.view_matrix_ubo.clone().unwrap()); // VIEW_MATRIX
        paramter.set_value("project_matrix", self.project_matrix_ubo.clone().unwrap()); // PROJECT_MATRIX

        let opacity = unsafe { opacitys.get_unchecked(render_obj.context) }.0;
        paramter.set_value("alpha", create_alpha_ubo(engine, opacity)); // alpha
        debug_println!("id: {}, alpha: {:?}", render_obj.context, opacity);

        let visibility = unsafe { visibilitys.get_unchecked(render_obj.context) }.0;
        render_obj.visibility = visibility;

        let hsv = unsafe { hsvs.get_unchecked(render_obj.context) };
        if !(hsv.h == 0.0 && hsv.s == 0.0 && hsv.v == 0.0) {
            render_obj.fs_defines.add("HSV");
            paramter.set_value("hsv", create_hsv_ubo(engine, hsv)); // hsv
        }
        
        // 如果透明， 修改bs和ds(bug, 某些透明物体的混合模式非BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha， 如canvas_text)
        if !render_obj.is_opacity {
            let mut bs = engine.gl.bs_get_desc(render_obj.state.bs.as_ref()).clone();
            let mut ds = engine.gl.ds_get_desc(render_obj.state.ds.as_ref()).clone();
            bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
            ds.set_write_enable(false);
            
            render_obj.state.bs = create_bs_res(engine, bs);
            render_obj.state.ds = create_ds_res(engine, ds);
        }
    }
}

// 监听is_opacity的修改，修改渲染状态， 创建新的渲染管线
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = ();
    type WriteData = ( &'a mut SingleCaseImpl<RenderObjs>,  &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData){
        match event.field {
            "is_opacity" => {
                let (render_objs, engine) = write;
                let render_obj = unsafe { render_objs.get_unchecked_mut(event.id) };
                modify_opacity(engine, render_obj);
                render_objs.get_notify().modify_event(event.id, "state", 0);
            },
            _ => (),
        }
    }
}

// 删除索引
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, DeleteEvent> for NodeAttrSys<C>{
    type ReadData = &'a SingleCaseImpl<RenderObjs>;
    type WriteData = &'a mut SingleCaseImpl<NodeRenderMap>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, node_render_map: Self::WriteData){
        let render_obj = unsafe { read.get_unchecked(event.id) };
        let notify = node_render_map.get_notify();
        unsafe{ node_render_map.remove_unchecked(render_obj.context, event.id, &notify) };
    }
}

// //深度变化， 世界矩阵ubo的z值
// impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ZDepth, ModifyEvent> for NodeAttrSys<C>{
//     type ReadData = &'a MultiCaseImpl<Node, ZDepth>;
//     type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<NodeRenderMap>);
//     fn listen(&mut self, event: &ModifyEvent, z_depths: Self::ReadData, write: Self::WriteData){
//         let (render_objs, node_render_map) = write;
//         let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };
//         let z_depth = unsafe{ z_depths.get_unchecked(event.id) }.0;

//         for id in obj_ids.iter() {
//             let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
//             render_obj.depth = z_depth + render_obj.depth_diff;
//             let paramter = &mut render_obj.paramter;
//             match paramter.get_value("worldMatrix") {
//                 Some(ubo) => {
//                     match ubo.get_value("worldMatrix").unwrap() {
//                         UniformValue::MatrixV()
//                     }
//                 },
//                 None => (),
//             };
//             let ubo = paramter.get_mut(&Z_DEPTH).unwrap();
//             Share::make_mut(ubo).set_float_1(&Z_DEPTH, -render_obj.depth/Z_MAX);
//             debug_println!("id: {}, z_depth: {:?}", render_obj.context, -render_obj.depth/Z_MAX);
//             render_objs.get_notify().modify_event(*id, "depth", 0);
//         }
//     }
// }

//不透明度变化， 设置ubo
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, Opacity>;
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<NodeRenderMap>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, opacitys: Self::ReadData, write: Self::WriteData){
        let opacity = unsafe { opacitys.get_unchecked(event.id).0 };
        let (render_objs, node_render_map, engine) = write;
        let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };

        for id in obj_ids.iter() {
            let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
            render_obj.paramter.as_ref().set_value("alpha", create_alpha_ubo(engine, opacity));
            render_objs.get_notify().modify_event(*id, "paramter", 0);
        }
    }
}

// 设置visibility
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Visibility, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, Visibility>;
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<NodeRenderMap>);
    fn listen(&mut self, event: &ModifyEvent, visibilitys: Self::ReadData, write: Self::WriteData){
        let (render_objs, node_render_map) = write;
        let visibility = unsafe { visibilitys.get_unchecked(event.id).0 };
        let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };

        for id in obj_ids.iter() {
            let notify = render_objs.get_notify();
            let mut render_obj = unsafe {render_objs.get_unchecked_write(*id, &notify)};
            render_obj.set_visibility(visibility);
            debug_println!("id: {}, visibility: {:?}", render_obj.value.context, visibility);
        }
    }
}

// 设置hsv
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, HSV, ModifyEvent> for NodeAttrSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, HSV>;
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<NodeRenderMap>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, hsvs: Self::ReadData, write: Self::WriteData){
        let (render_objs, node_render_map, engine) = write;
        let hsv = unsafe { hsvs.get_unchecked(event.id) };
        let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };

        if !(hsv.h == 0.0 && hsv.s == 0.0 && hsv.v == 0.0) {
            for id in obj_ids.iter() {
                let notify = render_objs.get_notify();
                let render_obj = unsafe {render_objs.get_unchecked_mut(*id)};
                render_obj.fs_defines.add("HSV");
                render_obj.paramter.set_value("hsv", create_hsv_ubo(engine, hsv)); // hsv            
                notify.modify_event(*id, "paramter", 0);
                notify.modify_event(*id, "fs_defines", 0);
            }
        } else {
            for id in obj_ids.iter() {
                let notify = render_objs.get_notify();
                let render_obj = unsafe {render_objs.get_unchecked_mut(*id)};
                render_obj.fs_defines.remove("HSV");          
                notify.modify_event(*id, "paramter", 0);
                notify.modify_event(*id, "fs_defines", 0);
            }
        }
        
    }
}

pub fn create_alpha_ubo<C: HalContext + 'static>( engine: &mut Engine<C>, opacity: f32) -> Share<dyn UniformBuffer> {
    let h = f32_1_hash(opacity);
    match engine.res_mgr.get::<AlphaUbo>(&h) {
        Some(r) => r,
        None => engine.res_mgr.create(h, AlphaUbo::new(UniformValue::Float(1, opacity, 0.0, 0.0, 0.0))),
    }
}

pub fn create_hsv_ubo<C: HalContext + 'static>( engine: &mut Engine<C>, hsv: &HSV) -> Share<dyn UniformBuffer> {
    let h = f32_3_hash(hsv.h, hsv.s, hsv.v);
    match engine.res_mgr.get::<HsvUbo>(&h) {
        Some(r) => r,
        None => engine.res_mgr.create(h, AlphaUbo::new(UniformValue::Float(3, hsv.h, hsv.s, hsv.v, 0.0))),
    }
}

fn modify_opacity<C: HalContext + 'static>(engine: &mut Engine<C>, render_obj: &mut RenderObj) {
    let mut bs = engine.gl.bs_get_desc(render_obj.state.bs.as_ref()).clone();
    let mut ds = engine.gl.ds_get_desc(render_obj.state.ds.as_ref()).clone();
    if render_obj.is_opacity == false {
        bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        ds.set_write_enable(false);
        
        render_obj.state.bs = create_bs_res(engine, bs);
        render_obj.state.ds = create_ds_res(engine, ds);
    }else {
        bs.set_rgb_factor(BlendFactor::One, BlendFactor::Zero);
        ds.set_write_enable(true);
        
        render_obj.state.bs = create_bs_res(engine, bs);
        render_obj.state.ds = create_ds_res(engine, ds);
    }
    
}

unsafe impl<C: HalContext + 'static> Sync for NodeAttrSys<C>{}
unsafe impl<C: HalContext + 'static> Send for NodeAttrSys<C>{}

impl_system!{
    NodeAttrSys<C> where [C: HalContext + 'static],
    true,
    {
        EntityListener<Node, CreateEvent>
        EntityListener<Node, DeleteEvent>
        SingleCaseListener<RenderObjs, CreateEvent>
        SingleCaseListener<RenderObjs, ModifyEvent>
        SingleCaseListener<RenderObjs, DeleteEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
        MultiCaseListener<Node, Visibility, ModifyEvent>
        MultiCaseListener<Node, HSV, ModifyEvent>
    }
}