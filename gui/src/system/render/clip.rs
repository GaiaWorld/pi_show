// 监听Overflow， 绘制裁剪纹理

use std::marker::PhantomData;
use std::sync::Arc;

use fnv::FnvHashMap;
use ecs::{ModifyEvent, CreateEvent, MultiCaseListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use hal_core::*;
use atom::Atom;

use component::calc::{ByOverflow};
use entity::{Node};
use single::*;
use render::engine:: { Engine, PipelineInfo };
use render::res::{SamplerRes};
use system::util::*;
use system::util::constant::*;
use system::render::shaders::clip::*;
use util::res_mgr::Res;

lazy_static! {
    static ref MESH_NUM: Atom = Atom::from("meshNum");
    // static ref MESH_INDEX: Atom = Atom::from("meshIndex");
    static ref CLIP_RENDER: Atom = Atom::from("clip_render");

    static ref COMMON: Atom = Atom::from("common");
    
    static ref CLIP_TEXTURE: Atom = Atom::from("clipTexture");
    static ref CLIP_INDICES: Atom = Atom::from("clipIndices");
    static ref CLIP_TEXTURE_SIZE: Atom = Atom::from("clipTextureSize");
}

pub struct ClipSys<C: Context + Share>{
    mark: PhantomData<C>,
    dirty: bool,
    render_target: Arc<<C as Context>::ContextRenderTarget>,
    by_ubo: Arc<Uniforms<C>>,
    rs: Arc<RasterState>,
    bs: Arc<BlendState>,
    ss: Arc<StencilState>,
    ds: Arc<DepthState>,
    pipeline: Arc<PipelineInfo>,
    geometry: Arc<<C as Context>::ContextGeometry>,
    ubos: FnvHashMap<Atom, Arc<Uniforms<C>>>,
    sampler: Res<SamplerRes<C>>,
}

impl<C: Context + Share> ClipSys<C>{
    pub fn new(engine: &mut Engine<C>, w: u32, h: u32) -> Self{
        let (rs, mut bs, ss, mut ds) = (Arc::new(RasterState::new()), BlendState::new(), Arc::new(StencilState::new()), DepthState::new());
        
        bs.set_rgb_factor(BlendFactor::One, BlendFactor::One);
        let bs = Arc::new(bs);

        ds.set_test_enable(false);
        ds.set_write_enable(false);
        let ds = Arc::new(ds);

        let defines = Vec::new();
        let pipeline = engine.create_pipeline(
            0,
            &CLIP_VS_SHADER_NAME,
            &CLIP_FS_SHADER_NAME,
            defines.as_slice(),
            rs.clone(),
            bs.clone(),
            ss.clone(),
            ds.clone(),
        );
        let s = SamplerDesc::default();
        let hash = sampler_desc_hash(&s);
        let sampler = match engine.res_mgr.get::<SamplerRes<C>>(&hash) {
            Some(r) => r.clone(),
            None => {
                let res = SamplerRes::new(hash, engine.gl.create_sampler(Arc::new(s)).unwrap());
                engine.res_mgr.create::<SamplerRes<C>>(res)
            },          
        };
        let size = next_power_of_two(w.max(h));
        let target = engine.gl.create_render_target(size, size, &PixelFormat::RGB, &DataFormat::UnsignedByte, false).unwrap();
        let mut by_ubo = engine.gl.create_uniforms();
        by_ubo.set_sampler(
            &CLIP_TEXTURE,
            &(sampler.value.clone() as Arc<dyn AsRef<<C as Context>::ContextSampler>>),
            &(target.get_color_texture(0).unwrap().clone() as  Arc<dyn AsRef<<C as Context>::ContextTexture>>)
        );
        by_ubo.set_float_1(&CLIP_TEXTURE_SIZE, size as f32);

        let ubos: FnvHashMap<Atom, Arc<Uniforms<C>>> = FnvHashMap::default();

        Self {
            mark: PhantomData,
            dirty: true,
            render_target: Arc::new(target),
            by_ubo: Arc::new(by_ubo),
            rs: rs,
            bs: bs,
            ss: ss,
            ds: ds,
            pipeline: pipeline,
            geometry: Arc::new(engine.gl.create_geometry().unwrap()),
            ubos: ubos,
            sampler: sampler,
        }
    }

    fn add_by_overflow(&self, by_overflow: usize, render_obj: &mut RenderObj<C>, engine: &mut SingleCaseImpl<Engine<C>>) -> bool{
        let defines = &mut render_obj.defines;
        let mut is_change = false;
        // 插入裁剪ubo 插入裁剪宏
        render_obj.ubos.entry(CLIP.clone()).or_insert_with(||{
            defines.push(CLIP.clone());
            is_change = true;
            self.by_ubo.clone()
        });

        if is_change {
            // 设置 by_clip_index
            render_obj.ubos.entry(CLIP_INDICES.clone()).and_modify(|ubo: &mut Arc<Uniforms<C>>|{
                debug_println!("modify clip ubo, by_overflow: {}", by_overflow);
                Arc::make_mut(ubo).set_float_1(&CLIP_INDICES, by_overflow as f32);
            }).or_insert_with(||{
                debug_println!("add clip ubo, by_overflow: {}", by_overflow);
                let mut ubo = engine.gl.create_uniforms();
                ubo.set_float_1(&CLIP_INDICES, by_overflow as f32);
                Arc::new(ubo)
            });

            // 重新创建渲染管线
            let pipeline = engine.create_pipeline(
                render_obj.pipeline.start_hash,
                &render_obj.pipeline.vs,
                &render_obj.pipeline.fs,
                render_obj.defines.as_slice(),
                render_obj.pipeline.rs.clone(),
                render_obj.pipeline.bs.clone(),
                render_obj.pipeline.ss.clone(),
                render_obj.pipeline.ds.clone(),
            );
            render_obj.pipeline = pipeline;
        }
        return is_change;
    }
}

impl<'a, C: Context + Share> Runner<'a> for ClipSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<OverflowClip>,
        &'a SingleCaseImpl<ProjectionMatrix>,
        &'a SingleCaseImpl<ViewMatrix>,
        &'a SingleCaseImpl<RenderBegin>,
    );
    type WriteData = &'a mut SingleCaseImpl<Engine<C>>;
    fn run(&mut self, read: Self::ReadData, engine: Self::WriteData){
        if self.dirty == false {
            return;
        }
        self.dirty = false;

        let (overflow, _projection, _view, view_port) = read;
        let gl = &mut engine.gl;

        // pub struct RenderBeginDesc {
        //     pub viewport: (i32, i32, i32, i32),    // x, y, 宽, 高，左上角为原点
        //     pub clear_color: Option<(f32, f32, f32, f32)>, // r, g, b, a，范围：0-1，为None代表不更新颜色
        //     pub clear_depth: Option<f32>,   // 0-1，1代表最远，为None代表不更新深度
        //     pub clear_stencil: Option<u8>, // 0-255，为None代表不更新模板
        // }
        let viewport = &view_port.0.viewport;
        let new_viewport = (viewport.0, viewport.1, viewport.2, viewport.3);
        let viewport = RenderBeginDesc{
            viewport: new_viewport,
            clear_color: Some((0.0, 0.0, 0.0, 1.0)),
            clear_depth: None,
            clear_stencil: None,
        };
        // gl.begin_render(
        //     &(gl.get_default_render_target().clone() as Arc<AsRef<C::ContextRenderTarget>>), 
        //     &(Arc::new(viewport) as Arc<AsRef<RenderBeginDesc>>)
        // );

        gl.begin_render(
            &(self.render_target.clone() as Arc<dyn AsRef<C::ContextRenderTarget>>), 
            &(Arc::new(viewport) as Arc<dyn AsRef<RenderBeginDesc>>)
        );

        // set_pipeline
        gl.set_pipeline(&(self.pipeline.pipeline.clone() as Arc<dyn AsRef<Pipeline>>));
        {
            let geometry_ref = unsafe {&mut *(self.geometry.as_ref() as *const C::ContextGeometry as usize as *mut C::ContextGeometry)};

            let mut positions = [0.0; 192];
            for i in 0..16 {
                let p = &overflow.clip[i];

                positions[i * 12 + 0] = p[0].x;
                positions[i * 12 + 1] = p[0].y;
                positions[i * 12 + 2] = 0.0;

                positions[i * 12 + 3] = p[1].x;
                positions[i * 12 + 4] = p[1].y;
                positions[i * 12 + 5] = 0.0;

                positions[i * 12 + 6] = p[2].x;
                positions[i * 12 + 7] = p[2].y;
                positions[i * 12 + 8] = 0.0;

                positions[i * 12 + 9] = p[3].x;
                positions[i * 12 + 10] = p[3].y;
                positions[i * 12 + 11] = 0.0;
            }
            geometry_ref.set_attribute(&AttributeName::Position, 3, Some(&positions[0..192]), true).unwrap();
        }
        let mut ubos: FnvHashMap<Atom, Arc<dyn AsRef<Uniforms<C>>>> = FnvHashMap::default();
        for (k, v) in self.ubos.iter() {
            ubos.insert(k.clone(), v.clone() as Arc<dyn AsRef<Uniforms<C>>>);
        }
        //draw
        gl.draw(&(self.geometry.clone() as Arc<dyn AsRef<<C as Context>::ContextGeometry>>), &ubos);

        gl.end_render();
    }
    
    fn setup(&mut self, read: Self::ReadData, engine: Self::WriteData){
        let mut indexs: Vec<f32> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        for i in 0..16 {
            indexs.extend_from_slice(&[i as f32, i as f32, i as f32, i as f32]);
            indices.extend_from_slice(&[4 * i + 0, 4 * i + 1, 4 * i + 2, 4 * i + 0, 4 * i + 2, 4 * i + 3]);
        }
        
        let geometry = unsafe {&mut *(self.geometry.as_ref() as *const C::ContextGeometry as usize as *mut C::ContextGeometry)};
        geometry.set_vertex_count(32);
        let _ = geometry.set_attribute(&AttributeName::SkinIndex, 1, Some(indexs.as_slice()), false);
        geometry.set_indices_short(indices.as_slice(), false).unwrap();

        let ( _, projection, view, _) = read;
        let view: &[f32; 16] = view.0.as_ref();
        let projection: &[f32; 16] = projection.0.as_ref();
        let mut ubo = engine.gl.create_uniforms();
        ubo.set_mat_4v(&VIEW_MATRIX, &view[0..16]);   
        ubo.set_mat_4v(&PROJECT_MATRIX, &projection[0..16]);
        ubo.set_float_1(&MESH_NUM, 16.0);
        self.ubos.insert(COMMON.clone(), Arc::new(ubo));
    
        self.dirty = true;
    }
}

