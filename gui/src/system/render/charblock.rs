/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use share::Share;


use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
use ecs::monitor::NotifyImpl;
use map::{ vecmap::VecMap } ;
use hal_core::*;
use atom::Atom;
use polygon::{mult_to_triangle, interp_mult_by_lg, split_by_lg, LgCfg, find_lg_endp};

use component::user::*;
use single::*;
use component::calc::{ZDepth, CharBlock, WorldMatrixRender, DirtyType};
use entity::{Node};
use render::engine::{ Engine , PipelineInfo};
use render::res::{ SamplerRes};
use render::res::GeometryRes;
use system::util::*;
use system::util::constant::*;
use system::render::shaders::text::{TEXT_FS_SHADER_NAME, TEXT_VS_SHADER_NAME};
use system::render::shaders::canvas_text::{CANVAS_TEXT_VS_SHADER_NAME, CANVAS_TEXT_FS_SHADER_NAME};
use font::font_sheet::FontSheet;
use font::sdf_font:: {GlyphInfo, SdfFont };
use util::res_mgr::Res;
use layout::FlexNode;
use FxHashMap32;


lazy_static! {
    static ref STROKE: Atom = Atom::from("STROKE");
    static ref UCOLOR: Atom = Atom::from("UCOLOR");
    static ref VERTEX_COLOR: Atom = Atom::from("VERTEX_COLOR");

    static ref STROKE_SIZE: Atom = Atom::from("strokeSize");
    static ref STROKE_COLOR: Atom = Atom::from("strokeColor");
    static ref U_COLOR: Atom = Atom::from("uColor");
}

pub struct CharBlockSys<C: Context + 'static, L: FlexNode + 'static>{
    render_map: VecMap<usize>,
    shadow_render_map: VecMap<usize>,
    dirtys: Vec<usize>,
    mark: PhantomData<(C, L)>,
    rs: Share<RasterState>,
    bs: Share<BlendState>,
    ss: Share<StencilState>,
    ds: Share<DepthState>,
    canvas_bs: Share<BlendState>,
    pipeline: Option<Share<PipelineInfo>>,
    default_sampler: Option<Res<SamplerRes<C>>>,
    point_sampler: Option<Res<SamplerRes<C>>>,
}

