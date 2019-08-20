/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use std::hash::{ Hash, Hasher };

use ordered_float::NotNan;
use fxhash::FxHasher32;

use ecs::{SingleCaseImpl, MultiCaseImpl, Runner, SingleCaseListener, MultiCaseListener, DeleteEvent, ModifyEvent};
use ecs::monitor::NotifyImpl;
use hal_core::*;
use polygon::{mult_to_triangle, interp_mult_by_lg, split_by_lg, LgCfg, find_lg_endp};
use share::Share;
use map::vecmap::VecMap;

use component::user::*;
use single::*;
use component::calc::*;
use entity::{Node};
use render::engine::{ Engine};
use render::res::*;
use system::util::*;
use system::render::shaders::text::{TEXT_FS_SHADER_NAME, TEXT_VS_SHADER_NAME};
use system::render::shaders::canvas_text::{CANVAS_TEXT_VS_SHADER_NAME, CANVAS_TEXT_FS_SHADER_NAME};
use font::font_sheet::*;
use layout::FlexNode;

const TEXT_STYLE_DIRTY: usize = StyleType::LetterSpacing as usize | 
                                StyleType::WordSpacing as usize |
                                StyleType::LineHeight as usize |
                                StyleType::Indent as usize |
                                StyleType::WhiteSpace as usize | 
                                StyleType::TextAlign as usize | 
                                StyleType::VerticalAlign as usize | 
                                StyleType::Color as usize | 
                                StyleType::Stroke as usize |
                                StyleType::FontStyle as usize | 
                                StyleType::FontFamily as usize | 
                                StyleType::FontSize as usize | 
                                StyleType::FontWeight as usize |
                                StyleType::TextShadow as usize |
                                StyleType::Layout as usize |
                                StyleType::Text as usize |
                                StyleType::Matrix as usize;
const TEXT_LAYOUT_DIRTY: usize =    StyleType::FontStyle as usize |
                                    StyleType::FontWeight as usize |
                                    StyleType::FontSize as usize |
                                    StyleType::FontFamily as usize |
                                    StyleType::LetterSpacing as usize |
                                    StyleType::WordSpacing as usize |
                                    StyleType::LineHeight as usize |
                                    StyleType::Indent as usize |
                                    StyleType::WhiteSpace as usize |
                                    StyleType::TextAlign as usize |
                                    StyleType::VerticalAlign as usize;

#[derive(Default, Clone, Debug)]
struct I{ text: usize, shadow: usize }

struct RenderCatch{
    fill_color_ubo: Share<dyn UniformBuffer>,
    stroke_ubo: Share<dyn UniformBuffer>,
    stroke_color_ubo: Share<dyn UniformBuffer>,
    shadow_color_ubo: Share<dyn UniformBuffer>,
    layout_hash: u64, // 布局属性的hash
}

