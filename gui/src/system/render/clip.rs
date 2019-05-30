// 监听Overflow， 绘制裁剪纹理

use std::marker::PhantomData;
use std::sync::Arc;

use std::collections::HashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use ecs::idtree::{ IdTree};
use map::vecmap::VecMap;
use hal_core::*;
use atom::Atom;

use component::user::*;
use component::calc::{Visibility, WorldMatrix, Opacity, ByOverflow, ZDepth};
use entity::{Node};
use single::*;
use render::engine:: { Engine, PipelineInfo };
use render::res::{SamplerRes, TextureRes};
use system::util::*;
use system::util::constant::*;
use system::render::shaders::clip::*;

lazy_static! {
    static ref MESH_NUM: Atom = Atom::from("meshNum");
    static ref MESH_INDEX: Atom = Atom::from("meshIndex");
    static ref CLIP_RENDER: Atom = Atom::from("clip_render");

    static ref COMMON: Atom = Atom::from("common");
    
    static ref CLIP_TEXTURE: Atom = Atom::from("clip_texture");
    static ref CLIP_INDEX: Atom = Atom::from("clip_index");
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
    ubos: HashMap<Atom, Arc<Uniforms<C>>>,
    sampler: Arc<SamplerRes<C>>,
}

impl<C: Context + Share> ClipSys<C>{
    fn new(engine: &mut Engine<C>, w: u32, h: u32) -> Self{
        let (rs, bs, ss, ds) = (Arc::new(RasterState::new()), Arc::new(BlendState::new()), Arc::new(StencilState::new()), Arc::new(DepthState::new()));
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
        let sampler = match engine.res_mgr.samplers.get(&hash) {
            Some(r) => r.clone(),
            None => {
                let res = SamplerRes::new(hash, engine.gl.create_sampler(Arc::new(s)).unwrap());
                engine.res_mgr.samplers.create(res)
            },          
        };
        let mut geometry = create_geometry(&mut engine.gl);
        let target = engine.gl.create_render_target(w, h, &PixelFormat::RGB, &DataFormat::UnsignedByte, false).unwrap();
        let mut by_ubo = engine.gl.create_uniforms();
        // by_ubo.set_sampler(
        //     &CLIP_TEXTURE,
        //     &(sampler.clone() as Arc<AsRef<<C as Context>::ContextSampler>>),
        //     &(target.get_color_texture(0).unwrap().clone())
        // );

        let mut ubos: HashMap<Atom, Arc<Uniforms<C>>> = HashMap::default();

        Self {
            mark: PhantomData,
            dirty: true,
            render_target: Arc::new(target),
            by_ubo: Arc::new(engine.gl.create_uniforms()),
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
}

impl<'a, C: Context + Share> Runner<'a> for ClipSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<OverflowClip>,
        &'a SingleCaseImpl<ProjectionMatrix>,
        &'a SingleCaseImpl<ViewMatrix>,
        &'a SingleCaseImpl<ViewPort>,
    );
    type WriteData = &'a mut SingleCaseImpl<Engine<C>>;
    fn run(&mut self, read: Self::ReadData, engine: Self::WriteData){
        let gl = &mut engine.gl;
        if self.dirty == false {
            return;
        }
        self.dirty = false;

        let (overflow, projection, view, view_port) = read;
        let gl = &mut engine.gl;
        gl.begin_render(
            &(self.render_target.clone() as Arc<AsRef<C::ContextRenderTarget>>), 
            &(view_port.0.clone() as Arc<AsRef<RenderBeginDesc>>)
        );

        // set_pipeline
        gl.set_pipeline(&(self.pipeline.pipeline.clone() as Arc<AsRef<Pipeline>>));
        {
            let geometry_ref = unsafe {&mut *(self.geometry.as_ref() as *const C::ContextGeometry as usize as *mut C::ContextGeometry)};

            let mut positions = [0.0; 96];
            for i in 0..8 {
                let p = &overflow.clip[i];

                positions[i * 12 + 0] = p[0].x;
                positions[i * 12 + 1] = p[0].y;
                positions[i * 12 + 2] = 0.0;

                positions[i * 12 + 3] = p[2].x;
                positions[i * 12 + 4] = p[2].y;
                positions[i * 12 + 5] = 0.0;

                positions[i * 12 + 6] = p[3].x;
                positions[i * 12 + 7] = p[3].y;
                positions[i * 12 + 8] = 0.0;

                positions[i * 12 + 9] = p[1].x;
                positions[i * 12 + 10] = p[1].y;
                positions[i * 12 + 11] = 0.0;
            }
            geometry_ref.set_attribute(&AttributeName::Position, 3, Some(&positions[0..96]), true);
        }
        let mut ubos: HashMap<Atom, Arc<AsRef<Uniforms<C>>>> = HashMap::new();
        for (k, v) in self.ubos.iter() {
            ubos.insert(k.clone(), v.clone() as Arc<AsRef<Uniforms<C>>>);
        }
        //draw
        gl.draw(&(self.geometry.clone() as Arc<AsRef<<C as Context>::ContextGeometry>>), &ubos);

        gl.end_render();

        // // 设置by_ubo
        // let by_ubo = unsafe {&mut *(self.by_ubo.as_ref() as *const Uniforms<C> as usize as *mut Uniforms<C>)};
        // by_ubo.set_sampler(
        //     &CLIP_TEXTURE,
        //     &(self.sampler.clone() as Arc<AsRef<<C as Context>::ContextSampler>>),
        //     &(target.get_color_texture(0))
        // );
    }
    