impl<C: Context + 'static, L: FlexNode + 'static> CharBlockSys<C, L> {
    pub fn new() -> Self{
        let mut bs = BlendState::new();
        let mut ds = DepthState::new();
        let mut canvas_bs = BlendState::new();
        bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        canvas_bs.set_rgb_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);
        ds.set_write_enable(false);
        CharBlockSys {
            render_map: VecMap::default(),
            shadow_render_map: VecMap::default(),
            dirtys: Vec::new(),
            mark: PhantomData,
            rs: Share::new(RasterState::new()),
            bs: Share::new(bs),
            ss: Share::new(StencilState::new()),
            ds: Share::new(ds),
            canvas_bs:  Share::new(canvas_bs),
            pipeline: None,
            default_sampler: None,
            point_sampler: None,
        }
    }
    pub fn create_render(&mut self, context: usize, z_depth: f32, engine: &mut Engine<C>, notify: &NotifyImpl, render_objs: &mut SingleCaseImpl<RenderObjs<C>>, world_matrix: &MultiCaseImpl<Node, WorldMatrixRender>) -> usize {
        let mut ubos: FxHashMap32<Atom, Share<Uniforms<C>>> = FxHashMap32::default();
        let common_ubo = engine.gl.create_uniforms();
        ubos.insert(COMMON.clone(), Share::new(common_ubo)); // COMMON

        let render_obj: RenderObj<C> = RenderObj {
            depth: z_depth + 0.2,
            depth_diff: 0.2,
            visibility: false,
            is_opacity: false,
            ubos: ubos,
            geometry: None,
            pipeline: self.pipeline.as_ref().unwrap().clone(),
            context: context,
            defines: Vec::new(),
        };   
        
        let index = render_objs.insert(render_obj, Some(notify.clone()));
        self.render_map.insert(context, index);
        self.modify_matrix(context, world_matrix, render_objs);
        index
    }

    pub fn create_shadow_render(&mut self, context: usize, z_depth: f32, engine: &mut Engine<C>, notify: &NotifyImpl, render_objs: &mut SingleCaseImpl<RenderObjs<C>>) -> usize {
        let mut ubos: FxHashMap32<Atom, Share<Uniforms<C>>> = FxHashMap32::default();
        let ucolor_ubo = engine.gl.create_uniforms();
        ubos.insert(UCOLOR.clone(), Share::new(ucolor_ubo)); // UCOLOR

        let common_ubo = engine.gl.create_uniforms();
        ubos.insert(COMMON.clone(), Share::new(common_ubo)); // COMMON

        let render_obj: RenderObj<C> = RenderObj {
            depth: z_depth + 0.1,
            depth_diff: 0.1,
            visibility: false,
            is_opacity: false,
            ubos: ubos,
            geometry: None,
            pipeline: self.pipeline.as_ref().unwrap().clone(),
            context: context,
            defines: vec![UCOLOR.clone()],
        };   
        
        let index = render_objs.insert(render_obj, Some(notify.clone()));
        self.shadow_render_map.insert(context, index);
        index
    }

    fn modify_matrix(
        &self,
        id: usize,
        world_matrixs: &MultiCaseImpl<Node, WorldMatrixRender>,
        render_objs: &mut SingleCaseImpl<RenderObjs<C>>
    ){
        if let Some(index) = self.render_map.get(id) {
            let world_matrix = unsafe { world_matrixs.get_unchecked(id) };
            let render_obj = unsafe { render_objs.get_unchecked_mut(*index) };

            // 渲染物件的顶点不是一个四边形， 保持其原有的矩阵
            let ubos = &mut render_obj.ubos;
            let slice: &[f32; 16] = world_matrix.0.as_ref();
            Share::make_mut(ubos.get_mut(&WORLD).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
            debug_println!("charblock, id: {}, world_matrix: {:?}", render_obj.context, &slice[0..16]);
            render_objs.get_notify().modify_event(*index, "ubos", 0);
        }
    }

    fn modify_shadow_matrix(
        &self,
        id: usize,
        world_matrixs: &MultiCaseImpl<Node, WorldMatrixRender>,
        render_objs: &mut SingleCaseImpl<RenderObjs<C>>,
        h: f32,
        v: f32,
    ){
        if let Some(index) = self.shadow_render_map.get(id) {
            let world_matrix = unsafe { &(world_matrixs.get_unchecked(id).0) } * Matrix4::new (
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                h,   v,   0.0, 1.0
            );
            let render_obj = unsafe { render_objs.get_unchecked_mut(*index) };

            // 渲染物件的顶点不是一个四边形， 保持其原有的矩阵
            let ubos = &mut render_obj.ubos;
            let slice: &[f32; 16] = world_matrix.as_ref();
            Share::make_mut(ubos.get_mut(&WORLD).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
            render_objs.get_notify().modify_event(*index, "ubos", 0);
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: Context + 'static, L: FlexNode + 'static> Runner<'a> for CharBlockSys<C, L>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, WorldMatrixRender>,
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a SingleCaseImpl<DefaultTable>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>, &'a mut MultiCaseImpl<Node, CharBlock<L>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let (z_depths, world_matrix, font_sheet, _default_table) = read;
        let (render_objs, engine, charblocks) = write;
        let notify = render_objs.get_notify();
        for id in self.dirtys.iter() {
            let charblock = match charblocks.get_mut(*id) {
                Some(r) => r,
                None => return,
            };
            if charblock.modify == 0 {
                continue;
            }

            let map = &mut self.render_map;
            let z_depth = unsafe{z_depths.get_unchecked(*id)}.0;
            let index = match map.get(*id) {
                Some(r) => *r,
                None =>  unsafe {&mut *( self as *const Self as usize as *mut Self ) }.create_render(*id, z_depth, engine, &notify, render_objs, world_matrix),
            };

            let shadow_index = if charblock.clazz.shadow.color.a == 0.0 {
                self.shadow_render_map.remove(*id);
                0
            } else {
                match self.shadow_render_map.get(*id) {
                    Some(r) => *r,
                    None =>  unsafe {&mut *( self as *const Self as usize as *mut Self ) }.create_shadow_render(*id, z_depth, engine, &notify, render_objs),
                }
            };

            let first_font = match font_sheet.get_first_font(&charblock.clazz.font.family) {
                Some(r) => r,
                None => {
                    debug_println!("font is not exist: {}", charblock.clazz.font.family.as_str());
                    return;
                }
            };

            let font_size = font_sheet.get_size(&charblock.clazz.font.family, &charblock.clazz.font.size);

            let render_obj = unsafe { &mut *(render_objs.get_unchecked(index) as *const RenderObj<C> as usize as *mut RenderObj<C>) };
            let (mut pipeline_change, mut geometry_change) = (false, false);
            let mut shadow_geometry_change = false;

            // 颜色脏， 如果是渐变色和纯色切换， 需要修改宏， 且顶点需要重新计算， 因此标记pipeline_change 和 geometry_change脏
            if charblock.modify & (DirtyType::Color as usize) != 0 {
                let exchange = modify_color(index, &charblock.clazz.style.color, engine, &notify, render_obj);
                pipeline_change = pipeline_change | exchange;
                geometry_change = geometry_change | exchange;
            }

            // 描边脏, 如果字体是canvas绘制类型(不支持Stroke的宽度修改)， 仅修改stroke的uniform， 否则， 如果Stroke是添加或删除， 还要修改Stroke宏， 因此可能设置pipeline_change脏
            if charblock.modify & (DirtyType::Stroke as usize) != 0 {
                pipeline_change = pipeline_change | modify_stroke(index, &charblock.clazz.style.stroke, render_obj, engine, &notify, &first_font);
            }

            // 阴影颜色脏， 修改ubo
            if charblock.modify & (DirtyType::ShadowColor as usize) != 0 && shadow_index > 0  {
                let shadow_render_obj = unsafe { render_objs.get_unchecked_mut(shadow_index) };
                modify_shadow_color(shadow_index, &charblock.clazz.shadow.color, &notify, shadow_render_obj, &first_font);         
            };

            // 阴影blur脏， 修改blur ubo
            if charblock.modify & (DirtyType::ShadowBlur as usize) != 0 {
                // 设置ubo
            }

            // 阴影 hv脏， 从新计算世界矩阵
            if charblock.modify & (DirtyType::ShadowHV as usize) != 0 {
                self.modify_shadow_matrix(*id, world_matrix, render_objs, charblock.clazz.shadow.h, charblock.clazz.shadow.v);
            }

            // 尝试修改字体， 如果发现famly 修改， 需要修改pipeline， geometry_change
            let font_change = try_modify_font(charblock.modify, index, render_obj, &first_font, font_size, &notify, self.default_sampler.as_ref().unwrap(), self.point_sampler.as_ref().unwrap());
            pipeline_change = pipeline_change | font_change;
            geometry_change = geometry_change | font_change;

            // 文字内容脏， 这是顶点流脏
            if charblock.modify & (DirtyType::Text as usize) != 0 {
                geometry_change = true;
                shadow_geometry_change = true;
            }
            
            // 如果渲染管线脏， 重新创建渲染管线
            if pipeline_change {
                let old_pipeline = render_obj.pipeline.clone();
                let pipeline = if first_font.get_dyn_type() == 0 {
                    engine.create_pipeline(
                        1,
                        &TEXT_VS_SHADER_NAME.clone(),
                        &TEXT_FS_SHADER_NAME.clone(),
                        render_obj.defines.as_slice(),
                        old_pipeline.rs.clone(),
                        self.bs.clone(),
                        old_pipeline.ss.clone(),
                        old_pipeline.ds.clone()
                    )
                }else {
                    let mut common_ubo = render_obj.ubos.get_mut(&COMMON).unwrap();
                    Share::make_mut(&mut common_ubo).set_float_4(&STROKE_COLOR, 0.0, 0.0, 0.0, 0.0);
                    engine.create_pipeline(
                        3,
                        &CANVAS_TEXT_VS_SHADER_NAME.clone(),
                        &CANVAS_TEXT_FS_SHADER_NAME.clone(),
                        render_obj.defines.as_slice(),
                        old_pipeline.rs.clone(),
                        self.canvas_bs.clone(),
                        old_pipeline.ss.clone(),
                        old_pipeline.ds.clone(),
                    )
                };
                render_obj.pipeline = pipeline;
            }
            
            // 文字顶点流改变， 重新生成顶点流
            if geometry_change {
                let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;        
                let (positions, uvs, colors, indices) = get_geo_flow(charblock, &first_font, &charblock.clazz.style.color, z_depth + 0.2, (0.0, 0.0));
                if positions.len() == 0 {
                    render_obj.geometry = None;
                } else {
                    let mut geometry = create_geometry(&mut engine.gl);
                    geometry.set_vertex_count((positions.len()/3) as u32);
                    geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false).unwrap();
                    geometry.set_attribute(&AttributeName::UV0, 2, Some(uvs.as_slice()), false).unwrap();
                    geometry.set_indices_short(indices.as_slice(), false).unwrap();
                    match colors {
                        Some(color) => {geometry.set_attribute(&AttributeName::Color, 4, Some(color.as_slice()), false).unwrap();},
                        None => ()
                    };
                    render_obj.geometry = Some(Res::new(500, Share::new(GeometryRes{name: 0, bind: geometry})));
                };
                render_objs.get_notify().modify_event(index, "geometry", 0);
            }

            if shadow_index > 0 {
                let shadow_render_obj = unsafe { render_objs.get_unchecked_mut(shadow_index) };

                // 修改阴影的顶点流
                if shadow_geometry_change  {
                    match &charblock.clazz.style.color {
                        Color::RGBA(_) => shadow_render_obj.geometry = render_obj.geometry.clone(),
                        Color::LinearGradient(_) => {
                            let (positions, uvs, indices) = get_shadow_geo_flow(charblock, &first_font, z_depth + 0.2);
                            if positions.len() == 0 {
                                shadow_render_obj.geometry = None;
                            } else {
                                let mut geometry = create_geometry(&mut engine.gl);
                                geometry.set_vertex_count((positions.len()/3) as u32);
                                geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false).unwrap();
                                geometry.set_attribute(&AttributeName::UV0, 2, Some(uvs.as_slice()), false).unwrap();
                                geometry.set_indices_short(indices.as_slice(), false).unwrap();
                                shadow_render_obj.geometry = Some(Res::new(500, Share::new(GeometryRes{name: 0, bind: geometry})));
                            };
                            notify.modify_event(shadow_index, "geometry", 0);
                        },
                    }
                }

                // 修改阴影的渲染管线
                if font_change {
                    let old_pipeline = shadow_render_obj.pipeline.clone();
                    let pipeline = if first_font.get_dyn_type() == 0 {
                        engine.create_pipeline(
                            1,
                            &TEXT_VS_SHADER_NAME.clone(),
                            &TEXT_FS_SHADER_NAME.clone(),
                            shadow_render_obj.defines.as_slice(),
                            old_pipeline.rs.clone(),
                            self.bs.clone(),
                            old_pipeline.ss.clone(),
                            old_pipeline.ds.clone()
                        )
                    }else {
                        engine.create_pipeline(
                            3,
                            &CANVAS_TEXT_VS_SHADER_NAME.clone(),
                            &CANVAS_TEXT_FS_SHADER_NAME.clone(),
                            shadow_render_obj.defines.as_slice(),
                            old_pipeline.rs.clone(),
                            self.canvas_bs.clone(),
                            old_pipeline.ss.clone(),
                            old_pipeline.ds.clone(),
                        )
                    };
                    shadow_render_obj.pipeline = pipeline;
                    notify.modify_event(shadow_index, "pipeline", 0);
                }
            }
            

            charblock.modify = 0;
        }
        self.dirtys.clear();
    }

    fn setup(&mut self, _: Self::ReadData, write: Self::WriteData){
        let (_, engine, _) = write;
        let s = SamplerDesc::default();
        let hash = sampler_desc_hash(&s);
        match engine.res_mgr.get::<SamplerRes<C>>(&hash) {
            Some(r) => self.default_sampler = Some(r.clone()),
            None => {
                let res = SamplerRes::new(hash, engine.gl.create_sampler(Share::new(s)).unwrap());
                self.default_sampler = Some(engine.res_mgr.create::<SamplerRes<C>>(res));
            }
        }

        let mut s = SamplerDesc::default();
        s.min_filter = TextureFilterMode::Nearest;
        s.mag_filter = TextureFilterMode::Nearest;
        let hash = sampler_desc_hash(&s);
        match engine.res_mgr.get::<SamplerRes<C>>(&hash) {
            Some(r) => self.default_sampler = Some(r.clone()),
            None => {
                let res = SamplerRes::new(hash, engine.gl.create_sampler(Share::new(s)).unwrap());
                self.point_sampler = Some(engine.res_mgr.create::<SamplerRes<C>>(res));
            }
        };

        self.pipeline = Some(engine.create_pipeline(
            3,
            &CANVAS_TEXT_VS_SHADER_NAME.clone(),
            &CANVAS_TEXT_FS_SHADER_NAME.clone(),
            &[],
            self.rs.clone(),
            self.canvas_bs.clone(),
            self.ss.clone(),
            self.ds.clone(),
        ));
    }
}