pub struct CharBlockSys<L: FlexNode + 'static, C: HalContext + 'static>{
    render_map: VecMap<I>,
    canvas_bs: Share<HalBlendState>,
    msdf_bs: Share<HalBlendState>,
    default_sampler: Share<HalSampler>, // 默认采样方式
    point_sampler: Share<HalSampler>, // 点采样， canvas着色器渲染时， 如果字体大小与纹理默认字体大小一致， 将采用点采样
    canvas_default_stroke_color: Share<CanvasTextStrokeColorUbo>,
    class_ubos: VecMap<RenderCatch>,
    default_ubos: RenderCatch,
    index_buffer: Share<HalBuffer>, // 索引 buffer， 长度： 600
    index_len: usize,
    texture_size_ubo: Share<TextTextureSize>,
    texture_change: bool,
    mark: PhantomData<(L, C)>,
} 

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, L: FlexNode + 'static, C: HalContext + 'static> Runner<'a> for CharBlockSys<L, C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, TextContent>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a MultiCaseImpl<Node, TextStyle>,
        &'a SingleCaseImpl<FontSheet>,
        &'a SingleCaseImpl<DefaultTable>,
        &'a SingleCaseImpl<DefaultState>,
        &'a SingleCaseImpl<ClassSheet>,
        &'a SingleCaseImpl<DirtyList>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<Engine<C>>, &'a mut MultiCaseImpl<Node, CharBlock<L>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let (world_matrixs, layouts, transforms, texts, style_marks, text_styles, font_sheet, default_table, default_state, class_sheet, dirty_list) = read;
        let (render_objs, engine, charblocks) = write;
        let notify = render_objs.get_notify();
        let default_transform = default_table.get::<Transform>().unwrap();

        if self.texture_change == true {
            let t = font_sheet.get_font_tex();
            self.texture_size_ubo = Share::new(TextTextureSize::new(UniformValue::Float2(t.width as f32, t.height as f32)));
            for i in self.render_map.iter() {
                match i {
                    Some(i) => {
                        unsafe { render_objs.get_unchecked(i.text)  }.paramter.set_value("textureSize", self.texture_size_ubo.clone());
                        if i.shadow > 0 {
                            unsafe { render_objs.get_unchecked(i.shadow)  }.paramter.set_value("textureSize", self.texture_size_ubo.clone());
                        }
                        notify.modify_event(i.text, "ubo", 0);
                    },
                    None => (),
                }
            }
            self.texture_change = false;
        }

        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => continue,
            };

            let mut dirty = style_mark.dirty;

            // 不存在Chablock关心的脏, 跳过
            if dirty & TEXT_STYLE_DIRTY == 0 {
                continue;
            }

            // 如果Text脏， 并且不存在Text组件， 尝试删除渲染对象
            let (index, charblock, text_style) = if dirty & StyleType::FontFamily as usize != 0 {
                if style_mark.local_style & StyleType::FontFamily as usize == 0 && style_mark.class_style & StyleType::FontFamily as usize == 0 {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                } else {
                    let text_style = unsafe { text_styles.get_unchecked(*id) };
                    let charblock = unsafe { charblocks.get_unchecked(*id) };
                    let have_shadow = text_style.shadow.color != CgColor::new(0.0, 0.0, 0.0, 0.0);
                    let r = self.create_render_obj(*id, render_objs, default_state, have_shadow, charblock.is_pixel);
                    dirty = dirty | TEXT_STYLE_DIRTY;
                    (r, charblock, text_style)
                }  
            } else {
                match self.render_map.get(*id) {
                    Some(r) => (r.clone(), unsafe { charblocks.get_unchecked(*id) }, unsafe { text_styles.get_unchecked(*id) }),
                    None => continue,
                }
            };

            let world_matrix = unsafe{ world_matrixs.get_unchecked(*id) };
            let layout = unsafe{ layouts.get_unchecked(*id) };
            let transform = match transforms.get(*id){
                Some(r) => r,
                None => default_transform,
            };

            let render_obj = unsafe { &mut *(render_objs.get_unchecked(index.text) as *const RenderObj as usize as *mut RenderObj) };

            let text = unsafe { texts.get_unchecked(*id) };

            let class_ubo = if let Some(class) = class_sheet.class.get(charblock.style_class) {
                if let Some(ubos) = self.class_ubos.get(class.text) {
                    ubos
                } else {
                    &self.default_ubos
                }
            }else {
                &self.default_ubos
            };

            let (mut program_change, mut geometry_change) = (false, false);
            let mut shadow_geometry_change  = false;

            // 颜色脏， 如果是渐变色和纯色切换， 需要修改宏， 且顶点需要重新计算， 因此标记program_change 和 geometry_change脏
            if dirty & (StyleType::Color as usize) != 0 {
                let exchange = modify_color(index.text, style_mark.local_style, &text_style.text.color, engine, &notify, render_obj, &class_ubo);
                program_change = program_change | exchange;
                geometry_change = geometry_change | exchange;
            }

            // 描边脏, 如果字体是canvas绘制类型(不支持Stroke的宽度修改)， 仅修改stroke的uniform， 否则， 如果Stroke是添加或删除， 还要修改Stroke宏， 因此可能设置program_change脏
            if dirty & (StyleType::Stroke as usize) != 0 {
                program_change = program_change | modify_stroke(index.text, style_mark.local_style, &text_style.text.stroke, render_obj, engine, &notify, charblock.is_pixel, &class_ubo);
            }

            // 尝试修改字体， 如果字体类型修改（dyn_type）， 需要修改pipeline， （字体类型修改应该重新创建paramter， TODO）
            if dirty & (StyleType::FontFamily as usize) != 0 {
                modify_font(index.text, render_obj, charblock.is_pixel, &font_sheet, &notify, &self.default_sampler, &self.point_sampler);
                program_change = true;
            }

            // 文字内容脏， 这是顶点流脏
            if dirty& (StyleType::Text as usize) != 0 {
                geometry_change = true;
                shadow_geometry_change = true;
            }
            
            // // 如果渲染管线脏， 重新创建渲染管线
            if program_change {
                notify.modify_event(index.text, "program_dirty", 0);
                // modify_program(render_obj, charblock.is_pixel, engine, &self.canvas_default_stroke_color);
            }
            
            // 文字属性流改变， 重新生成geometry
            if geometry_change {
                render_obj.geometry = create_geo(dirty, charblock, &text_style.text.color, text, text_style, &font_sheet, class_ubo, &self.index_buffer, engine);
                render_objs.get_notify().modify_event(index.text, "geometry", 0);
            }

            // 矩阵脏 
            if dirty & (StyleType::Matrix as usize) != 0 {
                modify_matrix(
                    index.text,
                    create_let_top_offset_matrix(layout, world_matrix, transform, 0.0, 0.0, render_obj.depth),
                    render_obj,
                    &notify,
                );
            } 

            // 阴影存在
            if index.shadow > 0 {
                let shadow_render_obj = unsafe { render_objs.get_unchecked_mut(index.shadow) };
                if dirty & (StyleType::TextShadow as usize) != 0 { 
                              
                    // 阴影颜色脏，或描边脏， 修改ubo
                    modify_shadow_color(index.shadow, style_mark.local_style, text_style, &notify, shadow_render_obj, engine, charblock.is_pixel, &class_ubo);
                    // 设置ubo TODO

                    // 尝试修改字体， 如果字体类型修改（dyn_type）， 需要修改pipeline， （字体类型修改应该重新创建paramter， TODO）
                    if dirty & (StyleType::FontFamily as usize) != 0 {
                        modify_font(index.text, shadow_render_obj, charblock.is_pixel, &font_sheet, &notify, &self.default_sampler, &self.point_sampler);
                    }

                    // // 修改阴影的渲染管线
                    // if program_change {
                    //     modify_program(shadow_render_obj, charblock.is_pixel, engine, &self.canvas_default_stroke_color);
                    // }
                    if program_change {
                        notify.modify_event(index.shadow, "program_dirty", 0);
                        // modify_program(render_obj, charblock.is_pixel, engine, &self.canvas_default_stroke_color);
                    }

                    // 修改阴影的顶点流
                    if shadow_geometry_change  {
                        // 如果填充色是纯色， 阴影的geo和文字的geo一样， 否则重新创建阴影的geo
                        match &text_style.text.color {
                            Color::RGBA(_) => shadow_render_obj.geometry = render_obj.geometry.clone(),
                            Color::LinearGradient(_) => {
                                let color = text_style.shadow.color.clone();
                                shadow_render_obj.geometry = create_geo(dirty, charblock, &Color::RGBA(color), text, text_style, font_sheet, class_ubo, &self.index_buffer, engine)
                            },
                        }
                        notify.modify_event(index.shadow, "geometry", 0);
                    }

                }
                
                if dirty & (StyleType::Matrix as usize) != 0 {
                    modify_matrix(
                        index.shadow,
                        create_let_top_offset_matrix(layout, world_matrix, transform, text_style.shadow.h, text_style.shadow.v, shadow_render_obj.depth),
                        shadow_render_obj,
                        &notify,
                    );
                }
            }   
        }
        // println!("run---------------------------{:?}", std::time::Instant::now() - time);
    }
}

