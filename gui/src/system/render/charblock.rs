use std::hash::{Hash, Hasher};
/**
 * 文字渲染对象的构建及其属性设置
 */
use std::marker::PhantomData;

use cgmath::InnerSpace;
use hash::DefaultHasher;
use ordered_float::NotNan;

use ecs::monitor::NotifyImpl;
use ecs::{
    DeleteEvent, ModifyEvent, MultiCaseImpl, MultiCaseListener, Runner, SingleCaseImpl,
	SingleCaseListener,
	StdCell,
};
use idtree::NodeList;
use hal_core::*;
use map::vecmap::VecMap;
use map::Map;
use polygon::{find_lg_endp, interp_mult_by_lg, mult_to_triangle, split_by_lg, LgCfg};
use res::ResMap;
use share::Share;

use component::{calc::*, calc::LayoutR, calc::CharNode};
use component::user::*;
use entity::Node;
use font::font_sheet::*;
use render::engine::{buffer_size, create_hash_res, Engine, ShareEngine, UnsafeMut};
use render::res::*;
use single::*;
use system::render::shaders::canvas_text::{
    CANVAS_TEXT_FS_SHADER_NAME, CANVAS_TEXT_VS_SHADER_NAME,
};
use system::render::shaders::text::{TEXT_FS_SHADER_NAME, TEXT_VS_SHADER_NAME};
use system::util::*;

const TEXT_STYLE_DIRTY: usize = StyleType::LetterSpacing as usize
    | StyleType::WordSpacing as usize
    | StyleType::LineHeight as usize
    | StyleType::Indent as usize
    | StyleType::WhiteSpace as usize
    | StyleType::TextAlign as usize
    | StyleType::VerticalAlign as usize
    | StyleType::Color as usize
    | StyleType::Stroke as usize
    | StyleType::FontStyle as usize
    | StyleType::FontFamily as usize
    | StyleType::FontSize as usize
    | StyleType::FontWeight as usize
    | StyleType::TextShadow as usize
    | StyleType::Layout as usize
    | StyleType::Text as usize
    | StyleType::Matrix as usize;
const TEXT_LAYOUT_DIRTY: usize = StyleType::FontStyle as usize
    | StyleType::FontWeight as usize
    | StyleType::FontSize as usize
    | StyleType::FontFamily as usize
    | StyleType::LetterSpacing as usize
    | StyleType::WordSpacing as usize
    | StyleType::LineHeight as usize
    | StyleType::Indent as usize
    | StyleType::WhiteSpace as usize
    | StyleType::TextAlign as usize
    | StyleType::Text as usize
    | StyleType::VerticalAlign as usize
    | StyleType::Layout as usize;

const FONT_DIRTY: usize = StyleType::FontStyle as usize
    | StyleType::FontWeight as usize
    | StyleType::FontSize as usize
    | StyleType::FontFamily as usize;

#[derive(Default, Clone, Debug)]
struct I {
    text: usize,
    shadow: usize,
}

struct RenderCatch {
    fill_color_ubo: Share<UColorUbo>,
    stroke_ubo: Share<MsdfStrokeUbo>,
    stroke_color_ubo: Share<CanvasTextStrokeColorUbo>,
    shadow_color_ubo: Share<UColorUbo>,
    layout_hash: u64, // 布局属性的hash
}