impl<'a, C: Context + 'static, L: FlexNode + 'static> MultiCaseListener<'a, Node, CharBlock<L>, ModifyEvent> for CharBlockSys<C, L>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        self.dirtys.push(event.id);
    }
}

// 删除渲染对象
impl<'a, C: Context + 'static, L: FlexNode + 'static> MultiCaseListener<'a, Node, CharBlock<L>, DeleteEvent> for CharBlockSys<C, L>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, write: Self::WriteData){
        match self.render_map.remove(event.id) {
            Some(index) => {
                let notify = write.get_notify();
                write.remove(index, Some(notify));
            }, 
            None => (),
        };
        match self.shadow_render_map.remove(event.id) {
            Some(index) => {
                let notify = write.get_notify();
                write.remove(index, Some(notify));
            }, 
            None => (),
        };
    }
}

type MatrixRead<'a, L> = (&'a MultiCaseImpl<Node, WorldMatrixRender>, &'a MultiCaseImpl<Node, CharBlock<L>>,);

impl<'a, C: Context + 'static, L: FlexNode + 'static> MultiCaseListener<'a, Node, WorldMatrixRender, ModifyEvent> for CharBlockSys<C, L>{
    type ReadData = MatrixRead<'a, L>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read.0, render_objs);
        let charblock = match read.1.get(event.id) {
            Some(r) => r,
            None => return,
        };
        self.modify_shadow_matrix(event.id, read.0, render_objs, charblock.clazz.shadow.h, charblock.clazz.shadow.v);
    }
}

