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
use render::res::Opacity as ROpacity;
use system::util::*;
use system::util::constant::{PROJECT_MATRIX, WORLD_MATRIX, VIEW_MATRIX, POSITION, COLOR, CLIP_indices, ALPHA, CLIP, VIEW, PROJECT, WORLD, COMMON, TEXTURE};
use system::render::shaders::text::{TEXT_FS_SHADER_NAME, TEXT_VS_SHADER_NAME};
use font::font_sheet::FontSheet;
use font::sdf_font::StaticSdfFont;

lazy_static! {
    static ref IMAGE_SHADER_NAME: Atom = Atom::from("char_block");
    static ref IMAGE_FS_SHADER_NAME: Atom = Atom::from("char_block_fs");
    static ref IMAGE_VS_SHADER_NAME: Atom = Atom::from("char_block_vs");

    static ref STROKE: Atom = Atom::from("STROKE");
    static ref UCOLOR: Atom = Atom::from("UCOLOR");
    static ref VERTEX_COLOR: Atom = Atom::from("VERTEX_COLOR");

    static ref STROKE_CLAMP: Atom = Atom::from("strokeClamp");
    static ref STROKE_COLOR: Atom = Atom::from("strokeColor");
    static ref FONT_CLAMP: Atom = Atom::from("fontClamp");  // 0-1的小数，超过这个值即认为有字体，默认传0.75
    static ref SMOOT_HRANFE: Atom = Atom::from("smoothRange");
    static ref TEXTURE: Atom = Atom::from("texture");
}

pub struct CharBlockSys<C: Context + Share>{
    charblock_render_map: VecMap<Item>,
    geometry_dirtys: Vec<usize>,
    mark: PhantomData<C>,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
    pipelines: HashMap<u64, Arc<PipelineInfo>>,
    default_sampler: Option<Arc<SamplerRes<C>>>,
}

impl<C: Context + Share> CharBlockSys<C> {
    pub fn new() -> Self{
        CharBlockSys {
            charblock_render_map: VecMap::default(),
            geometry_dirtys: Vec::new(),
            mark: PhantomData,
            rs: Arc::new(RasterState::new()),
            bs: Arc::new(BlendState::new()),
            ss: Arc::new(StencilState::new()),
            ds: Arc::new(DepthState::new()),
            pipelines: HashMap::default(),
            default_sampler: None,
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: Context + Share> Runner<'a> for CharBlockSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, TextStyle>,
        &'a MultiCaseImpl<Node, ZDepth>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let map = &mut self.charblock_render_map;
        let (layouts, z_depths) = read;
        let (render_objs, _) = write;
        for id in  self.geometry_dirtys.iter() {
            let item = unsafe { map.get_unchecked_mut(*id) };
            item.position_change = false;
            let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;
            let layout = unsafe { layouts.get_unchecked(*id) };
            let (positions, indices) = get_geo_flow(border_radius, layout, z_depth - 0.1);

            let mut render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
            let geometry = Arc::get_mut(&mut render_obj.geometry).unwrap();

            let vertex_count: u32 = (positions.len()/3) as u32;
            if vertex_count != geometry.get_vertex_count() {
                geometry.set_vertex_count(vertex_count);
            }
            geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false);
            geometry.set_indices_short(indices.as_slice(), false);
        }
        self.geometry_dirtys.clear();
    }

    fn setup(&mut self, _: Self::ReadData, write: Self::WriteData){
        let (_, engine) = write;
        let s = SamplerDesc::default();
        let hash = sampler_desc_hash(&s);
        if engine.res_mgr.samplers.get(&hash).is_none() {
            let res = SamplerRes::new(hash, engine.gl.create_sampler(Arc::new(s)).unwrap());
            self.default_sampler = Some(engine.res_mgr.samplers.create(res));
        }
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock<C>, CreateEvent> for CharBlockSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, CharBlock<C>>,
        &'a MultiCaseImpl<Node, TextStyle>,
        &'a MultiCaseImpl<Node, Font>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a SingleCaseImpl<FontSheet<C>>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs<C>>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (images, text_styles, fonts, z_depths, layouts, opacitys, font_sheet) = read;
        let (render_objs, engine) = write;
        let first_font = match font_sheet.get_first_font(&char_block.family) {
            Some(r) => r,
            None => {
                debug_println!("font is not exist: {}", **char_blockfamily);
                return;
            }
        }
        let texture = first_font.texture();
        let char_block = unsafe { images.get_unchecked(event.id) };
        let z_depth = unsafe { z_depths.get_unchecked(event.id) }.0;
        let layout = unsafe { layouts.get_unchecked(event.id) };
        let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
        let text_style = unsafe { text_styles.get_unchecked(event.id) }.0;
        let font = unsafe { fonts.get_unchecked(event.id) }.0;
        let mut defines = Vec::new();

        let mut geometry = create_geometry(&mut engine.gl);
        let mut ubos: HashMap<Atom, Arc<Uniforms<C>>> = HashMap::default();

        let mut common_ubo = engine.gl.create_uniforms();
        
        common_ubo.set_sampler(
            &TEXTURE,
            &(self.default_sampler.as_ref().unwrap().clone() as Arc<AsRef<<C as Context>::ContextSampler>>),
            &(first_font.texture() as Arc<AsRef<<C as Context>::ContextTexture>>)
        );
        ubos.insert(COMMON.clone(), Arc::new(common_ubo)); // COMMON

        match &text_style.color {
            Color::RGBA(c) => {
                let mut ucolor_ubo = engine.create_uniforms();
                ucolor_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b,c.a);
                render_obj.ubos.insert(&UCOLOR, Arc::new(ucolor_ubo));
                defines.push(UCOLOR.clone());
                debug_println!("text, id: {}, color: {:?}", event.id, c);
            },
            Color::LinearGradient(_) => {
                defines.push(VERTEX_COLOR.clone());
            },
            _ => (),
        }

