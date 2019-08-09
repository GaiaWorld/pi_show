// 监听Overflow， 绘制裁剪纹理
use share::Share;

use ordered_float::OrderedFloat;

use ecs::{ModifyEvent, CreateEvent, MultiCaseListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
use hal_core::*;

use component::calc::*;
use entity::{Node};
use single::*;
use render::engine:: { Engine };
use system::util::*;
use system::render::shaders::clip::*;

pub struct ClipSys{
    dirty: bool,
    clip_size_ubo: Share<ClipTextureSize>,
    sampler: Share<HalSampler>,

    rs: Share<HalRasterState>,
    bs: Share<HalBlendState>,
    ss: Share<HalStencilState>,
    ds: Share<HalDepthState>,
    render_target: HalRenderTarget,
    program: Share<HalProgram>,
    geometry: HalGeometry,
    indexs: HalBuffer,
    mumbers: HalBuffer,
    positions: HalBuffer,
    paramter: Share<dyn ProgramParamter>,
    begin_desc: RenderBeginDesc,
}

impl ClipSys{
    pub fn new(engine: &mut Engine, w: u32, h: u32, viewport: &(i32, i32, i32, i32)) -> Self{
        let (rs, mut bs, ss, mut ds) = (RasterStateDesc::default(), BlendStateDesc::default(), StencilStateDesc::default(), DepthStateDesc::default());
        bs.set_rgb_factor(BlendFactor::One, BlendFactor::One);
        ds.set_test_enable(false);
        ds.set_write_enable(false);

        let paramter = ClipParamter::default();

        let mut mumbers: Vec<f32> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        for i in 0..16 {
            mumbers.extend_from_slice(&[i as f32, i as f32, i as f32, i as f32]);
            indices.extend_from_slice(&[4 * i + 0, 4 * i + 1, 4 * i + 2, 4 * i + 0, 4 * i + 2, 4 * i + 3]);
        }

        let p_buffer = create_buffer(&engine.gl, BufferType::Attribute, 128, None, false);
        let m_buffer = create_buffer(&engine.gl, BufferType::Attribute, mumbers.len(), Some(BufferData::Float(mumbers.as_slice())), false);
        let i_buffer = create_buffer(&engine.gl, BufferType::Indices, indices.len(), Some(BufferData::Short(indices.as_slice())), false);

        let geo = create_geometry(&engine.gl);
        engine.gl.geometry_set_attribute(&geo, &AttributeName::Position, &p_buffer, 2).unwrap();
        engine.gl.geometry_set_attribute(&geo, &AttributeName::SkinIndex, &m_buffer, 1).unwrap();
        engine.gl.geometry_set_indices_short(&geo, &i_buffer).unwrap();

        let program = engine.create_program(
            CLIP_VS_SHADER_NAME.get_hash(),
            CLIP_FS_SHADER_NAME.get_hash(),
            CLIP_VS_SHADER_NAME.as_ref(),
            &VsDefines::default(),
            CLIP_FS_SHADER_NAME.as_ref(),
            &FsDefines::default(),
            &paramter,
        );

        let size = next_power_of_two(w.max(h));
        let target = engine.gl.rt_create(size, size, PixelFormat::RGB, DataFormat::UnsignedByte, false).unwrap();
        let mut clip_size_ubo = ClipTextureSize::default();
        clip_size_ubo.set_value("clipTextureSize", UniformValue::Float1(size as f32));

        // by_ubo.set_sampler(
        //     &CLIP_TEXTURE,
        //     &(sampler.value.clone() as Share<dyn AsRef<ContextSampler>>),
        //     &(target.get_color_texture(0).unwrap().clone() as  Share<dyn AsRef<ContextTexture>>)
        // );

        Self {
            dirty: true,
            clip_size_ubo: Share::new(clip_size_ubo),
            sampler: create_default_sampler(engine: &mut Engine),

            rs: create_rs_res(engine, rs),
            bs: create_bs_res(engine, bs),
            ss: create_ss_res(engine, ss),
            ds: create_ds_res(engine, ds),

            render_target: target,
            program: program,
            geometry: geo,
            indexs: i_buffer,
            mumbers: m_buffer,
            positions: p_buffer,
            paramter: Share::new(paramter),
            begin_desc: RenderBeginDesc{
                viewport: (viewport.0, viewport.1, viewport.2, viewport.3),
                clear_color: Some((OrderedFloat(0.0), OrderedFloat(0.0), OrderedFloat(0.0), OrderedFloat(1.0))),
                clear_depth: None,
                clear_stencil: None,
            },
        }
    }
}

impl<'a> Runner<'a> for ClipSys{
    type ReadData = (
        &'a SingleCaseImpl<OverflowClip>,
        &'a SingleCaseImpl<ProjectionMatrix>,
        &'a SingleCaseImpl<ViewMatrix>,
        &'a SingleCaseImpl<RenderBegin>,
    );
    type WriteData = &'a mut SingleCaseImpl<Engine>;
    fn run(&mut self, read: Self::ReadData, engine: Self::WriteData){
        if self.dirty == false {
            return;
        }

        self.dirty = false;

        let (overflow, _projection, _view, _view_port) = read;
        let gl = &mut engine.gl;

        // set_pipeline
        {
            let mut positions = [0.0; 128];
            for i in 0..16 {
                let p = &overflow.clip[i];

                positions[i * 8 + 0] = p[0].x;
                positions[i * 8 + 1] = p[0].y;

                positions[i * 8 + 3] = p[1].x;
                positions[i * 8 + 4] = p[1].y;

                positions[i * 8 + 6] = p[2].x;
                positions[i * 8 + 7] = p[2].y;

                positions[i * 8 + 9] = p[3].x;
                positions[i * 8 + 10] = p[3].y;
            }
            gl.buffer_update(&self.positions, 0, BufferData::Float(&positions[..]));
        }

        // 渲染裁剪平面
        gl.render_begin(&self.render_target, &self.begin_desc);
        gl.render_set_program(&self.program);
        gl.render_set_state(&self.bs, &self.ds, &self.rs, &self.ss);
        gl.render_draw(&self.geometry, &self.paramter);
        gl.render_end();
    }
    
    fn setup(&mut self, read: Self::ReadData, _engine: Self::WriteData){
        let ( _, view_matrix, projection_matrix, _) = read;

        let slice: &[f32; 16] = view_matrix.0.as_ref();
        let view_matrix_ubo = ViewMatrixUbo::new(UniformValue::MatrixV4(Vec::from(&slice[..])));
        debug_println!("view_matrix: {:?}", &slice[..]);

        let slice: &[f32; 16] = projection_matrix.0.as_ref();
        let project_matrix_ubo = ProjectMatrixUbo::new(UniformValue::MatrixV4(Vec::from(&slice[..])));
        debug_println!("projection_matrix: {:?}", &slice[..]);

        self.paramter.set_value("viewMatrix", Share::new(view_matrix_ubo)); // VIEW_MATRIX
        self.paramter.set_value("projectMatrix", Share::new(project_matrix_ubo)); // PROJECT_MATRIX
        self.paramter.set_single_uniform("meshNum", UniformValue::Float1(16.0));
    }
}

//创建RenderObj， 为renderobj添加裁剪宏及ubo
impl<'a> SingleCaseListener<'a, RenderObjs, CreateEvent> for ClipSys{
    type ReadData = &'a MultiCaseImpl<Node, ByOverflow>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &CreateEvent, by_overflows: Self::ReadData, render_objs: Self::WriteData){
        let render_obj = unsafe { render_objs.get_unchecked_mut(event.id) };
        let node_id = render_obj.context;
        let by_overflow = unsafe { by_overflows.get_unchecked(node_id).0 };
        if by_overflow > 0 {
            render_obj.paramter.set_single_uniform("clipIndices", UniformValue::Float1(by_overflow as f32));
            // 插入裁剪ubo 插入裁剪宏
            if let None = render_obj.fs_defines.add("CLIP") {
                render_objs.get_notify().modify_event(node_id, "program_dirty", 0);
            }
        }
    }
}