impl<'a, C: Context + 'static, L: FlexNode + 'static> MultiCaseListener<'a, Node, WorldMatrixRender, CreateEvent> for CharBlockSys<C, L>{
    type ReadData = MatrixRead<'a, L>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read.0, render_objs);
        let charblock = match read.1.get(event.id) {
            Some(r) => r,
            None => return,
        };
        self.modify_shadow_matrix(event.id, read.0, render_objs, charblock.clazz.shadow.h, charblock.clazz.shadow.v);
    }
}

#[inline]
fn modify_stroke<C: Context + 'static>(
    index: usize,
    text_stroke: &Stroke,
    render_obj: &mut RenderObj<C>,
    engine: &mut SingleCaseImpl<Engine<C>>,
    notify: &NotifyImpl,
    first_font: &Share<dyn SdfFont<Ctx=C>>,
) -> bool {
    notify.modify_event(index, "", 0);
    if first_font.get_dyn_type() > 0 {
        let mut common_ubo = render_obj.ubos.get_mut(&COMMON).unwrap();
        let color = &text_stroke.color;
        Share::make_mut(&mut common_ubo).set_float_4(&STROKE_COLOR, color.r, color.g, color.b, color.a);
        return false;
    }
    println!("text_stroke.width-------------{}", text_stroke.width);
    if text_stroke.width == 0.0 {
        //  删除边框的宏
        match render_obj.defines.remove_item(&STROKE) {
            Some(_) => {
                // 如果边框宏存在， 删除边框对应的ubo， 重新创建渲染管线
                render_obj.ubos.remove(&STROKE);
                true
            },
            None => false
        }
        
    } else {
        // 边框宽度不为0， 并且不存在STROKE宏， 应该添加STROKE宏， 并添加边框对应的ubo， 且重新创建渲染管线
        if find_item_from_vec(&render_obj.defines, &STROKE) == 0 {
            render_obj.defines.push(STROKE.clone());
            let mut stroke_ubo = engine.gl.create_uniforms();
            let color = &text_stroke.color;
            println!("text_stroke1-------------{}, {:?}", text_stroke.width / 10.0, color);
            stroke_ubo.set_float_1(&STROKE_SIZE, text_stroke.width / 10.0);
            stroke_ubo.set_float_4(&STROKE_COLOR, color.r, color.g, color.b, color.a);
            render_obj.ubos.insert(STROKE.clone(), Share::new(stroke_ubo));
            true
        }else {
            let stroke_ubo = render_obj.ubos.get_mut(&STROKE).unwrap();
            let color = &text_stroke.color;
            let stroke_ubo = Share::make_mut(stroke_ubo);
            println!("text_stroke2-------------{}, {:?}", text_stroke.width / 10.0, color);
            stroke_ubo.set_float_1(&STROKE_SIZE, text_stroke.width / 10.0);
            stroke_ubo.set_float_4(&STROKE_COLOR, color.r, color.g, color.b, color.a);
            false
        }
    }
}

