/**
 * 渲染对象的通用属性设置， 如alpha， hsv， visible， viewMatrix， projectMatrix
 */
use std::marker::PhantomData;

use share::Share;

use atom::Atom;
use single::IdTree;
use ecs::{
    CreateEvent, DeleteEvent, EntityImpl, EntityListener, ModifyEvent, MultiCaseImpl,
	MultiCaseListener, Runner, SingleCaseImpl, SingleCaseListener,
	monitor::NotifyImpl
};
use hal_core::*;
use res::{ResMap, ResMgr};

use component::calc::*;
use component::calc::Opacity;
use entity::Node;
use render::engine::{ShareEngine, UnsafeMut};
use single::*;
use single::oct::Oct;
use system::util::*;

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
		let obj_ids = &node_render_map[id];
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
        &'a SingleCaseImpl<IdTree>,
        &'a SingleCaseImpl<ViewMatrix>,
        &'a SingleCaseImpl<ProjectionMatrix>,
		&'a SingleCaseImpl<NodeRenderMap>,
		&'a SingleCaseImpl<DirtyList>,
		&'a SingleCaseImpl<Oct>,
		&'a SingleCaseImpl<RenderBegin>,
		&'a EntityImpl<Node>,
		
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
		&'a mut SingleCaseImpl<ShareEngine<C>>,
		&'a mut SingleCaseImpl<DirtyViewRect>
	);
	
	// type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
	// type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData) {
        let (transform_will_change_matrixs, style_marks, idtree, view_matrix, _, node_render_map, dirty_list, octree, render_begin, nodes) = read;
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
			
			
			match style_marks.get(*id) {
				Some(r) => {
					if r.dirty & StyleType::Oct as usize != 0 {
						continue;
					}
				},
				None => continue,
			};

			
			handler_modify_oct(*id, octree, render_begin, dirty_view_rect);
		}

        let mut modify = false;
        for id in self.transform_will_change_matrix_dirtys.iter() {
            if !nodes.is_exist(*id) {
                continue;
            }
            match transform_will_change_matrixs.get(*id) {
                Some(transform_will_change_matrix) => {
                    let m = &view_matrix.0 * &transform_will_change_matrix.0;
                    let slice: &[f32; 16] = m.0.as_ref();
                    let view_matrix_ubo: Share<dyn UniformBuffer> = Share::new(ViewMatrixUbo::new(
                        UniformValue::MatrixV4(Vec::from(&slice[..])),
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
        let (_, _, _, view_matrix, projection_matrix, _, _, _, _, _) = read;

        let slice: &[f32; 16] = view_matrix.0.as_ref();
        let view_matrix_ubo = ViewMatrixUbo::new(UniformValue::MatrixV4(Vec::from(&slice[..])));

        let slice: &[f32; 16] = projection_matrix.0.as_ref();
        let project_matrix_ubo =
            ProjectMatrixUbo::new(UniformValue::MatrixV4(Vec::from(&slice[..])));

        self.view_matrix_ubo = Some(Share::new(view_matrix_ubo));
        self.project_matrix_ubo = Some(Share::new(project_matrix_ubo));
    }
}

fn handler_modify_oct(
	id: usize,
	octree: &SingleCaseImpl<Oct>,
	render_begin: &SingleCaseImpl<RenderBegin>,
	dirty_view_rect: &mut SingleCaseImpl<DirtyViewRect>
) {
	let oct = match octree.get(id){
		Some(r) => r,
		None => return,
	};
	let oct = &oct.0;
	let viewport = render_begin.0.viewport;
	// println!("true2======================dirty_view_rect: {:?}", **dirty_view_rect);
	// 与包围盒求并
	dirty_view_rect.0 = dirty_view_rect.0.min(oct.min.x.max(0.0));
	dirty_view_rect.1 = dirty_view_rect.1.min(oct.min.y.max(0.0));
	dirty_view_rect.2 = dirty_view_rect.2.max(oct.max.x.min(viewport.2 as f32));
	dirty_view_rect.3 = dirty_view_rect.3.max(oct.max.y.min(viewport.3 as f32));
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
	if dirty_view_rect.0 != 0.0 || 
	dirty_view_rect.1 != 0.0 || 
	dirty_view_rect.2 != viewport.2 as f32 || 
	dirty_view_rect.3 != viewport.3  as f32 {

		// println!("true2======================dirty_view_rect: {:?}", **dirty_view_rect);
		// 与包围盒求并
		dirty_view_rect.0 = 0.0;
		dirty_view_rect.1 = 0.0;
		dirty_view_rect.2 = viewport.2 as f32;
		dirty_view_rect.3 = viewport.3 as f32;
	// println!("true3======================dirty_view_rect: {:?}, oct: {:?}", **dirty_view_rect, oct);

		// println!("true1======================oct: {:?}, dirty_view_rect:{:?}", oct, **dirty_view_rect);
		dirty_view_rect.4 = true;
	}
	
	
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, ProjectionMatrix, ModifyEvent>
    for NodeAttrSys<C>
{
    type ReadData = &'a SingleCaseImpl<ProjectionMatrix>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(
        &mut self,
        _event: &ModifyEvent,
        projection_matrix: Self::ReadData,
        render_objs: Self::WriteData,
    ) {
        let slice: &[f32; 16] = projection_matrix.0.as_ref();
        let project_matrix_ubo =
            ProjectMatrixUbo::new(UniformValue::MatrixV4(Vec::from(&slice[..])));
        self.project_matrix_ubo = Some(Share::new(project_matrix_ubo));
        for r in render_objs.iter_mut() {
            r.1.paramter
                .set_value("projectMatrix", self.project_matrix_ubo.clone().unwrap());
        }
        if render_objs.len() > 0 {
            render_objs.get_notify_ref().modify_event(1, "ubo", 0);
        }
    }
}

impl<'a, C: HalContext + 'static> EntityListener<'a, Node, CreateEvent> for NodeAttrSys<C> {
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<NodeRenderMap>;
    fn listen(
        &mut self,
        event: &CreateEvent,
        _read: Self::ReadData,
        node_render_map: Self::WriteData,
    ) {
        node_render_map.create(event.id);
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, TransformWillChangeMatrix, ModifyEvent> for NodeAttrSys<C>
{
    type ReadData = &'a SingleCaseImpl<RenderBegin>;
    type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
        self.transform_will_change_matrix_dirtys.push(event.id);
		set_max_view(read, write);
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, TransformWillChangeMatrix, CreateEvent> for NodeAttrSys<C>
{
    type ReadData = &'a SingleCaseImpl<RenderBegin>;
    type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData) {
        self.transform_will_change_matrix_dirtys.push(event.id);
		set_max_view(read, write);
    }
}

impl<'a, C: HalContext + 'static>
    MultiCaseListener<'a, Node, TransformWillChangeMatrix, DeleteEvent> for NodeAttrSys<C>
{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &DeleteEvent, _: Self::ReadData, _: Self::WriteData) {
        self.transform_will_change_matrix_dirtys.push(event.id);
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
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, HSV>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Culling>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
    );
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData) {
        let (opacitys, visibilitys, hsvs, z_depths, cullings) = read;
        let (render_objs, node_render_map) = write;
        let render_obj = &mut render_objs[event.id];
        let notify = unsafe { &*(node_render_map.get_notify_ref() as * const NotifyImpl) };
        node_render_map.add(render_obj.context, event.id, &notify);

        let paramter = &mut render_obj.paramter;

        paramter.set_value("viewMatrix", self.view_matrix_ubo.clone().unwrap()); // VIEW_MATRIX
        paramter.set_value("projectMatrix", self.project_matrix_ubo.clone().unwrap()); // PROJECT_MATRIX

        let z_depth = z_depths[render_obj.context].0;
        let opacity = opacitys[render_obj.context].0;
        paramter.set_single_uniform("alpha", UniformValue::Float1(opacity)); // alpha
        debug_println!("id: {}, alpha: {:?}", render_obj.context, opacity);

        let visibility = visibilitys[render_obj.context].0;
        let culling = cullings[render_obj.context].0;
        render_obj.visibility = visibility & !culling;

        render_obj.depth = z_depth + render_obj.depth_diff;

        let hsv = &hsvs[render_obj.context];
        if !(hsv.h == 0.0 && hsv.s == 0.0 && hsv.v == 0.0) {
            render_obj.fs_defines.add("HSV");
            paramter.set_value("hsvValue", self.create_hsv_ubo(hsv)); // hsv
        }
    }
}

// 删除索引
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, DeleteEvent>
    for NodeAttrSys<C>
{
    type ReadData = &'a SingleCaseImpl<RenderObjs>;
    type WriteData = &'a mut SingleCaseImpl<NodeRenderMap>;
    fn listen(
        &mut self,
        event: &DeleteEvent,
        read: Self::ReadData,
        node_render_map: Self::WriteData,
    ) {
        let render_obj = &read[event.id];
        let notify = unsafe { &*(node_render_map.get_notify_ref() as * const NotifyImpl) };
        node_render_map.remove(render_obj.context, event.id, &notify);
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
    fn listen(&mut self, event: &ModifyEvent, z_depths: Self::ReadData, write: Self::WriteData) {
        let (render_objs, node_render_map) = write;
        let obj_ids = &node_render_map[event.id];
        let z_depth = z_depths[event.id].0;

		if obj_ids.len() > 0 {
			for id in obj_ids.iter() {
				let render_obj = &mut render_objs[*id];
				render_obj.depth = z_depth + render_obj.depth_diff;
				render_objs.get_notify_ref().modify_event(*id, "depth", 0);
			}
		}
    }
}

//不透明度变化， 设置ubo
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Opacity, ModifyEvent>
    for NodeAttrSys<C>
{
    type ReadData = &'a MultiCaseImpl<Node, Opacity>;
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
    );
    fn listen(&mut self, event: &ModifyEvent, opacitys: Self::ReadData, write: Self::WriteData) {
        let opacity = opacitys[event.id].0;
        let (render_objs, node_render_map) = write;
        let obj_ids = &node_render_map[event.id];

        for id in obj_ids.iter() {
            let render_obj = &mut render_objs[*id];
            render_obj
                .paramter
                .as_ref()
				.set_single_uniform("alpha", UniformValue::Float1(opacity));
            render_objs.get_notify_ref().modify_event(*id, "paramter", 0);
        }
    }
}
// type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
// 	type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;

// 设置visibility
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Visibility, ModifyEvent>
    for NodeAttrSys<C>
{
    type ReadData = (
        &'a MultiCaseImpl<Node, Visibility>, 
        &'a MultiCaseImpl<Node, Culling>,
		&'a SingleCaseImpl<Oct>,
		&'a SingleCaseImpl<RenderBegin>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
		&'a mut SingleCaseImpl<DirtyViewRect>
    );
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
		
        modify_visible(event.id, (read.0, read.1), (write.0, write.1));
		// dirty_view_rect已经是最大范围了，不需要再修改
		if (write.2).4 == true {
			return;
		}

		handler_modify_oct(event.id, read.2, read.3, write.2);
    }
}

// 设置visibility
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Visibility, CreateEvent>
    for NodeAttrSys<C>
{
    type ReadData = (
        &'a MultiCaseImpl<Node, Visibility>, 
        &'a MultiCaseImpl<Node, Culling>,
		&'a SingleCaseImpl<Oct>,
		&'a SingleCaseImpl<RenderBegin>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
		&'a mut SingleCaseImpl<DirtyViewRect>
    );
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData) {
        modify_visible(event.id, (read.0, read.1), (write.0, write.1));;
		// dirty_view_rect已经是最大范围了，不需要再修改
		if (write.2).4 == true {
			return;
		}

		handler_modify_oct(event.id, read.2, read.3, write.2);
    }
}