    fn setup(&mut self, read: Self::ReadData, engine: Self::WriteData){
        let mut geometry = create_geometry(&mut engine.gl);

        let mut indexs: Vec<f32> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        for i in 0..8 {
            indexs.extend_from_slice(&[i as f32, i as f32, i as f32, i as f32]);
            indices.extend_from_slice(&[4 * i + 0, 4 * i + 1, 4 * i + 2, 4 * i + 0, 4 * i + 2, 4 * i + 3]);
        }
        
        let geometry = unsafe {&mut *(geometry.as_ref() as *const C::ContextGeometry as usize as *mut C::ContextGeometry)};
        geometry.set_indices_short(indices.as_slice(), false);
        geometry.set_attribute(&AttributeName::Custom(MESH_INDEX.clone()), 1, Some(indexs.as_slice()), true);

        let ( _, projection, view, _) = read;
        let view: &[f32; 16] = view.0.as_ref();
        let projection: &[f32; 16] = projection.0.as_ref();
        let mut ubo = engine.gl.create_uniforms();
        ubo.set_mat_4v(&VIEW_MATRIX, &view[0..16]);   
        ubo.set_mat_4v(&PROJECT_MATRIX, &projection[0..16]);
        ubo.set_float_1(&MESH_NUM, 8.0);
        self.ubos.insert(COMMON.clone(), Arc::new(ubo));

        self.dirty = true;
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
                render_obj.ubos.remove(&CLIP);

                //移除宏
                render_obj.defines.remove_item(&CLIP);
                
                // 重新创建渲染管线
                let pipeline = engine.create_pipeline(
                    0,
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
        } else {
            for id in obj_ids.iter() {
                let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };

                let defines = &mut render_obj.defines;

                // 插入裁剪ubo 插入裁剪宏
                render_obj.ubos.entry(CLIP.clone()).or_insert_with(||{
                    defines.push(CLIP.clone());
                    self.by_ubo.clone()
                });

                // // 设置 by_clip_index
                // render_obj.ubos.entry(CLIP_INDEX.clone()).or_insert_with(||{
                //     let mut ubo = engine.gl.create_uniforms();
                //     ubo.set_float_1(&CLIP_INDEX, by_overflow as f32);
                //     Arc::new(engine.gl.create_uniforms())
                // }).and_modify(|ubo: &mut Arc<Uniforms<C>>|{
                //     Arc::make_mut(ubo).set_float_1(&CLIP_INDEX, by_overflow as f32);
                // });
                
                // 重新创建渲染管线
                let pipeline = engine.create_pipeline(
                    0,
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
        }
    }
}


impl<'a, C: Context + Share> SingleCaseListener<'a, OverflowClip, ModifyEvent> for ClipSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        self.dirty = true;
    }
}
