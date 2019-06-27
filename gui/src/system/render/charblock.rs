/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use share::Share;

use fnv::FnvHashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseImpl, MultiCaseImpl, Share as ShareTrait, Runner};
use map::{ vecmap::VecMap } ;
use hal_core::*;
use atom::Atom;
use polygon::{mult_to_triangle, interp_mult_by_lg, split_by_lg, LgCfg, find_lg_endp};

use component::user::*;
use single::*;
use component::calc::{Opacity, ZDepth, CharBlock, WorldMatrixRender};
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


lazy_static! {
    static ref STROKE: Atom = Atom::from("STROKE");
    static ref UCOLOR: Atom = Atom::from("UCOLOR");
    static ref VERTEX_COLOR: Atom = Atom::from("VERTEX_COLOR");

    static ref STROKE_SIZE: Atom = Atom::from("strokeSize");
    static ref STROKE_COLOR: Atom = Atom::from("strokeColor");
    static ref U_COLOR: Atom = Atom::from("uColor");
}

pub struct CharBlockSys<C: Context + ShareTrait, L: FlexNode + ShareTrait>{
    render_map: VecMap<Item>,
    geometry_dirtys: Vec<usize>,
    mark: PhantomData<(C, L)>,
    rs: Share<RasterState>,
    bs: Share<BlendState>,
    ss: Share<StencilState>,
    ds: Share<DepthState>,

    canvas_bs: Share<BlendState>,

    pipelines: FnvHashMap<u64, Share<PipelineInfo>>,
    default_sampler: Option<Res<SamplerRes<C>>>,
}

impl<C: Context + ShareTrait, L: FlexNode + ShareTrait> CharBlockSys<C, L> {
    pub fn new() -> Self{
        let mut bs = BlendState::new();
        let mut ds = DepthState::new();
        let mut canvas_bs = BlendState::new();
        bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        canvas_bs.set_rgb_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);
        ds.set_write_enable(false);
        CharBlockSys {
            render_map: VecMap::default(),
            geometry_dirtys: Vec::new(),
            mark: PhantomData,
            rs: Share::new(RasterState::new()),
            bs: Share::new(bs),
            ss: Share::new(StencilState::new()),
            ds: Share::new(ds),
            canvas_bs:  Share::new(canvas_bs),
            pipelines: FnvHashMap::default(),
            default_sampler: None,
        }
    }   
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> Runner<'a> for CharBlockSys<C, L>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, TextStyle>,
        &'a MultiCaseImpl<Node, Font>,
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a SingleCaseImpl<DefaultTable>,
        &'a MultiCaseImpl<Node, WorldMatrixRender>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>, &'a mut MultiCaseImpl<Node, CharBlock<L>>,);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let (z_depths, text_styles, fonts, font_sheet, default_table, world_matrixs) = read;
        let (render_objs, engine, charblocks) = write;
        for id in  self.geometry_dirtys.iter() {
            let map = &mut self.render_map;
            let item = unsafe { map.get_unchecked_mut(*id) };
            item.position_change = false;
            let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;
            let charblock = unsafe { charblocks.get_unchecked_mut(*id) };
            let text_style = get_or_default(*id, text_styles, default_table);
            let font = get_or_default(*id, fonts, default_table);
            charblock.dirty = false;
            let first_font = match font_sheet.get_first_font(&font.family) {
                Some(r) => r,
                None => {
                    debug_println!("font is not exist: {}", font.family.as_str());
                    return;
                }
            };
            let (positions, uvs, colors, indices) = get_geo_flow(charblock, &first_font, &text_style.color, z_depth + 0.2, (0.0, 0.0));

            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
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
            render_objs.get_notify().modify_event(item.index, "geometry", 0);

            self.modify_matrix(*id, world_matrixs, render_objs);
        }
        self.geometry_dirtys.clear();
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
    }
}

