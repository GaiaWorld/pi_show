/**
 * 裁剪
 * 非旋转矩形裁剪区域采用向shader传入裁剪区域的的aabb， 在shader中通过aabb来判断是否裁剪
 * 旋转矩形裁剪区域采用在纹理中填入每像素的裁剪位标记，shader通过采样该纹理来确定是否渲染该像素
 */
use std::marker::PhantomData;

use ordered_float::OrderedFloat;

use ecs::monitor::{Event, NotifyImpl};
use ecs::{ModifyEvent, MultiCaseImpl, Runner, SingleCaseImpl, SingleCaseListener};
use hal_core::*;
use map::vecmap::VecMap;
use share::Share;

use crate::component::calc::*;
use crate::component::user::Aabb2;
use crate::entity::Node;
use crate::render::engine::{Engine, ShareEngine};
use crate::render::res::*;
use crate::single::*;
use crate::system::render::shaders::clip::*;

pub struct ClipSys<C> {
    dirty: bool,
    no_rotate_dirtys: VecMap<bool>,
    render_obj: Option<ClipTextureRender>,
	marker: PhantomData<C>,
	
	rotate_index: usize,
}

struct ClipTextureRender {
    clip_size_ubo: Share<ClipTextureSize>,
	sampler: Share<SamplerRes>,

    rs: Share<RasterStateRes>,
    bs: Share<BlendStateRes>,
    ss: Share<StencilStateRes>,
    ds: Share<DepthStateRes>,
    render_target: HalRenderTarget,
    program: Share<HalProgram>,
	geometry: HalGeometry,

    paramter: Share<dyn ProgramParamter>,
    begin_desc: RenderBeginDesc,
}