pub struct CharBlockSys<C: HalContext + 'static> {
    render_map: VecMap<I>,
    canvas_bs: Share<BlendStateRes>,
    msdf_bs: Share<BlendStateRes>,
    default_sampler: Share<SamplerRes>, // 默认采样方式
    point_sampler: Share<SamplerRes>, // 点采样， canvas着色器渲染时， 如果字体大小与纹理默认字体大小一致， 将采用点采样
    canvas_default_stroke_color: Share<CanvasTextStrokeColorUbo>,
    class_ubos: VecMap<RenderCatch>,
    default_ubos: RenderCatch,
    index_buffer: Share<BufferRes>, // 索引 buffer， 长度： 600
    index_len: usize,
    texture_size_ubo: Share<TextTextureSize>,
	
	old_texture_tex_version: usize,

    msdf_stroke_ubo_map: UnsafeMut<ResMap<MsdfStrokeUbo>>,
    canvas_stroke_ubo_map: UnsafeMut<ResMap<CanvasTextStrokeColorUbo>>,

    msdf_default_paramter: MsdfParamter,
    canvas_default_paramter: CanvasTextParamter,
    mark: PhantomData<(C)>,
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for CharBlockSys<C> {
    type ReadData = (
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, LayoutR>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, TextContent>,
        &'a MultiCaseImpl<Node, StyleMark>,
        &'a MultiCaseImpl<Node, TextStyle>,
        &'a SingleCaseImpl<Share<StdCell<FontSheet>>>,
        &'a SingleCaseImpl<DefaultTable>,
        &'a SingleCaseImpl<DefaultState>,
		&'a SingleCaseImpl<DirtyList>,
		&'a SingleCaseImpl<IdTree>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<ShareEngine<C>>,
        &'a mut MultiCaseImpl<Node, NodeState>,
    );
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        let (
            world_matrixs,
            layouts,
            transforms,
            texts,
            style_marks,
            text_styles,
            font_sheet,
            default_table,
            default_state,
			dirty_list,
			idtree
		) = read;
		let font_sheet = &font_sheet.borrow();
		let t = font_sheet.get_font_tex();
		let mut texture_change = false;
		if font_sheet.tex_version != self.old_texture_tex_version {
			texture_change = true;
			self.old_texture_tex_version = font_sheet.tex_version;
		}
        if dirty_list.0.len() == 0 && !texture_change {
            return;
        }
        let (render_objs, engine, node_states) = write;
        let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
        let default_transform = default_table.get::<Transform>().unwrap();

        if texture_change == true {
            self.texture_size_ubo = Share::new(TextTextureSize::new(UniformValue::Float2(
                t.width as f32,
                t.height as f32,
            )));
            for i in self.render_map.iter() {
                match i {
                    Some(i) => {
                        render_objs[i.text]
                            .paramter
                            .set_value("textureSize", self.texture_size_ubo.clone());
                        if i.shadow > 0 {
                            render_objs[i.shadow]
                                .paramter
                                .set_value("textureSize", self.texture_size_ubo.clone());
                        }
                        notify.modify_event(i.text, "ubo", 0);
                    }
                    None => (),
                }
            }
        }

        for id in dirty_list.0.iter() {
            let (style_mark, text) = match (style_marks.get(*id), texts.get(*id)) {
                (Some(r), Some(r1)) => (r, r1),
                _ => {
                    self.remove_render_obj(*id, render_objs);
                    continue;
                }
			};

            let mut dirty = style_mark.dirty;

            // 不存在Chablock关心的脏, 跳过
            if dirty & TEXT_STYLE_DIRTY == 0 {
                continue;
			}

            // 如果Text脏， 并且不存在Text组件， 尝试删除渲染对象
            let (index, children, text_style) = if dirty & StyleType::FontFamily as usize != 0 {
                self.remove_render_obj(*id, render_objs); // 可能存在旧的render_obj， 先尝试删除（FontFamily修改， 整renderobj中大部分值都会修改， 因此直接重新创建）
                let text_style = &text_styles[*id];
                let children = idtree[*id].children();
                let have_shadow = text_style.shadow.color != CgColor::new(0.0, 0.0, 0.0, 0.0);
                let r = self.create_render_obj(
                    *id,
                    render_objs,
                    default_state,
                    have_shadow,
                    true,//charblock.is_pixel,
                );
                dirty = dirty | TEXT_STYLE_DIRTY;
                (r, children, text_style)
            } else {
                match self.render_map.get(*id) {
                    Some(r) => (
                        r.clone(),
                        idtree[*id].children(),
                        &text_styles[*id],
                    ),
                    None => continue,
                }
            };
			let world_matrix = &world_matrixs[*id];
			let layout = &layouts[*id];
            let transform = match transforms.get(*id) {
                Some(r) => r,
                None => default_transform,
            };

            let render_obj = unsafe {
                &mut *(&render_objs[index.text] as *const RenderObj as usize
                    as *mut RenderObj)
            };

            let class_ubo = &self.default_ubos;

            let (mut program_change, mut geometry_change) = (false, false);
            let mut shadow_geometry_change = false;

            // 颜色脏， 如果是渐变色和纯色切换， 需要修改宏， 且顶点需要重新计算， 因此标记program_change 和 geometry_change脏
            if dirty & (StyleType::Color as usize) != 0 {
                let exchange = modify_color(
                    index.text,
                    style_mark.local_style,
                    &text_style.text.color,
                    engine,
                    &notify,
                    render_obj,
                    &class_ubo,
                );
                program_change = program_change | exchange;
                geometry_change = geometry_change | exchange;
            }
            // 描边脏, 如果字体是canvas绘制类型(不支持Stroke的宽度修改)， 仅修改stroke的uniform， 否则， 如果Stroke是添加或删除， 还要修改Stroke宏， 因此可能设置program_change脏
            if dirty & (StyleType::Stroke as usize) != 0 {
                program_change = program_change
                    | modify_stroke(
                        index.text,
                        style_mark.local_style,
                        &text_style.text.stroke,
                        render_obj,
                        &notify,
                        true,//charblock.is_pixel,
                        &class_ubo,
                        &mut *self.canvas_stroke_ubo_map,
                        &mut *self.msdf_stroke_ubo_map,
                    );
            }
            // 尝试修改字体， 如果字体类型修改（dyn_type）， 需要修改pipeline， （字体类型修改应该重新创建paramter， TODO）
            if dirty & FONT_DIRTY != 0 {
                modify_font(
                    index.text,
                    render_obj,
                    true,//charblock.is_pixel,
                    &font_sheet,
                    &notify,
                    &self.default_sampler,
                    &self.point_sampler,
                );
                program_change = true;
            }

            // 文字内容脏， 这是顶点流脏
            if dirty & TEXT_LAYOUT_DIRTY != 0 {
                geometry_change = true;
                shadow_geometry_change = true;
            }

            // // 如果渲染管线脏， 重新创建渲染管线
            if program_change {
                notify.modify_event(index.text, "program_dirty", 0);
                // modify_program(render_obj, charblock.is_pixel, engine, &self.canvas_default_stroke_color);
            }
            // 文字属性流改变， 重新生成geometry
            if geometry_change {
                let l = &mut self.index_len;
                render_obj.geometry = create_geo(
                    dirty,
					children,
					idtree,
					&node_states[*id],
					layout,
                    &text_style.text.color,
                    text,
                    text_style,
                    &font_sheet,
                    class_ubo,
                    &self.index_buffer,
                    l,
					engine,
					node_states[*id].0.scale
                );
                render_objs
                    .get_notify_ref()
                    .modify_event(index.text, "geometry", 0);
			}
			let (mut h, mut v) = (0.0, 0.0);
			if node_states[*id].0.is_vnode() {
				h = -layout.rect.start;
				v = -layout.rect.top;
			}

            // 矩阵脏
            if dirty & (StyleType::Matrix as usize) != 0 {
				
                modify_matrix(
                    index.text,
                    create_let_top_offset_matrix(
                        layout,
                        world_matrix,
                        transform,
                        h,
                        v,
                        render_obj.depth,
                    ),
                    render_obj,
                    &notify,
                );
            }
            notify.modify_event(index.text, "", 0);
            // 阴影存在
            if index.shadow > 0 {
                let shadow_render_obj = &mut render_objs[index.shadow];

                if dirty & (StyleType::TextShadow as usize) != 0 {
                    // 阴影颜色脏，或描边脏， 修改ubo
                    modify_shadow_color(
                        index.shadow,
                        style_mark.local_style,
                        text_style,
                        &notify,
                        shadow_render_obj,
                        engine,
                        true, // charblock.is_pixel,
                        &class_ubo,
                        &mut *self.canvas_stroke_ubo_map,
                    );
                    // 设置ubo TODO
                }

                // 尝试修改字体， 如果字体类型修改（dyn_type）， 需要修改pipeline， （字体类型修改应该重新创建paramter， TODO）
                if dirty & FONT_DIRTY != 0 {
                    modify_font(
                        index.text,
                        shadow_render_obj,
                        true, //charblock.is_pixel,
                        &font_sheet,
                        &notify,
                        &self.default_sampler,
                        &self.point_sampler,
                    );
                }

                if program_change {
                    notify.modify_event(index.shadow, "program_dirty", 0);
                }

                // 修改阴影的顶点流
                if shadow_geometry_change {
                    // 如果填充色是纯色， 阴影的geo和文字的geo一样， 否则重新创建阴影的geo
                    match &text_style.text.color {
                        Color::RGBA(_) => shadow_render_obj.geometry = render_obj.geometry.clone(),
                        Color::LinearGradient(_) => {
                            let color = text_style.shadow.color.clone();
                            let l = &mut self.index_len;
                            shadow_render_obj.geometry = create_geo(
								dirty,
								children,
								idtree,
								&node_states[*id],
								layout,
                                &Color::RGBA(color),
                                text,
                                text_style,
                                font_sheet,
                                class_ubo,
                                &self.index_buffer,
                                l,
								engine,
								node_states[*id].0.scale,
                            )
                        }
                    }
                    notify.modify_event(index.shadow, "geometry", 0);
                }

                if dirty & (StyleType::Matrix as usize) != 0
                    || dirty & (StyleType::TextShadow as usize) != 0
                {
                    modify_matrix(
                        index.shadow,
                        create_let_top_offset_matrix(
                            layout,
                            world_matrix,
                            transform,
                            text_style.shadow.h + h,
                            text_style.shadow.v + v,
                            shadow_render_obj.depth,
                        ),
                        shadow_render_obj,
                        &notify,
                    );
                }
                notify.modify_event(index.shadow, "", 0);
			}
        }
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, TextStyle, DeleteEvent> for CharBlockSys<C>
{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, render_objs: Self::WriteData) {
        self.remove_render_obj(event.id, render_objs)
    }
}

