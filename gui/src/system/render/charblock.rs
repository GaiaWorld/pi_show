use std::hash::{Hash, Hasher};
/**
 * 文字渲染对象的构建及其属性设置
 */
use std::marker::PhantomData;

use hash::{DefaultHasher, XHashMap};
use ordered_float::NotNan;

use ecs::monitor::{Event, NotifyImpl};
use ecs::{
    DeleteEvent, MultiCaseImpl, EntityListener, Runner, SingleCaseImpl,
	StdCell,
};
use idtree::NodeList;
use hal_core::*;
use map::vecmap::VecMap;
use pi_atom::Atom;
use pi_polygon::{find_lg_endp, interp_mult_by_lg, mult_to_triangle, split_by_lg, LgCfg};
use res::ResMap;
use share::Share;
use smallvec::SmallVec;

use crate::component::{calc::*, calc::LayoutR};
use crate::component::user::*;
use crate::entity::Node;
use crate::font::font_sheet::*;
use crate::render::engine::{buffer_size, create_hash_res, Engine, ShareEngine, UnsafeMut};
use crate::render::res::*;
use crate::single::*;
use crate::system::render::shaders::canvas_text::{
    CANVAS_TEXT_FS_SHADER_NAME, CANVAS_TEXT_VS_SHADER_NAME,
};
use crate::system::render::shaders::image::{
	IMAGE_VS_SHADER_NAME, IMAGE_FS_SHADER_NAME,
};
use crate::system::render::shaders::text::{TEXT_FS_SHADER_NAME, TEXT_VS_SHADER_NAME};
use crate::system::util::*;

use super::gassu_blur::add_gassu_blur;
lazy_static! {
	static ref TEXT_STYLE_DIRTY: StyleBit = style_bit().set_bit(StyleType::LetterSpacing as usize)
		.set_bit(StyleType::WordSpacing as usize)
		.set_bit(StyleType::LineHeight as usize)
		.set_bit(StyleType::TextIndent as usize)
		.set_bit(StyleType::WhiteSpace as usize)
		.set_bit(StyleType::TextAlign as usize)
		.set_bit(StyleType::VerticalAlign as usize)
		.set_bit(StyleType::Color as usize)
		.set_bit(StyleType::TextStroke as usize)
		.set_bit(StyleType::FontStyle as usize)
		.set_bit(StyleType::FontFamily as usize)
		.set_bit(StyleType::FontSize as usize)
		.set_bit(StyleType::FontWeight as usize)
		.set_bit(StyleType::TextShadow as usize)
		.set_bit(StyleType::TextContent as usize);
	static ref TEXT_LAYOUT_DIRTY: StyleBit = style_bit().set_bit(StyleType::FontStyle as usize)
		.set_bit(StyleType::FontWeight as usize)
		.set_bit(StyleType::FontSize as usize)
		.set_bit(StyleType::FontFamily as usize)
		.set_bit(StyleType::LetterSpacing as usize)
		.set_bit(StyleType::WordSpacing as usize)
		.set_bit(StyleType::LineHeight as usize)
		.set_bit(StyleType::TextIndent as usize)
		.set_bit(StyleType::WhiteSpace as usize)
		.set_bit(StyleType::TextAlign as usize)
		.set_bit(StyleType::TextContent as usize)
		.set_bit(StyleType::VerticalAlign as usize);

	static ref FONT_DIRTY: StyleBit = style_bit().set_bit(StyleType::FontStyle as usize)
		.set_bit(StyleType::FontWeight as usize)
		.set_bit(StyleType::FontSize as usize)
		.set_bit(StyleType::FontFamily as usize);
}


#[derive(Default, Clone, Debug)]
struct I {
    text: usize,
    shadow: SmallVec<[usize; 1]>,
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
	msdf_texture_size_ubo: XHashMap<usize, Share<TextTextureSize>>,
	
	old_texture_tex_version: usize,

    msdf_stroke_ubo_map: UnsafeMut<ResMap<MsdfStrokeUbo>>,
    canvas_stroke_ubo_map: UnsafeMut<ResMap<CanvasTextStrokeColorUbo>>,

