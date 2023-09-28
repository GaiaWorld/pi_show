/**
 * 渲染对象的通用属性设置， 如alpha， hsv， visible， viewMatrix， projectMatrix
 */
use std::{marker::PhantomData, cell::RefCell};

use crate::component::user::StyleType;
use share::Share;

use pi_atom::Atom;
use ecs::{
    CreateEvent, DeleteEvent, EntityImpl, EntityListener, ModifyEvent, MultiCaseImpl,
	MultiCaseListener, Runner, SingleCaseImpl, SingleCaseListener,
	monitor::{Event, NotifyImpl},
};
use hal_core::*;
use res::{ResMap, ResMgr};
use pi_style::style::BlendMode as BlendMode1;

use crate::{component::{calc::*, calc::LayoutR, user::{Aabb2, BorderRadius}, calc::Visibility as CVisibility}, single::dyn_texture::DynAtlasSet};
use crate::component::user::Opacity;
use crate::component::user::BlendMode;
use crate::entity::Node;
use crate::render::engine::{ShareEngine, UnsafeMut};
use crate::single::*;
use crate::single::DirtyViewRect;
use crate::single::oct::Oct;
use crate::system::util::*;
use crate::single::IdTree;
use crate::Z_MAX;

lazy_static! {
    static ref Z_DEPTH: Atom = Atom::from("zDepth");
    static ref HSV_MACRO: Atom = Atom::from("HSV");
    static ref HSV_ATTR: Atom = Atom::from("hsvValue");
}

pub struct NodeAttrSys<C: HalContext + 'static> {
    view_matrix_ubo: Option<Share<dyn UniformBuffer>>,
    project_matrix_ubo: Option<Share<dyn UniformBuffer>>,
    transform_will_change_matrix_dirtys: Vec<usize>,
    hsv_ubo_map: UnsafeMut<ResMap<HsvUbo>>,
    marker: PhantomData<C>,
}

impl<C: HalContext + 'static> NodeAttrSys<C> {
    pub fn new(res_mgr: &ResMgr) -> Self {
        NodeAttrSys {
            view_matrix_ubo: None,
            project_matrix_ubo: None,
            transform_will_change_matrix_dirtys: Vec::default(),
            hsv_ubo_map: UnsafeMut::new(res_mgr.fetch_map::<HsvUbo>(0).unwrap()),
            marker: PhantomData,
        }
    }

    pub fn create_hsv_ubo(&mut self, hsv: &HSV) -> Share<dyn UniformBuffer> {
        let h = f32_3_hash(hsv.h, hsv.s, hsv.v);
        match self.hsv_ubo_map.get(&h) {
            Some(r) => r,
            None => self.hsv_ubo_map.create(
                h,
                HsvUbo::new(UniformValue::Float3(hsv.h, hsv.s, hsv.v)),
                0,
                0,
            ), // TODO cost
        }
	}
	
	pub fn modifyHsv<'a>(&mut self, id: usize, hsvs: &'a MultiCaseImpl<Node, HSV>, write: (
		&'a mut SingleCaseImpl<RenderObjs>,
		&'a mut SingleCaseImpl<NodeRenderMap>,
	)) {
		let (render_objs, node_render_map) = write;
		let hsv = &hsvs[id];
		let obj_ids = match node_render_map.get(id) {
			Some(r) => r,
			None => return,
		};
		if !(hsv.h == 0.0 && hsv.s == 0.0 && hsv.v == 0.0) {
			for id in obj_ids.iter() {
				let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
				let render_obj = &mut render_objs[*id];
				render_obj.fs_defines.add("HSV");
				render_obj
					.paramter
					.set_value("hsvValue", self.create_hsv_ubo(hsv)); // hsv
	
				notify.modify_event(*id, "paramter", 0);
				notify.modify_event(*id, "program_dirty", 0);
			}
		} else {
			for id in obj_ids.iter() {
				let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
				let render_obj = &mut render_objs[*id];
				render_obj.fs_defines.remove("HSV");
	
				notify.modify_event(*id, "paramter", 0);
				notify.modify_event(*id, "program_dirty", 0);
			}
		}
	}
}

