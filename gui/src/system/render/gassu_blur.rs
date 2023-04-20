//! 添加高斯模糊效果，当前为一个工具函数，后续可能为一个System

use flex_layout::Size;
use hal_core::{ProgramParamter, UniformValue, HalContext};
use share::Share;

use crate::{single::{PostProcess, CommonState, RenderObj, PostProcessContext, PostProcessObj}, component::{calc::GaussBlurParamter, user::Aabb2}, render::engine::Engine, system::util::new_render_obj1};

use super::shaders::image::{GAUSS_BLUR_VS_SHADER_NAME, GAUSS_BLUR_FS_SHADER_NAME};

/// 为渲染对象添加一个高斯模糊后处理
/// 返回（是否添加了后处理, copy是否为0）
pub fn add_gassu_blur<C: HalContext + 'static, T: PostProcessObj>(
	index: usize,
	blur: f32,
    render_obj: &mut T,
    engine: &mut Engine<C>,
	content_box: &Aabb2,
	default_state: &CommonState,
) -> (bool, bool) {
	let post_process = render_obj.get_post_mut();
	match post_process {
		Some(r) => {
			if blur==0.0 {
				render_obj.set_post(None);
				return (false, false)
			} else {
				for i in 0..r.post_processes.len() {
					r.post_processes[i].render_obj.paramter.set_single_uniform(
						"blurRadius",
						UniformValue::Float1(blur)
					);
				}
				r.content_box = content_box.clone();
				return (true, r.copy == 0);
			}
		},
		None => {
			if blur != 0.0 {
				let width = content_box.maxs.x - content_box.mins.x;
				let height = content_box.maxs.y - content_box.mins.y;

				let mut vv = Vec::new();

				for i in 0..2 {
					let (mut w, mut h) = (width, height);
					let p: Share<dyn ProgramParamter> = Share::new(GaussBlurParamter::default());
					p.set_single_uniform(
						"blurRadius", 
						UniformValue::Float1(blur)
					);	
					w = w.round().max(1.0);
					h = h.round().max(1.0);

					let mut obj = new_render_obj1(
						index, 0.0, false, GAUSS_BLUR_VS_SHADER_NAME.clone(), GAUSS_BLUR_FS_SHADER_NAME.clone(), p, default_state,
					);
					if i == 0 {
						obj.vs_defines.add("VERTICAL");
						obj.vs_defines.remove("HORIZONTAL");
					} else {
						obj.vs_defines.add("HORIZONTAL");
						obj.vs_defines.remove("VERTICAL");
					}
					let post_process = PostProcess {
						render_size: Size{width: w, height: h},
						// render_size: Size::new(width/4, width/4),
						render_obj: obj, // 在RenderObjs中的索引
					};
					vv.push(post_process);
				}

				let post_process_context = PostProcessContext { 
					content_box: content_box.clone(), 
					render_target: None, // 如果是None，则分配纹理来渲染阴影
					post_processes: vv,
					result: None,
					copy: 0,
				};
				render_obj.set_post(Some(Box::new(post_process_context)));
				return (true, true);
			}
		}
	}
	(false, false)
}