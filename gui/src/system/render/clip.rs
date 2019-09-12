use std::marker::PhantomData;

// 监听Overflow， 绘制裁剪纹理
use share::Share;

use ordered_float::OrderedFloat;

use ecs::{ModifyEvent, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
use ecs::monitor::NotifyImpl;
use hal_core::*;
use map::vecmap::VecMap;

use component::calc::*;
use component::user::{Aabb3};
use entity::{Node};
use single::*;
use render::engine:: { ShareEngine, Engine };
use render::res::*;
use system::render::shaders::clip::*;

pub struct ClipSys<C>{
    dirty: bool,
    no_rotate_dirtys: VecMap<bool>,
    
    render_obj: Option<ClipTextureRender>,
    marker: PhantomData<C>,
}

struct ClipTextureRender{
    clip_size_ubo: Share<ClipTextureSize>,
    sampler: Share<SamplerRes>,

    rs: Share<RasterStateRes>,
    bs: Share<BlendStateRes>,
    ss: Share<StencilStateRes>,
    ds: Share<DepthStateRes>,
    render_target: HalRenderTarget,
    program: Share<HalProgram>,
    geometry: HalGeometry,

    paramter: Share<dyn ProgramParamter>,
    begin_desc: RenderBeginDesc,
}

impl<C: HalContext + 'static> ClipSys<C>{
    pub fn init_render(&mut self, engine: &mut Engine<C>,  viewport: &(i32, i32, i32, i32), view_matrix: &ViewMatrix, projection_matrix: &ProjectionMatrix){
        let (rs, mut bs, ss, mut ds) = (RasterStateDesc::default(), BlendStateDesc::default(), StencilStateDesc::default(), DepthStateDesc::default());
        bs.set_rgb_factor(BlendFactor::One, BlendFactor::One);
        ds.set_test_enable(false);
        ds.set_write_enable(false);

        let paramter = ClipParamter::default();

        let geo = engine.create_geometry();

        let program = engine.create_program(
            CLIP_VS_SHADER_NAME.get_hash(),
            CLIP_FS_SHADER_NAME.get_hash(),
            CLIP_VS_SHADER_NAME.as_ref(),
            &VsDefines::default(),
            CLIP_FS_SHADER_NAME.as_ref(),
            &FsDefines::default(),
            &paramter,
        );

        let size = next_power_of_two((viewport.2 as u32).max(viewport.2 as u32));
        let target = engine.gl.rt_create(size, size, PixelFormat::RGB, DataFormat::UnsignedByte, false).unwrap();
        let mut clip_size_ubo = ClipTextureSize::default();
        clip_size_ubo.set_value("clipTextureSize", UniformValue::Float1(size as f32));

        let slice: &[f32; 16] = view_matrix.0.as_ref();
        let view_matrix_ubo = ViewMatrixUbo::new(UniformValue::MatrixV4(Vec::from(&slice[..])));

        let slice: &[f32; 16] = projection_matrix.0.as_ref();
        let project_matrix_ubo = ProjectMatrixUbo::new(UniformValue::MatrixV4(Vec::from(&slice[..])));

        paramter.set_value("viewMatrix", Share::new(view_matrix_ubo)); // VIEW_MATRIX
        paramter.set_value("projectMatrix", Share::new(project_matrix_ubo)); // PROJECT_MATRIX

        self.render_obj = Some(ClipTextureRender {
            clip_size_ubo: Share::new(clip_size_ubo),
            sampler: engine.create_sampler_res(SamplerDesc::default()),

            rs: engine.create_rs_res(rs),
            bs: engine.create_bs_res(bs),
            ss: engine.create_ss_res(ss),
            ds: engine.create_ds_res(ds),

            render_target: target,
            program: program,
            geometry: geo,

            paramter: Share::new(paramter),
            begin_desc: RenderBeginDesc{
                viewport: (viewport.0, viewport.1, viewport.2, viewport.3),
                clear_color: Some((OrderedFloat(0.0), OrderedFloat(0.0), OrderedFloat(0.0), OrderedFloat(1.0))),
                clear_depth: None,
                clear_stencil: None,
            },
        });
    }

    pub fn new() -> Self{
        Self {
            dirty: false,
            no_rotate_dirtys: VecMap::default(),
            render_obj: None,
            marker: PhantomData,
        }
    }

    fn set_clip_uniform(
        &self, 
        id: usize, 
        by_overflow: usize, 
        aabb: Option<&(Aabb3, Share<dyn UniformBuffer>)>, 
        notify: &NotifyImpl, 
        render_obj: &mut RenderObj, 
        engine: &mut Engine<C>,
    ){
        match aabb {
            Some(item) => {
                // if render_obj.visibility {
                    render_obj.paramter.set_value("clipBox", item.1.clone());
                    if let None = render_obj.fs_defines.add("CLIP_BOX") {
                        render_obj.vs_defines.add("CLIP_BOX");
                        render_obj.fs_defines.remove("CLIP");
                        notify.modify_event(id, "program_dirty", 0);
                    }
                // }
            },
            None => {
                render_obj.paramter.set_single_uniform("clipIndices", UniformValue::Float1(by_overflow as f32));
                let clip_render = self.render_obj.as_ref().unwrap();
                // 插入裁剪ubo 插入裁剪宏
                if let None = render_obj.fs_defines.add("CLIP") {
                    render_obj.vs_defines.remove("CLIP_BOX");
                    render_obj.fs_defines.remove("CLIP_BOX");
                    render_obj.paramter.set_texture("clipTexture",  (engine.gl.rt_get_color_texture(&clip_render.render_target, 0).unwrap(), &clip_render.sampler) );
                    render_obj.paramter.set_value("clipTextureSize",  clip_render.clip_size_ubo.clone());
                    notify.modify_event(id, "program_dirty", 0);
                }
            },
        }
    }
}