// 监听图片等待列表的改变， 将已加载完成的图片设置到对应的组件上
impl<'a, L: FlexNode + 'static, C: HalContext + 'static> SingleCaseListener<'a, FontSheet, ModifyEvent> for CharBlockSys<L, C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, _: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        self.texture_change = true;
    }
}

impl<'a, L: FlexNode + 'static, C: HalContext + 'static> MultiCaseListener<'a, Node, TextStyle, DeleteEvent> for CharBlockSys<L, C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, render_objs: Self::WriteData){
        self.remove_render_obj(event.id, render_objs)
    }
}

impl<L: FlexNode + 'static, C: HalContext + 'static> CharBlockSys<L, C> {
    
    #[inline]
    pub fn new(engine: &mut Engine<C>, texture_size: (usize, usize)) -> Self {
        let mut canvas_bs = BlendStateDesc::default();
        canvas_bs.set_rgb_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);

        let mut hasher = FxHasher32::default();
        canvas_bs.hash(&mut hasher);
        let hash = hasher.finish();
        let canvas_bs = match engine.res_mgr.get::<HalBlendState>(&hash) {
            Some(r) => r,
            None => engine.res_mgr.create(hash, create_bs(&engine.gl, canvas_bs)),
        };

        let mut msdf_bs = BlendStateDesc::default();
        msdf_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        let mut hasher = FxHasher32::default();
        msdf_bs.hash(&mut hasher);
        let hash = hasher.finish();
        let msdf_bs = match engine.res_mgr.get::<HalBlendState>(&hash) {
            Some(r) => r,
            None => engine.res_mgr.create(hash, create_bs(&engine.gl, msdf_bs)),
        };

        let default_sampler = SamplerDesc::default();
        let mut hasher = FxHasher32::default();
        default_sampler.hash(&mut hasher);
        let hash = hasher.finish();
        let default_sampler = match engine.res_mgr.get::<HalSampler>(&hash) {
            Some(r) => r,
            None => engine.res_mgr.create(hash, create_sampler(&engine.gl, default_sampler)),
        };

        let mut point_sampler = SamplerDesc::default();
        point_sampler.min_filter = TextureFilterMode::Nearest;
        point_sampler.mag_filter = TextureFilterMode::Nearest;
        let mut hasher = FxHasher32::default();
        point_sampler.hash(&mut hasher);
        let hash = hasher.finish();
        let point_sampler = match engine.res_mgr.get::<HalSampler>(&hash) {
            Some(r) => r,
            None => engine.res_mgr.create(hash, create_sampler(&engine.gl, point_sampler)),
        };