    msdf_default_paramter: MsdfParamter,
    canvas_default_paramter: CanvasTextParamter,
    mark: PhantomData<C>,
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
		&'a MultiCaseImpl<Node, ContentBox>,
        &'a SingleCaseImpl<Share<StdCell<FontSheet>>>,
        &'a SingleCaseImpl<DefaultState>,
		&'a SingleCaseImpl<DirtyList>,
		&'a SingleCaseImpl<IdTree>,
		&'a SingleCaseImpl<PixelRatio>,
		&'a SingleCaseImpl<UnitQuad>,
		&'a SingleCaseImpl<PremultiState>,
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
			context_boxs,
            font_sheet,
            default_state,
			dirty_list,
			idtree,
			pixel_ratio,
			unit_quad,
			premulti_state,
		) = read;
		let font_sheet = &font_sheet.borrow();
		let t = font_sheet.get_font_tex();
		let mut texture_change = font_sheet.tex_change(&mut self.old_texture_tex_version);

        if dirty_list.0.len() == 0 && !texture_change {
            return;
        }
        let (render_objs, engine, node_states) = write;
        let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };

		// 文字纹理只有一张，文字在数量增加的过程中，纹理会被更新
		// 纹理更新后，需要重新设置纹理的尺寸
        if texture_change == true {
            self.texture_size_ubo = Share::new(TextTextureSize::new(UniformValue::Float2(
                t.width as f32,
                t.height as f32,
            )));
            for i in self.render_map.iter() {
                match i {
                    Some(i) => {
						let render_obj = &render_objs[i.text];
						// let is_pixel = match font_sheet.get_src(&text_styles[render_obj.context].font.family) {
						// 	Some(r) => r.is_pixel,
						// 	None => continue,
						// };
						// // canvas文字才会更新
						// if !is_pixel {
						// 	continue;
						// }
                        render_obj
                            .paramter
                            .set_value("textureSize", self.texture_size_ubo.clone());
                        if i.shadow.len() > 0 {
							for ii in i.shadow.iter() {
								if *ii > 0 {
									render_objs[*ii]
										.paramter
										.set_value("textureSize", self.texture_size_ubo.clone());
								}
							}
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
			let node = match idtree.get(*id) {
				Some(r) => r,
				None => continue,
			};
			if node.layer() == 0 {
				continue;
			}

            let mut dirty = style_mark.dirty;
			let mut dirty1 = style_mark.dirty1;

            // 不存在Chablock关心的脏, 跳过
            if !(dirty & &*TEXT_STYLE_DIRTY).any() && dirty1 & GEO_DIRTY_TYPE == 0 {
                continue;
			}

			let tex_font;
            // 如果FontFamily脏， 并需要删除原来的renderobj， 重新创建新的
			// 如果原有renderobj不存在，也需要创建新的
            let (index, children, text_style) = if dirty[StyleType::FontFamily as usize] || self.render_map.get(*id).is_none() {
				if dirty[StyleType::FontFamily as usize] {
					self.remove_render_obj(*id, render_objs); // 可能存在旧的render_obj， 先尝试删除（FontFamily修改， 整renderobj中大部分值都会修改， 因此直接重新创建）
				}
                
                let text_style = &text_styles[*id];
                let children = idtree[*id].children();
				tex_font = match font_sheet.get_first_src(&text_style.font.family) {
					Some(r) => r,
					None => continue,
				};
				
				let r = self.create_render_obj(
                    *id,
                    render_objs,
                    default_state,
                    tex_font,//charblock.is_pixel,
					text_style,
					pixel_ratio.0,
                );

				// 创建阴影的渲染对象
				if text_style.shadow.len() > 0 {
					for s in text_style.shadow.iter() {
						let shadow_index = self.create_render_obj1(*id, render_objs, default_state, tex_font, text_style, 0.0, pixel_ratio.0);
						self.render_map[*id].shadow.push(shadow_index);
					}
				}

                dirty = dirty | &*TEXT_STYLE_DIRTY;
				dirty1 = dirty1 | GEO_DIRTY_TYPE;
                (self.render_map[*id].clone(), children, text_style)
            } else {
                match self.render_map.get(*id) {
                    Some(r) => {
						tex_font = match font_sheet.get_first_src(&text_styles[*id].font.family) {
							Some(r) => r,
							None => continue,
						};
						(
							r.clone(),
							idtree[*id].children(),
							&text_styles[*id],
						)
					},
                    None => continue,
                }
            };
			let is_pixel = tex_font.is_pixel;
			let font_size = get_size(0, &text_style.font.size) as f32;
			let font_height = tex_font.get_font_height(font_size as usize, *text_style.text.stroke.width);
			let world_matrix = &world_matrixs[*id];
			let layout = &layouts[*id];
            let transform = &transforms[*id];

            let render_obj = unsafe {
                &mut *(&render_objs[index.text] as *const RenderObj as usize
                    as *mut RenderObj)
            };

            let class_ubo = &self.default_ubos;

            let (mut program_change, mut geometry_change) = (false, false);
            let mut shadow_geometry_change = false;

            // 颜色脏， 如果是渐变色和纯色切换， 需要修改宏， 且顶点需要重新计算， 因此标记program_change 和 geometry_change脏
            if dirty[StyleType::Color as usize] {
                let exchange = modify_color(
                    index.text,
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
            if dirty[StyleType::TextStroke as usize] {
                program_change = program_change
                    | modify_stroke(
                        index.text,
                        style_mark.local_style,
                        &text_style.text.stroke,
                        render_obj,
                        &notify,
                        tex_font,//charblock.is_pixel,
                        &class_ubo,
                        &mut *self.canvas_stroke_ubo_map,
                        &mut *self.msdf_stroke_ubo_map,
                    );
            }
			if !tex_font.is_pixel {
				if dirty[StyleType::TextStroke as usize] || dirty1 & (CalcType::Matrix as usize) != 0 || dirty[StyleType::FontWeight as usize] {
					let mut scale1 = node_states[*id].0.scale;
					let scale = scale1 * font_height / (tex_font.metrics.ascender - tex_font.metrics.descender);
					let sw = (scale1 * *text_style.text.stroke.width).round();
					let distance_px_range = scale * tex_font.metrics.distance_range;
					let mut fill_bound = 0.5 - (text_style.font.weight as f32 / 500 as f32 - 1.0) / distance_px_range ;
					let stroke = sw/2.0/distance_px_range;
					let stroke_bound = fill_bound - stroke;
					// fill_bound = fill_bound + stroke;
					// log::info!("=====state_scale:{:?}, scale: {}, font_height:{:?}, sw: {:?}, stroke_width: {:?}, distance_px_range: {:?}, ", node_states[*id].0.scale, scale, font_height, sw, text_style.text.stroke.width, distance_px_range);
					render_obj.paramter.set_single_uniform("line", UniformValue::Float4(distance_px_range, fill_bound, stroke_bound, 0.0));
					// log::info!("set line======================={:?}", index);
				}
			}
            // 尝试修改字体， 如果字体类型修改（dyn_type）， 需要修改pipeline， （字体类型修改应该重新创建paramter， TODO）
            if (dirty & &*FONT_DIRTY).any() {
                modify_font(
                    index.text,
                    render_obj,
                    tex_font,//charblock.is_pixel,
                    &font_sheet,
                    &notify,
                    &self.default_sampler,
                    &self.point_sampler,
                );
                program_change = true;
            }

            // 文字内容脏， 这是顶点流脏
            if (dirty & &*TEXT_LAYOUT_DIRTY).any() || dirty1 & CalcType::Layout as usize != 0 {
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
					dirty1,
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
					node_states[*id].0.scale,
					is_pixel,
					font_height,
                );
                render_objs
                    .get_notify_ref()
                    .modify_event(index.text, "geometry", 0);
			}
			let (mut h, mut v) = (0.0, 0.0);
			if node_states[*id].0.is_vnode() {
				h = -layout.rect.left;
				v = -layout.rect.top;
			}

            // 矩阵脏
            if dirty1 & (CalcType::Matrix as usize) != 0 {
				
                modify_matrix(
                    index.text,
                    create_let_top_offset_matrix(
                        layout,
                        world_matrix,
                        transform,
                        h,
                        v,
                        // render_obj.depth,
                    ),
                    render_obj,
                    &notify,
                );
            }
            notify.modify_event(index.text, "", 0);

			if index.shadow.len() > 0 {
				for i in 0..index.shadow.len() {
					let shadow_index = index.shadow[i];
					// 阴影存在
					if shadow_index > 0 {
						let shadow_render_obj = &mut render_objs[shadow_index];
						let shadow = &text_style.shadow[i];
						let mut has_blur = (false, false);

						let context_box = &context_boxs.get(*id).unwrap().0;
		
						if dirty[StyleType::TextShadow as usize] {
							// 阴影颜色脏，或描边脏， 修改ubo
							modify_shadow_color(
								shadow_index,
								text_style,
								&text_style.shadow[i].color,
								&notify,
								shadow_render_obj,
								engine,
								tex_font, // charblock.is_pixel,
								&class_ubo,
								&mut *self.canvas_stroke_ubo_map,
								&mut *self.msdf_stroke_ubo_map,
							);
						}
						if dirty[StyleType::TextShadow as usize] || dirty1 & (CalcType::Matrix as usize) != 0 {
							has_blur = add_gassu_blur(
								shadow_index,
								shadow.blur,
								shadow_render_obj,
								engine,
								context_box,
								&premulti_state.0
							);
							if has_blur.0 {
								shadow_render_obj.state.bs = default_state.one_one_bs.clone();
							} else {
								shadow_render_obj.state.bs = default_state.df_bs.clone();
							}
							// has_blur = modify_shadow_gassu_blur(
							// 	shadow_index,
							// 	shadow.blur,
							// 	shadow_render_obj,
							// 	engine,
							// 	Size{width: t.width, height: t.height},
							// 	&context_boxs.get(*id).unwrap().0,
							// 	&premulti_state.0,
							// );
							// 设置ubo TODO
						}
		
						// 尝试修改字体， 如果字体类型修改（dyn_type）， 需要修改pipeline， （字体类型修改应该重新创建paramter， TODO）
						if (dirty & &*FONT_DIRTY).any() {
							modify_font(
								index.text,
								shadow_render_obj,
								tex_font, //charblock.is_pixel,
								&font_sheet,
								&notify,
								&self.default_sampler,
								&self.point_sampler,
							);
						}
		
						if program_change {
							notify.modify_event(shadow_index, "program_dirty", 0);
						}
		
						// 修改阴影的顶点流
						if shadow_geometry_change {
							// 如果填充色是纯色， 阴影的geo和文字的geo一样， 否则重新创建阴影的geo
							match &text_style.text.color {
								Color::RGBA(_) => shadow_render_obj.geometry = render_obj.geometry.clone(),
								Color::LinearGradient(_) => {
									let color = shadow.color.clone();
									let l = &mut self.index_len;
									shadow_render_obj.geometry = create_geo(
										dirty,
										dirty1,
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
										is_pixel,
										font_height,
										
									)
								}
							}
							notify.modify_event(shadow_index, "geometry", 0);
						}
		
						if dirty1 & (CalcType::Matrix as usize) != 0
							|| dirty[StyleType::TextShadow as usize]
						{
							modify_matrix(
								shadow_index,
								create_let_top_offset_matrix(
									layout,
									world_matrix,
									transform,
									h,
									v,
									// shadow_render_obj.depth,
								),
								shadow_render_obj,
								&notify,
							);
						}
		
						if has_blur.0 {
							if has_blur.1 {
								
								let copy= new_render_obj(
									*id,
									0.0,
									false,
									IMAGE_VS_SHADER_NAME.clone(),
									IMAGE_FS_SHADER_NAME.clone(),
									Share::new(ImageParamter::default()),
									State {
										bs: default_state.df_bs.clone(),
										rs: default_state.df_rs.clone(),
										ss: default_state.df_ss.clone(),
										ds: default_state.tarns_ds.clone(),
									},
								);
								
				
								let notify = render_objs.get_notify();
								let copy = render_objs.insert(copy, Some(&notify));
								// log::warn!("create copy================={}, {}, {}, {}",id, shadow_index, render_objs[shadow_index].post_process.as_deref_mut().unwrap().copy, copy);
								render_objs[shadow_index].post_process.as_deref_mut().unwrap().copy = copy;
							}
							let copy_index = render_objs[shadow_index].post_process.as_deref_mut().unwrap().copy;
							let arr = create_unit_offset_matrix(
								layout.rect.right - layout.rect.left,
								layout.rect.bottom - layout.rect.top,
								layout.border.left + shadow.h,
								layout.border.top + shadow.v,
								layout,
								world_matrix,
								transform,
								0.0,
							);
						
							let rr = &mut render_objs[copy_index];
							rr.paramter.set_value(
								"worldMatrix",
								Share::new(WorldMatrixUbo::new(UniformValue::MatrixV4(arr))),
							);
		
							let geo = engine.create_geometry();
							engine
								.gl
								.geometry_set_attribute(
									&geo,
									&AttributeName::Position,
									&unit_quad.0.buffers[1],
									2,
								)
								.unwrap();
							engine
								.gl
								.geometry_set_indices_short(&geo, &unit_quad.0.buffers[0])
								.unwrap();
							let geo_res = GeometryRes {
								geo: geo,
								buffers: vec![],
							};
							rr.geometry =
								Some(Share::new(geo_res));
							notify.modify_event(copy_index, "", 0);
						}
		
						notify.modify_event(shadow_index, "", 0);
					}
				}
			}
            
        }
    }
}

// 监听实体销毁，删除索引
impl<'a, C: HalContext + 'static> EntityListener<'a, Node, DeleteEvent>
    for CharBlockSys<C>
{
    type ReadData = ();
    type WriteData = ();

    fn listen(&mut self, event: &Event, _read: Self::ReadData, _: Self::WriteData) {
		self.render_map.remove(event.id); // 移除索引
    }
}

impl<C: HalContext + 'static> CharBlockSys<C> {
    #[inline]
    pub fn with_capacity(engine: &mut Engine<C>, texture_size: (usize, usize), capacity: usize) -> Self {
        let mut canvas_bs = BlendStateDesc::default();
		canvas_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
		canvas_bs.set_alpha_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);
		// canvas_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        let canvas_bs = engine.create_bs_res(canvas_bs);

        let mut msdf_bs = BlendStateDesc::default();
        msdf_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
		msdf_bs.set_alpha_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);
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
			msdf_texture_size_ubo: XHashMap::default(),
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
        tex_font: &TexFont,
		text_style: &TextStyle,
		pixel_ratio: f32,
    ) -> I {
        // let shadow_index = if !have_shadow {
        //     0
        // } else {
        //     self.create_render_obj1(id, render_objs, default_state, tex_font, text_style, 0.0, pixel_ratio)
        // };
        let index = self.create_render_obj1(id, render_objs, default_state, tex_font, text_style,  0.1, pixel_ratio);

        // 创建RenderObj与Node实体的索引关系， 并设脏
        self.render_map.insert(
            id,
            I {
                text: index,
                shadow: SmallVec::default(),
            },
        );
        self.render_map[id].clone()
    }

    fn create_render_obj1(
        &mut self,
        id: usize,
        render_objs: &mut SingleCaseImpl<RenderObjs>,
        default_state: &DefaultState,
        tex_font: &TexFont,
		text_style: &TextStyle,
        depth_diff: f32,
		pixel_ratio: f32,
    ) -> usize {
        let (vs_name, fs_name, paramter, bs) = if tex_font.is_pixel {
            let paramter: Share<dyn ProgramParamter> =
                Share::new(self.canvas_default_paramter.clone());
			paramter.set_value("textureSize", self.texture_size_ubo.clone());
            paramter.set_value("strokeColor", self.canvas_default_stroke_color.clone());
            (
                CANVAS_TEXT_VS_SHADER_NAME.clone(),
                CANVAS_TEXT_FS_SHADER_NAME.clone(),
                paramter,
                self.canvas_bs.clone(),
            )
        } else {
            let paramter: Share<dyn ProgramParamter> =
                Share::new(self.msdf_default_paramter.clone());
			paramter.set_value("textureSize", self.texture_size_ubo.clone());
			paramter.set_value("strokeColor", self.canvas_default_stroke_color.clone());
			paramter.set_single_uniform("line", UniformValue::Float4(tex_font.metrics.distance_range, 0.5, 0.5, 0.0));
			// paramter.set_single_uniform("line", UniformValue::Float4(0.0, 0.0, 0.0, tex_font.distance_px_range));
			// let texture = &tex_font.textures.as_ref().unwrap()[0];
			// let size_ubo = self.msdf_texture_size_ubo.entry(tex_font.name).or_insert_with(|| {
			// 	Share::new(TextTextureSize::new(UniformValue::Float2(
			// 		texture.width as f32,
			// 		texture.height as f32,
			// 	)))
			// });
			// paramter.set_value("textureSize", size_ubo.clone());
			// paramter.set_single_uniform("pixelRatio", UniformValue::Float1(pixel_ratio));
			// paramter.set_single_uniform("weight", UniformValue::Float1(text_style.font.weight as f32));
			// paramter.set_single_uniform("fontSize", UniformValue::Float1(get_size(32, &text_style.font.size) as f32));
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
			let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
        render_objs.insert(render_obj, Some(notify))
    }

    #[inline]
    fn remove_render_obj(&mut self, id: usize, render_objs: &mut SingleCaseImpl<RenderObjs>) {
        match self.render_map.remove(id) {
            Some(index) => {
                let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
                render_objs.remove(index.text, Some(notify));
				if index.shadow.len() > 0 {
					for shadow_index in index.shadow.iter() {
						// log::warn!("del shadow============obj: {:?}, context: {:?}, copy: {}", shadow_index, id, render_objs[*shadow_index].post_process.as_deref_mut().unwrap().copy);
						render_objs.remove(*shadow_index, Some(notify));
					}
					
				}
            }
            None => (),
        };
    }
}

#[inline]
fn modify_stroke(
    index: usize,
    local_style: StyleBit,
    text_stroke: &Stroke,
    render_obj: &mut RenderObj,
    // engine: &mut SingleCaseImpl<Engine<C>>,
    notify: &NotifyImpl,
    tex_font: &TexFont,
    class_ubo: &RenderCatch,
    canvas_stroke_ubo_map: &mut ResMap<CanvasTextStrokeColorUbo>,
    msdf_stroke_ubo_map: &mut ResMap<MsdfStrokeUbo>,
) -> bool {
    notify.modify_event(index, "", 0);
	if text_stroke.width == 0.0 {
		//删除描边的宏
		match render_obj.fs_defines.remove("STROKE") {
			Some(r) => true,
			None => false
		}
	} else {
		let color = &text_stroke.color;
		let ubo = create_hash_res(
			CanvasTextStrokeColorUbo::new(UniformValue::Float4(color.x, color.y, color.z, color.w)),
			canvas_stroke_ubo_map,
		);
		render_obj.paramter.set_value("strokeColor", ubo);

		match render_obj.fs_defines.add("STROKE") {
			Some(_) => false,
			None => true,
		}
	}

    // // canvas 字体
    // if tex_font.is_pixel {
    //     let color = &text_stroke.color;
    //     let ubo = create_hash_res(
    //         CanvasTextStrokeColorUbo::new(UniformValue::Float4(color.r, color.g, color.b, color.a)),
    //         canvas_stroke_ubo_map,
    //     );
    //     render_obj.paramter.set_value("strokeColor", ubo);

		
    //     return false;
    // }

    // // msdf字体
    // if text_stroke.width == 0.0 {
    //     //删除描边的宏
    //     match render_obj.fs_defines.remove("STROKE") {
    //         Some(_) => true,
    //         None => false,
    //     }
    // } else {
	// 	// paramter.set_single_uniform("line", UniformValue::Float4(0.0, 0.0, 0.0, tex_font.distance_px_range));
	// 	// let stroke
	// 	if tex_font.is_pixel {
	// 		let ubo = if local_style & (StyleType::Stroke as usize) == 0 {
	// 			class_ubo.stroke_ubo.clone()
	// 		} else {
	// 			let color = &text_stroke.color;
	// 			create_hash_res(
	// 				MsdfStrokeUbo::new(
	// 					UniformValue::Float1(text_stroke.width),
	// 					UniformValue::Float4(color.r, color.g, color.b, color.a),
	// 				),
	// 				msdf_stroke_ubo_map,
	// 			)
	// 		};
	// 		render_obj.paramter.set_value("stroke", ubo);
	// 		match render_obj.fs_defines.add("STROKE") {
	// 			Some(_) => false,
	// 			None => true,
	// 		}
	// 	}
    // }
}

#[inline]
fn modify_color<C: HalContext + 'static>(
    index: usize,
    color: &Color,
    engine: &mut Engine<C>,
    notify: &NotifyImpl,
    render_obj: &mut RenderObj,
    _class_ubo: &RenderCatch,
) -> bool {
    let change = match color {
        Color::RGBA(c) => {
            let ubo = create_hash_res(
                UColorUbo::new(UniformValue::Float4(c.x, c.y, c.z, c.w)),
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
    tex_font: &TexFont,
    font_sheet: &FontSheet,
    notify: &NotifyImpl,
    default_sampler: &Share<SamplerRes>,
    point_sampler: &Share<SamplerRes>, // 点采样sampler
) {
    notify.modify_event(index, "ubo", 0);
    // 如果是canvas 字体绘制类型， 并且绘制fontsize 与字体本身fontsize一致， 应该使用点采样
    let sampler = if tex_font.is_pixel {
		render_obj
        .paramter
        .set_texture("texture", (&font_sheet.get_font_tex().bind, &point_sampler));
    } else {
		render_obj
        .paramter
        .set_texture("texture", (&font_sheet.get_font_tex().bind, &default_sampler));
		// render_obj
        // .paramter
        // .set_texture("texture", (&tex_font.textures.as_ref().unwrap()[0].bind, &default_sampler));
    };
}

#[inline]
fn modify_shadow_color<C: HalContext + 'static>(
    index: usize,
    text_style: &TextStyle,
	c: &CgColor,
    notify: &NotifyImpl,
    render_obj: &mut RenderObj,
    engine: &mut Engine<C>,
    tex_font: &TexFont,
    _class_ubo: &RenderCatch,
    canvas_stroke_ubo_map: &mut ResMap<CanvasTextStrokeColorUbo>,
	msdf_stroke_ubo_map: &mut ResMap<MsdfStrokeUbo>,
) {
	if *text_style.text.stroke.width > 0.0 {
		let ubo = create_hash_res(
			CanvasTextStrokeColorUbo::new(UniformValue::Float4(c.x, c.y, c.z, c.w)),
			canvas_stroke_ubo_map,
		);
		render_obj.paramter.set_value("strokeColor", ubo);
	}

    // if text_style.text.stroke.width > 0.0 && tex_font.is_pixel {
    //     let ubo = create_hash_res(
    //         CanvasTextStrokeColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a)),
    //         canvas_stroke_ubo_map,
    //     );
    //     render_obj.paramter.set_value("strokeColor", ubo);
    // } else if text_style.text.stroke.width > 0.0 {
	// 	let stroke_color_ubo = create_hash_res(
    //         MsdfStrokeUbo::new(UniformValue::Float1(text_style.text.stroke.width), UniformValue::Float4(c.r, c.g, c.b, c.a)) ,
    //         msdf_stroke_ubo_map,
    //     );
	// 	render_obj.paramter.set_value("stroke", stroke_color_ubo);
	// }
    // let ubo = if local_style & (StyleType::TextShadow as usize) == 0 {
    //     class_ubo.shadow_color_ubo.clone()
    // } else {
    //     create_hash_res(engine,  UColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a)) )
    // };
    let ubo = create_hash_res(
        UColorUbo::new(UniformValue::Float4(c.x, c.y, c.z, c.w)),
        &mut *engine.u_color_ubo_map,
    );
    render_obj.paramter.set_value("uColor", ubo);
    render_obj.fs_defines.add("UCOLOR");
    notify.modify_event(index, "ubo", 0);
}

// /// 对阴影应用高斯模糊
// fn modify_shadow_gassu_blur<C: HalContext + 'static>(
//     index: usize,
// 	blur: f32,
//     render_obj: &mut RenderObj,
//     engine: &mut Engine<C>,
// 	mut texture_size: Size<usize>,
// 	content_box: &Aabb2,
// 	default_state: &CommonState,
// ) -> (bool, bool) {
// 	// log::warn!("modify_shadow_gassu_blur======================blur:{}", blur);
// 	match &mut render_obj.post_process {
// 		Some(r) => {
// 			if blur==0.0 {
// 				render_obj.post_process = None;
// 				render_obj.state.bs = default_state.df_bs.clone();
// 				return (false, false)
// 			} else {
// 				for i in 0..r.post_processes.len() {
// 					r.post_processes[i].render_obj.paramter.set_single_uniform(
// 						"blurRadius", 
// 						UniformValue::Float2(blur/texture_size.width as f32, blur/texture_size.height as f32/10.0)
// 					);
// 				}
// 				r.content_box = content_box.clone();
// 				return (true, r.copy == 0);
// 			}
// 		},
// 		None => {
// 			if blur != 0.0 {
// 				render_obj.state.bs = default_state.one_one_bs.clone();
// 				let width = content_box.maxs.x - content_box.mins.x;
// 				let height = content_box.maxs.y - content_box.mins.y;

// 				// log::warn!("blur1================={}, {}", width, height);
// 				// log::warn!("size=================blur: {}, width: {}, height: {}", blur, texture_size.width, texture_size.height);

// 				let mut vv = Vec::new();

// 				for i in 0..2 {
// 					let (mut w, mut h) = (width, height);
// 					let p: Share<dyn ProgramParamter> = Share::new(GaussBlurParamter::default());
// 					p.set_single_uniform(
// 						"blurRadius", 
// 						UniformValue::Float1(blur)
// 					);	
// 					w = w.round().max(1.0);
// 					h = h.round().max(1.0);

// 					let mut obj = new_render_obj1(
// 						index, 0.0, false, GAUSS_BLUR_VS_SHADER_NAME.clone(), GAUSS_BLUR_FS_SHADER_NAME.clone(), p, default_state,
// 					);
// 					if i == 0 {
// 						obj.vs_defines.add("VERTICAL");
// 						obj.vs_defines.remove("HORIZONTAL");
// 					} else {
// 						obj.vs_defines.add("HORIZONTAL");
// 						obj.vs_defines.remove("VERTICAL");
// 					}
// 					let post_process = PostProcess {
// 						render_size: Size{width: w, height: h},
// 						// render_size: Size::new(width/4, width/4),
// 						render_obj: obj, // 在RenderObjs中的索引
// 					};
// 					texture_size = Size{width: w as usize, height: h as usize};
// 					vv.push(post_process);
// 				}

// 				let post_process_context = PostProcessContext { 
// 					content_box: content_box.clone(), 
// 					render_target: None, // 如果是None，则分配纹理来渲染阴影
// 					post_processes: vv,
// 					result: None,
// 					copy: 0,
// 				};
// 				render_obj.post_process = Some(Box::new(post_process_context));
// 				return (true, true);
// 			}
// 		}
// 	}
// 	(false, false)
// }

// fn modify_shadow_blur<C: HalContext + 'static>(
//     index: usize,
// 	blur: f32,
//     render_obj: &mut RenderObj,
//     engine: &mut Engine<C>,
// 	mut texture_size: Size<usize>,
// 	content_box: &Aabb2,
// 	default_state: &CommonState,
// ) -> (bool, bool) {
// 	// log::warn!("modify_shadow_blur======================blur:{}", blur);
// 	match &mut render_obj.post_process {
// 		Some(r) => {
// 			if blur==0.0 {
// 				render_obj.post_process = None;
// 				return (false, false)
// 			} else {
// 				for i in 0..r.post_processes.len() {
// 					r.post_processes[i].render_obj.paramter.set_single_uniform(
// 						"offset", 
// 						UniformValue::Float2(blur/texture_size.width as f32, blur/texture_size.height as f32/10.0)
// 					);
// 				}
// 				r.content_box = content_box.clone();
// 				return (true, false);
// 			}
// 		},
// 		None => {
// 			if blur != 0.0 {
// 				let width = content_box.maxs.x - content_box.mins.x;
// 				let height = content_box.maxs.y - content_box.mins.y;

// 				// log::warn!("blur1================={}, {}", width, height);
// 				// log::warn!("size=================blur: {}, width: {}, height: {}", blur, texture_size.width, texture_size.height);

// 				let mut vv = Vec::new();

// 				let (mut w, mut h) = (width, height);
// 				for i in 0..3 {
// 					let p: Share<dyn ProgramParamter> = Share::new(BlurParamter::default());
// 					p.set_single_uniform(
// 						"offset", 
// 						UniformValue::Float2(blur/texture_size.width as f32, blur/texture_size.height as f32)
// 					);
// 					w = (w / 2.0).round().max(1.0);
// 					h = (h / 2.0).round().max(1.0);
// 					let post_process = PostProcess {
// 						render_size: Size{width: w, height: h},
// 						// render_size: Size::new(width/4, width/4),
// 						render_obj: new_render_obj1(
// 							index, 0.0, true, BLUR_DOWN_VS_SHADER_NAME.clone(), BLUR_DOWN_FS_SHADER_NAME.clone(), p, default_state,
// 						), // 在RenderObjs中的索引
// 					};
// 					texture_size = Size{width: w as usize, height: h as usize};
// 					vv.push(post_process);
// 				}
// 				for i in 0..2 {
// 					let p: Share<dyn ProgramParamter> = Share::new(BlurParamter::default());
// 					p.set_single_uniform(
// 						"offset", 
// 						UniformValue::Float2(blur/texture_size.width as f32, blur/texture_size.height as f32)
// 					);
// 					w = (w * 2.0).round().max(1.0);
// 					h = (h * 2.0).round().max(1.0);
// 					let post_process = PostProcess {
// 						render_size: Size{width: w, height: h},
// 						// render_size: Size::new(width/4, width/4),
// 						render_obj: new_render_obj1(
// 							index, 0.0, true, BLUR_UP_VS_SHADER_NAME.clone(), BLUR_UP_FS_SHADER_NAME.clone(), p, default_state,
// 						), // 在RenderObjs中的索引
// 					};
// 					texture_size = Size{width: w as usize, height: h as usize};
// 					vv.push(post_process);
// 				}
// 				// let post_process1 = PostProcess {
// 				// 	render_size: Size{width: (width/2.0).max(1.0), height: (height/4.0).max(1.0)},
// 				// 	// render_size: Size::new(width/4, width/4),
// 				// 	render_obj: new_render_obj1(
// 				// 		index, 0.0, true, BLUR_DOWN_VS_SHADER_NAME.clone(), BLUR_DOWN_FS_SHADER_NAME.clone(), paramter.clone(), default_state,
// 				// 	), // 在RenderObjs中的索引
// 				// };
// 				// let post_process2 = PostProcess {
// 				// 	render_size: Size{width: (width/8.0).max(1.0), height: (height/8.0).max(1.0)},
// 				// 	// render_size: Size::new(width/4, width/4),
// 				// 	render_obj: new_render_obj1(
// 				// 		index, 0.0, true, BLUR_DOWN_VS_SHADER_NAME.clone(), BLUR_DOWN_FS_SHADER_NAME.clone(), paramter.clone(), default_state,
// 				// 	), // 在RenderObjs中的索引
// 				// };
// 				// let post_process3 = PostProcess {
// 				// 	render_size: Size{width: (width/4.0).max(1.0), height: (height/4.0).max(1.0)},
// 				// 	// render_size: Size::new(width/4, width/4),
// 				// 	render_obj: new_render_obj1(
// 				// 		index, 0.0, true, BLUR_UP_VS_SHADER_NAME.clone(), BLUR_UP_FS_SHADER_NAME.clone(), paramter, default_state,
// 				// 	), // 在RenderObjs中的索引
// 				// };

// 				// let post_process_context = PostProcessContext { 
// 				// 	content_box: content_box.clone(), 
// 				// 	render_target: None, // 如果是None，则分配纹理来渲染阴影
// 				// 	post_processes: vec![post_process1, post_process2, post_process3],
// 				// 	result: None,
// 				// 	copy: 0,
// 				// };

// 				let post_process_context = PostProcessContext { 
// 					content_box: content_box.clone(), 
// 					render_target: None, // 如果是None，则分配纹理来渲染阴影
// 					post_processes: vv,
// 					result: None,
// 					copy: 0,
// 				};
// 				render_obj.post_process = Some(Box::new(post_process_context));
// 				return (true, true);
// 			}
// 		}
// 	}
// 	(false, false)
// }

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
	dirty: StyleBit,
	dirty1: usize,
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
	is_pixel: bool,
	font_height: f32
) -> Option<Share<GeometryRes>> {
    // 是共享文字
    if text.0.0 == String::new() {
        let mut hasher = DefaultHasher::default();
        text.1.hash(&mut hasher);
        // 对于布局信息， 如果没有在style中设置， 可以直接使用class中的布局hash
        if !(dirty & &*TEXT_LAYOUT_DIRTY).any() && dirty1 & CalcType::Layout as usize == 0 {
            share_data.layout_hash.hash(&mut hasher);
        } else {
            //在style中使用了文字布局属性, 重新计算文字布局属性的hash
            let layout_hash = text_layout_hash(&text_style.text, &text_style.font);
            layout_hash.hash(&mut hasher);
        }

        // 如果是渐变色， 计算渐变色的hash
        if let Color::LinearGradient(ref blur) = color {
            blur.hash(&mut hasher);
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
			text_style,
			is_pixel,
			font_height,
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
			text_style,
			is_pixel,
			font_height,
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
            r.hash(hasher);
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
	text_style: &TextStyle,
	is_pixel: bool,
	mut font_height: f32,
) -> Option<Share<GeometryRes>> {
    let mut positions: Vec<f32> = Vec::with_capacity(8 * children.len);
    let mut uvs: Vec<f32> = Vec::with_capacity(8 * children.len);
    // let font_height = char_block.font_height;
	// font_height = font_height * scale;
	// log::warn!("fontheight1================={}, {}, {}", size, self.ascender, self.descender);
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
			let font_size = match font_sheet.get_font_info(&text_style.font.family) {
				Some(r) => get_size(r.1, &text_style.font.size) as f32,
				None => get_size(16, &text_style.font.size) as f32
			};
			let mut debug_infos = DebugInfos {
				font_size: font_size,
				scale,
				calc_font_size: font_size * scale,
				chars: Vec::new(),
				family: text_style.font.family.clone(),
				weight: text_style.font.weight as f32,
				stroke: *text_style.text.stroke.width,
			};
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
				// log::info!("glyph=============, id:{}, c:{}, glyph: {:?}", c.ch_id_or_count, c.ch , glyph);

				let mut debug_info = DebugInfo {
					ch: c.ch,
					size: c.size,
					pos: c.pos,
					glyph: glyph.clone(),
					positions: Vec::new(),
					uvs: Vec::new(),
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
						is_pixel,
						text_style.font.weight,
						*text_style.text.stroke.width,
						font_height,
						c.ch
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
						is_pixel,
						text_style.font.weight,
						*text_style.text.stroke.width,
						font_height,
						c.ch
					);
				}
				debug_info.positions.extend_from_slice(&positions[positions.len() - 8..positions.len()]);
				debug_info.uvs.extend_from_slice(&uvs[uvs.len() - 8..uvs.len()]);
				debug_infos.chars.push(debug_info);
			}

			if debug_infos.chars.len() != 0 {
				// log::info!("chars======{:?}", debug_infos);
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
			
			
			let endp = match node_state.0.is_vnode() {
				// 如果是虚拟节点，则节点自身的布局信息会在顶点上体现，此时找渐变端点需要考虑布局结果的起始点
				true => find_lg_endp(
					&[
						rect.left,
						rect.top,
						rect.left,
						rect.bottom,
						rect.right,
						rect.bottom,
						rect.right,
						rect.top,//渐变端点
					],
					color.direction,
				),
				// 非虚拟节点，顶点总是以0，0作为起始点，布局起始点体现在世界矩阵上
				false => find_lg_endp(
					&[
						0.0,
						0.0,
						0.0,
						rect.bottom - rect.top,
						rect.right - rect.left,
						rect.bottom - rect.top,
						rect.right - rect.left,
						0.0,
					],
					color.direction,
				),
			};

			let mut lg_pos = Vec::with_capacity(color.list.len());
			let mut lg_color = Vec::with_capacity(color.list.len() * 4);
			for v in color.list.iter() {
				lg_pos.push(v.position);
				lg_color.extend_from_slice(&[v.rgba.x, v.rgba.y, v.rgba.z, v.rgba.w]);
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
						is_pixel,
						text_style.font.weight,
						*text_style.text.stroke.width,
						font_height,
						c.ch
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
						is_pixel,
						text_style.font.weight,
						*text_style.text.stroke.width,
						font_height,
						c.ch
					);
				}

				// log::info!("position: {:?}, {:?}, {:?}, {:?}, {:?}", positions, node_state.0.text, lg_pos, &endp.0, &endp.1);
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
			// log::info!("position1: {:?}, {:?}, {:?}", positions , colors, indices);

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
		Some(hash) => {
			engine.geometry_res_map.create(hash, geo_res, size, 0)
		},
		None => Share::new(geo_res),
	})
}

