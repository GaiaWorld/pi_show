/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use share::Share;

use fnv::FnvHashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
use map::{ vecmap::VecMap } ;
use hal_core::*;
use atom::Atom;

use component::user::*;
use single::*;
use component::calc::{Opacity, ZDepth, CharBlock, WorldMatrixRender};
use entity::{Node};
use render::engine::{ Engine };
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
    static ref UCOLOR: Atom = Atom::from("UCOLOR");
    static ref STROKE_COLOR: Atom = Atom::from("strokeColor");

    static ref U_COLOR: Atom = Atom::from("uColor");
}

pub struct CharBlockShadowSys<C: Context, L: FlexNode>{
    render_map: VecMap<Item>,
    geometry_dirtys: Vec<usize>,
    mark: PhantomData<(C, L)>,
    rs: Share<RasterState>,
    bs: Share<BlendState>,
    ss: Share<StencilState>,
    ds: Share<DepthState>,
    canvas_bs: Share<BlendState>,
    default_sampler: Option<Res<SamplerRes<C>>>,
}

impl<C: Context, L: FlexNode> CharBlockShadowSys<C, L> {
    pub fn new() -> Self{
        let mut bs = BlendState::new();
        let mut canvas_bs = BlendState::new();
        let mut ds = DepthState::new();
        bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        canvas_bs.set_rgb_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);
        ds.set_write_enable(false);
        CharBlockShadowSys {
            render_map: VecMap::default(),
            geometry_dirtys: Vec::new(),
            mark: PhantomData,
            rs: Share::new(RasterState::new()),
            bs: Share::new(bs),
            ss: Share::new(StencilState::new()),
            ds: Share::new(ds),
            canvas_bs: Share::new(canvas_bs),
            default_sampler: None,
        }
    }

    pub fn create_render(
        &mut self,
        id: usize,
        read: (
            & MultiCaseImpl<Node, TextShadow>,
            & MultiCaseImpl<Node, Font>,
            & MultiCaseImpl<Node, ZDepth>,
            & MultiCaseImpl<Node, Opacity>,
            & MultiCaseImpl<Node, CharBlock<L>>,
            & SingleCaseImpl<FontSheet<C>>,
            & SingleCaseImpl<DefaultTable>,
        ),
        write: (
            & mut SingleCaseImpl<RenderObjs<C>>,
            & mut SingleCaseImpl<Engine<C>>,
        ),
    ){
        let (text_shadows, fonts, z_depths, opacitys, charblocks, font_sheet, default_table) = read;
        let (render_objs, engine) = write;

        let z_depth = unsafe { z_depths.get_unchecked(id) }.0;
        let _opacity = unsafe { opacitys.get_unchecked(id) }.0;

        let text_shadow = match text_shadows.get(id) {
            Some(r) => r,
            None => return,
        };
        if let None = charblocks.get(id) {
            return;
        }

        let font = get_or_default(id, fonts, default_table);
        let mut defines = Vec::new();

        let mut ubos: FnvHashMap<Atom, Share<Uniforms<C>>> = FnvHashMap::default();

        let mut common_ubo = engine.gl.create_uniforms();  
        let dyn_type = match font_sheet.get_first_font(&font.family) {
            Some(r) => {
                common_ubo.set_sampler(
                    &TEXTURE,
                    &(self.default_sampler.as_ref().unwrap().value.clone() as Share<dyn AsRef<<C as Context>::ContextSampler>>),
                    &(r.texture().value.clone() as Share<dyn AsRef<<C as Context>::ContextTexture>>)
                );
                r.get_dyn_type()
            },
            None => { debug_println!("font is not exist: {}", font.family.as_str()); 0 },
        };
        ubos.insert(COMMON.clone(), Share::new(common_ubo)); // COMMON
        let c = &text_shadow.color;
        let mut ucolor_ubo = engine.gl.create_uniforms();
        
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
        } else {
            ucolor_ubo.set_float_4(&STROKE_COLOR, c.r, c.g, c.b, c.a);
            pipeline = engine.create_pipeline(
                1,
                &CANVAS_TEXT_VS_SHADER_NAME.clone(),
                &CANVAS_TEXT_FS_SHADER_NAME.clone(),
                defines.as_slice(),
                self.rs.clone(),
                self.bs.clone(),
                self.ss.clone(),
                self.ds.clone()
            );
        }

        ucolor_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b, c.a);
        ubos.insert(UCOLOR.clone(), Share::new(ucolor_ubo));
        defines.push(UCOLOR.clone());
        debug_println!("text_shadow, id: {}, color: {:?}", id, c);
        
        let render_obj: RenderObj<C> = RenderObj {
            depth: z_depth + 0.1,
            depth_diff: 0.1,
            visibility: false,
            is_opacity: false,
            ubos: ubos,
            geometry: None,
            pipeline: pipeline.clone(),
            context: id,
            defines: defines,
        };

        let notify = render_objs.get_notify();
        let index = render_objs.insert(render_obj, Some(notify));
        self.render_map.insert(id, Item{index: index, position_change: true});
        self.geometry_dirtys.push(id);
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: Context, L: FlexNode> Runner<'a> for CharBlockShadowSys<C, L>{
    type ReadData = (
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, TextShadow>,
        &'a MultiCaseImpl<Node, CharBlock<L>>,
        &'a MultiCaseImpl<Node, Font>,
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a SingleCaseImpl<DefaultTable>,
        &'a MultiCaseImpl<Node, WorldMatrixRender>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let (z_depths, text_shadows, charblocks, fonts, font_sheet, default_table, world_matrixs, ) = read;
        let (render_objs, engine) = write;
        for id in  self.geometry_dirtys.iter() {
            let map = &mut self.render_map;
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
            let (positions, uvs, indices) = get_geo_flow(charblock, &first_font, z_depth + 0.2, (text_shadow.h, text_shadow.v));

            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
            if positions.len() == 0 {
                render_obj.geometry = None;
            } else {
                let mut geometry = create_geometry(&mut engine.gl);
                geometry.set_vertex_count((positions.len()/3) as u32);
                geometry.set_attribute(&AttributeName::Position, 3, Some(positions.as_slice()), false).unwrap();
                geometry.set_attribute(&AttributeName::UV0, 2, Some(uvs.as_slice()), false).unwrap();
                geometry.set_indices_short(indices.as_slice(), false).unwrap();
                render_obj.geometry = Some(Res::new(500, Share::new(GeometryRes{name: 0, bind: geometry})));
            };
            render_objs.get_notify().modify_event(item.index, "geometry", 0);

            self.modify_matrix(*id, world_matrixs, text_shadows, render_objs);
        }
        self.geometry_dirtys.clear();
    }

    fn setup(&mut self, _: Self::ReadData, write: Self::WriteData){
        let (_, engine) = write;
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
impl<'a, C: Context, L: FlexNode> MultiCaseListener<'a, Node, CharBlock<L>, CreateEvent> for CharBlockShadowSys<C, L>{
    type ReadData = (
        &'a MultiCaseImpl<Node, TextShadow>,
        &'a MultiCaseImpl<Node, Font>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, CharBlock<L>>,
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a SingleCaseImpl<DefaultTable>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs<C>>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        self.create_render(event.id, read, write);
    }
}


// 字体修改， 设置顶点数据脏
impl<'a, C: Context, L: FlexNode> MultiCaseListener<'a, Node, CharBlock<L>, ModifyEvent> for CharBlockShadowSys<C, L>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        if let Some(item) = self.render_map.get_mut(event.id){
            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(event.id);
            }
        }
    }
}