impl<C: HalContext + 'static> ClipSys<C> {
    pub fn init_render(
        &mut self,
        engine: &mut Engine<C>,
		viewport: &(i32, i32, i32, i32),
		scissor: &(i32, i32, i32, i32),
        view_matrix: &ViewMatrix,
        projection_matrix: &ProjectionMatrix,
    ) {
        let (rs, mut bs, ss, mut ds) = (
            RasterStateDesc::default(),
            BlendStateDesc::default(),
            StencilStateDesc::default(),
            DepthStateDesc::default(),
        );
        bs.set_rgb_factor(BlendFactor::One, BlendFactor::One);
        ds.set_test_enable(false);
        ds.set_write_enable(false);

        let paramter = ClipParamter::default();

        let geo = engine.create_geometry();

        let program = engine.create_program(
            CLIP_VS_SHADER_NAME.get_hash() as u64,
            CLIP_FS_SHADER_NAME.get_hash() as u64,
            CLIP_VS_SHADER_NAME.as_ref(),
            &VsDefines::default(),
            CLIP_FS_SHADER_NAME.as_ref(),
            &FsDefines::default(),
            &paramter,
        );

        let target = engine
            .gl
            .rt_create(
                None,
                (viewport.0 + viewport.2) as u32,
                (viewport.1 + viewport.3) as u32,
                PixelFormat::RGBA,
                DataFormat::UnsignedByte,
                false,
            )
			.unwrap();
			
        let mut clip_size_ubo = ClipTextureSize::default();
        clip_size_ubo.set_value("clipTextureSize", UniformValue::Float2((viewport.0 + viewport.2) as f32, (viewport.1 + viewport.3) as f32));

        let slice: &[f32] = view_matrix.0.as_slice();
        let view_matrix_ubo = ViewMatrixUbo::new(UniformValue::MatrixV4(Vec::from(&slice[..])));

        let slice: &[f32] = projection_matrix.0.as_slice();
        let project_matrix_ubo =
            ProjectMatrixUbo::new(UniformValue::MatrixV4(Vec::from(&slice[..])));

        paramter.set_value("viewMatrix", Share::new(view_matrix_ubo)); // VIEW_MATRIX
		paramter.set_value("projectMatrix", Share::new(project_matrix_ubo)); // PROJECT_MATRIX
		
		//https://stackoverflow.com/questions/3792027/webgl-and-the-power-of-two-image-size
		let mut sampler = SamplerDesc::default();
		sampler.min_filter = TextureFilterMode::Nearest;
		sampler.mag_filter = TextureFilterMode::Nearest;
		sampler.u_wrap = TextureWrapMode::ClampToEdge;
		sampler.v_wrap = TextureWrapMode::ClampToEdge;

        self.render_obj = Some(ClipTextureRender {
            clip_size_ubo: Share::new(clip_size_ubo),
            sampler: engine.create_sampler_res(sampler),

            rs: engine.create_rs_res(rs),
            bs: engine.create_bs_res(bs),
            ss: engine.create_ss_res(ss),
            ds: engine.create_ds_res(ds),

            render_target: target,
            program: program,
            geometry: geo,

            paramter: Share::new(paramter),
            begin_desc: RenderBeginDesc {
				viewport: (viewport.0, viewport.1, viewport.2, viewport.3),
				scissor: (viewport.0, viewport.1, viewport.2, viewport.3),
                clear_color: Some((
                    OrderedFloat(0.0),
                    OrderedFloat(0.0),
                    OrderedFloat(0.0),
                    OrderedFloat(0.0),
                )),
                clear_depth: None,
                clear_stencil: None,
            },
        });
    }

    pub fn new() -> Self {
        Self {
            dirty: false,
            no_rotate_dirtys: VecMap::default(),
            render_obj: None,
			marker: PhantomData,
			rotate_index: 0,
        }
    }

    fn set_clip_uniform(
        &self,
        id: usize,
        by_overflow: usize,
        aabb: Option<&(Aabb2, Share<dyn UniformBuffer>)>,
        notify: &NotifyImpl,
        render_obj: &mut RenderObj,
        engine: &mut Engine<C>,
    ) {
        match aabb {
            Some(item) => {
                // if render_obj.visibility {
                render_obj.paramter.set_value("clipBox", item.1.clone());
                if let None = render_obj.fs_defines.add("CLIP_BOX") {
                    render_obj.vs_defines.add("CLIP_BOX");
                    render_obj.fs_defines.remove("CLIP");
                    notify.modify_event(id, "program_dirty", 0);
                }
                // }
            }
            None => {
				let by_overflow = by_overflow & self.rotate_index;
                render_obj
                    .paramter
					.set_single_uniform("clipIndices1", UniformValue::Float1(by_overflow as f32));
				render_obj
                    .paramter
                    .set_single_uniform("clipIndices2", UniformValue::Float1((by_overflow >> 24) as f32));
                let clip_render = match self.render_obj.as_ref(){
					Some(r) => r,
					None => {
						log::warn!("clip render obj is not exist");
						panic!("xxxxxxxxxxxxxxxxxxx, by_overflow: {}", by_overflow);
					}
				};
                // 插入裁剪ubo 插入裁剪宏
                if let None = render_obj.fs_defines.add("CLIP") {
                    render_obj.vs_defines.remove("CLIP_BOX");
                    render_obj.fs_defines.remove("CLIP_BOX");
                    render_obj.paramter.set_texture(
                        "clipTexture",
                        (
                            match engine
                                .gl
                                .rt_get_color_texture(&clip_render.render_target, 0){
									Some(r) => r,
									None => {
										log::warn!("clip render target rt_get_color_texture fail");
										panic!("clip render target rt_get_color_texture fail");
									}
								},
                            &clip_render.sampler,
                        ),
                    );
                    render_obj
                        .paramter
                        .set_value("clipTextureSize", clip_render.clip_size_ubo.clone());
                    notify.modify_event(id, "program_dirty", 0);
                }
            }
        }
    }
}