#[inline]
fn modify_color<C: Context + 'static>(
    index: usize,
    color: &Color,
    engine: &mut SingleCaseImpl<Engine<C>>,
    notify: &NotifyImpl,
    render_obj: &mut RenderObj<C>,
) -> bool {
    let mut change = false;
    match color {
        Color::RGBA(c) => {
            // 如果未找到UCOLOR宏， 表示修改之前的颜色不为RGBA， 应该删除VERTEX_COLOR宏， 添加UCOLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
            if find_item_from_vec(&render_obj.defines, &UCOLOR) == 0 {
                render_obj.defines.remove_item(&VERTEX_COLOR);

                let ucolor_ubo = engine.gl.create_uniforms();
                render_obj.ubos.insert(UCOLOR.clone(), Share::new(ucolor_ubo));
                render_obj.defines.push(UCOLOR.clone());
                change = true;
            }
            // 设置ubo
            let ucolor_ubo = Share::make_mut(render_obj.ubos.get_mut(&UCOLOR).unwrap());
            debug_println!("text_color, color: {:?}", c);
            ucolor_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b, c.a);

            notify.modify_event(index, "", 0);
        },
        Color::LinearGradient(_) => {
            // 如果未找到VERTEX_COLOR宏， 表示修改之前的颜色不为LinearGradient， 应该删除UCOLOR宏， 添加VERTEX_COLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
            if find_item_from_vec(&render_obj.defines, &VERTEX_COLOR) == 0 {
                render_obj.defines.remove_item(&UCOLOR);
                render_obj.defines.push(VERTEX_COLOR.clone());
                render_obj.ubos.remove(&UCOLOR);   
                change = true;
            }
        },
    };
    change
}