impl<C: HalContext + 'static> CharBlockSys<C> {
    #[inline]
    pub fn with_capacity(engine: &mut Engine<C>, texture_size: (usize, usize), capacity: usize) -> Self {
        let mut canvas_bs = BlendStateDesc::default();
		canvas_bs.set_rgb_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);
		canvas_bs.set_alpha_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);
		// canvas_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        let canvas_bs = engine.create_bs_res(canvas_bs);

        let mut msdf_bs = BlendStateDesc::default();
        msdf_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        let msdf_bs = engine.create_bs_res(msdf_bs);

		let mut default_sampler = SamplerDesc::default();
		default_sampler.u_wrap = TextureWrapMode::ClampToEdge;
		default_sampler.v_wrap = TextureWrapMode::ClampToEdge;
        let default_sampler = engine.create_sampler_res(default_sampler);

        let mut point_sampler = SamplerDesc::default();
        point_sampler.min_filter = TextureFilterMode::Nearest;
		point_sampler.mag_filter = TextureFilterMode::Nearest;
		point_sampler.u_wrap = TextureWrapMode::ClampToEdge;
		point_sampler.v_wrap = TextureWrapMode::ClampToEdge;
        let point_sampler = engine.create_sampler_res(point_sampler);

        let default_color_ubo = engine.create_u_color_ubo(&CgColor::new(0.0, 0.0, 0.0, 1.0));

        let index_data = create_index_buffer(100);

		let res_mgr_ref = engine.res_mgr.borrow();
        let mut msdf_stroke_ubo_map =
            UnsafeMut::new(res_mgr_ref.fetch_map::<MsdfStrokeUbo>(0).unwrap());
        let mut canvas_stroke_ubo_map = UnsafeMut::new(
            res_mgr_ref
                .fetch_map::<CanvasTextStrokeColorUbo>(0)
                .unwrap(),
        );

        Self {
            render_map: VecMap::with_capacity(capacity),
            canvas_bs: canvas_bs,
            msdf_bs: msdf_bs,
            default_sampler: default_sampler,
            point_sampler: point_sampler,
            canvas_default_stroke_color: Share::new(CanvasTextStrokeColorUbo::new(
                UniformValue::Float4(0.0, 0.0, 0.0, 0.0),
            )),
            class_ubos: VecMap::default(),
            default_ubos: RenderCatch {
                fill_color_ubo: default_color_ubo.clone(),
                shadow_color_ubo: default_color_ubo,
                stroke_ubo: create_hash_res(
                    MsdfStrokeUbo::new(
                        UniformValue::Float1(0.0),
                        UniformValue::Float4(0.0, 0.0, 0.0, 0.0),
                    ),
                    &mut *msdf_stroke_ubo_map,
                ),
                stroke_color_ubo: create_hash_res(
                    CanvasTextStrokeColorUbo::new(UniformValue::Float4(0.0, 0.0, 0.0, 0.0)),
                    &mut *canvas_stroke_ubo_map,
                ),
                layout_hash: 0,
            },
            index_buffer: Share::new(BufferRes(engine.create_buffer(
                BufferType::Indices,
                600,
                Some(BufferData::Short(index_data.as_slice())),
                true,
            ))),
            index_len: 100,
            texture_size_ubo: Share::new(TextTextureSize::new(UniformValue::Float2(
                texture_size.0 as f32,
                texture_size.1 as f32,
            ))),
			old_texture_tex_version: 0,

            msdf_stroke_ubo_map,
            canvas_stroke_ubo_map,
            msdf_default_paramter: MsdfParamter::default(),
            canvas_default_paramter: CanvasTextParamter::default(),
            mark: PhantomData,
        }
    }

    #[inline]
    fn create_render_obj(
        &mut self,
        id: usize,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &DefaultState,
        have_shadow: bool,
        is_pixel: bool,
    ) -> I {
        let shadow_index = if !have_shadow {
            0
        } else {
            self.create_render_obj1(id, render_objs, default_state, is_pixel, 0.0)
        };
        let index = self.create_render_obj1(id, render_objs, default_state, is_pixel, 0.1);

        // 创建RenderObj与Node实体的索引关系， 并设脏
        self.render_map.insert(
            id,
            I {
                text: index,
                shadow: shadow_index,
            },
        );
        self.render_map[id].clone()
    }

    fn create_render_obj1(
        &self,
        id: usize,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &DefaultState,
        is_pixel: bool,
        depth_diff: f32,
    ) -> usize {
        let (vs_name, fs_name, paramter, bs) = if is_pixel {
            let paramter: Share<dyn ProgramParamter> =
                Share::new(self.canvas_default_paramter.clone());
            paramter.set_value("strokeColor", self.canvas_default_stroke_color.clone());
            paramter.set_value("textureSize", self.texture_size_ubo.clone());
            (
                CANVAS_TEXT_VS_SHADER_NAME.clone(),
                CANVAS_TEXT_FS_SHADER_NAME.clone(),
                paramter,
                self.canvas_bs.clone(),
            )
        } else {
            let paramter: Share<dyn ProgramParamter> =
                Share::new(self.msdf_default_paramter.clone());
            (
                TEXT_VS_SHADER_NAME.clone(),
                TEXT_FS_SHADER_NAME.clone(),
                paramter,
                self.msdf_bs.clone(),
            )
        };
        let state = State {
            bs: bs.clone(),
            rs: default_state.df_rs.clone(),
            ss: default_state.df_ss.clone(),
            ds: default_state.tarns_ds.clone(),
        };
        let render_obj = new_render_obj(id, depth_diff, false, vs_name, fs_name, paramter, state);
        render_obj
            .paramter
            .set_value("textureSize", self.texture_size_ubo.clone());
			let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
        render_objs.insert(render_obj, Some(notify))
    }

    #[inline]
    fn remove_render_obj(&mut self, id: usize, render_objs: &mut SingleCaseImpl<RenderObjs>) {
        match self.render_map.remove(id) {
            Some(index) => {
                let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
                render_objs.remove(index.text, Some(notify));
                render_objs.remove(index.shadow, Some(notify));
            }
            None => (),
        };
    }
}