impl<'a, C: HalContext + 'static> Runner<'a> for NodeAttrSys<C> {
    type ReadData = (
		&'a MultiCaseImpl<Node, TransformWillChangeMatrix>,
		&'a MultiCaseImpl<Node, StyleMark>,
		&'a MultiCaseImpl<Node, ContentBox>,
        &'a SingleCaseImpl<IdTree>,
        &'a SingleCaseImpl<ViewMatrix>,
        &'a SingleCaseImpl<ProjectionMatrix>,
		&'a SingleCaseImpl<NodeRenderMap>,
		&'a SingleCaseImpl<DirtyList>,
		&'a SingleCaseImpl<Oct>,
		&'a SingleCaseImpl<RenderBegin>,
		&'a EntityImpl<Node>,
		&'a MultiCaseImpl<Node, crate::component::user::BackgroundImage>,
		
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
		&'a mut SingleCaseImpl<ShareEngine<C>>,
		&'a mut SingleCaseImpl<DirtyViewRect>
	);
	
	// type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
	// type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        let (transform_will_change_matrixs, style_marks, content_boxs, idtree, view_matrix, _, node_render_map, dirty_list, octree, render_begin, nodes, images) = read;
		let (render_objs, _, dirty_view_rect) = write;
		
		for id in dirty_list.0.iter() {
			// dirty_view_rect已经是最大范围了，不需要再修改
			if dirty_view_rect.4 == true {
				break;
			}
			let node = match idtree.get(*id) {
				Some(r) => r,
				None => continue,
			};
			if node.layer() == 0 {
				continue;
			}
			
			let style_mark = match style_marks.get(*id) {
				Some(r) => r,
				None => continue,
			};

			if content_show_change(style_mark) {
				if let Some(content_box) = content_boxs.get(*id) {
					handler_modify_oct(*id, &content_box.0, render_begin, dirty_view_rect);
				}
			} else if style_mark.dirty1 & CalcType::Oct as usize == 0 { // oct不脏才更新，因为监听器已经处理了oct
				if let Some(oct) = octree.get(*id) {
					handler_modify_oct(*id, &oct.0, render_begin, dirty_view_rect);
				}
			}
		}

        let mut modify = false;
        for id in self.transform_will_change_matrix_dirtys.iter() {
            if !nodes.is_exist(*id) {
                continue;
            }
            match transform_will_change_matrixs.get(*id) {
                Some(transform_will_change_matrix) => {
                    let m = &view_matrix.0 * &transform_will_change_matrix.0;
                    let slice: &[f32] = m.0.as_slice();
                    let view_matrix_ubo: Share<dyn UniformBuffer> = Share::new(ViewMatrixUbo::new(
                        UniformValue::MatrixV4(Vec::from(slice)),
                    ));
                    recursive_set_view_matrix(
                        *id,
                        &mut modify,
                        transform_will_change_matrixs,
                        idtree,
                        &view_matrix_ubo,
                        node_render_map,
                        render_objs,
                    );
                }
                None => recursive_set_view_matrix(
                    *id,
                    &mut modify,
                    transform_will_change_matrixs,
                    idtree,
                    self.view_matrix_ubo.as_ref().unwrap(),
                    node_render_map,
                    render_objs,
                ),
            }
        }

        self.transform_will_change_matrix_dirtys.clear();
        if modify {
            render_objs.get_notify_ref().modify_event(0, "ubo", 0);
        }
    }
    fn setup(&mut self, read: Self::ReadData, _: Self::WriteData) {
        let (_, _, _, _, view_matrix, projection_matrix, _, _, _, _, _, _) = read;

        let slice: &[f32] = view_matrix.0.as_slice();
        let view_matrix_ubo = ViewMatrixUbo::new(UniformValue::MatrixV4(Vec::from(slice)));

        let slice: &[f32] = projection_matrix.0.as_slice();
        let project_matrix_ubo =
            ProjectMatrixUbo::new(UniformValue::MatrixV4(Vec::from(slice)));

        self.view_matrix_ubo = Some(Share::new(view_matrix_ubo));
        self.project_matrix_ubo = Some(Share::new(project_matrix_ubo));
    }
}

fn handler_modify_oct(
	id: usize,
	aabb: &Aabb2,
	// octree: &SingleCaseImpl<Oct>,
	render_begin: &SingleCaseImpl<RenderBegin>,
	dirty_view_rect: &mut SingleCaseImpl<DirtyViewRect>
) {
	
	// let oct = match octree.get(id){
	// 	Some(r) => r,
	// 	None => return,
	// };
	// let oct = &oct.0;
	let viewport = render_begin.0.viewport;
	// println!("true2======================dirty_view_rect: {:?}", **dirty_view_rect);
	// 与包围盒求并
	dirty_view_rect.0 = dirty_view_rect.0.min(aabb.mins.x.max(0.0));
	dirty_view_rect.1 = dirty_view_rect.1.min(aabb.mins.y.max(0.0));
	dirty_view_rect.2 = dirty_view_rect.2.max(aabb.maxs.x.min(viewport.2 as f32));
	dirty_view_rect.3 = dirty_view_rect.3.max(aabb.maxs.y.min(viewport.3 as f32));
	// println!("true3======================dirty_view_rect: {:?}, oct: {:?}", **dirty_view_rect, oct);

	// 如果与视口一样大，则设置dirty_view_rect.4为true, 后面的包围盒改变，将不再重新计算dirty_view_rect
	// 由于包围盒改变事件通常是从父到子的顺序传递，因此如果界面有大范围的改变，能够很快的将dirty_view_rect.4设置为true
	// 因此在大范围改变时，具有较好的优化
	// 另外，dirty_view_rect.4被设计的另一个原因是，外部很多时候能够预计即将改变的界面将是大范围，可以提前设置该值，来优化掉后面的计算（尽管这种计算并不很费）
	if dirty_view_rect.0 == 0.0 && 
	dirty_view_rect.1 == 0.0 && 
	dirty_view_rect.2 == viewport.2 as f32 && 
	dirty_view_rect.3 == viewport.3  as f32 {

		// println!("true1======================oct: {:?}, dirty_view_rect:{:?}", oct, **dirty_view_rect);
		dirty_view_rect.4 = true;
	}
}

