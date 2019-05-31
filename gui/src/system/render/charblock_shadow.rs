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
    static ref UCOLOR: Atom = Atom::from("UCOLOR");

    static ref U_COLOR: Atom = Atom::from("uColor");
}

pub struct CharBlockShadowSys<C: Context + Share>{
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

impl<C: Context + Share> CharBlockShadowSys<C> {
    pub fn new() -> Self{
        CharBlockShadowSys {
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
impl<'a, C: Context + Share> Runner<'a> for CharBlockShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, TextShadow>,
        &'a MultiCaseImpl<Node, CharBlock>,
        &'a MultiCaseImpl<Node, Font>,
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a SingleCaseImpl<DefaultTable>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let map = &mut self.charblock_render_map;
        let (z_depths, text_shadows, charblocks, fonts, font_sheet, default_table) = read;
        let (render_objs, _) = write;
        for id in  self.geometry_dirtys.iter() {
            let item = unsafe { map.get_unchecked_mut(*id) };
            item.position_change = false;
            let z_depth = unsafe { z_depths.get_unchecked(*id) }.0;
            let charblock = unsafe { charblocks.get_unchecked(*id) };
            let text_shadow = unsafe { text_shadows.get_unchecked(*id) };
            let font = get_or_default(*id, fonts, default_table);
            let first_font = match font_sheet.get_first_font(&font.family) {
                Some(r) => r,
                None => {
                    debug_println!("font is not exist: {}", font.family.as_str());
                    return;
                }
            };
            let (positions, uvs, indices) = get_geo_flow(charblock, &first_font, z_depth + 0.1, (text_shadow.h, text_shadow.v));

            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
            let geometry = unsafe {&mut *(render_obj.geometry.as_ref() as *const C::ContextGeometry as usize as *mut C::ContextGeometry)};

            let vertex_count: u32 = (positions.len()/3) as u32;
            if vertex_count != geometry.get_vertex_count() {
                geometry.set_vertex_count(vertex_count);
            }
            geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false).unwrap();
            geometry.set_attribute(&AttributeName::UV0, 2, Some(uvs.as_slice()), false).unwrap();
            geometry.set_indices_short(indices.as_slice(), false).unwrap();
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
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock, CreateEvent> for CharBlockShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, TextShadow>,
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
        let (text_shadows, fonts, z_depths, opacitys, font_sheet, default_table) = read;
        let (render_objs, engine) = write;
        let z_depth = unsafe { z_depths.get_unchecked(event.id) }.0;
        let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
        let text_shadow = unsafe { text_shadows.get_unchecked(event.id) };
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

        let c = &text_shadow.color;
        let mut ucolor_ubo = engine.gl.create_uniforms();
        ucolor_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b, c.a);
        ubos.insert(UCOLOR.clone(), Arc::new(ucolor_ubo));
        defines.push(UCOLOR.clone());
        debug_println!("text_shadow, id: {}, color: {:?}", event.id, c);

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
        
        let is_opacity = if opacity < 1.0 || text_shadow.color.a < 1.0{
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
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock, ModifyEvent> for CharBlockShadowSys<C>{
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
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock, DeleteEvent> for CharBlockShadowSys<C>{
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
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Font, ModifyEvent> for CharBlockShadowSys<C>{
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

// TextShadow修改， 设置对应的ubo和宏
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, TextShadow, CreateEvent> for CharBlockShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, TextShadow>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (opacitys, text_shadows) = read;   
        if let Some(item) = self.charblock_render_map.get_mut(event.id) {
            let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
            let text_shadow = unsafe { text_shadows.get_unchecked(event.id) };
            modify_color(item, event.id, text_shadow, render_objs);
            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(event.id);
            }
            let index = item.index;
            self.change_is_opacity(opacity, text_shadow, index, render_objs);
        }
    }
}

// TextShadow修改， 设置对应的ubo和宏
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, TextShadow, DeleteEvent> for CharBlockShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, TextShadow>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (opacitys, text_shadows) = read;   
        if let Some(item) = self.charblock_render_map.get_mut(event.id) {
            let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
            let text_shadow = unsafe { text_shadows.get_unchecked(event.id) };
            modify_color(item, event.id, text_shadow, render_objs);
            let index = item.index;
            self.change_is_opacity(opacity, text_shadow, index, render_objs);
        }
    }
}

// TextShadow修改， 设置对应的ubo和宏
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, TextShadow, ModifyEvent> for CharBlockShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, TextShadow>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (opacitys, text_shadows) = read;   
        println!("modify style {:?}, {:?}", event.field, unsafe { text_shadows.get_unchecked(event.id) });
        if let Some(item) = self.charblock_render_map.get_mut(event.id) {
            let text_shadow = unsafe { text_shadows.get_unchecked(event.id) };
            
            match event.field {
                "color" => {
                    modify_color(item, event.id, text_shadow, render_objs);
                     let opacity = unsafe { opacitys.get_unchecked(event.id) }.0;
                    let index = item.index;
                    self.change_is_opacity( opacity, text_shadow, index, render_objs);
                },
                "blur" => (),
                "h" | "v" => {
                    if item.position_change == false {
                        item.position_change = true;
                        self.geometry_dirtys.push(event.id);
                    }
                },
                _ => return,
            }
        }
    }
}