#[inline]
fn modify_stroke(
    index: usize,
    local_style: usize,
    text_stroke: &Stroke,
    render_obj: &mut RenderObj,
    // engine: &mut SingleCaseImpl<Engine<C>>,
    notify: &NotifyImpl,
    is_pixel: bool,
    class_ubo: &RenderCatch,
    canvas_stroke_ubo_map: &mut ResMap<CanvasTextStrokeColorUbo>,
    msdf_stroke_ubo_map: &mut ResMap<MsdfStrokeUbo>,
) -> bool {
    notify.modify_event(index, "", 0);
    // canvas 字体
    if is_pixel {
        let color = &text_stroke.color;
        let ubo = create_hash_res(
            CanvasTextStrokeColorUbo::new(UniformValue::Float4(color.r, color.g, color.b, color.a)),
            canvas_stroke_ubo_map,
        );
        render_obj.paramter.set_value("strokeColor", ubo);
        return false;
    }

    // msdf字体
    if text_stroke.width == 0.0 {
        //删除描边的宏
        match render_obj.vs_defines.remove("STROKE") {
            Some(_) => true,
            None => false,
        }
    } else {
        let ubo = if local_style & (StyleType::Stroke as usize) == 0 {
            class_ubo.stroke_ubo.clone()
        } else {
            let color = &text_stroke.color;
            create_hash_res(
                MsdfStrokeUbo::new(
                    UniformValue::Float1(text_stroke.width / 10.0),
                    UniformValue::Float4(color.r, color.g, color.b, color.a),
                ),
                msdf_stroke_ubo_map,
            )
        };
        render_obj.paramter.set_value("stroke", ubo);
        match render_obj.vs_defines.add("STROKE") {
            Some(_) => false,
            None => true,
        }
    }
}