// 设置为最大视口
fn set_max_view(
	render_begin: &SingleCaseImpl<RenderBegin>,
	dirty_view_rect: &mut SingleCaseImpl<DirtyViewRect>
) {
	let viewport = render_begin.0.viewport;
	// 如果与视口一样大，则设置dirty_view_rect.4为true, 后面的包围盒改变，将不再重新计算dirty_view_rect
	// 由于包围盒改变事件通常是从父到子的顺序传递，因此如果界面有大范围的改变，能够很快的将dirty_view_rect.4设置为true
	// 因此在大范围改变时，具有较好的优化
	// 另外，dirty_view_rect.4被设计的另一个原因是，外部很多时候能够预计即将改变的界面将是大范围，可以提前设置该值，来优化掉后面的计算（尽管这种计算并不很费）
	if dirty_view_rect.4 != true {
		

		// println!("true2======================dirty_view_rect: {:?}", **dirty_view_rect);
		// 与包围盒求并
		dirty_view_rect.0 = 0.0;
		dirty_view_rect.1 = 0.0;
		dirty_view_rect.2 = viewport.2 as f32;
		dirty_view_rect.3 = viewport.3 as f32;
	// println!("true3======================dirty_view_rect: {:?}, oct: {:?}", **dirty_view_rect, oct);
		println!("dirty_view_rect======================");
		// println!("true1======================oct: {:?}, dirty_view_rect:{:?}", oct, **dirty_view_rect);
		dirty_view_rect.4 = true;
	}
	
	
}

// impl<'a, C: HalContext + 'static> SingleCaseListener<'a, ProjectionMatrix, ModifyEvent>
//     for NodeAttrSys<C>
// {
//     type ReadData = &'a SingleCaseImpl<ProjectionMatrix>;
//     type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
//     fn listen(
//         &mut self,
//         _event: &Event,
//         projection_matrix: Self::ReadData,
//         render_objs: Self::WriteData,
//     ) {
//         let slice: &[f32] = projection_matrix.0.as_slice();
//         let project_matrix_ubo =
//             ProjectMatrixUbo::new(UniformValue::MatrixV4(Vec::from(slice)));
//         self.project_matrix_ubo = Some(Share::new(project_matrix_ubo));
//         for r in render_objs.iter_mut() {
//             r.1.paramter
//                 .set_value("projectMatrix", self.project_matrix_ubo.clone().unwrap());
//         }
//         if render_objs.len() > 0 {
//             render_objs.get_notify_ref().modify_event(1, "ubo", 0);
//         }
//     }
// }