        let default_color_ubo = create_hash_res(engine, UColorUbo::new(UniformValue::Float4(0.0, 0.0, 0.0, 1.0)));

        let mut index_data = Vec::with_capacity(600);
        let mut i = 0;
        while i < 400 {
            index_data.extend_from_slice(&[i, i + 1, i + 2, i, i + 2, i + 3]);
            i += 4;
        }

        Self {
            render_map: VecMap::default(),
            canvas_bs: canvas_bs,
            msdf_bs: msdf_bs,
            default_sampler: default_sampler,
            point_sampler: point_sampler,
            canvas_default_stroke_color: Share::new(CanvasTextStrokeColorUbo::new(UniformValue::Float4(0.0, 0.0, 0.0, 0.0))),
            class_ubos: VecMap::default(),
            default_ubos: RenderCatch {
                fill_color_ubo: default_color_ubo.clone(),
                shadow_color_ubo: default_color_ubo,
                stroke_ubo: create_hash_res(engine, MsdfStrokeUbo::new(UniformValue::Float1(0.0), UniformValue::Float4(0.0, 0.0, 0.0, 0.0))),
                stroke_color_ubo: create_hash_res(engine, CanvasTextStrokeColorUbo::new(UniformValue::Float4(0.0, 0.0, 0.0, 0.0))),
                layout_hash: 0,
            },
            index_buffer: Share::new(create_buffer(&engine.gl, BufferType::Indices, 600, Some(BufferData::Short(index_data.as_slice())), false)),
            index_len: 100,
            texture_size_ubo: Share::new(TextTextureSize::new(UniformValue::Float2(texture_size.0 as f32, texture_size.1 as f32))),
            texture_change: false,
            mark: PhantomData,
        }
    }

    #[inline]
    fn create_render_obj(
        &mut self,
        id: usize,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &DefaultState,
        have_shadow: bool,
        is_pixel: bool
    ) -> I {
        let shadow_index = if !have_shadow{
            0
        }else {
            self.create_render_obj1(id, render_objs, default_state, is_pixel)
        };
        let index = self.create_render_obj1(id, render_objs, default_state, is_pixel);

        // 创建RenderObj与Node实体的索引关系， 并设脏
        self.render_map.insert(id, I{text: index, shadow: shadow_index});
        unsafe { self.render_map.get_unchecked_mut(id).clone() }
    }

    fn create_render_obj1(
        &self,
        id: usize,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &DefaultState,
        is_pixel: bool,
    ) -> usize {
        let (vs_name, fs_name, paramter, bs) = if is_pixel {
            let paramter: Share<dyn ProgramParamter> = Share::new(CanvasTextParamter::default());
            paramter.set_value("strokeColor", self.canvas_default_stroke_color.clone());
            paramter.set_value("textureSize", self.texture_size_ubo.clone());
            (CANVAS_TEXT_VS_SHADER_NAME.clone(), CANVAS_TEXT_FS_SHADER_NAME.clone(), paramter, self.canvas_bs.clone())
        } else {
            let paramter: Share<dyn ProgramParamter> = Share::new(MsdfParamter::default());
            (TEXT_VS_SHADER_NAME.clone(), TEXT_FS_SHADER_NAME.clone(), paramter, self.msdf_bs.clone())
        };
        let state = State {
            bs: bs.clone(),
            rs: default_state.df_rs.clone(),
            ss: default_state.df_ss.clone(),
            ds: default_state.tarns_ds.clone(),
        };
        let render_obj = new_render_obj(id, 0.1, false, vs_name, fs_name, paramter, state);
        render_obj.paramter.set_value("textureSize", self.texture_size_ubo.clone());
        let notify = render_objs.get_notify();
        render_objs.insert(render_obj, Some(notify))
    }

    #[inline]
    fn remove_render_obj(&mut self, id: usize, render_objs: &mut SingleCaseImpl<RenderObjs>) {
        match self.render_map.remove(id) {
            Some(index) => {
                let notify = render_objs.get_notify();
                render_objs.remove(index.text, Some(notify.clone()));
                render_objs.remove(index.shadow, Some(notify));
            },
            None => ()
        };
    }
}