#[inline]
fn modify_color<C: HalContext + 'static>(
    index: usize,
    _local_style: usize,
    color: &Color,
    engine: &mut Engine<C>,
    notify: &NotifyImpl,
    render_obj: &mut RenderObj,
    _class_ubo: &RenderCatch,
) -> bool {
    let change = match color {
        Color::RGBA(c) => {
            let ubo = create_hash_res(
                UColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a)),
                &mut *engine.u_color_ubo_map,
            );
            render_obj.paramter.set_value("uColor", ubo);
            notify.modify_event(index, "", 0);

            // 如果未找到UCOLOR宏， 表示修改之前的颜色不为RGBA， 应该删除VERTEX_COLOR宏， 添加UCOLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
            to_ucolor_defines(
                render_obj.vs_defines.as_mut(),
                render_obj.fs_defines.as_mut(),
            )
        }
        Color::LinearGradient(_) => {
            // 如果未找到VERTEX_COLOR宏， 表示修改之前的颜色不为LinearGradient， 应该删除UCOLOR宏， 添加VERTEX_COLOR宏，并尝试设顶点脏， 否则， 不需要做任何处理
            to_vex_color_defines(
                render_obj.vs_defines.as_mut(),
                render_obj.fs_defines.as_mut(),
            )
        }
    };
    change
}

#[inline]
fn modify_font(
    index: usize,
    render_obj: &mut RenderObj,
    is_pixel: bool,
    font_sheet: &FontSheet,
    notify: &NotifyImpl,
    default_sampler: &Share<SamplerRes>,
    point_sampler: &Share<SamplerRes>, // 点采样sampler
) {
    notify.modify_event(index, "ubo", 0);
    // 如果是canvas 字体绘制类型， 并且绘制fontsize 与字体本身fontsize一致， 应该使用点采样
    let sampler = if is_pixel {
        point_sampler
    } else {
        default_sampler
    };
    render_obj
        .paramter
        .set_texture("texture", (&font_sheet.get_font_tex().bind, &sampler));
}

#[inline]
fn modify_shadow_color<C: HalContext + 'static>(
    index: usize,
    _local_style: usize,
    text_style: &TextStyle,
    notify: &NotifyImpl,
    render_obj: &mut RenderObj,
    engine: &mut Engine<C>,
    is_pixel: bool,
    _class_ubo: &RenderCatch,
    canvas_stroke_ubo_map: &mut ResMap<CanvasTextStrokeColorUbo>,
) {
    let c = &text_style.shadow.color;
    if text_style.text.stroke.width > 0.0 && is_pixel {
        let ubo = create_hash_res(
            CanvasTextStrokeColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a)),
            canvas_stroke_ubo_map,
        );
        render_obj.paramter.set_value("strokeColor", ubo);
    }
    // let ubo = if local_style & (StyleType::TextShadow as usize) == 0 {
    //     class_ubo.shadow_color_ubo.clone()
    // } else {
    //     create_hash_res(engine,  UColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a)) )
    // };
    let ubo = create_hash_res(
        UColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a)),
        &mut *engine.u_color_ubo_map,
    );
    render_obj.paramter.set_value("uColor", ubo);
    render_obj.fs_defines.add("UCOLOR");
    notify.modify_event(index, "ubo", 0);
}

#[inline]
fn set_canvas_default_stroke(
    render_obj: &RenderObj,
    canvas_default_stroke_color: &Share<CanvasTextStrokeColorUbo>,
) {
    match render_obj.paramter.get_value("strokeColor") {
        Some(_) => (),
        None => {
            render_obj
                .paramter
                .set_value("strokeColor", canvas_default_stroke_color.clone());
        }
    };
}