// 删除渲染对象
impl<'a, C: Context, L: FlexNode> MultiCaseListener<'a, Node, CharBlock<L>, DeleteEvent> for CharBlockShadowSys<C, L>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, write: Self::WriteData){
        if let Some(item) = self.render_map.remove(event.id){
            let notify = write.get_notify();
            write.remove(item.index, Some(notify));
            if item.position_change == true {
                self.geometry_dirtys.remove_item(&event.id);
            }
        }
    }
}

// 字体修改， 重新设置字体纹理
impl<'a, C: Context, L: FlexNode> MultiCaseListener<'a, Node, Font, ModifyEvent> for CharBlockShadowSys<C, L>{
    type ReadData = (
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a MultiCaseImpl<Node, Font>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (font_sheet, fonts) = read;
        if let Some(item) = self.render_map.get_mut(event.id) {
            modify_font(event.id, item, self.default_sampler.clone().unwrap(), font_sheet, fonts, write.0, write.1, &self.rs,  &self.bs,  &self.ss,  &self.ds, &self.canvas_bs);
            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(event.id);
            }
            write.0.get_notify().modify_event(event.id, "pipeline", 0);
        }
    }
}

// 字体修改， 重新设置字体纹理
impl<'a, C: Context, L: FlexNode> MultiCaseListener<'a, Node, Font, CreateEvent> for CharBlockShadowSys<C, L>{
    type ReadData = (
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a MultiCaseImpl<Node, Font>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (font_sheet, fonts) = read;
        if let Some(item) = self.render_map.get_mut(event.id) {
            modify_font(event.id, item, self.default_sampler.clone().unwrap(), font_sheet, fonts, write.0, write.1, &self.rs,  &self.bs,  &self.ss,  &self.ds,  &self.canvas_bs);
            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(event.id);
            }
            write.0.get_notify().modify_event(event.id, "pipeline", 0);
        }
    }
}

