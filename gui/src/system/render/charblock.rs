/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use std::sync::Arc;

use std::collections::HashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use map::{ vecmap::VecMap } ;
use hal_core::*;
use atom::Atom;
use polygon::{mult_to_triangle, interp_mult_by_lg, split_by_lg, LgCfg, find_lg_endp};

use component::user::*;
use single::*;
use component::calc::{Opacity, ZDepth, CharBlock};
use entity::{Node};
use render::engine::{ Engine , PipelineInfo};
use render::res::{ SamplerRes};
use system::util::*;
use system::util::constant::*;
use system::render::shaders::text::{TEXT_FS_SHADER_NAME, TEXT_VS_SHADER_NAME};
use font::font_sheet::FontSheet;
use font::sdf_font:: {GlyphInfo, SdfFont };


lazy_static! {
    static ref STROKE: Atom = Atom::from("STROKE");
    static ref UCOLOR: Atom = Atom::from("UCOLOR");
    static ref VERTEX_COLOR: Atom = Atom::from("VERTEX_COLOR");

    static ref STROKE_SIZE: Atom = Atom::from("strokeSize");
    static ref STROKE_COLOR: Atom = Atom::from("strokeColor");
    static ref U_COLOR: Atom = Atom::from("uColor");
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
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, TextStyle>,
        &'a MultiCaseImpl<Node, CharBlock>,
        &'a MultiCaseImpl<Node, Font>,
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a SingleCaseImpl<DefaultTable>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let map = &mut self.charblock_render_map;
        let (z_depths, text_styles, charblocks, fonts, font_sheet, default_table) = read;
        let (render_objs, _) = write;
        for id in  self.geometry_dirtys.iter() {
            let item = unsafe { map.get_unchecked_mut(*id) };
            item.position_change = false;
            let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;
            let charblock = unsafe { charblocks.get_unchecked(*id) };
            let text_style = get_or_default(*id, text_styles, default_table);
            let font = get_or_default(*id, fonts, default_table);
            let first_font = match font_sheet.get_first_font(&font.family) {
                Some(r) => r,
                None => {
                    debug_println!("font is not exist: {}", font.family.as_str());
                    return;
                }
            };
            let (positions, uvs, colors, indices) = get_geo_flow(charblock, &first_font, &text_style.color, z_depth + 0.1, (0.0, 0.0));

            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
            let geometry = unsafe {&mut *(render_obj.geometry.as_ref() as *const C::ContextGeometry as usize as *mut C::ContextGeometry)};

            let vertex_count: u32 = (positions.len()/3) as u32;
            if vertex_count != geometry.get_vertex_count() {
                geometry.set_vertex_count(vertex_count);
            }
            geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false).unwrap();
            geometry.set_attribute(&AttributeName::UV0, 2, Some(uvs.as_slice()), false).unwrap();
            geometry.set_indices_short(indices.as_slice(), false).unwrap();
            match colors {
                Some(color) => {geometry.set_attribute(&AttributeName::Color, 4, Some(color.as_slice()), false).unwrap();},
                None => ()
            };
        }
        self.geometry_dirtys.clear();
    }

    fn setup(&mut self, _: Self::ReadData, write: Self::WriteData){
        let (_, engine) = write;
        let s = SamplerDesc::default();
        let hash = sampler_desc_hash(&s);
        match engine.res_mgr.samplers.get(&hash) {
            Some(r) => self.default_sampler = Some(r.clone()),
            None => {
                let res = SamplerRes::new(hash, engine.gl.create_sampler(Arc::new(s)).unwrap());
                self.default_sampler = Some(engine.res_mgr.samplers.create(res));
            }
        }
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock, CreateEvent> for CharBlockSys<C>{
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
        let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
        let text_style = get_or_default(event.id, text_styles, default_table);
        let font = get_or_default(event.id, fonts, default_table);
        let first_font = match font_sheet.get_first_font(&font.family) {
            Some(r) => r,
            None => {
                debug_println!("font is not exist: {}", font.family.as_str());
                return;
            }
        };
        let mut defines = Vec::new();

        let geometry = create_geometry(&mut engine.gl);
        let mut ubos: HashMap<Atom, Arc<Uniforms<C>>> = HashMap::default();

        let mut common_ubo = engine.gl.create_uniforms();
        
        common_ubo.set_sampler(
            &TEXTURE,
            &(self.default_sampler.as_ref().unwrap().clone() as Arc<AsRef<<C as Context>::ContextSampler>>),
            &(first_font.texture().clone() as Arc<AsRef<<C as Context>::ContextTexture>>)
        );
        ubos.insert(COMMON.clone(), Arc::new(common_ubo)); // COMMON

        match &text_style.color {
            Color::RGBA(c) => {
                let mut ucolor_ubo = engine.gl.create_uniforms();
                ucolor_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b, c.a);
                ubos.insert(UCOLOR.clone(), Arc::new(ucolor_ubo));
                defines.push(UCOLOR.clone());
                debug_println!("text, id: {}, color: {:?}", event.id, c);
            },
            Color::LinearGradient(_) => {
                defines.push(VERTEX_COLOR.clone());
            },
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
        
        let is_opacity = if opacity < 1.0 || !text_style.color.is_opaque() || text_style.stroke.color.a < 1.0{
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
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock, ModifyEvent> for CharBlockSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        let item = unsafe { self.charblock_render_map.get_unchecked_mut(event.id) };
        if item.position_change == false {
            item.position_change = true;
            self.geometry_dirtys.push(event.id);
        }
    }
}


// 删除渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock, DeleteEvent> for CharBlockSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, write: Self::WriteData){
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
        &'a MultiCaseImpl<Node, Font>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (font_sheet, fonts) = read;
        if let Some(item) = self.charblock_render_map.get_mut(event.id) {
            let font = unsafe { fonts.get_unchecked(event.id) };
            let first_font = match font_sheet.get_first_font(&font.family) {
                Some(r) => r,
                None => {
                    debug_println!("font is not exist: {}", font.family.as_ref());
                    return;
                }
            };

            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
            let common_ubo = render_obj.ubos.get_mut(&COMMON).unwrap();
            let common_ubo = Arc::make_mut(common_ubo);
            common_ubo.set_sampler(
                &TEXTURE,
                &(self.default_sampler.as_ref().unwrap().clone() as Arc<AsRef<<C as Context>::ContextSampler>>),
                &(first_font.texture().clone() as Arc<AsRef<<C as Context>::ContextTexture>>)
            );
        }
    }
}

