/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use share::Share;
use std::hash::{ Hasher, Hash };

use fnv::{FnvHashMap, FnvHasher};
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
use map::{ vecmap::VecMap } ;
use hal_core::*;
use atom::Atom;
use polygon::*;

use component::user::*;
use component::calc::*;
use component::calc::{Opacity};
use entity::{Node};
use single::*;
use render::engine::{ Engine};
use render::res::GeometryRes;
use system::util::*;
use system::util::constant::*;
use system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};
use system::render::util::*;

pub struct BoxShadowSys<C: HalContext + 'static>{
    items: Items<usize>,
    share_ucolor_ubo: VecMap<Share<dyn UniformBuffer>>, // 如果存在BoxShadowClass， 也存在对应的ubo
    mark: PhantomData<C>,
}

impl<C: HalContext + 'static> Default for BoxShadowSys<C> {
    fn default() -> Self {
        Self {
            items: Items::default(),
            share_ucolor_ubo: VecMap::default(),
            mark: PhantomData,
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for BoxShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, BorderRadius>,
        &'a MultiCaseImpl<Node, BoxShadow>,
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
            box_shadows,
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
            
            // 因为不一定有border半径，所以要用Option。
            let border_radius = border_radiuses.get(*id);
            
            let layout = unsafe {layouts.get_unchecked(*id)};
            let color;

        }
        self.items.dirtys.clear();
    }
}

// 插入渲染对象
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BoxShadow, CreateEvent> for BoxShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a SingleCaseImpl<DefaultState>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;

    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, render_objs: Self::WriteData) {
        // 如果已经存在渲染对象，设置阴影脏，返回；
        if self.items.render_map.get(event.id).is_some() {
            self.items.set_dirty(event.id, DrityType::Color as usize);
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
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BoxShadow, ModifyEvent> for BoxShadowSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, BoxShadow>, &'a MultiCaseImpl<Node, Opacity>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &ModifyEvent, _: Self::ReadData, _: Self::WriteData){
        self.items.set_dirty(event.id, DrityType::Color as usize);
    }
}

// 删除渲染对象
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BoxShadow, DeleteEvent> for BoxShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ClassName>, 
        &'a SingleCaseImpl<ClassSheet>,  
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (class_names, class_sheet) = read;
        // 小燕：如果class中存在BoxShadow，设脏，目的是？
        let class_name = unsafe { class_names.get_unchecked(event.id) };
        if let Some(class) = class_sheet.class.get(class_name.0) {
            if let Some(_) = class_sheet.box_shadow.get(class.box_shadow) {
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
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ClassName, ModifyEvent> for BoxShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, ClassName>,
        &'a MultiCaseImpl<Node, BoxShadow>,

        &'a SingleCaseImpl<ClassSheet>,
        &'a SingleCaseImpl<DefaultState>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){

        // 如果class中含有BoxShadow的描述， 创建一个渲染对象
        // 小燕：如何防止修改别的class属性的修改时候也进来。
        let (z_depths, visibilitys, class_names, box_shadows, class_sheet, default_state) = read;
        if let Some(class_name) = class_names.get(event.id) {
            if let Some(class) = class_sheet.class.get(class_name.0) {
                if class.background_color > 0 {
                    // 如果已经存在， 设置color脏 返回
                    if box_shadows.get(event.id).is_some() {
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

        if box_shadows.get(event.id).is_some()  {
            return;
        }

        // 如果class中不存在boxshadow， style中也不存在， 应该删除渲染对象
        if let Some(item) = self.items.render_map.remove(event.id) {
            let notify = render_objs.get_notify();
            render_objs.remove(item.index, Some(notify));
        }
    }
}

// 监听一个backgroundColorClass的创建， 如果backgroundColor是rgba类型， 创建一个对应的ubo
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, ClassSheet, CreateEvent> for BoxShadowSys<C>{
    type ReadData = &'a SingleCaseImpl<ClassSheet>;
    type WriteData = &'a mut SingleCaseImpl<Engine<C>>;
    fn listen(&mut self, event: &CreateEvent, class_sheet: Self::ReadData, engine: Self::WriteData){
        let class = unsafe { class_sheet.class.get_unchecked(event.id)};

        if class.box_shadow > 0 {
            if let Color::RGBA(c) = unsafe { &class_sheet.box_shadow.get_unchecked(class.box_shadow).0 } {
                self.share_ucolor_ubo.insert(event.id, create_u_color_ubo(c, engine));
            }
        }
    }
}

impl<C: HalContext + 'static> BoxShadowSys<C> {
    
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
        self.items.create(id, index);
        index
    }
}
      
enum DrityType {
    // 阴影相关属性
    OffsetX = 1,
    OffsetY = 2,
    Blur = 4,
    Spread = 8,
    Color = 16,
    
    // 通用属性
    BorderRadius = 32,
    Matrix = 64,
    Opacity = 128,
    Layout = 512,
}

impl_system!{
    BoxShadowSys<C> where [C: HalContext + 'static],
    true,
    {
        MultiCaseListener<Node, BoxShadow, CreateEvent>
        MultiCaseListener<Node, BoxShadow, ModifyEvent>
        MultiCaseListener<Node, BoxShadow, DeleteEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
        MultiCaseListener<Node, WorldMatrix, ModifyEvent>
        MultiCaseListener<Node, BorderRadius, ModifyEvent>
        MultiCaseListener<Node, ClassName, ModifyEvent>
    }
}