// 返回position， uv， color， index
#[inline]
fn create_geo<C: HalContext + 'static>(
	dirty: usize,
	children: &NodeList,
	idtree: &SingleCaseImpl<IdTree>,
	node_state: &NodeState,
	layout: &LayoutR,
    // char_block: &CharBlock<L>,
    color: &Color,
    text: &TextContent,
    text_style: &TextStyle,
    font_sheet: &FontSheet,
    share_data: &RenderCatch,
    share_index_buffer: &Share<BufferRes>,
    index_buffer_max_len: &mut usize,
	engine: &mut Engine<C>,
	scale: f32,
) -> Option<Share<GeometryRes>> {
    // 是共享文字
    if text.0 == String::new() {
        let mut hasher = DefaultHasher::default();
        text.1.hash(&mut hasher);
        // 对于布局信息， 如果没有在style中设置， 可以直接使用class中的布局hash
        if dirty & TEXT_LAYOUT_DIRTY == 0 {
            share_data.layout_hash.hash(&mut hasher);
        } else {
            //在style中使用了文字布局属性, 重新计算文字布局属性的hash
            let layout_hash = text_layout_hash(&text_style.text, &text_style.font);
            layout_hash.hash(&mut hasher);
        }

        // 如果是渐变色， 计算渐变色的hash
        if let Color::LinearGradient(ref c) = color {
            c.hash(&mut hasher);
        }

        let hash = hasher.finish();
        // 从缓存中找到geo， 直接返回
        if let Some(geo) = engine.geometry_res_map.get(&hash) {
            return Some(geo);
        }

        // 缓存中不存在 对应的geo， 创建geo并缓存
        get_geo_flow(
			children,
			idtree,
			node_state,
			layout,
            color,
            font_sheet,
            engine,
            Some(hash),
            share_index_buffer,
			index_buffer_max_len,
			scale,
        )
    } else {
        // 如果文字不共享， 重新创建geo， 并且不缓存geo
        get_geo_flow(
			children,
			idtree,
			node_state,
			layout,
            color,
            font_sheet,
            engine,
            None,
            share_index_buffer,
			index_buffer_max_len,
			scale,
        )
    }
}

fn text_layout_hash(text_style: &Text, font: &Font) -> u64 {
    let mut hasher = DefaultHasher::default();
    let hasher = &mut hasher;
    NotNan::new(text_style.letter_spacing).unwrap().hash(hasher);
    NotNan::new(text_style.indent).unwrap().hash(hasher);
    NotNan::new(text_style.word_spacing).unwrap().hash(hasher);
    // TODO
    // match text_style.line_height {
    //     LineHeight::Normal => 0.hash(hasher),
    //     LineHeight::Length(r) => {
    //         1.hash(hasher);
    //         NotNan::new(r).unwrap().hash(hasher);
    //     },
    //     LineHeight::Number(r) => {
    //         2.hash(hasher);
    //         NotNan::new(r).unwrap().hash(hasher);
    //     },
    //     LineHeight::Percent(r) => {
    //         3.hash(hasher);
    //         NotNan::unew(r).unwrap().hash(hasher);
    //     },
    // };
    text_style.text_align.hash(hasher);
    text_style.white_space.hash(hasher);
    text_style.vertical_align.hash(hasher);
    font.weight.hash(hasher);
    match font.size {
        FontSize::None => 0.hash(hasher),
        FontSize::Length(r) => {
            1.hash(hasher);
            NotNan::new(r).unwrap().hash(hasher);
        }
        FontSize::Percent(r) => {
            3.hash(hasher);
            NotNan::new(r).unwrap().hash(hasher);
        }
    };
    font.style.hash(hasher);
    font.family.hash(hasher);
    hasher.finish()
}