// TextStyle修改， 设置对应的ubo和宏
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, TextStyle, CreateEvent> for CharBlockSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, TextStyle>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (opacitys, text_styles) = read;   
        let (render_objs, engine) = write;
        if let Some(item) = self.charblock_render_map.get_mut(event.id) {
            let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
            let text_style = unsafe { text_styles.get_unchecked(event.id) };
            modify_color(&mut self.geometry_dirtys, item, event.id, text_style, render_objs, engine);
            modify_stroke(item, text_style, render_objs, engine);
            let index = item.index;
            self.change_is_opacity(opacity, text_style, index, render_objs);
        }
    }
}

// TextStyle修改， 设置对应的ubo和宏
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, TextStyle, ModifyEvent> for CharBlockSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, TextStyle>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (opacitys, text_styles) = read;   
        let (render_objs, engine) = write;
        println!("modify style {:?}, {:?}", event.field, unsafe { text_styles.get_unchecked(event.id) });
        if let Some(item) = self.charblock_render_map.get_mut(event.id) {
            let text_style = unsafe { text_styles.get_unchecked(event.id) };
            
            match event.field {
                "color" => {
                    modify_color(&mut self.geometry_dirtys, item, event.id, text_style, render_objs, engine);
                },
                "stroke" => {
                    modify_stroke(item, text_style, render_objs, engine);
                },
                _ => return,
            }

            let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
            let index = item.index;
            self.change_is_opacity(opacity, text_style, index, render_objs);
        }
    }
}

//不透明度变化， 修改渲染对象的is_opacity属性
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for CharBlockSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, TextStyle>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (opacitys, text_styles) = read;
        if let Some(item) = self.charblock_render_map.get(event.id) {
            let opacity = unsafe { opacitys.get_unchecked(event.id).0 };
            let text_style = unsafe { text_styles.get_unchecked(event.id) };
            let index = item.index;
            self.change_is_opacity( opacity, text_style, index, write);
        }
    }
}

impl<'a, C: Context + Share> CharBlockSys<C> {
    fn change_is_opacity(&mut self, opacity: f32, text_style: &TextStyle, index: usize, render_objs: &mut SingleCaseImpl<RenderObjs<C>>){
        let is_opacity = if opacity < 1.0 || !text_style.color.is_opaque() || text_style.stroke.color.a < 1.0{
            false
        }else {
            true
        };

        let notify = render_objs.get_notify();
        unsafe { render_objs.get_unchecked_write(index, &notify).set_is_opacity(is_opacity)};
    }
}

struct Item {
    index: usize,
    position_change: bool,
}