impl<'a, C: HalContext + 'static> EntityListener<'a, Node, CreateEvent> for NodeAttrSys<C> {
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<NodeRenderMap>;
    fn listen(
        &mut self,
        event: &Event,
        _read: Self::ReadData,
        node_render_map: Self::WriteData,
    ) {
        node_render_map.create(event.id);
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, TransformWillChangeMatrix, (ModifyEvent, CreateEvent, DeleteEvent)> for NodeAttrSys<C>
{
    type ReadData = &'a SingleCaseImpl<RenderBegin>;
    type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
    fn listen(&mut self, event: &Event, read: Self::ReadData, write: Self::WriteData) {
        self.transform_will_change_matrix_dirtys.push(event.id);
		set_max_view(read, write);
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, BlendMode, (CreateEvent, ModifyEvent)> for NodeAttrSys<C>
{
	type ReadData = (&'a MultiCaseImpl<Node, BlendMode>, &'a SingleCaseImpl<DefaultState>);
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
    );
    fn listen(&mut self, event: &Event, (blend_modes, default_state): Self::ReadData, write: Self::WriteData) {
        let (render_objs, node_render_map) = write;
        let obj_ids = &node_render_map[event.id];
        let blend_mode = &blend_modes[event.id].0;

		if obj_ids.len() > 0 {
			let bs = match blend_mode {
				BlendMode1::Normal => &default_state.df_bs,
				BlendMode1::AlphaAdd => &default_state.alpha_add_bs,
				BlendMode1::Subtract => &default_state.subtract_bs,
				BlendMode1::Multiply => &default_state.multiply_bs,
				BlendMode1::OneOne => &default_state.one_one_bs,
			};

			for id in obj_ids.iter() {
				let render_obj = &mut render_objs[*id];
				render_obj.state.bs = bs.clone();
				render_objs.get_notify_ref().modify_event(*id, "blend", 0);
			}
		}
    }
}


fn recursive_set_view_matrix(
    id: usize,
    modify: &mut bool,
    transform_will_change_matrixs: &MultiCaseImpl<Node, TransformWillChangeMatrix>,
    idtree: &IdTree,
    ubo: &Share<dyn UniformBuffer>,
    node_render_map: &SingleCaseImpl<NodeRenderMap>,
    render_objs: &mut SingleCaseImpl<RenderObjs>,
) {
    let obj_ids = &match node_render_map.get(id) {
		Some(r) => r,
		None => return,
	};
    for id in obj_ids.iter() {
        let render_obj = &mut render_objs[*id];
        render_obj.paramter.set_value("viewMatrix", ubo.clone());
        *modify = true;
    }

    let first = idtree[id].children().head;
    for (child_id, _child) in idtree.iter(first) {
        if let Some(_) = transform_will_change_matrixs.get(child_id) {
            continue;
		}
        recursive_set_view_matrix(
            child_id,
            modify,
            transform_will_change_matrixs,
            idtree,
            ubo,
            node_render_map,
            render_objs,
        );
    }
}

//创建索引
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, CreateEvent>
    for NodeAttrSys<C>
{
    type ReadData = (
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, HSV>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Culling>,
		&'a MultiCaseImpl<Node, Opacity>,
		&'a MultiCaseImpl<Node, BlendMode>,
		&'a SingleCaseImpl<DefaultState>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
    );
    fn listen(&mut self, event: &Event, read: Self::ReadData, write: Self::WriteData) {
        let (visibilitys, hsvs, z_depths, cullings, opcaitys, blend_modes, default_state) = read;
        let (render_objs, node_render_map) = write;
        let render_obj = &mut render_objs[event.id];
        let notify = unsafe { &*(node_render_map.get_notify_ref() as * const NotifyImpl) };
        node_render_map.add(render_obj.context, event.id, &notify);

        let paramter = &mut render_obj.paramter;

        // paramter.set_value("viewMatrix", self.view_matrix_ubo.clone().unwrap()); // VIEW_MATRIX
        // paramter.set_value("projectMatrix", self.project_matrix_ubo.clone().unwrap()); // PROJECT_MATRIX

        let z_depth = z_depths[render_obj.context].0;
        // let opacity = opacitys[render_obj.context].0;
        // paramter.set_single_uniform("alpha", UniformValue::Float1(opacity)); // alpha
        // debug_println!("id: {}, alpha: {:?}", render_obj.context, opacity);

        let visibility = visibilitys[render_obj.context].0;
        let culling = cullings[render_obj.context].0;
		let opcaity = opcaitys[render_obj.context].0;
        render_obj.visibility = visibility && !culling && opcaity > 0.0;

        render_obj.depth = z_depth + render_obj.depth_diff;
		// let depth = -(render_obj.depth / (Z_MAX + 1.0) * 2.0 ) + 1.0;
		let depth = -render_obj.depth / (Z_MAX + 1.0);
		paramter.set_single_uniform("depth", UniformValue::Float1(depth));

        let hsv = &hsvs[render_obj.context];
        if !(hsv.h == 0.0 && hsv.s == 0.0 && hsv.v == 0.0) {
            render_obj.fs_defines.add("HSV");
            paramter.set_value("hsvValue", self.create_hsv_ubo(hsv)); // hsv
        }

		let blend_mode = &blend_modes[render_obj.context].0;
		match blend_mode {
			BlendMode1::AlphaAdd => render_obj.state.bs = default_state.alpha_add_bs.clone(),
			BlendMode1::Subtract => render_obj.state.bs = default_state.subtract_bs.clone(),
			BlendMode1::Multiply => render_obj.state.bs = default_state.multiply_bs.clone(),
			BlendMode1::OneOne => render_obj.state.bs = default_state.one_one_bs.clone(),
			_ => (),
		}
    }
}

// 删除索引
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, DeleteEvent>
    for NodeAttrSys<C>
{
    type ReadData = ();
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<NodeRenderMap>, &'a mut SingleCaseImpl<Share<RefCell<DynAtlasSet>>>);
    fn listen(
        &mut self,
        event: &Event,
        read: Self::ReadData,
        (mut render_objs, node_render_map, mut dyn_atlas_set): Self::WriteData,
    ) {
	
        let render_obj = &render_objs[event.id];
		// log::warn!("del obj============{:?}, {:?}, {:?}", event.id, render_obj.post_process.is_some(), render_obj.post_process.is_some());
        let notify = unsafe { &*(node_render_map.get_notify_ref() as * const NotifyImpl) };
		let context = render_obj.context;
        node_render_map.remove(context, event.id, &notify);

		// 释放后处理结果
		if let Some(post_process) = &render_obj.post_process {
			// log::warn!("del post_process============{:?}, {:?}, {}", event.id, post_process.copy, render_obj.context);
			if let Some(target_index) = &post_process.result {
				// log::info!("delete_rect renderobj remove============={}", target_index);
				unsafe{&mut *(dyn_atlas_set.as_ptr())}.delete_rect(*target_index);
			}
			// log::warn!("render_obj remove:{}, context: {}", post_process.copy, render_obj.context);
			// 删除copy
			if post_process.copy > 0 {
				let copy = post_process.copy;
				render_objs.remove(copy, None);
				node_render_map.remove(context, copy, &notify);
			}
		}
    }
}

//深度变化， 修改renderobj的深度值
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ZDepth, ModifyEvent>
    for NodeAttrSys<C>
{
    type ReadData = &'a MultiCaseImpl<Node, ZDepth>;
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
    );
    fn listen(&mut self, event: &Event, z_depths: Self::ReadData, write: Self::WriteData) {
        let (render_objs, node_render_map) = write;
        let obj_ids = &node_render_map[event.id];
        let z_depth = z_depths[event.id].0;

		if obj_ids.len() > 0 {
			for id in obj_ids.iter() {
				let render_obj = &mut render_objs[*id];
				render_obj.depth = z_depth + render_obj.depth_diff;
				// let depth = -(render_obj.depth / (Z_MAX + 1.0) * 2.0) + 1.0;
				let depth = -render_obj.depth / (Z_MAX + 1.0);
				render_obj.paramter.set_single_uniform("depth", UniformValue::Float1(depth));
				render_objs.get_notify_ref().modify_event(*id, "depth", 0);
			}
		}
    }
}

// // 设置visibility
// impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Visibility, (CreateEvent, ModifyEvent)>
//     for NodeAttrSys<C>
// {
//     type ReadData = (
//         &'a MultiCaseImpl<Node, Visibility>, 
//         &'a MultiCaseImpl<Node, Culling>,
// 		&'a MultiCaseImpl<Node, Opacity>,
// 		&'a SingleCaseImpl<Oct>,
// 		&'a SingleCaseImpl<RenderBegin>,
//     );
//     type WriteData = (
//         &'a mut SingleCaseImpl<RenderObjs>,
//         &'a mut SingleCaseImpl<NodeRenderMap>,
// 		&'a mut SingleCaseImpl<DirtyViewRect>
//     );
//     fn listen(&mut self, event: &Event, read: Self::ReadData, write: Self::WriteData) {
// 		// dirty_view_rect已经是最大范围了，不需要再修改
// 		if (write.2).4 == true {
// 			return;
// 		}

// 		if let Some(context_box) = read.3.get(event.id) {
// 			handler_modify_oct(event.id, &context_box.0, read.4, write.2);
// 		}
//     }
// }

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, CVisibility, (CreateEvent, ModifyEvent)>
    for NodeAttrSys<C>
{
    type ReadData = (
        &'a MultiCaseImpl<Node, Visibility>, 
        &'a MultiCaseImpl<Node, Culling>,
		&'a MultiCaseImpl<Node, Opacity>,
		&'a SingleCaseImpl<Oct>,
		&'a SingleCaseImpl<RenderBegin>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
		&'a mut SingleCaseImpl<DirtyViewRect>
    );
    fn listen(&mut self, event: &Event, read: Self::ReadData, write: Self::WriteData) {
        modify_visible(event.id, (read.0, read.1, read.2), (write.0, write.1));
    }
}

// 设置culling
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Culling, ModifyEvent>
    for NodeAttrSys<C>
{
    type ReadData = (
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, Culling>,
		&'a MultiCaseImpl<Node, Opacity>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
    );
    fn listen(&mut self, event: &Event, read: Self::ReadData, write: Self::WriteData) {
        modify_visible(event.id, read, write);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Opacity, (CreateEvent, ModifyEvent)>
    for NodeAttrSys<C>
{
    type ReadData = (
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, Culling>,
		&'a MultiCaseImpl<Node, Opacity>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
    );
    fn listen(&mut self, event: &Event, read: Self::ReadData, write: Self::WriteData) {
        modify_visible(event.id, read, write);
    }
}

// 设置hsv
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, HSV, (CreateEvent, ModifyEvent)> for NodeAttrSys<C> {
    type ReadData = &'a MultiCaseImpl<Node, HSV>;
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
    );
    fn listen(&mut self, event: &Event, hsvs: Self::ReadData, write: Self::WriteData) {
        self.modifyHsv(event.id, hsvs, write);
    }
}

// 包围盒修改，
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, Oct, (CreateEvent, ModifyEvent, DeleteEvent)>
    for NodeAttrSys<C>
{
	type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
	type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
	fn listen(
        &mut self,
        event: &Event,
        (octree, render_begin): Self::ReadData,
        dirty_view_rect: Self::WriteData,
    ) {
		// dirty_view_rect已经是最大范围了，不需要再修改
		if dirty_view_rect.4 == true {
			return;
		}
		if let Some(oct) = octree.get(event.id) {
			handler_modify_oct(event.id, &oct.0, render_begin, dirty_view_rect);
		}
    }
}

//创建索引
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, VertType, ModifyEvent> for NodeAttrSys<C> {
    type ReadData = (
		&'a MultiCaseImpl<Node, BorderRadius>,
		&'a MultiCaseImpl<Node, LayoutR>,
    );
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &Event, read: Self::ReadData, render_objs: Self::WriteData) {
        let (border_radiuses, layouts) = read;
        let render_obj = &mut render_objs[event.id];
        
		// 如果存在圆角，需要设置圆角的uniform
		if let Some(border_radius) = border_radiuses.get(render_obj.context) {
			let layout = &layouts[render_obj.context];
			let border_radius = cal_border_radius(border_radius, &layout.rect);
			set_radius(&border_radius, layout, render_obj);
		}
	}
}

const BORDER_RADIUS: &'static str = "BORDER_RADIUS";

/// 处理圆角的删除
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderRadius, DeleteEvent> for NodeAttrSys<C> {
    type ReadData = &'a SingleCaseImpl<NodeRenderMap>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &Event, node_render_map: Self::ReadData, render_objs: Self::WriteData) {
		if let Some(render_list) = node_render_map.get(event.id) {
			for i in render_list.iter() {
				if let Some(draw_obj) = render_objs.get_mut(*i) {
					// 移除宏
					if let Some(_) = draw_obj.fs_defines.remove(BORDER_RADIUS) {
						render_objs.get_notify_ref().modify_event(*i, "pipeline", 0);
					}
				}
			}
		}
    }
}