#[inline]
fn modify_stroke<C: HalContext + 'static>(
    index: usize,
    local_style: usize,
    text_stroke: &Stroke,
    render_obj: &mut RenderObj,
    engine: &mut SingleCaseImpl<Engine<C>>,
    notify: &NotifyImpl,
    is_pixel: bool,
    class_ubo: &RenderCatch,
) -> bool {
    notify.modify_event(index, "", 0);
    // canvas 字体
    if is_pixel {
        // let ubo = if local_style & (StyleType::Stroke as usize) == 0 {
        //     class_ubo.stroke_color_ubo.clone()
        // } else {
        //     let color = &text_stroke.color;
        //     create_hash_res(engine, CanvasTextStrokeColorUbo::new(UniformValue::Float4(color.r, color.g, color.b, color.a)))
        // };
        let color = &text_stroke.color;
        let ubo = create_hash_res(engine, CanvasTextStrokeColorUbo::new(UniformValue::Float4(color.r, color.g, color.b, color.a)));
        render_obj.paramter.set_value("strokeColor", ubo);
        return false;
    }

    // msdf字体
    if text_stroke.width == 0.0 {
        //删除描边的宏
        match render_obj.vs_defines.remove("STROKE") {
            Some(_) => true,
            None => false,
        } 
    } else { 
        let ubo = if local_style & (StyleType::Stroke as usize) == 0 {
            class_ubo.stroke_ubo.clone()
        } else {
            let color = &text_stroke.color;
            create_hash_res(engine, MsdfStrokeUbo::new(UniformValue::Float1(text_stroke.width/10.0), UniformValue::Float4(color.r, color.g, color.b, color.a)) )
        };
        render_obj.paramter.set_value("stroke", ubo);
        match render_obj.vs_defines.add("STROKE") {
            Some(_) => false,
            None => true,
        }
    }
}

#[inline]
fn modify_color<C: HalContext + 'static>(
    index: usize,
    _local_style: usize,
    color: &Color,
    engine: &mut Engine<C>,
    notify: &NotifyImpl,
    render_obj: &mut RenderObj,
    _class_ubo: &RenderCatch,
) -> bool {
    let change = match color {
        Color::RGBA(c) => {
            // // 如果是class样式中的颜色， 直接使用class_ubo.fill_color_ubo
            // let ubo = if local_style & (StyleType::Color as usize) == 0 {
            //     class_ubo.fill_color_ubo.clone()
            // } else {
            //     create_hash_res(engine, UColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a)))
            // };
            let ubo = create_hash_res(engine, UColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a)));
            render_obj.paramter.set_value("uColor", ubo );
            notify.modify_event(index, "", 0);

            // 如果未找到UCOLOR宏， 表示修改之前的颜色不为RGBA， 应该删除VERTEX_COLOR宏， 添加UCOLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
            to_ucolor_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut())
        },
        Color::LinearGradient(_) => {
            // 如果未找到VERTEX_COLOR宏， 表示修改之前的颜色不为LinearGradient， 应该删除UCOLOR宏， 添加VERTEX_COLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
            to_vex_color_defines(render_obj.vs_defines.as_mut(), render_obj.fs_defines.as_mut())
        },
    };
    change
}

#[inline]
fn modify_font (
    index: usize,
    render_obj: &mut RenderObj,
    is_pixel: bool,
    font_sheet: &FontSheet,
    notify: &NotifyImpl,
    default_sampler: &Share<SamplerRes>,
    point_sampler: &Share<SamplerRes>, // 点采样sampler
) {
    notify.modify_event(index, "ubo", 0);
    // 如果是canvas 字体绘制类型， 并且绘制fontsize 与字体本身fontsize一致， 应该使用点采样
    let sampler = if is_pixel  {
        point_sampler
    } else {
        default_sampler
    };
    render_obj.paramter.set_texture("texture", ( &font_sheet.get_font_tex().bind , &sampler));
}

#[inline]
fn modify_shadow_color<C: HalContext + 'static>(
    index: usize,
    _local_style: usize,
    text_style: &TextStyle,
    notify: &NotifyImpl,
    render_obj: &mut RenderObj,
    engine: &mut Engine<C>,
    is_pixel: bool,
    _class_ubo: &RenderCatch,
) {
    let c = &text_style.shadow.color;
    if text_style.text.stroke.width > 0.0 && is_pixel {
        render_obj.paramter.set_value("strokeColor", create_hash_res(engine,  CanvasTextStrokeColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a)) ));
    }
    // let ubo = if local_style & (StyleType::TextShadow as usize) == 0 {
    //     class_ubo.shadow_color_ubo.clone()
    // } else {
    //     create_hash_res(engine,  UColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a)) )
    // };
    let ubo  = create_hash_res(engine,  UColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a)) );
    render_obj.paramter.set_value("uColor", ubo);
    render_obj.fs_defines.add("UCOLOR");
    notify.modify_event(index, "ubo", 0);
}

#[inline]
fn set_canvas_default_stroke(render_obj: &RenderObj, canvas_default_stroke_color: &Share<CanvasTextStrokeColorUbo>) {
    match render_obj.paramter.get_value("strokeColor") {
        Some(_) => (),
        None => { render_obj.paramter.set_value("strokeColor", canvas_default_stroke_color.clone()); },
    };
}