        let pipeline = engine.create_pipeline(
            0,
            &TEXT_VS_SHADER_NAME.clone(),
            &TEXT_FS_SHADER_NAME.clone(),
            defines.as_slice(),
            self.rs.clone(),
            self.bs.clone(),
            self.ss.clone(),
            self.ds.clone()
        );
        
        let is_opacity = if opacity < 1.0 || !text_style.color.is_opacity() {
            false
        }else {
            true
        };
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
        self.charblock_render_map.insert(event.id, Item{index: index, position_change: true});
        self.geometry_dirtys.push(event.id);
    }
}


// 字体修改， 设置顶点数据脏
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock<C>, ModifyEvent> for CharBlockSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let item = unsafe { self.charblock_render_map.get_unchecked(event.id) };
        if item.position_change == false {
            item.position_change = true;
            self.geometry_dirtys.push(event.id);
        }
    }
}


// 删除渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock<C>, DeleteEvent> for CharBlockSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData){
        let item = self.charblock_render_map.remove(event.id).unwrap();
        let notify = write.get_notify();
        write.remove(item.index, Some(notify));
        if item.position_change == true {
            self.geometry_dirtys.remove_item(&event.id);
        }
    }
}

// 字体修改， 重新设置字体纹理
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Font, ModifyEvent> for CharBlockSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a MultiCaseImpl<Node, Font>>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (font_sheet, font) = read;
        if let Some(item) = unsafe { self.charblock_render_map.get_mut(event.id) } {
            let first_font = match font_sheet.get_first_font(&font.family) {
                Some(r) => r,
                None => {
                    debug_println!("font is not exist: {}", **font.family);
                    return;
                }
            };
            let texture = first_font.texture();
            let render_obj = unsafe { render_objs.get_mut(item.index) } {
            let common_ubo = render_obj.ubos.get_mut(&COMMON);
            let common_ubo = Arc::make_mut(common_ubo)
            common_ubo.set_sampler(
                &TEXTURE,
                &(self.default_sampler.as_ref().unwrap().clone() as Arc<AsRef<<C as Context>::ContextSampler>>),
                &(first_font.texture() as Arc<AsRef<<C as Context>::ContextTexture>>)
            );
        }
    }
}

