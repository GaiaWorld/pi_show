/**
 * Box Shadow
 */
use std::marker::PhantomData;
use share::Share;
use std::hash::{ Hasher, Hash };
use fxhash::FxHasher32;
use fnv::{ FnvHashMap, FnvHasher };
use ecs::{ CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner };
use map::{ vecmap::VecMap } ;
use hal_core::*;
use atom::Atom;
use polygon::*;

use component::user::*;
use component::calc::*;
use component::calc::{ Opacity };
use entity::{ Node };
use single::*;
use render::engine::{ Engine };
use render::res::GeometryRes;
use system::util::*;
use system::util::constant::*;
use system::render::shaders::color::{ COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME };
use system::render::util::*;

pub struct BoxShadowSys {
    render_map: VecMap<usize>,
    dirty_ty: usize,
    
}

impl Default for BoxShadowSys {
    fn default() -> Self {
        let dirty_ty = StyleType::BoxShadow as usize 
            | StyleType::Matrix as usize 
            | StyleType::BorderRadius as usize 
            | StyleType::Opacity as usize
            | StyleType::Layout as usize;

        Self {
            dirty_ty,
            render_map: VecMap::default(),
            
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a> Runner<'a> for BoxShadowSys {
    type ReadData = (
        &'a MultiCaseImpl<Node, BoxShadow>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, Layout>,

        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, ClassName>,
        &'a MultiCaseImpl<Node, StyleMark>,

        &'a SingleCaseImpl<DefaultTable>,
        &'a SingleCaseImpl<ClassSheet>,
        &'a SingleCaseImpl<DirtyList>,
        &'a SingleCaseImpl<DefaultState>,
    );

    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<Engine>);

    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        let (
            box_shadows,
            world_matrixs,
            border_radiuses,
            opacitys,
            layouts,

            z_depths,
            transforms,
            classes,
            style_marks,

            default_table,
            _class_sheet,
            dirty_list,
            default_state,
        ) = read;

        let (render_objs, engine) = write;
        
        let default_transform = default_table.get::<Transform>().unwrap();
        
        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => {
                    // 如果style_mark不存在， node也一定不存在， 应该删除对应的渲染对象
                    self.remove_render_obj(*id, render_objs);
                    continue;
                },
            };

            let dirty = style_mark.dirty;

            // 不存在BuckgroundColor关心的脏, 跳过
            if dirty & self.dirty_ty == 0 {
                continue;
            }

            // 阴影脏，如果不存在BoxShadow本地样式和class样式， 删除渲染对象
            if dirty & StyleType::BoxShadow as usize != 0 
            && style_mark.local_style & StyleType::BoxShadow as usize == 0 
            && style_mark.class_style & StyleType::BoxShadow as usize == 0 {
                self.remove_render_obj(*id, render_objs);
                continue;
            }

            // 不存在，则创建渲染对象
            let render_index = match self.render_map.get_mut(*id) {
                Some(r) => *r,
                None => self.create_render_obj(*id, 0.0, false, render_objs, default_state),
            };

            // 从组件中取出对应的数据
            let render_obj = unsafe {render_objs.get_unchecked_mut(render_index)};

            let border_radius = border_radiuses.get(*id);
            let layout = unsafe {layouts.get_unchecked(*id)};
            let shadow = unsafe {box_shadows.get_unchecked(*id)};

            // 如果Color脏， 或Opacity脏， 计算is_opacity
            if dirty & StyleType::Opacity as usize != 0
            || dirty & StyleType::BoxShadow as usize != 0 {
                let opacity = unsafe {opacitys.get_unchecked(*id)}.0;
                render_obj.is_opacity = color_is_opacity(opacity, &shadow.color);
            }

            // 如果阴影脏，或者边框半径改变，则重新创建geometry
            if style_mark.dirty & StyleType::BoxShadow as usize != 0
            || style_mark.dirty & StyleType::BorderRadius as usize != 0 {
                render_obj.program_dirty =  true;
                to_ucolor_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut());