// 返回position， uv， color， index
#[inline]
fn get_geo_flow<C: HalContext + 'static>(
	children: &NodeList,
	_idtree: &SingleCaseImpl<IdTree>,
	node_state: &NodeState,
	layout: &LayoutR,
    color: &Color,
    font_sheet: &FontSheet,
    engine: &mut Engine<C>,
    hash: Option<u64>,
    index_buffer: &Share<BufferRes>,
	index_buffer_max_len: &mut usize,
	scale: f32,
) -> Option<Share<GeometryRes>> {
    let mut positions: Vec<f32> = Vec::with_capacity(8 * children.len);
    let mut uvs: Vec<f32> = Vec::with_capacity(8 * children.len);
    // let font_height = char_block.font_height;
    let mut i = 0;
    let mut size = 0;

    let geo = engine.create_geometry();
    let mut geo_res = GeometryRes {
        geo: geo,
        buffers: Vec::with_capacity(3),
	};
	
	let rect = &layout.rect;
	let mut word_pos = (0.0, 0.0);
	let mut count = 0;
    match color {
		Color::RGBA(_) => {
			for c in node_state.0.text.iter(){
				if c.ch == char::from(0) {
					if c.ch_id_or_count > 0 {
						word_pos = c.pos;
						count = c.ch_id_or_count - 1;
					}
					continue;
				}
				if c.ch <= ' ' {
					continue;
				}

				let glyph = match font_sheet.get_glyph(c.ch_id_or_count) {
					Some(r) => r.1.clone(),
					None => continue,
				};
				if count > 0 {
					count -= 1;
					push_pos_uv(
						&mut positions,
						&mut uvs,
						word_pos.0 + c.pos.0,
						word_pos.1 + c.pos.1,
						&glyph,
						c.size.0,
						c.size.1,
						scale,
					);
				} else {
					push_pos_uv(
						&mut positions,
						&mut uvs,
						c.pos.0,
						c.pos.1,
						&glyph,
						c.size.0,
						c.size.1,
						scale,
					);
				}
			}
			// 更新buffer
			let l = positions.len() / 8;
			if l > *index_buffer_max_len {
				*index_buffer_max_len = l + 50;
				let buffer = create_index_buffer(*index_buffer_max_len);
				engine
					.gl
					.buffer_update(&index_buffer, 0, BufferData::Short(buffer.as_slice()));
			}
			engine
				.gl
				.geometry_set_indices_short_with_offset(
					&geo_res.geo,
					index_buffer,
					0,
					positions.len() / 8 * 6,
				)
				.unwrap();
		}
		Color::LinearGradient(color) => {
			let mut colors = vec![Vec::new()];
			let mut indices = Vec::with_capacity(6 * children.len);
			// let (start, end) = cal_all_size(children, idtree, node_state, layouts, font_sheet); // 渐变范围
																	 //渐变端点
			let endp = find_lg_endp(
				&[
					rect.start,
					rect.top,
					rect.start,
					rect.bottom,
					rect.end,
					rect.bottom,
					rect.end,
					rect.top,
				],
				color.direction,
			);

			let mut lg_pos = Vec::with_capacity(color.list.len());
			let mut lg_color = Vec::with_capacity(color.list.len() * 4);
			for v in color.list.iter() {
				lg_pos.push(v.position);
				lg_color.extend_from_slice(&[v.rgba.r, v.rgba.g, v.rgba.b, v.rgba.a]);
			}
			let lg_color = vec![LgCfg {
				unit: 4,
				data: lg_color,
			}];
			
			for c in node_state.0.text.iter(){
				if c.ch == char::from(0) {
					if c.ch_id_or_count > 0 {
						word_pos = c.pos;
						count = c.ch_id_or_count - 1;
					}
					continue;
				}
				if c.ch <= ' ' {
					continue;
				}

				let glyph = match font_sheet.get_glyph(c.ch_id_or_count) {
					Some(r) => r.1.clone(),
					None => continue,
				};

				if count > 0 {
					count -= 1;
					push_pos_uv(
						&mut positions,
						&mut uvs,
						word_pos.0 + c.pos.0,
						word_pos.1 + c.pos.1,
						&glyph,
						c.size.0,
						c.size.1,
						scale,
					);
				} else {
					push_pos_uv(
						&mut positions,
						&mut uvs,
						c.pos.0,
						c.pos.1,
						&glyph,
						c.size.0,
						c.size.1,
						scale,
					);
				}

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
				i = positions.len() as u16 / 2;
			}

			let colors = colors.pop().unwrap();
			let color_buffer = engine.create_buffer(
				BufferType::Attribute,
				colors.len(),
				Some(BufferData::Float(&colors)),
				false,
			);
			let i_buffer = engine.create_buffer(
				BufferType::Indices,
				indices.len(),
				Some(BufferData::Short(&indices)),
				false,
			);
			engine
				.gl
				.geometry_set_attribute(&geo_res.geo, &AttributeName::Color, &color_buffer, 4)
				.unwrap();
			engine
				.gl
				.geometry_set_indices_short(&geo_res.geo, &i_buffer)
				.unwrap();
			geo_res.buffers.push(Share::new(BufferRes(i_buffer)));
			geo_res.buffers.push(Share::new(BufferRes(color_buffer)));
			size += buffer_size(indices.len(), BufferType::Indices);
			size += buffer_size(colors.len(), BufferType::Attribute);
		}
	}

	let position_buffer = engine.create_buffer(
		BufferType::Attribute,
		positions.len(),
		Some(BufferData::Float(&positions)),
		false,
	);
	let uv_buffer = engine.create_buffer(
		BufferType::Attribute,
		uvs.len(),
		Some(BufferData::Float(&uvs)),
		false,
	);
	engine
		.gl
		.geometry_set_attribute(&geo_res.geo, &AttributeName::Position, &position_buffer, 2)
		.unwrap();
	engine
		.gl
		.geometry_set_attribute(&geo_res.geo, &AttributeName::UV0, &uv_buffer, 2)
		.unwrap();
	geo_res.buffers.push(Share::new(BufferRes(uv_buffer)));
	geo_res.buffers.push(Share::new(BufferRes(position_buffer)));
	size += buffer_size(positions.len(), BufferType::Attribute);
	size += buffer_size(uvs.len(), BufferType::Attribute);

	Some(match hash {
		Some(hash) => engine.geometry_res_map.create(hash, geo_res, size, 0),
		None => Share::new(geo_res),
	})
}