/// 处理圆角的修改
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderRadius, (CreateEvent, ModifyEvent)> for NodeAttrSys<C> {
    type ReadData = (
		&'a MultiCaseImpl<Node, BorderRadius>,
		&'a MultiCaseImpl<Node, LayoutR>,
		&'a SingleCaseImpl<NodeRenderMap>
	);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &Event, (border_radiuses, layouts, node_render_map): Self::ReadData, render_objs: Self::WriteData) {
		if let Some(render_list) = node_render_map.get(event.id) {
			if render_list.len() == 0 {
				return;
			}

			let (border_radius, layout) = (&border_radiuses[event.id], &layouts[event.id]);

			let border_radius = cal_border_radius(border_radius, &layout.rect);
			for i in render_list.iter() {
				if let Some(draw_obj) = render_objs.get_mut(*i) {
					if draw_obj.vert_type == VertType::Border {
						continue;
					}

					if set_radius(&border_radius, layout, draw_obj) {
						// 需要重新编译shader
						render_objs.get_notify_ref().modify_event(*i, "program_dirty", 0);
					} else {
						// 否则仅仅发送改变的通知
						render_objs.get_notify_ref().modify_event(*i, "", 0);
					}
				}
			}
		}
    }
}

// 设置圆角， 返回宏是否改变
fn set_radius(border_radius: &BorderRadiusPixel, layout: &LayoutR, draw_obj: &mut RenderObj) -> bool {
	let (width, height)  = (layout.rect.right - layout.rect.left, layout.rect.bottom - layout.rect.top);
	let (x, y, z, w) = match draw_obj.vert_type {
		VertType::BorderRect | VertType::ContentRect  => (
			width/2.0, 
			height/2.0, 
			width, 
			height, 
		),
		VertType::BorderNone | VertType::ContentNone => (
			width/2.0, 
			height/2.0, 
			1.0, 1.0
		),
		VertType::Border => return false, // 渲染边框，不需要额外添加圆角的uniform
	};

	let mut change = false;
	if draw_obj.fs_defines.add(BORDER_RADIUS).is_some() {
		change = true;
	}

	let temp;
	let border_radius = match draw_obj.vert_type {
		VertType::ContentNone | VertType::ContentRect  => {
			temp = cal_content_border_radius(&border_radius, (
				layout.border.top,
				layout.border.right,
				layout.border.bottom,
				layout.border.left,
			));
			&temp
		},
		VertType::BorderNone | VertType::BorderRect => &border_radius, 
		_ => return change,
	};

	// 设置uniform
	draw_obj
	.paramter
	.as_ref()
	.set_single_uniform("clipSdf", UniformValue::MatrixV4(vec![
		x, y, z, w, 
		width/2.0, height/2.0, 0.0, 0.0,
		border_radius.y[0], border_radius.x[0], border_radius.x[1], border_radius.y[1],
		border_radius.y[2], border_radius.x[2], border_radius.x[3], border_radius.y[3],
	]));

	change
}	