// 插入渲染对象
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, CharBlock<L>, CreateEvent> for CharBlockSys<C, L>{
    type ReadData = (
        &'a MultiCaseImpl<Node, TextStyle>,
        &'a MultiCaseImpl<Node, Font>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a SingleCaseImpl<DefaultTable>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs<C>>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (text_styles, fonts, z_depths, opacitys, font_sheet, default_table) = read;
        let (render_objs, engine) = write;
        let z_depth = unsafe { z_depths.get_unchecked(event.id) }.0;
        let _opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
        let text_style = get_or_default(event.id, text_styles, default_table);
        let font = get_or_default(event.id, fonts, default_table);
        let mut defines = Vec::new();

        let mut ubos: FnvHashMap<Atom, Share<Uniforms<C>>> = FnvHashMap::default();

        let mut common_ubo = engine.gl.create_uniforms();
        let dyn_type = match font_sheet.get_first_font(&font.family) {
            Some(r) => {
                println!("name1: {:?}, font_size: {}, font_sheet.get_size(&font.family, &font.size): {}", r.name(), r.font_size(), font_sheet.get_size(&font.family, &font.size));
                let sampler = if r.get_dyn_type() > 0 && r.font_size() == font_sheet.get_size(&font.family, &font.size)  {
                    let mut s = SamplerDesc::default();
                    s.min_filter = TextureFilterMode::Nearest;
                    s.mag_filter = TextureFilterMode::Nearest;
                    let hash = sampler_desc_hash(&s);
                    match engine.res_mgr.get::<SamplerRes<C>>(&hash) {
                        Some(r) => r.clone(),
                        None => {
                            let res = SamplerRes::new(hash, engine.gl.create_sampler(Share::new(s)).unwrap());
                            engine.res_mgr.create::<SamplerRes<C>>(res)
                        }
                    }
                } else {
                    self.default_sampler.clone().unwrap()
                };
                common_ubo.set_sampler(
                    &TEXTURE,
                    &(sampler.value.clone() as Share<dyn AsRef<<C as Context>::ContextSampler>>),
                    &(r.texture().value.clone() as Share<dyn AsRef<<C as Context>::ContextTexture>>)
                );
                r.get_dyn_type()
            },
            None => { debug_println!("font is not exist: {}", font.family.as_str()); 0 }
        }; 

        match &text_style.color {
            Color::RGBA(c) => {
                let mut ucolor_ubo = engine.gl.create_uniforms();
                ucolor_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b, c.a);
                ubos.insert(UCOLOR.clone(), Share::new(ucolor_ubo));
                defines.push(UCOLOR.clone());
                debug_println!("text, id: {}, color: {:?}", event.id, c);
            },
            Color::LinearGradient(_) => {
                defines.push(VERTEX_COLOR.clone());
            },
        }

        let pipeline;
        if dyn_type == 0 {
            pipeline = engine.create_pipeline(
                1,
                &TEXT_VS_SHADER_NAME.clone(),
                &TEXT_FS_SHADER_NAME.clone(),
                defines.as_slice(),
                self.rs.clone(),
                self.bs.clone(),
                self.ss.clone(),
                self.ds.clone()
            );
        }else {
            common_ubo.set_float_4(&STROKE_COLOR, 1.0, 1.0, 1.0, 1.0);
            pipeline = engine.create_pipeline(
                3,
                &CANVAS_TEXT_VS_SHADER_NAME.clone(),
                &CANVAS_TEXT_FS_SHADER_NAME.clone(),
                defines.as_slice(),
                self.rs.clone(),
                self.canvas_bs.clone(),
                self.ss.clone(),
                self.ds.clone(),
            );
        }

        ubos.insert(COMMON.clone(), Share::new(common_ubo)); // COMMON

        let render_obj: RenderObj<C> = RenderObj {
            depth: z_depth + 0.2,
            depth_diff: 0.2,
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


// 字体修改， 设置顶点数据脏
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, CharBlock<L>, ModifyEvent> for CharBlockSys<C, L>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        let item = unsafe { self.render_map.get_unchecked_mut(event.id) };
        debug_println!("CharBlock<L> modify-----------------------------, id: {}", event.id);
        if item.position_change == false {
            item.position_change = true;
            self.geometry_dirtys.push(event.id);
        }
    }
}


// 删除渲染对象
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, CharBlock<L>, DeleteEvent> for CharBlockSys<C, L>{
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

// 字体修改， 重新设置字体纹理
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, Font, ModifyEvent> for CharBlockSys<C, L>{
    type ReadData = (
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a MultiCaseImpl<Node, Font>
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        if let Some(item) = self.render_map.get_mut(event.id) {
            let (font_sheet, fonts) = read;
            let (render_objs, engine) = write;
            modify_font(event.id, item, self.default_sampler.clone().unwrap(), font_sheet, fonts, render_objs, engine, &self.rs,  &self.bs,  &self.ss,  &self.ds, &self.canvas_bs);
            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(event.id);
            }
            render_objs.get_notify().modify_event(event.id, "pipeline", 0);
        }
    }
}