// 插入渲染对象
impl<'a, C: Context, L: FlexNode> MultiCaseListener<'a, Node, TextShadow, CreateEvent> for CharBlockShadowSys<C, L>{
    type ReadData = (
        &'a MultiCaseImpl<Node, TextShadow>,
        &'a MultiCaseImpl<Node, Font>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, CharBlock<L>>,
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a SingleCaseImpl<DefaultTable>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs<C>>,
        &'a mut SingleCaseImpl<Engine<C>>,
    );
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        self.create_render(event.id, read, write);
    }
}
// 删除渲染对象
impl<'a, C: Context, L: FlexNode> MultiCaseListener<'a, Node, TextShadow, DeleteEvent> for CharBlockShadowSys<C, L>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, write: Self::WriteData){
        if let Some(item) = self.render_map.remove(event.id){
            let notify = write.get_notify();
            write.remove(item.index, Some(notify));
            if item.position_change == true {
                self.geometry_dirtys.remove_item(&event.id);
            }
        }
    }
}

// TextShadow修改， 设置对应的ubo和宏
impl<'a, C: Context, L: FlexNode> MultiCaseListener<'a, Node, TextShadow, ModifyEvent> for CharBlockShadowSys<C, L>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, TextShadow>,
        &'a MultiCaseImpl<Node, Font>,
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a SingleCaseImpl<DefaultTable>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (_opacitys, text_shadows, fonts, font_sheet, default_table) = read;   
        if let Some(item) = self.render_map.get_mut(event.id) {
            let text_shadow = unsafe { text_shadows.get_unchecked(event.id) };
            
            match event.field {
                "color" => {
                    let font = get_or_default(event.id, fonts, default_table);
                    modify_color(item, event.id, text_shadow, render_objs, font, font_sheet);
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

type MatrixRead<'a> = (
    &'a MultiCaseImpl<Node, WorldMatrixRender>,
    &'a MultiCaseImpl<Node, TextShadow>
);

impl<'a, C: Context, L: FlexNode> MultiCaseListener<'a, Node, WorldMatrixRender, ModifyEvent> for CharBlockShadowSys<C, L>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read.0, read.1, render_objs);
    }
}

impl<'a, C: Context, L: FlexNode> MultiCaseListener<'a, Node, WorldMatrixRender, CreateEvent> for CharBlockShadowSys<C, L>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read.0, read.1, render_objs);
    }
}