impl<'a, C: HalContext + 'static> EntityListener<'a, Node, ModifyEvent>
    for NodeAttrSys<C>
{
    type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
	type WriteData = (&'a mut SingleCaseImpl<DirtyViewRect>, &'a mut SingleCaseImpl<RenderObjs>, &'a mut SingleCaseImpl<NodeRenderMap>, &'a mut SingleCaseImpl<Share<RefCell<DynAtlasSet>>>);
	fn listen(
        &mut self,
        event: &Event,
        (octree, render_begin): Self::ReadData,
        (dirty_view_rect, render_objs, node_render_map, dyn_atlas_set): Self::WriteData,
    ) {
		if let Some(r) = node_render_map.get(event.id) {
			if r.len() > 0 {
				// let notify = render_objs.get_notify();
				for item in r.iter() {
					// 释放后处理结果
					let render_obj = &render_objs[*item];
					if let Some(post_process) = &render_obj.post_process {
						// log::warn!("del post_process============{:?}, {:?}, {}", event.id, post_process.copy, render_obj.context);
						if let Some(target_index) = &post_process.result {
							// log::info!("delete_rect renderobj remove============={}", target_index);
							unsafe{&mut *(dyn_atlas_set.as_ptr())}.delete_rect(*target_index);
						}
					}

					render_objs.remove(*item, None);
				}
				unsafe {node_render_map.destroy_unchecked( event.id)}
			}
		}
		// dirty_view_rect已经是最大范围了，不需要再修改
		if dirty_view_rect.4 == true {
			return;
		}
		if let Some(oct) = octree.get(event.id) {
			handler_modify_oct(event.id, &oct.0, render_begin, dirty_view_rect);
		}
    }
}