// 字体修改， 重新设置字体纹理
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, Font, CreateEvent> for CharBlockSys<C, L>{
    type ReadData = (
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a MultiCaseImpl<Node, Font>
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        if let Some(item) = self.render_map.get_mut(event.id) {
            let (font_sheet, fonts) = read;
            let (render_objs, engine) = write;
            modify_font(event.id, item, self.default_sampler.clone().unwrap(), font_sheet, fonts, render_objs, engine, &self.rs,  &self.bs,  &self.ss,  &self.ds, &self.canvas_bs);
            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(event.id);
            }
            render_objs.get_notify().modify_event(event.id, "pipeline", 0);
        }
    }
}

// TextStyle修改， 设置对应的ubo和宏
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, TextStyle, CreateEvent> for CharBlockSys<C, L>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, TextStyle>,
        &'a MultiCaseImpl<Node, Font>,
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a SingleCaseImpl<DefaultTable>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (opacitys, text_styles, fonts, font_sheet, default_table) = read;   
        let (render_objs, engine) = write;
        if let Some(item) = self.render_map.get_mut(event.id) {
            let _opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
            let text_style = unsafe { text_styles.get_unchecked(event.id) };
            modify_color(&mut self.geometry_dirtys, item, event.id, text_style, render_objs, engine);
            let font = get_or_default(event.id, fonts, default_table);
            modify_stroke(item, text_style, render_objs, engine, font, font_sheet);
            // let index = item.index;
            // self.change_is_opacity(opacity, text_style, index, render_objs);
        }
    }
}

// TextStyle修改， 设置对应的ubo和宏
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, TextStyle, DeleteEvent> for CharBlockSys<C, L>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, TextStyle>,
        &'a MultiCaseImpl<Node, Font>,
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a SingleCaseImpl<DefaultTable>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData){
        let (opacitys, text_styles, fonts, font_sheet, default_table) = read;   
        let (render_objs, engine) = write;
        if let Some(item) = self.render_map.get_mut(event.id) {
            let _opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
            let text_style = unsafe { text_styles.get_unchecked(event.id) };
            modify_color(&mut self.geometry_dirtys, item, event.id, text_style, render_objs, engine);
            let font = get_or_default(event.id, fonts, default_table);
            modify_stroke(item, text_style, render_objs, engine, font, font_sheet);
            // let index = item.index;
            // self.change_is_opacity(opacity, text_style, index, render_objs);
        }
    }
}

// TextStyle修改， 设置对应的ubo和宏
impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, TextStyle, ModifyEvent> for CharBlockSys<C, L>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, TextStyle>,
        &'a MultiCaseImpl<Node, Font>,
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a SingleCaseImpl<DefaultTable>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (_opacitys, text_styles, fonts, font_sheet, default_table) = read;   
        let (render_objs, engine) = write;
        if let Some(item) = self.render_map.get_mut(event.id) {
            let text_style = unsafe { text_styles.get_unchecked(event.id) };
            
            match event.field {
                "color" => {
                    modify_color(&mut self.geometry_dirtys, item, event.id, text_style, render_objs, engine);
                },
                "stroke" => {
                    let font = get_or_default(event.id, fonts, default_table);
                    modify_stroke(item, text_style, render_objs, engine, font, font_sheet);
                },
                _ => return,
            }

            // let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
            // let index = item.index;
            // self.change_is_opacity(opacity, text_style, index, render_objs);
        }
    }
}

type MatrixRead<'a> = &'a MultiCaseImpl<Node, WorldMatrixRender>;

impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, WorldMatrixRender, ModifyEvent> for CharBlockSys<C, L>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read, render_objs);
    }
}

impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, WorldMatrixRender, CreateEvent> for CharBlockSys<C, L>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read, render_objs);
    }
}

impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> CharBlockSys<C, L>{
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
            debug_println!("charblock, id: {}, world_matrix: {:?}", render_obj.context, &slice[0..16]);
            render_objs.get_notify().modify_event(item.index, "ubos", 0);
        }
    }
}