// 设置culling
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Culling, ModifyEvent>
    for NodeAttrSys<C>
{
    type ReadData = (
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, Culling>,
    );
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
    );
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData) {
        modify_visible(event.id, read, write);
    }
}

// 设置hsv
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, HSV, ModifyEvent> for NodeAttrSys<C> {
    type ReadData = &'a MultiCaseImpl<Node, HSV>;
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
    );
    fn listen(&mut self, event: &ModifyEvent, hsvs: Self::ReadData, write: Self::WriteData) {
        self.modifyHsv(event.id, hsvs, write);
    }
}

// 设置hsv
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, HSV, CreateEvent> for NodeAttrSys<C> {
    type ReadData = &'a MultiCaseImpl<Node, HSV>;
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<NodeRenderMap>,
    );
    fn listen(&mut self, event: &CreateEvent, hsvs: Self::ReadData, write: Self::WriteData) {
        self.modifyHsv(event.id, hsvs, write);
    }
}

// 包围盒修改，
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, Oct, DeleteEvent>
    for NodeAttrSys<C>
{
	type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
	type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
	fn listen(
        &mut self,
        event: &DeleteEvent,
        read: Self::ReadData,
        dirty_view_rect: Self::WriteData,
    ) {
		// dirty_view_rect已经是最大范围了，不需要再修改
		if dirty_view_rect.4 == true {
			return;
		}

		handler_modify_oct(event.id, read.0, read.1, dirty_view_rect);
    }
}