impl<'a, C: HalContext + 'static> Runner<'a> for ClipSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a SingleCaseImpl<DirtyList>,
        &'a SingleCaseImpl<NodeRenderMap>,
        &'a SingleCaseImpl<OverflowClip>,
        &'a SingleCaseImpl<ProjectionMatrix>,
        &'a SingleCaseImpl<ViewMatrix>,
        &'a SingleCaseImpl<RenderBegin>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<OverflowClip>, 
        &'a mut MultiCaseImpl<Node, Culling>, 
        &'a mut SingleCaseImpl<RenderObjs>, 
        &'a mut SingleCaseImpl<ShareEngine<C>>,
    );
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let (by_overflows, style_marks, dirty_list, node_render_map, overflow, projection, view, view_port) = read;
        let (overflow_clip, cullings, render_objs, engine) = write;

        if self.dirty {
            self.dirty = false;

            if self.render_obj.is_none() {
                self.init_render(engine, &view_port.0.viewport, view, projection);
            }
            let clip_render = self.render_obj.as_ref().unwrap();
            
            let mut positions = Vec::default();
            for (_i, c) in overflow.clip.iter() {
                if c.has_rotate {
                    let p = &c.view;
                    positions.push(p[0].x);
                    positions.push(p[0].y);
                    positions.push(p[1].x);
                    positions.push(p[1].y);
                    positions.push(p[2].x);
                    positions.push(p[2].y);
                    positions.push(p[3].x);
                    positions.push(p[3].y);
                } 
            }

            let mut mumbers: Vec<f32> = Vec::new();
            let mut indices: Vec<u16> = Vec::new();

            let mut count = positions.len()/8;
            if count > 16 {
                count = 16;
                unsafe {positions.set_len(128)};
            }
            for i in 0..count as u16 {
                mumbers.extend_from_slice(&[i as f32, i as f32, i as f32, i as f32]);
                indices.extend_from_slice(&[4 * i + 0, 4 * i + 1, 4 * i + 2, 4 * i + 0, 4 * i + 2, 4 * i + 3]);
            }

            let p_buffer = engine.create_buffer(BufferType::Attribute, 128, None, false);
            let m_buffer = engine.create_buffer(BufferType::Attribute, mumbers.len(), Some(BufferData::Float(mumbers.as_slice())), false);
            let i_buffer = engine.create_buffer(BufferType::Indices, indices.len(), Some(BufferData::Short(indices.as_slice())), false);
            
            engine.gl.geometry_set_attribute(&clip_render.geometry, &AttributeName::Position, &p_buffer, 2).unwrap();
            engine.gl.geometry_set_attribute(&clip_render.geometry, &AttributeName::SkinIndex, &m_buffer, 1).unwrap();
            engine.gl.geometry_set_indices_short(&clip_render.geometry, &i_buffer).unwrap();
            clip_render.paramter.set_single_uniform("meshNum", UniformValue::Float1(count as f32));

            // 渲染裁剪平面
            engine.gl.render_begin(Some(&clip_render.render_target), &clip_render.begin_desc);
            engine.gl.render_set_program(&clip_render.program);
            engine.gl.render_set_state(&clip_render.bs, &clip_render.ds, &clip_render.rs, &clip_render.ss);
            engine.gl.render_draw(&clip_render.geometry, &clip_render.paramter);
            engine.gl.render_end();
        } 

        let notify = render_objs.get_notify();
        let mut pre_by_overflow = 0;
        let mut aabb = None;
        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => continue,
            };

            let by_overflow = unsafe {by_overflows.get_unchecked(*id)}.0;
            let obj_ids = unsafe{ node_render_map.get_unchecked(*id) };

            if (style_mark.dirty & StyleType::Matrix as usize != 0 || style_mark.dirty & StyleType::ByOverflow as usize != 0) && by_overflow > 0 {
                if by_overflow != pre_by_overflow {
                    pre_by_overflow = by_overflow;
                    aabb = overflow_clip.clip_map.get(&by_overflow);
                }

                for id in obj_ids.iter() {
                    let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
                    self.set_clip_uniform(*id, by_overflow, aabb, &notify, render_obj, engine);
                }
            } else if style_mark.dirty & StyleType::ByOverflow as usize != 0 && by_overflow == 0 {
                // 裁剪剔除
                unsafe {cullings.get_unchecked_write(*id) }.set_0(false);
                for id in obj_ids.iter() {
                    let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
                    render_obj.vs_defines.remove("CLIP_BOX");
                    render_obj.fs_defines.remove("CLIP_BOX");
                    render_obj.fs_defines.remove("CLIP");
                    render_objs.get_notify().modify_event(*id, "program_dirty", 0);
                }
            }  
        }
    }
}