impl<'a, C: HalContext + 'static> Runner<'a> for ClipSys<C> {
    type ReadData = (
        &'a MultiCaseImpl<Node, ByOverflow>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a SingleCaseImpl<DirtyList>,
        &'a SingleCaseImpl<NodeRenderMap>,
        &'a SingleCaseImpl<OverflowClip>,
        &'a SingleCaseImpl<ProjectionMatrix>,
        &'a SingleCaseImpl<ViewMatrix>,
        &'a SingleCaseImpl<RenderBegin>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<OverflowClip>,
        &'a mut MultiCaseImpl<Node, Culling>,
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<ShareEngine<C>>,
    );
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        let (
            by_overflows,
            style_marks,
            dirty_list,
            node_render_map,
            overflow,
            projection,
            view,
            view_port,
        ) = read;
        let (overflow_clip, cullings, render_objs, engine) = write;

        if self.dirty {
            self.dirty = false;
            if self.render_obj.is_none() {
                self.init_render(engine, &view_port.0.viewport, &view_port.0.scissor, view, projection);
            }
            let clip_render = self.render_obj.as_ref().unwrap();

			let mut positions: Vec<f32> = Vec::new();
			let mut mumbers: Vec<f32> = Vec::new();

			self.rotate_index = 0;
            for (i, c) in overflow.clip.iter() {
                if c.has_rotate {
                    let p = &c.view;
                    positions.push(p[0].x);
                    positions.push(p[0].y);
                    positions.push(p[1].x);
                    positions.push(p[1].y);
                    positions.push(p[2].x);
                    positions.push(p[2].y);
                    positions.push(p[3].x);
					positions.push(p[3].y);
					let m = i as f32 -1.0;
					mumbers.extend_from_slice(&[m,m,m,m]);
					self.rotate_index |= 1 << m as usize;
				}
            }

            let mut indices: Vec<u16> = Vec::new();

			// clip数量超出16个，设置clip只有16个
            let mut count = positions.len() / 8;
            if count > 32 {
                count = 32;
                unsafe { positions.set_len(256) };
            }
            for i in 0..count as u16 {
                indices.extend_from_slice(&[
                    4 * i + 0,
                    4 * i + 1,
                    4 * i + 2,
                    4 * i + 0,
                    4 * i + 2,
                    4 * i + 3,
                ]);
            }

            let p_buffer = engine.create_buffer(BufferType::Attribute, positions.len(), Some(BufferData::Float(positions.as_slice())), false);
            let m_buffer = engine.create_buffer(
                BufferType::Attribute,
                mumbers.len(),
                Some(BufferData::Float(mumbers.as_slice())),
                false,
            );
            let i_buffer = engine.create_buffer(
                BufferType::Indices,
                indices.len(),
                Some(BufferData::Short(indices.as_slice())),
                false,
            );

            engine
                .gl
                .geometry_set_attribute(
                    &clip_render.geometry,
                    &AttributeName::Position,
                    &p_buffer,
                    2,
                )
                .unwrap();
            engine
                .gl
                .geometry_set_attribute(
                    &clip_render.geometry,
                    &AttributeName::SkinIndex,
                    &m_buffer,
                    1,
                )
                .unwrap();
            engine
                .gl
                .geometry_set_indices_short(&clip_render.geometry, &i_buffer)
                .unwrap();
            clip_render
                .paramter
                .set_single_uniform("meshNum", UniformValue::Float1(count as f32));

            // 渲染裁剪平面
            engine
                .gl
                .render_begin(Some(&clip_render.render_target), &clip_render.begin_desc, true);
            engine.gl.render_set_program(&clip_render.program);
            engine.gl.render_set_state(
                &clip_render.bs,
                &clip_render.ds,
                &clip_render.rs,
                &clip_render.ss,
            );
            engine
                .gl
                .render_draw(&clip_render.geometry, &clip_render.paramter);
            engine.gl.render_end();
        }

        let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
        let mut pre_by_overflow = 0;
        let mut aabb = None;
        for id in dirty_list.0.iter() {
            let style_mark = match style_marks.get(*id) {
                Some(r) => r,
                None => continue,
            };

            let by_overflow = by_overflows[*id].0;
            let obj_ids = &node_render_map[*id];

            if (style_mark.dirty & StyleType::Matrix as usize != 0
                || style_mark.dirty & StyleType::ByOverflow as usize != 0)
                && by_overflow > 0
            {
                if by_overflow != pre_by_overflow {
                    pre_by_overflow = by_overflow;
                    aabb = overflow_clip.clip_map.get(&by_overflow);
                }
                for id in obj_ids.iter() {
                    let render_obj = &mut render_objs[*id];
                   
                    self.set_clip_uniform(*id, by_overflow, aabb, &notify, render_obj, engine);
                }
            } else if style_mark.dirty & StyleType::ByOverflow as usize != 0 && by_overflow == 0 {
                // 裁剪剔除
                unsafe {cullings.get_unchecked_write(*id)}.set_0(false);
                for id in obj_ids.iter() {
                    let render_obj = &mut render_objs[*id];
                    render_obj.vs_defines.remove("CLIP_BOX");
                    render_obj.fs_defines.remove("CLIP_BOX");
                    render_obj.fs_defines.remove("CLIP");
                    render_objs
                        .get_notify_ref()
                        .modify_event(*id, "program_dirty", 0);
                }
            }
        }
    }
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, ProjectionMatrix, ModifyEvent>
    for ClipSys<C>
{
    type ReadData = &'a SingleCaseImpl<ProjectionMatrix>;
    type WriteData = ();
    fn listen(
        &mut self,
        _event: &Event,
        projection_matrix: Self::ReadData,
        _: Self::WriteData,
    ) {
        if let Some(render_obj) = &self.render_obj {
            let slice: &[f32] = projection_matrix.0.as_slice();
            let project_matrix_ubo =
                ProjectMatrixUbo::new(UniformValue::MatrixV4(Vec::from(slice)));
            render_obj
                .paramter
				.set_value("projectMatrix", Share::new(project_matrix_ubo)); // PROJECT_MATRIX
            self.dirty = true;
        }
    }
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderBegin, ModifyEvent>
    for ClipSys<C>
{
    type ReadData = &'a SingleCaseImpl<RenderBegin>;
    type WriteData = ();
    fn listen(
        &mut self,
        _event: &Event,
        render_begin: Self::ReadData,
        _: Self::WriteData,
    ) {
        if let Some(render_obj) = &mut self.render_obj {
			let viewport = &render_begin.0.viewport;
			render_obj.begin_desc.viewport = (viewport.0, viewport.1, viewport.2, viewport.3);
			render_obj.begin_desc.scissor = (viewport.0, viewport.1, viewport.2, viewport.3);
			self.dirty = true;
		}
	}
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, OverflowClip, ModifyEvent> for ClipSys<C> {
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<OverflowClip>;
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
		let c = &write.clip[event.id];
        if c.has_rotate || c.old_has_rotate {
            self.dirty = true;
        }
    }
}

// 是否相交
#[inline]
fn is_intersect(a: &Aabb2, b: &Aabb2) -> bool {
    if a.mins.x >= b.maxs.x || a.mins.y > b.maxs.y || b.mins.x > a.maxs.x || b.mins.y > a.maxs.y {
        return false;
    } else {
        true
    }
}

// a是否包含b
#[inline]
fn is_include(a: &Aabb2, b: &Aabb2) -> bool {
    if a.mins.x <= b.mins.x && a.maxs.x >= b.maxs.x && a.mins.y <= b.mins.y && a.maxs.y >= b.maxs.y {
        return true;
    } else {
        false
    }
}

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

impl_system! {
    ClipSys<C> where [C: HalContext + 'static],
    true,
    {
        SingleCaseListener<OverflowClip, ModifyEvent>
		SingleCaseListener<ProjectionMatrix, ModifyEvent>
		SingleCaseListener<RenderBegin, ModifyEvent>
    }
}