// 包围盒修改，
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, Oct, ModifyEvent>
    for NodeAttrSys<C>
{
	type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
	type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
	fn listen(
        &mut self,
        event: &ModifyEvent,
        read: Self::ReadData,
        dirty_view_rect: Self::WriteData,
    ) {
		// dirty_view_rect已经是最大范围了，不需要再修改
		if dirty_view_rect.4 == true {
			return;
		}

		handler_modify_oct(event.id, read.0, read.1, dirty_view_rect);
    }
}

// 包围盒修改，
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, Oct, CreateEvent>
    for NodeAttrSys<C>
{
	type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
	type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
	fn listen(
        &mut self,
        event: &CreateEvent,
        read: Self::ReadData,
        dirty_view_rect: Self::WriteData,
    ) {
		// dirty_view_rect已经是最大范围了，不需要再修改
		if dirty_view_rect.4 == true {
			return;
		}

		handler_modify_oct(event.id, read.0, read.1, dirty_view_rect);
    }
}

type ReadData<'a> = (
    &'a MultiCaseImpl<Node, Visibility>,
    &'a MultiCaseImpl<Node, Culling>,
);
type WriteData<'a> = (
    &'a mut SingleCaseImpl<RenderObjs>,
    &'a mut SingleCaseImpl<NodeRenderMap>,
);