// TextStyle修改， 设置对应的ubo和宏
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, TextStyle, ModifyEvent> for CharBlockSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, TextStyle>>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (opacitys, text_styles) = read;   
        let (render_objs, engine) = write;
        if let Some(item) = unsafe { self.charblock_render_map.get_mut(event.id) } {
            let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
            let text_style = unsafe { text_styles.get_unchecked(event.id) };
            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };

            match event.field {
                "color" => {
                    match &text_style.color {
                        Color::RGBA(c) => {
                            // 如果未找到UCOLOR宏， 表示修改之前的颜色不为RGBA， 应该删除VERTEX_COLOR宏， 添加UCOLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
                            if find_item_from_vec(&render_obj.defines, &UCOLOR) == 0 {
                                render_obj.defines.remove_item(&VERTEX_COLOR);

                                let mut ucolor_ubo = engine.create_uniforms();
                                render_obj.ubos.insert(&UCOLOR, Arc::new(ucolor_ubo));
                                render_obj.defines.push(UCOLOR.clone());

                                if item.position_change == false {
                                    item.position_change = true;
                                    self.geometry_dirtys.push(event.id);
                                }
                            }
                            // 设置ubo
                            let ucolor_ubo = Arc::make_mut(render_obj.ubos.get_mut(&UCOLOR).unwrap());
                            debug_println!("text_color, id: {}, color: {:?}", event.id, c);
                            common_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b, c.a);

                            render_objs.get_notify().modify_event(item.index, "", 0);
                        },
                        Color::LinearGradient(_) => {
                            // 如果未找到VERTEX_COLOR宏， 表示修改之前的颜色不为LinearGradient， 应该删除UCOLOR宏， 添加VERTEX_COLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
                            if find_item_from_vec(&render_obj.defines, &VERTEX_COLOR) == 0 {
                                render_obj.defines.remove_item(&UCOLOR);
                                render_obj.defines.push(VERTEX_COLOR.clone());
                                render_obj.ubos.remove(&UCOLOR);   
                                if item.position_change == false {
                                    item.position_change = true;
                                    self.geometry_dirtys.push(event.id);
                                }
                            }
                        },
                        _ => (),
                    }

                },
                "stroke" => {
                    if text_style.stroke.width == 0.0 {
                        //  删除边框的宏
                        match render_obj.defines.remove_item(&STROKE) {
                            Some(_) => {
                                // 如果边框宏存在， 删除边框对应的ubo， 重新创建渲染管线
                                render_obj.ubos.remove(&STROKE);
                                render_obj.pipeline = engine.create_pipeline(
                                    0,
                                    &TEXT_VS_SHADER_NAME.clone(),
                                    &TEXT_FS_SHADER_NAME.clone(),
                                    render_obj.defines.as_slice(),
                                    self.rs.clone(),
                                    self.bs.clone(),
                                    self.ss.clone(),
                                    self.ds.clone()
                                );
                            },
                            None => ()
                        };
                        
                    } else {
                        // 边框宽度不为0， 并且不存在STROKE宏， 应该添加STROKE宏， 并添加边框对应的ubo， 且重新创建渲染管线
                        if find_item_from_vec(&render_obj.defines, &STROKE) == 0 {
                            render_obj.defines.push(STROKE);
                            let mut stroke_ubo = engine.create_uniforms();
                            let color = &text_style.color;
                            stroke_ubo.set_float_1(&STROKE_SIZE, text_style.stroke.width);
                            stroke_ubo.set_float_4(&STROKE_COLOR, color.r, color.g, color.b, color.a);
                            render_obj.ubos.insert(&STROKE, Arc::new(stroke_ubo));
                            render_obj.pipeline = engine.create_pipeline(
                                0,
                                &TEXT_VS_SHADER_NAME.clone(),
                                &TEXT_FS_SHADER_NAME.clone(),
                                render_obj.defines.as_slice(),
                                self.rs.clone(),
                                self.bs.clone(),
                                self.ss.clone(),
                                self.ds.clone()
                            );
                        }
                    }
                },
                _ => return,
            }

            let is_opacity = if opacity < 1.0 || !text_style.color.is_opacity() {
                false
            }else {
                true
            };
            let notify = render_objs.get_notify();
            unsafe { render_objs.get_unchecked_write(item.index, &notify).set_is_opacity(is_opacity)};
        }
    }
}

//不透明度变化， 修改渲染对象的is_opacity属性
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for CharBlockSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, CharBlock<C>>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (opacitys, images) = read;
        if let Some(item) = unsafe { self.charblock_render_map.get(event.id) } {
            let opacity = unsafe { opacitys.get_unchecked(id).0 };
            let char_block = unsafe { images.get_unchecked(id) };
            self.change_is_opacity(event.id, opacity, char_block, item, write);
        }
    }
}

impl<'a, C: Context + Share> CharBlockSys<C> {
    fn change_is_opacity(&mut self, id: usize, opacity: f32, char_block: &CharBlock<C>, item: &Item, render_objs: &mut SingleCaseImpl<RenderObjs<C>>){
        let is_opacity = if opacity < 1.0 {
            false
        }else if let ROpacity::Opaque = char_block.src.opacity{
            true
        }else {
            false
        };

        let notify = render_objs.get_notify();
        unsafe { render_objs.get_unchecked_write(item.index, &notify).set_is_opacity(is_opacity)};
    }
}

struct Item {
    index: usize,
    position_change: bool,
}

//取几何体的顶点流、 颜色流和属性流
fn get_geo_flow(char_block: &CharBlock, color: &Color, layout: &Layout, z_depth: f32) -> (Vec<f32>, Option<Vec<f32>>, Vec<u16>) {
    unimplemented!()
    

    // (positions, uvs, indices)
}

unsafe impl<C: Context + Share> Sync for CharBlockSys<C>{}
unsafe impl<C: Context + Share> Send for CharBlockSys<C>{}

impl_system!{
    CharBlockSys<C> where [C: Context + Share],
    true,
    {
        MultiCaseListener<Node, CharBlock<C>, CreateEvent>
        MultiCaseListener<Node, CharBlock<C>, ModifyEvent>
        MultiCaseListener<Node, CharBlock<C>, DeleteEvent>
        MultiCaseListener<Node, Layout, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
    }
}