fn modify_program<C: HalContext + 'static>(render_obj: &mut RenderObj, is_pixel: bool, engine: &mut Engine<C>, canvas_default_stroke_color: &Share<CanvasTextStrokeColorUbo>) {
    render_obj.program = if !is_pixel {
        Some(engine.create_program(
            TEXT_VS_SHADER_NAME.get_hash(),
            TEXT_FS_SHADER_NAME.get_hash(),
            TEXT_VS_SHADER_NAME.as_ref(),
            &*render_obj.vs_defines,
            TEXT_FS_SHADER_NAME.as_ref(),
            &*render_obj.fs_defines,
            render_obj.paramter.as_ref(),
        ))
    }else {
        render_obj.vs_defines.remove("STROKE");
        set_canvas_default_stroke(render_obj, canvas_default_stroke_color);
        Some(engine.create_program(
            CANVAS_TEXT_VS_SHADER_NAME.get_hash(),
            CANVAS_TEXT_FS_SHADER_NAME.get_hash(),
            CANVAS_TEXT_VS_SHADER_NAME.as_ref(),
            &*render_obj.vs_defines,
            CANVAS_TEXT_FS_SHADER_NAME.as_ref(),
            &*render_obj.fs_defines,
            render_obj.paramter.as_ref(),
        ))
    };
}

// 返回position， uv， color， index
#[inline]
fn create_geo<L: FlexNode + 'static, C: HalContext + 'static>(
    dirty: usize,
    char_block: &CharBlock<L>,
    color: &Color,
    text: &TextContent,
    text_style: &TextStyle,
    font_sheet: &FontSheet,
    share_data: &RenderCatch,
    share_index_buffer: &Share<HalBuffer>,
    engine: &mut Engine<C>,
) -> Option<Share<GeometryRes>> {
    // 是共享文字
    if text.0 == String::new() {
        let mut hasher = FxHasher32::default();
        text.1.hash(&mut hasher);
        // 对于布局信息， 如果没有在style中设置， 可以直接使用class中的布局hash
        if dirty & TEXT_LAYOUT_DIRTY == 0 {
            share_data.layout_hash.hash(&mut hasher); 
        }else {
            //在style中使用了文字布局属性, 重新计算文字布局属性的hash
            let layout_hash = text_layout_hash(&text_style.text, &text_style.font);
            layout_hash.hash(&mut hasher); 
        }

        // 如果是渐变色， 计算渐变色的hash
        if let Color::LinearGradient(ref c) = color {
            c.hash(&mut hasher); 
        }

        let hash = hasher.finish();
        // 从缓存中找到geo， 直接返回
        if let Some(geo) = engine.res_mgr.get::<GeometryRes>(&hash) {
            return Some(geo);
        }

        // 缓存中不存在 对应的geo， 创建geo并缓存
        get_geo_flow(char_block, color, font_sheet, engine, Some(hash), share_index_buffer)
    } else {
        // 如果文字不共享， 重新创建geo， 并且不缓存geo
        get_geo_flow(char_block, color, font_sheet, engine, None, share_index_buffer)
    }
}


fn text_layout_hash(text_style: &Text, font: &Font) -> u64 {
    let mut hasher = FxHasher32::default();
    let hasher = &mut hasher;
    unsafe { NotNan::unchecked_new(text_style.letter_spacing).hash(hasher) };
    unsafe { NotNan::unchecked_new(text_style.indent).hash(hasher) };
    unsafe { NotNan::unchecked_new(text_style.word_spacing).hash(hasher) };
    // TODO
    // match text_style.line_height {
    //     LineHeight::Normal => 0.hash(hasher),
    //     LineHeight::Length(r) => {
    //         1.hash(hasher);
    //         unsafe { NotNan::unchecked_new(r).hash(hasher) };
    //     },
    //     LineHeight::Number(r) => {
    //         2.hash(hasher);
    //         unsafe { NotNan::unchecked_new(r).hash(hasher) };
    //     },
    //     LineHeight::Percent(r) => {
    //         3.hash(hasher);
    //         unsafe { NotNan::unchecked_new(r).hash(hasher) };
    //     },
    // };
    text_style.text_align.hash(hasher);
    text_style.white_space.hash(hasher);
    text_style.vertical_align.hash(hasher);
    font.weight.hash(hasher);
    match font.size {
        FontSize::None => 0.hash(hasher),
        FontSize::Length(r) => {
            1.hash(hasher);
            unsafe { NotNan::unchecked_new(r).hash(hasher) };
        },
        FontSize::Percent(r) => {
            3.hash(hasher);
            unsafe { NotNan::unchecked_new(r).hash(hasher) };
        },
    };
    font.style.hash(hasher);
    font.family.hash(hasher);
    hasher.finish()
}