//不透明度变化， 修改渲染对象的is_opacity属性
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for CharBlockShadowSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, Opacity>, &'a MultiCaseImpl<Node, TextShadow>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (opacitys, text_shadows) = read;
        if let Some(item) = self.charblock_render_map.get(event.id) {
            let opacity = unsafe { opacitys.get_unchecked(event.id).0 };
            let text_shadow = unsafe { text_shadows.get_unchecked(event.id) };
            let index = item.index;
            self.change_is_opacity( opacity, text_shadow, index, write);
        }
    }
}

impl<'a, C: Context + Share> CharBlockShadowSys<C> {
    fn change_is_opacity(&mut self, opacity: f32, text_shadow: &TextShadow, index: usize, render_objs: &mut SingleCaseImpl<RenderObjs<C>>){
        let is_opacity = if opacity < 1.0 || text_shadow.color.a < 1.0{
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

fn modify_color<C: Context + Share>(item: &mut Item, id: usize, text_shadow: &TextShadow, render_objs: &mut SingleCaseImpl<RenderObjs<C>>) {
    let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
    // 设置ubo
    let c = &text_shadow.color;
    let ucolor_ubo = Arc::make_mut(render_obj.ubos.get_mut(&UCOLOR).unwrap());
    debug_println!("text_shadow_color, id: {}, color: {:?}", id, c);
    ucolor_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b, c.a);
    render_objs.get_notify().modify_event(item.index, "", 0);
}

// 返回position， uv， color， index
fn get_geo_flow<C: Context + Share>(
    char_block: &CharBlock,
    sdf_font: &Arc<SdfFont<Ctx = C>>,
    z_depth: f32,
    offset: (f32, f32)
) -> (Vec<f32>, Vec<f32>, Vec<u16>) {
    let mut positions: Vec<f32> = Vec::new();
    let mut uvs: Vec<f32> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    let font_size = char_block.font_size;
    let mut i = 0;
    // let line_height = sdf_font.line_height;

    if char_block.chars.len() > 0 {
        for c in char_block.chars.iter() {
            let glyph = match sdf_font.glyph_info(c.ch, font_size) {
                Some(r) => r,
                None => continue,
            };
            push_pos_uv(&mut positions, &mut uvs, &c.pos, &offset, &glyph, z_depth);
            // let (v_min, v_max) = (1.0 - glyph.v_max, 1.0 - glyph.v_min);
            indices.extend_from_slice(&[4 * i + 0, 4 * i + 1, 4 * i + 2, 4 * i + 0, 4 * i + 2, 4 * i + 3]);
            i += 1;  
        }
        return (positions, uvs, indices);
    } else {
        return (positions, uvs, indices);
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
    positions.extend_from_slice(&ps[0..16]);
}

// //取几何体的顶点流、 颜色流和属性流
// fn get_geo_flow(char_block: &CharBlock, color: &Color, layout: &Layout, z_depth: f32) -> (Vec<f32>, Option<Vec<f32>>, Vec<u16>) {
//     unimplemented!()
    

//     // (positions, uvs, indices)
// }

unsafe impl<C: Context + Share> Sync for CharBlockShadowSys<C>{}
unsafe impl<C: Context + Share> Send for CharBlockShadowSys<C>{}

impl_system!{
    CharBlockShadowSys<C> where [C: Context + Share],
    true,
    {
        MultiCaseListener<Node, CharBlock, CreateEvent>
        MultiCaseListener<Node, CharBlock, ModifyEvent>
        MultiCaseListener<Node, CharBlock, DeleteEvent>
        MultiCaseListener<Node, TextShadow, CreateEvent>
        MultiCaseListener<Node, TextShadow, ModifyEvent>
        MultiCaseListener<Node, TextShadow, DeleteEvent>
        MultiCaseListener<Node, Font, ModifyEvent>
        MultiCaseListener<Node, Opacity, ModifyEvent>
    }
}