                render_obj.paramter.as_ref().set_value("uColor", create_u_color_ubo(&shadow.color, engine));
                // TODO
                // render_obj.geometry = create_shadow_geo(engine, layout, shadow, border_radius);
            }

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
            
            // TODO 矩阵脏，或者布局脏
            // if dirty & StyleType::Matrix as usize != 0 
            // || dirty & StyleType::Layout as usize != 0 {
            //     let world_matrix = unsafe { world_matrixs.get_unchecked(*id) };
            //     let transform =  match transforms.get(*id) {
            //         Some(r) => r,
            //         None => default_transform,
            //     };
            //     let depth = unsafe{z_depths.get_unchecked(*id)}.0;
            //     let is_unit_geo = match &color.0 {
            //         Color::RGBA(_) => {
            //             let radius = cal_border_radius(border_radius, layout);
            //             let g_b = geo_box(layout);
            //             if radius.x <= g_b.min.x {
            //                 true
            //             } else {
            //                 false
            //             }
            //         },
            //         Color::LinearGradient(_) => false,
            //     };
            //     modify_matrix(render_obj, depth, world_matrix, transform, layout, is_unit_geo);
            // }
        }
    }
}

impl BoxShadowSys {

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
        z_depth: f32,
        visibility: bool,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &DefaultState,
    ) -> usize{
        
        let render_obj = RenderObj {
            depth: z_depth - 0.3,
            depth_diff: -0.3,
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
        self.render_map.insert(id, index);
        index
    }
}

#[inline]
fn color_is_opacity(opacity: f32, color: &CgColor) -> bool {
    opacity == 1.0 && color.a == 1.0
}

#[inline]
fn create_u_color_ubo(c: &CgColor, engine: &mut Engine) -> Share<dyn UniformBuffer> {
    let h = f32_4_hash(c.r, c.g, c.b, c.a);
    match engine.res_mgr.get::<UColorUbo>(&h) {
        Some(r) => r,
        None => engine.res_mgr.create(h, UColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a))),
    }
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

// #[inline]
// fn create_shadow_geo(
//     engine: &mut Engine,
//     layout: &Layout,
//     shadow: &BoxShadow,
//     border_radius: Option<&BorderRadius>) -> Option<Share<GeometryRes>> {
    
//     let radius = cal_border_radius(border_radius, layout);
//     let g_b = geo_box(layout);
//     if g_b.min.x - g_b.max.x == 0.0 || g_b.min.y - g_b.max.y == 0.0 {
//         return None;
//     }

//     if radius.x <= g_b.min.x {
//         return Some(unit_quad.clone());
//     } else {
//         let mut hasher = FxHasher32::default();
//         radius_quad_hash(&mut hasher, radius.x, layout.width, layout.height);
//         let hash = hasher.finish();
//         match engine.res_mgr.get::<GeometryRes>(&hash) {
//             Some(r) => Some(r.clone()),
//             None => {
//                 println!("g_b: {:?}, radius.x - g_b.min.x: {}", g_b, radius.x - g_b.min.x);
//                 let r = split_by_radius(g_b.min.x, g_b.min.y, g_b.max.x - g_b.min.x, g_b.max.y - g_b.min.y, radius.x - g_b.min.x, None);
//                 println!("r: {:?}", r);
//                 if r.0.len() == 0 {
//                     return None;
//                 } else {
//                     let indices = to_triangle(&r.1, Vec::with_capacity(r.1.len()));
//                     println!("indices: {:?}", indices);
//                     // 创建geo， 设置attribut
//                     let positions = create_buffer(&engine.gl, BufferType::Attribute, r.0.len(), Some(BufferData::Float(r.0.as_slice())), false);
//                     let indices = create_buffer(&engine.gl, BufferType::Indices, indices.len(), Some(BufferData::Short(indices.as_slice())), false);
//                     let geo = create_geometry(&engine.gl);
//                     engine.gl.geometry_set_vertex_count(&geo, (r.0.len()/2) as u32);
//                     engine.gl.geometry_set_attribute(&geo, &AttributeName::Position, &positions, 2).unwrap();
//                     engine.gl.geometry_set_indices_short(&geo, &indices).unwrap();

//                     // 创建缓存
//                     let geo_res = GeometryRes{geo: geo, buffers: vec![Share::new(positions), Share::new(indices)]};
//                     let share_geo = engine.res_mgr.create(hash, geo_res);
//                     return Some(share_geo);
//                 }
//             }
//         }
//     }
// }

impl_system!{
    BoxShadowSys,
    true,
    {
    }
}