// //不透明度变化， 修改渲染对象的is_opacity属性
// impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for CharBlockSys<C, L>{
//     type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, TextStyle>);
//     type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
//     fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
//         let (opacitys, text_styles) = read;
//         if let Some(item) = self.render_map.get(event.id) {
//             let opacity = unsafe { opacitys.get_unchecked(event.id).0 };
//             // let text_style = unsafe { text_styles.get_unchecked(event.id) };
//             // let index = item.index;
//             // self.change_is_opacity( opacity, text_style, index, write);
//         }
//     }
// }

// impl<'a, C: Context + ShareTrait, L: FlexNode + ShareTrait> CharBlockSys<C, L> {
//     fn change_is_opacity(&mut self, opacity: f32, text_style: &TextStyle, index: usize, render_objs: &mut SingleCaseImpl<RenderObjs<C>>){
//         let is_opacity = if opacity < 1.0 || !text_style.color.is_opaque() || text_style.stroke.color.a < 1.0{
//             false
//         }else {
//             true
//         };

//         let notify = render_objs.get_notify();
//         unsafe { render_objs.get_unchecked_write(index, &notify).set_is_opacity(is_opacity)};
//     }
// }

struct Item {
    index: usize,
    position_change: bool,
}

fn modify_stroke<C: Context + ShareTrait>(
    item: &mut Item,
    text_style: &TextStyle,
    render_objs: &mut SingleCaseImpl<RenderObjs<C>>,
    engine: &mut SingleCaseImpl<Engine<C>>,
    font: &Font,
    font_sheet: &SingleCaseImpl<FontSheet<C>>,
) {
    let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
    let first_font = match font_sheet.get_first_font(&font.family) {
        Some(r) => r,
        None => {
            debug_println!("font is not exist: {}", font.family.as_ref());
            return;
        }
    };
    if first_font.get_dyn_type() > 0 {
        let mut common_ubo = render_obj.ubos.get_mut(&COMMON).unwrap();
        let color = &text_style.stroke.color;
        Share::make_mut(&mut common_ubo).set_float_4(&STROKE_COLOR, color.r, color.g, color.b, color.a);
        return;
    }
    if text_style.stroke.width == 0.0 {
        //  删除边框的宏
        match render_obj.defines.remove_item(&STROKE) {
            Some(_) => {
                // 如果边框宏存在， 删除边框对应的ubo， 重新创建渲染管线
                render_obj.ubos.remove(&STROKE);
                let old_pipeline = render_obj.pipeline.clone();
                render_obj.pipeline = engine.create_pipeline(
                    old_pipeline.start_hash,
                    &TEXT_VS_SHADER_NAME.clone(),
                    &TEXT_FS_SHADER_NAME.clone(),
                    render_obj.defines.as_slice(),
                    old_pipeline.rs.clone(),
                    old_pipeline.bs.clone(),
                    old_pipeline.ss.clone(),
                    old_pipeline.ds.clone()
                );
                render_objs.get_notify().modify_event(item.index, "pipeline", 0);
            },
            None => ()
        };
        
    } else {
        // 边框宽度不为0， 并且不存在STROKE宏， 应该添加STROKE宏， 并添加边框对应的ubo， 且重新创建渲染管线
        if find_item_from_vec(&render_obj.defines, &STROKE) == 0 {
            render_obj.defines.push(STROKE.clone());
            let mut stroke_ubo = engine.gl.create_uniforms();
            let color = &text_style.stroke.color;
            stroke_ubo.set_float_1(&STROKE_SIZE, text_style.stroke.width);
            stroke_ubo.set_float_4(&STROKE_COLOR, color.r, color.g, color.b, color.a);
            render_obj.ubos.insert(STROKE.clone(), Share::new(stroke_ubo));
            let old_pipeline = render_obj.pipeline.clone();
            render_obj.pipeline = engine.create_pipeline(
                old_pipeline.start_hash,
                &TEXT_VS_SHADER_NAME.clone(),
                &TEXT_FS_SHADER_NAME.clone(),
                render_obj.defines.as_slice(),
                old_pipeline.rs.clone(),
                old_pipeline.bs.clone(),
                old_pipeline.ss.clone(),
                old_pipeline.ds.clone()
            );
            render_objs.get_notify().modify_event(item.index, "pipeline", 0);
        }else {
            let stroke_ubo = render_obj.ubos.get_mut(&STROKE).unwrap();
            let color = &text_style.stroke.color;
            let stroke_ubo = Share::make_mut(stroke_ubo);
            stroke_ubo.set_float_1(&STROKE_SIZE, text_style.stroke.width);
            stroke_ubo.set_float_4(&STROKE_COLOR, color.r, color.g, color.b, color.a);
        }
    }   
}