// //创建RenderObj， 为renderobj添加裁剪宏及ubo
// impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, CreateEvent> for ClipSys<C>{
//     type ReadData = &'a MultiCaseImpl<Node, ByOverflow>;
//     type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<ShareEngine<C>>, &'a mut SingleCaseImpl<OverflowClip>);
//     fn listen(&mut self, event: &CreateEvent, by_overflows: Self::ReadData, write: Self::WriteData){
//         let (render_objs, engine, overflow_clip) = write;
//         let notify = render_objs.get_notify();
//         let render_obj = unsafe { render_objs.get_unchecked_mut(event.id) };
//         let node_id = render_obj.context;
//         let by_overflow = unsafe { by_overflows.get_unchecked(node_id).0 };
//         if by_overflow > 0 {
//             let aabb = overflow_clip.clip_map.get(&by_overflow);
//             self.set_clip_uniform(event.id, by_overflow, aabb, &notify, render_obj, engine);
//         }
//     }
// }


// //by_overfolw变化， 设置ubo， 修改宏， 并重新创建渲染管线
// impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for ClipSys<C>{
//     type ReadData = (&'a MultiCaseImpl<Node, ByOverflow>, &'a SingleCaseImpl<NodeRenderMap>, &'a SingleCaseImpl<OverflowClip>, &'a SingleCaseImpl<Oct>);
//     type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<ShareEngine<C>>, &'a mut MultiCaseImpl<Node, Culling>);
//     fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
//         let (by_overflows, node_render_map, overflow_clip, octree) = read;
//         let (render_objs, engine, cullings) = write;
//         let by_overflow = unsafe { by_overflows.get_unchecked(event.id).0 };
//         let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };
//         let notify = render_objs.get_notify();
//         // ByOverflow
//         if by_overflow == 0 {
//             // 裁剪剔除
//             unsafe {cullings.get_unchecked_write(event.id) }.set_0(false);
//             for id in obj_ids.iter() {
//                 let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
//                 render_obj.vs_defines.add("CLIP_BOX");
//                 render_obj.fs_defines.add("CLIP_BOX");
//                 render_obj.fs_defines.remove("CLIP");
//                 render_objs.get_notify().modify_event(*id, "program_dirty", 0);
//             }
//         } else {
//             let mut aabb = None;
//             // 裁剪剔除
//             if let Some(item) = overflow_clip.clip_map.get(&by_overflow) {
//                 unsafe { cullings.get_unchecked_write(event.id) }.set_0(!is_intersect(&item.0, &unsafe { octree.get_unchecked(event.id) }.0));
//                 aabb = Some(item);
//             }
//             for id in obj_ids.iter() {
//                 let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
//                 self.set_clip_uniform(*id, by_overflow, aabb, &notify, render_obj, engine);
//             }
//         }
//     }
// }