type ReadData<'a> = (
    &'a MultiCaseImpl<Node, Visibility>,
    &'a MultiCaseImpl<Node, Culling>,
	&'a MultiCaseImpl<Node, Opacity>,
);
type WriteData<'a> = (
    &'a mut SingleCaseImpl<RenderObjs>,
    &'a mut SingleCaseImpl<NodeRenderMap>,
);

fn modify_visible(id: usize, read: ReadData, write: WriteData) {
    let (visibilitys, cullings, opacitys) = read;
    let (render_objs, node_render_map) = write;
    let visibility = visibilitys[id].0;
    let culling = cullings[id].0;
	let opcaity = opacitys[id].0;
    let obj_ids = &node_render_map[id];

	// log::info!("opcaity============={:?}, {:?}", opcaity, id);

    for id in obj_ids.iter() {
		let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
        let mut render_obj = RenderObjs::get_write(&mut *render_objs, *id, &notify);
        render_obj.set_visibility(visibility && !culling && opcaity > 0.0);
    }
}
lazy_static! {
	static ref CONTENT_DIRTY: StyleBit = style_bit().set_bit(StyleType::Hsi as usize) 
		.set_bit(StyleType::Opacity as usize)
		.set_bit(StyleType::BackgroundColor as usize) // 还有其他属性 TODO
		.set_bit(StyleType::Display as usize)
		.set_bit(StyleType::Visibility as usize)
		.set_bit( StyleType::Overflow as usize)
		.set_bit(StyleType::MaskImage as usize)
		.set_bit(StyleType::MaskImageClip as usize)
		.set_bit(StyleType::Blur as usize);
}


	// | StyleType::MaskTexture as usize

fn content_show_change(style_mark: &StyleMark) -> bool{
	if (style_mark.dirty & &*CONTENT_DIRTY).any() || style_mark.dirty1 & CalcType::MaskImageTexture as usize != 0 {
		true
	} else {
		false
	}
}
// // 枚举样式的类型
// #[derive(Debug)]
// pub enum StyleType {
//     Text = 1,
//     FontStyle = 2,
//     FontWeight = 4,
//     FontSize = 0x8,
//     FontFamily = 0x10,
//     LetterSpacing = 0x20,
//     WordSpacing = 0x40,
//     LineHeight = 0x80,
//     Indent = 0x100,
//     WhiteSpace = 0x200,
//     TextAlign = 0x400,
//     VerticalAlign = 0x800,
//     Color = 0x1000,
//     Stroke = 0x2000,
//     TextShadow = 0x4000,

//     Image = 0x8000,
//     ImageClip = 0x10000,
//     ObjectFit = 0x20000,