#[inline]
fn try_modify_font<C: Context + 'static> (
    modify: usize,
    index: usize,
    render_obj: &mut RenderObj<C>,
    first_font: &Share<dyn SdfFont<Ctx=C>>,
    font_size: f32,
    notify: &NotifyImpl,
    default_sampler: &Res<SamplerRes<C>>,
    point_sampler: &Res<SamplerRes<C>>, // 点采样sampler
) -> bool {
    let mut change = false;
    notify.modify_event(index, "", 0);
    if (modify & (DirtyType::FontFamily as usize) != 0) || (modify & (DirtyType::FontSize as usize) != 0) {
        let common_ubo = render_obj.ubos.get_mut(&COMMON).unwrap();
        let common_ubo = Share::make_mut(common_ubo);
        // 如果是canvas 字体绘制类型， 并且绘制fontsize 与字体本身fontsize一致， 应该使用点采样
        let sampler = if first_font.get_dyn_type() > 0 && first_font.font_size() == font_size  {
            point_sampler
        } else {
            default_sampler
        };
        common_ubo.set_sampler(
            &TEXTURE,
            &(sampler.value.clone() as Share<dyn AsRef<<C as Context>::ContextSampler>>),
            &(first_font.texture().value.clone() as Share<dyn AsRef<<C as Context>::ContextTexture>>)
        );
        change = true;
    }
    
    if modify & (DirtyType::FontFamily as usize) != 0{
        // canvas字体绘制， 总是不需要STROKE宏
        if first_font.get_dyn_type() > 0 {
            if find_item_from_vec(&render_obj.defines, &STROKE) > 0 {
                render_obj.ubos.remove(&STROKE);
            }
        }
        change = true
    }
    change
}