//by_overfolw变化， 设置ubo， 修改宏， 并重新创建渲染管线
impl<'a> MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for ClipSys{
    type ReadData = (&'a MultiCaseImpl<Node, ByOverflow>, &'a SingleCaseImpl<NodeRenderMap>);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, render_objs: Self::WriteData){
        let (by_overflows, node_render_map) = read;
        let by_overflow = unsafe { by_overflows.get_unchecked(event.id).0 };
        let obj_ids = unsafe{ node_render_map.get_unchecked(event.id) };

        if by_overflow == 0 {
            for id in obj_ids.iter() {
                let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
                if let Some(_) = render_obj.vs_defines.remove("CLIP") {
                    render_objs.get_notify().modify_event(*id, "program_dirty", 0);
                }
            }
        } else {
            for id in obj_ids.iter() {
                let render_obj = unsafe { render_objs.get_unchecked_mut(*id) };
                render_obj.paramter.set_single_uniform("clipIndices", UniformValue::Float1(by_overflow as f32));
                // 插入裁剪ubo 插入裁剪宏
                if let None = render_obj.fs_defines.add("CLIP") {
                    render_objs.get_notify().modify_event(*id, "program_dirty", 0);
                }
            }
        }
    }
}


impl<'a> SingleCaseListener<'a, OverflowClip, ModifyEvent> for ClipSys{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, _event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        self.dirty = true;
    }
}

unsafe impl Sync for ClipSys{}
unsafe impl Send for ClipSys{}



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
    ClipSys,
    true,
    {
        SingleCaseListener<OverflowClip, ModifyEvent>
        SingleCaseListener<RenderObjs, CreateEvent>
        MultiCaseListener<Node, ByOverflow, ModifyEvent>  
    }
}