//     BorderImage = 0x40000,
//     BorderImageClip = 0x80000,
//     BorderImageSlice = 0x100000,
//     BorderImageRepeat = 0x200000,

//     BorderColor = 0x400000,

//     BackgroundColor = 0x800000,

//     BoxShadow = 0x1000000,

//     Matrix = 0x2000000,
//     Opacity = 0x4000000,
//     Layout = 0x8000000,
//     BorderRadius = 0x10000000,
//     ByOverflow = 0x20000000,
// 	Filter = 0x40000000,
// 	Oct = std::isize::MIN,
// }

// // 枚举样式的类型
// #[derive(Debug)]
// pub enum StyleType1 {
//     // Width = 1,
//     // Height = 2,
//     // Margin = 4,
//     // Padding = 8,
//     // Border = 0x10,
//     // Position = 0x20,
//     // MinWidth = 0x40,
//     // MinHeight = 0x80,
//     // MaxHeight = 0x100,
//     // MaxWidth = 0x200,
//     // FlexBasis = 0x400,
//     // FlexShrink = 0x800,
//     // FlexGrow = 0x1000,
//     // PositionType = 0x2000,
//     // FlexWrap = 0x4000,
//     // FlexDirection = 0x8000,
//     // AlignContent = 0x10000,
//     // AlignItems = 0x20000,
//     // AlignSelf = 0x40000,
// 	TransformOrigin = 0x4000,
// 	ContentBox = 0x8000,
// 	Direction = 0x10000,
// 	AspectRatio = 0x20000,
// 	Order = 0x40000,
// 	FlexBasis = 0x80000,

//     Display = 0x100000,
//     Visibility = 0x200000,
//     Enable = 0x400000,
//     ZIndex = 0x800000,
//     Transform = 0x1000000,
//     TransformWillChange = 0x2000000,
// 	Overflow = 0x4000000,
	
// 	Create = 0x8000000,
// 	Delete = 0x10000000,

// 	MaskImage = 0x20000000,
// 	MaskImageClip = 0x40000000,
// 	MaskTexture = std::isize::MIN,
// }

// // 布局属性标记
// pub enum StyleType2 {
// 	Width = 1,
//     Height = 2,
	
// 	MarginTop = 4,
// 	MarginRight = 8,
// 	MarginBottom = 0x10,
// 	MarginLeft = 0x20,

// 	PaddingTop = 0x40,
// 	PaddingRight = 0x80,
// 	PaddingBottom = 0x100,
// 	PaddingLeft = 0x200,

// 	BorderTop = 0x400,
// 	BorderRight = 0x800,
// 	BorderBottom = 0x1000,
// 	BorderLeft = 0x2000,

// 	PositionTop = 0x4000,
// 	PositionRight = 0x8000,
// 	PositionBottom = 0x10000,
// 	PositionLeft = 0x20000,
	
//     MinWidth = 0x40000,
//     MinHeight = 0x80000,
//     MaxHeight = 0x100000,
// 	MaxWidth = 0x200000,
// 	JustifyContent = 0x400000,
//     FlexShrink = 0x800000,
// 	FlexGrow = 0x1000000,
// 	PositionType = 0x2000000,
//     FlexWrap = 0x4000000,
//     FlexDirection = 0x8000000,
//     AlignContent = 0x10000000,
//     AlignItems = 0x20000000,
//     AlignSelf = 0x40000000,
// 	BlendMode = std::isize::MIN,
// }

impl_system! {
    NodeAttrSys<C> where [C: HalContext + 'static],
    true,
    {
        EntityListener<Node, CreateEvent>
        SingleCaseListener<RenderObjs, CreateEvent>
		SingleCaseListener<RenderObjs, DeleteEvent>
		// SingleCaseListener<ProjectionMatrix, ModifyEvent>
		EntityListener<Node, ModifyEvent>
		SingleCaseListener<Oct, (CreateEvent, ModifyEvent, DeleteEvent)>
		SingleCaseListener<VertType, ModifyEvent>
		MultiCaseListener<Node, BorderRadius, (CreateEvent, ModifyEvent)>
		MultiCaseListener<Node, BorderRadius, DeleteEvent>
		MultiCaseListener<Node, Visibility, (CreateEvent, ModifyEvent)>
		MultiCaseListener<Node, CVisibility, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, Culling, ModifyEvent>
		MultiCaseListener<Node, Opacity, (CreateEvent, ModifyEvent)>
		MultiCaseListener<Node, HSV, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, ZDepth, ModifyEvent>
        MultiCaseListener<Node, TransformWillChangeMatrix, (ModifyEvent, CreateEvent, DeleteEvent)>
		MultiCaseListener<Node, BlendMode, (CreateEvent, ModifyEvent)>
    }
}