fn modify_stroke<C: Context + Share>(item: &mut Item, text_style: &TextStyle, render_objs: &mut SingleCaseImpl<RenderObjs<C>>, engine: &mut SingleCaseImpl<Engine<C>>) {
    let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
    if text_style.stroke.width == 0.0 {
        //  删除边框的宏
        match render_obj.defines.remove_item(&STROKE) {
            Some(_) => {
                // 如果边框宏存在， 删除边框对应的ubo， 重新创建渲染管线
                render_obj.ubos.remove(&STROKE);
                let old_pipeline = render_obj.pipeline.clone();
                render_obj.pipeline = engine.create_pipeline(
                    0,
                    &TEXT_VS_SHADER_NAME.clone(),
                    &TEXT_FS_SHADER_NAME.clone(),
                    render_obj.defines.as_slice(),
                    old_pipeline.rs.clone(),
                    old_pipeline.bs.clone(),
                    old_pipeline.ss.clone(),
                    old_pipeline.ds.clone()
                );
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
            render_obj.ubos.insert(STROKE.clone(), Arc::new(stroke_ubo));
            let old_pipeline = render_obj.pipeline.clone();
            render_obj.pipeline = engine.create_pipeline(
                0,
                &TEXT_VS_SHADER_NAME.clone(),
                &TEXT_FS_SHADER_NAME.clone(),
                render_obj.defines.as_slice(),
                old_pipeline.rs.clone(),
                old_pipeline.bs.clone(),
                old_pipeline.ss.clone(),
                old_pipeline.ds.clone()
            );
        }
    }   
}

fn modify_color<C: Context + Share>(geometry_dirtys: &mut Vec<usize>, item: &mut Item, id: usize, text_style: &TextStyle, render_objs: &mut SingleCaseImpl<RenderObjs<C>>, engine: &mut SingleCaseImpl<Engine<C>>) {
    let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
    match &text_style.color {
        Color::RGBA(c) => {
            // 如果未找到UCOLOR宏， 表示修改之前的颜色不为RGBA， 应该删除VERTEX_COLOR宏， 添加UCOLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
            if find_item_from_vec(&render_obj.defines, &UCOLOR) == 0 {
                render_obj.defines.remove_item(&VERTEX_COLOR);

                let ucolor_ubo = engine.gl.create_uniforms();
                render_obj.ubos.insert(UCOLOR.clone(), Arc::new(ucolor_ubo));
                render_obj.defines.push(UCOLOR.clone());

                if item.position_change == false {
                    item.position_change = true;
                    geometry_dirtys.push(id);
                }
            }
            // 设置ubo
            let ucolor_ubo = Arc::make_mut(render_obj.ubos.get_mut(&UCOLOR).unwrap());
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

// 返回position， uv， color， index
fn get_geo_flow<C: Context + Share>(
    char_block: &CharBlock,
    sdf_font: &Arc<SdfFont<Ctx = C>>,
    color: &Color,
    z_depth: f32,
    offset: (f32, f32)
) -> (Vec<f32>, Vec<f32>, Option<Vec<f32>>, Vec<u16>) {
    let mut positions: Vec<f32> = Vec::new();
    let mut uvs: Vec<f32> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    let font_size = char_block.font_size;
    let mut i = 0;
    // let line_height = sdf_font.line_height;

    println!("char_block---------------------------- {:?}", char_block);
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

                println!("text, Linear start: {:?}, end: {:?}, direction: {:?}, endp:[({}, {}), ({}, {})]", start, end, color.direction, (endp.0).0, (endp.0).1, (endp.1).0, (endp.1).1);

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
                    
                    println!("text, split_by_lg, positions: {:?}, indices:{:?}, lg_pos{:?}", positions, [i, i + 1, i + 2, i + 3], lg_pos.as_slice());
                    let (ps, indices_arr) = split_by_lg(
                        positions,
                        vec![i, i + 1, i + 2, i + 3],
                        lg_pos.as_slice(),
                        endp.0.clone(),
                        endp.1.clone(),
                    );
                    positions = ps;

                    println!("text, fill_uv, positions: {:?}, uvs:{:?}, i: {}", &positions, &uvs, i);
                    // 尝试为新增的点计算uv
                    fill_uv(&mut positions, &mut uvs, i as usize);

                    println!("text, interp_mult_by_lg, positions: {:?}, indices_arr:{:?}, lg_color: {:?}, lg_pos: {:?}", &positions, &indices_arr, lg_color, lg_pos);
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

fn cal_all_size<C: Context + Share>(char_block: &CharBlock, font_size: f32, sdf_font: &Arc<SdfFont<Ctx = C>>,) -> (Point2, Point2) {
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
// fn get_geo_flow(char_block: &CharBlock, color: &Color, layout: &Layout, z_depth: f32) -> (Vec<f32>, Option<Vec<f32>>, Vec<u16>) {
//     unimplemented!()
    

//     // (positions, uvs, indices)
// }

unsafe impl<C: Context + Share> Sync for CharBlockSys<C>{}
unsafe impl<C: Context + Share> Send for CharBlockSys<C>{}

impl_system!{
    CharBlockSys<C> where [C: Context + Share],
    true,
    {
        MultiCaseListener<Node, CharBlock, CreateEvent>
        MultiCaseListener<Node, CharBlock, ModifyEvent>
        MultiCaseListener<Node, CharBlock, DeleteEvent>
        MultiCaseListener<Node, TextStyle, ModifyEvent>
        MultiCaseListener<Node, Font, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
    }
}