impl<'a, C: Context, L: FlexNode> CharBlockShadowSys<C, L>{
    fn modify_matrix(
        &self,
        id: usize,
        world_matrixs: &MultiCaseImpl<Node, WorldMatrixRender>,
        text_shadows: &MultiCaseImpl<Node, TextShadow>,
        render_objs: &mut SingleCaseImpl<RenderObjs<C>>
    ){
        if let Some(item) = self.render_map.get(id) {
            let world_matrix = unsafe { world_matrixs.get_unchecked(id) };
            let text_shadow = unsafe { text_shadows.get_unchecked(id) };
            let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };

            let world_matrix = world_matrix.0 * Matrix4::from_translation(Vector3::new(text_shadow.h, text_shadow.v, 1.0));
            // 渲染物件的顶点不是一个四边形， 保持其原有的矩阵
            let ubos = &mut render_obj.ubos;
            let slice: &[f32; 16] = world_matrix.as_ref();
            Share::make_mut(ubos.get_mut(&WORLD).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
            debug_println!("charblock_shadow, id: {}, world_matrix: {:?}", render_obj.context, &slice[0..16]);
            render_objs.get_notify().modify_event(item.index, "ubos", 0);
        }
    }
}

struct Item {
    index: usize,
    position_change: bool,
}

fn modify_font<C: Context> (
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
    common_ubo.set_sampler(
        &TEXTURE,
        &(default_sampler.value.clone() as Share<dyn AsRef<<C as Context>::ContextSampler>>),
        &(first_font.texture().value.clone() as Share<dyn AsRef<<C as Context>::ContextTexture>>)
    );

    if first_font.get_dyn_type() == 0 {
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
            ds.clone()
        );
    }
}

fn modify_color<C: Context>(
    item: &mut Item,
    id: usize,
    text_shadow: &TextShadow,
    render_objs: &mut SingleCaseImpl<RenderObjs<C>>,
    font: &Font,
    font_sheet: &SingleCaseImpl<FontSheet<C>>,
) {
    let render_obj = unsafe { render_objs.get_unchecked_mut(item.index) };
    let c = &text_shadow.color;
    let ucolor_ubo = Share::make_mut(render_obj.ubos.get_mut(&UCOLOR).unwrap());

    let first_font = match font_sheet.get_first_font(&font.family) {
        Some(r) => r,
        None => {
            debug_println!("font is not exist: {}", font.family.as_ref());
            return;
        }
    };

    if first_font.get_dyn_type() > 0 {
        ucolor_ubo.set_float_4(&STROKE_COLOR, c.r, c.g, c.b, c.a);
    }
    // 设置ubo
   
    debug_println!("text_shadow_color, id: {}, color: {:?}", id, c);
    ucolor_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b, c.a);
    render_objs.get_notify().modify_event(item.index, "", 0);
}

// 返回position， uv， color， index
fn get_geo_flow<C: Context, L: FlexNode>(
    char_block: &CharBlock<L>,
    sdf_font: &Share<dyn SdfFont<Ctx = C>>,
    z_depth: f32,
    mut offset: (f32, f32)
) -> (Vec<f32>, Vec<f32>, Vec<u16>) {
    let mut positions: Vec<f32> = Vec::new();
    let mut uvs: Vec<f32> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    let font_size = char_block.font_size;
    let mut i = 0;
    offset.1 += (char_block.line_height - font_size)/2.0;

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

unsafe impl<C: Context, L: FlexNode> Sync for CharBlockShadowSys<C, L>{}
unsafe impl<C: Context, L: FlexNode> Send for CharBlockShadowSys<C, L>{}

impl_system!{
    CharBlockShadowSys<C, L> where [C: Context, L: FlexNode],
    true,
    {
        MultiCaseListener<Node, CharBlock<L>, CreateEvent>
        MultiCaseListener<Node, CharBlock<L>, ModifyEvent>
        MultiCaseListener<Node, CharBlock<L>, DeleteEvent>
        MultiCaseListener<Node, TextShadow, CreateEvent>
        MultiCaseListener<Node, TextShadow, ModifyEvent>
        MultiCaseListener<Node, TextShadow, DeleteEvent>
        MultiCaseListener<Node, Font, ModifyEvent>
        MultiCaseListener<Node, WorldMatrixRender, CreateEvent>
        MultiCaseListener<Node, WorldMatrixRender, ModifyEvent>
    }
}