#[derive(Debug)]
pub struct DebugInfo {
	pub ch: char,
	pub size: (f32, f32),        // 字符大小
    pub pos: (f32, f32),
	pub glyph: Glyph,
	pub positions: Vec<f32>,
	pub uvs: Vec<f32>,
}

#[derive(Debug)]
pub struct DebugInfos {
	pub font_size: f32,
	family: Atom,
	weight: f32,
	stroke: f32,
	pub scale: f32,
	pub calc_font_size: f32,
	pub chars: Vec<DebugInfo>,
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
	mut x: f32,
	mut y: f32,
    glyph: &Glyph,
	width: f32,
	height: f32,
	scale: f32,
	is_pixel: bool,
	weight: usize,
	stroke_width: f32,
	font_height: f32, // 单位： 逻辑像素
	c: char,
) {
	if is_pixel {
		push_pos_uv_canvas(
			positions,
			uvs,
			x,
			y,
			glyph,
			width,
			height,
			scale,
		);
		return;
	}

	y += (height - font_height) / 2.0;
	let (xx, font_width) = fix_box(is_pixel, width, weight, stroke_width);
	x += xx;

	let font_ratio = font_width/glyph.advance;

	let ox = font_height * glyph.ox;
	let oy = font_height * glyph.oy;

	let w = (glyph.width - 1.0)*font_ratio;
	let h = (glyph.height - 1.0)*font_ratio;
	// height为行高， 当行高高于字体高度时，需要居中
	// if is_pixel {
	// 	y += (height - h)/2.0;
	// } else {
	// 	y += yy;
	// 	y += (height - oy - h) / 2.0;

		
	// }
	
	let left_top = (
		x + ox,
		y  + oy, // 保证顶点对应整数像素
	);
	let right_bootom = (
		left_top.0 + w,
		left_top.1 + h,
	);
	// log::info!("y=====c: {:?},is_pixel: {:?},left_top: {:?}, right_bootom: {:?}, font_width: {:?}, font_height: {:?}, glyph: {:?}, x: {}, y: {}, width: {}, height: {}, ox: {}, oy: {}, w: {}, h: {}", c, is_pixel, left_top, right_bootom, font_width, font_height, glyph, x, y, width, height, ox, oy, w, h);

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
	// 加0.5和减0.5，是为了保证采样不超出文字范围
	let uv = [
        glyph.x + 0.5,
        glyph.y + 0.5,
        glyph.x + 0.5,
        glyph.y + glyph.height - 0.5,
        glyph.x + glyph.width - 0.5,
        glyph.y + glyph.height - 0.5,
        glyph.x + glyph.width - 0.5,
        glyph.y + 0.5,
	];
    uvs.extend_from_slice(&uv);
	// log::info!("uv=================={:?}, {:?}, w:{:?},h:{:?},scale:{:?},glyph:{:?}", uv, ps, width, height, scale, glyph);
    positions.extend_from_slice(&ps[..]);
}