// 返回position， uv， color， index
#[inline]
fn get_geo_flow<L: FlexNode + 'static, C: HalContext + 'static>(
    char_block: &CharBlock<L>,
    color: &Color,
    font_sheet: &FontSheet,
    engine: &mut Engine<C>,
    hash: Option<u64>,
    index_buffer: &Share<HalBuffer>,
) -> Option<Share<GeometryRes>> {
    let len = char_block.chars.len();
    let mut positions: Vec<f32> = Vec::with_capacity(8 * len);
    let mut uvs: Vec<f32> = Vec::with_capacity(8 * len);
    // let font_height = char_block.font_height;
    let mut i = 0;
    let offset = (char_block.pos.x, char_block.pos.y);

    let geo = create_geometry(&engine.gl);
    let mut geo_res = GeometryRes{geo: geo, buffers: Vec::with_capacity(3)};

    if len > 0 {
        match color {
            Color::RGBA(_) => {
                for c in char_block.chars.iter() {
                    if c.ch <= ' ' {
                        continue;
                    }
                    let glyph = match font_sheet.get_glyph(c.ch_id_or_count) {
                        Some(r) => r.1.clone(),
                        None => continue,
                    };
                    push_pos_uv(&mut positions, &mut uvs, &c.pos, &offset, &glyph, c.width, char_block.font_height); 
                }
                engine.gl.geometry_set_indices_short_with_offset(&geo_res.geo, index_buffer, 0, positions.len()/8 * 6).unwrap();
            },
            Color::LinearGradient(color) => {
                let mut colors = vec![Vec::new()];
                let mut indices = Vec::with_capacity(6 * len);
                let (start, end) = cal_all_size(char_block, font_sheet); // 渐变范围
                //渐变端点
                let endp = find_lg_endp(&[
                    start.x, start.y + offset.1,
                    start.x, end.y +  offset.1 ,
                    end.x, end.y + offset.1,
                    end.x, start.y + offset.1,
                ], color.direction);

                let mut lg_pos = Vec::with_capacity(color.list.len());
                let mut lg_color = Vec::with_capacity(color.list.len() * 4);
                for v in color.list.iter() {
                    lg_pos.push(v.position);
                    lg_color.extend_from_slice(&[v.rgba.r, v.rgba.g, v.rgba.b, v.rgba.a]);
                }
                let lg_color = vec![LgCfg{unit:4, data: lg_color}];

                for c in char_block.chars.iter() {
                    if c.ch <= ' ' {
                        continue;
                    }
                    let glyph = match font_sheet.get_glyph(c.ch_id_or_count) {
                        Some(r) => r.1.clone(),
                        None => continue,
                    };

                    push_pos_uv(&mut positions, &mut uvs, &c.pos, &offset, &glyph, c.width, char_block.font_height);
                    
                    let (ps, indices_arr) = split_by_lg(
                        positions,
                        vec![i, i + 1, i + 2, i + 3],
                        lg_pos.as_slice(),
                        endp.0.clone(),
                        endp.1.clone(),
                    );
                    positions = ps;
                    
                    // 尝试为新增的点计算uv
                    fill_uv(&mut positions, &mut uvs, i as usize);

                    // 颜色插值
                    colors = interp_mult_by_lg(
                        positions.as_slice(),
                        &indices_arr,
                        colors,
                        lg_color.clone(),
                        lg_pos.as_slice(),
                        endp.0.clone(),
                        endp.1.clone(),
                    );

                    indices = mult_to_triangle(&indices_arr, indices);
                    i = positions.len() as u16 / 2;
                }

                let colors = colors.pop().unwrap();
                let color_buffer = create_buffer(&engine.gl, BufferType::Attribute, colors.len(), Some(BufferData::Float(&colors)), false);
                let i_buffer = create_buffer(&engine.gl, BufferType::Indices, indices.len(), Some(BufferData::Short(&indices)), false);
                engine.gl.geometry_set_attribute(&geo_res.geo, &AttributeName::Color, &color_buffer, 4).unwrap();
                engine.gl.geometry_set_indices_short(&geo_res.geo, &index_buffer).unwrap();
                geo_res.buffers.push(Share::new(i_buffer));
                geo_res.buffers.push(Share::new(color_buffer));
            }
        }

        let position_buffer = create_buffer(&engine.gl, BufferType::Attribute, positions.len(), Some(BufferData::Float(&positions)), false);
        let uv_buffer = create_buffer(&engine.gl, BufferType::Attribute, uvs.len(), Some(BufferData::Float(&uvs)), false);
        engine.gl.geometry_set_attribute(&geo_res.geo, &AttributeName::Position, &position_buffer, 2).unwrap();
        engine.gl.geometry_set_attribute(&geo_res.geo, &AttributeName::UV0, &uv_buffer, 2).unwrap();
        geo_res.buffers.push(Share::new(uv_buffer));
        geo_res.buffers.push(Share::new(position_buffer));
        
        Some(match hash {
            Some(hash) => engine.res_mgr.create::<GeometryRes>(hash, geo_res),
            None => Share::new(geo_res),
        })
    } else {
        None
    }
}