fn modify_visible(id: usize, read: ReadData, write: WriteData) {
    let (visibilitys, cullings) = read;
    let (render_objs, node_render_map) = write;
    let visibility = visibilitys[id].0;
    let culling = cullings[id].0;
    let obj_ids = &node_render_map[id];

    for id in obj_ids.iter() {
		let notify = unsafe { &*(render_objs.get_notify_ref() as * const NotifyImpl) };
        let mut render_obj = RenderObjs::get_write(&mut *render_objs, *id, &notify);
        render_obj.set_visibility(visibility & !culling);
    }
}

impl_system! {
    NodeAttrSys<C> where [C: HalContext + 'static],
    true,
    {
        EntityListener<Node, CreateEvent>
        SingleCaseListener<RenderObjs, CreateEvent>
		SingleCaseListener<RenderObjs, DeleteEvent>
		SingleCaseListener<ProjectionMatrix, ModifyEvent>
		SingleCaseListener<Oct, DeleteEvent>
		SingleCaseListener<Oct, CreateEvent>
		SingleCaseListener<Oct, ModifyEvent>
		MultiCaseListener<Node, Opacity, ModifyEvent>
		MultiCaseListener<Node, Visibility, ModifyEvent>
		MultiCaseListener<Node, Visibility, CreateEvent>
        MultiCaseListener<Node, Culling, ModifyEvent>
		MultiCaseListener<Node, HSV, ModifyEvent>
		MultiCaseListener<Node, HSV, CreateEvent>
        MultiCaseListener<Node, ZDepth, ModifyEvent>
        MultiCaseListener<Node, TransformWillChangeMatrix, CreateEvent>
        MultiCaseListener<Node, TransformWillChangeMatrix, ModifyEvent>
        MultiCaseListener<Node, TransformWillChangeMatrix, DeleteEvent>
    }
}
