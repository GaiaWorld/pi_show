/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use std::sync::Arc;

use fnv::FnvHashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use map::{ vecmap::VecMap } ;
use hal_core::*;
use atom::Atom;

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
use font::font_sheet::FontSheet;
use font::sdf_font:: {GlyphInfo, SdfFont };
use util::res_mgr::Res;


lazy_static! {
    static ref UCOLOR: Atom = Atom::from("UCOLOR");

    static ref U_COLOR: Atom = Atom::from("uColor");
}

pub struct CharBlockShadowSys<C: Context + Share>{
    render_map: VecMap<Item>,
    geometry_dirtys: Vec<usize>,
    mark: PhantomData<C>,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
    pipelines: FnvHashMap<u64, Arc<PipelineInfo>>,
    default_sampler: Option<Res<SamplerRes<C>>>,
}

impl<C: Context + Share> CharBlockShadowSys<C> {
    pub fn new() -> Self{
        let mut bs = BlendState::new();
        let mut ds = DepthState::new();
        bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        ds.set_write_enable(false);
        CharBlockShadowSys {
            render_map: VecMap::default(),
            geometry_dirtys: Vec::new(),
            mark: PhantomData,
            rs: Arc::new(RasterState::new()),
            bs: Arc::new(bs),
            ss: Arc::new(StencilState::new()),
            ds: Arc::new(ds),
            pipelines: FnvHashMap::default(),
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
            & MultiCaseImpl<Node, CharBlock>,
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

        let mut ubos: FnvHashMap<Atom, Arc<Uniforms<C>>> = FnvHashMap::default();

        let mut common_ubo = engine.gl.create_uniforms();  
        match font_sheet.get_first_font(&font.family) {
            Some(r) => {
                common_ubo.set_sampler(
                    &TEXTURE,
                    &(self.default_sampler.as_ref().unwrap().value.clone() as Arc<dyn AsRef<<C as Context>::ContextSampler>>),
                    &(r.texture().value.clone() as Arc<dyn AsRef<<C as Context>::ContextTexture>>)
                );
            },
            None => debug_println!("font is not exist: {}", font.family.as_str()),
        };
        ubos.insert(COMMON.clone(), Arc::new(common_ubo)); // COMMON

        let c = &text_shadow.color;
        let mut ucolor_ubo = engine.gl.create_uniforms();
        ucolor_ubo.set_float_4(&U_COLOR, c.r, c.g, c.b, c.a);
        ubos.insert(UCOLOR.clone(), Arc::new(ucolor_ubo));
        defines.push(UCOLOR.clone());
        debug_println!("text_shadow, id: {}, color: {:?}", id, c);

        let pipeline = engine.create_pipeline(
            1,
            &TEXT_VS_SHADER_NAME.clone(),
            &TEXT_FS_SHADER_NAME.clone(),
            defines.as_slice(),
            self.rs.clone(),
            self.bs.clone(),
            self.ss.clone(),
            self.ds.clone()
        );
        
        // let is_opacity = if opacity < 1.0 || text_shadow.color.a < 1.0{
        //     false
        // }else {
        //     true
        // };
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
        let map = &mut self.render_map;
        let (z_depths, text_shadows, charblocks, fonts, font_sheet, default_table) = read;
        let (render_objs, engine) = write;
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
                render_obj.geometry = Some(Res::new(500, Arc::new(GeometryRes{name: 0, bind: geometry})));
            };
            render_objs.get_notify().modify_event(item.index, "geometry", 0);
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
                let res = SamplerRes::new(hash, engine.gl.create_sampler(Arc::new(s)).unwrap());
                self.default_sampler = Some(engine.res_mgr.create::<SamplerRes<C>>(res));
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
        &'a MultiCaseImpl<Node, CharBlock>,
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
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock, ModifyEvent> for CharBlockShadowSys<C>{
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
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, CharBlock, DeleteEvent> for CharBlockShadowSys<C>{
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
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, Font, ModifyEvent> for CharBlockShadowSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<FontSheet<C>>,
        &'a MultiCaseImpl<Node, Font>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (font_sheet, fonts) = read;
        if let Some(item) = self.render_map.get_mut(event.id) {
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
                &(self.default_sampler.as_ref().unwrap().value.clone() as Arc<dyn AsRef<<C as Context>::ContextSampler>>),
                &(first_font.texture().value.clone() as Arc<dyn AsRef<<C as Context>::ContextTexture>>)
            );

            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(event.id);
            }
        }
    }
}

// 插入渲染对象
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, TextShadow, CreateEvent> for CharBlockShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, TextShadow>,
        &'a MultiCaseImpl<Node, Font>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, CharBlock>,
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
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, TextShadow, DeleteEvent> for CharBlockShadowSys<C>{
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
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, TextShadow, ModifyEvent> for CharBlockShadowSys<C>{
    type ReadData = (
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, TextShadow>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (_opacitys, text_shadows) = read;   
        if let Some(item) = self.render_map.get_mut(event.id) {
            let text_shadow = unsafe { text_shadows.get_unchecked(event.id) };
            
            match event.field {
                "color" => {
                    modify_color(item, event.id, text_shadow, render_objs);
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

impl<'a, C: Context + Share> MultiCaseListener<'a, Node, WorldMatrixRender, ModifyEvent> for CharBlockShadowSys<C>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read.0, read.1, render_objs);
    }
}

impl<'a, C: Context + Share> MultiCaseListener<'a, Node, WorldMatrixRender, CreateEvent> for CharBlockShadowSys<C>{
    type ReadData = MatrixRead<'a>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, render_objs: Self::WriteData){
        self.modify_matrix(event.id, read.0, read.1, render_objs);
    }
}

impl<'a, C: Context + Share> CharBlockShadowSys<C>{
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
            Arc::make_mut(ubos.get_mut(&WORLD).unwrap()).set_mat_4v(&WORLD_MATRIX, &slice[0..16]);
            debug_println!("charblock_shadow, id: {}, world_matrix: {:?}", render_obj.context, &slice[0..16]);
            render_objs.get_notify().modify_event(item.index, "ubos", 0);
        }
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
    sdf_font: &Arc<dyn SdfFont<Ctx = C>>,
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
        MultiCaseListener<Node, WorldMatrixRender, CreateEvent>
        MultiCaseListener<Node, WorldMatrixRender, ModifyEvent>
    }
}