//创建RenderObj， 为renderobj添加裁剪宏及ubo
impl<'a, C: Context + Share> SingleCaseListener<'a, RenderObjs<C>, CreateEvent> for ClipSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, ByOverflow>;
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &CreateEvent, by_overflows: Self::ReadData, write: Self::WriteData){
        let (render_objs, engine) = write;
        let render_obj = unsafe { render_objs.get_unchecked_mut(event.id) };
        let node_id = render_obj.context;
        let by_overflow = unsafe { by_overflows.get_unchecked(node_id).0 };
        if by_overflow > 0 {
            if self.add_by_overflow(by_overflow, render_obj, engine) {
                render_objs.get_notify().modify_event(node_id, "pipeline", 0);
            }
        }
    }
}

//by_overfolw变化， 设置ubo， 修改宏， 并重新创建渲染管线
impl<'a, C: Context + Share> MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for ClipSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, ByOverflow>, &'a SingleCaseImpl<NodeRenderMap>);
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let (by_overflows, node_render_map) = read;
        let (render_objs, engine) = write;
        let by_overflow = unsafe { by_overflows.get_unchecked(event.id).0 };
        let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };

        if by_overflow == 0 {
            for id in obj_ids.iter() {
                let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };

                // 移除ubo
                if let Some(_) = render_obj.ubos.remove(&CLIP) {
                    //移除宏
                    render_obj.defines.remove_item(&CLIP);
                    
                    // 重新创建渲染管线
                    let pipeline = engine.create_pipeline(
                        render_obj.pipeline.start_hash,
                        &render_obj.pipeline.vs,
                        &render_obj.pipeline.fs,
                        render_obj.defines.as_slice(),
                        render_obj.pipeline.rs.clone(),
                        render_obj.pipeline.bs.clone(),
                        render_obj.pipeline.ss.clone(),
                        render_obj.pipeline.ds.clone(),
                    );
                    render_obj.pipeline = pipeline;
                    render_objs.get_notify().modify_event(*id, "pipeline", 0);
                }  
            }
        } else {
            for id in obj_ids.iter() {
                let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
                if self.add_by_overflow(by_overflow, render_obj, engine) {
                    render_objs.get_notify().modify_event(*id, "pipeline", 0);
                }
            }
        }
    }
}


impl<'a, C: Context + Share> SingleCaseListener<'a, OverflowClip, ModifyEvent> for ClipSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, _event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        self.dirty = true;
    }
}

unsafe impl<C: Context + Share> Sync for ClipSys<C>{}
unsafe impl<C: Context + Share> Send for ClipSys<C>{}



fn next_power_of_two(value: u32) -> u32 {
    let mut value = value - 1;
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
    value += 1;
    value
}

impl_system!{
    ClipSys<C> where [C: Context + Share],
    true,
    {
        SingleCaseListener<OverflowClip, ModifyEvent>
        SingleCaseListener<RenderObjs<C>, CreateEvent>
        MultiCaseListener<Node, ByOverflow, ModifyEvent>  
    }
}