#[inline]
fn modify_shadow_color<C: Context + 'static>(
    index: usize,
    c: &CgColor,
    notify: &NotifyImpl,
    render_obj: &mut RenderObj<C>,
    first_font: &Share<dyn SdfFont<Ctx=C>>,
) {
    let ucolor_ubo = Share::make_mut(render_obj.ubos.get_mut(&UCOLOR).unwrap());
    if first_font.get_dyn_type() > 0 {
        ucolor_ubo.set_float_4(&STROKE_COLOR, c.r, c.g, c.b, c.a);
    }
    ucolor_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b, c.a);
    notify.modify_event(index, "", 0);
}

// 返回position， uv， color， index
#[inline]
fn get_geo_flow<C: Context + 'static, L: FlexNode + 'static>(
    char_block: &CharBlock<L>,
    sdf_font: &Share<dyn SdfFont<Ctx = C>>,
    color: &Color,
    z_depth: f32,
    mut offset: (f32, f32)
) -> (Vec<f32>, Vec<f32>, Option<Vec<f32>>, Vec<u16>) {
    let len = char_block.chars.len();
    let mut positions: Vec<f32> = Vec::with_capacity(12 * len);
    let mut uvs: Vec<f32> = Vec::with_capacity(8 * len);
    let mut indices: Vec<u16> = Vec::with_capacity(6 * len);
    let font_size = char_block.font_size;
    let mut i = 0;
    offset.0 += char_block.pos.x;
    offset.1 += char_block.pos.y;

    debug_println!("charblock get_geo_flow: {:?}", char_block);
    if len > 0 {
        match color {
            Color::RGBA(_) => {
                for c in char_block.chars.iter() {
                    let glyph = match sdf_font.glyph_info(c.ch, font_size) {
                        Some(r) => r,
                        None => continue,
                    };
                    push_pos_uv(&mut positions, &mut uvs, &c.pos, &offset, &glyph, z_depth);
                    indices.extend_from_slice(&[i, i + 1, i + 2, i + 0, i + 2, i + 3]);
                    i += 4;  
                }
                return (positions, uvs, None, indices);
            },
            Color::LinearGradient(color) => {
                let mut colors = vec![Vec::new()];
                let (start, end) = cal_all_size(char_block, font_size, sdf_font); // 渐变范围
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
                    let glyph = match sdf_font.glyph_info(c.ch, font_size) {
                        Some(r) => r,
                        None => continue,
                    };

                    push_pos_uv(&mut positions, &mut uvs, &c.pos, &offset, &glyph, z_depth);
                    
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
                    i = positions.len() as u16 / 3;
                }
                return (positions, uvs, Some(colors.pop().unwrap()), indices);
            }
        }
    } else {
        return (positions, uvs, None, indices);
    }
}

// 返回position， uv， color， index
#[inline]
fn get_shadow_geo_flow<C: Context + 'static, L: FlexNode + 'static>(
    char_block: &CharBlock<L>,
    sdf_font: &Share<dyn SdfFont<Ctx = C>>,
    z_depth: f32,
) -> (Vec<f32>, Vec<f32>, Vec<u16>) {
    let len = char_block.chars.len();
    let mut positions: Vec<f32> = Vec::with_capacity(12 * len);
    let mut uvs: Vec<f32> = Vec::with_capacity(8 * len);
    let mut indices: Vec<u16> = Vec::with_capacity(6 * len);
    let font_size = char_block.font_size;
    let mut i = 0;

    let offset = (char_block.pos.x, char_block.pos.y);

    if char_block.chars.len() > 0 {
        for c in char_block.chars.iter() {
            let glyph = match sdf_font.glyph_info(c.ch, font_size) {
                Some(r) => r,
                None => continue,
            };
            push_pos_uv(&mut positions, &mut uvs, &c.pos, &offset, &glyph, z_depth);
            indices.extend_from_slice(&[i, i + 1, i + 2, i + 0, i + 2, i + 3]);
            i += 4;  
        }
        return (positions, uvs, indices);
    } else {
        return (positions, uvs, indices);
    }
}