#[inline]
// fn cal_all_size<>(
// 	children: &NodeList,
// 	idtree: &SingleCaseImpl<IdTree>,
// 	node_state: &NodeState,
// 	layouts: &MultiCaseImpl<Node, LayoutR>,
//     _font_sheet: &FontSheet,
// ) -> (Point2, Point2) {
//     let mut start = Point2::new(std::f32::MAX, std::f32::MAX);
//     let mut end = Point2::new(std::f32::MIN, std::f32::MIN);

// 	if children.len == 0 {
// 		return (Point2::new(0.0, 0.0), Point2::new(0.0, 0.0));
// 	}

//     for n in idtree.iter(children.head) {
// 		let ch = &node_states[n.0];
// 		let layout = &layouts[n.0];

//         if layout.rect.start < start.x {
//             start.x = layout.rect.start;
//         }
//         let end_x = layout.rect.start + ch.width;
//         if end_x > end.x {
//             end.x = end_x;
//         }
//         if layout.rect.top < start.y {
//             start.y = layout.rect.top;
//         }
//         let end_y = layout.rect.bottom - layout.rect.top;
//         if end_y > end.y {
//             end.y = end_y;
// 		}
// 	}
//     (start, end)
// }

#[inline]
fn fill_uv(positions: &mut Vec<f32>, uvs: &mut Vec<f32>, i: usize) {
    let pi = i * 2;
    let uvi = i * 2;
    let len = positions.len() - pi;
    let (p1, p4) = (
        (positions[pi], positions[pi + 1]),
        (positions[pi + 4], positions[pi + 5]),
    );
    let (u1, u4) = ((uvs[uvi], uvs[uvi + 1]), (uvs[uvi + 4], uvs[uvi + 5]));
    if len > 8 {
        let mut i = pi + 8;
        for _j in 0..(len - 8) / 2 {
            let pos_x = positions[i];
            let pos_y = positions[i + 1];
            let uv;
            if (pos_x - p1.0).abs() < 0.001 {
                let base = p4.1 - p1.1;
                let ratio = if base == 0.0 {
                    0.0
                } else {
                    (pos_y - p1.1) / (p4.1 - p1.1)
                };
                uv = (u1.0, u1.1 * (1.0 - ratio) + u4.1 * ratio);
            } else if (pos_x - p4.0).abs() < 0.001 {
                let base = p4.1 - p1.1;
                let ratio = if base == 0.0 {
                    0.0
                } else {
                    (pos_y - p1.1) / (p4.1 - p1.1)
                };
                uv = (u4.0, u1.1 * (1.0 - ratio) + u4.1 * ratio);
            } else if (pos_y - p1.1).abs() < 0.001 {
                let base = p4.0 - p1.0;
                let ratio = if base == 0.0 {
                    0.0
                } else {
                    (pos_x - p1.0) / (p4.0 - p1.0)
                };
                uv = (u1.0 * (1.0 - ratio) + u4.0 * ratio, u1.1);
            } else {
                // }else if pos_y == p4.1{
                let base = p4.0 - p1.0;
                let ratio = if base == 0.0 {
                    0.0
                } else {
                    (pos_x - p1.0) / (p4.0 - p1.0)
                };
                uv = (u1.0 * (1.0 - ratio) + u4.0 * ratio, u4.1);
            }
            uvs.push(uv.0);
            uvs.push(uv.1);
            i += 2;
        }
    }
}

fn push_pos_uv(
    positions: &mut Vec<f32>,
    uvs: &mut Vec<f32>,
	x: f32,
	mut y: f32,
    glyph: &Glyph,
	width: f32,
	height: f32,
	scale: f32,
) {
	let ratio = 1.0/scale;
	let w = glyph.width.ceil();
	let h = glyph.height.ceil();
	// height为行高， 当行高高于字体高度时，需要居中
	y += (height - ratio * glyph.height)/2.0;
    let left_top = (
        x + ratio * glyph.ox,
        ((y  + glyph.oy * ratio) * scale).ceil()/scale, // 保证顶点对应整数像素
    );
    let right_bootom = (
        left_top.0 + w / scale,
        left_top.1 + h / scale,
	);

    let ps = [
        left_top.0,
        left_top.1,
        left_top.0,
        right_bootom.1,
        right_bootom.0,
        right_bootom.1,
        right_bootom.0,
        left_top.1,
	];
	let uv = [
        glyph.x,
        glyph.y,
        glyph.x,
        glyph.y + h,
        glyph.x + w,
        glyph.y + h,
        glyph.x + w,
        glyph.y,
	];
    uvs.extend_from_slice(&uv);
    positions.extend_from_slice(&ps[..]);
}

fn create_index_buffer(count: usize) -> Vec<u16> {
    let mut index_data: Vec<u16> = Vec::with_capacity(count * 6);
    let mut i: u16 = 0;
    while (i as usize) < count * 4 {
        index_data.extend_from_slice(&[i, i + 1, i + 2, i, i + 2, i + 3]);
        i += 4;
    }
    index_data
}

impl_system! {
    CharBlockSys<C> where [C: HalContext + 'static],
    true,
    {
        MultiCaseListener<Node, TextStyle, DeleteEvent>
    }
}