// // 世界矩阵改变， 如果该节点by_overflow > 0, 应该判断其是否被裁剪平面剔除
// impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for ClipSys<C>{
//     type ReadData = (&'a MultiCaseImpl<Node, ByOverflow>, &'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<NodeRenderMap>);
//     type WriteData = (&'a mut SingleCaseImpl<OverflowClip>, &'a mut MultiCaseImpl<Node, Culling>, &'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<ShareEngine<C>>);
//     fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
//         let (by_overflows, octree, node_render_map) = read;
//         let (overflow_clip, cullings, render_objs, engine) = write;

//         let by_overflow = unsafe {by_overflows.get_unchecked(event.id)}.0;
//         if by_overflow > 0 {
//             let notify = render_objs.get_notify();
//             let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };
//             let mut aabb = None;
//             // 裁剪剔除
//             if let Some(item) = overflow_clip.clip_map.get(&by_overflow) {
//                 unsafe { cullings.get_unchecked_write(event.id) }.set_0(!is_intersect(&item.0, &unsafe { octree.get_unchecked(event.id) }.0));
//                 aabb = Some(item);
//             }
//             for id in obj_ids.iter() {
//                 let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
//                 self.set_clip_uniform(*id, by_overflow, aabb, &notify, render_obj, engine);
//             }
//         }
//     }
// }


impl<'a, C: HalContext + 'static> SingleCaseListener<'a, OverflowClip, ModifyEvent> for ClipSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<OverflowClip>;
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, write: Self::WriteData) {
        let c = unsafe { write.clip.get_unchecked(event.id) };
        if c.has_rotate || c.old_has_rotate {
            self.dirty = true;
        }
    }
}



// impl<C: HalContext + 'static> ClipSys<C>{
//     pub fn culling(engine: &mut Engine<C>, id_tree: &IdTree) {
//         if self.no_rotate_dirty
//     }
// }

// /// 计算剔除用的包围盒
// fn cal_culling_box(
//     mut by_overflow: usize,
//     overflow: &OverflowClip,
// ) -> Option<Aabb3> {
//     let mut i = 1;
//     let mut r = Some(Aabb3::new(Point3::new(std::f32::MIN, std::f32::MIN, 0.0), Point3::new(std::f32::MAX, std::f32::MAX, 0.0)));
//     while i < 33 {
//         if by_overflow & (1 << i) != 0 {
//             let b = &unsafe { overflow.clip.get_unchecked(i)};
//             if b.has_rotate {
//                 return None;
//             }
//             let b = Aabb3::new(Point3::new(b.view[0].x, b.view[0].y, 0.0), Point3::new(b.view[4].x, b.view[4].y, 0.0));
//             r = intersect(r.as_ref().unwrap(), &b);
//         }
//         i += 1;
//     }
//     r
// }