#[inline]
fn cal_all_size<C: Context + 'static, L: FlexNode + 'static>(char_block: &CharBlock<L>, font_size: f32, sdf_font: &Share<dyn SdfFont<Ctx = C>>,) -> (Point2, Point2) {
    let mut start = Point2::new(0.0, 0.0);
    let mut end = Point2::new(0.0, 0.0);
    let mut j = 0;
    for i in 0..char_block.chars.len() {
        let pos = &char_block.chars[i].pos;
        let glyph = match sdf_font.glyph_info(char_block.chars[i].ch, font_size) {
            Some(r) => r,
            None => continue,
        };
        start = Point2::new(pos.x + glyph.ox, pos.y + glyph.oy);
        end = Point2::new(start.x + glyph.width, start.y + glyph.height);
        j += 1;
        break;
    }
    for i in j..char_block.chars.len() {
        let pos = &char_block.chars[i].pos;
        let glyph = match sdf_font.glyph_info(char_block.chars[i].ch, font_size) {
            Some(r) => r,
            None => continue,
        };
        if pos.x < start.x{
            start.x = pos.x;
        }
        let end_x = pos.x + glyph.width;
        if end_x > end.x {
            end.x = end_x;
        } 
        if pos.y < start.y{
            start.y = pos.y;
        }
        let end_y = pos.y + font_size;
        if end_y > end.y {
            end.y = end_y;
        } 
    }
    (start, end)
}

#[inline]
fn fill_uv(positions: &mut Vec<f32>, uvs: &mut Vec<f32>, i: usize){
    let pi = i * 3;
    let uvi = i * 2;
    let len = positions.len() - pi;
    let (p1, p4) = (
        (
            positions[pi],
            positions[pi + 1],
        ),
        (
            positions[pi + 6],
            positions[pi + 7],
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
    debug_println!("p1: {}, {}, p4: {}, {}, u1: {},{}, u4: {}, {}", p1.0, p1.1, p4.0, p4.1, u1.0, u1.1, u4.0, u4.1);
    if len > 12 {
        let mut i = pi + 12;
        for _j in 0..(len - 12)/3 {
            let pos_x = positions[i];
            let pos_y = positions[i + 1];
            debug_println!("pos_x: {}, pos_y: {}, i:derive_deref{}", pos_x, pos_y, i);
            let uv;
            if (pos_x - p1.0).abs() < 0.001{
                debug_println!("pos_x == p1.0, i: {}, pos_x: {}, p1.0: {}", i, pos_x, p1.0);
                let base = p4.1 - p1.1;
                let ratio = if base == 0.0 {
                    0.0
                } else {
                    (pos_y - p1.1)/(p4.1 - p1.1)
                };
                uv = (u1.0, u1.1 * (1.0 - ratio) + u4.1 * ratio );
            }else if (pos_x - p4.0).abs() < 0.001{
                debug_println!("pos_x == p4.0, i: {}, pos_x: {}, p4.0:{}", i, pos_x, p4.0);
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
            debug_println!("uvs: {}, {}", uv.0, uv.1);
            i += 3;
        }
    }
}



fn push_pos_uv(positions: &mut Vec<f32>, uvs: &mut Vec<f32>, pos: &Point2 , offset: &(f32, f32), glyph: &GlyphInfo, z_depth: f32){
    let left_top = (pos.x + offset.0 + glyph.ox, pos.y + offset.1 + glyph.oy);
    let right_bootom = (left_top.0 + glyph.width, left_top.1 + glyph.height);
    let ps = [
        left_top.0,     left_top.1,     z_depth,
        left_top.0,     right_bootom.1, z_depth,
        right_bootom.0, right_bootom.1, z_depth,
        right_bootom.0, left_top.1,     z_depth,
    ];
    uvs.extend_from_slice(&[
        glyph.u_min, glyph.v_min,
        glyph.u_min, glyph.v_max,
        glyph.u_max, glyph.v_max,
        glyph.u_max, glyph.v_min,
    ]);
    positions.extend_from_slice(&ps[0..12]);
}

unsafe impl<C: Context + 'static, L: FlexNode + 'static> Sync for CharBlockSys<C, L>{}
unsafe impl<C: Context + 'static, L: FlexNode + 'static> Send for CharBlockSys<C, L>{}

impl_system!{
    CharBlockSys<C, L> where [C: Context + 'static, L: FlexNode + 'static],
    true,
    {
        MultiCaseListener<Node, CharBlock<L>, ModifyEvent>
        MultiCaseListener<Node, CharBlock<L>, DeleteEvent>
        MultiCaseListener<Node, WorldMatrixRender, CreateEvent>
        MultiCaseListener<Node, WorldMatrixRender, ModifyEvent>
    }
}