#[inline]
fn cal_all_size<L: FlexNode + 'static>(char_block: &CharBlock<L>, font_sheet: &FontSheet) -> (Point2, Point2) {
    let mut start = Point2::new(0.0, 0.0);
    let mut end = Point2::new(0.0, 0.0);
    let mut j = 0;
    for i in 0..char_block.chars.len() {
        let c = &char_block.chars[i]; 
        let pos = &c.pos;
        if c.ch <= ' ' {
            continue;
        }
        let glyph = match font_sheet.get_glyph(c.ch_id_or_count) {
            Some(r) => r.1.clone(),
            None => continue,
        };
        start = Point2::new(pos.x + glyph.ox, pos.y + glyph.oy);
        end = Point2::new(start.x + c.width, start.y + char_block.font_height);
        j += 1;
        break;
    }
    for i in j..char_block.chars.len() {
        let ch = &char_block.chars[i]; 
        let pos = &ch.pos;
        // let glyph = match font_sheet.get_glyph(char_block.chars[i].ch_id_or_count) {
        //     Some(r) => r.1.clone(),
        //     None => continue,
        // };

        if pos.x < start.x{
            start.x = pos.x;
        }
        let end_x = pos.x + ch.width;
        if end_x > end.x {
            end.x = end_x;
        } 
        if pos.y < start.y{
            start.y = pos.y;
        }
        let end_y = char_block.font_height;
        if end_y > end.y {
            end.y = end_y;
        } 
    }
    (start, end)
}

#[inline]
fn fill_uv(positions: &mut Vec<f32>, uvs: &mut Vec<f32>, i: usize){
    let pi = i * 2;
    let uvi = i * 2;
    let len = positions.len() - pi;
    let (p1, p4) = (
        (
            positions[pi],
            positions[pi + 1],
        ),
        (
            positions[pi + 4],
            positions[pi + 5],
        ),
    );
    let (u1, u4) = (
        (
            uvs[uvi],
            uvs[uvi + 1]
        ),
        (
            uvs[uvi + 4],
            uvs[uvi + 5]
        ),
    );
    if len > 8 {
        let mut i = pi + 8;
        for _j in 0..(len - 8)/2 {
            let pos_x = positions[i];
            let pos_y = positions[i + 1];
            let uv;
            if (pos_x - p1.0).abs() < 0.001{
                let base = p4.1 - p1.1;
                let ratio = if base == 0.0 {
                    0.0
                } else {
                    (pos_y - p1.1)/(p4.1 - p1.1)
                };
                uv = (u1.0, u1.1 * (1.0 - ratio) + u4.1 * ratio );
            }else if (pos_x - p4.0).abs() < 0.001{
                let base = p4.1 - p1.1;
                let ratio = if base == 0.0 {
                    0.0
                } else {
                    (pos_y - p1.1)/(p4.1 - p1.1)
                };
                uv = (u4.0, u1.1  * (1.0 - ratio) + u4.1 * ratio );
            }else if (pos_y - p1.1).abs() < 0.001 {
                let base = p4.0 - p1.0;
                let ratio = if base == 0.0 {
                    0.0
                } else {
                    (pos_x - p1.0)/(p4.0 - p1.0)
                };
                uv = (u1.0 * (1.0 - ratio) + u4.0 * ratio, u1.1 );
            }else {
            // }else if pos_y == p4.1{
                let base = p4.0 - p1.0;
                let ratio = if base == 0.0 {
                    0.0
                } else {
                    (pos_x - p1.0)/(p4.0 - p1.0)
                };
                uv = (u1.0 * (1.0 - ratio) + u4.0 * ratio , u4.1 );
            }
            uvs.push(uv.0);
            uvs.push(uv.1);
            i += 2;
        }
    }
}

fn push_pos_uv(positions: &mut Vec<f32>, uvs: &mut Vec<f32>, pos: &Point2 , offset: &(f32, f32), glyph: &Glyph, width: f32, font_height: f32){
    let left_top = (pos.x + offset.0 + glyph.ox, pos.y + offset.1 + glyph.oy);
    let right_bootom = (left_top.0 + width, left_top.1 + font_height);
    let ps = [
        left_top.0,     left_top.1,    
        left_top.0,     right_bootom.1,
        right_bootom.0, right_bootom.1,
        right_bootom.0, left_top.1,    
    ];
    uvs.extend_from_slice(&[
        glyph.x, glyph.y,
        glyph.x, glyph.y + glyph.height,
        glyph.x + glyph.width, glyph.y + glyph.height,
        glyph.x + glyph.width, glyph.y,
    ]);
    positions.extend_from_slice(&ps[..]);
}

impl_system!{
    CharBlockSys<L, C> where [L: FlexNode + 'static, C: HalContext + 'static],
    true,
    {
        SingleCaseListener<FontSheet, ModifyEvent>
        MultiCaseListener<Node, TextStyle, DeleteEvent>
    }
}