fn push_pos_uv_canvas(
    positions: &mut Vec<f32>,
    uvs: &mut Vec<f32>,
	x: f32,
	mut y: f32,
    glyph: &Glyph,
	width: f32,
	height: f32,
	scale: f32,
) {
	let font_ratio = width/glyph.advance;
	let w = glyph.width*font_ratio;
	let h = glyph.height*font_ratio;
	let ox = font_ratio * glyph.ox;
	let oy = font_ratio * glyph.oy;
	// height为行高， 当行高高于字体高度时，需要居中
	y += (height - h)/2.0;
    let left_top = (
        ((x + ox) * scale).round()/scale,
        ((y  + oy) * scale).round()/scale, // 保证顶点对应整数像素
    );
    let right_bootom = (
        left_top.0 + (w*scale).round()/scale,
        left_top.1 + (h*scale).round()/scale,
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
        glyph.x + 0.5,
        glyph.y + 0.5,
        glyph.x + 0.5,
        glyph.y + glyph.height - 0.5,
        glyph.x + glyph.width - 0.5,
        glyph.y + glyph.height - 0.5,
        glyph.x + glyph.width - 0.5,
        glyph.y + 0.5,
	];
    uvs.extend_from_slice(&uv);
	// log::info!("uv=================={:?}, {:?}, w:{:?},h:{:?},scale:{:?},glyph:{:?}", uv, ps, width, height, scale, glyph);
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
        EntityListener<Node, DeleteEvent>
    }
}