fn modify_color<C: Context + ShareTrait>(geometry_dirtys: &mut Vec<usize>, item: &mut Item, id: usize, text_style: &TextStyle, render_objs: &mut SingleCaseImpl<RenderObjs<C>>, engine: &mut SingleCaseImpl<Engine<C>>) {
    let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
    match &text_style.color {
        Color::RGBA(c) => {
            // 如果未找到UCOLOR宏， 表示修改之前的颜色不为RGBA， 应该删除VERTEX_COLOR宏， 添加UCOLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
            if find_item_from_vec(&render_obj.defines, &UCOLOR) == 0 {
                render_obj.defines.remove_item(&VERTEX_COLOR);

                let ucolor_ubo = engine.gl.create_uniforms();
                render_obj.ubos.insert(UCOLOR.clone(), Share::new(ucolor_ubo));
                render_obj.defines.push(UCOLOR.clone());

                if item.position_change == false {
                    item.position_change = true;
                    geometry_dirtys.push(id);
                }
            }
            // 设置ubo
            let ucolor_ubo = Share::make_mut(render_obj.ubos.get_mut(&UCOLOR).unwrap());
            debug_println!("text_color, id: {}, color: {:?}", id, c);
            ucolor_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b, c.a);

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
                    geometry_dirtys.push(id);
                }
            }
        },
    }
}

fn modify_font<C: Context + ShareTrait> (
    id: usize,
    item: &mut Item,
    default_sampler: Res<SamplerRes<C>>,
    font_sheet: &SingleCaseImpl<FontSheet<C>>,
    fonts: &MultiCaseImpl<Node, Font>,
    render_objs: &mut SingleCaseImpl<RenderObjs<C>>,
    engine:  &mut SingleCaseImpl<Engine<C>>,
    rs: &Share<RasterState>,
    bs: &Share<BlendState>,
    ss: &Share<StencilState>,
    ds: &Share<DepthState>,
    canvas_bs: &Share<BlendState>,
) {
    let font = unsafe { fonts.get_unchecked(id) };
    let first_font = match font_sheet.get_first_font(&font.family) {
        Some(r) => r,
        None => {
            debug_println!("font is not exist: {}", font.family.as_ref());
            return;
        }
    };

    let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
    let common_ubo = render_obj.ubos.get_mut(&COMMON).unwrap();
    let common_ubo = Share::make_mut(common_ubo);
    println!("name: {:?}, font_size: {}, font_sheet.get_size(&font.family, &font.size): {}", first_font.name(), first_font.font_size(), font_sheet.get_size(&font.family, &font.size));
    let sampler = if first_font.get_dyn_type() > 0 && first_font.font_size() == font_sheet.get_size(&font.family, &font.size)  {
        let mut s = SamplerDesc::default();
        s.min_filter = TextureFilterMode::Nearest;
        s.mag_filter = TextureFilterMode::Nearest;
        let hash = sampler_desc_hash(&s);
        match engine.res_mgr.get::<SamplerRes<C>>(&hash) {
            Some(r) => r.clone(),
            None => {
                let res = SamplerRes::new(hash, engine.gl.create_sampler(Share::new(s)).unwrap());
                engine.res_mgr.create::<SamplerRes<C>>(res)
            }
        }
    } else {
        default_sampler
    };
    common_ubo.set_sampler(
        &TEXTURE,
        &(sampler.value.clone() as Share<dyn AsRef<<C as Context>::ContextSampler>>),
        &(first_font.texture().value.clone() as Share<dyn AsRef<<C as Context>::ContextTexture>>)
    );

    if first_font.get_dyn_type() == 0 {
        if find_item_from_vec(&render_obj.defines, &STROKE) > 0 {
            render_obj.ubos.remove(&STROKE);
        }
        Share::make_mut(render_obj.ubos.get_mut(&COMMON).unwrap()).remove(&STROKE_COLOR);
        render_obj.pipeline = engine.create_pipeline(
            1,
            &TEXT_VS_SHADER_NAME.clone(),
            &TEXT_FS_SHADER_NAME.clone(),
            render_obj.defines.as_slice(),
            rs.clone(),
            bs.clone(),
            ss.clone(),
            ds.clone()
        );
    } else {
        render_obj.pipeline = engine.create_pipeline(
            3,
            &CANVAS_TEXT_VS_SHADER_NAME.clone(),
            &CANVAS_TEXT_FS_SHADER_NAME.clone(),
            render_obj.defines.as_slice(),
            rs.clone(),
            canvas_bs.clone(),
            ss.clone(),
            ds.clone(),
        );
    } 
}