// /// 递归剔除
// fn recursive_culling(
//     node_id: usize,
//     mut culling_box: &Aabb3,
//     mark: &mut usize,
//     overflows: &MultiCaseImpl<Node, Overflow>,
//     by_overflows: &MultiCaseImpl<Node, ByOverflow>,
//     cullings: &MultiCaseImpl<Node, Culling>,
//     overflow: &OverflowClip,
//     octree: &Oct,
//     id_tree: &IdTree,
// ) {
//     for (id, _n) in id_tree.iter(node_id) {
//         let is_overflow = unsafe {overflows.get_unchecked(id)}.0;
//         let re;
//         if is_overflow {
//             let c_index = overflow.id_map.get(&id).unwrap();
//             if *c_index-1 & *mark != 0 {
//                 continue;
//             }
//             *mark |= c_index-1;
//             let c = unsafe { overflow.clip.get_unchecked(*c_index) };

//             match overflow.clip.get()
//             match intersect(culling_box, &Aabb3::new(Point3::new(c.view[0].x, c.view[0].y, 0.0), Point3::new(c.view[2].x, c.view[2].y, 0.0))) {
//                 Some(r) => re = r,
//                 None => continue,
//             };
//             culling_box = &re;  
//         };
        
//         cullings.insert(id, Culling(!is_intersect(culling_box, &unsafe { octree.get_unchecked(id) }.0)));
//         recursive_culling(id, culling_box, mark, overflows, by_overflows, cullings, overflow, octree, id_tree);
//     }
// }

// 是否相交
#[inline]
fn is_intersect(a: &Aabb3, b: &Aabb3) -> bool {
    if a.min.x >= b.max.x || a.min.y > b.max.y || b.min.x > a.max.x || b.min.y > a.max.y{
        return false;
    } else {
        true
    }
}

// a是否包含b
#[inline]
fn is_include(a: &Aabb3, b: &Aabb3) -> bool {
    if a.min.x <= b.min.x && a.max.x >= b.max.x && a.min.y <= b.min.y && a.max.y >= b.max.y {
        return true;
    } else {
        false
    }
}


// // aabb相交
// fn intersect(a: &Aabb3, b: &Aabb3) -> Option<Aabb3> {
//     if is_intersect(a, b){
//         return None;
//     }
//     if a.min.x >= b.min.x {
//         //a在b的右下
//         if a.min.y >= b.min.y {
//             return Some(Aabb3::new(a.min, b.max));
//         } else { // 右上
//             return Some(Aabb3::new(Point3::new(a.min.x, b.min.y, 0.0), Point3::new(b.max.x, a.max.y, 0.0)));
//         }
//     } else {
//         //b在a的右下
//         if b.min.y >= a.min.y {
//             return Some(Aabb3::new(b.min, a.max));
//         } else { // 右上
//             return Some(Aabb3::new(Point3::new(b.min.x, a.min.y, 0.0), Point3::new(a.max.x, b.max.y, 0.0)));
//         }
//     }
// }


fn next_power_of_two(value: u32) -> u32 {
    let mut value = value - 1;
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
    value += 1;
    value
}

impl_system!{
    ClipSys<C> where [C: HalContext + 'static],
    true,
    {
        SingleCaseListener<OverflowClip, ModifyEvent>
        // SingleCaseListener<RenderObjs, CreateEvent>
        // MultiCaseListener<Node, ByOverflow, ModifyEvent>
        // MultiCaseListener<Node, Oct, ModifyEvent>  
        // EntityListener<Node, CreateEvent>
    }
}