// 返回position， uv， color， index
fn get_geo_flow<C: Context + ShareTrait, L: FlexNode + ShareTrait>(
    char_block: &CharBlock<L>,
    sdf_font: &Share<dyn SdfFont<Ctx = C>>,
    color: &Color,
    z_depth: f32,
    mut offset: (f32, f32)
) -> (Vec<f32>, Vec<f32>, Option<Vec<f32>>, Vec<u16>) {
    let mut positions: Vec<f32> = Vec::new();
    let mut uvs: Vec<f32> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    let font_size = char_block.font_size;
    let mut i = 0;
    offset.1 += (char_block.line_height - font_size)/2.0;

    debug_println!("charblock get_geo_flow: {:?}", char_block);
    if char_block.chars.len() > 0 {
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
                    start.x, start.y,
                    start.x, end.y,
                    end.x, end.y,
                    end.x, start.y,
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

fn cal_all_size<C: Context + ShareTrait, L: FlexNode + ShareTrait>(char_block: &CharBlock<L>, font_size: f32, sdf_font: &Share<dyn SdfFont<Ctx = C>>,) -> (Point2, Point2) {
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
            debug_println!("pos_x: {}, pos_x: {}, i:derive_deref{}", pos_x, pos_y, i);
            let uv;
            if pos_x - p1.0 < 0.001 || p1.0 - pos_x < 0.001 {
                debug_println!("pos_x == p1.0, i: {}", i);
                let base = p4.1 - p1.1;
                let ratio = if base == 0.0 {
                    0.0
                } else {
                    (pos_y - p1.1)/(p4.1 - p1.1)
                };
                uv = (u1.0, u1.1 * (1.0 - ratio) + u4.1 * ratio );
            }else if pos_x - p4.0 < 0.001 || p4.0 - pos_x < 0.001{
                debug_println!("pos_x == p4.0, i: {}", i);
                let base = p4.1 - p1.1;
                let ratio = if base == 0.0 {
                    0.0
                } else {
                    (pos_y - p1.1)/(p4.1 - p1.1)
                };
                uv = (u4.0, u1.1  * (1.0 - ratio) + u4.1 * ratio );
            }else if pos_y - p1.1 < 0.001 || p1.1 - pos_y < 0.001 {
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

// //取几何体的顶点流、 颜色流和属性流
// fn get_geo_flow(char_block: &CharBlock<L>, color: &Color, layout: &Layout, z_depth: f32) -> (Vec<f32>, Option<Vec<f32>>, Vec<u16>) {
//     unimplemented!()
    

//     // (positions, uvs, indices)
// }

unsafe impl<C: Context + ShareTrait, L: FlexNode + ShareTrait> Sync for CharBlockSys<C, L>{}
unsafe impl<C: Context + ShareTrait, L: FlexNode + ShareTrait> Send for CharBlockSys<C, L>{}

impl_system!{
    CharBlockSys<C, L> where [C: Context + ShareTrait, L: FlexNode + ShareTrait],
    true,
    {
        MultiCaseListener<Node, CharBlock<L>, CreateEvent>
        MultiCaseListener<Node, CharBlock<L>, ModifyEvent>
        MultiCaseListener<Node, CharBlock<L>, DeleteEvent>
        MultiCaseListener<Node, TextStyle, CreateEvent>
        MultiCaseListener<Node, TextStyle, ModifyEvent>
        MultiCaseListener<Node, TextStyle, DeleteEvent>
        MultiCaseListener<Node, Font, ModifyEvent>
        MultiCaseListener<Node, WorldMatrixRender, CreateEvent>
        MultiCaseListener<Node, WorldMatrixRender, ModifyEvent>
    }
}